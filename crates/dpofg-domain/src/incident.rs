//! Het incident: datalek, beveiligingsincident, of beide tegelijk.
//!
//! # Waarom dit het scharnierpunt is
//!
//! Eén gebeurtenis kan vijf klokken starten in twee rechtsregimes, met
//! verschillende ankers en verschillende rekenregels:
//!
//! | Klok | Anker | Duur |
//! |---|---|---|
//! | melding aan de toezichthouder (AVG) | kennisname door de verwerkingsverantwoordelijke | 72 uur |
//! | mededeling aan betrokkenen (AVG) | vaststelling van een hoog risico | onverwijld |
//! | vroegtijdige waarschuwing (zorgplicht) | kennisname van een significant incident | 24 uur |
//! | incidentmelding (zorgplicht) | idem | 72 uur |
//! | eindrapport (zorgplicht) | verzending van de incidentmelding | 1 maand |
//!
//! De ankers vallen **niet** samen. De kennisname voor de AVG is een ander
//! moment dan de vaststelling dat een incident significant is, en het anker
//! van het eindrapport is de verzending van de melding — niet het incident
//! zelf (randgeval T-05). Wie één ankerveld gebruikt voor alle vijf, rekent
//! er minstens twee fout.
//!
//! # De gevaarlijkste beslissing in het product
//!
//! "Dit is geen meldenswaardig datalek" is de beslissing waar het in de
//! praktijk misgaat, en zij is onomkeerbaar in haar gevolgen: wie niet meldt
//! en het later toch had gemoeten, kan de 72 uur niet terughalen. Daarom
//! kent dit record de zwaarste beveiliging in het model — zie
//! [`Meldbesluit`].

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    basis::{Compartiment, Herkomst, Id, Motivering, Status},
    volledigheid::{Ontbrekend, Volledig},
    DomeinFout, Resultaat,
};

/// Hoe het incident aan het licht kwam.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Herkomstkanaal {
    /// Intern vastgesteld door de organisatie zelf.
    InternVastgesteld,
    /// Gemeld door een verwerker. Randgeval T-31: de klok van de
    /// verwerkingsverantwoordelijke start bij ontvangst van díe melding, niet
    /// bij het optreden van het incident bij de verwerker.
    MeldingVanVerwerker,
    /// Gemeld door een betrokkene.
    MeldingVanBetrokkene,
    /// Gemeld door een derde, bijvoorbeeld een onderzoeker.
    MeldingVanDerde,
    /// Vastgesteld door een toezichthouder of opsporingsdienst.
    ExterneInstantie,
}

/// De aantasting die het incident veroorzaakte.
///
/// Alle drie kunnen tegelijk spelen. Het beschikbaarheidsaspect wordt in de
/// praktijk het vaakst vergeten — een versleutelde back-up die onbereikbaar is,
/// is een datalek — en is daarom een apart, verplicht te beantwoorden veld
/// (controleregel LEK-09).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Aantasting {
    /// Onbevoegde kennisname of verstrekking.
    pub vertrouwelijkheid: bool,
    /// Onbevoegde wijziging of onbedoelde vernietiging.
    pub integriteit: bool,
    /// Verlies van toegang tot de gegevens.
    pub beschikbaarheid: bool,
}

impl Aantasting {
    /// Of ten minste één aspect is aangetast.
    pub fn is_aangetast(&self) -> bool {
        self.vertrouwelijkheid || self.integriteit || self.beschikbaarheid
    }
}

/// De uitkomst van de risicoweging.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Risiconiveau {
    /// Geen risico voor de rechten en vrijheden van betrokkenen.
    GeenRisico,
    /// Een risico: melden aan de toezichthouder.
    Risico,
    /// Een hoog risico: daarnaast mededeling aan de betrokkenen.
    HoogRisico,
}

impl Risiconiveau {
    /// Of dit niveau leidt tot een melding aan de toezichthouder.
    pub fn leidt_tot_melding(&self) -> bool {
        !matches!(self, Self::GeenRisico)
    }

    /// Of dit niveau leidt tot mededeling aan de betrokkenen.
    pub fn leidt_tot_mededeling(&self) -> bool {
        matches!(self, Self::HoogRisico)
    }

    pub fn omschrijving(&self) -> &'static str {
        match self {
            Self::GeenRisico => "geen risico voor de rechten en vrijheden van betrokkenen",
            Self::Risico => "een risico voor de rechten en vrijheden van betrokkenen",
            Self::HoogRisico => "een hoog risico voor de rechten en vrijheden van betrokkenen",
        }
    }
}

