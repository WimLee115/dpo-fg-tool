//! De gegevensbeschermingseffectbeoordeling als eigen dossier.
//!
//! # Wat dit dossier wel en niet doet
//!
//! Het legt vast **dat** er is beoordeeld, **wanneer**, **door wie**, en wat de
//! vier onderdelen van artikel 35 lid 7 hebben opgeleverd. Het beslist niets:
//! of een effectbeoordeling verplicht is, of een restrisico hoog is en of
//! voorafgaande raadpleging nodig is, zijn oordelen. De tool telt de criteria,
//! toont wat er ontbreekt en bewaakt de klok — het oordeel blijft bij de mens,
//! en de motivering daarvan wordt vastgelegd zodat het achteraf te volgen is.
//!
//! # De volgorde is geen vormvereiste
//!
//! Een restrisico is per definitie wat er overblijft ná de maatregelen. Het is
//! daarom niet vast te leggen zolang de risico's en de maatregelen niet zijn
//! benoemd: dan zou het een oordeel zijn over iets wat nog niet is beschreven.
//! Dat is de enige plaats waar dit dossier een volgorde afdwingt, en die volgt
//! rechtstreeks uit artikel 35 lid 7 onder c en d.
//!
//! # De klok
//!
//! De termijn voor voorafgaande raadpleging (artikel 36 lid 2 AVG: acht weken,
//! eenmaal verlengbaar met zes weken, op te schorten zolang de toezichthouder
//! op opgevraagde informatie wacht) wordt niet hier berekend. De motor in
//! `dpofg_terms` rekent; dit dossier bewaart de lopende termijn en bewaakt de
//! invarianten eromheen. De termijnsoort wordt bij indiening bevroren: de
//! termijn die gold bij indiening is de termijn die geldt, ook wanneer het
//! kennispakket later wordt gecorrigeerd.

use chrono::{DateTime, Utc};
use dpofg_terms::LopendeTermijn;
use serde::{Deserialize, Serialize};

use crate::{
    basis::{Compartiment, Herkomst, Id, Motivering, Status},
    error::{DomeinFout, Resultaat},
    volledigheid::{Ontbrekend, Volledig},
};

/// De uitkomst van de voortoets: is een effectbeoordeling nodig?
///
/// Dit is een **antwoord van de gebruiker**, niet van de tool. De tool telt de
/// criteria en toont ze; wie de verwerking kent, beslist.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Voortoets {
    /// Beoordeeld en niet nodig bevonden. De motivering hoort er dan wél te
    /// zijn: artikel 5 lid 2 vraagt om aantoonbaarheid, en juist een negatief
    /// besluit moet later te volgen zijn.
    NietNodig,
    /// Een effectbeoordeling is nodig.
    Vereist,
    /// Niet verplicht, maar toch uitgevoerd.
    Vrijwillig,
}

impl Voortoets {
    pub fn omschrijving(&self) -> &'static str {
        match self {
            Self::NietNodig => "niet nodig",
            Self::Vereist => "vereist",
            Self::Vrijwillig => "vrijwillig uitgevoerd",
        }
    }

    pub fn alle() -> [Self; 3] {
        [Self::NietNodig, Self::Vereist, Self::Vrijwillig]
    }
}

/// Het niveau van het risico dat na de maatregelen overblijft.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Restrisiconiveau {
    Laag,
    Gemiddeld,
    Hoog,
}

impl Restrisiconiveau {
    pub fn omschrijving(&self) -> &'static str {
        match self {
            Self::Laag => "laag",
            Self::Gemiddeld => "gemiddeld",
            Self::Hoog => "hoog",
        }
    }

    pub fn is_hoog(&self) -> bool {
        matches!(self, Self::Hoog)
    }

    pub fn alle() -> [Self; 3] {
        [Self::Laag, Self::Gemiddeld, Self::Hoog]
    }
}

/// Het risico dat na de maatregelen overblijft, met de weging eronder.
///
/// Niveau en motivering zitten in één type omdat een niveau zonder weging niets
/// zegt: "hoog" is geen bevinding maar een conclusie, en een conclusie zonder
/// redenering is bij een uitvraag onbruikbaar. Dit type is buiten deze module
/// niet te construeren zonder motivering.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Restrisico {
    pub niveau: Restrisiconiveau,
    pub motivering: Motivering,
    /// Hoeveel maatregelen er bij de weging op tafel lagen.
    ///
    /// Legt vast waartegen is afgewogen. Worden er later maatregelen
    /// bijgeschreven, dan is zichtbaar dat de weging op minder berustte.
    pub gewogen_maatregelen: usize,
}

