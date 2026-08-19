//! Het persoonlijke dossier van de functionaris.
//!
//! Een eigen sessie met een eigen wachtwoordzin, en een eigen, korte lijst
//! commando's. Het venster van de organisatie kan deze niet aanroepen en dit
//! venster de andere niet: dat staat in de rechten per vensterlabel en het
//! staat in de bouw van de schil.

use chrono::Utc;
use dpofg_domain::fg::{spiegelstand, Advies, Onafhankelijkheidsincident, Spiegelstand};
use dpofg_store::{spiegelhash, Kluis, Spiegelregel, SPIEGELSOORT};
use serde::Serialize;
use tauri::{Manager, State};

use crate::sessie::Sessie;

#[derive(Debug, Clone, Serialize)]
pub struct Adviesregel {
    pub kenmerk: String,
    pub onderwerp: String,
    pub uitgebracht_aan: String,
    pub uitgebracht_op: chrono::DateTime<Utc>,
    pub tijdig_betrokken: String,
    pub reactie: Option<String>,
    pub escalatiestappen: usize,
    pub spiegelstand: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Onafhankelijkheidsregel {
    pub kenmerk: String,
    pub soort: String,
    pub grondslag: String,
    pub datum: chrono::DateTime<Utc>,
    pub van: String,
    pub opvolging: Option<String>,
    pub spiegelstand: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Persoonlijkdossier {
    pub pad: String,
    pub ontgrendeld: bool,
    pub adviezen: Vec<Adviesregel>,
    pub gebeurtenissen: Vec<Onafhankelijkheidsregel>,
}

/// De sessie van het persoonlijke dossier.
///
/// Een eigen type en niet dezelfde `Sessie` als die van de organisatie. Twee
/// verschillende typen betekent dat de commando's van het ene venster het
/// andere niet kunnen bereiken, ook niet door een vergissing bij het
/// vastleggen van de rechten.
#[derive(Default)]
pub struct Fgsessie(pub Sessie);

fn standaardpad() -> Result<std::path::PathBuf, String> {
    crate::brug::gegevensmap().map(|m| m.join("fg-persoonlijk.dpofg"))
}

fn fout(e: impl std::fmt::Display) -> String {
    e.to_string()
}

fn stand_van(kluis: &Kluis, pad: &str) -> Result<Persoonlijkdossier, String> {
    let mut adviezen = Vec::new();
    for k in kluis.lijst("advies").map_err(fout)? {
        let a: Advies = kluis.laad("advies", &k.id).map_err(fout)?;
        let hash = spiegelhash("advies", &a.spiegelbaar()).map_err(fout)?;
        adviezen.push(Adviesregel {
            kenmerk: a.kenmerk.clone(),
            onderwerp: a.onderwerp.clone(),
            uitgebracht_aan: a.uitgebracht_aan.clone(),
            uitgebracht_op: a.uitgebracht_op,
            tijdig_betrokken: a.tijdig_betrokken.omschrijving().to_string(),
            reactie: a.bestuursreactie.as_ref().map(|r| r.status.omschrijving().to_string()),
            escalatiestappen: a.escalatie.len(),
            spiegelstand: naam_van(spiegelstand(&a.spiegelingen, &hash)),
        });
    }

    let mut gebeurtenissen = Vec::new();
    for k in kluis.lijst("onafhankelijkheidsincident").map_err(fout)? {
        let i: Onafhankelijkheidsincident =
            kluis.laad("onafhankelijkheidsincident", &k.id).map_err(fout)?;
        let hash = spiegelhash("onafhankelijkheidsincident", &i.spiegelbaar()).map_err(fout)?;
        gebeurtenissen.push(Onafhankelijkheidsregel {
            kenmerk: i.kenmerk.clone(),
            soort: i.soort.omschrijving().to_string(),
            grondslag: i.soort.grondslag().to_string(),
            datum: i.datum,
            van: i.van.clone(),
            opvolging: i.opvolging.clone(),
            spiegelstand: naam_van(spiegelstand(&i.spiegelingen, &hash)),
        });
    }

    Ok(Persoonlijkdossier { pad: pad.to_string(), ontgrendeld: true, adviezen, gebeurtenissen })
}

fn naam_van(stand: Spiegelstand) -> String {
    match stand {
        Spiegelstand::NooitGespiegeld => "nooit_gespiegeld",
        Spiegelstand::Sluitend { .. } => "sluitend",
        Spiegelstand::Gewijzigd { .. } => "gewijzigd",
    }
    .to_string()
}

#[tauri::command]
pub fn fg_ontgrendel(
    wachtwoord: String,
    sessie: State<'_, Fgsessie>,
) -> Result<Persoonlijkdossier, String> {
    let pad = standaardpad()?;
    if !pad.exists() {
        return Err(format!(
            "er staat geen persoonlijk dossier op {}. Maak er een aan met 'dpofg fg nieuw'",
            pad.display()
        ));
    }
    let zin = dpofg_crypto::Wachtwoordzin::nieuw(wachtwoord);
    let nu = Utc::now();
    let mut kluis = Kluis::openen(&pad, &zin, nu).map_err(fout)?;
    let namen: Vec<String> = kluis.compartimenten().iter().map(|s| s.to_string()).collect();
    for naam in namen {
        kluis.compartiment_ontgrendelen(&naam).map_err(fout)?;
    }
    let dossier = stand_van(&kluis, &pad.display().to_string())?;
    sessie.0.zet(kluis)?;
    Ok(dossier)
}

#[tauri::command]
pub fn fg_vergrendel(sessie: State<'_, Fgsessie>) -> Result<(), String> {
    sessie.0.sluit()
}

/// Legt de hash van een record vast in de kluis van de organisatie.
///
/// Dit is het enige commando dat beide kluizen aanraakt, en het doet dat in
/// één richting: er gaat een hash naar de organisatiekluis en er komt niets
/// terug. De organisatiekluis moet daarvoor openstaan; is dat niet zo, dan
/// zegt het commando dat in plaats van erom te vragen — het persoonlijke
/// venster hoort niet naar de zin van de organisatie te vragen.
#[tauri::command]
pub fn fg_spiegel(
    kenmerk: String,
    fg: State<'_, Fgsessie>,
    organisatie: State<'_, Sessie>,
) -> Result<Persoonlijkdossier, String> {
    let nu = Utc::now();
    if !organisatie.is_open() {
        return Err(
            "de kluis van de organisatie staat niet open; open haar in het andere venster, \
             want daar hoort de hash te landen"
                .into(),
        );
    }

    let (soort, hash, id, status) = fg.0.met_kluis(|kluis| bepaal(kluis, &kenmerk))?;

    organisatie.met_kluis(|kluis| {
        if kluis.lijst(SPIEGELSOORT).map_err(fout)?.iter().any(|k| k.id == hash) {
            return Err(
                "deze hash staat al in de kluis van de organisatie; het record is sinds het \
                 spiegelen niet gewijzigd"
                    .into(),
            );
        }
        let regel = Spiegelregel { hash: hash.clone(), soort: soort.clone(), vastgelegd_op: nu };
        kluis
            .bewaar(
                SPIEGELSOORT,
                &hash,
                "algemeen",
                "vastgesteld",
                None,
                &regel,
                &actor(),
                dpofg_audit::Handeling::RecordAangemaakt,
                &format!("hash van een {soort} uit het persoonlijke dossier vastgelegd"),
                nu,
            )
            .map_err(fout)?;
        Ok(())
    })?;

    // En de spiegeling in het eigen dossier noteren, zodat later te zien is
    // dát er is gespiegeld en niet alleen dat het niet meer klopt.
    fg.0.met_kluis(|kluis| noteer(kluis, &kenmerk, &soort, &hash, &id, &status, nu))?;
    fg.0.met_kluis(|kluis| stand_van(kluis, &standaardpad()?.display().to_string()))
}

fn bepaal(kluis: &mut Kluis, kenmerk: &str) -> Result<(String, String, String, String), String> {
    if let Some(k) = kluis
        .lijst("advies")
        .map_err(fout)?
        .into_iter()
        .find(|r| r.kenmerk.as_deref() == Some(kenmerk))
    {
        let a: Advies = kluis.laad("advies", &k.id).map_err(fout)?;
        let hash = spiegelhash("advies", &a.spiegelbaar()).map_err(fout)?;
        return Ok(("advies".into(), hash, k.id, k.status));
    }
    let k = kluis
        .lijst("onafhankelijkheidsincident")
        .map_err(fout)?
        .into_iter()
        .find(|r| r.kenmerk.as_deref() == Some(kenmerk))
        .ok_or_else(|| format!("geen advies en geen gebeurtenis met kenmerk '{kenmerk}'"))?;
    let i: Onafhankelijkheidsincident =
        kluis.laad("onafhankelijkheidsincident", &k.id).map_err(fout)?;
    let hash = spiegelhash("onafhankelijkheidsincident", &i.spiegelbaar()).map_err(fout)?;
    Ok(("onafhankelijkheidsincident".into(), hash, k.id, k.status))
}

fn noteer(
    kluis: &mut Kluis,
    kenmerk: &str,
    soort: &str,
    hash: &str,
    id: &str,
    status: &str,
    nu: chrono::DateTime<Utc>,
) -> Result<(), String> {
    let spiegeling = dpofg_domain::fg::Spiegeling { hash: hash.to_string(), op: nu };
    if soort == "advies" {
        let mut a: Advies = kluis.laad("advies", id).map_err(fout)?;
        a.spiegelingen.push(spiegeling);
        kluis
            .bewaar(
                "advies",
                id,
                "fg-persoonlijk",
                status,
                Some(kenmerk),
                &a,
                &actor(),
                dpofg_audit::Handeling::RecordGewijzigd,
                "gespiegeld",
                nu,
            )
            .map_err(fout)?;
    } else {
        let mut i: Onafhankelijkheidsincident =
            kluis.laad("onafhankelijkheidsincident", id).map_err(fout)?;
        i.spiegelingen.push(spiegeling);
        kluis
            .bewaar(
                "onafhankelijkheidsincident",
                id,
                "fg-persoonlijk",
                status,
                Some(kenmerk),
                &i,
                &actor(),
                dpofg_audit::Handeling::RecordGewijzigd,
                "gespiegeld",
                nu,
            )
            .map_err(fout)?;
    }
    Ok(())
}

fn actor() -> dpofg_audit::Actor {
    let naam = std::env::var("USER")
        .or_else(|_| std::env::var("USERNAME"))
        .unwrap_or_else(|_| "onbekend".into());
    dpofg_audit::Actor::nieuw(naam.clone(), naam, "gebruiker")
}

/// Opent het venster van het persoonlijke dossier.
///
/// Vanuit Rust en niet vanuit de webview: dan hoeft het venster van de
/// organisatie geen recht te krijgen om vensters te maken.
#[tauri::command]
pub fn toon_persoonlijk_venster(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(venster) = app.get_webview_window("fg-persoonlijk") {
        venster.show().map_err(fout)?;
        venster.set_focus().map_err(fout)?;
        return Ok(());
    }
    tauri::WebviewWindowBuilder::new(
        &app,
        "fg-persoonlijk",
        tauri::WebviewUrl::App("fg.html".into()),
    )
    .title("Persoonlijk dossier — dpo-fg-tool")
    .inner_size(1000.0, 780.0)
    .min_inner_size(720.0, 520.0)
    .build()
    .map_err(fout)?;
    Ok(())
}
