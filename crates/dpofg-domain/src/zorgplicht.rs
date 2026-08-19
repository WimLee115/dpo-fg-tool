//! De zorgplichtcontrolset: de maatregelen van artikel 21 lid 3 van de
//! Cyberbeveiligingswet, met bewijs dat een einddatum draagt.
//!
//! # Waarom de set wordt afgeleid en niet samengesteld
//!
//! Een controlset waarin de gebruiker zelf maatregelen toevoegt, is altijd
//! compleet: hij bevat immers precies wat er is bedacht. Dat is de kern van
//! het probleem. De tien onderdelen a tot en met j staan in de wet; wat er per
//! onderdeel aan maatregelen hoort, staat in het kennispakket. Het dossier
//! wordt daaruit in één handeling afgeleid en er is geen methode om er een
//! maatregel bij te zetten of uit te halen. Een set met negen van de tien
//! onderdelen is daarmee geen toestand die kan bestaan.
//!
//! # Waarom de stand niet te zetten is
//!
//! `Maatregelstand` is een berekening, geen veld. Er is geen `zet_stand` en
//! geen invoerroute waarmee iemand "aantoonbaar" kan aanvinken. Aantoonbaar
//! wordt een maatregel uitsluitend doordat er een bewijsstuk aan hangt met de
//! rol uitvoering waarvan het geldigheidsvenster op dit moment openstaat.
//! Verloopt dat venster, dan valt de maatregel vanzelf terug — er is niets dat
//! iemand moet bijwerken en dus ook niets dat iemand kan vergeten.
//!
//! Dat is strenger dan invariant I1 uit het foutbestendigheidshoofdstuk, die
//! alleen eist dat `geldig_tot` in de toekomst ligt. Een stuk dat pas volgende
//! maand ingaat, bewijst vandaag niets.
//!
//! # Wat dit dossier niet doet
//!
//! Geen score en geen volwassenheidsniveau. Het plan noemt daarvoor vier
//! veldnamen en geen enkele schaal, weging of aggregatieregel. Een getal dat
//! niet zegt waarop het is gebaseerd, is in een verantwoordingsgesprek erger
//! dan geen getal. Wat dit dossier wél afgeeft is de teller uit het
//! volledigheidsrapport, de stand per maatregel en de datum waarop bewijs
//! vervalt.
//!
//! Geen crosswalk naar informatiebeveiligingsnormen. Zolang die er niet is,
//! kan de tool ook niet suggereren dat een certificaat een wettelijke eis
//! afdekt — in deze eerste stap eerder een voordeel dan een gemis.

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    basis::{Compartiment, Herkomst, Id, Motivering, Status},
    error::{DomeinFout, Resultaat},
    volledigheid::{Ontbrekend, Volledig},
};

/// De tien onderdelen van artikel 21 lid 3 van de Cyberbeveiligingswet.
///
/// De letters en hun onderwerp zijn wetsstructuur en staan daarom in code,
/// net als de acht onderdelen van artikel 28 lid 3 AVG. Wat er per onderdeel
/// aan maatregelen hoort, is pakketinhoud.
///
/// De verwijzing luidt lid **3**. In NIS2 zelf staan de tien categorieën in
/// artikel 21 lid 2; de Nederlandse wet nummert anders. Die verwijzing staat
/// hier op één plaats, zodat de vergissing niet over het product verspreid kan
/// raken.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Zorgplichtonderdeel {
    Beleid,
    Incidenten,
    Continuiteit,
    Toeleveringsketen,
    Ontwikkeling,
    Effectiviteit,
    Cyberhygiene,
    Cryptografie,
    Personeel,
    Authenticatie,
}

impl Zorgplichtonderdeel {
    pub fn letter(&self) -> &'static str {
        match self {
            Self::Beleid => "a",
            Self::Incidenten => "b",
            Self::Continuiteit => "c",
            Self::Toeleveringsketen => "d",
            Self::Ontwikkeling => "e",
            Self::Effectiviteit => "f",
            Self::Cyberhygiene => "g",
            Self::Cryptografie => "h",
            Self::Personeel => "i",
            Self::Authenticatie => "j",
        }
    }

    /// Een verkorte aanduiding van het onderwerp.
    ///
    /// Dit is nadrukkelijk geen citaat van de wettekst maar een samenvatting;
    /// dat voorbehoud staat ook in `dpofg pakket voorbehoud`.
    pub fn omschrijving(&self) -> &'static str {
        match self {
            Self::Beleid => "beleid voor risicoanalyse en beveiliging van informatiesystemen",
            Self::Incidenten => "behandeling van incidenten",
            Self::Continuiteit => "bedrijfscontinuïteit, back-up, herstel en crisisbeheer",
            Self::Toeleveringsketen => "beveiliging van de toeleveringsketen",
            Self::Ontwikkeling => {
                "verwerving, ontwikkeling en onderhoud, met kwetsbaarhedenrespons"
            }
            Self::Effectiviteit => {
                "beleid en procedures om de doeltreffendheid van de maatregelen te beoordelen"
            }
            Self::Cyberhygiene => "cyberhygiëne en opleiding",
            Self::Cryptografie => "cryptografie en versleuteling",
            Self::Personeel => "personeel, toegangsbeleid en beheer van bedrijfsmiddelen",
            Self::Authenticatie => "meerfactorauthenticatie en beveiligde communicatie",
        }
    }

    pub fn grondslag(&self) -> String {
        format!("art. 21 lid 3 onder {} Cyberbeveiligingswet", self.letter())
    }

    pub fn alle() -> [Self; 10] {
        [
            Self::Beleid,
            Self::Incidenten,
            Self::Continuiteit,
            Self::Toeleveringsketen,
            Self::Ontwikkeling,
            Self::Effectiviteit,
            Self::Cyberhygiene,
            Self::Cryptografie,
            Self::Personeel,
            Self::Authenticatie,
        ]
    }
}

/// Uit welk soort normenkader de controlset is afgeleid.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Raamwerkvariant {
    /// De uitwerking van de zorgplicht in het Cyberbeveiligingsbesluit.
    A,
    /// Het raamwerk uit de uitvoeringsverordening, voor de entiteitstypen
    /// waarvoor dat in de plaats komt.
    B,
    /// Een bij regeling voorgeschreven normenkader. Afwijken vereist dan een
    /// grondslag in die regeling zelf.
    C,
}

impl Raamwerkvariant {
    pub fn letter(&self) -> &'static str {
        match self {
            Self::A => "A",
            Self::B => "B",
            Self::C => "C",
        }
    }

    pub fn omschrijving(&self) -> &'static str {
        match self {
            Self::A => "de uitwerking van de zorgplicht in het Cyberbeveiligingsbesluit",
            Self::B => "het raamwerk uit de uitvoeringsverordening",
            Self::C => "een bij regeling voorgeschreven normenkader",
        }
    }
}

/// Wat een bewijsstuk bewijst.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Bewijsrol {
    /// Dat de maatregel is vastgesteld: beleid, besluit, procedure.
    Vaststelling,
    /// Dat de maatregel is uitgevoerd: verslag, uitdraai, logboek.
    Uitvoering,
    /// Dat een ander ernaar heeft gekeken: audit, penetratietest, verklaring.
    Toetsing,
}

impl Bewijsrol {
    pub fn omschrijving(&self) -> &'static str {
        match self {
            Self::Vaststelling => "vaststelling",
            Self::Uitvoering => "uitvoering",
            Self::Toetsing => "toetsing",
        }
    }
}

/// Of het bewijs op de eigen verklaring berust of door een ander is getoetst.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Bewijskracht {
    Zelfgerapporteerd,
    Geverifieerd,
}

impl Bewijskracht {
    pub fn omschrijving(&self) -> &'static str {
        match self {
            Self::Zelfgerapporteerd => "zelfgerapporteerd",
            Self::Geverifieerd => "door een ander geverifieerd",
        }
    }
}

/// Op welke wijze van deze maatregel mag worden afgeweken.
///
/// Dit is norminhoud en komt uit het kennispakket. Het plan spreekt zichzelf
/// tegen over de vraag of de motiveringsplicht alleen in variant B geldt of in
/// alle drie de varianten; die vraag is juridisch en hoort daarom in het
/// pakket beantwoord te worden, per maatregel, en niet in de programmacode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Niettoepassingsvorm {
    /// De eis is onvoorwaardelijk geformuleerd; niet toepassen is geen keuze.
    Verboden,
    /// De eis is met een voorbehoud geformuleerd; afwijken vraagt een eigen
    /// motivering.
    EigenMotivering,
    /// Het kader is voorgeschreven; afwijken vraagt een grondslag in de
    /// regeling zelf.
    GrondslagInRegeling,
}

impl Niettoepassingsvorm {
    pub fn omschrijving(&self) -> &'static str {
        match self {
            Self::Verboden => "niet toepassen is niet toegestaan",
            Self::EigenMotivering => "niet toepassen mag met een eigen motivering",
            Self::GrondslagInRegeling => {
                "niet toepassen mag alleen met een grondslag in de regeling"
            }
        }
    }
}

/// De onderbouwing waarom een maatregel niet wordt toegepast.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Niettoepassing {
    EigenMotivering(Motivering),
    GrondslagInRegeling { regeling: String, artikel: String, motivering: Motivering },
}

impl Niettoepassing {
    pub fn motivering(&self) -> &Motivering {
        match self {
            Self::EigenMotivering(m) => m,
            Self::GrondslagInRegeling { motivering, .. } => motivering,
        }
    }

    pub fn aanduiding(&self) -> String {
        match self {
            Self::EigenMotivering(_) => "eigen motivering".into(),
            Self::GrondslagInRegeling { regeling, artikel, .. } => {
                format!("{regeling}, {artikel}")
            }
        }
    }
}

/// Of de maatregel wordt ingericht, gemotiveerd niet wordt toegepast, of nog
/// geen oordeel heeft gekregen.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Toepassing {
    NogNietBeoordeeld,
    Ingericht,
    NietToegepast(Niettoepassing),
}

impl Toepassing {
    pub fn omschrijving(&self) -> String {
        match self {
            Self::NogNietBeoordeeld => "nog niet beoordeeld".into(),
            Self::Ingericht => "ingericht".into(),
            Self::NietToegepast(n) => format!("gemotiveerd niet toegepast ({})", n.aanduiding()),
        }
    }
}