/// Het dossier van één effectbeoordeling.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Dpia {
    pub id: Id,
    pub kenmerk: String,
    pub omschrijving: String,
    pub status: Status,
    pub compartiment: Compartiment,
    pub herkomst: Herkomst,

    // --- artikel 35 lid 1: waarom deze beoordeling bestaat ---
    /// De verwerking waarop de beoordeling ziet. Verplicht bij het aanmaken:
    /// een effectbeoordeling zonder verwerking beoordeelt niets.
    pub verwerking_id: Id,
    pub voortoets: Option<Voortoets>,
    pub voortoets_motivering: Option<Motivering>,

    // --- artikel 35 lid 7: de beoordeling zelf ---
    pub datum: Option<DateTime<Utc>>,
    /// De gebruikte methode. Open veld: de verordening schrijft geen methode
    /// voor, en een gesloten lijst zou een keuze afdwingen die de wet vrijlaat.
    pub methode: Option<String>,
    pub uitgevoerd_door: Option<String>,
    /// Of de beoordeling is uitgevoerd vóór de verwerking begon.
    ///
    /// `None` betekent: nog niet beantwoord. Dat is iets anders dan "nee", en
    /// regel DPIA-03 slaat daarom niet aan zolang de vraag openstaat.
    pub vooraf_uitgevoerd: Option<bool>,
    /// Artikel 35 lid 7 onder a.
    pub systematische_beschrijving: Option<String>,
    /// Artikel 35 lid 7 onder b.
    pub noodzaak_en_evenredigheid: Option<String>,
    /// Artikel 35 lid 7 onder c.
    pub risicos: Vec<String>,
    /// Artikel 35 lid 7 onder d.
    pub maatregelen: Vec<String>,
    pub restrisico: Option<Restrisico>,

    // --- artikel 35 lid 2 ---
    pub advies_functionaris: Option<Motivering>,

    // --- artikel 36 ---
    pub raadpleging: Option<LopendeTermijn>,
    /// Welke stand van het kennispakket gold bij indiening.
    ///
    /// De termijnsoort wordt bij indiening bevroren; deze aanduiding maakt
    /// afleesbaar op welke stand van de inhoud de klok berust.
    pub raadpleging_pakket: Option<String>,
    pub advies_referentie: Option<String>,
}

impl Dpia {
    pub fn nieuw(
        kenmerk: impl Into<String>,
        omschrijving: impl Into<String>,
        verwerking_id: Id,
        door: impl Into<String>,
        op: DateTime<Utc>,
    ) -> Self {
        Self {
            id: Id::nieuw(),
            kenmerk: kenmerk.into(),
            omschrijving: omschrijving.into(),
            status: Status::Concept,
            // Het dossier draagt het restrisico-oordeel, het advies van de
            // toezichthouder en de namen van wie beoordeelde en vaststelde.
            compartiment: Compartiment::nieuw(Compartiment::VERTROUWELIJK),
            herkomst: Herkomst::nieuw(door, op),
            verwerking_id,
            voortoets: None,
            voortoets_motivering: None,
            datum: None,
            methode: None,
            uitgevoerd_door: None,
            vooraf_uitgevoerd: None,
            systematische_beschrijving: None,
            noodzaak_en_evenredigheid: None,
            risicos: Vec::new(),
            maatregelen: Vec::new(),
            restrisico: None,
            advies_functionaris: None,
            raadpleging: None,
            raadpleging_pakket: None,
            advies_referentie: None,
        }
    }

    /// Of voorafgaande raadpleging aan de orde is.
    ///
    /// Afgeleid uit het restrisico en niet apart vastgelegd, zodat de twee niet
    /// uit elkaar kunnen lopen. Dit berust op de lezing dat artikel 36 lid 1
    /// aangrijpt op het risico dat ná de maatregelen overblijft; die lezing is
    /// gangbaar, maar het blijft een lezing en zij hoort daarom vindbaar te
    /// zijn in plaats van verstopt in een vergelijking.
    pub fn raadpleging_nodig(&self) -> bool {
        self.restrisico.as_ref().is_some_and(|r| r.niveau.is_hoog())
    }

