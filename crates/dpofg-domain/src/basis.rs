//! Basistypen die het hele domein deelt.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

/// Identificatie van een record.
///
/// UUID versie 7: tijdgeordend, zodat records in aanmaakvolgorde sorteren
/// zonder dat daar een aparte kolom voor nodig is, en zonder dat een oplopende
/// teller verraadt hoeveel records er zijn.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Id(Uuid);

impl Id {
    pub fn nieuw() -> Self {
        Self(Uuid::now_v7())
    }

    pub fn uit_uuid(u: Uuid) -> Self {
        Self(u)
    }

    pub fn uuid(&self) -> Uuid {
        self.0
    }
}

impl fmt::Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::str::FromStr for Id {
    type Err = uuid::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(Uuid::parse_str(s)?))
    }
}

/// Het compartiment waarin een record valt.
///
/// Het compartiment is een eigenschap van het object, niet van de weergave
/// (modelleerprincipe uit het plan). Wie de sleutel van een compartiment niet
/// heeft, ziet de inhoud niet — ook niet door een fout in de toegangscontrole.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Compartiment(String);

impl Compartiment {
    pub const ALGEMEEN: &'static str = "algemeen";
    pub const VERTROUWELIJK: &'static str = "vertrouwelijk";
    pub const FG_PERSOONLIJK: &'static str = "fg-persoonlijk";

    pub fn nieuw(naam: impl Into<String>) -> Self {
        Self(naam.into())
    }

    pub fn algemeen() -> Self {
        Self(Self::ALGEMEEN.into())
    }

    pub fn naam(&self) -> &str {
        &self.0
    }
}

impl Default for Compartiment {
    fn default() -> Self {
        Self::algemeen()
    }
}

impl fmt::Display for Compartiment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// De vaststellingsstatus van een record.
///
/// Concept is een geldige toestand (acceptatiecriterium 14 uit het
/// foutbestendigheidshoofdstuk): werk mag halverwege blijven staan zonder dat
/// het verloren gaat en zonder dat het meetelt als vastgesteld.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    /// In bewerking. Telt niet mee als vastgesteld en blokkeert niets.
    Concept,
    /// Vastgesteld en in gebruik.
    Vastgesteld,
    /// Vastgesteld, maar er is iets gewijzigd waardoor herziening nodig is.
    HerzieningNodig,
    /// Niet meer in gebruik; blijft bewaard voor de verantwoording.
    Ingetrokken,
}

impl Status {
    pub fn is_actief(&self) -> bool {
        matches!(self, Self::Vastgesteld | Self::HerzieningNodig)
    }

    pub fn omschrijving(&self) -> &'static str {
        match self {
            Self::Concept => "concept",
            Self::Vastgesteld => "vastgesteld",
            Self::HerzieningNodig => "herziening nodig",
            Self::Ingetrokken => "ingetrokken",
        }
    }
}

/// Een verplichte motivering.
///
/// Modelleerprincipe uit het plan: elke motiveringsplicht is een verplicht
/// veld, niet een tekstvak dat leeg mag blijven. Daarom draagt dit type de
/// naam van degene die motiveerde en het tijdstip; een motivering zonder
/// afzender is geen motivering.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Motivering {
    pub tekst: String,
    pub door: String,
    pub op: DateTime<Utc>,
}

impl Motivering {
    /// Legt een motivering vast. Faalt bij een lege of nietszeggende tekst.
    pub fn nieuw(
        tekst: impl Into<String>,
        door: impl Into<String>,
        op: DateTime<Utc>,
    ) -> Result<Self, crate::DomeinFout> {
        let tekst = tekst.into();
        let opgeschoond = tekst.trim();
        if opgeschoond.len() < 10 {
            return Err(crate::DomeinFout::MotiveringTeKort {
                gekregen: opgeschoond.len(),
                minimaal: 10,
            });
        }
        Ok(Self { tekst: opgeschoond.to_string(), door: door.into(), op })
    }
}

/// Wie een record voor het laatst heeft aangeraakt.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Herkomst {
    pub aangemaakt_door: String,
    pub aangemaakt_op: DateTime<Utc>,
    pub gewijzigd_door: String,
    pub gewijzigd_op: DateTime<Utc>,
    pub vastgesteld_door: Option<String>,
    pub vastgesteld_op: Option<DateTime<Utc>>,
    /// Wanneer het record voor het laatst is herzien.
    ///
    /// Los van `gewijzigd_op`: een spelfout herstellen is geen herziening.
    pub herzien_op: Option<DateTime<Utc>>,
}

