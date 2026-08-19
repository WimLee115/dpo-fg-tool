//! De werkbak: één lijst met wat er openstaat, over alle regimes heen.

use anyhow::Result;
use chrono::{DateTime, Utc};
use clap::Args;
use dpofg_domain::{correctie::Correctie, verzoek::Betrokkenenverzoek, woo::Wooverzoek, Incident};
use dpofg_report::werkbak::{
    werkbak, Band, Bronnen, Kalendercontext, Termijnbron, Termijnkenmerk, Werkbakregel,
    NIET_IN_DE_LIJST,
};
use dpofg_store::Kluis;
use std::path::PathBuf;

use crate::uitvoer::{blokkade, gelukt, kop, let_op, tabel, terzijde};

#[derive(Args, Debug)]
pub struct Werkbakargumenten {
    /// Toon alleen wat binnen dit aantal dagen speelt.
    #[arg(long, value_parser = clap::value_parser!(i64).range(1..=3650))]
    pub tot: Option<i64>,
    /// Toon alleen regels voor deze dossiersoort. Herhaalbaar.
    #[arg(long = "soort")]
    pub soorten: Vec<String>,
    /// Lever de lijst als JSON, voor een schil of een geplande taak.
    #[arg(long)]
    pub json: bool,
}

/// De termijnen uit het kennispakket.
///
/// Welke termijn onherstelbaar is, staat niet in het pakket en ook niet in de
/// wet: het is een uitspraak over de aard van de termijn. Een meldtermijn is
/// niet in te halen — te laat is te laat, en de toezichthouder ziet het aan de
/// datum. Een herzieningstermijn wel. Die indeling staat hier, zichtbaar en op
/// één plaats, in plaats van verspreid over de regels.
struct Pakkettermijnen {
    pakket: dpofg_content::Pakketinhoud,
}

impl Termijnbron for Pakkettermijnen {
    fn duur(&self, code: &str) -> Option<Termijnkenmerk> {
        let t = self.pakket.termijn(code).ok()?;
        let uren = match t.eenheid {
            dpofg_terms::Eenheid::Klokuren => i64::from(t.duur),
            dpofg_terms::Eenheid::Kalenderdagen | dpofg_terms::Eenheid::Werkdagen => {
                i64::from(t.duur) * 24
            }
            dpofg_terms::Eenheid::Weken => i64::from(t.duur) * 24 * 7,
            dpofg_terms::Eenheid::Maanden => i64::from(t.duur) * 24 * 30,
            dpofg_terms::Eenheid::Jaren => i64::from(t.duur) * 24 * 365,
        };
        Some(Termijnkenmerk {
            uren,
            omschrijving: t.naam.clone(),
            grondslag: t.grondslag.clone(),
            onherstelbaar: onherstelbaar(code),
        })
    }
}

/// Of een gemiste termijn onherstelbaar is.
///
/// Vier meldtermijnen naar buiten zijn dat: wie te laat meldt, kan dat niet
/// alsnog op tijd doen, en het verschil staat in de melding zelf. Al het
/// andere is in te halen.
fn onherstelbaar(code: &str) -> bool {
    matches!(
        code,
        "AVG-33-MELDING"
            | "AVG-34-MEDEDELING"
            | "ZORG-WAARSCHUWING"
            | "ZORG-MELDING"
            | "ZORG-EINDRAPPORT"
            | "AVG-12-3-VERZOEK"
            | "WOO-BESLISTERMIJN"
    )
}

