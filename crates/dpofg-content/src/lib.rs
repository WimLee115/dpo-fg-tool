//! Kennispakketten: de juridische inhoud, buiten de programmacode.
//!
//! Wetteksten, termijnen, drempels, feestdagen, autoriteiten en alle datums
//! zitten in ondertekende pakketten met een versie en een consolidatiedatum.
//! Reden: wetgeving verandert sneller dan een softwarerelease, en een termijn
//! in de binary betekent dat een organisatie fout rekent tot er een nieuwe
//! uitgave is.
//!
//! # Wat er bij het installeren wordt gecontroleerd
//!
//! 1. De handtekening klopt, en is gezet met een **vooraf vertrouwde** sleutel.
//!    Een handtekening die alleen klopt met de bijgeleverde sleutel bewijst
//!    niets: die komt uit hetzelfde bestand.
//! 2. Het pakket is niet ouder dan wat er al staat. Terugrollen van juridische
//!    inhoud is een aanval, geen vergissing.
//! 3. De programmaversie is niet te oud voor dit pakket.

#![forbid(unsafe_code)]
#![warn(missing_debug_implementations, rust_2018_idioms)]

mod error;
pub mod pakket;
pub mod startpakket;

pub use error::{ContentFout, Resultaat};
pub use pakket::{
    nieuw_uitgeverspaar, Doorgifteinstrument, Instrumentstatus, Kennispakket, Pakketinhoud,
    Rechtsfeit,
};
pub use startpakket::startpakket;

/// Versie van deze crate.
pub const VERSIE: &str = env!("CARGO_PKG_VERSION");
