//! Wat er in het logboek terechtkomt.
//!
//! Het logboek legt **handelingen** vast, niet gegevens. De inhoud van een
//! gewijzigd veld staat er niet in; wél wat er is gewijzigd, door wie, wanneer
//! en met welke motivering. Daarmee blijft het logboek bruikbaar als bewijs
//! zonder zelf een tweede kopie van de persoonsgegevens te worden — een
//! logboek dat de gegevens dupliceert, vergroot het lekoppervlak in plaats van
//! het te verkleinen.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Soort handeling. Bewust een gesloten opsomming: een vrij tekstveld zou
/// betekenen dat elke ontwikkelaar zijn eigen benaming verzint, waardoor
/// zoeken in het logboek onbetrouwbaar wordt.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Handeling {
    // --- levenscyclus van de kluis ---
    KluisAangemaakt,
    KluisGeopend,
    KluisGesloten,
    KluisOpenenMislukt,
    WachtwoordGewijzigd,
    SleutelGeroteerd,
    CompartimentAangemaakt,
    ParametersVerzwaard,

    // --- gegevens ---
    RecordAangemaakt,
    RecordGewijzigd,
    RecordVastgesteld,
    RecordIngetrokken,
    RecordHersteld,
    BijlageToegevoegd,
    BijlageVerwijderd,

    // --- besluiten met rechtsgevolg ---
    BesluitGenomen,
    BesluitOmgekeerd,
    TweedePersoonBevestigd,
    AfkoelperiodeGestart,
    MotiveringVastgelegd,

    // --- termijnen ---
    TermijnGestart,
    TermijnGestuit,
    TermijnHervat,
    TermijnVerstreken,
    TermijnVerlengd,

    // --- naar buiten ---
    DossierSamengesteld,
    DossierVerstrekt,
    ExportGemaakt,
    MeldingKlaargezet,
    MeldingVerzonden,

    // --- integriteit ---
    KetenGeverifieerd,
    AnkerGeplaatst,
    IntegriteitsfoutVastgesteld,
    KlokafwijkingVastgesteld,

    // --- controleregels ---
    ControleGeblokkeerd,
    ControleGenegeerd,
    ControleAfwijkingToegestaan,
}

impl Handeling {
    /// Geeft aan of deze handeling zonder tussenkomst van een mens onmogelijk is.
    ///
    /// Handelingen met rechtsgevolg moeten een genoemde actor hebben; een
    /// systeemtaak mag ze niet vastleggen.
    pub fn vereist_actor(&self) -> bool {
        !matches!(
            self,
            Self::TermijnVerstreken
                | Self::KetenGeverifieerd
                | Self::AnkerGeplaatst
                | Self::IntegriteitsfoutVastgesteld
                | Self::KlokafwijkingVastgesteld
        )
    }

    /// Geeft aan of deze handeling naar buiten werkt en dus onomkeerbaar is.
    pub fn is_onomkeerbaar(&self) -> bool {
        matches!(self, Self::MeldingVerzonden | Self::DossierVerstrekt | Self::ExportGemaakt)
    }
}

/// Wie de handeling verrichtte.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Actor {
    /// Vaste identificatie van de gebruiker binnen deze installatie.
    pub id: String,
    /// Weergavenaam op het moment van handelen; wordt niet meegewijzigd als de
    /// naam later verandert, want het logboek beschrijft het verleden.
    pub naam: String,
    /// Rol op het moment van handelen.
    pub rol: String,
}

impl Actor {
    pub fn nieuw(id: impl Into<String>, naam: impl Into<String>, rol: impl Into<String>) -> Self {
        Self { id: id.into(), naam: naam.into(), rol: rol.into() }
    }

    /// De handelende partij bij automatische taken.
    pub fn systeem() -> Self {
        Self::nieuw("systeem", "systeem", "systeem")
    }
}

/// Eén regel in het logboek, vóór ketening.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Gebeurtenis {
    /// Wat er gebeurde.
    pub handeling: Handeling,
    /// Wie het deed.
    pub actor: Actor,
    /// Tijdstip in UTC. Altijd UTC: de weergave in lokale tijd is een
    /// presentatiekwestie, opslag in lokale tijd maakt zomertijd tot een bug.
    pub tijdstip: DateTime<Utc>,
    /// Soort record waarop de handeling betrekking heeft, bijvoorbeeld
    /// `verwerking` of `datalek`.
    pub onderwerp_soort: String,
    /// Identificatie van het record.
    pub onderwerp_id: String,
    /// Compartiment waarin het record valt.
    pub compartiment: String,
    /// Beknopte omschrijving; bevat geen persoonsgegevens.
    pub omschrijving: String,
    /// Motivering, wanneer de handeling die vereist.
    pub motivering: Option<String>,
    /// Hash van de inhoud vóór de wijziging, wanneer van toepassing.
    pub inhoud_voor: Option<String>,
    /// Hash van de inhoud na de wijziging, wanneer van toepassing.
    pub inhoud_na: Option<String>,
}

impl Gebeurtenis {
    pub fn nieuw(
        handeling: Handeling,
        actor: Actor,
        tijdstip: DateTime<Utc>,
        onderwerp_soort: impl Into<String>,
        onderwerp_id: impl Into<String>,
        compartiment: impl Into<String>,
        omschrijving: impl Into<String>,
    ) -> Self {
        Self {
            handeling,
            actor,
            tijdstip,
            onderwerp_soort: onderwerp_soort.into(),
            onderwerp_id: onderwerp_id.into(),
            compartiment: compartiment.into(),
            omschrijving: omschrijving.into(),
            motivering: None,
            inhoud_voor: None,
            inhoud_na: None,
        }
    }

    pub fn met_motivering(mut self, motivering: impl Into<String>) -> Self {
        self.motivering = Some(motivering.into());
        self
    }

    pub fn met_inhoudswijziging(
        mut self,
        voor: Option<String>,
        na: Option<String>,
    ) -> Self {
        self.inhoud_voor = voor;
        self.inhoud_na = na;
        self
    }
}
