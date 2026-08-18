//! Het verwerkingsregister van artikel 30 AVG.
//!
//! # Waarom dit record zoveel afdwingt
//!
//! Uit het foutbestendigheidshoofdstuk, paragraaf 3.4: de tool leidt de
//! verplichtingen af uit de gegeven antwoorden, zodat de gebruiker de regel
//! niet hoeft te kennen. Concreet:
//!
//! | Antwoord | Wat er automatisch verplicht wordt |
//! |---|---|
//! | bijzondere categorie aangevinkt | uitzonderingsgrond art. 9 lid 2 |
//! | uitzondering b, g, h, i of j gekozen | daarnaast een bepaling uit nationaal recht |
//! | grondslag = gerechtvaardigd belang | belangenafweging |
//! | grondslag = toestemming | bewijsvorm en intrekkingsroute |
//! | grondslag = wettelijke verplichting of algemeen belang | aanwijsbare wettelijke bepaling |
//! | burgerservicenummer gebruikt | wettelijke grondslag voor dat gebruik |
//! | strafrechtelijke gegevens | uitzonderingsgrond uit de UAVG |
//! | verwerker gekoppeld | verwerkersovereenkomst |
//! | doorgifte buiten de EER | waarborg uit hoofdstuk V |
//!
//! Geen van deze afleidingen is een waarschuwing. Ze verschijnen als
//! openstaand onderdeel in het dossier en houden vaststelling tegen zolang ze
//! niet zijn ingevuld.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    avg::{BijzondereCategorie, Grondslag, Rol, UitzonderingArtikel9},
    basis::{Compartiment, Herkomst, Id, Motivering, Overgenomen, Status},
    volledigheid::{Ontbrekend, Volledig},
    DomeinFout, Resultaat,
};

/// Hoe lang gegevens worden bewaard.
///
/// Bewust geen los tekstveld: "zolang als nodig" is geen bewaartermijn maar het
/// ontbreken ervan. Wie de termijn nog niet kent, legt dat expliciet vast met
/// een datum waarop het wél bekend moet zijn — dan blijft het zichtbaar in
/// plaats van te verdwijnen in een zin.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "soort", rename_all = "snake_case")]
pub enum Bewaartermijn {
    /// Een vaste termijn met een aanwijsbare grondslag.
    Vast {
        duur: u32,
        eenheid: crate::Termijneenheid,
        /// Waarop de termijn berust: een wet, een selectielijst, een besluit.
        grondslag: String,
        /// Vanaf welke gebeurtenis de termijn loopt.
        vanaf: String,
    },
    /// De gegevens worden bewaard zolang een toestand voortduurt, met een
    /// aanwijsbaar einde en een opruimtermijn daarna.
    ZolangToestand {
        toestand: String,
        na_afloop_duur: u32,
        na_afloop_eenheid: crate::Termijneenheid,
        grondslag: String,
    },
    /// Nog niet vastgesteld, met een afspraak wanneer dat gebeurt.
    ///
    /// Dit is een geldige toestand — maar wel een zichtbare. Zie
    /// `openstaande uitstelafspraken` in de meetnormen.
    NogTeBepalen {
        motivering: Motivering,
        uiterlijk_bepaald_op: DateTime<Utc>,
        eigenaar: String,
    },
}

impl Bewaartermijn {
    pub fn is_vastgesteld(&self) -> bool {
        !matches!(self, Self::NogTeBepalen { .. })
    }

    /// Of de uitstelafspraak is verstreken.
    pub fn uitstel_verlopen(&self, nu: DateTime<Utc>) -> bool {
        matches!(self, Self::NogTeBepalen { uiterlijk_bepaald_op, .. } if nu > *uiterlijk_bepaald_op)
    }
}

