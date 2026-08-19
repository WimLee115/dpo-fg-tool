//! Het spoor van de Wet politiegegevens.
//!
//! # Waarom dit naast de AVG staat en er niet in past
//!
//! Voor politiegegevens geldt een eigen regime met een eigen
//! verantwoordingsplicht. De kern daarvan is niet een register maar een
//! **controlecyclus**: jaarlijks intern controleren, vierjaarlijks extern laten
//! auditen, en bij bevindingen een verbeterplan opstellen en afwerken.
//!
//! Dat is een ander soort verplichting dan de AVG kent. Zij loopt door zonder
//! dat er een verzoek, een incident of een wijziging aan te pas komt: de klok
//! tikt vanzelf. Wie hem niet bijhoudt, ontdekt bij de eerstvolgende audit dat
//! de vorige vier jaar geleden was.
//!
//! # Wat dit dossier afdwingt
//!
//! * **Van toepassing of niet is een besluit met een motivering.** Dat het
//!   regime niet geldt, is even goed een standpunt als dat het wel geldt, en
//!   het hoort te worden verantwoord.
//! * **Een audit met bevindingen vraagt een verbeterplan.** Een rapport
//!   opbergen zonder plan is de meest voorkomende manier waarop een audit geen
//!   gevolg krijgt.
//! * **Een verbeterplan zonder einddatum wordt niet vastgelegd.** Een maatregel
//!   zonder datum is een voornemen.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    basis::{Compartiment, Herkomst, Id, Motivering, Status},
    error::{DomeinFout, Resultaat},
    volledigheid::{Ontbrekend, Volledig},
};

/// Eén uitgevoerde controle of audit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Controle {
    pub uitgevoerd_op: DateTime<Utc>,
    pub uitvoerder: String,
    /// Het kenmerk waaronder het rapport is opgeborgen.
    pub rapport_kenmerk: Option<String>,
    /// Hoeveel bevindingen de controle opleverde.
    pub bevindingen: usize,
    pub toelichting: Option<String>,
}

/// Eén maatregel uit het verbeterplan.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Maatregel {
    pub omschrijving: String,
    pub eigenaar: String,
    pub gereed_uiterlijk: DateTime<Utc>,
    pub afgerond_op: Option<DateTime<Utc>>,
}

impl Maatregel {
    pub fn is_afgerond(&self) -> bool {
        self.afgerond_op.is_some()
    }

    pub fn is_verlopen(&self, nu: DateTime<Utc>) -> bool {
        !self.is_afgerond() && nu > self.gereed_uiterlijk
    }
}

/// Het verbeterplan dat op een audit met bevindingen volgt.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Verbeterplan {
    pub vastgesteld_op: DateTime<Utc>,
    pub vastgesteld_door: String,
    pub maatregelen: Vec<Maatregel>,
}

impl Verbeterplan {
    pub fn openstaand(&self) -> usize {
        self.maatregelen.iter().filter(|m| !m.is_afgerond()).count()
    }

    pub fn verlopen(&self, nu: DateTime<Utc>) -> Vec<&Maatregel> {
        self.maatregelen.iter().filter(|m| m.is_verlopen(nu)).collect()
    }
}

/// Het Wpg-spoor van één organisatie.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Wpgspoor {
    pub id: Id,
    pub kenmerk: String,
    pub omschrijving: String,
    pub status: Status,
    pub compartiment: Compartiment,
    pub herkomst: Herkomst,

    /// Of het regime van toepassing is. `None` betekent: nog niet beoordeeld.
    pub van_toepassing: Option<bool>,
    pub van_toepassing_motivering: Option<Motivering>,

    pub interne_controles: Vec<Controle>,
    pub externe_audits: Vec<Controle>,
    pub verbeterplan: Option<Verbeterplan>,
}

impl Wpgspoor {
    pub fn nieuw(
        kenmerk: impl Into<String>,
        omschrijving: impl Into<String>,
        door: impl Into<String>,
        op: DateTime<Utc>,
    ) -> Self {
        Self {
            id: Id::nieuw(),
            kenmerk: kenmerk.into(),
            omschrijving: omschrijving.into(),
            status: Status::Concept,
            compartiment: Compartiment::nieuw(Compartiment::VERTROUWELIJK),
            herkomst: Herkomst::nieuw(door, op),
            van_toepassing: None,
            van_toepassing_motivering: None,
            interne_controles: Vec::new(),
            externe_audits: Vec::new(),
            verbeterplan: None,
        }
    }

    /// De laatste externe audit, als die er is.
    pub fn laatste_audit(&self) -> Option<&Controle> {
        self.externe_audits.iter().max_by_key(|c| c.uitgevoerd_op)
    }

