//! De belangenafweging bij een gerechtvaardigd belang.
//!
//! # Waarom dit een dossier is en geen vinkje
//!
//! Artikel 6 lid 1 onder f is de enige grondslag waarbij de verwerking rust op
//! een afweging in plaats van op een feit. Toestemming is gegeven of niet, een
//! wettelijke verplichting bestaat of niet — maar een gerechtvaardigd belang
//! *weegt* tegen de belangen en de grondrechten van de betrokkene, en die
//! weging is de grondslag zelf. Wie haar niet opschrijft, heeft geen grondslag
//! maar een bewering.
//!
//! Daarom kent dit dossier vier onderdelen die alle vier moeten worden
//! ingevuld, en niet één veld met vrije tekst:
//!
//! 1. **Het belang.** Welk belang wordt behartigd, en van wie.
//! 2. **De noodzaak.** Kan het doel ook worden bereikt met minder gegevens of
//!    met een minder ingrijpend middel? Zo ja, dan houdt de afweging hier op.
//! 3. **De afweging.** Wat staat er tegenover aan belangen en grondrechten van
//!    de betrokkene?
//! 4. **De redelijke verwachtingen.** Kon de betrokkene dit verwachten, gelet
//!    op zijn verhouding tot de verwerkingsverantwoordelijke?
//!
//! De waarborgen staan er los naast: zij kunnen de uitslag kantelen, maar zij
//! vervangen de afweging niet.
//!
//! # De tool weegt niet
//!
//! De uitkomst is een oordeel van een mens. Wat de tool afdwingt is dat de vier
//! onderdelen er zijn vóórdat er een uitkomst mag worden vastgelegd — een
//! conclusie die aan de redenering voorafgaat, is geen conclusie.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    basis::{Compartiment, Herkomst, Id, Motivering, Status},
    error::{DomeinFout, Resultaat},
    volledigheid::{Ontbrekend, Volledig},
};

/// De uitkomst van de afweging.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Afwegingsuitkomst {
    /// Het belang weegt op tegen de belangen van de betrokkene.
    BelangWeegtOp,
    /// Alleen met de vastgelegde waarborgen.
    BelangWeegtOpMetWaarborgen,
    /// De verwerking kan niet op deze grondslag rusten.
    BelangWeegtNietOp,
}

impl Afwegingsuitkomst {
    pub fn omschrijving(&self) -> &'static str {
        match self {
            Self::BelangWeegtOp => "het belang weegt op",
            Self::BelangWeegtOpMetWaarborgen => "het belang weegt op, met waarborgen",
            Self::BelangWeegtNietOp => "het belang weegt niet op",
        }
    }

    /// Of de verwerking op deze grondslag kan rusten.
    pub fn draagt_de_grondslag(&self) -> bool {
        !matches!(self, Self::BelangWeegtNietOp)
    }

    pub fn alle() -> [Self; 3] {
        [Self::BelangWeegtOp, Self::BelangWeegtOpMetWaarborgen, Self::BelangWeegtNietOp]
    }
}

/// Een belangenafweging bij één verwerking.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Belangenafweging {
    pub id: Id,
    pub kenmerk: String,
    pub omschrijving: String,
    pub status: Status,
    pub compartiment: Compartiment,
    pub herkomst: Herkomst,

    pub verwerking_id: Id,
    pub verwerking_kenmerk: String,

    /// Welk belang wordt behartigd, en van wie.
    pub gerechtvaardigd_belang: Option<String>,
    /// Kan het doel met minder worden bereikt?
    pub noodzakelijkheidstoets: Option<Motivering>,
    /// Wat staat er tegenover aan belangen en grondrechten?
    pub afweging: Option<Motivering>,
    /// Kon de betrokkene dit verwachten?
    pub redelijke_verwachtingen: Option<Motivering>,
    /// Maatregelen die de uitslag kunnen kantelen.
    pub waarborgen: Vec<String>,

    pub uitkomst: Option<Afwegingsuitkomst>,
    pub uitgevoerd_door: Option<String>,
    pub datum: Option<DateTime<Utc>>,
}

impl Belangenafweging {
    pub fn nieuw(
        kenmerk: impl Into<String>,
        omschrijving: impl Into<String>,
        verwerking_id: Id,
        verwerking_kenmerk: impl Into<String>,
        door: impl Into<String>,
        op: DateTime<Utc>,
    ) -> Self {
        Self {
            id: Id::nieuw(),
            kenmerk: kenmerk.into(),
            omschrijving: omschrijving.into(),
            status: Status::Concept,
            compartiment: Compartiment::algemeen(),
            herkomst: Herkomst::nieuw(door, op),
            verwerking_id,
            verwerking_kenmerk: verwerking_kenmerk.into(),
            gerechtvaardigd_belang: None,
            noodzakelijkheidstoets: None,
            afweging: None,
            redelijke_verwachtingen: None,
            waarborgen: Vec::new(),
            uitkomst: None,
            uitgevoerd_door: None,
            datum: None,
        }
    }