/// Een categorie ontvangers van de gegevens.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ontvanger {
    pub omschrijving: String,
    /// Of deze ontvanger als verwerker optreedt; dan is een overeenkomst nodig.
    pub is_verwerker: bool,
    /// Verwijzing naar het leveranciersdossier, wanneer bekend.
    pub leverancier_id: Option<Id>,
    /// Of de ontvanger buiten de Europese Economische Ruimte zit.
    pub buiten_eer: bool,
}

/// Een verwerking in het register.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Verwerking {
    pub id: Id,
    /// Vast kenmerk voor verwijzing in correspondentie.
    pub kenmerk: String,
    pub naam: String,
    pub status: Status,
    pub compartiment: Compartiment,
    pub herkomst: Herkomst,
    /// Gevuld wanneer deze regel uit een eerdere administratie komt.
    pub overgenomen: Option<Overgenomen>,

    // --- art. 30 lid 1 onder b: doeleinden ---
    pub rol: Rol,
    pub doeleinden: Vec<String>,

    // --- grondslag ---
    pub grondslag: Option<Grondslag>,
    /// Onderbouwing van de grondslagkeuze. Altijd verplicht: de keuze zelf zegt
    /// niets zonder de reden.
    pub grondslag_motivering: Option<Motivering>,
    /// De bepaling waarop een wettelijke verplichting of taak van algemeen
    /// belang berust.
    pub wettelijke_bepaling: Option<String>,
    /// Verwijzing naar de belangenafweging bij een gerechtvaardigd belang.
    pub belangenafweging_id: Option<Id>,
    /// Verwijzing naar het toestemmingsdossier.
    pub toestemming_id: Option<Id>,

    // --- art. 30 lid 1 onder c: betrokkenen en gegevens ---
    pub categorieen_betrokkenen: Vec<String>,
    pub categorieen_gegevens: Vec<String>,
    pub bijzondere_categorieen: Vec<BijzondereCategorie>,
    pub uitzondering_artikel9: Option<UitzonderingArtikel9>,
    /// De nationale bepaling die de uitzondering van artikel 9 draagt.
    pub uitzondering_nationale_bepaling: Option<String>,
    pub strafrechtelijke_gegevens: bool,
    /// De uitzonderingsgrond uit de UAVG voor strafrechtelijke gegevens.
    pub uitzondering_strafrechtelijk: Option<String>,
    pub burgerservicenummer: bool,
    /// De wettelijke grondslag voor het gebruik van het burgerservicenummer.
    pub bsn_grondslag: Option<String>,
    /// Of er betrokkenen jonger dan zestien jaar zijn.
    pub minderjarigen: bool,

    // --- art. 30 lid 1 onder d en e: ontvangers en doorgiften ---
    pub ontvangers: Vec<Ontvanger>,
    /// Verwijzingen naar doorgiftedossiers voor ontvangers buiten de EER.
    pub doorgiften: Vec<Id>,

    // --- art. 30 lid 1 onder f: bewaartermijn ---
    pub bewaartermijn: Option<Bewaartermijn>,

    // --- art. 30 lid 1 onder g: beveiliging ---
    pub beveiligingsmaatregelen: Option<String>,

    // --- verbanden ---
    pub systemen: Vec<Id>,
    /// Verwijzingen naar verwerkersovereenkomsten per verwerker.
    pub verwerkersovereenkomsten: Vec<Id>,
    /// Verwijzing naar de gegevensbeschermingseffectbeoordeling.
    pub dpia_id: Option<Id>,
    /// Verwijzing naar de regeling bij gezamenlijke verantwoordelijkheid.
    pub gezamenlijke_regeling_id: Option<Id>,
    /// Verwijzing naar het dossier geautomatiseerde besluitvorming.
    pub geautomatiseerde_besluitvorming_id: Option<Id>,
    /// Of er uitsluitend geautomatiseerd wordt besloten met rechtsgevolg.
    pub uitsluitend_geautomatiseerde_besluitvorming: bool,

    pub eigenaar: String,
}

