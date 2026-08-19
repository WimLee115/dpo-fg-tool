//! Domeinmodel van `dpo-fg-tool`.
//!
//! Deze crate bevat de begrippen, de records en de regels die daaruit volgen.
//! Hij doet geen invoer of uitvoer en kent geen opslag: dat maakt het model
//! testbaar zonder database en zorgt dat de regels op één plaats staan in
//! plaats van verspreid over schermen en query's.
//!
//! # Het ordenende idee
//!
//! Een record weet zelf wat het nog mist. Niet het scherm, niet de database,
//! niet een losse controlelaag — het record. Daaruit volgt:
//!
//! * Een concept mag onvolledig zijn; vaststellen mag pas als het klopt.
//! * Wat ontbreekt is een teller met een grondslag, geen foutmelding.
//! * Elke afgeleide verplichting volgt uit een antwoord dat de gebruiker al
//!   heeft gegeven, zodat hij de regel niet hoeft te kennen.
//!
//! # Voorbeeld
//!
//! ```
//! use chrono::{TimeZone, Utc};
//! use dpofg_domain::{
//!     avg::{BijzondereCategorie, Grondslag, Rol},
//!     Verwerking, Volledig,
//! };
//!
//! let nu = Utc.with_ymd_and_hms(2026, 8, 18, 9, 0, 0).unwrap();
//! let mut v = Verwerking::nieuw(
//!     "0412-K",
//!     "Verzuimregistratie",
//!     Rol::Verwerkingsverantwoordelijke,
//!     "afdeling P&O",
//!     "u1",
//!     nu,
//! );
//!
//! // Een leeg concept meldt precies wat er nog moet gebeuren.
//! let rapport = v.volledigheid();
//! assert!(!rapport.mag_vaststellen());
//! assert!(rapport.teller().starts_with("0 van de"));
//!
//! // Zodra gezondheidsgegevens worden aangevinkt, komt de uitzonderingsgrond
//! // van artikel 9 erbij — zonder dat iemand daarom hoefde te vragen.
//! v.bijzondere_categorieen.push(BijzondereCategorie::Gezondheidsgegevens);
//! assert!(v
//!     .volledigheid()
//!     .ontbreekt
//!     .iter()
//!     .any(|o| o.veld == "verwerking.uitzondering_artikel9"));
//!
//! // En met een gerechtvaardigd belang komt de belangenafweging erbij.
//! v.grondslag = Some(Grondslag::GerechtvaardigdBelang);
//! assert!(v
//!     .volledigheid()
//!     .ontbreekt
//!     .iter()
//!     .any(|o| o.veld == "verwerking.belangenafweging"));
//! ```

#![forbid(unsafe_code)]
#![warn(missing_debug_implementations, rust_2018_idioms)]

pub mod avg;
pub mod basis;
pub mod belangenafweging;
pub mod doorgifte;
pub mod dpia;
mod error;
pub mod incident;
pub mod klokken;
pub mod leverancier;
pub mod mapping;
pub mod redactie;
pub mod verwerking;
pub mod verzoek;
pub mod volledigheid;
pub mod woo;
pub mod wpg;
pub mod zorgplicht;

pub use basis::{Compartiment, Herkomst, Id, Motivering, Overgenomen, Status};
pub use belangenafweging::{Afwegingsuitkomst, Belangenafweging};
pub use doorgifte::{
    Beoordelingsuitkomst, Doorgifte, Doorgiftebeoordeling, Doorgifteinstrumentsoort,
};
pub use dpia::{Dpia, Restrisico, Restrisiconiveau, Voortoets};
pub use error::{DomeinFout, Resultaat};
pub use incident::{Aantasting, Herkomstkanaal, Incident, Meldbesluit, Risiconiveau};
pub use klokken::{
    verplichtingen_uit_incident, AfgeleideVerplichting, Ankertype, Verplichtingcode,
    Zorgplichtcontext,
};
pub use leverancier::{Contracteis, Kritikaliteit, Leverancier, Verwerkersovereenkomst};
pub use mapping::{Mappingprofiel, Veldkoppeling, Verschilrapport};
pub use redactie::{Controlesoort, Controleuitkomst, Redactiecategorie, Redactieopdracht};
pub use verwerking::{Bewaartermijn, Ontvanger, Verwerking};
pub use verzoek::{
    Betrokkenenverzoek, Termijnlezing, Verzoekkanaal, Verzoeksoort, Verzoekuitkomst,
    Vindplaatsuitkomst,
};
pub use volledigheid::{Ontbrekend, Registerrapport, Volledig, Volledigheidsrapport};
pub use woo::{Weigeringsgrond, Woouitkomst, Wooverzoek};

/// De eenheid waarin een bewaartermijn is uitgedrukt.
///
/// Een eigen, kleinere opsomming dan die van de termijnenmotor: een
/// bewaartermijn wordt niet in uren of werkdagen uitgedrukt, en die opties
/// aanbieden nodigt alleen maar uit tot een verkeerde keuze.
pub use bewaareenheid::Termijneenheid;

mod bewaareenheid {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
    #[serde(rename_all = "snake_case")]
    pub enum Termijneenheid {
        Dagen,
        Maanden,
        Jaren,
    }

    impl Termijneenheid {
        pub fn enkelvoud(&self) -> &'static str {
            match self {
                Self::Dagen => "dag",
                Self::Maanden => "maand",
                Self::Jaren => "jaar",
            }
        }

        pub fn meervoud(&self) -> &'static str {
            match self {
                Self::Dagen => "dagen",
                Self::Maanden => "maanden",
                Self::Jaren => "jaar",
            }
        }
    }
}

/// Versie van deze crate.
pub const VERSIE: &str = env!("CARGO_PKG_VERSION");
