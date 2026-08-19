//! De Rust-kant van de grafische schil.
//!
//! # Wat hier wel en niet gebeurt
//!
//! Alles wat rekent, ontsleutelt of naar de schijf schrijft, gebeurt hier. De
//! webview is een weergavelaag: er zitten geen sleutels in, geen
//! cryptografie, geen bestands-invoer en geen klokken. Daarmee vervalt een
//! hele klasse verschillen tussen de drie webviewmotoren, en beperkt een fout
//! in de interface de schade.
//!
//! # Waarom de kluis in een mutex zit en niet per aanroep opengaat
//!
//! Het openen van de kluis kost door de sleutelafleiding bewust tijd. Dat per
//! aanroep doen zou de schil onbruikbaar maken en de gebruiker aanzetten tot
//! een korte wachtwoordzin. De kluis blijft dus open in het geheugen van het
//! Rust-proces, en gaat dicht wanneer de gebruiker dat zegt.

pub mod brug;
pub mod fg;
pub mod sessie;
pub mod vorm;

use tauri::Manager;

/// Of deze binary zijn scherm bij een ontwikkelserver ophaalt.
///
/// Tauri leidt dit af uit de feature `custom-protocol`: ontbreekt die, dan
/// wijst de schil naar `devUrl` en toont een geïnstalleerd programma bij het
/// starten alleen een foutmelding van de webview. Het verschil is aan de
/// binary niet te zien, dus het is hier opvraagbaar.
pub fn is_ontwikkelbouw() -> bool {
    tauri::is_dev()
}

/// De bouwsoort in één woord.
pub fn bouwsoort() -> &'static str {
    if is_ontwikkelbouw() {
        "ontwikkelbouw"
    } else {
        "uitgave"
    }
}

/// Waar het scherm vandaan komt.
pub fn schermbron() -> &'static str {
    if is_ontwikkelbouw() {
        "de ontwikkelserver (niet geschikt om te installeren)"
    } else {
        "de ingebouwde bundel"
    }
}

/// Start de schil.
pub fn draai() {
    tauri::Builder::default()
        .setup(|app| {
            app.manage(sessie::Sessie::nieuw());
            app.manage(fg::Fgsessie::default());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            brug::ontgrendel,
            brug::vergrendel,
            brug::stand,
            brug::werkbak,
            brug::buitenbeeld,
            brug::dossier,
            brug::controle,
            brug::prognose,
            fg::fg_ontgrendel,
            fg::fg_vergrendel,
            fg::fg_spiegel,
            fg::toon_persoonlijk_venster,
        ])
        .run(tauri::generate_context!())
        .expect("de schil kon niet worden gestart");
}

#[cfg(test)]
mod tests {
    use super::*;

    // Deze test legt de koppeling vast die anders alleen uit de documentatie
    // van Tauri blijkt, en die bij een upgrade stilletjes kan wijzigen: de
    // feature bepaalt de bouwsoort, en niets anders.
    #[test]
    fn de_bouwsoort_volgt_de_feature() {
        if cfg!(feature = "custom-protocol") {
            assert!(!is_ontwikkelbouw(), "met custom-protocol hoort dit een uitgave te zijn");
            assert_eq!(bouwsoort(), "uitgave");
            assert!(schermbron().contains("ingebouwde"));
        } else {
            assert!(
                is_ontwikkelbouw(),
                "zonder custom-protocol hoort dit een ontwikkelbouw te zijn"
            );
            assert_eq!(bouwsoort(), "ontwikkelbouw");
            assert!(schermbron().contains("niet geschikt om te installeren"));
        }
    }
}