/// Het besluit om wel of niet te melden.
///
/// # De zwaarste beveiliging in het model
///
/// Een besluit om **niet** te melden kent drie lagen, elk met een andere
/// faalmodus, zodat één laag die faalt niet het hele besluit meesleept:
///
/// 1. **Verplichte motivering** — de weging wordt opgeschreven, niet alleen de
///    uitkomst. Faalt bij iemand die goed kan schrijven.
/// 2. **Tweede persoon** — een ander bevestigt. Faalt bij groepsdenken of een
///    afwezige collega.
/// 3. **Afkoelperiode** — het besluit wordt pas na een wachttijd definitief.
///    Faalt bij tijdsdruk, maar niet op dezelfde manier als de andere twee.
///
/// Bij bijzondere gegevens, een burgerservicenummer of financiële gegevens
/// vervalt de mogelijkheid om met de afkoelperiode alleen te volstaan: daar is
/// de tweede persoon verplicht (controleregel LEK-07).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Meldbesluit {
    /// Nog niet beoordeeld.
    NogTeNemen,
    /// Er wordt gemeld.
    Melden { motivering: Motivering },
    /// Er wordt niet gemeld. Dit besluit draagt zijn eigen waarborgen.
    NietMelden {
        motivering: Motivering,
        /// De tweede persoon die het besluit bevestigde.
        tweede_persoon: Option<String>,
        tweede_persoon_op: Option<DateTime<Utc>>,
        /// Wanneer de afkoelperiode afloopt en het besluit definitief wordt.
        afkoelperiode_tot: Option<DateTime<Utc>>,
        /// Of het besluit na tegenspraak alsnog omsloeg.
        ///
        /// Dit veld voedt de belangrijkste meetwaarde uit het plan: het
        /// omkeerpercentage. Is dat langdurig nul, dan werkt de barrière niet —
        /// hij bestaat alleen.
        omgekeerd_na_tegenspraak: bool,
    },
}

impl Meldbesluit {
    pub fn is_genomen(&self) -> bool {
        !matches!(self, Self::NogTeNemen)
    }

    pub fn is_niet_melden(&self) -> bool {
        matches!(self, Self::NietMelden { .. })
    }

    /// Of het besluit definitief is op een peilmoment.
    pub fn is_definitief(&self, nu: DateTime<Utc>) -> bool {
        match self {
            Self::NogTeNemen => false,
            Self::Melden { .. } => true,
            Self::NietMelden { afkoelperiode_tot, .. } => afkoelperiode_tot.is_none_or(|t| nu >= t),
        }
    }
}

/// De trap in de zorgplichtmeldketen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Zorgtrap {
    /// De vroegtijdige waarschuwing.
    Waarschuwing,
    /// De incidentmelding.
    Melding,
    /// Het eindrapport.
    Eindrapport,
    /// Het tussentijdse voortgangsrapport bij een lopend incident.
    Voortgang,
}

impl Zorgtrap {
    pub fn omschrijving(&self) -> &'static str {
        match self {
            Self::Waarschuwing => "vroegtijdige waarschuwing",
            Self::Melding => "incidentmelding",
            Self::Eindrapport => "eindrapport",
            Self::Voortgang => "voortgangsrapport",
        }
    }

    pub fn alle() -> [Self; 4] {
        [Self::Waarschuwing, Self::Melding, Self::Eindrapport, Self::Voortgang]
    }
}

/// Wanneer elke trap van de zorgplichtmeldketen is verzonden.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Zorgketenverzendingen {
    pub waarschuwing_op: Option<DateTime<Utc>>,
    pub melding_op: Option<DateTime<Utc>>,
    pub eindrapport_op: Option<DateTime<Utc>>,
    pub voortgang_op: Option<DateTime<Utc>>,
}

impl Zorgketenverzendingen {
    pub fn van(&self, trap: Zorgtrap) -> Option<DateTime<Utc>> {
        match trap {
            Zorgtrap::Waarschuwing => self.waarschuwing_op,
            Zorgtrap::Melding => self.melding_op,
            Zorgtrap::Eindrapport => self.eindrapport_op,
            Zorgtrap::Voortgang => self.voortgang_op,
        }
    }

    fn zet(&mut self, trap: Zorgtrap, op: DateTime<Utc>) {
        match trap {
            Zorgtrap::Waarschuwing => self.waarschuwing_op = Some(op),
            Zorgtrap::Melding => self.melding_op = Some(op),
            Zorgtrap::Eindrapport => self.eindrapport_op = Some(op),
            Zorgtrap::Voortgang => self.voortgang_op = Some(op),
        }
    }
}

