//! Fouttypen van het domeinmodel.

use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DomeinFout {
    /// Een motivering is te kort om iets te betekenen.
    MotiveringTeKort { gekregen: usize, minimaal: usize },
    /// Vaststellen is geweigerd omdat verplichte onderdelen ontbreken.
    NietVolledig { soort: String, ontbreekt: Vec<String> },
    /// Een statusovergang is niet toegestaan.
    OngeldigeStatusovergang { van: String, naar: String, reden: String },
    /// Een waarde valt buiten het toegestane bereik.
    OngeldigeWaarde { veld: String, reden: String },
    /// Een verplichte verwijzing ontbreekt.
    OntbrekendeVerwijzing { veld: String, naar: String },
    /// Een tijdstip is onmogelijk, bijvoorbeeld kennisname vóór het optreden.
    OnmogelijkTijdstip { veld: String, reden: String },
    /// Een handeling vereist een tweede persoon.
    TweedePersoonVereist { handeling: String, reden: String },
    /// Een handeling is onomkeerbaar en vereist een uitdrukkelijke bevestiging.
    BevestigingVereist { handeling: String, punten: Vec<String> },
}

impl fmt::Display for DomeinFout {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MotiveringTeKort { gekregen, minimaal } => write!(
                f,
                "de motivering telt {gekregen} tekens; er zijn er minimaal {minimaal} nodig. \
                 Beschrijf waarom dit besluit is genomen, niet dát het is genomen"
            ),
            Self::NietVolledig { soort, ontbreekt } => write!(
                f,
                "dit {soort} kan nog niet worden vastgesteld; deze onderdelen ontbreken: {}",
                ontbreekt.join(", ")
            ),
            Self::OngeldigeStatusovergang { van, naar, reden } => {
                write!(f, "overgang van '{van}' naar '{naar}' is niet mogelijk: {reden}")
            }
            Self::OngeldigeWaarde { veld, reden } => write!(f, "veld '{veld}': {reden}"),
            Self::OntbrekendeVerwijzing { veld, naar } => {
                write!(f, "veld '{veld}' moet verwijzen naar een {naar}")
            }
            Self::OnmogelijkTijdstip { veld, reden } => write!(f, "tijdstip '{veld}': {reden}"),
            Self::TweedePersoonVereist { handeling, reden } => {
                write!(f, "'{handeling}' vereist bevestiging door een tweede persoon: {reden}")
            }
            Self::BevestigingVereist { handeling, punten } => write!(
                f,
                "'{handeling}' is onomkeerbaar; bevestig eerst punt voor punt: {}",
                punten.join("; ")
            ),
        }
    }
}

impl std::error::Error for DomeinFout {}

pub type Resultaat<T> = std::result::Result<T, DomeinFout>;
