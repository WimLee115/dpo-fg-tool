//! De feestdagenkalender.
//!
//! Deze kalender staat **niet** in de programmacode maar wordt als gegeven
//! geladen (ontwerpprincipe P1 uit het plan). Reden: feestdagen verschillen per
//! land, veranderen per jaar en zijn soms pas laat bekend. Een kalender in de
//! binary betekent dat een nieuwe uitgave van het programma nodig is om een
//! termijn juist te berekenen — en dat is precies het soort afhankelijkheid dat
//! een gemiste wettelijke termijn oplevert.
//!
//! De kalender kent daarom een **dekkingsvenster**. Wordt een termijn berekend
//! die buiten dat venster valt, dan is de uitkomst niet stilzwijgend fout maar
//! wordt hij als onbetrouwbaar gemarkeerd.

use std::collections::BTreeSet;

use chrono::{Datelike, NaiveDate, Weekday};
use serde::{Deserialize, Serialize};

use crate::{Resultaat, TermijnFout};

/// Een verzameling algemeen erkende feestdagen voor één rechtsgebied.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Feestdagenkalender {
    /// Rechtsgebied, bijvoorbeeld `NL`.
    pub jurisdictie: String,
    /// Eerste jaar dat volledig gedekt is.
    pub dekking_van: i32,
    /// Laatste jaar dat volledig gedekt is.
    pub dekking_tot_en_met: i32,
    /// Vindplaats van de bron, voor verantwoording in het dossier.
    pub bron: String,
    /// De feestdagen zelf.
    pub dagen: BTreeSet<NaiveDate>,
}

impl Feestdagenkalender {
    /// Maakt een lege kalender waarin alleen weekenden niet-werkdagen zijn.
    pub fn leeg(jurisdictie: impl Into<String>, van: i32, tot_en_met: i32) -> Self {
        Self {
            jurisdictie: jurisdictie.into(),
            dekking_van: van,
            dekking_tot_en_met: tot_en_met,
            bron: "geen feestdagen vastgelegd".into(),
            dagen: BTreeSet::new(),
        }
    }

    /// Geeft aan of een datum binnen het dekkingsvenster valt.
    pub fn dekt(&self, datum: NaiveDate) -> bool {
        datum.year() >= self.dekking_van && datum.year() <= self.dekking_tot_en_met
    }

    /// Geeft aan of een datum een algemeen erkende feestdag is.
    pub fn is_feestdag(&self, datum: NaiveDate) -> bool {
        self.dagen.contains(&datum)
    }

    /// Geeft aan of een datum in het weekend valt.
    pub fn is_weekend(datum: NaiveDate) -> bool {
        matches!(datum.weekday(), Weekday::Sat | Weekday::Sun)
    }

    /// Geeft aan of een datum een werkdag is.
    ///
    /// Let op: deze functie kijkt niet naar het dekkingsvenster. Buiten dat
    /// venster is de uitkomst niet betrouwbaar, want een feestdag die niet in
    /// de kalender staat, telt hier als werkdag. Gebruik daarom in de
    /// termijnberekening altijd [`Self::controleer_dekking`] vooraf.
    pub fn is_werkdag(&self, datum: NaiveDate) -> bool {
        !Self::is_weekend(datum) && !self.is_feestdag(datum)
    }

    /// Faalt wanneer een datum buiten het dekkingsvenster valt.
    ///
    /// Dit is de bewaking die voorkomt dat de motor stilzwijgend "geen
    /// verlenging nodig" meldt voor een dag waarvan hij niet weet of het een
    /// feestdag is. Zonder deze controle zou een onvolledig kennispakket een
    /// te vroege deadline opleveren zonder enig signaal.
    pub fn controleer_dekking(&self, datum: NaiveDate) -> Resultaat<()> {
        if self.dekt(datum) {
            Ok(())
        } else {
            Err(TermijnFout::KalenderDektNiet {
                datum: datum.to_string(),
                jurisdictie: self.jurisdictie.clone(),
                van: self.dekking_van,
                tot_en_met: self.dekking_tot_en_met,
            })
        }
    }

    /// Schuift door naar de eerstvolgende werkdag, of geeft de datum terug
    /// wanneer die al een werkdag is.
    ///
    /// Faalt wanneer de datum buiten het dekkingsvenster valt: dan is niet te
    /// zeggen of de eerstvolgende dag een feestdag is, en een gok op een
    /// wettelijke termijn is onaanvaardbaar.
    pub fn eerstvolgende_werkdag(&self, datum: NaiveDate) -> Resultaat<NaiveDate> {
        if !self.dekt(datum) {
            return Err(TermijnFout::KalenderDektNiet {
                datum: datum.to_string(),
                jurisdictie: self.jurisdictie.clone(),
                van: self.dekking_van,
                tot_en_met: self.dekking_tot_en_met,
            });
        }
        let mut d = datum;
        // Ruime bovengrens: een aaneengesloten reeks vrije dagen van meer dan
        // veertien dagen bestaat niet en wijst op een fout in de gegevens.
        for _ in 0..14 {
            if self.is_werkdag(d) {
                return Ok(d);
            }
            d = d.succ_opt().ok_or(TermijnFout::DatumBuitenBereik)?;
            if !self.dekt(d) {
                return Err(TermijnFout::KalenderDektNiet {
                    datum: d.to_string(),
                    jurisdictie: self.jurisdictie.clone(),
                    van: self.dekking_van,
                    tot_en_met: self.dekking_tot_en_met,
                });
            }
        }
        Err(TermijnFout::TeveelVrijeDagen(datum.to_string()))
    }