    /// Wanneer het verzoek om raadpleging is ingediend.
    pub fn ingediend_op(&self) -> Option<DateTime<Utc>> {
        self.raadpleging.as_ref().map(|t| t.anker)
    }

    /// Wanneer het advies van de toezichthouder is vastgelegd.
    pub fn advies_ontvangen_op(&self) -> Option<DateTime<Utc>> {
        self.raadpleging.as_ref().and_then(|t| t.afgerond_op)
    }

    /// Hoeveel maanden er sinds de beoordeling zijn verstreken.
    ///
    /// Ruwe maat op dertig dagen, gelijk aan [`Herkomst::maanden_sinds_herziening`]:
    /// een herbeoordelingstermijn van zesendertig maanden hoeft niet op de dag
    /// nauwkeurig te zijn, en een exacte kalenderberekening zou hier een
    /// feestdagenkalender vergen die jaren vooruit moet reiken.
    pub fn maanden_sinds_beoordeling(&self, nu: DateTime<Utc>) -> Option<i64> {
        self.datum.map(|d| (nu - d).num_days() / 30)
    }

    /// Legt vast dat en wanneer er is beoordeeld.
    ///
    /// Weigert uitsluitend een datum in de toekomst. Een datum ná de aanvang
    /// van de verwerking wordt **niet** geweigerd: dat is een feit dat is
    /// voorgevallen, geen invoerfout, en regel DPIA-03 maakt het zichtbaar.
    /// Weigeren zou de gebruiker dwingen te liegen om verder te komen.
    pub fn leg_beoordeling_vast(
        &mut self,
        datum: DateTime<Utc>,
        uitgevoerd_door: impl Into<String>,
        vooraf: Option<bool>,
        op: DateTime<Utc>,
    ) -> Resultaat<()> {
        if datum > op {
            return Err(DomeinFout::OnmogelijkTijdstip {
                veld: "dpia.datum".into(),
                reden: "de beoordeling zou in de toekomst zijn uitgevoerd; controleer de datum"
                    .into(),
            });
        }
        let door = uitgevoerd_door.into();
        if door.trim().is_empty() {
            return Err(DomeinFout::OngeldigeWaarde {
                veld: "dpia.uitgevoerd_door".into(),
                reden: "noteer wie de beoordeling heeft uitgevoerd; zonder naam is zij niet \
                        aan iemand toe te schrijven"
                    .into(),
            });
        }
        self.datum = Some(datum);
        self.uitgevoerd_door = Some(door);
        if vooraf.is_some() {
            self.vooraf_uitgevoerd = vooraf;
        }
        self.herkomst.wijzig("beoordeling vastgelegd", op);
        Ok(())
    }

    /// Stelt het restrisico vast.
    ///
    /// Het niveau komt van de gebruiker; de tool kiest het nooit. Wat de tool
    /// wél afdwingt, is dat er iets is om tegen af te wegen: een restrisico is
    /// per definitie wat er overblijft ná de maatregelen.
    pub fn stel_restrisico_vast(
        &mut self,
        niveau: Restrisiconiveau,
        motivering: Motivering,
        op: DateTime<Utc>,
    ) -> Resultaat<()> {
        if self.datum.is_none() {
            return Err(DomeinFout::OngeldigeWaarde {
                veld: "dpia.restrisico".into(),
                reden: "leg eerst vast wanneer en door wie is beoordeeld".into(),
            });
        }
        if self.risicos.is_empty() {
            return Err(DomeinFout::OngeldigeWaarde {
                veld: "dpia.restrisico".into(),
                reden: "benoem eerst de risico's voor de rechten en vrijheden van betrokkenen \
                        (art. 35 lid 7 onder c)"
                    .into(),
            });
        }
        if self.maatregelen.is_empty() {
            return Err(DomeinFout::OngeldigeWaarde {
                veld: "dpia.restrisico".into(),
                reden: "een restrisico is wat overblijft ná de maatregelen; leg die eerst vast \
                        (art. 35 lid 7 onder d)"
                    .into(),
            });
        }
        self.restrisico =
            Some(Restrisico { niveau, motivering, gewogen_maatregelen: self.maatregelen.len() });
        self.herkomst.wijzig("restrisico vastgesteld", op);
        Ok(())
    }

