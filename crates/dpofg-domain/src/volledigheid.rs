//! Onvolledigheid zichtbaar maken in plaats van verbergen.
//!
//! Dit is het mechanisme achter paragraaf 3.6 van het foutbestendigheidshoofd-
//! stuk. Uitgangspunt: een register dat "opgeslagen" meldt terwijl er drie
//! verplichte velden leeg zijn, liegt tegen zijn gebruiker. Daarom kan elk
//! record zeggen wat er nog ontbreekt, met de bepaling erbij waaruit die eis
//! volgt.
//!
//! Twee ontwerpregels die hier worden afgedwongen:
//!
//! 1. **Onvolledigheid is een teller, geen foutmelding.** "11 van de 14
//!    onderdelen compleet" is voortgang; "verplicht veld" is een verwijt. Het
//!    eerste nodigt uit om verder te gaan, het tweede om het scherm te sluiten.
//! 2. **Er bestaat geen weergave waarin het register completer lijkt dan het
//!    is.** Exports, rapportages en de weergave voor een toezichthouder tonen
//!    dezelfde teller. Het rapport is daarom onderdeel van het domeinmodel en
//!    niet van de interface.

use serde::{Deserialize, Serialize};

/// Eén ontbrekend onderdeel.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ontbrekend {
    /// Het veld of onderdeel, in puntnotatie: `verwerking.bewaartermijn`.
    pub veld: String,
    /// Wat er van de gebruiker wordt gevraagd, in gewone taal.
    pub omschrijving: String,
    /// De bepaling waaruit de eis volgt.
    pub grondslag: String,
    /// Of dit onderdeel vaststellen onmogelijk maakt.
    ///
    /// Blokkerend betekent: het record kan concept blijven, maar niet
    /// vastgesteld worden. Niet-blokkerend betekent: het record mag worden
    /// vastgesteld, maar het onderdeel blijft zichtbaar ontbreken.
    pub blokkeert_vaststelling: bool,
}

impl Ontbrekend {
    pub fn blokkerend(
        veld: impl Into<String>,
        omschrijving: impl Into<String>,
        grondslag: impl Into<String>,
    ) -> Self {
        Self {
            veld: veld.into(),
            omschrijving: omschrijving.into(),
            grondslag: grondslag.into(),
            blokkeert_vaststelling: true,
        }
    }

    pub fn signalerend(
        veld: impl Into<String>,
        omschrijving: impl Into<String>,
        grondslag: impl Into<String>,
    ) -> Self {
        Self {
            veld: veld.into(),
            omschrijving: omschrijving.into(),
            grondslag: grondslag.into(),
            blokkeert_vaststelling: false,
        }
    }
}

/// De volledigheid van één record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Volledigheidsrapport {
    /// Soort record, bijvoorbeeld `verwerking`.
    pub soort: String,
    /// Aantal onderdelen dat verplicht is.
    pub verplicht: usize,
    /// Aantal daarvan dat is ingevuld.
    pub compleet: usize,
    /// Wat er nog ontbreekt.
    pub ontbreekt: Vec<Ontbrekend>,
}

impl Volledigheidsrapport {
    pub fn nieuw(soort: impl Into<String>, verplicht: usize, ontbreekt: Vec<Ontbrekend>) -> Self {
        let compleet = verplicht.saturating_sub(ontbreekt.len());
        Self { soort: soort.into(), verplicht, compleet, ontbreekt }
    }

    /// Of alles is ingevuld.
    pub fn is_volledig(&self) -> bool {
        self.ontbreekt.is_empty()
    }

    /// Of het record mag worden vastgesteld.
    pub fn mag_vaststellen(&self) -> bool {
        !self.ontbreekt.iter().any(|o| o.blokkeert_vaststelling)
    }

    /// De onderdelen die vaststellen tegenhouden.
    pub fn blokkades(&self) -> Vec<&Ontbrekend> {
        self.ontbreekt.iter().filter(|o| o.blokkeert_vaststelling).collect()
    }

    /// Voortgang als percentage, afgerond naar beneden.
    pub fn percentage(&self) -> u8 {
        if self.verplicht == 0 {
            return 100;
        }
        ((self.compleet * 100) / self.verplicht) as u8
    }

    /// De teller zoals die in beeld hoort te staan.
    ///
    /// Bewust deze formulering en niet "3 fouten": het is voortgang, geen
    /// verwijt.
    pub fn teller(&self) -> String {
        format!("{} van de {} verplichte onderdelen", self.compleet, self.verplicht)
    }
}

/// Elk record dat verplichte onderdelen kent, kan zeggen wat er ontbreekt.
pub trait Volledig {
    /// Wat er nog ontbreekt.
    fn ontbrekende_onderdelen(&self) -> Vec<Ontbrekend>;

    /// Hoeveel onderdelen er in totaal verplicht zijn.
    fn aantal_verplichte_onderdelen(&self) -> usize;

    /// Soortaanduiding voor het rapport.
    fn soortnaam(&self) -> &'static str;

    /// Het volledige rapport.
    fn volledigheid(&self) -> Volledigheidsrapport {
        Volledigheidsrapport::nieuw(
            self.soortnaam(),
            self.aantal_verplichte_onderdelen(),
            self.ontbrekende_onderdelen(),
        )
    }

    /// Of het record mag worden vastgesteld.
    fn mag_vaststellen(&self) -> bool {
        self.volledigheid().mag_vaststellen()
    }
}

