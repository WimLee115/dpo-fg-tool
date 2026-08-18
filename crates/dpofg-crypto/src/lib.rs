//! Cryptografische bouwstenen van `dpo-fg-tool`.
//!
//! Deze crate bevat geen domeinlogica en doet geen invoer of uitvoer. Hij levert
//! uitsluitend: veilige geheimtypen, sleutelafleiding, versleuteling en de
//! sleutelhiërarchie van de kluis.
//!
//! # Uitgangspunten
//!
//! 1. **Geen eigen cryptografische primitieven.** Alles steunt op beproefde,
//!    gepinde bibliotheken. Deze crate zet ze samen; hij vindt niets uit.
//! 2. **Sleutelmateriaal wordt overschreven** zodra het niet meer nodig is, en
//!    is nooit zichtbaar in `Debug`, logregels of foutmeldingen.
//! 3. **Elke versleuteling is gebonden aan haar context.** Een envelop die uit
//!    zijn record wordt gelicht, is onleesbaar.
//! 4. **Fouten verraden niets.** Een verkeerd wachtwoord en gemanipuleerde
//!    gegevens leveren dezelfde melding op.
//!
//! # Voorbeeld
//!
//! ```
//! use dpofg_crypto::{
//!     aead::{self, Binding},
//!     kdf::KdfParameters,
//!     keys::GeopendeKluis,
//!     Wachtwoordzin,
//! };
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let wachtwoord = Wachtwoordzin::nieuw("paard batterij niet vastzetten");
//! let kluis = GeopendeKluis::aanmaken(&wachtwoord, KdfParameters::TEST_ONVEILIG)?;
//!
//! let (hoofd, sleutel) = kluis.compartiment_aanmaken("algemeen")?;
//! let binding = Binding::nieuw("verwerking.omschrijving", "0412-K", "algemeen");
//!
//! let envelop = aead::versleutel(&sleutel, &binding, b"Verzuimregistratie")?;
//! let terug = aead::ontsleutel(&sleutel, &binding, &envelop)?;
//! assert_eq!(terug, b"Verzuimregistratie");
//!
//! // Het kluishoofd en het compartimenthoofd mogen onversleuteld worden bewaard.
//! let _ = serde_json::to_string(kluis.hoofd())?;
//! let _ = serde_json::to_string(&hoofd)?;
//! # Ok(())
//! # }
//! ```

#![forbid(unsafe_code)]
#![warn(missing_debug_implementations, rust_2018_idioms)]

pub mod aead;
pub mod blind_index;
mod error;
pub mod kdf;
pub mod keys;
mod secret;

pub use error::{CryptoFout, Resultaat};
pub use secret::{gelijk_in_constante_tijd, Geheim, Wachtwoordzin};

/// Versie van deze crate, voor vastlegging in het auditlogboek.
pub const VERSIE: &str = env!("CARGO_PKG_VERSION");
