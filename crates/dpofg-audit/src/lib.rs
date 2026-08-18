//! Manipulatiebestendig ketenlogboek voor `dpo-fg-tool`.
//!
//! Dit logboek is de bewijsdrager van het product. Het legt handelingen vast in
//! een hashketen, zodat wijzigen en verwijderen zichtbaar worden, en het kent
//! ankers waarmee ook afkappen aan het einde detecteerbaar wordt.
//!
//! # Wat het logboek bewijst — en wat niet
//!
//! | Bewering | Onderbouwd door |
//! |---|---|
//! | Deze regels staan in deze volgorde | de hashketen |
//! | Regel N is niet gewijzigd | de hashketen |
//! | Regel N is niet verwijderd | volgnummers plus de hashketen |
//! | Er is aan het eind niets weggehaald | **alleen** een anker |
//! | Regel N is op tijdstip T geschreven | **niets in dit logboek**; alleen een extern vastgelegd anker begrenst het moment |
//!
//! Die laatste twee rijen zijn de reden dat elk verificatierapport zijn eigen
//! reikwijdte meedraagt. Een rapport zonder die zin overdrijft wat het aantoont.
//!
//! # Voorbeeld
//!
//! ```
//! use chrono::{TimeZone, Utc};
//! use dpofg_audit::{keten_aan, verifieer, Actor, Anker, Gebeurtenis, Handeling, Ketenstand};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let mut stand = Ketenstand::leeg();
//! let mut regels = Vec::new();
//!
//! let gebeurtenis = Gebeurtenis::nieuw(
//!     Handeling::RecordVastgesteld,
//!     Actor::nieuw("u1", "A. de Vries", "fg"),
//!     Utc.with_ymd_and_hms(2026, 8, 18, 9, 14, 0).unwrap(),
//!     "verwerking",
//!     "0412-K",
//!     "algemeen",
//!     "registerregel vastgesteld",
//! );
//! let (regel, nieuwe_stand) = keten_aan(&stand, gebeurtenis)?;
//! regels.push(regel);
//! stand = nieuwe_stand;
//!
//! // Anker de stand en bewaar het anker buiten het systeem.
//! let sleutel = dpofg_audit::anker::nieuw_sleutelpaar();
//! let anker = Anker::plaats(&sleutel, "kluis-1", &stand, Utc::now())?;
//!
//! let rapport = verifieer(&regels, Some(&anker))?;
//! assert!(rapport.is_ongeschonden());
//! println!("{}", rapport.reikwijdte());
//! # Ok(())
//! # }
//! ```

#![forbid(unsafe_code)]
#![warn(missing_debug_implementations, rust_2018_idioms)]

pub mod anker;
mod error;
mod gebeurtenis;
mod keten;
pub mod verificatie;

pub use anker::Anker;
pub use error::{AuditFout, Resultaat};
pub use gebeurtenis::{Actor, Gebeurtenis, Handeling};
pub use keten::{keten_aan, Ketenregel, Ketenstand, GENESIS};
pub use verificatie::{verifieer, Ankerstatus, Bevinding, Bevindingsoort, Verificatierapport};

/// Versie van deze crate.
pub const VERSIE: &str = env!("CARGO_PKG_VERSION");
