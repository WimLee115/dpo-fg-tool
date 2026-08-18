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
            Self::NietMelden { afkoelperiode_tot, .. } => {
                afkoelperiode_tot.is_none_or(|t| nu >= t)
            }
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
    pub mededeling_betrokkenen_besluit: Option<Motivering>,
    pub mededeling_betrokkenen_op: Option<DateTime<Utc>>,

    // --- afronding ---
    pub oorzaakcategorie: Option<String>,
    pub maatregelen: Vec<Id>,
    pub afgehandeld_op: Option<DateTime<Utc>>,

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
            mededeling_betrokkenen_besluit: None,
            mededeling_betrokkenen_op: None,
            oorzaakcategorie: None,
            maatregelen: Vec::new(),
            afgehandeld_op: None,
            behandelaar: behandelaar.into(),
        }
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
        if self.maatregelen.is_empty() {
            uit.push(Ontbrekend::blokkerend(
                "incident.maatregelen",
                "leg ten minste één maatregel vast met een eigenaar",
                "art. 33 lid 3 onder d AVG",
            ));
        }

        uit
    }
}
