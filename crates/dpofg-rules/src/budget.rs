//! Het waarschuwingsbudget.
//!
//! Uit paragraaf 3.9 van het foutbestendigheidshoofdstuk: **meer dan vijf
//! onderbrekende meldingen per gebruiker per week is een defect in het
//! ontwerp, geen probleem van de gebruiker.**
//!
//! De reden is gedragsmatig en goed onderbouwd: wie bij elke stap wordt
//! tegengehouden, leert de melding wegklikken. Daarna werkt ook de melding niet
//! meer die er wél toe doet — en dat is precies de melding waarvoor het hele
//! systeem is gebouwd. Een product dat te veel waarschuwt, is gevaarlijker dan
//! een product dat niet waarschuwt, want het wekt de indruk dat er wordt
//! opgelet.
//!
//! Daarom is dit budget geen instelling maar een meetwaarde die als **defect**
//! wordt gerapporteerd. Wie een nieuwe blokkerende regel wil toevoegen, moet
//! een bestaande laten vervallen of aantonen dat het budget het toelaat
//! (acceptatiecriterium 7).

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Het aantal onderbrekende meldingen dat per gebruiker per week aanvaardbaar is.
pub const BUDGET_PER_WEEK: usize = 5;

/// Het aandeel waarboven een signalerende regel in de review gaat.
///
/// Een regel die in een kwartaal meer dan tachtig procent wordt genegeerd,
/// stuurt geen gedrag meer aan; hij vult alleen de lijst.
pub const NEGEERGRENS: f64 = 0.80;

/// Bijhouden van onderbrekingen per gebruiker.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Waarschuwingsbudget {
    /// Per gebruiker de tijdstippen van de onderbrekingen.
    onderbrekingen: BTreeMap<String, Vec<DateTime<Utc>>>,
    /// Per regel: hoe vaak getoond en hoe vaak opgevolgd.
    getoond: BTreeMap<String, usize>,
    opgevolgd: BTreeMap<String, usize>,
}

/// De stand van het budget voor één gebruiker.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Budgetstand {
    pub gebruiker: String,
    pub deze_week: usize,
    pub budget: usize,
    /// Of het budget is overschreden. Dan is er een ontwerpdefect te melden.
    pub overschreden: bool,
}

impl Budgetstand {
    /// De melding die bij een overschrijding hoort.
    ///
    /// Gericht aan de beheerder, niet aan de gebruiker: die kan er niets aan doen.
    pub fn defectmelding(&self) -> Option<String> {
        self.overschreden.then(|| {
            format!(
                "{} kreeg deze week {} onderbrekende meldingen; het budget is {}. \
                 Dit is een ontwerpdefect: beoordeel welke blokkerende regel kan vervallen of \
                 signalerend kan worden. Een gebruiker die leert wegklikken, klikt ook de \
                 melding weg die er wel toe doet.",
                self.gebruiker, self.deze_week, self.budget
            )
        })
    }
}

impl Waarschuwingsbudget {
    pub fn nieuw() -> Self {
        Self::default()
    }

    /// Legt een onderbreking vast.
    pub fn onderbreking(&mut self, gebruiker: &str, op: DateTime<Utc>) {
        self.onderbrekingen.entry(gebruiker.to_string()).or_default().push(op);
    }

    /// Legt vast dat een regel is getoond.
    pub fn getoond(&mut self, regelcode: &str) {
        *self.getoond.entry(regelcode.to_string()).or_default() += 1;
    }

    /// Legt vast dat een getoonde regel is opgevolgd.
    pub fn opgevolgd(&mut self, regelcode: &str) {
        *self.opgevolgd.entry(regelcode.to_string()).or_default() += 1;
    }

    /// De stand voor één gebruiker over de zeven dagen vóór het peilmoment.
    pub fn stand(&self, gebruiker: &str, nu: DateTime<Utc>) -> Budgetstand {
        let grens = nu - Duration::days(7);
        let deze_week = self
            .onderbrekingen
            .get(gebruiker)
            .map(|v| v.iter().filter(|t| **t > grens && **t <= nu).count())
            .unwrap_or(0);
        Budgetstand {
            gebruiker: gebruiker.to_string(),
            deze_week,
            budget: BUDGET_PER_WEEK,
            overschreden: deze_week > BUDGET_PER_WEEK,
        }
    }

