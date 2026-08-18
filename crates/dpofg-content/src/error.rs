//! Fouttypen van de kennispakketlaag.

use std::fmt;

#[derive(Debug)]
pub enum ContentFout {
    /// De handtekening onder het pakket klopt niet.
    OngeldigeHandtekening(String),
    /// Het pakket is ondertekend met een sleutel die niet is vertrouwd.
    OnbekendeUitgever { sleutel: String },
    /// Er wordt geprobeerd een oudere versie te installeren dan de huidige.
    ///
    /// Terugrollen naar een oudere versie van de juridische inhoud is een
    /// aanval, geen vergissing: het zet termijnen en drempels terug naar een
    /// toestand waarin de organisatie in overtreding is zonder het te zien.
    Terugrol { huidig: String, aangeboden: String },
    /// Het pakket is niet leesbaar.
    OngeldigFormaat(String),
    /// Er is geen kennispakket geïnstalleerd.
    GeenPakket,
    /// Het gevraagde onderdeel staat niet in het pakket.
    OnbekendeCode { soort: String, code: String },
    /// De consolidatiedatum ligt te ver in het verleden.
    Verouderd { consolidatiedatum: String, dagen: i64 },
}

impl fmt::Display for ContentFout {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OngeldigeHandtekening(m) => write!(
                f,
                "de handtekening onder dit kennispakket klopt niet ({m}); het pakket wordt niet \
                 geïnstalleerd"
            ),
            Self::OnbekendeUitgever { sleutel } => write!(
                f,
                "dit kennispakket is ondertekend met een onbekende sleutel ({}…); \
                 alleen pakketten van een vertrouwde uitgever worden geïnstalleerd",
                &sleutel[..sleutel.len().min(16)]
            ),
            Self::Terugrol { huidig, aangeboden } => write!(
                f,
                "het aangeboden pakket ({aangeboden}) is ouder dan het geïnstalleerde ({huidig}). \
                 Terugrollen van juridische inhoud zet termijnen en drempels terug naar een \
                 toestand die niet meer geldt en wordt geweigerd"
            ),
            Self::OngeldigFormaat(m) => write!(f, "onleesbaar kennispakket: {m}"),
            Self::GeenPakket => write!(
                f,
                "er is geen kennispakket geïnstalleerd; zonder juridische inhoud kunnen geen \
                 termijnen worden berekend"
            ),
            Self::OnbekendeCode { soort, code } => {
                write!(f, "het kennispakket kent geen {soort} met code '{code}'")
            }
            Self::Verouderd { consolidatiedatum, dagen } => write!(
                f,
                "het kennispakket is geconsolideerd op {consolidatiedatum}, {dagen} dagen geleden. \
                 Controleer of er sindsdien wetgeving is gewijzigd"
            ),
        }
    }
}

impl std::error::Error for ContentFout {}

pub type Resultaat<T> = std::result::Result<T, ContentFout>;