/// Een incident.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Incident {
    pub id: Id,
    pub kenmerk: String,
    pub omschrijving: String,
    pub status: Status,
    pub compartiment: Compartiment,
    pub herkomst: Herkomst,

    // --- ankers ---
    /// Wanneer het incident zich voordeed, voor zover bekend.
    pub opgetreden_op: Option<DateTime<Utc>>,
    /// Wanneer het eerste signaal binnenkwam.
    pub signaal_op: DateTime<Utc>,
    /// Wanneer de organisatie een redelijke mate van zekerheid had.
    ///
    /// Dit is het anker van de 72-uursklok. Het ligt na het signaal: een korte
    /// eerste verificatie is toegestaan, mits die zelf wordt gedocumenteerd en
    /// niet als uitstelmechanisme wordt gebruikt.
    pub kennisname_op: Option<DateTime<Utc>>,
    /// Onderbouwing van de verificatieperiode tussen signaal en kennisname.
    pub verificatie_onderbouwing: Option<Motivering>,
    /// Wanneer het incident is geregistreerd in dit systeem.
    pub geregistreerd_op: DateTime<Utc>,
    /// Wanneer het als significant is aangemerkt, voor de zorgplichtketen.
    pub significant_vastgesteld_op: Option<DateTime<Utc>>,

    pub kanaal: Herkomstkanaal,
    /// Bij een melding van een verwerker: wanneer die melding binnenkwam.
    pub melding_verwerker_ontvangen_op: Option<DateTime<Utc>>,
    /// Bij een melding van een verwerker: wanneer het incident bij de verwerker
    /// optrad. Wordt vastgelegd maar is niet het anker.
    pub incident_bij_verwerker_op: Option<DateTime<Utc>>,
    pub verwerker_id: Option<Id>,

    // --- inhoud ---
    pub aantasting: Aantasting,
    pub aantal_betrokkenen: Option<u64>,
    pub aantal_betrokkenen_geschat: bool,
    pub categorieen_gegevens: Vec<String>,
    pub bijzondere_gegevens: bool,
    pub burgerservicenummer: bool,
    pub financiele_gegevens: bool,
    /// Of gegevensuitvoer naar buiten is uit te sluiten.
    pub exfiltratie_uitgesloten: Option<bool>,
    pub getroffen_verwerkingen: Vec<Id>,
    pub getroffen_systemen: Vec<Id>,

    // --- weging en besluit ---
    pub risiconiveau: Option<Risiconiveau>,
    pub risicoweging: Option<Motivering>,
    pub meldbesluit: Meldbesluit,
    pub gemeld_op: Option<DateTime<Utc>>,
    /// Het referentienummer dat de toezichthouder bij de melding teruggaf.
    ///
    /// Zonder referentie is een verzending later alleen met de eigen opgave te
    /// onderbouwen; met referentie is zij bij de ontvanger na te gaan.
    pub meldreferentie: Option<String>,
    pub mededeling_betrokkenen_besluit: Option<Motivering>,
    pub mededeling_betrokkenen_op: Option<DateTime<Utc>>,

    // --- afronding ---
    pub oorzaakcategorie: Option<String>,
    /// Verwijzingen naar records in het maatregelenregister.
    ///
    /// Dat register bestaat in deze uitgave nog niet; zolang dat zo is, wordt
    /// een maatregel bij de afronding als tekst vastgelegd in
    /// [`Incident::maatregelen_omschrijving`]. Twee velden voor één begrip is
    /// geen fraaie toestand, maar de andere twee wegen zwaarder: een
    /// verwijzing verzinnen naar een record dat niet bestaat maakt de
    /// verwijzing waardeloos, en het veld leeg laten zou betekenen dat elk
    /// afgerond incident een blokkerende bevinding houdt die niemand kan
    /// wegnemen.
    pub maatregelen: Vec<Id>,
    /// Maatregelen zoals bij de afronding opgeschreven.
    ///
    /// `serde(default)`: incidenten die met een eerdere uitgave zijn
    /// weggeschreven kennen dit veld niet, en die moeten leesbaar blijven.
    #[serde(default)]
    pub maatregelen_omschrijving: Vec<String>,
    pub afgehandeld_op: Option<DateTime<Utc>>,
    /// Wanneer de betrokkenen zijn geïnformeerd, bij een hoog risico.
    ///
    /// Zonder dit veld heeft de mededelingsplicht van artikel 34 geen enkele
    /// afdoening: de verplichting zou blijven staan tot het einde der tijden,
    /// ook nadat zij is nagekomen. Een lijst waaruit niets kan verdwijnen,
    /// leert de gebruiker haar te negeren.
    pub betrokkenen_geinformeerd_op: Option<DateTime<Utc>>,
    /// Wanneer de trappen van de zorgplichtmeldketen zijn verzonden.
    ///
    /// Drie afzonderlijke velden en geen enkele vlag: de vroegtijdige
    /// waarschuwing, de incidentmelding en het eindrapport zijn drie
    /// verzendingen met elk hun eigen termijn en hun eigen anker. Wie ze
    /// samentrekt, kan later niet meer laten zien welke van de drie te laat
    /// was.
    pub zorgketen: Zorgketenverzendingen,

    pub behandelaar: String,
}

