//! Doorgiften buiten de Europese Economische Ruimte.
//!
//! # Waarom het instrument niet genoeg is
//!
//! Hoofdstuk V staat een doorgifte toe op grond van een instrument: een
//! adequaatheidsbesluit, modelcontractbepalingen, bindende bedrijfsvoorschriften
//! en zo verder. Het aanwijzen van dat instrument is de makkelijke helft.
//!
//! De andere helft is dat een instrument iets moet wáármaken. Bij
//! modelcontractbepalingen betekent dat een beoordeling van het recht en de
//! praktijk in het ontvangstland: bieden die daadwerkelijk een beschermingsniveau
//! dat in grote lijnen overeenkomt met dat in de Unie, en zo nee, welke
//! aanvullende maatregelen dichten het gat? Dat is de doorgiftebeoordeling, en
//! zonder haar is het contract een handtekening onder een aanname.
//!
//! # Twee dingen die vanzelf verlopen
//!
//! Een instrument kan worden ingetrokken of onder toetsing komen te staan
//! zonder dat er in de organisatie iets verandert. Dan blijft de doorgifte
//! lopen op een waarborg die er niet meer is — en niemand merkt het, want er is
//! geen gebeurtenis. De status van het instrument komt daarom uit het
//! kennispakket en wordt hier tegen de doorgifte aan gehouden.
//!
//! Het tweede is artikel 49. Die uitzonderingen zijn er voor incidentele
//! gevallen. Wie ze structureel gebruikt, gebruikt geen uitzondering meer maar
//! een instrument dat hij niet heeft — en dat is precies wat wél te tellen is.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    basis::{Compartiment, Herkomst, Id, Motivering, Status},
    error::{DomeinFout, Resultaat},
    volledigheid::{Ontbrekend, Volledig},
};

/// Waarop de doorgifte berust.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Doorgifteinstrumentsoort {
    Adequaatheidsbesluit,
    Modelbepalingen,
    BindendeBedrijfsvoorschriften,
    Gedragscode,
    Certificering,
    Artikel49Uitzondering,
    /// Er is geen instrument. Dat is geen keuze maar een constatering.
    Geen,
}

impl Doorgifteinstrumentsoort {
    pub fn omschrijving(&self) -> &'static str {
        match self {
            Self::Adequaatheidsbesluit => "adequaatheidsbesluit",
            Self::Modelbepalingen => "modelcontractbepalingen",
            Self::BindendeBedrijfsvoorschriften => "bindende bedrijfsvoorschriften",
            Self::Gedragscode => "goedgekeurde gedragscode",
            Self::Certificering => "goedgekeurd certificeringsmechanisme",
            Self::Artikel49Uitzondering => "uitzondering van artikel 49",
            Self::Geen => "geen instrument",
        }
    }

    pub fn grondslag(&self) -> &'static str {
        match self {
            Self::Adequaatheidsbesluit => "art. 45 AVG",
            Self::Modelbepalingen
            | Self::BindendeBedrijfsvoorschriften
            | Self::Gedragscode
            | Self::Certificering => "art. 46 AVG",
            Self::Artikel49Uitzondering => "art. 49 AVG",
            Self::Geen => "hoofdstuk V AVG",
        }
    }

    /// Of dit instrument een beoordeling van het ontvangstland vergt.
    ///
    /// Bij een adequaatheidsbesluit heeft de Commissie die beoordeling al
    /// gedaan; bij de instrumenten van artikel 46 doet de organisatie het zelf.
    pub fn vraagt_beoordeling(&self) -> bool {
        matches!(
            self,
            Self::Modelbepalingen
                | Self::BindendeBedrijfsvoorschriften
                | Self::Gedragscode
                | Self::Certificering
        )
    }

    pub fn alle() -> [Self; 7] {
        [
            Self::Adequaatheidsbesluit,
            Self::Modelbepalingen,
            Self::BindendeBedrijfsvoorschriften,
            Self::Gedragscode,
            Self::Certificering,
            Self::Artikel49Uitzondering,
            Self::Geen,
        ]
    }
}

/// De uitkomst van de doorgiftebeoordeling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Beoordelingsuitkomst {
    /// Het beschermingsniveau komt in grote lijnen overeen.
    Gelijkwaardig,
    /// Alleen met aanvullende maatregelen.
    GelijkwaardigMetMaatregelen,
    /// Ook met maatregelen niet; de doorgifte kan niet doorgaan.
    NietGelijkwaardig,
}

