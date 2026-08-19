//! Leveranciers en verwerkersovereenkomsten.
//!
//! # Waarom een vindplaats en niet een vinkje
//!
//! Artikel 28 lid 3 somt acht onderwerpen op die in de overeenkomst met een
//! verwerker moeten staan. De verleiding is om die acht af te vinken. Dat is
//! precies wat er misgaat: een vinkje zegt dat iemand ooit dacht dat het
//! geregeld was, en bij een uitvraag moet worden aangewezen wáár het staat.
//!
//! Daarom draagt elk onderdeel hier een **vindplaats** — artikel, bijlage,
//! paragraaf — en geldt een onderdeel zonder vindplaats als niet geregeld. Dat
//! is regel VWO-02, en het is de reden dat dit dossier meer werk is dan een
//! lijstje.
//!
//! # Drie dingen die vanzelf verlopen
//!
//! * **De meldtermijn van de verwerker.** Die staat in het contract, en hij is
//!   te lang zodra de organisatie er haar eigen tweeënzeventig uur niet meer
//!   mee haalt. Wie een verwerker vier dagen geeft om te melden, heeft zijn
//!   eigen termijn weggegeven.
//! * **De subverwerkerslijst.** Die verandert zonder dat iemand het merkt; de
//!   overeenkomst schrijft doorgaans voor dat wijzigingen worden gemeld, maar
//!   melden is iets anders dan controleren.
//! * **Het moment van tekenen.** Een overeenkomst die is getekend nadat de
//!   verwerking al liep, dekt de periode ervóór niet. Dat is een feit dat
//!   blijft staan; het wordt zichtbaar gemaakt en niet weggewerkt.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    basis::{Compartiment, Herkomst, Id, Motivering, Status},
    error::{DomeinFout, Resultaat},
    volledigheid::{Ontbrekend, Volledig},
};

/// De acht onderwerpen van artikel 28 lid 3.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Contracteis {
    /// Onder a: uitsluitend op gedocumenteerde instructies.
    Instructies,
    /// Onder b: geheimhoudingsplicht voor wie toegang heeft.
    Geheimhouding,
    /// Onder c: beveiligingsmaatregelen op grond van artikel 32.
    Beveiliging,
    /// Onder d: de voorwaarden voor het inschakelen van subverwerkers.
    Subverwerkers,
    /// Onder e: bijstand bij verzoeken van betrokkenen.
    BijstandVerzoeken,
    /// Onder f: bijstand bij beveiliging, datalekken en effectbeoordelingen.
    BijstandVerplichtingen,
    /// Onder g: wissen of teruggeven na afloop van de dienstverlening.
    WissenOfTeruggeven,
    /// Onder h: informatie beschikbaar stellen en aan audits meewerken.
    AuditsEnInformatie,
}

impl Contracteis {
    pub fn letter(&self) -> &'static str {
        match self {
            Self::Instructies => "a",
            Self::Geheimhouding => "b",
            Self::Beveiliging => "c",
            Self::Subverwerkers => "d",
            Self::BijstandVerzoeken => "e",
            Self::BijstandVerplichtingen => "f",
            Self::WissenOfTeruggeven => "g",
            Self::AuditsEnInformatie => "h",
        }
    }

    pub fn omschrijving(&self) -> &'static str {
        match self {
            Self::Instructies => "verwerkt uitsluitend op gedocumenteerde instructies",
            Self::Geheimhouding => "wie toegang heeft, is tot geheimhouding gehouden",
            Self::Beveiliging => "treft de beveiligingsmaatregelen van artikel 32",
            Self::Subverwerkers => "schakelt geen subverwerker in zonder toestemming",
            Self::BijstandVerzoeken => "verleent bijstand bij verzoeken van betrokkenen",
            Self::BijstandVerplichtingen => {
                "verleent bijstand bij beveiliging, datalekken en effectbeoordelingen"
            }
            Self::WissenOfTeruggeven => "wist of geeft de gegevens terug na afloop",
            Self::AuditsEnInformatie => "stelt informatie beschikbaar en werkt mee aan audits",
        }
    }

    pub fn grondslag(&self) -> String {
        format!("art. 28 lid 3 onder {} AVG", self.letter())
    }

    pub fn alle() -> [Self; 8] {
        [
            Self::Instructies,
            Self::Geheimhouding,
            Self::Beveiliging,
            Self::Subverwerkers,
            Self::BijstandVerzoeken,
            Self::BijstandVerplichtingen,
            Self::WissenOfTeruggeven,
            Self::AuditsEnInformatie,
        ]
    }
}