impl Incident {
    pub fn nieuw(
        kenmerk: impl Into<String>,
        omschrijving: impl Into<String>,
        signaal_op: DateTime<Utc>,
        geregistreerd_op: DateTime<Utc>,
        kanaal: Herkomstkanaal,
        behandelaar: impl Into<String>,
        door: impl Into<String>,
    ) -> Self {
        Self {
            id: Id::nieuw(),
            kenmerk: kenmerk.into(),
            omschrijving: omschrijving.into(),
            status: Status::Concept,
            compartiment: Compartiment::nieuw(Compartiment::VERTROUWELIJK),
            herkomst: Herkomst::nieuw(door, geregistreerd_op),
            opgetreden_op: None,
            signaal_op,
            kennisname_op: None,
            verificatie_onderbouwing: None,
            geregistreerd_op,
            significant_vastgesteld_op: None,
            kanaal,
            melding_verwerker_ontvangen_op: None,
            incident_bij_verwerker_op: None,
            verwerker_id: None,
            aantasting: Aantasting {
                vertrouwelijkheid: false,
                integriteit: false,
                beschikbaarheid: false,
            },
            aantal_betrokkenen: None,
            aantal_betrokkenen_geschat: false,
            categorieen_gegevens: Vec::new(),
            bijzondere_gegevens: false,
            burgerservicenummer: false,
            financiele_gegevens: false,
            exfiltratie_uitgesloten: None,
            getroffen_verwerkingen: Vec::new(),
            getroffen_systemen: Vec::new(),
            risiconiveau: None,
            risicoweging: None,
            meldbesluit: Meldbesluit::NogTeNemen,
            gemeld_op: None,
            meldreferentie: None,
            mededeling_betrokkenen_besluit: None,
            mededeling_betrokkenen_op: None,
            oorzaakcategorie: None,
            maatregelen: Vec::new(),
            maatregelen_omschrijving: Vec::new(),
            afgehandeld_op: None,
            betrokkenen_geinformeerd_op: None,
            zorgketen: Zorgketenverzendingen::default(),
            behandelaar: behandelaar.into(),
        }
    }

    /// Of er bij de afronding geen enkele maatregel is vastgelegd.
    ///
    /// Telt beide vormen: een verwijzing naar het maatregelenregister en een
    /// omschrijving. Zolang dat register er niet is, is de omschrijving de
    /// enige vorm die de gebruiker kan invullen.
    pub fn zonder_maatregel(&self) -> bool {
        self.maatregelen.is_empty() && self.maatregelen_omschrijving.is_empty()
    }

    /// Legt het moment van kennisname vast.
    ///
    /// Randgeval T-08: kennisname vóór het optreden van het incident is een
    /// invoerfout en wordt blokkerend geweigerd, met een toelichting die zegt
    /// wat er waarschijnlijk is verwisseld.
    pub fn stel_kennisname_vast(
        &mut self,
        op: DateTime<Utc>,
        onderbouwing: Option<Motivering>,
    ) -> Resultaat<()> {
        if let Some(opgetreden) = self.opgetreden_op {
            if op < opgetreden {
                return Err(DomeinFout::OnmogelijkTijdstip {
                    veld: "kennisname_op".into(),
                    reden: format!(
                        "kennisname op {} ligt vóór het optreden van het incident op {}. \
                         Waarschijnlijk zijn beide velden verwisseld",
                        op.to_rfc3339(),
                        opgetreden.to_rfc3339()
                    ),
                });
            }
        }
        if op < self.signaal_op {
            return Err(DomeinFout::OnmogelijkTijdstip {
                veld: "kennisname_op".into(),
                reden: format!(
                    "kennisname op {} ligt vóór het eerste signaal op {}",
                    op.to_rfc3339(),
                    self.signaal_op.to_rfc3339()
                ),
            });
        }
        self.kennisname_op = Some(op);
        self.verificatie_onderbouwing = onderbouwing;
        Ok(())
    }

