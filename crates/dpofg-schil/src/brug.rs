//! De commando's die de schil kan aanroepen.
//!
//! Een eindige lijst met vaste namen. Elk commando leest wat het nodig heeft
//! uit de kluis en geeft terug wat er in beeld komt; er wordt hier niets
//! bewaard tussen twee aanroepen door.

use chrono::Utc;
use dpofg_domain::{
    correctie::Correctie, verzoek::Betrokkenenverzoek, woo::Wooverzoek, Incident, Volledig,
};
use dpofg_report::werkbak::{self, Bronnen, Kalendercontext, Termijnbron, Termijnkenmerk};
use dpofg_store::Kluis;
use tauri::State;

use crate::{sessie::Sessie, vorm};

/// De map waarin dit product zijn gegevens bewaart.
pub fn gegevensmap() -> Result<std::path::PathBuf, String> {
    let basis = if cfg!(target_os = "windows") {
        std::env::var_os("APPDATA").map(std::path::PathBuf::from)
    } else if cfg!(target_os = "macos") {
        std::env::var_os("HOME")
            .map(std::path::PathBuf::from)
            .map(|t| t.join("Library").join("Application Support"))
    } else {
        std::env::var_os("XDG_DATA_HOME").map(std::path::PathBuf::from).or_else(|| {
            std::env::var_os("HOME")
                .map(|t| std::path::PathBuf::from(t).join(".local").join("share"))
        })
    };
    basis
        .map(|b| b.join("dpo-fg-tool"))
        .ok_or_else(|| "de standaardlocatie voor gegevens is op dit systeem niet te bepalen".into())
}

/// Waar de kluis van de organisatie staat.
fn standaardpad() -> Result<std::path::PathBuf, String> {
    gegevensmap().map(|m| m.join("dossier.dpofg"))
}

fn fout(e: impl std::fmt::Display) -> String {
    e.to_string()
}

fn stand_van(kluis: &mut Kluis, pad: &str) -> Result<vorm::Kluisstand, String> {
    let nu = Utc::now();
    let pakket = dpofg_content::startpakket(nu.date_naive());
    let rapport = kluis.verifieer_logboek().map_err(fout)?;
    Ok(vorm::Kluisstand {
        pad: pad.to_string(),
        ontgrendeld: true,
        kennispakket: format!("{} {}", pakket.code, pakket.versienaam),
        consolidatiedatum: pakket.consolidatiedatum.to_string(),
        ketenreikwijdte: rapport.reikwijdte(),
        keten_in_orde: rapport.bevindingen.is_empty(),
    })
}

#[tauri::command]
pub fn ontgrendel(
    pad: Option<String>,
    wachtwoord: String,
    sessie: State<'_, Sessie>,
) -> Result<vorm::Kluisstand, String> {
    let pad = match pad {
        Some(p) if !p.trim().is_empty() => std::path::PathBuf::from(p),
        _ => standaardpad()?,
    };
    if !pad.exists() {
        return Err(format!(
            "er staat geen kluis op {}. Maak er een aan met 'dpofg kluis nieuw'",
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
    let stand = stand_van(&mut kluis, &pad.display().to_string())?;
    sessie.zet(kluis)?;
    Ok(stand)
}

#[tauri::command]
pub fn vergrendel(sessie: State<'_, Sessie>) -> Result<(), String> {
    sessie.sluit()
}

#[tauri::command]
pub fn stand(sessie: State<'_, Sessie>) -> Result<vorm::Kluisstand, String> {
    if !sessie.is_open() {
        let pad = standaardpad()?;
        let pakket = dpofg_content::startpakket(Utc::now().date_naive());
        return Ok(vorm::Kluisstand {
            pad: pad.display().to_string(),
            ontgrendeld: false,
            kennispakket: format!("{} {}", pakket.code, pakket.versienaam),
            consolidatiedatum: pakket.consolidatiedatum.to_string(),
            ketenreikwijdte: String::new(),
            keten_in_orde: true,
        });
    }
    let pad = standaardpad()?.display().to_string();
    sessie.met_kluis(|kluis| stand_van(kluis, &pad))
}

/// De termijnen uit het kennispakket, voor de werkbak.
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
            onherstelbaar: matches!(
                code,
                "AVG-33-MELDING"
                    | "AVG-34-MEDEDELING"
                    | "ZORG-WAARSCHUWING"
                    | "ZORG-MELDING"
                    | "ZORG-EINDRAPPORT"
                    | "AVG-12-3-VERZOEK"
                    | "WOO-BESLISTERMIJN"
            ),
        })
    }
}