/// Waar één onderdeel in het contract staat.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Vindplaats {
    pub eis: Contracteis,
    /// Artikel, bijlage of paragraaf. Zonder deze aanduiding geldt het
    /// onderdeel als niet geregeld.
    pub aanduiding: String,
    pub toelichting: Option<String>,
}

/// Een subverwerker onder deze leverancier.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Subverwerker {
    pub naam: String,
    pub land: String,
    pub dienst: String,
}

/// Hoe belangrijk deze leverancier is voor de dienstverlening.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Kritikaliteit {
    Laag,
    Gemiddeld,
    Hoog,
}

impl Kritikaliteit {
    pub fn omschrijving(&self) -> &'static str {
        match self {
            Self::Laag => "laag",
            Self::Gemiddeld => "gemiddeld",
            Self::Hoog => "hoog",
        }
    }
}

/// De verwerkersovereenkomst met deze leverancier.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Verwerkersovereenkomst {
    pub kenmerk: String,
    pub ondertekend_op: DateTime<Utc>,
    /// Wanneer de verwerking feitelijk begon.
    ///
    /// Ligt dit vóór de ondertekening, dan dekt de overeenkomst die periode
    /// niet. Dat is regel VWO-13.
    pub verwerking_begon_op: Option<DateTime<Utc>>,
    /// Binnen hoeveel uur de verwerker een inbreuk moet melden.
    ///
    /// Staat er geen termijn in het contract, dan is dit `None` — en dat is
    /// erger dan een te lange termijn, want dan is er niets afgesproken.
    pub meldtermijn_uren: Option<u32>,
    pub vindplaatsen: Vec<Vindplaats>,
}

impl Verwerkersovereenkomst {
    /// De onderdelen van artikel 28 lid 3 zonder vindplaats.
    pub fn eisen_zonder_vindplaats(&self) -> Vec<Contracteis> {
        Contracteis::alle()
            .into_iter()
            .filter(|e| {
                !self.vindplaatsen.iter().any(|v| v.eis == *e && !v.aanduiding.trim().is_empty())
            })
            .collect()
    }

    /// Of de overeenkomst is getekend nadat de verwerking al liep.
    pub fn getekend_na_aanvang(&self) -> bool {
        self.verwerking_begon_op.is_some_and(|start| self.ondertekend_op > start)
    }
}

/// Een leverancier die persoonsgegevens verwerkt.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Leverancier {
    pub id: Id,
    pub kenmerk: String,
    pub naam: String,
    pub status: Status,
    pub compartiment: Compartiment,
    pub herkomst: Herkomst,

    pub land: String,
    pub kritikaliteit: Option<Kritikaliteit>,
    /// Of deze leverancier rechtstreeks aan de organisatie levert.
    pub is_rechtstreeks: bool,
    pub kvk_nummer: Option<String>,

    pub overeenkomst: Option<Verwerkersovereenkomst>,

    pub subverwerkers: Vec<Subverwerker>,
    /// Wanneer de subverwerkerslijst voor het laatst is gecontroleerd.
    ///
    /// Melden is iets anders dan controleren: dit is het moment waarop iemand
    /// de lijst daadwerkelijk heeft nagelopen.
    pub subverwerkers_gecontroleerd_op: Option<DateTime<Utc>>,

    /// Een besluit om deze leverancier te weren, met de reden.
    pub weringsbesluit: Option<Motivering>,
    pub weringsbesluit_uitgevoerd_op: Option<DateTime<Utc>>,
}