    /// Legt vast dat de toezichthouder om voorafgaande raadpleging is gevraagd.
    ///
    /// De klok is elders gestart; dit dossier bewaart hem en bewaakt de
    /// invarianten. Zo blijft de rekenkunde in de termijnenmotor en hoeft het
    /// domein geen tijdzones te kennen.
    pub fn dien_raadpleging_in(
        &mut self,
        klok: LopendeTermijn,
        op: DateTime<Utc>,
    ) -> Resultaat<()> {
        if self.raadpleging.is_some() {
            return Err(DomeinFout::OngeldigeStatusovergang {
                van: "raadpleging ingediend".into(),
                naar: "raadpleging ingediend".into(),
                reden: "er loopt al een verzoek om voorafgaande raadpleging voor dit dossier"
                    .into(),
            });
        }
        let Some(datum) = self.datum else {
            return Err(DomeinFout::OngeldigeWaarde {
                veld: "dpia.raadpleging".into(),
                reden: "de effectbeoordeling gaat mee met het verzoek; leg haar eerst vast \
                        (art. 36 lid 3 onder e)"
                    .into(),
            });
        };
        if klok.anker < datum {
            return Err(DomeinFout::OnmogelijkTijdstip {
                veld: "dpia.raadpleging".into(),
                reden: "het verzoek zou zijn ingediend vóórdat de beoordeling was uitgevoerd; \
                        controleer welk van de twee tijdstippen verwisseld is"
                    .into(),
            });
        }
        self.raadpleging = Some(klok);
        self.herkomst.wijzig("verzoek om voorafgaande raadpleging ingediend", op);
        Ok(())
    }

    /// Legt het advies van de toezichthouder vast en rondt de klok af.
    pub fn leg_advies_vast(
        &mut self,
        ontvangen_op: DateTime<Utc>,
        referentie: impl Into<String>,
        op: DateTime<Utc>,
    ) -> Resultaat<()> {
        if ontvangen_op > op {
            return Err(DomeinFout::OnmogelijkTijdstip {
                veld: "dpia.advies_referentie".into(),
                reden: "het advies zou in de toekomst zijn ontvangen; controleer de datum. Een \
                        datum die vooruit ligt, zet bovendien de bewaking van de \
                        raadplegingstermijn uit"
                    .into(),
            });
        }
        let Some(klok) = self.raadpleging.as_mut() else {
            return Err(DomeinFout::OntbrekendeVerwijzing {
                veld: "dpia.raadpleging".into(),
                naar: "een ingediend verzoek om voorafgaande raadpleging".into(),
            });
        };
        if ontvangen_op < klok.anker {
            return Err(DomeinFout::OnmogelijkTijdstip {
                veld: "dpia.advies_referentie".into(),
                reden: "het advies zou zijn ontvangen vóórdat het verzoek was ingediend".into(),
            });
        }
        klok.rond_af(ontvangen_op);
        self.advies_referentie = Some(referentie.into());
        self.herkomst.wijzig("advies van de toezichthouder vastgelegd", op);
        Ok(())
    }

