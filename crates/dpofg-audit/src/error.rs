//! Fouttypen van het ketenlogboek.

use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuditFout {
    /// De keten is gebroken: de vastgelegde vorige hash klopt niet.
    Ketenbreuk { volgnummer: u64, verwacht: String, gevonden: String },
    /// Er ontbreekt een volgnummer: een regel is verwijderd.
    OntbrekendVolgnummer { verwacht: u64, gevonden: u64 },
    /// Een volgnummer komt twee keer voor.
    DubbelVolgnummer(u64),
    /// De inhoud van een regel komt niet overeen met de vastgelegde hash.
    InhoudGewijzigd { volgnummer: u64 },
    /// De tijdstempel loopt terug ten opzichte van de voorgaande regel.
    TijdLooptTerug { volgnummer: u64, vorige: String, deze: String },
    /// Een zegel of anker is niet geldig.
    OngeldigZegel(String),
    /// Het logboek is leeg terwijl er een regel werd verwacht.
    LeegLogboek,
    /// Serialisatie is mislukt.
    Serialisatie(String),
}

impl fmt::Display for AuditFout {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Ketenbreuk { volgnummer, verwacht, gevonden } => write!(
                f,
                "ketenbreuk bij regel {volgnummer}: verwachte voorgaande hash {verwacht}, gevonden {gevonden}"
            ),
            Self::OntbrekendVolgnummer { verwacht, gevonden } => write!(
                f,
                "regel {verwacht} ontbreekt; eerstvolgende regel is {gevonden}"
            ),
            Self::DubbelVolgnummer(n) => write!(f, "volgnummer {n} komt meer dan eens voor"),
            Self::InhoudGewijzigd { volgnummer } => {
                write!(f, "de inhoud van regel {volgnummer} is gewijzigd")
            }
            Self::TijdLooptTerug { volgnummer, vorige, deze } => write!(
                f,
                "tijdstip van regel {volgnummer} ({deze}) ligt vóór dat van de voorgaande regel ({vorige})"
            ),
            Self::OngeldigZegel(m) => write!(f, "ongeldig zegel: {m}"),
            Self::LeegLogboek => write!(f, "het logboek bevat geen regels"),
            Self::Serialisatie(m) => write!(f, "serialisatie mislukt: {m}"),
        }
    }
}

impl std::error::Error for AuditFout {}

pub type Resultaat<T> = std::result::Result<T, AuditFout>;
