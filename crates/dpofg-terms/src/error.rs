//! Fouttypen van de termijnenmotor.

use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TermijnFout {
    /// De kalender dekt de gevraagde datum niet; de uitkomst zou een gok zijn.
    KalenderDektNiet { datum: String, jurisdictie: String, van: i32, tot_en_met: i32 },
    /// Een datum valt buiten het bereik dat de kalenderrekenkunde aankan.
    DatumBuitenBereik,
    /// Er is een onwaarschijnlijk lange reeks vrije dagen aangetroffen.
    TeveelVrijeDagen(String),
    /// De opgegeven tijdzone is onbekend.
    OnbekendeTijdzone(String),
    /// Het ankertijdstip bestaat niet in de opgegeven tijdzone (zomertijdgat).
    TijdstipBestaatNiet(String),
    /// De duur van de termijn is nul of negatief.
    OngeldigeDuur(String),
    /// Een opschorting eindigt vóór zij begint.
    OpschortingLooptTerug { van: String, tot: String },
    /// Een opschorting begint vóór de termijn zelf.
    OpschortingVoorAanvang { aanvang: String, opschorting: String },
    /// Er wordt geprobeerd te hervatten terwijl er geen opschorting loopt.
    GeenLopendeOpschorting,
    /// Er wordt geprobeerd op te schorten terwijl er al een opschorting loopt.
    OpschortingLooptAl,
    /// De termijn is van een soort die niet opgeschort mag worden.
    OpschortingNietToegestaan(String),
}

impl fmt::Display for TermijnFout {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::KalenderDektNiet { datum, jurisdictie, van, tot_en_met } => write!(
                f,
                "de feestdagenkalender voor {jurisdictie} dekt {van} tot en met {tot_en_met} \
                 en daarmee niet {datum}; werk het kennispakket bij voordat deze termijn wordt berekend"
            ),
            Self::DatumBuitenBereik => write!(f, "de berekende datum valt buiten het geldige bereik"),
            Self::TeveelVrijeDagen(d) => write!(
                f,
                "vanaf {d} is binnen veertien dagen geen werkdag gevonden; \
                 controleer de feestdagenkalender"
            ),
            Self::OnbekendeTijdzone(z) => write!(f, "onbekende tijdzone: {z}"),
            Self::TijdstipBestaatNiet(t) => write!(
                f,
                "het tijdstip {t} bestaat niet in deze tijdzone; \
                 dit valt in het uur dat bij de overgang naar zomertijd wordt overgeslagen"
            ),
            Self::OngeldigeDuur(m) => write!(f, "ongeldige duur: {m}"),
            Self::OpschortingLooptTerug { van, tot } => {
                write!(f, "opschorting van {van} tot {tot} loopt terug in de tijd")
            }
            Self::OpschortingVoorAanvang { aanvang, opschorting } => write!(
                f,
                "opschorting op {opschorting} ligt vóór de aanvang van de termijn op {aanvang}"
            ),
            Self::GeenLopendeOpschorting => {
                write!(f, "hervatten is niet mogelijk: er loopt geen opschorting")
            }
            Self::OpschortingLooptAl => {
                write!(f, "opschorten is niet mogelijk: er loopt al een opschorting")
            }
            Self::OpschortingNietToegestaan(m) => write!(
                f,
                "deze termijn kan niet worden opgeschort: {m}"
            ),
        }
    }
}

impl std::error::Error for TermijnFout {}

pub type Resultaat<T> = std::result::Result<T, TermijnFout>;