    /// Of de vier onderdelen van de redenering er zijn.
    pub fn redenering_is_compleet(&self) -> bool {
        self.gerechtvaardigd_belang.is_some()
            && self.noodzakelijkheidstoets.is_some()
            && self.afweging.is_some()
            && self.redelijke_verwachtingen.is_some()
    }

    /// Legt de uitkomst vast.
    ///
    /// Kan pas nadat alle vier de onderdelen er zijn: een conclusie die aan de
    /// redenering voorafgaat, is geen conclusie.
    pub fn stel_uitkomst_vast(
        &mut self,
        uitkomst: Afwegingsuitkomst,
        uitgevoerd_door: impl Into<String>,
        datum: DateTime<Utc>,
        op: DateTime<Utc>,
    ) -> Resultaat<()> {
        if datum > op {
            return Err(DomeinFout::OnmogelijkTijdstip {
                veld: "lia.datum".into(),
                reden: "de afweging zou in de toekomst zijn uitgevoerd; controleer de datum".into(),
            });
        }
        if !self.redenering_is_compleet() {
            let mut ontbreekt = Vec::new();
            if self.gerechtvaardigd_belang.is_none() {
                ontbreekt.push("het belang dat wordt behartigd".to_string());
            }
            if self.noodzakelijkheidstoets.is_none() {
                ontbreekt.push("de noodzakelijkheidstoets".to_string());
            }
            if self.afweging.is_none() {
                ontbreekt.push("de afweging tegen de belangen van de betrokkene".to_string());
            }
            if self.redelijke_verwachtingen.is_none() {
                ontbreekt.push("de redelijke verwachtingen van de betrokkene".to_string());
            }
            return Err(DomeinFout::NietVolledig { soort: "belangenafweging".into(), ontbreekt });
        }
        if uitkomst == Afwegingsuitkomst::BelangWeegtOpMetWaarborgen && self.waarborgen.is_empty() {
            return Err(DomeinFout::OngeldigeWaarde {
                veld: "lia.waarborgen".into(),
                reden: "deze uitkomst berust op waarborgen; benoem er ten minste één, anders \
                        berust zij nergens op"
                    .into(),
            });
        }
        let door = uitgevoerd_door.into();
        if door.trim().is_empty() {
            return Err(DomeinFout::OngeldigeWaarde {
                veld: "lia.uitgevoerd_door".into(),
                reden: "noteer wie de afweging heeft gemaakt".into(),
            });
        }
        self.uitkomst = Some(uitkomst);
        self.uitgevoerd_door = Some(door);
        self.datum = Some(datum);
        self.status = Status::Vastgesteld;
        self.herkomst.stel_vast(self.herkomst.aangemaakt_door.clone(), op);
        Ok(())
    }

    /// Markeert de afweging als te herzien.
    pub fn markeer_herziening_nodig(&mut self, reden: impl Into<String>, op: DateTime<Utc>) {
        if self.status == Status::Vastgesteld {
            self.status = Status::HerzieningNodig;
        }
        self.herkomst.wijzig(format!("systeem: {}", reden.into()), op);
    }
}