    /// Legt vast dat dit incident bij een verwerker optrad en wanneer diens
    /// melding binnenkwam.
    ///
    /// Beide tijdstippen worden bewaard, maar ze doen verschillend werk. Het
    /// moment waarop de verwerker meldde, is het anker van de eigen klok van
    /// tweeënzeventig uur; het moment waarop het incident bij de verwerker
    /// optrad, is het begin van de contractuele meldtermijn van die verwerker.
    /// Regel LEK-16 rekent het verschil na.
    pub fn leg_verwerkersmelding_vast(
        &mut self,
        verwerker: Id,
        opgetreden_bij_verwerker: Option<DateTime<Utc>>,
        melding_ontvangen: Option<DateTime<Utc>>,
        nu: DateTime<Utc>,
    ) -> Resultaat<()> {
        for (veld, tijdstip) in [
            ("incident_bij_verwerker_op", opgetreden_bij_verwerker),
            ("melding_verwerker_ontvangen_op", melding_ontvangen),
        ] {
            if let Some(t) = tijdstip {
                if t > nu {
                    return Err(DomeinFout::OnmogelijkTijdstip {
                        veld: veld.into(),
                        reden: format!("{} ligt in de toekomst", t.to_rfc3339()),
                    });
                }
            }
        }
        if let (Some(opgetreden), Some(ontvangen)) = (opgetreden_bij_verwerker, melding_ontvangen) {
            if ontvangen < opgetreden {
                return Err(DomeinFout::OnmogelijkTijdstip {
                    veld: "melding_verwerker_ontvangen_op".into(),
                    reden: format!(
                        "de melding kwam binnen op {}, vóór het incident bij de verwerker \
                         optrad op {}. Waarschijnlijk zijn beide velden verwisseld",
                        ontvangen.to_rfc3339(),
                        opgetreden.to_rfc3339()
                    ),
                });
            }
        }

        self.verwerker_id = Some(verwerker);
        if opgetreden_bij_verwerker.is_some() {
            self.incident_bij_verwerker_op = opgetreden_bij_verwerker;
        }
        if melding_ontvangen.is_some() {
            self.melding_verwerker_ontvangen_op = melding_ontvangen;
        }
        Ok(())
    }

    /// Legt vast dat de melding aan de toezichthouder is verzonden.
    ///
    /// Dit is de handeling die de meldklok afdoet. Zij bestond niet: het veld
    /// `gemeld_op` werd nergens gezet, waardoor de verplichting nooit uit een
    /// werkvoorraad kon verdwijnen en het eindrapport uit de zorgplichtketen
    /// geen anker kreeg.
    ///
    /// De verzending is een feit en geen besluit. Of er gemeld moest worden,
    /// is een eerdere vraag met een eigen bewaking; deze methode registreert
    /// alleen dát en wanneer.
    pub fn leg_melding_vast(
        &mut self,
        op: DateTime<Utc>,
        referentie: Option<String>,
        nu: DateTime<Utc>,
    ) -> Resultaat<()> {
        if let Some(eerder) = self.gemeld_op {
            return Err(DomeinFout::OngeldigeStatusovergang {
                van: "gemeld".into(),
                naar: "gemeld".into(),
                reden: format!(
                    "er is al een melding vastgelegd op {}; een tweede verzending is een \\
                     aanvulling en hoort als zodanig te worden vastgelegd",
                    eerder.to_rfc3339()
                ),
            });
        }
        if op > nu {
            return Err(DomeinFout::OnmogelijkTijdstip {
                veld: "gemeld_op".into(),
                reden: "de melding zou in de toekomst zijn verzonden".into(),
            });
        }
        let anker = self.anker_meldklok().ok_or(DomeinFout::OntbrekendeVerwijzing {
            veld: "gemeld_op".into(),
            naar: "kennisname, want de meldklok heeft nog geen anker".into(),
        })?;
        if op < anker {
            return Err(DomeinFout::OnmogelijkTijdstip {
                veld: "gemeld_op".into(),
                reden: format!(
                    "de melding zou zijn verzonden op {}, vóór het anker van de meldklok op {}",
                    op.to_rfc3339(),
                    anker.to_rfc3339()
                ),
            });
        }
        if self.meldbesluit.is_niet_melden() {
            return Err(DomeinFout::OngeldigeStatusovergang {
                van: "besloten niet te melden".into(),
                naar: "gemeld".into(),
                reden: "er ligt een besluit om niet te melden; keer dat eerst om, zodat de \\
                        omkering met haar motivering in het logboek staat"
                    .into(),
            });
        }
        self.gemeld_op = Some(op);
        self.meldreferentie = referentie.filter(|r| !r.trim().is_empty());
        self.herkomst.wijzig("melding aan de toezichthouder vastgelegd", nu);
        Ok(())
    }

