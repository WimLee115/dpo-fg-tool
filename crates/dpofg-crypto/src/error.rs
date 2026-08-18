//! Fouttypen van de cryptografielaag.
//!
//! Bewust grofkorrelig: een aanvaller mag uit een foutmelding niet kunnen
//! afleiden *waarom* een ontsleuteling faalde. `Ontsleuteling` dekt daarom
//! zowel een verkeerde sleutel als een gemanipuleerde authenticatietag.

use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CryptoFout {
    /// Sleutelafleiding is mislukt (ongeldige parameters of te weinig geheugen).
    Sleutelafleiding(String),
    /// Ontsleutelen is mislukt. Verkeerde sleutel of gemanipuleerde gegevens —
    /// het onderscheid wordt bewust niet gemaakt.
    Ontsleuteling,
    /// Versleutelen is mislukt.
    Versleuteling,
    /// Een veld heeft niet de vereiste lengte.
    OngeldigeLengte { veld: &'static str, verwacht: usize, gekregen: usize },
    /// Het formaat van een geserialiseerde envelop is onbekend of beschadigd.
    OngeldigFormaat(String),
    /// De opgegeven versie van het envelopformaat wordt niet ondersteund.
    OnbekendeVersie(u8),
    /// De willekeurbron leverde geen bruikbare waarde.
    Willekeurbron,
    /// Een sleutel is gebruikt voor een ander doel dan waarvoor hij is afgeleid.
    VerkeerdDoel { verwacht: String, gekregen: String },
}

impl fmt::Display for CryptoFout {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Sleutelafleiding(m) => write!(f, "sleutelafleiding mislukt: {m}"),
            Self::Ontsleuteling => write!(
                f,
                "ontsleutelen mislukt: onjuiste sleutel of gewijzigde gegevens"
            ),
            Self::Versleuteling => write!(f, "versleutelen mislukt"),
            Self::OngeldigeLengte { veld, verwacht, gekregen } => write!(
                f,
                "veld '{veld}' heeft lengte {gekregen}, verwacht {verwacht}"
            ),
            Self::OngeldigFormaat(m) => write!(f, "ongeldig formaat: {m}"),
            Self::OnbekendeVersie(v) => write!(f, "onbekende formaatversie: {v}"),
            Self::Willekeurbron => write!(f, "willekeurbron niet beschikbaar"),
            Self::VerkeerdDoel { verwacht, gekregen } => write!(
                f,
                "sleutel is afgeleid voor doel '{gekregen}' maar wordt gebruikt voor '{verwacht}'"
            ),
        }
    }
}

impl std::error::Error for CryptoFout {}

pub type Resultaat<T> = std::result::Result<T, CryptoFout>;
