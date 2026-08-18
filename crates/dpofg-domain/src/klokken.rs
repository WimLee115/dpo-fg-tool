//! Welke verplichtingen uit één gebeurtenis ontstaan.
//!
//! Dit is de vertaling van "de ochtend waarop een incident binnenkomt en er
//! vijf klokken tegelijk gaan lopen" naar code. De regel is telkens dezelfde:
//! **de tool leidt de verplichting af, de gebruiker hoeft de regel niet te
//! kennen.**
//!
//! # Wat hier bewust níet gebeurt
//!
//! De duren en grondslagen staan niet in deze module. Zij komen uit het
//! kennispakket (ontwerpprincipe P1). Wat hier staat is uitsluitend de
//! *afleidingslogica*: welk anker hoort bij welke verplichting, en onder welke
//! voorwaarde die ontstaat. Dat is domeinlogica en geen juridische inhoud —
//! als een termijn van 72 naar 48 uur gaat, hoeft aan deze module niets te
//! veranderen.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    basis::Id,
    incident::{Incident, Meldbesluit},
    Risiconiveau,
};

/// De code van een verplichting, zoals het kennispakket die kent.
///
/// Een code en geen duur: de duur hoort bij de code in het kennispakket.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Verplichtingcode(String);

impl Verplichtingcode {
    /// Melding aan de toezichthouder na een inbreuk.
    pub const AVG_MELDING: &'static str = "AVG-33-MELDING";
    /// Mededeling aan de betrokkenen bij een hoog risico.
    pub const AVG_MEDEDELING: &'static str = "AVG-34-MEDEDELING";
    /// Vastlegging in het interne register, ook bij niet melden.
    pub const AVG_INTERN_REGISTER: &'static str = "AVG-33-5-REGISTER";
    /// Vroegtijdige waarschuwing in de zorgplichtketen.
    pub const ZORG_WAARSCHUWING: &'static str = "ZORG-WAARSCHUWING";
    /// Incidentmelding in de zorgplichtketen.
    pub const ZORG_MELDING: &'static str = "ZORG-MELDING";
    /// Eindrapport in de zorgplichtketen.
    pub const ZORG_EINDRAPPORT: &'static str = "ZORG-EINDRAPPORT";
    /// Tussentijds voortgangsrapport bij een lopend incident.
    pub const ZORG_VOORTGANG: &'static str = "ZORG-VOORTGANG";

    pub fn nieuw(code: impl Into<String>) -> Self {
        Self(code.into())
    }

    pub fn code(&self) -> &str {
        &self.0
    }
}

/// Waarop een verplichting is verankerd.
///
/// Het ankertype is belangrijker dan het tijdstip: het legt vast *waarom* dit
/// het startmoment is, zodat een latere correctie van dat moment alle
/// afhankelijke klokken meeneemt (randgeval T-09).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Ankertype {
    /// Kennisname door de organisatie.
    Kennisname,
    /// Ontvangst van een melding van een verwerker.
    OntvangstVerwerkersmelding,
    /// Vaststelling dat er een hoog risico is.
    VaststellingHoogRisico,
    /// Vaststelling dat het incident significant is.
    VaststellingSignificant,
    /// Verzending van een eerdere melding.
    VerzendingMelding,
    /// Afronding van de afhandeling.
    Afhandeling,
}

impl Ankertype {
    pub fn omschrijving(&self) -> &'static str {
        match self {
            Self::Kennisname => "kennisname door de organisatie",
            Self::OntvangstVerwerkersmelding => "ontvangst van de melding van de verwerker",
            Self::VaststellingHoogRisico => "vaststelling van een hoog risico",
            Self::VaststellingSignificant => "vaststelling dat het incident significant is",
            Self::VerzendingMelding => "verzending van de melding",
            Self::Afhandeling => "afronding van de afhandeling",
        }
    }
}

/// Een afgeleide verplichting: wat moet, wanneer, en waarom dan.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AfgeleideVerplichting {
    /// Het dossier waaruit deze verplichting volgt.
    pub bron_id: Id,
    /// De code waarmee het kennispakket de duur en grondslag levert.
    pub code: Verplichtingcode,
    /// Waarop de klok is verankerd.
    pub ankertype: Ankertype,
    /// Het ankertijdstip, wanneer dat al bekend is.
    pub anker: Option<DateTime<Utc>>,
    /// Waarom deze verplichting is ontstaan, in gewone taal.
    ///
    /// Verschijnt bij de klok in beeld, zodat de gebruiker ziet welk antwoord
    /// van hem deze verplichting heeft opgeroepen.
    pub reden: String,
    /// Of het anker nog ontbreekt en de klok dus nog niet loopt.
    pub wacht_op_anker: bool,
}