impl Beoordelingsuitkomst {
    pub fn omschrijving(&self) -> &'static str {
        match self {
            Self::Gelijkwaardig => "in grote lijnen gelijkwaardig",
            Self::GelijkwaardigMetMaatregelen => "gelijkwaardig met aanvullende maatregelen",
            Self::NietGelijkwaardig => "niet gelijkwaardig",
        }
    }

    pub fn draagt_de_doorgifte(&self) -> bool {
        !matches!(self, Self::NietGelijkwaardig)
    }
}

/// De beoordeling van het recht en de praktijk in het ontvangstland.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Doorgiftebeoordeling {
    pub datum: DateTime<Utc>,
    pub uitvoerder: String,
    /// Wanneer de rechtsontwikkelingen in het ontvangstland voor het laatst
    /// zijn nagelopen. Een beoordeling van drie jaar oud beschrijft een land
    /// dat sindsdien kan zijn veranderd.
    pub rechtsontwikkelingen_geraadpleegd_op: DateTime<Utc>,
    pub uitkomst: Beoordelingsuitkomst,
    pub restrisico: Motivering,
    pub besluit_door: String,
}

/// Eén doorgifte naar een ontvanger buiten de EER.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Doorgifte {
    pub id: Id,
    pub kenmerk: String,
    pub omschrijving: String,
    pub status: Status,
    pub compartiment: Compartiment,
    pub herkomst: Herkomst,

    pub verwerking_id: Id,
    pub verwerking_kenmerk: String,

    pub ontvanger: String,
    pub ontvangerland: String,

    pub instrument: Option<Doorgifteinstrumentsoort>,
    /// De code van het instrument in het kennispakket, bijvoorbeeld SCC-2021.
    pub instrument_code: Option<String>,
    /// De status zoals die bij de laatste controle in het kennispakket stond.
    pub instrument_status_bij_controle: Option<String>,

    pub beoordeling: Option<Doorgiftebeoordeling>,
    pub aanvullende_maatregelen: Vec<String>,

    /// Bij een uitzondering van artikel 49: welke grond, en hoe vaak zij dit
    /// jaar is toegepast.
    pub artikel49_grond: Option<String>,
    pub artikel49_toepassingen_dit_jaar: u32,

    pub informatieplicht_uitgevoerd_op: Option<DateTime<Utc>>,
}