fn laad<T: serde::de::DeserializeOwned>(kluis: &Kluis, soort: &str) -> Result<Vec<T>, String> {
    let mut uit = Vec::new();
    for k in kluis.lijst(soort).map_err(fout)? {
        uit.push(kluis.laad(soort, &k.id).map_err(fout)?);
    }
    Ok(uit)
}

#[tauri::command]
pub fn werkbak(sessie: State<'_, Sessie>) -> Result<Vec<werkbak::Werkbakregel>, String> {
    sessie.met_kluis(|kluis| {
        let nu = Utc::now();
        let pakket = dpofg_content::startpakket(nu.date_naive());
        let incidenten: Vec<Incident> = laad(kluis, "incident")?;
        let verzoeken: Vec<Betrokkenenverzoek> = laad(kluis, "verzoek")?;
        let wooverzoeken: Vec<Wooverzoek> = laad(kluis, "woo")?;
        let correcties: Vec<Correctie> = laad(kluis, "correctie")?;

        let bronnen = Bronnen {
            incidenten: &incidenten,
            verzoeken: &verzoeken,
            wooverzoeken: &wooverzoeken,
            correcties: &correcties,
        };
        let kalender = pakket.kalender("NL").map_err(fout)?;
        let context = Kalendercontext { zone: chrono_tz::Europe::Amsterdam, kalender };
        let termijnen = Pakkettermijnen { pakket: pakket.clone() };
        Ok(werkbak::werkbak(&bronnen, &termijnen, &context, nu))
    })
}

#[tauri::command]
pub fn buitenbeeld() -> Vec<vorm::Buitenbeeld> {
    werkbak::NIET_IN_DE_LIJST
        .iter()
        .map(|(wat, waar)| vorm::Buitenbeeld { wat: (*wat).to_string(), waar: (*waar).to_string() })
        .collect()
}

/// Zet een record om naar wat er in beeld komt.
///
/// De velden komen uit de JSON-vorm van het record. Dat is met opzet generiek:
/// een schil die per dossiersoort een eigen omzetting kent, loopt achter zodra
/// er een veld bijkomt, en dan toont zij minder dan er staat.
fn velden_uit(waarde: &serde_json::Value) -> Vec<vorm::Veld> {
    let Some(object) = waarde.as_object() else {
        return Vec::new();
    };
    object
        .iter()
        .filter(|(naam, _)| !matches!(naam.as_str(), "id" | "compartiment" | "herkomst"))
        .map(|(naam, v)| vorm::Veld {
            naam: naam.replace('_', " "),
            waarde: match v {
                serde_json::Value::String(s) => s.clone(),
                serde_json::Value::Null => "—".into(),
                andere => andere.to_string(),
            },
            herkomst: None,
        })
        .collect()
}