/// De volledigheid van een hele verzameling records.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Registerrapport {
    pub soort: String,
    pub totaal: usize,
    pub vastgesteld: usize,
    pub concept: usize,
    pub volledig: usize,
    /// Records met ten minste één blokkerend ontbrekend onderdeel.
    pub geblokkeerd: usize,
    /// Hoe vaak elk ontbrekend onderdeel voorkomt, aflopend gesorteerd.
    ///
    /// Dit is de lijst waarmee de gebruiker ziet wáár het structureel misgaat,
    /// in plaats van record voor record te moeten zoeken.
    pub ontbreekt_per_onderdeel: Vec<(String, usize)>,
}

impl Registerrapport {
    /// Stelt het rapport samen uit de losse volledigheidsrapporten.
    pub fn uit(
        soort: impl Into<String>,
        rapporten: &[(crate::Status, Volledigheidsrapport)],
    ) -> Self {
        let mut tellers: std::collections::BTreeMap<String, usize> = Default::default();
        let mut volledig = 0;
        let mut geblokkeerd = 0;
        let mut vastgesteld = 0;
        let mut concept = 0;

        for (status, rapport) in rapporten {
            match status {
                crate::Status::Vastgesteld => vastgesteld += 1,
                crate::Status::Concept => concept += 1,
                _ => {}
            }
            if rapport.is_volledig() {
                volledig += 1;
            }
            if !rapport.mag_vaststellen() {
                geblokkeerd += 1;
            }
            for o in &rapport.ontbreekt {
                *tellers.entry(o.veld.clone()).or_default() += 1;
            }
        }

        let mut ontbreekt_per_onderdeel: Vec<(String, usize)> = tellers.into_iter().collect();
        // Aflopend op aantal, daarna op naam voor een stabiele volgorde.
        ontbreekt_per_onderdeel.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));

        Self {
            soort: soort.into(),
            totaal: rapporten.len(),
            vastgesteld,
            concept,
            volledig,
            geblokkeerd,
            ontbreekt_per_onderdeel,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Status;

    fn rapport(ontbreekt: Vec<Ontbrekend>) -> Volledigheidsrapport {
        Volledigheidsrapport::nieuw("verwerking", 14, ontbreekt)
    }

    #[test]
    fn volledig_record() {
        let r = rapport(vec![]);
        assert!(r.is_volledig());
        assert!(r.mag_vaststellen());
        assert_eq!(r.compleet, 14);
        assert_eq!(r.percentage(), 100);
        assert_eq!(r.teller(), "14 van de 14 verplichte onderdelen");
    }

    #[test]
    fn blokkerend_onderdeel_houdt_vaststellen_tegen() {
        let r = rapport(vec![Ontbrekend::blokkerend(
            "verwerking.bewaartermijn",
            "leg de bewaartermijn vast",
            "art. 30 lid 1 onder f AVG",
        )]);
        assert!(!r.is_volledig());
        assert!(!r.mag_vaststellen());
        assert_eq!(r.blokkades().len(), 1);
        assert_eq!(r.compleet, 13);
    }

    #[test]
    fn signalerend_onderdeel_houdt_vaststellen_niet_tegen() {
        let r = rapport(vec![Ontbrekend::signalerend(
            "verwerking.archiefselectielijst",
            "verwijs naar de selectielijst",
            "Archiefwet",
        )]);
        assert!(!r.is_volledig());
        assert!(r.mag_vaststellen(), "signaleren mag vaststellen niet tegenhouden");
    }

    #[test]
    fn teller_is_voortgang_geen_verwijt() {
        let r = rapport(vec![
            Ontbrekend::blokkerend("a", "a", "x"),
            Ontbrekend::blokkerend("b", "b", "x"),
            Ontbrekend::signalerend("c", "c", "x"),
        ]);
        assert_eq!(r.teller(), "11 van de 14 verplichte onderdelen");
        assert_eq!(r.percentage(), 78);
        assert!(!r.teller().contains("fout"));
        assert!(!r.teller().contains("verplicht veld"));
    }

    #[test]
    fn registerrapport_telt_en_sorteert() {
        let rapporten = vec![
            (
                Status::Vastgesteld,
                rapport(vec![Ontbrekend::signalerend("verwerking.bewaartermijn", "x", "y")]),
            ),
            (
                Status::Concept,
                rapport(vec![
                    Ontbrekend::blokkerend("verwerking.bewaartermijn", "x", "y"),
                    Ontbrekend::blokkerend("verwerking.grondslag", "x", "y"),
                ]),
            ),
            (Status::Vastgesteld, rapport(vec![])),
        ];
        let r = Registerrapport::uit("verwerkingsregister", &rapporten);

        assert_eq!(r.totaal, 3);
        assert_eq!(r.vastgesteld, 2);
        assert_eq!(r.concept, 1);
        assert_eq!(r.volledig, 1);
        assert_eq!(r.geblokkeerd, 1);
        assert_eq!(
            r.ontbreekt_per_onderdeel,
            vec![("verwerking.bewaartermijn".to_string(), 2), ("verwerking.grondslag".to_string(), 1)]
        );
    }

    #[test]
    fn leeg_register_is_hanteerbaar() {
        let r = Registerrapport::uit("verwerkingsregister", &[]);
        assert_eq!(r.totaal, 0);
        assert!(r.ontbreekt_per_onderdeel.is_empty());
    }
}
