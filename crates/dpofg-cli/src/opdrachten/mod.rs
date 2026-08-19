//! De opdrachten van de bedieningsschil.

pub mod controle;
pub mod doorgifte;
pub mod dossier;
pub mod dpia;
pub mod incident;
pub mod kluis;
pub mod leverancier;
pub mod lia;
pub mod logboek;
pub mod mapping;
pub mod pakket;
pub mod redactie;
pub mod register;
pub mod termijn;
pub mod verzoek;
pub mod woo;
pub mod wpg;

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use dpofg_store::Kluis;
use std::path::{Path, PathBuf};

/// Zoekt het kluisbestand.
///
/// Volgorde: het meegegeven pad, dan de omgevingsvariabele, dan de
/// standaardlocatie van het besturingssysteem. Zo werkt de schil zonder
/// instellingen én is hij te sturen voor wie meerdere kluizen beheert.
pub fn kluispad(meegegeven: Option<PathBuf>) -> Result<PathBuf> {
    if let Some(p) = meegegeven {
        return Ok(p);
    }
    let map = standaardmap()?;
    Ok(map.join("dossier.dpofg"))
}

/// De standaardlocatie voor gegevens, per besturingssysteem.
pub fn standaardmap() -> Result<PathBuf> {
    // Bewust geen extra afhankelijkheid: de drie gevallen zijn eenvoudig en
    // expliciet leesbaar. De keuzes volgen het hoofdstuk platformondersteuning.
    let basis = if cfg!(target_os = "windows") {
        std::env::var_os("APPDATA").map(PathBuf::from).context("APPDATA is niet gezet")?
    } else if cfg!(target_os = "macos") {
        let thuis = std::env::var_os("HOME").map(PathBuf::from).context("HOME is niet gezet")?;
        thuis.join("Library").join("Application Support")
    } else {
        match std::env::var_os("XDG_DATA_HOME") {
            Some(p) => PathBuf::from(p),
            None => {
                let thuis =
                    std::env::var_os("HOME").map(PathBuf::from).context("HOME is niet gezet")?;
                thuis.join(".local").join("share")
            }
        }
    };
    Ok(basis.join("dpo-fg-tool"))
}

/// Opent de kluis en ontgrendelt de compartimenten.
pub fn open_kluis(pad: &Path, nu: DateTime<Utc>) -> Result<Kluis> {
    if !pad.exists() {
        anyhow::bail!(
            "er staat geen kluis op {}. Maak er een aan met 'dpofg kluis nieuw'",
            pad.display()
        );
    }
    let wachtwoord = crate::wachtwoord::vraag("Wachtwoordzin")?;
    let mut kluis = Kluis::openen(pad, &wachtwoord, nu)?;

    let namen: Vec<String> = kluis.compartimenten().iter().map(|s| s.to_string()).collect();
    for naam in namen {
        kluis.compartiment_ontgrendelen(&naam)?;
    }

    if kluis.parameters_verouderd() {
        crate::uitvoer::let_op(
            "De sleutelafleiding van deze kluis gebruikt parameters die onder de huidige norm \
             liggen. Wijzig het wachtwoord om te verzwaren; de gegevens hoeven daarvoor niet te \
             worden herversleuteld.",
        );
    }
    Ok(kluis)
}

/// De actor die de handeling verricht.
///
/// In deze uitgave is dat de aangemelde gebruiker van het besturingssysteem.
/// Zodra er een rollenmodel is, komt de rol hiervandaan.
pub fn actor() -> dpofg_audit::Actor {
    let naam = std::env::var("DPOFG_GEBRUIKER")
        .or_else(|_| std::env::var("USER"))
        .or_else(|_| std::env::var("USERNAME"))
        .unwrap_or_else(|_| "onbekend".into());
    dpofg_audit::Actor::nieuw(naam.clone(), naam, "gebruiker")
}