/// De stand van één maatregel. Wordt berekend, nooit gezet.
///
/// De vier waarden komen uit het datamodel van het plan.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Maatregelstand {
    /// Er is een oordeel nodig van een mens voordat er iets te zeggen valt.
    MenselijkOordeelVereist,
    /// Niet ingericht, of gemotiveerd niet toegepast.
    NietIngericht,
    /// Ingericht, maar er ligt geen geldig uitvoeringsbewijs.
    VastgesteldNietAantoonbaar,
    /// Ingericht én op dit moment met bewijs te onderbouwen.
    Aantoonbaar,
}

impl Maatregelstand {
    pub fn omschrijving(&self) -> &'static str {
        match self {
            Self::MenselijkOordeelVereist => "menselijk oordeel vereist",
            Self::NietIngericht => "niet ingericht",
            Self::VastgesteldNietAantoonbaar => "vastgesteld, niet aantoonbaar",
            Self::Aantoonbaar => "aantoonbaar",
        }
    }
}

/// Wie een maatregel uitvoert: een rol met een bezetting.
///
/// Een naam zonder rol verdwijnt bij het eerste vertrek; een rol zonder
/// bezetting wordt door niemand uitgevoerd. Daarom allebei.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Eigenaar {
    pub rol: String,
    pub persoon: String,
}

/// Een zelf vastgestelde uitvoeringsfrequentie.
///
/// De wet noemt voor de meeste maatregelen geen frequentie. Wie er dan zelf
/// een kiest, moet vastleggen wie dat heeft gedaan en waarom — anders is de
/// termijn later niet te verdedigen en evenmin te herzien.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Frequentie {
    pub maanden: u32,
    pub vastgesteld_door: String,
    pub vastgesteld_op: DateTime<Utc>,
    pub motivering: Motivering,
}

/// Een bewijsstuk dat in de kluis staat, met het venster waarin het geldt.
///
/// `geldig_tot` is met opzet geen `Option`. Bewijs zonder einddatum houdt een
/// dossier voor altijd groen; de tekortkoming die daarmee wordt gemaskeerd,
/// komt pas bij een uitvraag boven water.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Bewijsaanwijzing {
    pub rol: Bewijsrol,
    pub omschrijving: String,
    /// De inhoudshash waaronder het bestand in de kluis staat.
    pub bijlagehash: String,
    pub bestandsnaam: String,
    pub geldig_van: DateTime<Utc>,
    pub geldig_tot: DateTime<Utc>,
    pub bewijskracht: Bewijskracht,
    pub aangewezen_door: String,
    pub aangewezen_op: DateTime<Utc>,
}

impl Bewijsaanwijzing {
    /// Of het venster op dit moment openstaat.
    pub fn geldt_op(&self, nu: DateTime<Utc>) -> bool {
        self.geldig_van <= nu && nu < self.geldig_tot
    }

    /// Over hoeveel dagen het bewijs vervalt. Negatief als het al verlopen is.
    pub fn dagen_tot_verval(&self, nu: DateTime<Utc>) -> i64 {
        (self.geldig_tot - nu).num_days()
    }

    fn controleer(&self, nu: DateTime<Utc>) -> Resultaat<()> {
        if self.omschrijving.trim().is_empty() {
            return Err(DomeinFout::OngeldigeWaarde {
                veld: "zorgplicht.bewijs.omschrijving".into(),
                reden: "zonder omschrijving weet niemand over een jaar nog wat dit stuk \
                        bewijst"
                    .into(),
            });
        }
        if self.bijlagehash.len() != 64 || !self.bijlagehash.chars().all(|c| c.is_ascii_hexdigit())
        {
            return Err(DomeinFout::OngeldigeWaarde {
                veld: "zorgplicht.bewijs.bijlagehash".into(),
                reden: "de aanduiding is geen inhoudshash uit de kluis; bewijs wordt \
                        aangewezen door een bestand aan te leveren, niet door een verwijzing \
                        over te typen"
                    .into(),
            });
        }
        if self.geldig_tot <= self.geldig_van {
            return Err(DomeinFout::OnmogelijkTijdstip {
                veld: "zorgplicht.bewijs.geldig_tot".into(),
                reden: "het bewijs zou verlopen voordat het geldt".into(),
            });
        }
        if self.geldig_tot <= nu {
            return Err(DomeinFout::OnmogelijkTijdstip {
                veld: "zorgplicht.bewijs.geldig_tot".into(),
                reden: "dit stuk is nu al verlopen; verlopen bewijs aanwijzen als \
                        onderbouwing maakt het dossier onbetrouwbaar"
                    .into(),
            });
        }
        Ok(())
    }
}

/// Eén maatregel uit het normenkader, met wat de organisatie ermee heeft
/// gedaan.
///
/// De eerste zes velden komen uit het kennispakket en kennen geen
/// wijzigingsmethode. Wie de norm wil veranderen, verandert het pakket.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Zorgplichtmaatregel {
    pub code: String,
    pub onderdeel: Zorgplichtonderdeel,
    /// Waar de maatregel in het kader staat, bijvoorbeeld `art. 6 Cbb`.
    pub normvindplaats: String,
    pub omschrijving: String,
    pub periodiek: bool,
    pub niettoepassingsvorm: Niettoepassingsvorm,
    pub externe_toetsing_verwacht: bool,

    pub toepassing: Toepassing,
    pub eigenaar: Option<Eigenaar>,
    pub frequentie: Option<Frequentie>,
    pub bewijs: Vec<Bewijsaanwijzing>,
}

impl Zorgplichtmaatregel {
    /// Het uitvoeringsbewijs dat op dit moment geldt, als dat er is.
    ///
    /// Bij meerdere geldige stukken wint het stuk waarvan de uitvoering het
    /// meest recent is, en niet het stuk met het ruimste venster. Anders zou
    /// een oud stuk met een lange looptijd een verse uitdraai blijven
    /// overschaduwen, en zou de vervaldatum die het dossier toont niet de
    /// datum zijn waarop de laatste uitvoering vervalt.
    pub fn geldig_uitvoeringsbewijs(&self, nu: DateTime<Utc>) -> Option<&Bewijsaanwijzing> {
        self.bewijs
            .iter()
            .filter(|b| b.rol == Bewijsrol::Uitvoering && b.geldt_op(nu))
            .max_by_key(|b| (b.geldig_van, b.aangewezen_op))
    }

    /// Het bewijsstuk dat als eerste vervalt van de stukken die nu gelden.
    pub fn eerst_vervallend(&self, nu: DateTime<Utc>) -> Option<&Bewijsaanwijzing> {
        self.bewijs.iter().filter(|b| b.geldt_op(nu)).min_by_key(|b| b.geldig_tot)
    }

    /// De stand van deze maatregel. Berekend, in volgorde van zwaarte.
    pub fn stand(&self, nu: DateTime<Utc>) -> Maatregelstand {
        if self.eigenaar.is_none() {
            return Maatregelstand::MenselijkOordeelVereist;
        }
        if self.periodiek && self.frequentie.is_none() {
            return Maatregelstand::MenselijkOordeelVereist;
        }
        match &self.toepassing {
            Toepassing::NogNietBeoordeeld => Maatregelstand::MenselijkOordeelVereist,
            Toepassing::NietToegepast(_) => Maatregelstand::NietIngericht,
            Toepassing::Ingericht => {
                if self.geldig_uitvoeringsbewijs(nu).is_some() {
                    Maatregelstand::Aantoonbaar
                } else {
                    Maatregelstand::VastgesteldNietAantoonbaar
                }
            }
        }
    }

    /// Hoeveel maanden geleden de uitvoering voor het laatst is onderbouwd.
    ///
    /// Gemeten aan het begin van het geldigheidsvenster: dat is het moment
    /// waarop de uitvoering plaatsvond, niet het moment waarop het stuk
    /// verloopt.
    ///
    /// Stukken waarvan het venster nog moet ingaan tellen niet mee. Een
    /// uitvoering die nog moet plaatsvinden, heeft niet plaatsgevonden; zou
    /// zo een stuk wel meetellen, dan zou het aanwijzen van bewijs met een
    /// datum in de toekomst regel ZRP-08 het zwijgen opleggen.
    pub fn maanden_sinds_uitvoering(&self, nu: DateTime<Utc>) -> Option<i64> {
        self.bewijs
            .iter()
            .filter(|b| b.rol == Bewijsrol::Uitvoering)
            .map(|b| b.geldig_van)
            .filter(|v| *v <= nu)
            .max()
            .map(|v| (nu - v).num_days() / 30)
    }
}

/// Eén maatregel zoals het kennispakket hem aanlevert.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Kadermaatregel {
    pub code: String,
    pub onderdeel: Zorgplichtonderdeel,
    pub normvindplaats: String,
    pub omschrijving: String,
    /// Of het kader deze maatregel periodiek uitgevoerd wil zien.
    ///
    /// Zonder `serde(default)`, met opzet: een ontbrekend veld zou terugvallen
    /// op `false`, en dat is de kant die controle wegneemt. Een kader met een
    /// typefout hoort te weigeren en niet stilzwijgend de frequentie-eis te
    /// laten vervallen.
    pub periodiek: bool,
    pub niettoepassingsvorm: Niettoepassingsvorm,
    /// Of het kader bij deze maatregel toetsing door een ander verwacht.
    ///
    /// Zonder `serde(default)`, om dezelfde reden als bij `periodiek`.
    pub externe_toetsing_verwacht: bool,
}

/// Een normenkader zoals het kennispakket het aanlevert.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Kaderdefinitie {
    pub kenmerk: String,
    pub variant: Raamwerkvariant,
    pub versie: String,
    pub bron: String,
    /// Wanneer een mens dit kader tegen de bron heeft gehouden.
    #[serde(default)]
    pub geverifieerd_op: Option<NaiveDate>,
    #[serde(default)]
    pub toelichting: Option<String>,
    pub maatregelen: Vec<Kadermaatregel>,
}

impl Kaderdefinitie {
    /// De onderdelen a tot en met j waarvoor dit kader geen maatregel kent.
    pub fn onderdelen_zonder_maatregel(&self) -> Vec<Zorgplichtonderdeel> {
        Zorgplichtonderdeel::alle()
            .into_iter()
            .filter(|o| !self.maatregelen.iter().any(|m| m.onderdeel == *o))
            .collect()
    }
}

/// De risicobeoordeling waarop de controlset steunt.
///
/// Nog geen zelfstandig dossier: hier alleen de verwijzing met het bewijs
/// erbij, zodat het dossier niet kan doen alsof er een beoordeling ligt.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Risicobeoordeling {
    pub omschrijving: String,
    pub methode: String,
    pub scope: String,
    pub uitgevoerd_door: String,
    pub uitgevoerd_op: DateTime<Utc>,
    pub geldig_tot: DateTime<Utc>,
    pub bewijs: Bewijsaanwijzing,
}