    /// Telt `aantal` werkdagen op bij een datum.
    ///
    /// De startdatum telt niet mee; het resultaat is altijd een werkdag.
    pub fn tel_werkdagen_op(&self, datum: NaiveDate, aantal: u32) -> Resultaat<NaiveDate> {
        let mut d = datum;
        let mut over = aantal;
        let mut stappen = 0u32;
        while over > 0 {
            d = d.succ_opt().ok_or(TermijnFout::DatumBuitenBereik)?;
            if !self.dekt(d) {
                return Err(TermijnFout::KalenderDektNiet {
                    datum: d.to_string(),
                    jurisdictie: self.jurisdictie.clone(),
                    van: self.dekking_van,
                    tot_en_met: self.dekking_tot_en_met,
                });
            }
            if self.is_werkdag(d) {
                over -= 1;
            }
            stappen += 1;
            if stappen > aantal.saturating_mul(4) + 30 {
                return Err(TermijnFout::TeveelVrijeDagen(datum.to_string()));
            }
        }
        Ok(d)
    }
}

#[cfg(test)]
pub(crate) mod testkalender {
    use super::*;

    /// Kalender met de Nederlandse algemeen erkende feestdagen voor 2026 en 2027.
    ///
    /// Uitsluitend voor tests. In het product komt deze verzameling uit het
    /// kennispakket.
    pub fn nl() -> Feestdagenkalender {
        let dagen: BTreeSet<NaiveDate> = [
            // 2026
            (2026, 1, 1),   // Nieuwjaarsdag
            (2026, 4, 3),   // Goede Vrijdag
            (2026, 4, 5),   // Eerste paasdag
            (2026, 4, 6),   // Tweede paasdag
            (2026, 4, 27),  // Koningsdag
            (2026, 5, 5),   // Bevrijdingsdag
            (2026, 5, 14),  // Hemelvaartsdag
            (2026, 5, 24),  // Eerste pinksterdag
            (2026, 5, 25),  // Tweede pinksterdag
            (2026, 12, 25), // Eerste kerstdag
            (2026, 12, 26), // Tweede kerstdag
            // 2027
            (2027, 1, 1),
            (2027, 3, 26),
            (2027, 3, 28),
            (2027, 3, 29),
            (2027, 4, 27),
            (2027, 5, 5),
            (2027, 5, 6),
            (2027, 5, 16),
            (2027, 5, 17),
            (2027, 12, 25),
            (2027, 12, 26),
        ]
        .into_iter()
        .map(|(j, m, d)| NaiveDate::from_ymd_opt(j, m, d).unwrap())
        .collect();

        Feestdagenkalender {
            jurisdictie: "NL".into(),
            dekking_van: 2026,
            dekking_tot_en_met: 2027,
            bron: "testkalender".into(),
            dagen,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(j: i32, m: u32, dag: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(j, m, dag).unwrap()
    }

    fn kalender() -> Feestdagenkalender {
        super::testkalender::nl()
    }

    #[test]
    fn weekend_is_geen_werkdag() {
        let k = kalender();
        assert!(!k.is_werkdag(d(2026, 8, 22))); // zaterdag
        assert!(!k.is_werkdag(d(2026, 8, 23))); // zondag
        assert!(k.is_werkdag(d(2026, 8, 24))); // maandag
    }

    #[test]
    fn feestdag_is_geen_werkdag() {
        let k = kalender();
        assert!(!k.is_werkdag(d(2026, 12, 25)));
        assert!(!k.is_werkdag(d(2026, 4, 27)));
    }

    #[test]
    fn eerstvolgende_werkdag_slaat_weekend_over() {
        let k = kalender();
        assert_eq!(k.eerstvolgende_werkdag(d(2026, 8, 22)).unwrap(), d(2026, 8, 24));
    }

    #[test]
    fn eerstvolgende_werkdag_slaat_kerst_en_weekend_over() {
        let k = kalender();
        // 25 en 26 december 2026 zijn vrijdag en zaterdag; 27 december is zondag.
        assert_eq!(k.eerstvolgende_werkdag(d(2026, 12, 25)).unwrap(), d(2026, 12, 28));
    }

    #[test]
    fn werkdag_blijft_ongewijzigd() {
        let k = kalender();
        assert_eq!(k.eerstvolgende_werkdag(d(2026, 8, 18)).unwrap(), d(2026, 8, 18));
    }

    #[test]
    fn buiten_dekking_faalt_zichtbaar() {
        let k = kalender();
        assert!(matches!(
            k.eerstvolgende_werkdag(d(2030, 1, 1)).unwrap_err(),
            TermijnFout::KalenderDektNiet { .. }
        ));
    }

    #[test]
    fn werkdagen_optellen() {
        let k = kalender();
        // Dinsdag 18 augustus 2026 + 3 werkdagen = vrijdag 21 augustus.
        assert_eq!(k.tel_werkdagen_op(d(2026, 8, 18), 3).unwrap(), d(2026, 8, 21));
        // + 4 werkdagen springt over het weekend heen naar maandag 24 augustus.
        assert_eq!(k.tel_werkdagen_op(d(2026, 8, 18), 4).unwrap(), d(2026, 8, 24));
    }
}