#[tauri::command]
pub fn dossier(
    soort: String,
    kenmerk: String,
    sessie: State<'_, Sessie>,
) -> Result<vorm::Dossier, String> {
    sessie.met_kluis(|kluis| {
        let kop = kluis
            .lijst(&soort)
            .map_err(fout)?
            .into_iter()
            .find(|r| r.kenmerk.as_deref() == Some(kenmerk.as_str()))
            .ok_or_else(|| format!("geen {soort} met kenmerk '{kenmerk}'"))?;
        let waarde: serde_json::Value = kluis.laad(&soort, &kop.id).map_err(fout)?;
        let volledigheid = volledigheid_van(&soort, &waarde)?;
        Ok(vorm::Dossier {
            kop: vorm::Recordkop {
                id: kop.id.clone(),
                soort: kop.soort.clone(),
                kenmerk: kop.kenmerk.clone(),
                status: kop.status.clone(),
                gewijzigd_op: kop.gewijzigd_op,
            },
            volledigheid,
            velden: velden_uit(&waarde),
        })
    })
}

/// De volledigheid van een record, per dossiersoort.
///
/// Hier staat wél een lijst per soort, en dat kan niet anders: het
/// `Volledig`-trait zit op de domeintypen, en die moeten dus stuk voor stuk
/// worden ingelezen. Een soort die hier ontbreekt, levert een lege teller op
/// en dat zou een dossier completer laten lijken dan het is — daarom geeft
/// deze functie in dat geval een fout in plaats van niets.
fn volledigheid_van(soort: &str, waarde: &serde_json::Value) -> Result<vorm::Volledigheid, String> {
    fn omzetten<T: Volledig>(record: T) -> vorm::Volledigheid {
        let r = record.volledigheid();
        vorm::Volledigheid {
            soort: r.soort,
            verplicht: r.verplicht,
            compleet: r.compleet,
            ontbreekt: r
                .ontbreekt
                .into_iter()
                .map(|o| vorm::Ontbrekend {
                    veld: o.veld,
                    omschrijving: o.omschrijving,
                    grondslag: o.grondslag,
                    blokkeert_vaststelling: o.blokkeert_vaststelling,
                })
                .collect(),
        }
    }
    macro_rules! probeer {
        ($type:ty) => {
            omzetten(serde_json::from_value::<$type>(waarde.clone()).map_err(fout)?)
        };
    }
    Ok(match soort {
        "verwerking" => probeer!(dpofg_domain::Verwerking),
        "incident" => probeer!(dpofg_domain::Incident),
        "dpia" => probeer!(dpofg_domain::Dpia),
        "verzoek" => probeer!(dpofg_domain::verzoek::Betrokkenenverzoek),
        "woo" => probeer!(dpofg_domain::woo::Wooverzoek),
        "leverancier" => probeer!(dpofg_domain::leverancier::Leverancier),
        "zorgplicht" => probeer!(dpofg_domain::zorgplicht::Zorgplichtdossier),
        "risico" => probeer!(dpofg_domain::risico::Risicobeoordeling),
        "correctie" => probeer!(dpofg_domain::correctie::Correctie),
        "doorgifte" => probeer!(dpofg_domain::doorgifte::Doorgifte),
        andere => {
            return Err(format!(
                "de schil kent de volledigheid van soort '{andere}' niet; zij toont liever niets \
                 dan een teller die het dossier completer laat lijken dan het is"
            ))
        }
    })
}