impl Risicobeoordeling {
    pub fn is_verlopen(&self, nu: DateTime<Utc>) -> bool {
        self.geldig_tot <= nu
    }
}

/// Het besluit waarmee het bestuur het maatregelenpakket heeft vastgesteld.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Bestuursvaststelling {
    pub datum: DateTime<Utc>,
    pub besluittekst: String,
    /// De kaderversie waarover dit besluit gaat.
    pub goedgekeurde_kaderversie: String,
    pub aanwezigen: Vec<String>,
    pub bewijs: Bewijsaanwijzing,
}

impl Bestuursvaststelling {
    pub fn maanden_oud(&self, nu: DateTime<Utc>) -> i64 {
        (nu - self.datum).num_days() / 30
    }
}

/// Het zorgplichtdossier van één entiteit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Zorgplichtdossier {
    pub id: Id,
    pub kenmerk: String,
    /// De naam van de entiteit waarvoor de zorgplicht geldt.
    pub naam: String,
    pub status: Status,
    pub compartiment: Compartiment,
    pub herkomst: Herkomst,

    pub variant: Raamwerkvariant,
    pub kaderkenmerk: String,
    pub kaderversie: String,
    pub kaderbron: String,
    pub kader_geverifieerd_op: Option<NaiveDate>,
    /// Alleen bij variant C: de regeling die dit kader voorschrijft.
    pub regelingsverwijzing: Option<String>,

    /// De naam van de aangemelde functionaris, om te kunnen controleren dat
    /// hij geen eigenaar is van een maatregel waarop hij toezicht houdt.
    pub aangemelde_functionaris: String,

    pub maatregelen: Vec<Zorgplichtmaatregel>,
    pub risicobeoordeling: Option<Risicobeoordeling>,
    pub bestuursvaststelling: Option<Bestuursvaststelling>,
}

impl Zorgplichtdossier {
    /// Leidt het dossier af uit een normenkader.
    ///
    /// Heet niet `nieuw` omdat een controlset niet wordt bedacht maar
    /// afgeleid, en geeft anders dan de meeste constructors een `Resultaat`:
    /// aan een kader valt objectief iets mis te gaan.
    pub fn leid_af(
        kenmerk: impl Into<String>,
        naam: impl Into<String>,
        aangemelde_functionaris: impl Into<String>,
        kader: &Kaderdefinitie,
        regelingsverwijzing: Option<String>,
        door: impl Into<String>,
        op: DateTime<Utc>,
    ) -> Resultaat<Self> {
        let ontbrekend = kader.onderdelen_zonder_maatregel();
        if !ontbrekend.is_empty() {
            let letters: Vec<_> = ontbrekend.iter().map(|o| o.letter()).collect();
            return Err(DomeinFout::OngeldigeWaarde {
                veld: "zorgplicht.kader".into(),
                reden: format!(
                    "het kader kent geen maatregel voor onderdeel {}; een controlset die niet \
                     alle tien onderdelen van artikel 21 lid 3 dekt, is geen controlset maar \
                     een selectie",
                    letters.join(", ")
                ),
            });
        }
        for (i, m) in kader.maatregelen.iter().enumerate() {
            if kader.maatregelen[..i].iter().any(|e| e.code == m.code) {
                return Err(DomeinFout::OngeldigeWaarde {
                    veld: "zorgplicht.kader".into(),
                    reden: format!(
                        "maatregelcode '{}' komt twee keer voor; twee maatregelen met dezelfde \
                         code zijn in een dossier niet uit elkaar te houden",
                        m.code
                    ),
                });
            }
        }
        if kader.versie.trim().is_empty() {
            return Err(DomeinFout::OngeldigeWaarde {
                veld: "zorgplicht.kaderversie".into(),
                reden: "een kader zonder versieaanduiding is later niet te herleiden; een \
                        bewijsstuk dat aan de verkeerde versie hangt, is bij een uitvraag een \
                        aangrijpingspunt"
                    .into(),
            });
        }

        let verwijzing = regelingsverwijzing.filter(|r| !r.trim().is_empty());
        match (kader.variant, &verwijzing) {
            (Raamwerkvariant::C, None) => {
                return Err(DomeinFout::OntbrekendeVerwijzing {
                    veld: "zorgplicht.regelingsverwijzing".into(),
                    naar: "ministeriële regeling die dit kader voorschrijft".into(),
                })
            }
            (Raamwerkvariant::A | Raamwerkvariant::B, Some(_)) => {
                return Err(DomeinFout::OngeldigeWaarde {
                    veld: "zorgplicht.regelingsverwijzing".into(),
                    reden: "alleen bij variant C is het kader door een regeling voorgeschreven; \
                            deze verwijzing zou een dwingendheid suggereren die er niet is"
                        .into(),
                })
            }
            _ => {}
        }

        let functionaris = aangemelde_functionaris.into();
        if functionaris.trim().is_empty() {
            return Err(DomeinFout::OngeldigeWaarde {
                veld: "zorgplicht.aangemelde_functionaris".into(),
                reden: "zonder de naam van de aangemelde functionaris kan de tool niet \
                        controleren dat hij geen eigenaar is van een maatregel waarop hij \
                        toezicht houdt (art. 38 lid 6 AVG)"
                    .into(),
            });
        }

        let door = door.into();
        Ok(Self {
            id: Id::nieuw(),
            kenmerk: kenmerk.into(),
            naam: naam.into(),
            status: Status::Concept,
            compartiment: Compartiment::algemeen(),
            herkomst: Herkomst::nieuw(door, op),
            variant: kader.variant,
            kaderkenmerk: kader.kenmerk.clone(),
            kaderversie: kader.versie.clone(),
            kaderbron: kader.bron.clone(),
            kader_geverifieerd_op: kader.geverifieerd_op,
            regelingsverwijzing: verwijzing,
            aangemelde_functionaris: functionaris.trim().to_string(),
            maatregelen: kader
                .maatregelen
                .iter()
                .map(|m| Zorgplichtmaatregel {
                    code: m.code.clone(),
                    onderdeel: m.onderdeel,
                    normvindplaats: m.normvindplaats.clone(),
                    omschrijving: m.omschrijving.clone(),
                    periodiek: m.periodiek,
                    niettoepassingsvorm: m.niettoepassingsvorm,
                    externe_toetsing_verwacht: m.externe_toetsing_verwacht,
                    toepassing: Toepassing::NogNietBeoordeeld,
                    eigenaar: None,
                    frequentie: None,
                    bewijs: Vec::new(),
                })
                .collect(),
            risicobeoordeling: None,
            bestuursvaststelling: None,
        })
    }

    fn maatregel_mut(&mut self, code: &str) -> Resultaat<&mut Zorgplichtmaatregel> {
        self.maatregelen.iter_mut().find(|m| m.code == code).ok_or_else(|| {
            DomeinFout::OntbrekendeVerwijzing {
                veld: "zorgplicht.maatregel".into(),
                naar: format!("maatregel met code '{code}' in dit kader"),
            }
        })
    }

    pub fn maatregel(&self, code: &str) -> Option<&Zorgplichtmaatregel> {
        self.maatregelen.iter().find(|m| m.code == code)
    }

    /// Of deze naam die van de aangemelde functionaris is.
    ///
    /// Vergelijkt op de genormaliseerde naam. Dat is zwak — een tweede
    /// schrijfwijze ontsnapt eraan — en het blijft zwak tot rollen en
    /// bezettingen een eigen record hebben. Dat staat hier zodat niemand denkt
    /// dat deze controle sterker is dan zij is.
    pub fn is_de_functionaris(&self, naam: &str) -> bool {
        naam.trim().eq_ignore_ascii_case(self.aangemelde_functionaris.trim())
    }

    /// Wijst een eigenaar aan voor één maatregel.
    pub fn wijs_eigenaar_toe(
        &mut self,
        code: &str,
        rol: impl Into<String>,
        persoon: impl Into<String>,
        op: DateTime<Utc>,
    ) -> Resultaat<()> {
        let rol = rol.into();
        let persoon = persoon.into();
        if rol.trim().is_empty() || persoon.trim().is_empty() {
            return Err(DomeinFout::OngeldigeWaarde {
                veld: "zorgplicht.eigenaar".into(),
                reden: "noem zowel de rol als de bezetting: een maatregel zonder rol wordt door \
                        niemand uitgevoerd, en een naam zonder rol verdwijnt bij het eerste \
                        vertrek"
                    .into(),
            });
        }
        if self.is_de_functionaris(&persoon) {
            return Err(DomeinFout::OngeldigeWaarde {
                veld: "zorgplicht.eigenaar.persoon".into(),
                reden: "de aangemelde functionaris kan geen eigenaar zijn van een maatregel \
                        waarop hij toezicht houdt (art. 38 lid 6 AVG)"
                    .into(),
            });
        }
        let m = self.maatregel_mut(code)?;
        m.eigenaar = Some(Eigenaar { rol: rol.trim().into(), persoon: persoon.trim().into() });
        self.herkomst.wijzig(format!("eigenaar van {code} aangewezen"), op);
        Ok(())
    }

    /// Legt vast dat de maatregel is ingericht.
    ///
    /// Zet nadrukkelijk geen stand: dat inrichten iets aantoonbaar maakt, is
    /// precies de illusie die dit dossier moet uitsluiten.
    pub fn richt_in(&mut self, code: &str, op: DateTime<Utc>) -> Resultaat<()> {
        let m = self.maatregel_mut(code)?;
        m.toepassing = Toepassing::Ingericht;
        self.herkomst.wijzig(format!("{code} ingericht"), op);
        Ok(())
    }