impl Doorgifte {
    // Acht argumenten, en dat is er één meer dan clippy fraai vindt. Ze zijn
    // alle acht nodig om een doorgifte te kunnen identificeren: het dossier,
    // de registerregel waaraan het hangt, de ontvanger met zijn land, en wie
    // het wanneer aanmaakte. Een bouwertype ertussen zou een begrip toevoegen
    // dat verder nergens voorkomt.
    #[allow(clippy::too_many_arguments)]
    pub fn nieuw(
        kenmerk: impl Into<String>,
        omschrijving: impl Into<String>,
        verwerking_id: Id,
        verwerking_kenmerk: impl Into<String>,
        ontvanger: impl Into<String>,
        ontvangerland: impl Into<String>,
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
            ontvanger: ontvanger.into(),
            ontvangerland: ontvangerland.into(),
            instrument: None,
            instrument_code: None,
            instrument_status_bij_controle: None,
            beoordeling: None,
            aanvullende_maatregelen: Vec::new(),
            artikel49_grond: None,
            artikel49_toepassingen_dit_jaar: 0,
            informatieplicht_uitgevoerd_op: None,
        }
    }

    /// Of deze doorgifte een beoordeling nodig heeft die er nog niet is.
    pub fn mist_beoordeling(&self) -> bool {
        self.instrument.is_some_and(|i| i.vraagt_beoordeling()) && self.beoordeling.is_none()
    }

    /// Of de uitzondering van artikel 49 structureel wordt gebruikt.
    ///
    /// De drempel komt van de aanroeper: hoeveel "incidenteel" is, staat in het
    /// kennispakket en niet in deze code.
    pub fn gebruikt_uitzondering_structureel(&self, drempel: u32) -> bool {
        self.instrument == Some(Doorgifteinstrumentsoort::Artikel49Uitzondering)
            && self.artikel49_toepassingen_dit_jaar > drempel
    }

    /// Wijst het instrument aan.
    pub fn kies_instrument(
        &mut self,
        instrument: Doorgifteinstrumentsoort,
        code: Option<String>,
        op: DateTime<Utc>,
    ) -> Resultaat<()> {
        if instrument == Doorgifteinstrumentsoort::Artikel49Uitzondering
            && self.artikel49_grond.is_none()
        {
            return Err(DomeinFout::OngeldigeWaarde {
                veld: "doorgifte.artikel49_grond".into(),
                reden: "benoem eerst welke uitzondering van artikel 49 wordt ingeroepen; de \
                        opsomming daar is limitatief"
                    .into(),
            });
        }
        self.instrument = Some(instrument);
        self.instrument_code = code;
        self.herkomst.wijzig(format!("instrument: {}", instrument.omschrijving()), op);
        Ok(())
    }

    /// Legt de doorgiftebeoordeling vast.
    pub fn leg_beoordeling_vast(
        &mut self,
        beoordeling: Doorgiftebeoordeling,
        op: DateTime<Utc>,
    ) -> Resultaat<()> {
        if beoordeling.datum > op {
            return Err(DomeinFout::OnmogelijkTijdstip {
                veld: "doorgifte.beoordeling".into(),
                reden: "de beoordeling zou in de toekomst zijn uitgevoerd".into(),
            });
        }
        if beoordeling.rechtsontwikkelingen_geraadpleegd_op > op {
            return Err(DomeinFout::OnmogelijkTijdstip {
                veld: "doorgifte.beoordeling".into(),
                reden: "de rechtsontwikkelingen zouden in de toekomst zijn geraadpleegd".into(),
            });
        }
        if beoordeling.uitkomst == Beoordelingsuitkomst::GelijkwaardigMetMaatregelen
            && self.aanvullende_maatregelen.is_empty()
        {
            return Err(DomeinFout::OngeldigeWaarde {
                veld: "doorgifte.aanvullende_maatregelen".into(),
                reden: "deze uitkomst berust op aanvullende maatregelen; benoem er ten minste \
                        één, anders berust zij nergens op"
                    .into(),
            });
        }
        self.beoordeling = Some(beoordeling);
        self.herkomst.wijzig("doorgiftebeoordeling vastgelegd", op);
        Ok(())
    }

    /// Legt vast wat de status van het instrument was bij de laatste controle.
    ///
    /// Staat die op ingetrokken of onder toetsing, dan gaat de doorgifte op
    /// herziening: de waarborg waarop zij rust, is er niet meer of staat ter
    /// discussie.
    pub fn controleer_instrument(
        &mut self,
        status: impl Into<String>,
        vereist_herbeoordeling: bool,
        op: DateTime<Utc>,
    ) {
        let status = status.into();
        self.instrument_status_bij_controle = Some(status.clone());
        if vereist_herbeoordeling {
            if self.status == Status::Vastgesteld {
                self.status = Status::HerzieningNodig;
            }
            self.herkomst.wijzig(
                format!("systeem: het instrument staat op '{status}' en vraagt om herbeoordeling"),
                op,
            );
        } else {
            self.herkomst.wijzig(format!("instrument gecontroleerd: {status}"), op);
        }
    }

    /// Stelt de doorgifte vast.
    pub fn stel_vast(&mut self, door: impl Into<String>, op: DateTime<Utc>) -> Resultaat<()> {
        let rapport = self.volledigheid();
        if !rapport.mag_vaststellen() {
            return Err(DomeinFout::NietVolledig {
                soort: "doorgifte".into(),
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

impl Volledig for Doorgifte {
    fn soortnaam(&self) -> &'static str {
        "doorgifte"
    }

    fn aantal_verplichte_onderdelen(&self) -> usize {
        // Het instrument en de informatieplicht staan er altijd.
        let mut verplicht = 2;
        // Vraagt het instrument een beoordeling, dan komt die erbij.
        if self.instrument.is_some_and(|i| i.vraagt_beoordeling()) {
            verplicht += 1;
        }
        // Bij artikel 49 komt de grond erbij.
        if self.instrument == Some(Doorgifteinstrumentsoort::Artikel49Uitzondering) {
            verplicht += 1;
        }
        verplicht
    }

    fn ontbrekende_onderdelen(&self) -> Vec<Ontbrekend> {
        let mut uit = Vec::new();

        match self.instrument {
            None => uit.push(Ontbrekend::blokkerend(
                "doorgifte.instrument",
                "wijs aan waarop deze doorgifte berust",
                "hoofdstuk V AVG",
            )),
            // Geen instrument is geen keuze maar een constatering, en die houdt
            // de doorgifte tegen.
            Some(Doorgifteinstrumentsoort::Geen) => uit.push(Ontbrekend::blokkerend(
                "doorgifte.instrument",
                "er is geen instrument aangewezen; zonder waarborg uit hoofdstuk V mag deze \
                 doorgifte niet plaatsvinden",
                "art. 44 AVG",
            )),
            Some(_) => {}
        }

        if self.mist_beoordeling() {
            uit.push(Ontbrekend::blokkerend(
                "doorgifte.beoordeling",
                "beoordeel het recht en de praktijk in het ontvangstland; zonder die beoordeling \
                 is het contract een handtekening onder een aanname",
                "art. 46 AVG",
            ));
        }
        if self.instrument == Some(Doorgifteinstrumentsoort::Artikel49Uitzondering)
            && self.artikel49_grond.is_none()
        {
            uit.push(Ontbrekend::blokkerend(
                "doorgifte.artikel49_grond",
                "benoem welke uitzondering van artikel 49 wordt ingeroepen",
                "art. 49 lid 1 AVG",
            ));
        }
        if self.informatieplicht_uitgevoerd_op.is_none() {
            uit.push(Ontbrekend::signalerend(
                "doorgifte.informatieplicht",
                "informeer de betrokkene over de doorgifte en de waarborg waarop zij berust",
                "art. 13 lid 1 onder f AVG",
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

    fn doorgifte() -> Doorgifte {
        Doorgifte::nieuw(
            "EER-0412",
            "hosting bij een aanbieder in de Verenigde Staten",
            Id::nieuw(),
            "0412-K",
            "de hostingaanbieder",
            "Verenigde Staten",
            "u1",
            nu(),
        )
    }

    fn beoordeling(uitkomst: Beoordelingsuitkomst) -> Doorgiftebeoordeling {
        Doorgiftebeoordeling {
            datum: nu(),
            uitvoerder: "A. de Vries".into(),
            rechtsontwikkelingen_geraadpleegd_op: nu(),
            uitkomst,
            restrisico: motivering("toegang door overheidsdiensten blijft mogelijk"),
            besluit_door: "de directie".into(),
        }
    }

    /// Geen instrument is geen keuze maar een constatering.
    #[test]
    fn zonder_instrument_mag_de_doorgifte_niet() {
        let d = doorgifte();
        let velden: Vec<_> = d.ontbrekende_onderdelen().into_iter().map(|o| o.veld).collect();
        assert!(velden.contains(&"doorgifte.instrument".to_string()));

        let mut d = doorgifte();
        d.kies_instrument(Doorgifteinstrumentsoort::Geen, None, nu()).unwrap();
        let blokkades: Vec<_> = d
            .ontbrekende_onderdelen()
            .into_iter()
            .filter(|o| o.blokkeert_vaststelling)
            .map(|o| o.omschrijving)
            .collect();
        assert!(blokkades.iter().any(|b| b.contains("mag deze doorgifte niet plaatsvinden")));
    }

    /// Bij een adequaatheidsbesluit heeft de Commissie de beoordeling al gedaan.
    #[test]
    fn een_adequaatheidsbesluit_vraagt_geen_eigen_beoordeling() {
        let mut d = doorgifte();
        d.kies_instrument(Doorgifteinstrumentsoort::Adequaatheidsbesluit, None, nu()).unwrap();
        assert!(!d.mist_beoordeling());
    }

    /// Modelbepalingen zonder beoordeling zijn een handtekening onder een
    /// aanname. Dit voedt regel EER-03.
    #[test]
    fn modelbepalingen_vragen_een_beoordeling() {
        let mut d = doorgifte();
        d.kies_instrument(Doorgifteinstrumentsoort::Modelbepalingen, Some("SCC-2021".into()), nu())
            .unwrap();
        assert!(d.mist_beoordeling());

        d.leg_beoordeling_vast(beoordeling(Beoordelingsuitkomst::Gelijkwaardig), nu()).unwrap();
        assert!(!d.mist_beoordeling());
    }

    #[test]
    fn met_maatregelen_zonder_maatregelen_wordt_geweigerd() {
        let mut d = doorgifte();
        d.kies_instrument(Doorgifteinstrumentsoort::Modelbepalingen, None, nu()).unwrap();
        let fout = d
            .leg_beoordeling_vast(
                beoordeling(Beoordelingsuitkomst::GelijkwaardigMetMaatregelen),
                nu(),
            )
            .unwrap_err();
        assert!(fout.to_string().contains("berust zij nergens op"));

        d.aanvullende_maatregelen.push("versleuteling met sleutelbeheer binnen de EER".into());
        d.leg_beoordeling_vast(
            beoordeling(Beoordelingsuitkomst::GelijkwaardigMetMaatregelen),
            nu(),
        )
        .unwrap();
    }

    /// De uitzondering van artikel 49 vergt eerst een grond uit die limitatieve
    /// opsomming.
    #[test]
    fn artikel_49_vergt_eerst_een_grond() {
        let mut d = doorgifte();
        let fout = d
            .kies_instrument(Doorgifteinstrumentsoort::Artikel49Uitzondering, None, nu())
            .unwrap_err();
        assert!(fout.to_string().contains("limitatief"));

        d.artikel49_grond = Some("uitdrukkelijke toestemming van de betrokkene".into());
        d.kies_instrument(Doorgifteinstrumentsoort::Artikel49Uitzondering, None, nu()).unwrap();
    }

    /// Structureel gebruik is geen uitzondering meer. Dit voedt regel EER-06.
    #[test]
    fn structureel_gebruik_van_een_uitzondering_is_te_tellen() {
        let mut d = doorgifte();
        d.artikel49_grond = Some("uitdrukkelijke toestemming".into());
        d.kies_instrument(Doorgifteinstrumentsoort::Artikel49Uitzondering, None, nu()).unwrap();

        d.artikel49_toepassingen_dit_jaar = 2;
        assert!(!d.gebruikt_uitzondering_structureel(2));
        d.artikel49_toepassingen_dit_jaar = 3;
        assert!(d.gebruikt_uitzondering_structureel(2));
    }

    /// Een instrument kan verlopen zonder dat er in de organisatie iets
    /// gebeurt. Dit voedt regel EER-07.
    #[test]
    fn een_ingetrokken_instrument_zet_de_doorgifte_op_herziening() {
        let mut d = doorgifte();
        d.kies_instrument(Doorgifteinstrumentsoort::Modelbepalingen, Some("SCC-2021".into()), nu())
            .unwrap();
        d.leg_beoordeling_vast(beoordeling(Beoordelingsuitkomst::Gelijkwaardig), nu()).unwrap();
        d.informatieplicht_uitgevoerd_op = Some(nu());
        d.stel_vast("A. de Vries", nu()).unwrap();

        d.controleer_instrument("ingetrokken", true, nu());
        assert_eq!(d.status, Status::HerzieningNodig);
        assert!(d.herkomst.gewijzigd_door.contains("herbeoordeling"));
    }

    #[test]
    fn een_geldig_instrument_laat_de_status_staan() {
        let mut d = doorgifte();
        d.kies_instrument(Doorgifteinstrumentsoort::Modelbepalingen, Some("SCC-2021".into()), nu())
            .unwrap();
        d.leg_beoordeling_vast(beoordeling(Beoordelingsuitkomst::Gelijkwaardig), nu()).unwrap();
        d.informatieplicht_uitgevoerd_op = Some(nu());
        d.stel_vast("A. de Vries", nu()).unwrap();

        d.controleer_instrument("geldig", false, nu());
        assert_eq!(d.status, Status::Vastgesteld);
    }

    /// De informatieplicht blokkeert niet, maar blijft wel zichtbaar.
    #[test]
    fn de_informatieplicht_signaleert_maar_blokkeert_niet() {
        let mut d = doorgifte();
        d.kies_instrument(Doorgifteinstrumentsoort::Adequaatheidsbesluit, None, nu()).unwrap();
        let onderdelen = d.ontbrekende_onderdelen();
        let informatie = onderdelen
            .iter()
            .find(|o| o.veld == "doorgifte.informatieplicht")
            .expect("hoort er te staan");
        assert!(!informatie.blokkeert_vaststelling);
        assert!(d.stel_vast("A. de Vries", nu()).is_ok());
    }

    #[test]
    fn elk_instrument_draagt_een_grondslag() {
        for i in Doorgifteinstrumentsoort::alle() {
            assert!(
                i.grondslag().starts_with("art.") || i.grondslag().starts_with("hoofdstuk"),
                "{i:?} mist een grondslag"
            );
        }
    }

    #[test]
    fn de_doorgifte_overleeft_serialisatie() {
        let mut d = doorgifte();
        d.kies_instrument(Doorgifteinstrumentsoort::Modelbepalingen, Some("SCC-2021".into()), nu())
            .unwrap();
        d.aanvullende_maatregelen.push("versleuteling met sleutelbeheer binnen de EER".into());
        d.leg_beoordeling_vast(
            beoordeling(Beoordelingsuitkomst::GelijkwaardigMetMaatregelen),
            nu(),
        )
        .unwrap();

        let json = serde_json::to_string(&d).unwrap();
        let terug: Doorgifte = serde_json::from_str(&json).unwrap();
        assert_eq!(d, terug);
    }
}