    /// Stelt het dossier vast.
    pub fn stel_vast(&mut self, door: impl Into<String>, op: DateTime<Utc>) -> Resultaat<()> {
        let rapport = self.volledigheid();
        if !rapport.mag_vaststellen() {
            return Err(DomeinFout::NietVolledig {
                soort: "effectbeoordeling".into(),
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

    /// Markeert het dossier als te herzien.
    ///
    /// Gebeurt wanneer de onderliggende verwerking verandert: artikel 35 lid 11
    /// vraagt om herbeoordeling wanneer het risico van de verwerking wijzigt.
    pub fn markeer_herziening_nodig(&mut self, reden: impl Into<String>, op: DateTime<Utc>) {
        if self.status == Status::Vastgesteld {
            self.status = Status::HerzieningNodig;
        }
        self.herkomst.wijzig(format!("systeem: {}", reden.into()), op);
    }
}

impl Volledig for Dpia {
    fn soortnaam(&self) -> &'static str {
        "effectbeoordeling"
    }

    fn aantal_verplichte_onderdelen(&self) -> usize {
        // De voortoets en haar motivering staan er altijd. Zegt de voortoets
        // dat er geen beoordeling nodig is, dan is het dossier daarmee klaar:
        // de teller vraagt niet om een beoordeling die volgens het vastgelegde
        // oordeel niet hoeft.
        let vast = 2;
        if self.voortoets == Some(Voortoets::NietNodig) {
            return vast;
        }
        // Zolang de voortoets nog niet is beantwoord, weet de teller niet wat
        // er komt. Hij raadt dan niet: twee blijft twee.
        if self.voortoets.is_none() {
            return vast;
        }

        // De beoordeling zelf: datum, uitvoerder, methode, het moment ten
        // opzichte van de verwerking, de vier onderdelen van lid 7 en het
        // restrisico.
        let mut afgeleid = 9;
        // Het advies van de functionaris (art. 35 lid 2).
        afgeleid += 1;
        // Is het restrisico hoog, dan komt de raadpleging erbij.
        if self.raadpleging_nodig() {
            afgeleid += 1;
        }
        vast + afgeleid
    }

    fn ontbrekende_onderdelen(&self) -> Vec<Ontbrekend> {
        let mut uit = Vec::new();

        if self.voortoets.is_none() {
            uit.push(Ontbrekend::blokkerend(
                "dpia.voortoets",
                "beantwoord of een effectbeoordeling nodig is; de criteria die deze \
                 verwerking raakt staan bij 'register toon'",
                "art. 35 lid 1 AVG",
            ));
        }
        // De motivering telt ook wanneer de uitkomst nog ontbreekt: juist een
        // besluit dat er géén beoordeling nodig is, moet later te volgen zijn.
        if self.voortoets_motivering.is_none() {
            uit.push(Ontbrekend::blokkerend(
                "dpia.voortoets_motivering",
                "schrijf op waaróm de beoordeling wel of niet nodig is",
                "art. 5 lid 2 AVG",
            ));
        }

        if self.voortoets == Some(Voortoets::NietNodig) || self.voortoets.is_none() {
            return uit;
        }

        if self.datum.is_none() {
            uit.push(Ontbrekend::blokkerend(
                "dpia.datum",
                "leg vast wanneer de beoordeling is uitgevoerd",
                "art. 35 lid 1 AVG",
            ));
        }
        if self.uitgevoerd_door.is_none() {
            uit.push(Ontbrekend::blokkerend(
                "dpia.uitgevoerd_door",
                "noteer wie de beoordeling heeft uitgevoerd",
                "art. 35 lid 2 AVG",
            ));
        }
        if self.systematische_beschrijving.is_none() {
            uit.push(Ontbrekend::blokkerend(
                "dpia.systematische_beschrijving",
                "beschrijf de beoogde verwerkingen en de doeleinden systematisch",
                "art. 35 lid 7 onder a AVG",
            ));
        }
        if self.noodzaak_en_evenredigheid.is_none() {
            uit.push(Ontbrekend::blokkerend(
                "dpia.noodzaak_en_evenredigheid",
                "beoordeel de noodzaak en de evenredigheid ten opzichte van de doeleinden",
                "art. 35 lid 7 onder b AVG",
            ));
        }
        if self.risicos.is_empty() {
            uit.push(Ontbrekend::blokkerend(
                "dpia.risicos",
                "benoem de risico's voor de rechten en vrijheden van betrokkenen",
                "art. 35 lid 7 onder c AVG",
            ));
        }
        if self.maatregelen.is_empty() {
            uit.push(Ontbrekend::blokkerend(
                "dpia.maatregelen",
                "benoem de maatregelen waarmee de risico's worden aangepakt",
                "art. 35 lid 7 onder d AVG",
            ));
        }
        if self.restrisico.is_none() {
            uit.push(Ontbrekend::blokkerend(
                "dpia.restrisico",
                "weeg wat er ná de maatregelen aan risico overblijft",
                "art. 35 lid 7 onder c en d AVG",
            ));
        }

        // Signalerend: de verordening schrijft geen methode voor.
        if self.methode.is_none() {
            uit.push(Ontbrekend::signalerend(
                "dpia.methode",
                "noteer welke methode is gebruikt, zodat de beoordeling herhaalbaar is",
                "geen wettelijk voorschrift; interne norm",
            ));
        }
        if self.vooraf_uitgevoerd.is_none() {
            uit.push(Ontbrekend::signalerend(
                "dpia.vooraf_uitgevoerd",
                "beantwoord of de beoordeling vóór de verwerking is uitgevoerd",
                "art. 35 lid 1 AVG",
            ));
        }
        // Signalerend en niet blokkerend: artikel 35 lid 2 geldt "indien deze
        // is aangewezen", en of er een functionaris is aangewezen kan dit
        // product nog niet vaststellen. Blokkeren zou op een aanname rusten.
        if self.advies_functionaris.is_none() {
            uit.push(Ontbrekend::signalerend(
                "dpia.advies_functionaris",
                "vraag het advies van de functionaris en leg het vast",
                "art. 35 lid 2 AVG",
            ));
        }
        // Signalerend: blokkeren zou het dossier acht weken op slot zetten,
        // terwijl juist de vastgestelde beoordeling het verzoek moet
        // vergezellen (art. 36 lid 3 onder e).
        if self.raadpleging_nodig() && self.raadpleging.is_none() {
            uit.push(Ontbrekend::signalerend(
                "dpia.raadpleging",
                "het restrisico is hoog beoordeeld; raadpleeg de toezichthouder vooraf",
                "art. 36 lid 1 AVG",
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

    fn dossier() -> Dpia {
        Dpia::nieuw("DPIA-0412", "Verzuimregistratie", Id::nieuw(), "u1", nu())
    }

    #[test]
    fn een_leeg_dossier_vraagt_alleen_om_de_voortoets() {
        let d = dossier();
        let rapport = d.volledigheid();
        assert_eq!(rapport.verplicht, 2);
        assert_eq!(rapport.compleet, 0);
        assert_eq!(rapport.ontbreekt.len(), 2);
    }

    #[test]
    fn een_gemotiveerd_niet_nodig_sluit_het_dossier() {
        let mut d = dossier();
        d.voortoets = Some(Voortoets::NietNodig);
        d.voortoets_motivering = Some(motivering("geen van de criteria wordt geraakt"));

        let rapport = d.volledigheid();
        assert_eq!(rapport.verplicht, 2);
        assert!(rapport.ontbreekt.is_empty());
        assert!(rapport.mag_vaststellen());
    }

    #[test]
    fn een_niet_nodig_zonder_motivering_blokkeert() {
        let mut d = dossier();
        d.voortoets = Some(Voortoets::NietNodig);
        assert!(!d.mag_vaststellen(), "juist een negatief besluit moet te volgen zijn");
    }

    #[test]
    fn een_restrisico_zonder_maatregelen_wordt_geweigerd() {
        let mut d = dossier();
        d.leg_beoordeling_vast(nu(), "A. de Vries", Some(true), nu()).unwrap();
        d.risicos.push("onbevoegde inzage door collega's".into());

        let fout = d
            .stel_restrisico_vast(Restrisiconiveau::Laag, motivering("beperkte kring"), nu())
            .unwrap_err();
        assert!(fout.to_string().contains("overblijft ná de maatregelen"), "kreeg: {fout}");
    }

    #[test]
    fn een_restrisico_zonder_risicos_wordt_geweigerd() {
        let mut d = dossier();
        d.leg_beoordeling_vast(nu(), "A. de Vries", Some(true), nu()).unwrap();
        d.maatregelen.push("toegang op rolbasis".into());
        assert!(d
            .stel_restrisico_vast(Restrisiconiveau::Laag, motivering("beperkte kring"), nu())
            .is_err());
    }

    #[test]
    fn het_restrisico_legt_vast_waartegen_is_afgewogen() {
        let mut d = dossier();
        d.leg_beoordeling_vast(nu(), "A. de Vries", Some(true), nu()).unwrap();
        d.risicos.push("onbevoegde inzage".into());
        d.maatregelen.push("toegang op rolbasis".into());
        d.maatregelen.push("logging op inzage".into());
        d.stel_restrisico_vast(Restrisiconiveau::Gemiddeld, motivering("beperkte kring"), nu())
            .unwrap();

        assert_eq!(d.restrisico.as_ref().unwrap().gewogen_maatregelen, 2);
        assert!(!d.raadpleging_nodig());
    }

    #[test]
    fn een_hoog_restrisico_roept_de_raadpleging_op() {
        let mut d = dossier();
        d.voortoets = Some(Voortoets::Vereist);
        d.leg_beoordeling_vast(nu(), "A. de Vries", Some(true), nu()).unwrap();
        d.risicos.push("stelselmatige monitoring".into());
        d.maatregelen.push("beperkte bewaartermijn".into());
        d.stel_restrisico_vast(Restrisiconiveau::Hoog, motivering("het risico blijft groot"), nu())
            .unwrap();

        assert!(d.raadpleging_nodig());
        let velden: Vec<_> = d.ontbrekende_onderdelen().into_iter().map(|o| o.veld).collect();
        assert!(velden.iter().any(|v| v == "dpia.raadpleging"));
    }

    #[test]
    fn een_beoordeling_in_de_toekomst_wordt_geweigerd() {
        let mut d = dossier();
        let fout = d
            .leg_beoordeling_vast(nu() + chrono::Duration::days(1), "A. de Vries", None, nu())
            .unwrap_err();
        assert!(fout.to_string().contains("toekomst"));
    }

    /// Een beoordeling ná de aanvang van de verwerking is een feit dat is
    /// voorgevallen, geen invoerfout. Weigeren zou de gebruiker dwingen te
    /// liegen om verder te komen; regel DPIA-03 maakt het zichtbaar.
    #[test]
    fn een_te_late_beoordeling_wordt_wel_vastgelegd() {
        let mut d = dossier();
        d.leg_beoordeling_vast(nu(), "A. de Vries", Some(false), nu()).unwrap();
        assert_eq!(d.vooraf_uitgevoerd, Some(false));
    }

    #[test]
    fn een_advies_in_de_toekomst_wordt_geweigerd() {
        let mut d = dossier();
        d.voortoets = Some(Voortoets::Vereist);
        d.leg_beoordeling_vast(nu(), "A. de Vries", Some(true), nu()).unwrap();
        let fout = d
            .leg_advies_vast(nu() + chrono::Duration::days(365), "AP-2026-1234", nu())
            .unwrap_err();
        assert!(fout.to_string().contains("in de toekomst"), "kreeg: {fout}");
    }

    #[test]
    fn advies_zonder_verzoek_wordt_geweigerd() {
        let mut d = dossier();
        let fout = d.leg_advies_vast(nu(), "AP-2026-1234", nu()).unwrap_err();
        assert!(matches!(fout, DomeinFout::OntbrekendeVerwijzing { .. }));
    }

    #[test]
    fn de_teller_klopt_bij_elke_uitkomst_van_de_voortoets() {
        for uitkomst in Voortoets::alle() {
            let mut d = dossier();
            d.voortoets = Some(uitkomst);
            let verwacht = match uitkomst {
                Voortoets::NietNodig => 2,
                _ => 12,
            };
            assert_eq!(d.aantal_verplichte_onderdelen(), verwacht, "bij voortoets {uitkomst:?}");
        }
    }

    #[test]
    fn de_teller_telt_nooit_minder_onderdelen_dan_er_ontbreken() {
        for uitkomst in [None, Some(Voortoets::NietNodig), Some(Voortoets::Vereist)] {
            let mut d = dossier();
            d.voortoets = uitkomst;
            let rapport = d.volledigheid();
            assert!(
                rapport.ontbreekt.len() <= rapport.verplicht,
                "meer ontbrekend dan verplicht bij {uitkomst:?}: {} van {}",
                rapport.ontbreekt.len(),
                rapport.verplicht
            );
        }
    }

    #[test]
    fn geen_enkel_ontbrekend_onderdeel_wordt_dubbel_gemeld() {
        let mut d = dossier();
        d.voortoets = Some(Voortoets::Vereist);
        let velden: Vec<_> = d.ontbrekende_onderdelen().into_iter().map(|o| o.veld).collect();
        let uniek: std::collections::BTreeSet<_> = velden.iter().collect();
        assert_eq!(velden.len(), uniek.len(), "dubbele melding in {velden:?}");
    }

    #[test]
    fn elk_ontbrekend_onderdeel_draagt_een_grondslag() {
        let mut d = dossier();
        d.voortoets = Some(Voortoets::Vereist);
        for o in d.ontbrekende_onderdelen() {
            assert!(!o.grondslag.is_empty(), "{} mist een grondslag", o.veld);
            assert!(!o.omschrijving.is_empty(), "{} mist een omschrijving", o.veld);
        }
    }

    #[test]
    fn het_dossier_overleeft_serialisatie() {
        let mut d = dossier();
        d.voortoets = Some(Voortoets::Vereist);
        d.voortoets_motivering = Some(motivering("twee criteria worden geraakt"));
        d.leg_beoordeling_vast(nu(), "A. de Vries", Some(true), nu()).unwrap();
        d.risicos.push("onbevoegde inzage".into());
        d.maatregelen.push("toegang op rolbasis".into());
        d.stel_restrisico_vast(Restrisiconiveau::Hoog, motivering("blijft groot"), nu()).unwrap();

        let json = serde_json::to_string(&d).unwrap();
        let terug: Dpia = serde_json::from_str(&json).unwrap();
        assert_eq!(d, terug);
    }
}