    /// Legt vast dat de maatregel gemotiveerd niet wordt toegepast.
    pub fn pas_niet_toe(
        &mut self,
        code: &str,
        niettoepassing: Niettoepassing,
        op: DateTime<Utc>,
    ) -> Resultaat<()> {
        let vorm = self.maatregel(code).map(|m| m.niettoepassingsvorm).ok_or_else(|| {
            DomeinFout::OntbrekendeVerwijzing {
                veld: "zorgplicht.maatregel".into(),
                naar: format!("maatregel met code '{code}' in dit kader"),
            }
        })?;
        let eis = self.maatregel(code).map(|m| m.omschrijving.clone()).unwrap_or_default();

        match (vorm, &niettoepassing) {
            (Niettoepassingsvorm::Verboden, _) => {
                return Err(DomeinFout::OngeldigeWaarde {
                    veld: "zorgplicht.toepassing".into(),
                    reden: "deze eis is onvoorwaardelijk geformuleerd; niet toepassen is geen \
                            keuze die met een motivering te maken is. Haalt u de eis niet, dan \
                            blijft de maatregel niet ingericht en is dat wat het dossier toont"
                        .into(),
                })
            }
            (Niettoepassingsvorm::GrondslagInRegeling, Niettoepassing::EigenMotivering(_)) => {
                return Err(DomeinFout::OngeldigeWaarde {
                    veld: "zorgplicht.toepassing".into(),
                    reden: "bij een voorgeschreven kader is een eigen motivering niet genoeg; \
                            afwijken vereist een grondslag in de regeling zelf. Noem de \
                            regeling en het artikel"
                        .into(),
                })
            }
            (Niettoepassingsvorm::EigenMotivering, Niettoepassing::GrondslagInRegeling { .. }) => {
                return Err(DomeinFout::OngeldigeWaarde {
                    veld: "zorgplicht.toepassing".into(),
                    reden: "dit kader is niet bij regeling voorgeschreven; een verwijzing naar \
                            een regeling zou een dwingendheid suggereren die er niet is. \
                            Motiveer zelf waarom deze maatregel niet wordt toegepast"
                        .into(),
                })
            }
            _ => {}
        }

        if let Niettoepassing::GrondslagInRegeling { regeling, artikel, .. } = &niettoepassing {
            if regeling.trim().is_empty() || artikel.trim().is_empty() {
                return Err(DomeinFout::OngeldigeWaarde {
                    veld: "zorgplicht.toepassing.grondslag".into(),
                    reden: "een grondslag zonder regeling en artikelaanduiding is niet na te \
                            lopen"
                        .into(),
                });
            }
        }

        if genormaliseerd(&niettoepassing.motivering().tekst) == genormaliseerd(&eis) {
            return Err(DomeinFout::OngeldigeWaarde {
                veld: "zorgplicht.toepassing.motivering".into(),
                reden: "de motivering herhaalt de tekst van de eis; leg uit waarom deze eis in \
                        uw situatie niet wordt toegepast, niet wat de eis inhoudt"
                    .into(),
            });
        }

        let m = self.maatregel_mut(code)?;
        m.toepassing = Toepassing::NietToegepast(niettoepassing);
        self.herkomst.wijzig(format!("{code} gemotiveerd niet toegepast"), op);
        Ok(())
    }

    /// Legt de zelf vastgestelde uitvoeringsfrequentie vast.
    ///
    /// Er staat met opzet geen bovengrens in deze code. Hoeveel maanden te
    /// lang is, is een norm en hoort in het kennispakket; regel ZRP-07 meet
    /// ertegen.
    pub fn stel_frequentie_vast(
        &mut self,
        code: &str,
        maanden: u32,
        vastgesteld_door: impl Into<String>,
        motivering: Motivering,
        op: DateTime<Utc>,
    ) -> Resultaat<()> {
        if maanden == 0 {
            return Err(DomeinFout::OngeldigeWaarde {
                veld: "zorgplicht.frequentie".into(),
                reden: "een frequentie van nul maanden is geen frequentie".into(),
            });
        }
        let periodiek = self.maatregel(code).map(|m| m.periodiek).ok_or_else(|| {
            DomeinFout::OntbrekendeVerwijzing {
                veld: "zorgplicht.maatregel".into(),
                naar: format!("maatregel met code '{code}' in dit kader"),
            }
        })?;
        if !periodiek {
            return Err(DomeinFout::OngeldigeWaarde {
                veld: "zorgplicht.frequentie".into(),
                reden: "deze maatregel kent volgens het kader geen periodieke uitvoering; er \
                        een zelfbedachte klok op zetten maakt hem niet aantoonbaarder"
                    .into(),
            });
        }
        let m = self.maatregel_mut(code)?;
        m.frequentie = Some(Frequentie {
            maanden,
            vastgesteld_door: vastgesteld_door.into(),
            vastgesteld_op: op,
            motivering,
        });
        self.herkomst.wijzig(format!("frequentie van {code} vastgesteld"), op);
        Ok(())
    }

    /// Keurt een voorgenomen bewijsaanwijzing zonder iets vast te leggen.
    ///
    /// Bestaat omdat de bedieningsschil het bestand versleuteld in de kluis
    /// zet vóórdat het dossier wordt bewaard, en die handeling in de keten
    /// hangt en dus niet terug te draaien is. Wie eerst keurt, laat bij een
    /// typefout in de maatregelcode geen bijlage achter die aan niets hangt.
    pub fn keur_bewijs(
        &self,
        code: &str,
        rol: Bewijsrol,
        omschrijving: &str,
        geldig_van: DateTime<Utc>,
        geldig_tot: DateTime<Utc>,
        nu: DateTime<Utc>,
    ) -> Resultaat<()> {
        if self.maatregel(code).is_none() {
            return Err(DomeinFout::OntbrekendeVerwijzing {
                veld: "zorgplicht.maatregel".into(),
                naar: format!("maatregel met code '{code}' in dit kader"),
            });
        }
        Bewijsaanwijzing {
            rol,
            omschrijving: omschrijving.to_string(),
            // Een geldige plaatsvervangende hash: de echte komt pas uit de
            // kluis, en die stap volgt op deze keuring.
            bijlagehash: "0".repeat(64),
            bestandsnaam: String::new(),
            geldig_van,
            geldig_tot,
            bewijskracht: Bewijskracht::Zelfgerapporteerd,
            aangewezen_door: String::new(),
            aangewezen_op: nu,
        }
        .controleer(nu)
    }

    /// Wijst een bewijsstuk aan bij een maatregel.
    pub fn wijs_bewijs_aan(
        &mut self,
        code: &str,
        aanwijzing: Bewijsaanwijzing,
        op: DateTime<Utc>,
    ) -> Resultaat<()> {
        aanwijzing.controleer(op)?;
        let m = self.maatregel_mut(code)?;
        m.bewijs.retain(|b| b.bijlagehash != aanwijzing.bijlagehash || b.rol != aanwijzing.rol);
        m.bewijs.push(aanwijzing);
        self.herkomst.wijzig(format!("bewijs bij {code} aangewezen"), op);
        Ok(())
    }

    /// Legt de risicobeoordeling vast waarop de controlset steunt.
    pub fn leg_risicobeoordeling_vast(
        &mut self,
        beoordeling: Risicobeoordeling,
        op: DateTime<Utc>,
    ) -> Resultaat<()> {
        if beoordeling.methode.trim().is_empty() || beoordeling.scope.trim().is_empty() {
            return Err(DomeinFout::OngeldigeWaarde {
                veld: "zorgplicht.risicobeoordeling".into(),
                reden: "noem de methode en de reikwijdte; zonder die twee is een beoordeling \
                        niet te herhalen en niet te toetsen"
                    .into(),
            });
        }
        if beoordeling.uitgevoerd_op > op {
            return Err(DomeinFout::OnmogelijkTijdstip {
                veld: "zorgplicht.risicobeoordeling.uitgevoerd_op".into(),
                reden: "de beoordeling zou in de toekomst zijn uitgevoerd".into(),
            });
        }
        if beoordeling.geldig_tot <= beoordeling.uitgevoerd_op {
            return Err(DomeinFout::OnmogelijkTijdstip {
                veld: "zorgplicht.risicobeoordeling.geldig_tot".into(),
                reden: "de beoordeling zou verlopen voordat zij is uitgevoerd".into(),
            });
        }
        beoordeling.bewijs.controleer(op)?;
        self.risicobeoordeling = Some(beoordeling);
        self.herkomst.wijzig("risicobeoordeling vastgelegd", op);
        Ok(())
    }

    /// Legt het bestuursbesluit vast waarmee het maatregelenpakket is
    /// vastgesteld.
    pub fn leg_bestuursvaststelling_vast(
        &mut self,
        vaststelling: Bestuursvaststelling,
        op: DateTime<Utc>,
    ) -> Resultaat<()> {
        if vaststelling.datum > op {
            return Err(DomeinFout::OnmogelijkTijdstip {
                veld: "zorgplicht.bestuursvaststelling.datum".into(),
                reden: "het besluit zou in de toekomst zijn genomen".into(),
            });
        }
        if vaststelling.besluittekst.trim().is_empty() {
            return Err(DomeinFout::OngeldigeWaarde {
                veld: "zorgplicht.bestuursvaststelling.besluittekst".into(),
                reden: "leg vast wát het bestuur heeft besloten; een datum zonder besluittekst \
                        toont niet aan dat het pakket is goedgekeurd"
                    .into(),
            });
        }
        if vaststelling.aanwezigen.iter().all(|a| a.trim().is_empty()) {
            return Err(DomeinFout::OngeldigeWaarde {
                veld: "zorgplicht.bestuursvaststelling.aanwezigen".into(),
                reden: "een besluit zonder aanwezigen is geen vergaderbesluit maar een \
                        aantekening"
                    .into(),
            });
        }
        if vaststelling.goedgekeurde_kaderversie != self.kaderversie {
            return Err(DomeinFout::OngeldigeWaarde {
                veld: "zorgplicht.bestuursvaststelling.goedgekeurde_kaderversie".into(),
                reden: format!(
                    "dit besluit gaat over kaderversie {} terwijl het dossier op versie {} \
                     staat; goedkeuring geldt per versie van het maatregelenpakket",
                    vaststelling.goedgekeurde_kaderversie, self.kaderversie
                ),
            });
        }
        vaststelling.bewijs.controleer(op)?;
        self.bestuursvaststelling = Some(vaststelling);
        self.herkomst.wijzig("bestuursvaststelling vastgelegd", op);
        Ok(())
    }

    /// Legt een nieuwe aangemelde functionaris vast.
    ///
    /// Weigert niet wanneer daardoor een zittende eigenaar ineens de
    /// functionaris blijkt: die wissel kan de gebruiker niet ongedaan maken en
    /// blokkeren zou hem alleen beletten de werkelijkheid vast te leggen. Het
    /// conflict wordt zichtbaar via regel ZRP-02.
    pub fn wijzig_aangemelde_functionaris(
        &mut self,
        naam: impl Into<String>,
        op: DateTime<Utc>,
    ) -> Resultaat<()> {
        let naam = naam.into();
        if naam.trim().is_empty() {
            return Err(DomeinFout::OngeldigeWaarde {
                veld: "zorgplicht.aangemelde_functionaris".into(),
                reden: "de naam mag niet leeg zijn".into(),
            });
        }
        self.aangemelde_functionaris = naam.trim().to_string();
        self.herkomst.wijzig("aangemelde functionaris gewijzigd", op);
        Ok(())
    }