    /// Legt vast dat een trap van de zorgplichtmeldketen is verzonden.
    ///
    /// De volgorde wordt afgedwongen: het eindrapport hangt aan de verzending
    /// van de incidentmelding, en die aan de waarschuwing. Een eindrapport
    /// vastleggen zonder melding zou een keten opleveren waarin de tweede trap
    /// ontbreekt terwijl de derde is afgedaan.
    pub fn leg_zorgverzending_vast(
        &mut self,
        trap: Zorgtrap,
        op: DateTime<Utc>,
        nu: DateTime<Utc>,
    ) -> Resultaat<()> {
        if let Some(eerder) = self.zorgketen.van(trap) {
            return Err(DomeinFout::OngeldigeStatusovergang {
                van: "verzonden".into(),
                naar: "verzonden".into(),
                reden: format!(
                    "de {} is al vastgelegd op {}",
                    trap.omschrijving(),
                    eerder.to_rfc3339()
                ),
            });
        }
        if op > nu {
            return Err(DomeinFout::OnmogelijkTijdstip {
                veld: "zorgketen".into(),
                reden: format!("de {} zou in de toekomst zijn verzonden", trap.omschrijving()),
            });
        }
        let voorwaarde = match trap {
            Zorgtrap::Waarschuwing => None,
            Zorgtrap::Melding => Some((Zorgtrap::Waarschuwing, self.zorgketen.waarschuwing_op)),
            Zorgtrap::Eindrapport | Zorgtrap::Voortgang => {
                Some((Zorgtrap::Melding, self.zorgketen.melding_op))
            }
        };
        if let Some((eerdere, moment)) = voorwaarde {
            let Some(moment) = moment else {
                return Err(DomeinFout::OntbrekendeVerwijzing {
                    veld: "zorgketen".into(),
                    naar: format!(
                        "verzonden {}; die gaat aan de {} vooraf",
                        eerdere.omschrijving(),
                        trap.omschrijving()
                    ),
                });
            };
            if op < moment {
                return Err(DomeinFout::OnmogelijkTijdstip {
                    veld: "zorgketen".into(),
                    reden: format!(
                        "de {} zou vóór de {} zijn verzonden",
                        trap.omschrijving(),
                        eerdere.omschrijving()
                    ),
                });
            }
        }
        self.zorgketen.zet(trap, op);
        self.herkomst.wijzig(format!("{} verzonden", trap.omschrijving()), nu);
        Ok(())
    }

    /// Legt vast dat de betrokkenen zijn geïnformeerd.
    pub fn leg_mededeling_vast(&mut self, op: DateTime<Utc>, nu: DateTime<Utc>) -> Resultaat<()> {
        if !self.risiconiveau.is_some_and(|r| r.leidt_tot_mededeling()) {
            return Err(DomeinFout::OngeldigeStatusovergang {
                van: "geen hoog risico".into(),
                naar: "betrokkenen geïnformeerd".into(),
                reden: "de mededeling aan betrokkenen hoort bij een hoog risico; is dat de \\
                        uitkomst van de weging, leg die dan eerst vast"
                    .into(),
            });
        }
        if op > nu {
            return Err(DomeinFout::OnmogelijkTijdstip {
                veld: "betrokkenen_geinformeerd_op".into(),
                reden: "de mededeling zou in de toekomst zijn gedaan".into(),
            });
        }
        self.betrokkenen_geinformeerd_op = Some(op);
        self.herkomst.wijzig("betrokkenen geïnformeerd", nu);
        Ok(())
    }

    /// Het anker van de 72-uursklok van artikel 33 AVG.
    ///
    /// Bij een melding van een verwerker is dat het moment waarop díe melding
    /// binnenkwam, en niet het moment waarop het incident bij de verwerker
    /// optrad (randgeval T-31). Beide tijdstippen worden vastgelegd; alleen het
    /// eerste telt.
    pub fn anker_meldklok(&self) -> Option<DateTime<Utc>> {
        match self.kanaal {
            Herkomstkanaal::MeldingVanVerwerker => {
                self.melding_verwerker_ontvangen_op.or(self.kennisname_op)
            }
            _ => self.kennisname_op,
        }
    }

    /// Hoe lang de verificatieperiode duurde.
    pub fn verificatieduur(&self) -> Option<Duration> {
        self.kennisname_op.map(|k| k - self.signaal_op)
    }