    /// De laatste interne controle, als die er is.
    pub fn laatste_controle(&self) -> Option<&Controle> {
        self.interne_controles.iter().max_by_key(|c| c.uitgevoerd_op)
    }

    /// Hoeveel maanden er sinds de laatste externe audit zijn verstreken.
    ///
    /// Ruwe maat op dertig dagen, gelijk aan die van de effectbeoordeling: een
    /// vierjaarlijkse verplichting hoeft niet op de dag nauwkeurig te zijn, en
    /// een exacte berekening zou een feestdagenkalender vergen die vier jaar
    /// vooruit reikt.
    pub fn maanden_sinds_audit(&self, nu: DateTime<Utc>) -> Option<i64> {
        self.laatste_audit().map(|c| (nu - c.uitgevoerd_op).num_days() / 30)
    }

    pub fn maanden_sinds_controle(&self, nu: DateTime<Utc>) -> Option<i64> {
        self.laatste_controle().map(|c| (nu - c.uitgevoerd_op).num_days() / 30)
    }

    /// Legt vast of het regime van toepassing is.
    pub fn stel_toepasselijkheid_vast(
        &mut self,
        van_toepassing: bool,
        motivering: Motivering,
        op: DateTime<Utc>,
    ) -> Resultaat<()> {
        self.van_toepassing = Some(van_toepassing);
        self.van_toepassing_motivering = Some(motivering);
        self.herkomst.wijzig("toepasselijkheid vastgesteld", op);
        Ok(())
    }

    /// Legt een uitgevoerde controle of audit vast.
    pub fn leg_controle_vast(
        &mut self,
        extern_uitgevoerd: bool,
        controle: Controle,
        op: DateTime<Utc>,
    ) -> Resultaat<()> {
        if controle.uitgevoerd_op > op {
            return Err(DomeinFout::OnmogelijkTijdstip {
                veld: "wpg.controle".into(),
                reden: "de controle zou in de toekomst zijn uitgevoerd; controleer de datum".into(),
            });
        }
        if controle.uitvoerder.trim().is_empty() {
            return Err(DomeinFout::OngeldigeWaarde {
                veld: "wpg.controle".into(),
                reden: "noteer wie de controle heeft uitgevoerd; bij een externe audit is dat de \
                        onafhankelijkheid zelf"
                    .into(),
            });
        }
        if extern_uitgevoerd {
            self.externe_audits.push(controle);
            self.herkomst.wijzig("externe audit vastgelegd", op);
        } else {
            self.interne_controles.push(controle);
            self.herkomst.wijzig("interne controle vastgelegd", op);
        }
        Ok(())
    }

    /// Stelt het verbeterplan vast.
    pub fn stel_verbeterplan_vast(
        &mut self,
        vastgesteld_door: impl Into<String>,
        maatregelen: Vec<Maatregel>,
        moment: DateTime<Utc>,
        op: DateTime<Utc>,
    ) -> Resultaat<()> {
        if maatregelen.is_empty() {
            return Err(DomeinFout::OngeldigeWaarde {
                veld: "wpg.verbeterplan".into(),
                reden: "een verbeterplan zonder maatregelen is geen plan".into(),
            });
        }
        if let Some(m) = maatregelen.iter().find(|m| m.eigenaar.trim().is_empty()) {
            return Err(DomeinFout::OngeldigeWaarde {
                veld: "wpg.verbeterplan".into(),
                reden: format!(
                    "maatregel '{}' heeft geen eigenaar; een maatregel zonder eigenaar wordt door \
                     niemand uitgevoerd",
                    m.omschrijving
                ),
            });
        }
        self.verbeterplan = Some(Verbeterplan {
            vastgesteld_op: moment,
            vastgesteld_door: vastgesteld_door.into(),
            maatregelen,
        });
        self.herkomst.wijzig("verbeterplan vastgesteld", op);
        Ok(())
    }

    /// Rondt één maatregel af.
    pub fn rond_maatregel_af(
        &mut self,
        omschrijving: &str,
        moment: DateTime<Utc>,
        op: DateTime<Utc>,
    ) -> Resultaat<()> {
        if moment > op {
            return Err(DomeinFout::OnmogelijkTijdstip {
                veld: "wpg.verbeterplan".into(),
                reden: "de afronding zou in de toekomst liggen".into(),
            });
        }
        let plan = self.verbeterplan.as_mut().ok_or_else(|| DomeinFout::OntbrekendeVerwijzing {
            veld: "wpg.verbeterplan".into(),
            naar: "een vastgesteld verbeterplan".into(),
        })?;
        let m = plan.maatregelen.iter_mut().find(|m| m.omschrijving == omschrijving).ok_or_else(
            || DomeinFout::OntbrekendeVerwijzing {
                veld: "wpg.verbeterplan".into(),
                naar: format!("een maatregel '{omschrijving}'"),
            },
        )?;
        m.afgerond_op = Some(moment);
        self.herkomst.wijzig(format!("maatregel '{omschrijving}' afgerond"), op);
        Ok(())
    }