    /// De maatregelen waarvan de eigenaar de aangemelde functionaris is.
    pub fn eigenaarsconflicten(&self) -> Vec<&Zorgplichtmaatregel> {
        self.maatregelen
            .iter()
            .filter(|m| m.eigenaar.as_ref().is_some_and(|e| self.is_de_functionaris(&e.persoon)))
            .collect()
    }

    /// Hoeveel maatregelen er per stand zijn.
    pub fn standen(&self, nu: DateTime<Utc>) -> Vec<(Maatregelstand, usize)> {
        let mut uit = Vec::new();
        for stand in [
            Maatregelstand::Aantoonbaar,
            Maatregelstand::VastgesteldNietAantoonbaar,
            Maatregelstand::NietIngericht,
            Maatregelstand::MenselijkOordeelVereist,
        ] {
            let aantal = self.maatregelen.iter().filter(|m| m.stand(nu) == stand).count();
            if aantal > 0 {
                uit.push((stand, aantal));
            }
        }
        uit
    }

    /// Het aandeel afwijkbare maatregelen dat gemotiveerd niet wordt
    /// toegepast, in procenten.
    ///
    /// De noemer is niet de hele set maar het aantal maatregelen waarvan het
    /// kader zegt dat afwijken is toegestaan. Dat is de enige noemer die iets
    /// zegt: bij een kader waarin dertien van de vijftien maatregelen
    /// onvoorwaardelijk zijn, kan het aandeel over de hele set nooit boven de
    /// dertien procent komen, en zou elke drempel daarboven een regel
    /// opleveren die nooit kan aanslaan.
    ///
    /// `None` wanneer er geen enkele afwijkbare maatregel is: dan valt er
    /// niets te meten, en nul melden zou suggereren dat er niet wordt
    /// afgeweken terwijl er niet afgeweken kán worden.
    pub fn aandeel_niet_toegepast(&self) -> Option<u32> {
        let afwijkbaar = self
            .maatregelen
            .iter()
            .filter(|m| m.niettoepassingsvorm != Niettoepassingsvorm::Verboden)
            .count();
        if afwijkbaar == 0 {
            return None;
        }
        let aantal = self
            .maatregelen
            .iter()
            .filter(|m| matches!(m.toepassing, Toepassing::NietToegepast(_)))
            .count();
        Some(((aantal * 100) / afwijkbaar) as u32)
    }

    /// Hoeveel maatregelen het kader afwijkbaar noemt.
    pub fn aantal_afwijkbaar(&self) -> usize {
        self.maatregelen
            .iter()
            .filter(|m| m.niettoepassingsvorm != Niettoepassingsvorm::Verboden)
            .count()
    }

    /// Wat er op een peildatum niet meer met geldig bewijs is te onderbouwen.
    ///
    /// Geen prognose met een score, maar een lijst: code, omschrijving,
    /// eigenaar en de datum waarop het bewijs vervalt.
    ///
    /// Meet aan het bewijs en niet aan de stand. Een maatregel die op
    /// menselijk oordeel wacht omdat de eigenaar of de frequentie ontbreekt,
    /// kan wel degelijk geldig uitvoeringsbewijs dragen; die uit de lijst
    /// weglaten zou juist de dossiers waar het meeste openstaat het rustigst
    /// laten ogen.
    pub fn vervalt_voor(&self, peildatum: DateTime<Utc>, nu: DateTime<Utc>) -> Vec<Vervalregel> {
        self.maatregelen
            .iter()
            .filter_map(|m| {
                let bewijs = m.geldig_uitvoeringsbewijs(nu)?;
                if bewijs.geldig_tot > peildatum {
                    return None;
                }
                Some(Vervalregel {
                    code: m.code.clone(),
                    onderdeel: m.onderdeel,
                    omschrijving: m.omschrijving.clone(),
                    eigenaar: m.eigenaar.clone(),
                    vervalt_op: bewijs.geldig_tot,
                })
            })
            .collect()
    }

    /// Stelt het dossier vast.
    pub fn stel_vast(&mut self, door: impl Into<String>, op: DateTime<Utc>) -> Resultaat<()> {
        let rapport = self.volledigheid();
        if !rapport.mag_vaststellen() {
            return Err(DomeinFout::NietVolledig {
                soort: "zorgplichtdossier".into(),
                ontbreekt: rapport.blokkades().iter().map(|o| o.veld.clone()).collect(),
            });
        }
        self.status = Status::Vastgesteld;
        self.herkomst.stel_vast(door, op);
        Ok(())
    }
}

/// Eén regel uit de vervallijst.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Vervalregel {
    pub code: String,
    pub onderdeel: Zorgplichtonderdeel,
    pub omschrijving: String,
    pub eigenaar: Option<Eigenaar>,
    pub vervalt_op: DateTime<Utc>,
}

/// Normaliseert tekst voor de vergelijking motivering tegen eis.
fn genormaliseerd(tekst: &str) -> String {
    tekst
        .to_lowercase()
        .chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace())
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