pub fn draai(o: Werkbakargumenten, kluispad: Option<PathBuf>, nu: DateTime<Utc>) -> Result<()> {
    let pad = super::kluispad(kluispad)?;
    let kluis = super::open_kluis(&pad, nu)?;
    let pakket = dpofg_content::startpakket(nu.date_naive());

    let incidenten: Vec<Incident> = laad(&kluis, "incident")?;
    let verzoeken: Vec<Betrokkenenverzoek> = laad(&kluis, "verzoek")?;
    let wooverzoeken: Vec<Wooverzoek> = laad(&kluis, "woo")?;
    let correcties: Vec<Correctie> = laad(&kluis, "correctie")?;

    let bronnen = Bronnen {
        incidenten: &incidenten,
        verzoeken: &verzoeken,
        wooverzoeken: &wooverzoeken,
        correcties: &correcties,
    };
    let kalender = pakket.kalender("NL").map_err(|e| {
        anyhow::anyhow!("het kennispakket bevat geen feestdagenkalender voor NL: {e}")
    })?;
    let context = Kalendercontext { zone: chrono_tz::Europe::Amsterdam, kalender };
    let termijnen = Pakkettermijnen { pakket: pakket.clone() };

    let mut regels = werkbak(&bronnen, &termijnen, &context, nu);
    let totaal = regels.len();
    if !o.soorten.is_empty() {
        regels.retain(|r| o.soorten.contains(&r.record_soort));
    }
    if let Some(dagen) = o.tot {
        let grens = nu + chrono::Duration::days(dagen);
        regels.retain(|r| r.deadline.is_none_or(|d| d <= grens));
    }
    let weggefilterd = totaal - regels.len();

    if o.json {
        println!("{}", serde_json::to_string_pretty(&regels)?);
        return Ok(());
    }

    kop("Werkbak");
    terzijde(&format!(
        "{} openstaande verplichting(en) op {}",
        regels.len(),
        nu.format("%d-%m-%Y %H:%M UTC")
    ));

    if regels.is_empty() {
        gelukt("er staat niets open in deze lijst");
    } else {
        toon_banden(&regels, nu);
    }

    if weggefilterd > 0 {
        let_op(&format!(
            "{weggefilterd} regel(s) vallen buiten de gekozen filter en staan hier niet"
        ));
    }

    // Wat er buiten valt, staat er altijd bij. Een lege lijst die als "klaar"
    // wordt gelezen is de duurste fout die een werkvoorraad kan maken.
    kop("Niet in deze lijst");
    for (wat, waar) in NIET_IN_DE_LIJST {
        terzijde(&format!("{wat} — {waar}"));
    }
    Ok(())
}

fn toon_banden(regels: &[Werkbakregel], nu: DateTime<Utc>) {
    let mut vorige: Option<Band> = None;
    let mut t = tabel(&["", "dossier", "wat er moet", "grondslag", "wanneer", "eigenaar"]);
    for r in regels {
        if vorige != Some(r.band) {
            if vorige.is_some() {
                println!("{t}");
                t = tabel(&["", "dossier", "wat er moet", "grondslag", "wanneer", "eigenaar"]);
            }
            kop(r.band.omschrijving());
            vorige = Some(r.band);
        }
        let wanneer = match (r.deadline, r.uren_tot_deadline(nu)) {
            (Some(d), Some(u)) if u.abs() < 48 => {
                format!("{} ({u} uur)", d.format("%d-%m-%Y %H:%M"))
            }
            (Some(d), _) => format!(
                "{} ({} dagen)",
                d.format("%d-%m-%Y"),
                r.dagen_tot_deadline(nu).unwrap_or_default()
            ),
            _ => "geen anker".to_string(),
        };
        t.add_row(vec![
            r.spoor.map(|s| format!("{}/{}", s.nummer, s.totaal)).unwrap_or_default(),
            format!("{} {}", r.record_soort, r.record_kenmerk),
            r.wat.clone(),
            r.grondslag.clone(),
            wanneer,
            r.eigenaar.clone().unwrap_or_else(|| "geen".into()),
        ]);
    }
    println!("{t}");

    let onherstelbaar = regels.iter().filter(|r| r.onherstelbaar).count();
    if onherstelbaar > 0 {
        blokkade(&format!(
            "{onherstelbaar} van deze verplichtingen {} onherstelbaar: te laat is te laat, en \
             dat staat in de melding zelf",
            if onherstelbaar == 1 { "is" } else { "zijn" }
        ));
    }
    let zonder_anker = regels.iter().filter(|r| r.deadline.is_none()).count();
    if zonder_anker > 0 {
        terzijde(&format!(
            "{zonder_anker} verplichting(en) wachten op een anker; hun klok loopt nog niet"
        ));
    }
    terzijde(
        "De volgorde ligt vast en is niet om te draaien: onherstelbaar gaat vóór herstelbaar, \
         en verstreken vóór aanstaand.",
    );
}

fn laad<T: serde::de::DeserializeOwned>(kluis: &Kluis, soort: &str) -> Result<Vec<T>> {
    let mut uit = Vec::new();
    for k in kluis.lijst(soort)? {
        uit.push(kluis.laad(soort, &k.id)?);
    }
    Ok(uit)
}
