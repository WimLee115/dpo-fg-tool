//! Termijnenmotor van `dpo-fg-tool`.
//!
//! Dit is het onderdeel waar één rekenfout elk dossier ongemerkt te laat maakt.
//! Daarom gelden hier strengere regels dan elders in het product:
//!
//! 1. **De eenheid is onderdeel van het type.** Een maandtermijn kan niet per
//!    ongeluk in dagen worden doorgerekend, want er is geen functie die dat kan.
//! 2. **Urentermijnen rekenen in UTC**, kalendertermijnen in lokale
//!    kalenderdagen. Beide zijn immuun voor zomertijd, maar om verschillende
//!    redenen.
//! 3. **Elke uitkomst draagt haar verantwoording mee.** Een [`Deadline`] zonder
//!    de toegepaste regel en de bepaling waarop die berust, bestaat niet.
//! 4. **De feestdagenkalender komt uit het kennispakket.** Valt een berekening
//!    buiten het dekkingsvenster, dan faalt zij zichtbaar in plaats van te gokken.
//! 5. **Termijnsoorten staan niet in deze crate.** Wetteksten, duren en
//!    grondslagen zijn kennispakketinhoud; hier staan alleen de typen waarmee
//!    ze worden beschreven.
//!
//! # Voorbeeld
//!
//! ```
//! use chrono::{TimeZone, Utc};
//! use dpofg_terms::{
//!     bereken, tijdzone, Feestdagenkalender, Termijnsoort,
//! };
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let soort = Termijnsoort::uren(
//!     "AVG-33-MELDING",
//!     "melding datalek aan de toezichthouder",
//!     72,
//!     "art. 33 lid 1 AVG",
//! );
//!
//! // Vrijdagmiddag 16:40 lokale tijd.
//! let anker = Utc.with_ymd_and_hms(2026, 8, 21, 14, 40, 0).unwrap();
//! let kalender = Feestdagenkalender::leeg("NL", 2026, 2027);
//!
//! let deadline = bereken(&soort, anker, tijdzone("Europe/Amsterdam")?, &kalender)?;
//!
//! // 72 uur later: maandag, zelfde tijdstip. Geen verlenging voor het weekend.
//! assert_eq!(deadline.moment, Utc.with_ymd_and_hms(2026, 8, 24, 14, 40, 0).unwrap());
//! assert!(deadline.verantwoording.contains("zonder verlenging"));
//! # Ok(())
//! # }
//! ```

#![forbid(unsafe_code)]
#![warn(missing_debug_implementations, rust_2018_idioms)]

pub mod berekening;
mod error;
pub mod kalender;
pub mod kalenderrekenen;
pub mod opschorting;
pub mod soort;

pub use berekening::{bereken, tijdzone, Deadline, TIJDZONE_NL};
pub use error::{Resultaat, TermijnFout};
pub use kalender::Feestdagenkalender;
pub use kalenderrekenen::{
    is_schrikkeljaar, tel_dagen_op, tel_jaren_op, tel_maanden_op, tel_weken_op,
};
pub use opschorting::{LopendeTermijn, Opschorting, Termijnstatus};
pub use soort::{
    Aanvang, Eenheid, Rechtsstelsel, Termijnsoort, ToegepasteVerlenging, Verlengingsrecht,
};

/// Versie van deze crate.
pub const VERSIE: &str = env!("CARGO_PKG_VERSION");
