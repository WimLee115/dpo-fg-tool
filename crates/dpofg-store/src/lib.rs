//! Versleutelde opslag voor `dpo-fg-tool`.
//!
//! Eén kluisbestand bevat alles: de records, hun versiegeschiedenis, de
//! bijlagen, het ketenlogboek en de kennispakketten. De inhoud is versleuteld
//! per compartiment; wat onversleuteld in de tabellen staat is bewust beperkt
//! en in [`schema`] benoemd.
//!
//! # Wat deze laag garandeert
//!
//! * Bewaren en loggen gebeuren in één transactie. Er bestaat geen toestand
//!   waarin een wijziging is opgeslagen zonder logboekregel.
//! * Het logboek is append-only, ook op databaseniveau: twee triggers weigeren
//!   elke wijziging en elke verwijdering.
//! * Niets wordt hard overschreven; elke vorige versie blijft leesbaar.
//! * Een kluis van een nieuwere uitgave wordt geweigerd in plaats van
//!   half begrepen geopend.

#![forbid(unsafe_code)]
#![warn(rust_2018_idioms)]

mod error;
pub mod kluis;
pub mod schema;
pub mod spiegel;

pub use error::{Resultaat, StoreFout};
pub use kluis::{Installatiekop, Kluis, Recordkop};
pub use schema::SCHEMAVERSIE;
pub use spiegel::{spiegelhash, Spiegelregel, SPIEGELSOORT};

/// Versie van deze crate.
pub const VERSIE: &str = env!("CARGO_PKG_VERSION");