    /// Het gat tussen kennisname en registratie in dit systeem.
    ///
    /// Controleregel LEK-03: meer dan vier uur zonder toelichting is
    /// blokkerend. Een incident dat een halve dag in een mailbox blijft liggen
    /// voordat iemand het registreert, is de meest voorkomende manier waarop de
    /// 72 uur verdampt.
    pub fn registratievertraging(&self) -> Option<Duration> {
        self.kennisname_op.map(|k| self.geregistreerd_op - k)
    }

    /// Of het besluit "geen risico" een tweede persoon vereist.
    ///
    /// Controleregel LEK-07: bij bijzondere gegevens, een burgerservicenummer
    /// of financiële gegevens is een tweede persoon verplicht en volstaat de
    /// afkoelperiode niet.
    pub fn tweede_persoon_verplicht(&self) -> bool {
        self.bijzondere_gegevens || self.burgerservicenummer || self.financiele_gegevens
    }

    /// Of de omvang tegenspraak vereist.
    ///
    /// Controleregel LEK-06: "geen risico" bij meer dan tweehonderdvijftig
    /// betrokkenen is niet onmogelijk, maar wel iets waar iemand naar hoort te
    /// kijken.
    pub fn omvang_vereist_tegenspraak(&self) -> bool {
        self.aantal_betrokkenen.is_some_and(|n| n > 250)
    }

    /// Neemt het besluit om niet te melden.
    ///
    /// Faalt wanneer de vereiste waarborgen ontbreken. Deze functie is de plek
    /// waar het product het hardst tegenspreekt, en dat is bewust: dit is de
    /// enige beslissing waarvan de gevolgen niet terug te draaien zijn met een
    /// herstelknop.
    pub fn besluit_niet_melden(
        &mut self,
        motivering: Motivering,
        tweede_persoon: Option<String>,
        nu: DateTime<Utc>,
        afkoelperiode: Duration,
    ) -> Resultaat<()> {
        if self.risiconiveau.is_none() {
            return Err(DomeinFout::OngeldigeWaarde {
                veld: "risiconiveau".into(),
                reden: "weeg eerst het risico; zonder weging is er geen besluit te nemen".into(),
            });
        }
        if self.risiconiveau.is_some_and(|r| r.leidt_tot_melding()) {
            return Err(DomeinFout::OngeldigeWaarde {
                veld: "meldbesluit".into(),
                reden: format!(
                    "de weging kwam uit op '{}'; niet melden is dan niet te rijmen met de weging. \
                     Herzie eerst de weging of meld",
                    self.risiconiveau.unwrap().omschrijving()
                ),
            });
        }
        if self.exfiltratie_uitgesloten.is_none() {
            return Err(DomeinFout::OngeldigeWaarde {
                veld: "exfiltratie_uitgesloten".into(),
                reden: "beantwoord eerst of gegevensuitvoer naar buiten is uit te sluiten".into(),
            });
        }
        if !self.aantasting.is_aangetast() {
            return Err(DomeinFout::OngeldigeWaarde {
                veld: "aantasting".into(),
                reden: "geen enkel aspect is aangeraakt; beoordeel eerst vertrouwelijkheid, \
                        integriteit en beschikbaarheid"
                    .into(),
            });
        }
        if self.tweede_persoon_verplicht() && tweede_persoon.is_none() {
            return Err(DomeinFout::TweedePersoonVereist {
                handeling: "besluiten om niet te melden".into(),
                reden: format!(
                    "dit incident raakt {}; bij deze gegevens is bevestiging door een tweede \
                     persoon verplicht en volstaat een afkoelperiode niet",
                    self.gevoelige_kenmerken().join(" en ")
                ),
            });
        }
        if tweede_persoon.is_none() && afkoelperiode.is_zero() {
            return Err(DomeinFout::TweedePersoonVereist {
                handeling: "besluiten om niet te melden".into(),
                reden: "kies een tweede persoon, of stel een afkoelperiode in waarna het besluit \
                        definitief wordt"
                    .into(),
            });
        }

        self.meldbesluit = Meldbesluit::NietMelden {
            motivering,
            tweede_persoon: tweede_persoon.clone(),
            tweede_persoon_op: tweede_persoon.as_ref().map(|_| nu),
            afkoelperiode_tot: if afkoelperiode.is_zero() {
                None
            } else {
                Some(nu + afkoelperiode)
            },
            omgekeerd_na_tegenspraak: false,
        };
        Ok(())
    }