impl Herkomst {
    pub fn nieuw(door: impl Into<String>, op: DateTime<Utc>) -> Self {
        let door = door.into();
        Self {
            aangemaakt_door: door.clone(),
            aangemaakt_op: op,
            gewijzigd_door: door,
            gewijzigd_op: op,
            vastgesteld_door: None,
            vastgesteld_op: None,
            herzien_op: None,
        }
    }

    pub fn wijzig(&mut self, door: impl Into<String>, op: DateTime<Utc>) {
        self.gewijzigd_door = door.into();
        self.gewijzigd_op = op;
    }

    pub fn stel_vast(&mut self, door: impl Into<String>, op: DateTime<Utc>) {
        let door = door.into();
        self.vastgesteld_door = Some(door.clone());
        self.vastgesteld_op = Some(op);
        self.herzien_op = Some(op);
        self.wijzig(door, op);
    }

    /// Hoeveel maanden geleden het record is herzien, ten opzichte van een
    /// peilmoment. `None` wanneer het nooit is herzien.
    pub fn maanden_sinds_herziening(&self, nu: DateTime<Utc>) -> Option<i64> {
        self.herzien_op.map(|h| {
            let dagen = (nu - h).num_days();
            // Ruwe maat, uitsluitend voor signalering; niet voor termijnen.
            dagen / 30
        })
    }
}

/// Aanduiding dat een record is overgenomen uit een eerdere administratie en
/// nog niet is geverifieerd.
///
/// Uit het foutbestendigheidshoofdstuk: dit kenmerk gaat mee in élke export.
/// Er mag geen weergave bestaan waarin het register er completer uitziet dan
/// het is.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Overgenomen {
    pub bron: String,
    pub overgenomen_op: DateTime<Utc>,
    pub geverifieerd_op: Option<DateTime<Utc>>,
    pub geverifieerd_door: Option<String>,
}

impl Overgenomen {
    pub fn is_geverifieerd(&self) -> bool {
        self.geverifieerd_op.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn nu() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 8, 18, 9, 0, 0).unwrap()
    }

    #[test]
    fn identificaties_zijn_uniek_en_tijdgeordend() {
        let a = Id::nieuw();
        let b = Id::nieuw();
        assert_ne!(a, b);
        assert!(a < b, "uuid v7 hoort in aanmaakvolgorde te sorteren");
    }

    #[test]
    fn identificatie_heen_en_terug() {
        let a = Id::nieuw();
        let tekst = a.to_string();
        let terug: Id = tekst.parse().unwrap();
        assert_eq!(a, terug);
    }

    #[test]
    fn motivering_weigert_nietszeggende_tekst() {
        assert!(Motivering::nieuw("", "u1", nu()).is_err());
        assert!(Motivering::nieuw("   ", "u1", nu()).is_err());
        assert!(Motivering::nieuw("ok", "u1", nu()).is_err());
        assert!(Motivering::nieuw("niet van toepassing", "u1", nu()).is_ok());
    }

    #[test]
    fn motivering_wordt_opgeschoond() {
        let m = Motivering::nieuw("  wel een echte reden  ", "u1", nu()).unwrap();
        assert_eq!(m.tekst, "wel een echte reden");
    }

    #[test]
    fn status_actief() {
        assert!(!Status::Concept.is_actief());
        assert!(Status::Vastgesteld.is_actief());
        assert!(Status::HerzieningNodig.is_actief());
        assert!(!Status::Ingetrokken.is_actief());
    }

    #[test]
    fn vaststellen_zet_herzieningsdatum() {
        let mut h = Herkomst::nieuw("u1", nu());
        assert!(h.herzien_op.is_none());
        h.stel_vast("u2", nu());
        assert_eq!(h.vastgesteld_door.as_deref(), Some("u2"));
        assert!(h.herzien_op.is_some());
    }

    #[test]
    fn maanden_sinds_herziening() {
        let mut h = Herkomst::nieuw("u1", nu());
        h.stel_vast("u1", nu());
        let veertien_maanden_later = Utc.with_ymd_and_hms(2027, 10, 18, 9, 0, 0).unwrap();
        assert!(h.maanden_sinds_herziening(veertien_maanden_later).unwrap() >= 14);
    }

    #[test]
    fn compartiment_standaard_is_algemeen() {
        assert_eq!(Compartiment::default().naam(), "algemeen");
    }
}