impl Verwerking {
    /// Maakt een nieuwe verwerking aan als concept.
    ///
    /// Bewust weinig verplichte parameters: een concept mag onvolledig zijn.
    /// Wat ontbreekt is zichtbaar via [`Volledig::volledigheid`], niet via een
    /// foutmelding bij het aanmaken.
    pub fn nieuw(
        kenmerk: impl Into<String>,
        naam: impl Into<String>,
        rol: Rol,
        eigenaar: impl Into<String>,
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
            overgenomen: None,
            rol,
            doeleinden: Vec::new(),
            grondslag: None,
            grondslag_motivering: None,
            wettelijke_bepaling: None,
            belangenafweging_id: None,
            toestemming_id: None,
            categorieen_betrokkenen: Vec::new(),
            categorieen_gegevens: Vec::new(),
            bijzondere_categorieen: Vec::new(),
            uitzondering_artikel9: None,
            uitzondering_nationale_bepaling: None,
            strafrechtelijke_gegevens: false,
            uitzondering_strafrechtelijk: None,
            burgerservicenummer: false,
            bsn_grondslag: None,
            minderjarigen: false,
            ontvangers: Vec::new(),
            doorgiften: Vec::new(),
            bewaartermijn: None,
            beveiligingsmaatregelen: None,
            systemen: Vec::new(),
            verwerkersovereenkomsten: Vec::new(),
            dpia_id: None,
            gezamenlijke_regeling_id: None,
            geautomatiseerde_besluitvorming_id: None,
            uitsluitend_geautomatiseerde_besluitvorming: false,
            eigenaar: eigenaar.into(),
        }
    }

    /// Of er bijzondere categorieën in het spel zijn.
    pub fn heeft_bijzondere_gegevens(&self) -> bool {
        !self.bijzondere_categorieen.is_empty()
    }

    /// Of er verwerkers zijn gekoppeld.
    pub fn heeft_verwerkers(&self) -> bool {
        self.ontvangers.iter().any(|o| o.is_verwerker)
    }

    /// Hoeveel verwerkers er zijn.
    pub fn aantal_verwerkers(&self) -> usize {
        self.ontvangers.iter().filter(|o| o.is_verwerker).count()
    }

    /// Of er ontvangers buiten de EER zijn.
    pub fn heeft_doorgifte_buiten_eer(&self) -> bool {
        self.ontvangers.iter().any(|o| o.buiten_eer)
    }

    /// Hoeveel ontvangers buiten de EER er zijn.
    pub fn aantal_ontvangers_buiten_eer(&self) -> usize {
        self.ontvangers.iter().filter(|o| o.buiten_eer).count()
    }

    /// Hoeveel van de negen criteria uit de richtsnoeren voor een
    /// gegevensbeschermingseffectbeoordeling deze verwerking raakt.
    ///
    /// Bij twee of meer criteria is een beoordeling in beginsel verplicht. Deze
    /// telling is een **hulpmiddel bij het gesprek**, geen oordeel: de tool
    /// telt de criteria die zij uit het register kan afleiden en toont welke,
    /// zodat de functionaris de overige zelf kan beoordelen.
    pub fn getelde_dpia_criteria(&self) -> Vec<&'static str> {
        let mut criteria = Vec::new();
        if self.uitsluitend_geautomatiseerde_besluitvorming {
            criteria.push("evaluatie of scoretoekenning met rechtsgevolg");
        }
        if self.heeft_bijzondere_gegevens() {
            criteria.push("gevoelige gegevens of gegevens van zeer persoonlijke aard");
        }
        if self.strafrechtelijke_gegevens {
            criteria.push("strafrechtelijke gegevens");
        }
        if self.minderjarigen {
            criteria.push("kwetsbare betrokkenen");
        }
        if self.heeft_doorgifte_buiten_eer() {
            criteria.push("doorgifte buiten de Europese Economische Ruimte");
        }
        criteria
    }

    /// Of de telling wijst op een verplichte beoordeling.
    pub fn dpia_waarschijnlijk_verplicht(&self) -> bool {
        self.getelde_dpia_criteria().len() >= 2
    }

    /// Stelt de verwerking vast.
    ///
    /// Faalt met een opsomming van wat er ontbreekt, in plaats van met een
    /// algemene melding. De gebruiker moet uit de fout kunnen opmaken wat hij
    /// moet doen.
    pub fn stel_vast(&mut self, door: impl Into<String>, op: DateTime<Utc>) -> Resultaat<()> {
        let rapport = self.volledigheid();
        if !rapport.mag_vaststellen() {
            return Err(DomeinFout::NietVolledig {
                soort: "verwerking".into(),
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

    /// Markeert de verwerking als te herzien.
    ///
    /// Gebeurt automatisch wanneer iets verandert waarvan deze verwerking
    /// afhangt: een ingetrokken adequaatheidsbesluit, een gewijzigde
    /// subverwerkerslijst, een nieuwe versie van een kennispakket.
    pub fn markeer_herziening_nodig(&mut self, reden: impl Into<String>, op: DateTime<Utc>) {
        if self.status == Status::Vastgesteld {
            self.status = Status::HerzieningNodig;
        }
        self.herkomst.wijzig(format!("systeem: {}", reden.into()), op);
    }
}

impl Volledig for Verwerking {
    fn soortnaam(&self) -> &'static str {
        "verwerking"
    }

    fn aantal_verplichte_onderdelen(&self) -> usize {
        // De vaste kern van artikel 30, plus de onderdelen die uit de gegeven
        // antwoorden volgen. De teller groeit dus mee met de complexiteit van
        // de verwerking — wat klopt: een verwerking met bijzondere gegevens
        // en een doorgifte heeft meer aan te tonen dan een adressenlijst.
        let vast = 8;
        let mut afgeleid = 0;
        if self.heeft_bijzondere_gegevens() {
            afgeleid += 1;
            if self
                .uitzondering_artikel9
                .is_some_and(|u| u.vereist_nationale_bepaling())
            {
                afgeleid += 1;
            }
        }
        if self.strafrechtelijke_gegevens {
            afgeleid += 1;
        }
        if self.burgerservicenummer {
            afgeleid += 1;
        }
        if self.grondslag.is_some_and(|g| g.vereist_belangenafweging()) {
            afgeleid += 1;
        }
        if self.grondslag.is_some_and(|g| g.vereist_toestemmingsbewijs()) {
            afgeleid += 1;
        }
        if self.grondslag.is_some_and(|g| g.vereist_wettelijke_bepaling()) {
            afgeleid += 1;
        }
        if self.heeft_verwerkers() {
            afgeleid += 1;
        }
        if self.heeft_doorgifte_buiten_eer() {
            afgeleid += 1;
        }
        if self.rol == Rol::GezamenlijkVerantwoordelijke {
            afgeleid += 1;
        }
        if self.uitsluitend_geautomatiseerde_besluitvorming {
            afgeleid += 1;
        }
        vast + afgeleid
    }

    fn ontbrekende_onderdelen(&self) -> Vec<Ontbrekend> {
        let mut uit = Vec::new();

        // --- vaste kern van artikel 30 ---
        if self.doeleinden.is_empty() {
            uit.push(Ontbrekend::blokkerend(
                "verwerking.doeleinden",
                "beschrijf waarvoor de gegevens worden verwerkt",
                "art. 30 lid 1 onder b AVG",
            ));
        }
        if self.categorieen_betrokkenen.is_empty() {
            uit.push(Ontbrekend::blokkerend(
                "verwerking.categorieen_betrokkenen",
                "benoem om wiens gegevens het gaat",
                "art. 30 lid 1 onder c AVG",
            ));
        }
        if self.categorieen_gegevens.is_empty() {
            uit.push(Ontbrekend::blokkerend(
                "verwerking.categorieen_gegevens",
                "benoem welke gegevens worden verwerkt",
                "art. 30 lid 1 onder c AVG",
            ));
        }
        if self.ontvangers.is_empty() {
            uit.push(Ontbrekend::blokkerend(
                "verwerking.ontvangers",
                "benoem aan wie de gegevens worden verstrekt, of leg vast dat dat aan niemand gebeurt",
                "art. 30 lid 1 onder d AVG",
            ));
        }
        match &self.bewaartermijn {
            None => uit.push(Ontbrekend::blokkerend(
                "verwerking.bewaartermijn",
                "leg vast hoe lang de gegevens worden bewaard",
                "art. 30 lid 1 onder f AVG",
            )),
            Some(b) if !b.is_vastgesteld() => uit.push(Ontbrekend::signalerend(
                "verwerking.bewaartermijn",
                "de bewaartermijn is uitgesteld en moet nog worden vastgesteld",
                "art. 30 lid 1 onder f AVG",
            )),
            _ => {}
        }
        if self.beveiligingsmaatregelen.is_none() {
            uit.push(Ontbrekend::blokkerend(
                "verwerking.beveiligingsmaatregelen",
                "beschrijf de technische en organisatorische maatregelen",
                "art. 30 lid 1 onder g en art. 32 AVG",
            ));
        }

        // --- grondslag ---
        // De motivering wordt altijd geteld, ook wanneer de grondslag zelf nog
        // ontbreekt. Anders zou een leeg record melden dat er al één onderdeel
        // compleet is, en dat is precies de misleiding die deze teller moet
        // voorkomen.
        if self.grondslag_motivering.is_none() {
            uit.push(Ontbrekend::blokkerend(
                "verwerking.grondslag_motivering",
                "onderbouw waarom de gekozen grondslag past bij dit doel",
                "art. 5 lid 2 AVG",
            ));
        }
        match self.grondslag {
            None => uit.push(Ontbrekend::blokkerend(
                "verwerking.grondslag",
                "kies de grondslag voor deze verwerking",
                "art. 6 lid 1 AVG",
            )),
            Some(g) => {
                if g.vereist_wettelijke_bepaling() && self.wettelijke_bepaling.is_none() {
                    uit.push(Ontbrekend::blokkerend(
                        "verwerking.wettelijke_bepaling",
                        "wijs de wettelijke bepaling aan waarop deze grondslag berust",
                        &format!("{}, en art. 6 lid 3 AVG", g.grondslagverwijzing()),
                    ));
                }
                if g.vereist_belangenafweging() && self.belangenafweging_id.is_none() {
                    uit.push(Ontbrekend::blokkerend(
                        "verwerking.belangenafweging",
                        "voer de belangenafweging uit; zonder afweging is er geen gerechtvaardigd belang",
                        "art. 6 lid 1 onder f AVG",
                    ));
                }
                if g.vereist_toestemmingsbewijs() && self.toestemming_id.is_none() {
                    uit.push(Ontbrekend::blokkerend(
                        "verwerking.toestemming",
                        "leg vast hoe de toestemming wordt verkregen, bewaard en ingetrokken",
                        "art. 7 lid 1 en lid 3 AVG",
                    ));
                }
            }
        }
        // --- bijzondere categorieën ---
        if self.heeft_bijzondere_gegevens() {
            match self.uitzondering_artikel9 {
                None => uit.push(Ontbrekend::blokkerend(
                    "verwerking.uitzondering_artikel9",
                    "kies de uitzondering die het verwerken van deze bijzondere gegevens toestaat",
                    "art. 9 lid 1 en lid 2 AVG",
                )),
                Some(u) if u.vereist_nationale_bepaling()
                    && self.uitzondering_nationale_bepaling.is_none() =>
                {
                    uit.push(Ontbrekend::blokkerend(
                        "verwerking.uitzondering_nationale_bepaling",
                        "wijs de bepaling uit het nationale recht aan die deze uitzondering draagt",
                        &format!("{}, uitgewerkt in de UAVG", u.grondslagverwijzing()),
                    ));
                }
                _ => {}
            }
        }

        if self.strafrechtelijke_gegevens && self.uitzondering_strafrechtelijk.is_none() {
            uit.push(Ontbrekend::blokkerend(
                "verwerking.uitzondering_strafrechtelijk",
                "wijs de uitzonderingsgrond aan voor het verwerken van strafrechtelijke gegevens",
                "art. 10 AVG en de UAVG",
            ));
        }

        if self.burgerservicenummer && self.bsn_grondslag.is_none() {
            uit.push(Ontbrekend::blokkerend(
                "verwerking.bsn_grondslag",
                "wijs de wettelijke bepaling aan die het gebruik van het burgerservicenummer toestaat",
                "art. 87 AVG en art. 46 UAVG",
            ));
        }

        // --- verwerkers ---
        if self.heeft_verwerkers()
            && self.verwerkersovereenkomsten.len() < self.aantal_verwerkers()
        {
            uit.push(Ontbrekend::blokkerend(
                "verwerking.verwerkersovereenkomsten",
                &format!(
                    "leg voor alle {} verwerkers een verwerkersovereenkomst vast; er {} nu {} gekoppeld",
                    self.aantal_verwerkers(),
                    if self.verwerkersovereenkomsten.len() == 1 { "is" } else { "zijn" },
                    self.verwerkersovereenkomsten.len()
                ),
                "art. 28 lid 3 AVG",
            ));
        }

        // --- doorgiften ---
        if self.heeft_doorgifte_buiten_eer()
            && self.doorgiften.len() < self.aantal_ontvangers_buiten_eer()
        {
            uit.push(Ontbrekend::blokkerend(
                "verwerking.doorgiften",
                &format!(
                    "leg voor alle {} ontvangers buiten de EER het doorgifte-instrument vast",
                    self.aantal_ontvangers_buiten_eer()
                ),
                "hoofdstuk V AVG",
            ));
        }

        // --- gezamenlijke verantwoordelijkheid ---
        if self.rol == Rol::GezamenlijkVerantwoordelijke && self.gezamenlijke_regeling_id.is_none()
        {
            uit.push(Ontbrekend::blokkerend(
                "verwerking.gezamenlijke_regeling",
                "leg de onderlinge regeling vast en publiceer de wezenlijke inhoud daarvan",
                "art. 26 lid 1 en lid 2 AVG",
            ));
        }

        // --- geautomatiseerde besluitvorming ---
        if self.uitsluitend_geautomatiseerde_besluitvorming
            && self.geautomatiseerde_besluitvorming_id.is_none()
        {
            uit.push(Ontbrekend::blokkerend(
                "verwerking.geautomatiseerde_besluitvorming",
                "leg de grondslag, de onderliggende logica, de gevolgen en de menselijke tussenkomst vast",
                "art. 22 en art. 13 lid 2 onder f AVG",
            ));
        }

        // --- effectbeoordeling ---
        if self.dpia_waarschijnlijk_verplicht() && self.dpia_id.is_none() {
            uit.push(Ontbrekend::signalerend(
                "verwerking.dpia",
                &format!(
                    "deze verwerking raakt {} van de criteria voor een effectbeoordeling; \
                     voer de toets uit of leg gemotiveerd vast waarom die niet nodig is",
                    self.getelde_dpia_criteria().len()
                ),
                "art. 35 lid 1 AVG",
            ));
        }

        // --- overgenomen zonder verificatie ---
        if let Some(o) = &self.overgenomen {
            if !o.is_geverifieerd() {
                uit.push(Ontbrekend::signalerend(
                    "verwerking.verificatie_overname",
                    &format!(
                        "deze regel is overgenomen uit {} en nog niet geverifieerd",
                        o.bron
                    ),
                    "art. 5 lid 2 AVG",
                ));
            }
        }

        uit
    }
}
