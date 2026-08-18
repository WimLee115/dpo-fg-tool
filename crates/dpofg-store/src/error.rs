//! Fouttypen van de opslaglaag.

use std::fmt;

#[derive(Debug)]
pub enum StoreFout {
    /// Fout uit de databaselaag.
    Database(rusqlite::Error),
    /// Fout uit de cryptografielaag.
    Crypto(dpofg_crypto::CryptoFout),
    /// Fout uit het ketenlogboek.
    Audit(dpofg_audit::AuditFout),
    /// Serialisatie van een record is mislukt.
    Serialisatie(serde_json::Error),
    /// Invoer of uitvoer op het bestandssysteem is mislukt.
    Bestand(std::io::Error),
    /// De kluis heeft een schemaversie die deze uitgave niet kent.
    ///
    /// Bewust een harde fout: doorgaan met een onbekend schema betekent
    /// gegevens verkeerd lezen zonder dat iemand het merkt.
    OnbekendeSchemaversie { in_kluis: u32, ondersteund: u32 },
    /// De kluis is van een nieuwere uitgave dan dit programma.
    KluisIsNieuwer { in_kluis: u32, ondersteund: u32 },
    /// Er is geen record met deze identificatie.
    NietGevonden { soort: String, id: String },
    /// Het compartiment is niet ontgrendeld.
    CompartimentGesloten(String),
    /// Het compartiment bestaat niet in deze kluis.
    OnbekendCompartiment(String),
    /// De kluis is al in gebruik door een ander proces.
    KluisInGebruik(String),
    /// Het bestand is geen kluis van dit product.
    GeenKluisbestand(String),
    /// Een blob komt niet overeen met zijn hash.
    BlobBeschadigd { hash: String },
}

impl fmt::Display for StoreFout {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Database(e) => write!(f, "databasefout: {e}"),
            Self::Crypto(e) => write!(f, "{e}"),
            Self::Audit(e) => write!(f, "{e}"),
            Self::Serialisatie(e) => write!(f, "serialisatie mislukt: {e}"),
            Self::Bestand(e) => write!(f, "bestandsfout: {e}"),
            Self::OnbekendeSchemaversie { in_kluis, ondersteund } => write!(
                f,
                "de kluis draait schemaversie {in_kluis}; deze uitgave kent versie {ondersteund}. \
                 Werk het programma bij voordat u verdergaat"
            ),
            Self::KluisIsNieuwer { in_kluis, ondersteund } => write!(
                f,
                "de kluis is aangemaakt met een nieuwere uitgave (schemaversie {in_kluis}, deze \
                 uitgave kent {ondersteund}). Openen met deze uitgave zou gegevens kunnen \
                 beschadigen en wordt geweigerd"
            ),
            Self::NietGevonden { soort, id } => write!(f, "geen {soort} gevonden met kenmerk {id}"),
            Self::CompartimentGesloten(c) => write!(
                f,
                "het compartiment '{c}' is niet ontgrendeld; de inhoud is niet te lezen"
            ),
            Self::OnbekendCompartiment(c) => write!(f, "de kluis kent geen compartiment '{c}'"),
            Self::KluisInGebruik(p) => write!(
                f,
                "de kluis is in gebruik door een ander proces ({p}). Sluit dat eerst af; twee \
                 processen tegelijk in dezelfde kluis leidt tot verlies van wijzigingen"
            ),
            Self::GeenKluisbestand(p) => write!(f, "{p} is geen kluisbestand van dit product"),
            Self::BlobBeschadigd { hash } => write!(
                f,
                "de bijlage met hash {hash} komt niet overeen met haar inhoud en is beschadigd"
            ),
        }
    }
}

impl std::error::Error for StoreFout {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Database(e) => Some(e),
            Self::Crypto(e) => Some(e),
            Self::Audit(e) => Some(e),
            Self::Serialisatie(e) => Some(e),
            Self::Bestand(e) => Some(e),
            _ => None,
        }
    }
}

impl From<rusqlite::Error> for StoreFout {
    fn from(e: rusqlite::Error) -> Self {
        Self::Database(e)
    }
}
impl From<dpofg_crypto::CryptoFout> for StoreFout {
    fn from(e: dpofg_crypto::CryptoFout) -> Self {
        Self::Crypto(e)
    }
}
impl From<dpofg_audit::AuditFout> for StoreFout {
    fn from(e: dpofg_audit::AuditFout) -> Self {
        Self::Audit(e)
    }
}
impl From<serde_json::Error> for StoreFout {
    fn from(e: serde_json::Error) -> Self {
        Self::Serialisatie(e)
    }
}
impl From<std::io::Error> for StoreFout {
    fn from(e: std::io::Error) -> Self {
        Self::Bestand(e)
    }
}

pub type Resultaat<T> = std::result::Result<T, StoreFout>;