impl Volledig for Belangenafweging {
    fn soortnaam(&self) -> &'static str {
        "belangenafweging"
    }

    fn aantal_verplichte_onderdelen(&self) -> usize {
        // De vier onderdelen van de redenering, plus de uitkomst met de naam
        // van wie haar heeft gemaakt.
        6
    }

    fn ontbrekende_onderdelen(&self) -> Vec<Ontbrekend> {
        let mut uit = Vec::new();
        if self.gerechtvaardigd_belang.is_none() {
            uit.push(Ontbrekend::blokkerend(
                "lia.gerechtvaardigd_belang",
                "benoem welk belang wordt behartigd, en van wie",
                "art. 6 lid 1 onder f AVG",
            ));
        }
        if self.noodzakelijkheidstoets.is_none() {
            uit.push(Ontbrekend::blokkerend(
                "lia.noodzakelijkheidstoets",
                "toets of het doel ook met minder gegevens of een minder ingrijpend middel kan \
                 worden bereikt",
                "art. 6 lid 1 onder f AVG; noodzakelijkheid",
            ));
        }
        if self.afweging.is_none() {
            uit.push(Ontbrekend::blokkerend(
                "lia.afweging",
                "weeg het belang tegen de belangen en grondrechten van de betrokkene",
                "art. 6 lid 1 onder f AVG",
            ));
        }
        if self.redelijke_verwachtingen.is_none() {
            uit.push(Ontbrekend::blokkerend(
                "lia.redelijke_verwachtingen",
                "beoordeel of de betrokkene deze verwerking redelijkerwijs kon verwachten",
                "overweging 47 AVG",
            ));
        }
        if self.uitkomst.is_none() {
            uit.push(Ontbrekend::blokkerend(
                "lia.uitkomst",
                "leg de uitkomst van de afweging vast",
                "art. 5 lid 2 AVG",
            ));
        }
        if self.uitgevoerd_door.is_none() {
            uit.push(Ontbrekend::blokkerend(
                "lia.uitgevoerd_door",
                "noteer wie de afweging heeft gemaakt",
                "art. 5 lid 2 AVG",
            ));
        }
        uit
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn nu() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 8, 19, 9, 0, 0).unwrap()
    }

    fn motivering(tekst: &str) -> Motivering {
        Motivering::nieuw(tekst, "u1", nu()).unwrap()
    }

    fn afweging() -> Belangenafweging {
        Belangenafweging::nieuw(
            "LIA-0412",
            "cameratoezicht bij de ingang",
            Id::nieuw(),
            "0412-K",
            "u1",
            nu(),
        )
    }

    fn compleet() -> Belangenafweging {
        let mut a = afweging();
        a.gerechtvaardigd_belang = Some("het voorkomen van diefstal uit het magazijn".into());
        a.noodzakelijkheidstoets =
            Some(motivering("een sluitsysteem alleen bleek onvoldoende bij herhaalde inbraak"));
        a.afweging = Some(motivering(
            "de camera's richten zich op de ingang en niet op werkplekken; de inbreuk blijft \
             beperkt tot passanten",
        ));
        a.redelijke_verwachtingen =
            Some(motivering("bij een bedrijfsingang mag cameratoezicht worden verwacht"));
        a
    }

    #[test]
    fn een_lege_afweging_vraagt_om_zes_onderdelen() {
        let a = afweging();
        assert_eq!(a.volledigheid().verplicht, 6);
        assert_eq!(a.ontbrekende_onderdelen().len(), 6);
    }

    /// Een conclusie die aan de redenering voorafgaat, is geen conclusie.
    #[test]
    fn een_uitkomst_zonder_redenering_wordt_geweigerd() {
        let mut a = afweging();
        let fout = a
            .stel_uitkomst_vast(Afwegingsuitkomst::BelangWeegtOp, "A. de Vries", nu(), nu())
            .unwrap_err();
        let tekst = fout.to_string();
        assert!(tekst.contains("noodzakelijkheidstoets"), "kreeg: {tekst}");
        assert!(tekst.contains("redelijke verwachtingen"));
    }

    #[test]
    fn een_complete_redenering_draagt_een_uitkomst() {
        let mut a = compleet();
        a.stel_uitkomst_vast(Afwegingsuitkomst::BelangWeegtOp, "A. de Vries", nu(), nu()).unwrap();
        assert_eq!(a.status, Status::Vastgesteld);
        assert!(a.volledigheid().is_volledig());
        assert!(a.uitkomst.unwrap().draagt_de_grondslag());
    }

    /// Een uitkomst die op waarborgen berust, vergt waarborgen.
    #[test]
    fn met_waarborgen_zonder_waarborgen_wordt_geweigerd() {
        let mut a = compleet();
        let fout = a
            .stel_uitkomst_vast(
                Afwegingsuitkomst::BelangWeegtOpMetWaarborgen,
                "A. de Vries",
                nu(),
                nu(),
            )
            .unwrap_err();
        assert!(fout.to_string().contains("berust zij nergens op"));

        a.waarborgen.push("beelden worden na zeven dagen automatisch gewist".into());
        a.stel_uitkomst_vast(
            Afwegingsuitkomst::BelangWeegtOpMetWaarborgen,
            "A. de Vries",
            nu(),
            nu(),
        )
        .unwrap();
    }

    /// De tool weegt niet: ook een negatieve uitkomst wordt gewoon vastgelegd.
    #[test]
    fn een_negatieve_uitkomst_wordt_vastgelegd() {
        let mut a = compleet();
        a.stel_uitkomst_vast(Afwegingsuitkomst::BelangWeegtNietOp, "A. de Vries", nu(), nu())
            .unwrap();
        assert!(!a.uitkomst.unwrap().draagt_de_grondslag());
        assert!(a.volledigheid().is_volledig());
    }

    #[test]
    fn een_afweging_in_de_toekomst_wordt_geweigerd() {
        let mut a = compleet();
        let fout = a
            .stel_uitkomst_vast(
                Afwegingsuitkomst::BelangWeegtOp,
                "A. de Vries",
                nu() + chrono::Duration::days(1),
                nu(),
            )
            .unwrap_err();
        assert!(fout.to_string().contains("toekomst"));
    }

    #[test]
    fn de_afweging_overleeft_serialisatie() {
        let mut a = compleet();
        a.waarborgen.push("beelden worden na zeven dagen gewist".into());
        a.stel_uitkomst_vast(
            Afwegingsuitkomst::BelangWeegtOpMetWaarborgen,
            "A. de Vries",
            nu(),
            nu(),
        )
        .unwrap();

        let json = serde_json::to_string(&a).unwrap();
        let terug: Belangenafweging = serde_json::from_str(&json).unwrap();
        assert_eq!(a, terug);
    }
}