    /// Of er een audit met bevindingen ligt waarop nog geen plan volgde.
    pub fn audit_zonder_plan(&self) -> bool {
        self.laatste_audit().is_some_and(|a| a.bevindingen > 0) && self.verbeterplan.is_none()
    }
}

impl Volledig for Wpgspoor {
    fn soortnaam(&self) -> &'static str {
        "Wpg-spoor"
    }

    fn aantal_verplichte_onderdelen(&self) -> usize {
        // De toepasselijkheid en haar motivering staan er altijd.
        let vast = 2;
        // Geldt het regime niet, dan is het dossier daarmee klaar.
        if self.van_toepassing == Some(false) || self.van_toepassing.is_none() {
            return vast;
        }
        // Anders: een interne controle, een externe audit, en bij bevindingen
        // een verbeterplan.
        let mut afgeleid = 2;
        if self.laatste_audit().is_some_and(|a| a.bevindingen > 0) {
            afgeleid += 1;
        }
        vast + afgeleid
    }

    fn ontbrekende_onderdelen(&self) -> Vec<Ontbrekend> {
        let mut uit = Vec::new();

        if self.van_toepassing.is_none() {
            uit.push(Ontbrekend::blokkerend(
                "wpg.van_toepassing",
                "beoordeel of het regime van de Wet politiegegevens van toepassing is",
                "art. 1 Wet politiegegevens",
            ));
        }
        if self.van_toepassing_motivering.is_none() {
            uit.push(Ontbrekend::blokkerend(
                "wpg.van_toepassing_motivering",
                "schrijf op waaróm het regime wel of niet geldt",
                "art. 5 lid 2 AVG; verantwoordingsplicht",
            ));
        }

        if self.van_toepassing != Some(true) {
            return uit;
        }

        if self.interne_controles.is_empty() {
            uit.push(Ontbrekend::blokkerend(
                "wpg.interne_controle",
                "leg de jaarlijkse interne controle vast",
                "art. 33 lid 1 Wet politiegegevens",
            ));
        }
        if self.externe_audits.is_empty() {
            uit.push(Ontbrekend::blokkerend(
                "wpg.externe_audit",
                "leg de vierjaarlijkse externe audit vast",
                "art. 33 lid 3 Wet politiegegevens",
            ));
        }
        if self.audit_zonder_plan() {
            uit.push(Ontbrekend::blokkerend(
                "wpg.verbeterplan",
                "de laatste audit leverde bevindingen op; stel een verbeterplan vast met een \
                 eigenaar en een einddatum per maatregel",
                "art. 33 lid 4 Wet politiegegevens",
            ));
        }

        uit
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, TimeZone};

    fn nu() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 8, 19, 9, 0, 0).unwrap()
    }

    fn motivering(tekst: &str) -> Motivering {
        Motivering::nieuw(tekst, "u1", nu()).unwrap()
    }

    fn spoor() -> Wpgspoor {
        Wpgspoor::nieuw("WPG-2026", "handhaving openbare orde", "u1", nu())
    }

    fn controle(bevindingen: usize) -> Controle {
        Controle {
            uitgevoerd_op: nu(),
            uitvoerder: "de interne auditdienst".into(),
            rapport_kenmerk: Some("AUD-2026-01".into()),
            bevindingen,
            toelichting: None,
        }
    }

    fn maatregel(naam: &str) -> Maatregel {
        Maatregel {
            omschrijving: naam.into(),
            eigenaar: "de teamleider".into(),
            gereed_uiterlijk: nu() + Duration::days(90),
            afgerond_op: None,
        }
    }

    #[test]
    fn een_leeg_spoor_vraagt_om_de_toepasselijkheid() {
        let s = spoor();
        assert_eq!(s.volledigheid().verplicht, 2);
        assert_eq!(s.ontbrekende_onderdelen().len(), 2);
    }

    /// Dat het regime niet geldt, is even goed een standpunt.
    #[test]
    fn niet_van_toepassing_sluit_het_dossier_met_een_motivering() {
        let mut s = spoor();
        s.stel_toepasselijkheid_vast(
            false,
            motivering("de organisatie verwerkt geen politiegegevens"),
            nu(),
        )
        .unwrap();
        assert!(s.volledigheid().is_volledig());
    }

    #[test]
    fn van_toepassing_vraagt_om_controle_en_audit() {
        let mut s = spoor();
        s.stel_toepasselijkheid_vast(true, motivering("de boa's verwerken politiegegevens"), nu())
            .unwrap();
        let velden: Vec<_> = s.ontbrekende_onderdelen().into_iter().map(|o| o.veld).collect();
        assert!(velden.contains(&"wpg.interne_controle".to_string()));
        assert!(velden.contains(&"wpg.externe_audit".to_string()));
    }

    /// Een rapport opbergen zonder plan is de manier waarop een audit geen
    /// gevolg krijgt.
    #[test]
    fn een_audit_met_bevindingen_vraagt_een_verbeterplan() {
        let mut s = spoor();
        s.stel_toepasselijkheid_vast(true, motivering("de boa's verwerken politiegegevens"), nu())
            .unwrap();
        s.leg_controle_vast(false, controle(0), nu()).unwrap();
        s.leg_controle_vast(true, controle(3), nu()).unwrap();

        assert!(s.audit_zonder_plan());
        let velden: Vec<_> = s.ontbrekende_onderdelen().into_iter().map(|o| o.veld).collect();
        assert!(velden.contains(&"wpg.verbeterplan".to_string()));

        s.stel_verbeterplan_vast("de directie", vec![maatregel("logging aanzetten")], nu(), nu())
            .unwrap();
        assert!(!s.audit_zonder_plan());
        assert!(s.volledigheid().is_volledig());
    }

    #[test]
    fn een_audit_zonder_bevindingen_vraagt_geen_plan() {
        let mut s = spoor();
        s.stel_toepasselijkheid_vast(true, motivering("de boa's verwerken politiegegevens"), nu())
            .unwrap();
        s.leg_controle_vast(false, controle(0), nu()).unwrap();
        s.leg_controle_vast(true, controle(0), nu()).unwrap();
        assert!(s.volledigheid().is_volledig());
    }

    #[test]
    fn een_maatregel_zonder_eigenaar_wordt_geweigerd() {
        let mut s = spoor();
        let mut m = maatregel("logging aanzetten");
        m.eigenaar = "  ".into();
        let fout = s.stel_verbeterplan_vast("de directie", vec![m], nu(), nu()).unwrap_err();
        assert!(fout.to_string().contains("door niemand uitgevoerd"));
    }

    #[test]
    fn een_leeg_verbeterplan_wordt_geweigerd() {
        let mut s = spoor();
        assert!(s.stel_verbeterplan_vast("de directie", Vec::new(), nu(), nu()).is_err());
    }

    #[test]
    fn een_verlopen_maatregel_is_zichtbaar() {
        let mut s = spoor();
        s.stel_verbeterplan_vast("de directie", vec![maatregel("logging aanzetten")], nu(), nu())
            .unwrap();
        let plan = s.verbeterplan.as_ref().unwrap();
        assert!(plan.verlopen(nu()).is_empty());
        assert_eq!(plan.verlopen(nu() + Duration::days(100)).len(), 1);

        s.rond_maatregel_af(
            "logging aanzetten",
            nu() + Duration::days(30),
            nu() + Duration::days(30),
        )
        .unwrap();
        assert_eq!(s.verbeterplan.as_ref().unwrap().openstaand(), 0);
    }

    #[test]
    fn de_maanden_sinds_de_audit_zijn_te_bepalen() {
        let mut s = spoor();
        let lang_geleden =
            Controle { uitgevoerd_op: nu() - Duration::days(30 * 50), ..controle(0) };
        s.leg_controle_vast(true, lang_geleden, nu()).unwrap();
        assert_eq!(s.maanden_sinds_audit(nu()), Some(50));
    }

    #[test]
    fn een_controle_in_de_toekomst_wordt_geweigerd() {
        let mut s = spoor();
        let vooruit = Controle { uitgevoerd_op: nu() + Duration::days(1), ..controle(0) };
        assert!(s.leg_controle_vast(false, vooruit, nu()).is_err());
    }

    #[test]
    fn het_spoor_overleeft_serialisatie() {
        let mut s = spoor();
        s.stel_toepasselijkheid_vast(true, motivering("de boa's verwerken politiegegevens"), nu())
            .unwrap();
        s.leg_controle_vast(true, controle(2), nu()).unwrap();
        s.stel_verbeterplan_vast("de directie", vec![maatregel("logging aanzetten")], nu(), nu())
            .unwrap();

        let json = serde_json::to_string(&s).unwrap();
        let terug: Wpgspoor = serde_json::from_str(&json).unwrap();
        assert_eq!(s, terug);
    }
}