impl Leverancier {
    pub fn nieuw(
        kenmerk: impl Into<String>,
        naam: impl Into<String>,
        land: impl Into<String>,
        door: impl Into<String>,
        op: DateTime<Utc>,
    ) -> Self {
        Self {
            id: Id::nieuw(),
            kenmerk: kenmerk.into(),
            naam: naam.into(),
            status: Status::Concept,
            compartiment: Compartiment::algemeen(),
            herkomst: Herkomst::nieuw(door, op),
            land: land.into(),
            kritikaliteit: None,
            is_rechtstreeks: true,
            kvk_nummer: None,
            overeenkomst: None,
            subverwerkers: Vec::new(),
            subverwerkers_gecontroleerd_op: None,
            weringsbesluit: None,
            weringsbesluit_uitgevoerd_op: None,
        }
    }

    /// Hoeveel maanden er sinds de laatste subverwerkerscontrole zijn verstreken.
    pub fn maanden_sinds_subverwerkerscontrole(&self, nu: DateTime<Utc>) -> Option<i64> {
        self.subverwerkers_gecontroleerd_op.map(|d| (nu - d).num_days() / 30)
    }

    /// Of de meldtermijn in het contract te lang is.
    ///
    /// De drempel komt van de aanroeper: hoeveel uur nog werkbaar is, hangt af
    /// van de eigen meldtermijn en hoort in het kennispakket te staan.
    pub fn meldtermijn_te_lang(&self, drempel_uren: u32) -> bool {
        self.overeenkomst
            .as_ref()
            .and_then(|o| o.meldtermijn_uren)
            .is_some_and(|uren| uren > drempel_uren)
    }

    /// Legt de verwerkersovereenkomst vast.
    pub fn leg_overeenkomst_vast(
        &mut self,
        kenmerk: impl Into<String>,
        ondertekend_op: DateTime<Utc>,
        verwerking_begon_op: Option<DateTime<Utc>>,
        meldtermijn_uren: Option<u32>,
        op: DateTime<Utc>,
    ) -> Resultaat<()> {
        if ondertekend_op > op {
            return Err(DomeinFout::OnmogelijkTijdstip {
                veld: "leverancier.overeenkomst".into(),
                reden: "de overeenkomst zou in de toekomst zijn getekend; controleer de datum"
                    .into(),
            });
        }
        if meldtermijn_uren == Some(0) {
            return Err(DomeinFout::OngeldigeWaarde {
                veld: "leverancier.meldtermijn".into(),
                reden: "een meldtermijn van nul uur is geen afspraak maar een onmogelijkheid"
                    .into(),
            });
        }
        // De bestaande vindplaatsen blijven staan: die gaan over dezelfde
        // onderwerpen, en bij een nieuw contract worden zij opnieuw aangewezen.
        let vindplaatsen =
            self.overeenkomst.as_ref().map(|o| o.vindplaatsen.clone()).unwrap_or_default();
        self.overeenkomst = Some(Verwerkersovereenkomst {
            kenmerk: kenmerk.into(),
            ondertekend_op,
            verwerking_begon_op,
            meldtermijn_uren,
            vindplaatsen,
        });
        self.herkomst.wijzig("verwerkersovereenkomst vastgelegd", op);
        Ok(())
    }

    /// Wijst aan waar één onderdeel van artikel 28 lid 3 in het contract staat.
    pub fn wijs_vindplaats_aan(
        &mut self,
        eis: Contracteis,
        aanduiding: impl Into<String>,
        toelichting: Option<String>,
        op: DateTime<Utc>,
    ) -> Resultaat<()> {
        let aanduiding = aanduiding.into();
        if aanduiding.trim().is_empty() {
            return Err(DomeinFout::OngeldigeWaarde {
                veld: "leverancier.vindplaats".into(),
                reden: format!(
                    "wijs aan wáár onderdeel {} in het contract staat; een vinkje zegt alleen dat \
                     iemand ooit dacht dat het geregeld was",
                    eis.letter()
                ),
            });
        }
        let overeenkomst =
            self.overeenkomst.as_mut().ok_or_else(|| DomeinFout::OntbrekendeVerwijzing {
                veld: "leverancier.overeenkomst".into(),
                naar: "een vastgelegde verwerkersovereenkomst".into(),
            })?;
        if let Some(bestaand) = overeenkomst.vindplaatsen.iter_mut().find(|v| v.eis == eis) {
            bestaand.aanduiding = aanduiding;
            bestaand.toelichting = toelichting;
        } else {
            overeenkomst.vindplaatsen.push(Vindplaats { eis, aanduiding, toelichting });
        }
        self.herkomst.wijzig(format!("vindplaats voor onderdeel {} aangewezen", eis.letter()), op);
        Ok(())
    }