impl AfgeleideVerplichting {
    fn nieuw(
        bron_id: Id,
        code: &str,
        ankertype: Ankertype,
        anker: Option<DateTime<Utc>>,
        reden: impl Into<String>,
    ) -> Self {
        Self {
            bron_id,
            code: Verplichtingcode::nieuw(code),
            ankertype,
            anker,
            reden: reden.into(),
            wacht_op_anker: anker.is_none(),
        }
    }
}

/// Of de organisatie onder de zorgplichtketen valt voor dit incident.
///
/// Deze vaststelling komt van buiten dit model: zij volgt uit de classificatie
/// van de entiteit en uit de significantietoets. De klokkenmotor beslist daar
/// niet over; hij vraagt ernaar.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Zorgplichtcontext {
    /// Of de entiteit onder de meldketen valt.
    pub valt_onder_meldketen: bool,
    /// Of dit incident als significant is aangemerkt.
    pub is_significant: bool,
}

impl Zorgplichtcontext {
    pub fn niet_van_toepassing() -> Self {
        Self { valt_onder_meldketen: false, is_significant: false }
    }
}

/// Leidt alle verplichtingen af die uit dit incident volgen.
///
/// De uitkomst is bewust ook gevuld wanneer ankers nog ontbreken: een
/// verplichting die *gaat* ontstaan hoort zichtbaar te zijn vóórdat de klok
/// loopt, niet erna. Zie het veld `wacht_op_anker`.
pub fn verplichtingen_uit_incident(
    incident: &Incident,
    zorgplicht: Zorgplichtcontext,
) -> Vec<AfgeleideVerplichting> {
    let mut uit = Vec::new();
    let id = incident.id;

    // --- 1. De meldklok van artikel 33 AVG ---
    // Ontstaat zodra er persoonsgegevens in het spel zijn. Het besluit om niet
    // te melden laat de klok niet verdwijnen: hij blijft lopen tot het besluit
    // definitief is, want een omkering moet nog binnen de termijn passen.
    let (ankertype, anker) = match incident.kanaal {
        crate::incident::Herkomstkanaal::MeldingVanVerwerker => (
            Ankertype::OntvangstVerwerkersmelding,
            incident.melding_verwerker_ontvangen_op.or(incident.kennisname_op),
        ),
        _ => (Ankertype::Kennisname, incident.kennisname_op),
    };
    uit.push(AfgeleideVerplichting::nieuw(
        id,
        Verplichtingcode::AVG_MELDING,
        ankertype,
        anker,
        format!("de klok loopt vanaf {}", ankertype.omschrijving()),
    ));

    // --- 2. De mededeling aan betrokkenen ---
    // Alleen bij een hoog risico, en verankerd op de vaststelling daarvan —
    // niet op de kennisname. Wie beide op hetzelfde anker zet, laat de tweede
    // klok te vroeg aflopen.
    if incident.risiconiveau.is_some_and(|r| r.leidt_tot_mededeling()) {
        uit.push(AfgeleideVerplichting::nieuw(
            id,
            Verplichtingcode::AVG_MEDEDELING,
            Ankertype::VaststellingHoogRisico,
            incident.risicoweging.as_ref().map(|m| m.op),
            "de weging kwam uit op een hoog risico voor de betrokkenen",
        ));
    }

    // --- 3. Vastlegging in het interne register ---
    // Geldt altijd, ook — juist — wanneer er niet wordt gemeld. Dat is de
    // vastlegging waarop een toezichthouder als eerste vraagt.
    uit.push(AfgeleideVerplichting::nieuw(
        id,
        Verplichtingcode::AVG_INTERN_REGISTER,
        Ankertype::Kennisname,
        incident.kennisname_op,
        if incident.meldbesluit.is_niet_melden() {
            "er wordt niet gemeld; de vastlegging in het interne register is dan de enige \
             verantwoording die overblijft"
        } else {
            "elke inbreuk wordt intern vastgelegd, ongeacht het meldbesluit"
        },
    ));

    // --- 4 tot en met 6. De zorgplichtketen ---
    if zorgplicht.valt_onder_meldketen && zorgplicht.is_significant {
        let significant_anker = incident.significant_vastgesteld_op.or(incident.kennisname_op);

        uit.push(AfgeleideVerplichting::nieuw(
            id,
            Verplichtingcode::ZORG_WAARSCHUWING,
            Ankertype::VaststellingSignificant,
            significant_anker,
            "het incident is als significant aangemerkt",
        ));
        uit.push(AfgeleideVerplichting::nieuw(
            id,
            Verplichtingcode::ZORG_MELDING,
            Ankertype::VaststellingSignificant,
            significant_anker,
            "het incident is als significant aangemerkt",
        ));

        // Randgeval T-05: het eindrapport hangt aan de verzending van de
        // melding, niet aan het incident. Zolang er niet is gemeld, is het
        // anker er nog niet — en dat hoort zichtbaar te zijn.
        uit.push(AfgeleideVerplichting::nieuw(
            id,
            Verplichtingcode::ZORG_EINDRAPPORT,
            Ankertype::VerzendingMelding,
            incident.gemeld_op,
            "het eindrapport loopt vanaf de verzending van de incidentmelding, \
             niet vanaf het incident zelf",
        ));
    }

    uit
}

