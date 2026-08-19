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
pub mod sessie;
pub mod vorm;

use tauri::Manager;

/// Start de schil.
pub fn draai() {
    tauri::Builder::default()
        .setup(|app| {
            app.manage(sessie::Sessie::nieuw());
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
        ])
        .run(tauri::generate_context!())
        .expect("de schil kon niet worden gestart");
}