    /// Legt vast dat de subverwerkerslijst is nagelopen.
    pub fn controleer_subverwerkers(
        &mut self,
        moment: DateTime<Utc>,
        op: DateTime<Utc>,
    ) -> Resultaat<()> {
        if moment > op {
            return Err(DomeinFout::OnmogelijkTijdstip {
                veld: "leverancier.subverwerkers_gecontroleerd_op".into(),
                reden: "de controle zou in de toekomst liggen".into(),
            });
        }
        self.subverwerkers_gecontroleerd_op = Some(moment);
        self.herkomst.wijzig("subverwerkerslijst gecontroleerd", op);
        Ok(())
    }

    /// Voegt een subverwerker toe.
    ///
    /// De controledatum wordt hierbij **niet** bijgewerkt: iets toevoegen is
    /// niet hetzelfde als de lijst nalopen.
    pub fn voeg_subverwerker_toe(
        &mut self,
        naam: impl Into<String>,
        land: impl Into<String>,
        dienst: impl Into<String>,
        op: DateTime<Utc>,
    ) -> Resultaat<()> {
        let naam = naam.into();
        if self.subverwerkers.iter().any(|s| s.naam == naam) {
            return Err(DomeinFout::OngeldigeWaarde {
                veld: "leverancier.subverwerkers".into(),
                reden: format!("'{naam}' staat al in de lijst"),
            });
        }
        self.subverwerkers.push(Subverwerker { naam, land: land.into(), dienst: dienst.into() });
        self.herkomst.wijzig("subverwerker toegevoegd", op);
        Ok(())
    }

    /// Stelt de leverancier vast.
    pub fn stel_vast(&mut self, door: impl Into<String>, op: DateTime<Utc>) -> Resultaat<()> {
        let rapport = self.volledigheid();
        if !rapport.mag_vaststellen() {
            return Err(DomeinFout::NietVolledig {
                soort: "leverancier".into(),
                ontbreekt: rapport
                    .blokkades()
                    .into_iter()
                    .map(|o| format!("{} ({})", o.omschrijving, o.grondslag))
                    .collect(),
            });
        }
        self.status = Status::Vastgesteld;
        self.herkomst.stel_vast(door, op);
        Ok(())
    }
}