/// Randgeval T-06: het incident duurt nog voort op de datum van het eindrapport.
///
/// Dan ontstaat een voortgangsrapport nú, en schuift het eindrapport op naar de
/// afronding. Deze functie levert de vervangende verplichtingen.
pub fn verplichtingen_bij_voortdurend_incident(
    incident: &Incident,
    nu: DateTime<Utc>,
) -> Vec<AfgeleideVerplichting> {
    if incident.afgehandeld_op.is_some() {
        return Vec::new();
    }
    vec![
        AfgeleideVerplichting::nieuw(
            incident.id,
            Verplichtingcode::ZORG_VOORTGANG,
            Ankertype::VerzendingMelding,
            Some(nu),
            "het incident duurt voort op het moment waarop het eindrapport verwacht werd",
        ),
        AfgeleideVerplichting::nieuw(
            incident.id,
            Verplichtingcode::ZORG_EINDRAPPORT,
            Ankertype::Afhandeling,
            None,
            "het eindrapport loopt opnieuw vanaf de afronding van de afhandeling",
        ),
    ]
}

/// Randgeval T-09: het ankertijdstip wordt gecorrigeerd.
///
/// Alle klokken die op dat ankertype rusten worden herberekend. De oude
/// waarden blijven zichtbaar en de correctie vereist bevestiging door een
/// tweede persoon — het is een handeling die vijf termijnen tegelijk verschuift.
pub fn getroffen_door_ankercorrectie(
    verplichtingen: &[AfgeleideVerplichting],
    ankertype: Ankertype,
) -> Vec<&AfgeleideVerplichting> {
    verplichtingen.iter().filter(|v| v.ankertype == ankertype).collect()
}

/// Of het meldbesluit de meldklok mag laten vervallen.
///
/// Nee, zolang het besluit niet definitief is: een besluit binnen de
/// afkoelperiode kan nog omslaan, en dan moet de oorspronkelijke termijn nog
/// haalbaar zijn. De klok verdwijnt pas als het besluit vaststaat.
pub fn meldklok_vervalt(besluit: &Meldbesluit, nu: DateTime<Utc>) -> bool {
    besluit.is_niet_melden() && besluit.is_definitief(nu)
}

/// Of de weging en het besluit met elkaar te rijmen zijn.
///
/// Controleregel LEK-08 op het niveau van het dossier.
pub fn besluit_past_bij_weging(incident: &Incident) -> bool {
    match (&incident.risiconiveau, &incident.meldbesluit) {
        (_, Meldbesluit::NogTeNemen) => true,
        (Some(Risiconiveau::GeenRisico), Meldbesluit::NietMelden { .. }) => true,
        (Some(Risiconiveau::GeenRisico), Meldbesluit::Melden { .. }) => true, // melden mag altijd
        (Some(_), Meldbesluit::Melden { .. }) => true,
        (Some(_), Meldbesluit::NietMelden { .. }) => false,
        (None, _) => false,
    }
}