#[tauri::command]
pub fn controle(sessie: State<'_, Sessie>) -> Result<Vec<vorm::Bevinding>, String> {
    sessie.met_kluis(|kluis| {
        let nu = Utc::now();
        let motor = dpofg_rules::regels::standaardmotor();
        let mut uit = Vec::new();

        for k in kluis.lijst("verwerking").map_err(fout)? {
            let v: dpofg_domain::Verwerking = kluis.laad("verwerking", &k.id).map_err(fout)?;
            uit.extend(dpofg_rules::regels::beoordeel_verwerking(&motor, &v, nu));
        }
        for k in kluis.lijst("zorgplicht").map_err(fout)? {
            let d: dpofg_domain::zorgplicht::Zorgplichtdossier =
                kluis.laad("zorgplicht", &k.id).map_err(fout)?;
            let beoordelingen: Vec<dpofg_domain::risico::Risicobeoordeling> =
                laad(kluis, "risico")?;
            uit.extend(dpofg_rules::regels::beoordeel_zorgplicht(
                &motor,
                &d,
                &beoordelingen,
                dpofg_rules::regels::Zorgplichtdrempels {
                    beoordelingstermijn_dagen: 30,
                    bewijshorizon_dagen: 60,
                    frequentiedrempel_maanden: 12,
                    bestuursvaststelling_maanden: 12,
                    afwijkingsaandeel_procent: 50,
                },
                nu,
            ));
        }

        Ok(uit
            .into_iter()
            .map(|b| vorm::Bevinding {
                regelcode: b.regelcode,
                niveau: match b.niveau {
                    dpofg_rules::motor::Niveau::Blokkerend => "blokkerend",
                    dpofg_rules::motor::Niveau::Signalerend => "signalerend",
                    dpofg_rules::motor::Niveau::Rapporterend => "rapporterend",
                }
                .to_string(),
                ontvanger: b.ontvanger.omschrijving().to_string(),
                record_soort: b.record_soort,
                record_kenmerk: b.record_kenmerk,
                toelichting: b.toelichting,
                grondslag: b.grondslag,
                afwijking_tot: b.afwijking.and_then(|a| a.geldig_tot),
            })
            .collect())
    })
}

#[tauri::command]
pub fn prognose(dagen: i64, sessie: State<'_, Sessie>) -> Result<Vec<vorm::Vervalpunt>, String> {
    sessie.met_kluis(|kluis| {
        let nu = Utc::now();
        let pakket = dpofg_content::startpakket(nu.date_naive());
        let maanden = |code: &str, terugval: u32| {
            pakket
                .termijn(code)
                .ok()
                .filter(|t| t.eenheid == dpofg_terms::Eenheid::Maanden)
                .map(|t| t.duur)
                .unwrap_or(terugval)
        };
        let termijnen = dpofg_report::prognose::Prognosetermijnen {
            bestuursvaststelling_maanden: maanden("INTERN-ZORGPLICHT-BESTUURSVASTSTELLING", 12),
            subverwerkerscontrole_maanden: maanden("INTERN-SUBVERWERKERSCONTROLE", 12),
            effectbeoordeling_maanden: maanden("INTERN-DPIA-HERBEOORDELING", 36),
            wpg_audit_maanden: maanden("WPG-EXTERNE-AUDIT", 48),
            wpg_controle_maanden: maanden("WPG-INTERNE-CONTROLE", 12),
        };

        let zorgplicht: Vec<dpofg_domain::zorgplicht::Zorgplichtdossier> =
            laad(kluis, "zorgplicht")?;
        let beoordelingen: Vec<dpofg_domain::risico::Risicobeoordeling> = laad(kluis, "risico")?;
        let leveranciers: Vec<dpofg_domain::leverancier::Leverancier> = laad(kluis, "leverancier")?;
        let effectbeoordelingen: Vec<dpofg_domain::Dpia> = laad(kluis, "dpia")?;
        let wpgsporen: Vec<dpofg_domain::wpg::Wpgspoor> = laad(kluis, "wpg")?;

        let bronnen = dpofg_report::prognose::Bronnen {
            zorgplicht: &zorgplicht,
            risicobeoordelingen: &beoordelingen,
            leveranciers: &leveranciers,
            effectbeoordelingen: &effectbeoordelingen,
            wpgsporen: &wpgsporen,
        };
        let peildatum = nu + chrono::Duration::days(dagen.clamp(1, 3650));
        Ok(dpofg_report::prognose::prognose(&bronnen, termijnen, peildatum)
            .into_iter()
            .map(|v| vorm::Vervalpunt {
                eis: v.eis,
                grondslag: v.grondslag,
                oorzaak: v.oorzaak.omschrijving().to_string(),
                record_soort: v.record_soort,
                record_kenmerk: v.record_kenmerk,
                eigenaar: v.eigenaar,
                vervalt_op: v.vervalt_op,
            })
            .collect())
    })
}