    /// De standen van alle gebruikers die het budget overschrijden.
    pub fn overschrijdingen(&self, nu: DateTime<Utc>) -> Vec<Budgetstand> {
        self.onderbrekingen.keys().map(|g| self.stand(g, nu)).filter(|s| s.overschreden).collect()
    }

    /// Welk deel van de keren dat een regel werd getoond, hij werd genegeerd.
    pub fn negeerpercentage(&self, regelcode: &str) -> Option<f64> {
        let getoond = *self.getoond.get(regelcode)? as f64;
        if getoond == 0.0 {
            return None;
        }
        let opgevolgd = *self.opgevolgd.get(regelcode).unwrap_or(&0) as f64;
        Some((getoond - opgevolgd) / getoond)
    }

    /// De regels die zo vaak worden genegeerd dat ze in de review horen.
    pub fn regels_voor_review(&self) -> Vec<(String, f64)> {
        let mut uit: Vec<(String, f64)> = self
            .getoond
            .keys()
            .filter_map(|c| self.negeerpercentage(c).map(|p| (c.clone(), p)))
            .filter(|(_, p)| *p > NEGEERGRENS)
            .collect();
        uit.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        uit
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn t(dag: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 8, dag, 9, 0, 0).unwrap()
    }

    #[test]
    fn binnen_het_budget_geen_defect() {
        let mut b = Waarschuwingsbudget::nieuw();
        for dag in 12..=16 {
            b.onderbreking("u1", t(dag));
        }
        let stand = b.stand("u1", t(18));
        assert_eq!(stand.deze_week, 5);
        assert!(!stand.overschreden);
        assert!(stand.defectmelding().is_none());
    }

    #[test]
    fn boven_het_budget_is_een_ontwerpdefect() {
        let mut b = Waarschuwingsbudget::nieuw();
        for dag in 12..=18 {
            b.onderbreking("u1", t(dag));
        }
        let stand = b.stand("u1", t(18));
        assert!(stand.overschreden);
        let melding = stand.defectmelding().unwrap();
        assert!(melding.contains("ontwerpdefect"));
        assert!(melding.contains("wegklikken"));
        assert!(!melding.contains("de gebruiker moet"), "de melding wijst niet naar de gebruiker");
    }

    #[test]
    fn oude_onderbrekingen_tellen_niet_mee() {
        let mut b = Waarschuwingsbudget::nieuw();
        for dag in 1..=10 {
            b.onderbreking("u1", t(dag));
        }
        assert_eq!(b.stand("u1", t(18)).deze_week, 0, "alles ligt meer dan zeven dagen terug");
    }

    #[test]
    fn overschrijdingen_worden_opgesomd() {
        let mut b = Waarschuwingsbudget::nieuw();
        for dag in 12..=18 {
            b.onderbreking("u1", t(dag));
        }
        b.onderbreking("u2", t(17));
        let lijst = b.overschrijdingen(t(18));
        assert_eq!(lijst.len(), 1);
        assert_eq!(lijst[0].gebruiker, "u1");
    }

    #[test]
    fn een_genegeerde_regel_gaat_in_de_review() {
        let mut b = Waarschuwingsbudget::nieuw();
        for _ in 0..100 {
            b.getoond("REG-07");
        }
        for _ in 0..10 {
            b.opgevolgd("REG-07");
        }
        assert!((b.negeerpercentage("REG-07").unwrap() - 0.90).abs() < 0.001);

        for _ in 0..100 {
            b.getoond("LEK-02");
        }
        for _ in 0..95 {
            b.opgevolgd("LEK-02");
        }

        let review = b.regels_voor_review();
        assert_eq!(review.len(), 1);
        assert_eq!(review[0].0, "REG-07");
    }

    #[test]
    fn een_onbekende_regel_heeft_geen_percentage() {
        let b = Waarschuwingsbudget::nieuw();
        assert!(b.negeerpercentage("BESTAAT-NIET").is_none());
    }
}