impl Volledig for Zorgplichtdossier {
    fn soortnaam(&self) -> &'static str {
        "zorgplichtdossier"
    }

    fn aantal_verplichte_onderdelen(&self) -> usize {
        // Per maatregel: een eigenaar en een oordeel over de toepassing.
        let mut totaal = self.maatregelen.len() * 2;
        // Voor periodieke maatregelen daarbovenop een frequentie, tenzij
        // juist is besloten dat de maatregel niet wordt uitgevoerd: hoe vaak
        // iets gebeurt dat niet gebeurt, is geen zinnige vraag.
        totaal += self
            .maatregelen
            .iter()
            .filter(|m| m.periodiek && !matches!(m.toepassing, Toepassing::NietToegepast(_)))
            .count();
        // Voor ingerichte maatregelen daarbovenop geldig uitvoeringsbewijs.
        totaal += self
            .maatregelen
            .iter()
            .filter(|m| matches!(m.toepassing, Toepassing::Ingericht))
            .count();
        // Kaderverificatie, risicobeoordeling en bestuursvaststelling.
        totaal + 3
    }

    fn ontbrekende_onderdelen(&self) -> Vec<Ontbrekend> {
        let mut uit = Vec::new();

        // Signalerend en niet blokkerend. De verificatie van het kader is werk
        // van een jurist buiten deze toepassing, en het meegeleverde pakket
        // draagt die stempel uitdrukkelijk niet. Zou dit vaststellen
        // tegenhouden, dan levert het product een werkproces dat met de eigen
        // inhoud nooit is af te maken — en dat is geen bewaking maar een dood
        // spoor. Het gat blijft wel in elke uitvoer en in elk dossier staan.
        if self.kader_geverifieerd_op.is_none() {
            uit.push(Ontbrekend::signalerend(
                "zorgplicht.kader",
                format!(
                    "het kader '{}' is niet tegen de bron geverifieerd; de indeling van de \
                     maatregelen over de tien onderdelen is een vertrekpunt en geen \
                     vastgestelde controlset",
                    self.kaderkenmerk
                ),
                "voorbehoud bij het kennispakket; geen wettelijke bepaling",
            ));
        }

        for m in &self.maatregelen {
            if m.eigenaar.is_none() {
                uit.push(Ontbrekend::blokkerend(
                    format!("zorgplicht.maatregel.{}.eigenaar", m.code),
                    format!("wijs een rol met bezetting aan voor: {}", m.omschrijving),
                    "art. 6 lid 4 Cyberbeveiligingsbesluit",
                ));
            }
            if matches!(m.toepassing, Toepassing::NogNietBeoordeeld) {
                uit.push(Ontbrekend::blokkerend(
                    format!("zorgplicht.maatregel.{}.toepassing", m.code),
                    format!("beoordeel of deze maatregel wordt ingericht: {}", m.omschrijving),
                    m.onderdeel.grondslag(),
                ));
            }
            if m.periodiek
                && m.frequentie.is_none()
                && !matches!(m.toepassing, Toepassing::NietToegepast(_))
            {
                uit.push(Ontbrekend::blokkerend(
                    format!("zorgplicht.maatregel.{}.frequentie", m.code),
                    "stel zelf vast hoe vaak deze maatregel wordt uitgevoerd, met een \
                     motivering waarom die termijn passend is"
                        .to_string(),
                    "zelf vastgestelde termijn; de wet noemt hier geen frequentie",
                ));
            }
            // Op de rol en niet op "er ligt iets": een vastgesteld beleidsstuk
            // bewijst per definitie geen uitvoering, en zou anders het gat
            // vullen dat het juist zichtbaar moet houden. Het venster telt hier
            // niet mee — het volledigheidsrapport draagt geen peilmoment — dus
            // dat verlopen bewijs de maatregel laat terugvallen, blijkt uit de
            // stand en uit regel ZRP-04, niet uit deze teller.
            if matches!(m.toepassing, Toepassing::Ingericht)
                && !m.bewijs.iter().any(|b| b.rol == Bewijsrol::Uitvoering)
            {
                uit.push(Ontbrekend::signalerend(
                    format!("zorgplicht.maatregel.{}.bewijs", m.code),
                    "deze maatregel is ingericht maar er ligt geen bewijs van de uitvoering; \
                     tot dat er is, geldt hij als vastgesteld en niet als aantoonbaar"
                        .to_string(),
                    "art. 6 lid 4 Cyberbeveiligingsbesluit",
                ));
            }
        }

        if self.risicobeoordeling.is_none() {
            uit.push(Ontbrekend::blokkerend(
                "zorgplicht.risicobeoordeling",
                "leg vast welke risicobeoordeling aan deze controlset ten grondslag ligt, met \
                 methode, reikwijdte, uitvoerder en bewijs",
                "art. 21 lid 1 en 2 Cyberbeveiligingswet",
            ));
        }
        if self.bestuursvaststelling.is_none() {
            uit.push(Ontbrekend::blokkerend(
                "zorgplicht.bestuursvaststelling",
                "laat het bestuur deze versie van het maatregelenpakket vaststellen",
                "art. 24 lid 1 Cyberbeveiligingswet",
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

    fn kadermaatregel(onderdeel: Zorgplichtonderdeel) -> Kadermaatregel {
        Kadermaatregel {
            code: format!("CBB-{}", onderdeel.letter()),
            onderdeel,
            normvindplaats: "art. 6 Cbb".into(),
            omschrijving: format!("maatregelen voor {}", onderdeel.omschrijving()),
            periodiek: false,
            niettoepassingsvorm: Niettoepassingsvorm::EigenMotivering,
            externe_toetsing_verwacht: false,
        }
    }

    fn kader() -> Kaderdefinitie {
        Kaderdefinitie {
            kenmerk: "CBB-ZORGPLICHT-A".into(),
            variant: Raamwerkvariant::A,
            versie: "2026-08-01".into(),
            bron: "Cyberbeveiligingsbesluit art. 6 tot en met 18".into(),
            geverifieerd_op: NaiveDate::from_ymd_opt(2026, 8, 1),
            toelichting: None,
            maatregelen: Zorgplichtonderdeel::alle().into_iter().map(kadermaatregel).collect(),
        }
    }

    fn dossier() -> Zorgplichtdossier {
        Zorgplichtdossier::leid_af(
            "ZRP-2026",
            "Gemeente Voorbeeld",
            "A. de Vries",
            &kader(),
            None,
            "u1",
            nu(),
        )
        .unwrap()
    }

    fn bewijs(rol: Bewijsrol, van: DateTime<Utc>, tot: DateTime<Utc>) -> Bewijsaanwijzing {
        Bewijsaanwijzing {
            rol,
            omschrijving: "uitdraai uit het beheersysteem".into(),
            bijlagehash: "a".repeat(64),
            bestandsnaam: "uitdraai.pdf".into(),
            geldig_van: van,
            geldig_tot: tot,
            bewijskracht: Bewijskracht::Zelfgerapporteerd,
            aangewezen_door: "u1".into(),
            aangewezen_op: nu(),
        }
    }

    /// Een controlset die niet alle tien onderdelen dekt, is een selectie.
    #[test]
    fn een_kader_met_een_gat_wordt_geweigerd() {
        let mut k = kader();
        k.maatregelen.retain(|m| m.onderdeel != Zorgplichtonderdeel::Cryptografie);
        let fout = Zorgplichtdossier::leid_af(
            "ZRP-2026",
            "Gemeente Voorbeeld",
            "A. de Vries",
            &k,
            None,
            "u1",
            nu(),
        )
        .unwrap_err();
        assert!(fout.to_string().contains("onderdeel h"), "kreeg: {fout}");
        assert!(fout.to_string().contains("geen controlset maar een selectie"));
    }

    #[test]
    fn een_kader_met_een_dubbele_code_wordt_geweigerd() {
        let mut k = kader();
        k.maatregelen.push(kadermaatregel(Zorgplichtonderdeel::Beleid));
        let fout = Zorgplichtdossier::leid_af(
            "ZRP-2026",
            "Gemeente Voorbeeld",
            "A. de Vries",
            &k,
            None,
            "u1",
            nu(),
        )
        .unwrap_err();
        assert!(fout.to_string().contains("twee keer"), "kreeg: {fout}");
    }

    /// De letters en hun grondslag staan op één plaats en verwijzen naar het
    /// derde lid; NIS2 nummert anders en die vergissing mag niet insluipen.
    #[test]
    fn elk_onderdeel_draagt_zijn_letter_en_de_juiste_grondslag() {
        let letters: Vec<_> = Zorgplichtonderdeel::alle().iter().map(|o| o.letter()).collect();
        assert_eq!(letters, ["a", "b", "c", "d", "e", "f", "g", "h", "i", "j"]);
        for o in Zorgplichtonderdeel::alle() {
            assert!(o.grondslag().contains("lid 3"));
            assert!(!o.grondslag().contains("lid 2"));
        }
    }

    /// Er is geen route waarlangs een maatregel bij de set komt of eruit gaat.
    #[test]
    fn het_dossier_telt_altijd_tien_onderdelen() {
        let d = dossier();
        for o in Zorgplichtonderdeel::alle() {
            assert!(d.maatregelen.iter().any(|m| m.onderdeel == o), "{o:?} ontbreekt");
        }
    }

    /// De aangemelde functionaris houdt geen toezicht op zijn eigen werk.
    #[test]
    fn de_functionaris_kan_geen_eigenaar_zijn() {
        let mut d = dossier();
        let fout =
            d.wijs_eigenaar_toe("CBB-a", "beleidsadviseur", "a. de vries", nu()).unwrap_err();
        assert!(fout.to_string().contains("art. 38 lid 6 AVG"), "kreeg: {fout}");

        d.wijs_eigenaar_toe("CBB-a", "beleidsadviseur", "J. Jansen", nu()).unwrap();
        assert!(d.eigenaarsconflicten().is_empty());
    }

    /// Bij een rolwissel kan een zittende eigenaar ineens de functionaris zijn.
    /// Dat is niet met een weigering te dichten en moet dus zichtbaar worden.
    #[test]
    fn een_rolwissel_maakt_het_conflict_zichtbaar() {
        let mut d = dossier();
        d.wijs_eigenaar_toe("CBB-a", "beleidsadviseur", "J. Jansen", nu()).unwrap();
        d.wijzig_aangemelde_functionaris("J. Jansen", nu()).unwrap();

        let conflicten = d.eigenaarsconflicten();
        assert_eq!(conflicten.len(), 1);
        assert_eq!(conflicten[0].code, "CBB-a");
    }

    #[test]
    fn een_eigenaar_zonder_rol_of_zonder_bezetting_wordt_geweigerd() {
        let mut d = dossier();
        assert!(d.wijs_eigenaar_toe("CBB-a", "  ", "J. Jansen", nu()).is_err());
        assert!(d.wijs_eigenaar_toe("CBB-a", "beleidsadviseur", "  ", nu()).is_err());
    }

    /// Inrichten maakt niets aantoonbaar. Dat is de kern van deze module.
    #[test]
    fn inrichten_levert_hoogstens_vastgesteld_niet_aantoonbaar_op() {
        let mut d = dossier();
        d.wijs_eigenaar_toe("CBB-a", "beleidsadviseur", "J. Jansen", nu()).unwrap();
        d.richt_in("CBB-a", nu()).unwrap();
        assert_eq!(
            d.maatregel("CBB-a").unwrap().stand(nu()),
            Maatregelstand::VastgesteldNietAantoonbaar
        );

        d.wijs_bewijs_aan(
            "CBB-a",
            bewijs(Bewijsrol::Uitvoering, nu() - Duration::days(10), nu() + Duration::days(300)),
            nu(),
        )
        .unwrap();
        assert_eq!(d.maatregel("CBB-a").unwrap().stand(nu()), Maatregelstand::Aantoonbaar);
    }

    /// Bewijs van de verkeerde soort maakt een maatregel niet aantoonbaar.
    #[test]
    fn vaststellingsbewijs_is_geen_uitvoeringsbewijs() {
        let mut d = dossier();
        d.wijs_eigenaar_toe("CBB-a", "beleidsadviseur", "J. Jansen", nu()).unwrap();
        d.richt_in("CBB-a", nu()).unwrap();
        d.wijs_bewijs_aan(
            "CBB-a",
            bewijs(Bewijsrol::Vaststelling, nu(), nu() + Duration::days(300)),
            nu(),
        )
        .unwrap();
        assert_eq!(
            d.maatregel("CBB-a").unwrap().stand(nu()),
            Maatregelstand::VastgesteldNietAantoonbaar
        );
    }

    /// Een stuk dat pas volgende maand ingaat, bewijst vandaag niets.
    #[test]
    fn bewijs_dat_nog_niet_geldt_telt_niet_mee() {
        let mut d = dossier();
        d.wijs_eigenaar_toe("CBB-a", "beleidsadviseur", "J. Jansen", nu()).unwrap();
        d.richt_in("CBB-a", nu()).unwrap();
        d.wijs_bewijs_aan(
            "CBB-a",
            bewijs(Bewijsrol::Uitvoering, nu() + Duration::days(30), nu() + Duration::days(400)),
            nu(),
        )
        .unwrap();
        assert_eq!(
            d.maatregel("CBB-a").unwrap().stand(nu()),
            Maatregelstand::VastgesteldNietAantoonbaar
        );
    }

    /// De stand valt vanzelf terug; er is niets dat iemand kan vergeten.
    #[test]
    fn verlopen_bewijs_laat_de_maatregel_terugvallen() {
        let mut d = dossier();
        d.wijs_eigenaar_toe("CBB-a", "beleidsadviseur", "J. Jansen", nu()).unwrap();
        d.richt_in("CBB-a", nu()).unwrap();
        d.wijs_bewijs_aan(
            "CBB-a",
            bewijs(Bewijsrol::Uitvoering, nu() - Duration::days(10), nu() + Duration::days(30)),
            nu(),
        )
        .unwrap();

        let m = d.maatregel("CBB-a").unwrap();
        assert_eq!(m.stand(nu()), Maatregelstand::Aantoonbaar);
        assert_eq!(m.stand(nu() + Duration::days(31)), Maatregelstand::VastgesteldNietAantoonbaar);
    }

    #[test]
    fn verlopen_bewijs_aanwijzen_wordt_geweigerd() {
        let mut d = dossier();
        let fout = d
            .wijs_bewijs_aan(
                "CBB-a",
                bewijs(
                    Bewijsrol::Uitvoering,
                    nu() - Duration::days(400),
                    nu() - Duration::days(10),
                ),
                nu(),
            )
            .unwrap_err();
        assert!(fout.to_string().contains("nu al verlopen"), "kreeg: {fout}");
    }

    #[test]
    fn een_verzonnen_bewijsverwijzing_wordt_geweigerd() {
        let mut d = dossier();
        let mut b = bewijs(Bewijsrol::Uitvoering, nu(), nu() + Duration::days(30));
        b.bijlagehash = "zie de map op de netwerkschijf".into();
        let fout = d.wijs_bewijs_aan("CBB-a", b, nu()).unwrap_err();
        assert!(fout.to_string().contains("niet door een verwijzing over te typen"));
    }

    /// Een onvoorwaardelijke eis is geen keuze die met een motivering te maken
    /// is.
    #[test]
    fn een_verboden_afwijking_wordt_geweigerd() {
        let mut k = kader();
        k.maatregelen[0].niettoepassingsvorm = Niettoepassingsvorm::Verboden;
        let mut d = Zorgplichtdossier::leid_af(
            "ZRP-2026",
            "Gemeente Voorbeeld",
            "A. de Vries",
            &k,
            None,
            "u1",
            nu(),
        )
        .unwrap();

        let fout = d
            .pas_niet_toe(
                "CBB-a",
                Niettoepassing::EigenMotivering(motivering(
                    "dit past niet bij onze omvang en middelen",
                )),
                nu(),
            )
            .unwrap_err();
        assert!(fout.to_string().contains("onvoorwaardelijk geformuleerd"), "kreeg: {fout}");
    }

    /// De vorm van de afwijking wordt door het kader bepaald, niet door de
    /// gebruiker.
    #[test]
    fn de_vorm_van_de_afwijking_volgt_het_kader() {
        let mut k = kader();
        k.maatregelen[0].niettoepassingsvorm = Niettoepassingsvorm::GrondslagInRegeling;
        let mut d = Zorgplichtdossier::leid_af(
            "ZRP-2026",
            "Gemeente Voorbeeld",
            "A. de Vries",
            &k,
            None,
            "u1",
            nu(),
        )
        .unwrap();

        let fout = d
            .pas_niet_toe(
                "CBB-a",
                Niettoepassing::EigenMotivering(motivering("dit past niet bij onze omvang")),
                nu(),
            )
            .unwrap_err();
        assert!(fout.to_string().contains("grondslag in de regeling zelf"), "kreeg: {fout}");

        d.pas_niet_toe(
            "CBB-a",
            Niettoepassing::GrondslagInRegeling {
                regeling: "de sectorale regeling".into(),
                artikel: "art. 4 lid 2".into(),
                motivering: motivering("de regeling voorziet hier zelf in een uitzondering"),
            },
            nu(),
        )
        .unwrap();

        // En andersom: een regelingsgrondslag waar het kader die niet vraagt.
        let mut d2 = dossier();
        let fout = d2
            .pas_niet_toe(
                "CBB-b",
                Niettoepassing::GrondslagInRegeling {
                    regeling: "de sectorale regeling".into(),
                    artikel: "art. 4 lid 2".into(),
                    motivering: motivering("de regeling voorziet hier zelf in"),
                },
                nu(),
            )
            .unwrap_err();
        assert!(fout.to_string().contains("dwingendheid suggereren"), "kreeg: {fout}");
    }

    /// Een motivering die de eis herhaalt, motiveert niets.
    #[test]
    fn een_motivering_die_de_eis_herhaalt_wordt_geweigerd() {
        let mut d = dossier();
        let eis = d.maatregel("CBB-a").unwrap().omschrijving.clone();
        let fout = d
            .pas_niet_toe("CBB-a", Niettoepassing::EigenMotivering(motivering(&eis)), nu())
            .unwrap_err();
        assert!(fout.to_string().contains("herhaalt de tekst van de eis"), "kreeg: {fout}");
    }

    #[test]
    fn een_frequentie_op_een_niet_periodieke_maatregel_wordt_geweigerd() {
        let mut d = dossier();
        let fout = d
            .stel_frequentie_vast(
                "CBB-a",
                12,
                "de directie",
                motivering("jaarlijks is passend"),
                nu(),
            )
            .unwrap_err();
        assert!(fout.to_string().contains("geen periodieke uitvoering"), "kreeg: {fout}");
    }

    #[test]
    fn een_frequentie_van_nul_maanden_wordt_geweigerd() {
        let mut k = kader();
        k.maatregelen[0].periodiek = true;
        let mut d = Zorgplichtdossier::leid_af(
            "ZRP-2026",
            "Gemeente Voorbeeld",
            "A. de Vries",
            &k,
            None,
            "u1",
            nu(),
        )
        .unwrap();
        assert!(d
            .stel_frequentie_vast("CBB-a", 0, "de directie", motivering("zo vaak als nodig"), nu())
            .is_err());
    }

    /// Een periodieke maatregel zonder eigen klok is geen oordeel maar een
    /// open eind.
    #[test]
    fn een_periodieke_maatregel_zonder_frequentie_vraagt_een_oordeel() {
        let mut k = kader();
        k.maatregelen[0].periodiek = true;
        let mut d = Zorgplichtdossier::leid_af(
            "ZRP-2026",
            "Gemeente Voorbeeld",
            "A. de Vries",
            &k,
            None,
            "u1",
            nu(),
        )
        .unwrap();
        d.wijs_eigenaar_toe("CBB-a", "beleidsadviseur", "J. Jansen", nu()).unwrap();
        d.richt_in("CBB-a", nu()).unwrap();
        assert_eq!(
            d.maatregel("CBB-a").unwrap().stand(nu()),
            Maatregelstand::MenselijkOordeelVereist
        );
    }

    /// Goedkeuring geldt per versie van het maatregelenpakket.
    #[test]
    fn een_besluit_over_een_andere_kaderversie_wordt_geweigerd() {
        let mut d = dossier();
        let fout = d
            .leg_bestuursvaststelling_vast(
                Bestuursvaststelling {
                    datum: nu() - Duration::days(5),
                    besluittekst: "het maatregelenpakket is vastgesteld".into(),
                    goedgekeurde_kaderversie: "2025-01-01".into(),
                    aanwezigen: vec!["de directie".into()],
                    bewijs: bewijs(
                        Bewijsrol::Vaststelling,
                        nu() - Duration::days(5),
                        nu() + Duration::days(300),
                    ),
                },
                nu(),
            )
            .unwrap_err();
        assert!(fout.to_string().contains("goedkeuring geldt per versie"), "kreeg: {fout}");
    }

    #[test]
    fn een_besluit_zonder_aanwezigen_wordt_geweigerd() {
        let mut d = dossier();
        let fout = d
            .leg_bestuursvaststelling_vast(
                Bestuursvaststelling {
                    datum: nu(),
                    besluittekst: "vastgesteld".into(),
                    goedgekeurde_kaderversie: "2026-08-01".into(),
                    aanwezigen: vec!["   ".into()],
                    bewijs: bewijs(Bewijsrol::Vaststelling, nu(), nu() + Duration::days(300)),
                },
                nu(),
            )
            .unwrap_err();
        assert!(fout.to_string().contains("geen vergaderbesluit"), "kreeg: {fout}");
    }

    #[test]
    fn een_risicobeoordeling_zonder_methode_wordt_geweigerd() {
        let mut d = dossier();
        let fout = d
            .leg_risicobeoordeling_vast(
                Risicobeoordeling {
                    omschrijving: "jaarlijkse beoordeling".into(),
                    methode: "  ".into(),
                    scope: "de hele organisatie".into(),
                    uitgevoerd_door: "de security officer".into(),
                    uitgevoerd_op: nu() - Duration::days(30),
                    geldig_tot: nu() + Duration::days(300),
                    bewijs: bewijs(
                        Bewijsrol::Toetsing,
                        nu() - Duration::days(30),
                        nu() + Duration::days(300),
                    ),
                },
                nu(),
            )
            .unwrap_err();
        assert!(fout.to_string().contains("methode en de reikwijdte"), "kreeg: {fout}");
    }

    /// Een kader dat niemand tegen de bron heeft gehouden, mag wel worden
    /// gebruikt maar niet worden vastgesteld.
    #[test]
    fn een_ongeverifieerd_kader_blokkeert_het_vaststellen() {
        let mut k = kader();
        k.geverifieerd_op = None;
        let d = Zorgplichtdossier::leid_af(
            "ZRP-2026",
            "Gemeente Voorbeeld",
            "A. de Vries",
            &k,
            None,
            "u1",
            nu(),
        )
        .unwrap();
        let velden: Vec<_> = d.ontbrekende_onderdelen().into_iter().map(|o| o.veld).collect();
        assert!(velden.contains(&"zorgplicht.kader".to_string()));
    }

    /// De teller mag nooit onder nul zakken doordat er meer ontbreekt dan er
    /// verplicht is. Dat is de stilste manier waarop zo een teller wegglijdt.
    #[test]
    fn de_teller_dekt_alles_wat_kan_ontbreken() {
        let mut k = kader();
        for m in &mut k.maatregelen {
            m.periodiek = true;
        }
        k.geverifieerd_op = None;
        let mut d = Zorgplichtdossier::leid_af(
            "ZRP-2026",
            "Gemeente Voorbeeld",
            "A. de Vries",
            &k,
            None,
            "u1",
            nu(),
        )
        .unwrap();
        // Alles ingericht, niets onderbouwd: dat levert het meeste op.
        for code in d.maatregelen.iter().map(|m| m.code.clone()).collect::<Vec<_>>() {
            d.richt_in(&code, nu()).unwrap();
        }
        let r = d.volledigheid();
        assert!(
            r.ontbreekt.len() <= r.verplicht,
            "{} ontbrekend tegenover {} verplicht",
            r.ontbreekt.len(),
            r.verplicht
        );
    }

    /// Een vastgesteld beleidsstuk bewijst geen uitvoering. Telde het toch mee,
    /// dan zou de teller "alles ingevuld" melden terwijl geen enkele maatregel
    /// aantoonbaar is — precies de weergave die het volledigheidsmechanisme
    /// uitsluit.
    #[test]
    fn vaststellingsbewijs_vult_het_gat_van_het_uitvoeringsbewijs_niet() {
        let mut d = dossier();
        for code in d.maatregelen.iter().map(|m| m.code.clone()).collect::<Vec<_>>() {
            d.wijs_eigenaar_toe(&code, "beleidsadviseur", "J. Jansen", nu()).unwrap();
            d.richt_in(&code, nu()).unwrap();
            d.wijs_bewijs_aan(
                &code,
                bewijs(Bewijsrol::Vaststelling, nu(), nu() + Duration::days(300)),
                nu(),
            )
            .unwrap();
        }
        let velden: Vec<_> = d.ontbrekende_onderdelen().into_iter().map(|o| o.veld).collect();
        assert!(
            velden.contains(&"zorgplicht.maatregel.CBB-a.bewijs".to_string()),
            "kreeg: {velden:?}"
        );
        assert!(d.standen(nu()).iter().all(|(s, _)| *s != Maatregelstand::Aantoonbaar));
    }

    #[test]
    fn vaststellen_kan_pas_als_alles_is_beoordeeld() {
        let mut d = dossier();
        assert!(d.stel_vast("A. de Vries", nu()).is_err());

        for code in d.maatregelen.iter().map(|m| m.code.clone()).collect::<Vec<_>>() {
            d.wijs_eigenaar_toe(&code, "beleidsadviseur", "J. Jansen", nu()).unwrap();
            d.richt_in(&code, nu()).unwrap();
        }
        d.leg_risicobeoordeling_vast(
            Risicobeoordeling {
                omschrijving: "jaarlijkse beoordeling".into(),
                methode: "scenarioanalyse".into(),
                scope: "de hele organisatie".into(),
                uitgevoerd_door: "de security officer".into(),
                uitgevoerd_op: nu() - Duration::days(30),
                geldig_tot: nu() + Duration::days(300),
                bewijs: bewijs(
                    Bewijsrol::Toetsing,
                    nu() - Duration::days(30),
                    nu() + Duration::days(300),
                ),
            },
            nu(),
        )
        .unwrap();
        d.leg_bestuursvaststelling_vast(
            Bestuursvaststelling {
                datum: nu() - Duration::days(5),
                besluittekst: "het maatregelenpakket is vastgesteld".into(),
                goedgekeurde_kaderversie: "2026-08-01".into(),
                aanwezigen: vec!["de directie".into()],
                bewijs: bewijs(
                    Bewijsrol::Vaststelling,
                    nu() - Duration::days(5),
                    nu() + Duration::days(300),
                ),
            },
            nu(),
        )
        .unwrap();

        d.stel_vast("A. de Vries", nu()).unwrap();
        assert_eq!(d.status, Status::Vastgesteld);

        // En toch is de helft niet aantoonbaar. Dat hoort zichtbaar te blijven.
        assert_eq!(d.standen(nu()), vec![(Maatregelstand::VastgesteldNietAantoonbaar, 10)]);
    }

    /// De vervallijst is een lijst met datums, geen prognose met een score.
    #[test]
    fn de_vervallijst_noemt_wat_er_omvalt_en_wanneer() {
        let mut d = dossier();
        d.wijs_eigenaar_toe("CBB-a", "beleidsadviseur", "J. Jansen", nu()).unwrap();
        d.richt_in("CBB-a", nu()).unwrap();
        d.wijs_bewijs_aan(
            "CBB-a",
            bewijs(Bewijsrol::Uitvoering, nu() - Duration::days(10), nu() + Duration::days(45)),
            nu(),
        )
        .unwrap();

        assert!(d.vervalt_voor(nu() + Duration::days(30), nu()).is_empty());
        let vervalt = d.vervalt_voor(nu() + Duration::days(90), nu());
        assert_eq!(vervalt.len(), 1);
        assert_eq!(vervalt[0].code, "CBB-a");
        assert_eq!(vervalt[0].eigenaar.as_ref().unwrap().persoon, "J. Jansen");
    }

    /// De noemer is het aantal maatregelen waar het kader afwijken toestaat.
    /// Zou hij de hele set zijn, dan kon het aandeel bij een kader met
    /// overwegend onvoorwaardelijke eisen nooit boven een zinnige drempel
    /// komen, en had regel ZRP-13 nooit kunnen aanslaan.
    #[test]
    fn het_aandeel_meet_over_de_maatregelen_waar_afwijken_mag() {
        let mut k = kader();
        for m in &mut k.maatregelen {
            m.niettoepassingsvorm = Niettoepassingsvorm::Verboden;
        }
        k.maatregelen[0].niettoepassingsvorm = Niettoepassingsvorm::EigenMotivering;
        k.maatregelen[1].niettoepassingsvorm = Niettoepassingsvorm::EigenMotivering;
        let mut d = Zorgplichtdossier::leid_af(
            "ZRP-2026",
            "Gemeente Voorbeeld",
            "A. de Vries",
            &k,
            None,
            "u1",
            nu(),
        )
        .unwrap();

        assert_eq!(d.aantal_afwijkbaar(), 2);
        assert_eq!(d.aandeel_niet_toegepast(), Some(0));

        d.pas_niet_toe(
            "CBB-a",
            Niettoepassing::EigenMotivering(motivering("dit past niet bij onze omvang")),
            nu(),
        )
        .unwrap();
        assert_eq!(d.aandeel_niet_toegepast(), Some(50));
    }

    /// Kan er nergens worden afgeweken, dan valt er niets te meten. Nul melden
    /// zou suggereren dat er niet wordt afgeweken terwijl er niet afgeweken
    /// kán worden.
    #[test]
    fn zonder_afwijkbare_maatregelen_is_er_geen_aandeel() {
        let mut k = kader();
        for m in &mut k.maatregelen {
            m.niettoepassingsvorm = Niettoepassingsvorm::Verboden;
        }
        let d = Zorgplichtdossier::leid_af(
            "ZRP-2026",
            "Gemeente Voorbeeld",
            "A. de Vries",
            &k,
            None,
            "u1",
            nu(),
        )
        .unwrap();
        assert_eq!(d.aandeel_niet_toegepast(), None);
    }

    /// Bewijs waarvan het venster nog moet ingaan, bewijst geen uitvoering die
    /// al heeft plaatsgevonden. Telde het mee, dan legde het aanwijzen van een
    /// toekomstige datum regel ZRP-08 het zwijgen op.
    #[test]
    fn toekomstig_bewijs_verschuift_de_laatste_uitvoering_niet() {
        let mut d = dossier();
        d.wijs_bewijs_aan(
            "CBB-a",
            bewijs(Bewijsrol::Uitvoering, nu() - Duration::days(400), nu() + Duration::days(30)),
            nu(),
        )
        .unwrap();
        assert_eq!(d.maatregel("CBB-a").unwrap().maanden_sinds_uitvoering(nu()), Some(13));

        let mut later =
            bewijs(Bewijsrol::Uitvoering, nu() + Duration::days(300), nu() + Duration::days(600));
        later.bijlagehash = "b".repeat(64);
        d.wijs_bewijs_aan("CBB-a", later, nu()).unwrap();
        assert_eq!(d.maatregel("CBB-a").unwrap().maanden_sinds_uitvoering(nu()), Some(13));
    }

    /// Bij twee geldige stukken wint de meest recente uitvoering, niet het
    /// ruimste venster: anders blijft een oud stuk met een lange looptijd een
    /// verse uitdraai overschaduwen.
    #[test]
    fn het_nieuwste_uitvoeringsbewijs_wint() {
        let mut d = dossier();
        d.wijs_eigenaar_toe("CBB-a", "beleidsadviseur", "J. Jansen", nu()).unwrap();
        d.richt_in("CBB-a", nu()).unwrap();
        d.wijs_bewijs_aan(
            "CBB-a",
            bewijs(Bewijsrol::Uitvoering, nu() - Duration::days(300), nu() + Duration::days(400)),
            nu(),
        )
        .unwrap();
        let mut vers =
            bewijs(Bewijsrol::Uitvoering, nu() - Duration::days(5), nu() + Duration::days(90));
        vers.bijlagehash = "c".repeat(64);
        d.wijs_bewijs_aan("CBB-a", vers, nu()).unwrap();

        let gekozen = d.maatregel("CBB-a").unwrap().geldig_uitvoeringsbewijs(nu()).unwrap();
        assert_eq!(gekozen.geldig_tot, nu() + Duration::days(90));
    }

    /// Hoe vaak iets gebeurt dat niet gebeurt, is geen zinnige vraag.
    #[test]
    fn een_niet_toegepaste_maatregel_vraagt_geen_uitvoeringstermijn() {
        let mut k = kader();
        k.maatregelen[0].periodiek = true;
        let mut d = Zorgplichtdossier::leid_af(
            "ZRP-2026",
            "Gemeente Voorbeeld",
            "A. de Vries",
            &k,
            None,
            "u1",
            nu(),
        )
        .unwrap();
        let velden: Vec<_> = d.ontbrekende_onderdelen().into_iter().map(|o| o.veld).collect();
        assert!(velden.contains(&"zorgplicht.maatregel.CBB-a.frequentie".to_string()));

        d.pas_niet_toe(
            "CBB-a",
            Niettoepassing::EigenMotivering(motivering("dit past niet bij onze omvang")),
            nu(),
        )
        .unwrap();
        let velden: Vec<_> = d.ontbrekende_onderdelen().into_iter().map(|o| o.veld).collect();
        assert!(!velden.contains(&"zorgplicht.maatregel.CBB-a.frequentie".to_string()));
    }

    /// De vervallijst meet aan het bewijs en niet aan de stand: juist het
    /// dossier waar nog het meeste openstaat, zou anders het rustigst ogen.
    #[test]
    fn de_vervallijst_slaat_wachtende_maatregelen_niet_over() {
        let mut d = dossier();
        // Geen eigenaar, dus de stand is menselijk oordeel vereist — maar het
        // uitvoeringsbewijs vervalt wel degelijk.
        d.richt_in("CBB-a", nu()).unwrap();
        d.wijs_bewijs_aan(
            "CBB-a",
            bewijs(Bewijsrol::Uitvoering, nu(), nu() + Duration::days(45)),
            nu(),
        )
        .unwrap();
        assert_eq!(
            d.maatregel("CBB-a").unwrap().stand(nu()),
            Maatregelstand::MenselijkOordeelVereist
        );

        let vervalt = d.vervalt_voor(nu() + Duration::days(90), nu());
        assert_eq!(vervalt.len(), 1);
        assert_eq!(vervalt[0].code, "CBB-a");
    }

    /// Een kader met een ontbrekend veld hoort te weigeren en niet stilzwijgend
    /// terug te vallen op de kant die controle wegneemt.
    #[test]
    fn een_kader_zonder_periodiek_wordt_niet_gelezen() {
        let json = serde_json::json!({
            "code": "CBB-06",
            "onderdeel": "beleid",
            "normvindplaats": "art. 6 Cbb",
            "omschrijving": "beleid voor informatiebeveiliging",
            "niettoepassingsvorm": "verboden",
            "externe_toetsing_verwacht": false
        });
        assert!(serde_json::from_value::<Kadermaatregel>(json).is_err());
    }

    /// De keuring gaat vooraf aan het opslaan, omdat het opslaan in de kluis
    /// niet terug te draaien is.
    #[test]
    fn een_bewijsaanwijzing_is_vooraf_te_keuren() {
        let d = dossier();
        assert!(d
            .keur_bewijs(
                "CBB-a",
                Bewijsrol::Uitvoering,
                "uitdraai",
                nu(),
                nu() + Duration::days(30),
                nu()
            )
            .is_ok());
        assert!(d
            .keur_bewijs(
                "BESTAAT-NIET",
                Bewijsrol::Uitvoering,
                "uitdraai",
                nu(),
                nu() + Duration::days(30),
                nu()
            )
            .is_err());
        assert!(d
            .keur_bewijs(
                "CBB-a",
                Bewijsrol::Uitvoering,
                "uitdraai",
                nu() - Duration::days(400),
                nu() - Duration::days(10),
                nu()
            )
            .is_err());
    }

    #[test]
    fn het_dossier_overleeft_serialisatie() {
        let mut d = dossier();
        d.wijs_eigenaar_toe("CBB-a", "beleidsadviseur", "J. Jansen", nu()).unwrap();
        d.richt_in("CBB-a", nu()).unwrap();
        d.wijs_bewijs_aan(
            "CBB-a",
            bewijs(Bewijsrol::Uitvoering, nu(), nu() + Duration::days(300)),
            nu(),
        )
        .unwrap();
        let json = serde_json::to_string(&d).unwrap();
        let terug: Zorgplichtdossier = serde_json::from_str(&json).unwrap();
        assert_eq!(d, terug);
    }
}