impl Volledig for Leverancier {
    fn soortnaam(&self) -> &'static str {
        "leverancier"
    }

    fn aantal_verplichte_onderdelen(&self) -> usize {
        // De kritikaliteit, de overeenkomst, de acht onderdelen van artikel 28
        // lid 3, en de controle van de subverwerkerslijst.
        2 + 8 + 1
    }

    fn ontbrekende_onderdelen(&self) -> Vec<Ontbrekend> {
        let mut uit = Vec::new();

        if self.kritikaliteit.is_none() {
            uit.push(Ontbrekend::signalerend(
                "leverancier.kritikaliteit",
                "beoordeel hoe belangrijk deze leverancier is voor de dienstverlening",
                "interne norm",
            ));
        }

        let Some(overeenkomst) = &self.overeenkomst else {
            uit.push(Ontbrekend::blokkerend(
                "leverancier.overeenkomst",
                "leg de verwerkersovereenkomst vast; zonder overeenkomst mag deze verwerker geen \
                 persoonsgegevens verwerken",
                "art. 28 lid 3 AVG",
            ));
            return uit;
        };

        for eis in overeenkomst.eisen_zonder_vindplaats() {
            uit.push(Ontbrekend::blokkerend(
                format!("leverancier.eis.{}", eis.letter()),
                format!(
                    "wijs aan waar in het contract staat dat de verwerker {}",
                    eis.omschrijving()
                ),
                eis.grondslag(),
            ));
        }

        if overeenkomst.meldtermijn_uren.is_none() {
            uit.push(Ontbrekend::blokkerend(
                "leverancier.meldtermijn",
                "leg vast binnen hoeveel uur de verwerker een inbreuk moet melden; zonder termijn \
                 is er niets afgesproken en loopt uw eigen termijn van tweeënzeventig uur door",
                "art. 33 lid 2 AVG",
            ));
        }

        if self.subverwerkers_gecontroleerd_op.is_none() {
            uit.push(Ontbrekend::signalerend(
                "leverancier.subverwerkers",
                "loop de subverwerkerslijst na; melden is iets anders dan controleren",
                "art. 28 lid 2 en lid 4 AVG",
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

    fn leverancier() -> Leverancier {
        Leverancier::nieuw("LEV-014", "de hostingaanbieder", "Nederland", "u1", nu())
    }

    fn met_overeenkomst() -> Leverancier {
        let mut l = leverancier();
        l.leg_overeenkomst_vast("VWO-2026-014", nu(), None, Some(24), nu()).unwrap();
        l
    }

    fn volledig() -> Leverancier {
        let mut l = met_overeenkomst();
        l.kritikaliteit = Some(Kritikaliteit::Gemiddeld);
        for eis in Contracteis::alle() {
            l.wijs_vindplaats_aan(eis, format!("artikel {}", eis.letter()), None, nu()).unwrap();
        }
        l.controleer_subverwerkers(nu(), nu()).unwrap();
        l
    }

    #[test]
    fn zonder_overeenkomst_mag_de_verwerker_niets() {
        let l = leverancier();
        let blokkades: Vec<_> = l
            .ontbrekende_onderdelen()
            .into_iter()
            .filter(|o| o.blokkeert_vaststelling)
            .map(|o| o.omschrijving)
            .collect();
        assert!(blokkades.iter().any(|b| b.contains("geen persoonsgegevens verwerken")));
    }

    /// Een vinkje zegt alleen dat iemand ooit dacht dat het geregeld was.
    #[test]
    fn een_onderdeel_zonder_vindplaats_geldt_als_niet_geregeld() {
        let l = met_overeenkomst();
        assert_eq!(l.overeenkomst.as_ref().unwrap().eisen_zonder_vindplaats().len(), 8);

        let velden: Vec<_> = l.ontbrekende_onderdelen().into_iter().map(|o| o.veld).collect();
        for eis in Contracteis::alle() {
            assert!(velden.contains(&format!("leverancier.eis.{}", eis.letter())));
        }
    }

    #[test]
    fn een_lege_vindplaats_wordt_geweigerd() {
        let mut l = met_overeenkomst();
        let fout = l.wijs_vindplaats_aan(Contracteis::Instructies, "  ", None, nu()).unwrap_err();
        assert!(fout.to_string().contains("een vinkje zegt alleen"));
    }

    #[test]
    fn acht_vindplaatsen_maken_de_overeenkomst_compleet() {
        let l = volledig();
        assert!(l.overeenkomst.as_ref().unwrap().eisen_zonder_vindplaats().is_empty());
        assert!(l.volledigheid().is_volledig());
    }

    #[test]
    fn elk_onderdeel_draagt_zijn_eigen_letter_en_grondslag() {
        for eis in Contracteis::alle() {
            assert!(eis.grondslag().contains(&format!("onder {}", eis.letter())));
        }
    }

    /// Wie een verwerker vier dagen geeft, heeft zijn eigen termijn weggegeven.
    #[test]
    fn een_te_lange_meldtermijn_is_te_bepalen() {
        let mut l = met_overeenkomst();
        assert!(!l.meldtermijn_te_lang(48));

        l.leg_overeenkomst_vast("VWO-2026-014", nu(), None, Some(96), nu()).unwrap();
        assert!(l.meldtermijn_te_lang(48));
    }

    /// Geen termijn is erger dan een te lange termijn: dan is er niets
    /// afgesproken.
    #[test]
    fn zonder_meldtermijn_blokkeert_het_dossier() {
        let mut l = leverancier();
        l.leg_overeenkomst_vast("VWO-2026-014", nu(), None, None, nu()).unwrap();
        assert!(!l.meldtermijn_te_lang(48), "er is geen termijn om te lang te zijn");

        let blokkades: Vec<_> = l
            .ontbrekende_onderdelen()
            .into_iter()
            .filter(|o| o.blokkeert_vaststelling)
            .map(|o| o.omschrijving)
            .collect();
        assert!(blokkades.iter().any(|b| b.contains("niets afgesproken")));
    }

    #[test]
    fn een_meldtermijn_van_nul_uur_wordt_geweigerd() {
        let mut l = leverancier();
        let fout = l.leg_overeenkomst_vast("VWO-2026-014", nu(), None, Some(0), nu()).unwrap_err();
        assert!(fout.to_string().contains("onmogelijkheid"));
    }

    /// Een overeenkomst die na de aanvang is getekend, dekt de periode ervóór
    /// niet. Dat is een feit dat blijft staan.
    #[test]
    fn tekenen_na_aanvang_is_zichtbaar() {
        let mut l = leverancier();
        let start = nu() - Duration::days(60);
        l.leg_overeenkomst_vast("VWO-2026-014", nu(), Some(start), Some(24), nu()).unwrap();
        assert!(l.overeenkomst.as_ref().unwrap().getekend_na_aanvang());

        // En andersom niet.
        l.leg_overeenkomst_vast("VWO-2026-014", start, Some(nu()), Some(24), nu()).unwrap();
        assert!(!l.overeenkomst.as_ref().unwrap().getekend_na_aanvang());
    }

    /// Iets toevoegen is niet hetzelfde als de lijst nalopen.
    #[test]
    fn een_subverwerker_toevoegen_is_geen_controle() {
        let mut l = met_overeenkomst();
        l.voeg_subverwerker_toe("het rekencentrum", "Duitsland", "opslag", nu()).unwrap();
        assert_eq!(l.subverwerkers_gecontroleerd_op, None);

        l.controleer_subverwerkers(nu(), nu()).unwrap();
        assert_eq!(l.maanden_sinds_subverwerkerscontrole(nu()), Some(0));
    }

    #[test]
    fn de_maanden_sinds_de_controle_zijn_te_bepalen() {
        let mut l = met_overeenkomst();
        l.controleer_subverwerkers(nu() - Duration::days(30 * 14), nu()).unwrap();
        assert_eq!(l.maanden_sinds_subverwerkerscontrole(nu()), Some(14));
    }

    #[test]
    fn dezelfde_subverwerker_komt_er_niet_twee_keer_in() {
        let mut l = met_overeenkomst();
        l.voeg_subverwerker_toe("het rekencentrum", "Duitsland", "opslag", nu()).unwrap();
        assert!(l.voeg_subverwerker_toe("het rekencentrum", "Duitsland", "opslag", nu()).is_err());
    }

    /// Een nieuw contract wist de aangewezen vindplaatsen niet: die gaan over
    /// dezelfde acht onderwerpen.
    #[test]
    fn een_nieuw_contract_behoudt_de_vindplaatsen() {
        let mut l = volledig();
        l.leg_overeenkomst_vast("VWO-2027-014", nu(), None, Some(24), nu()).unwrap();
        assert!(l.overeenkomst.as_ref().unwrap().eisen_zonder_vindplaats().is_empty());
    }

    #[test]
    fn vaststellen_kan_pas_als_alles_is_aangewezen() {
        let mut l = met_overeenkomst();
        assert!(l.stel_vast("A. de Vries", nu()).is_err());

        let mut l = volledig();
        l.stel_vast("A. de Vries", nu()).unwrap();
        assert_eq!(l.status, Status::Vastgesteld);
    }

    #[test]
    fn de_leverancier_overleeft_serialisatie() {
        let mut l = volledig();
        l.voeg_subverwerker_toe("het rekencentrum", "Duitsland", "opslag", nu()).unwrap();
        let json = serde_json::to_string(&l).unwrap();
        let terug: Leverancier = serde_json::from_str(&json).unwrap();
        assert_eq!(l, terug);
    }
}