    /// Draait een besluit om niet te melden terug.
    ///
    /// Dit is geen uitzondering maar een verwachte gebeurtenis: het
    /// omkeerpercentage is de maat waaraan af te lezen is of de tegenspraak
    /// werkt.
    pub fn keer_besluit_om(&mut self, motivering: Motivering) -> Resultaat<()> {
        match &self.meldbesluit {
            Meldbesluit::NietMelden { .. } => {
                self.meldbesluit = Meldbesluit::Melden { motivering };
                if let Meldbesluit::Melden { .. } = self.meldbesluit {
                    // Het feit dát er is omgekeerd blijft vindbaar in het
                    // auditspoor; hier telt alleen de nieuwe toestand.
                }
                Ok(())
            }
            _ => Err(DomeinFout::OngeldigeStatusovergang {
                van: "meldbesluit".into(),
                naar: "melden".into(),
                reden: "er ligt geen besluit om niet te melden dat kan worden omgekeerd".into(),
            }),
        }
    }

    fn gevoelige_kenmerken(&self) -> Vec<&'static str> {
        let mut uit = Vec::new();
        if self.bijzondere_gegevens {
            uit.push("bijzondere persoonsgegevens");
        }
        if self.burgerservicenummer {
            uit.push("het burgerservicenummer");
        }
        if self.financiele_gegevens {
            uit.push("financiële gegevens");
        }
        uit
    }
}

impl Volledig for Incident {
    fn soortnaam(&self) -> &'static str {
        "incident"
    }

    fn aantal_verplichte_onderdelen(&self) -> usize {
        // kennisname, aantasting, gegevenscategorieën, aantal betrokkenen,
        // exfiltratie, risiconiveau, risicoweging, meldbesluit, oorzaak, maatregel
        10
    }

    fn ontbrekende_onderdelen(&self) -> Vec<Ontbrekend> {
        let mut uit = Vec::new();

        if self.kennisname_op.is_none() {
            uit.push(Ontbrekend::blokkerend(
                "incident.kennisname_op",
                "leg vast wanneer de organisatie redelijke zekerheid had; hierop start de 72-uursklok",
                "art. 33 lid 1 AVG",
            ));
        }
        if !self.aantasting.is_aangetast() {
            uit.push(Ontbrekend::blokkerend(
                "incident.aantasting",
                "beoordeel vertrouwelijkheid, integriteit én beschikbaarheid; ook verlies van \
                 toegang is een inbreuk",
                "art. 4 onder 12 AVG",
            ));
        }
        if self.categorieen_gegevens.is_empty() {
            uit.push(Ontbrekend::blokkerend(
                "incident.categorieen_gegevens",
                "benoem welke soorten gegevens het betreft",
                "art. 33 lid 3 onder a AVG",
            ));
        }
        if self.aantal_betrokkenen.is_none() {
            uit.push(Ontbrekend::blokkerend(
                "incident.aantal_betrokkenen",
                "geef het aantal betrokkenen, desnoods als schatting met die aanduiding erbij",
                "art. 33 lid 3 onder a AVG",
            ));
        }
        if self.exfiltratie_uitgesloten.is_none() {
            uit.push(Ontbrekend::blokkerend(
                "incident.exfiltratie_uitgesloten",
                "beantwoord of gegevensuitvoer naar buiten is uit te sluiten",
                "art. 33 lid 3 onder c AVG",
            ));
        }
        if self.risiconiveau.is_none() {
            uit.push(Ontbrekend::blokkerend(
                "incident.risiconiveau",
                "weeg het risico voor de rechten en vrijheden van de betrokkenen",
                "art. 33 lid 1 en art. 34 lid 1 AVG",
            ));
        }
        if self.risicoweging.is_none() {
            uit.push(Ontbrekend::blokkerend(
                "incident.risicoweging",
                "schrijf de weging op, niet alleen de uitkomst",
                "art. 5 lid 2 AVG",
            ));
        }
        if !self.meldbesluit.is_genomen() {
            uit.push(Ontbrekend::blokkerend(
                "incident.meldbesluit",
                "neem het besluit om wel of niet te melden",
                "art. 33 lid 1 AVG",
            ));
        }
        if self.oorzaakcategorie.is_none() {
            uit.push(Ontbrekend::blokkerend(
                "incident.oorzaakcategorie",
                "kies de oorzaakcategorie; zonder oorzaak is er geen patroon te zien",
                "art. 33 lid 5 AVG",
            ));
        }
        if self.zonder_maatregel() {
            uit.push(Ontbrekend::blokkerend(
                "incident.maatregelen",
                "leg ten minste één maatregel vast met een eigenaar",
                "art. 33 lid 3 onder d AVG",
            ));
        }

        uit
    }
}
