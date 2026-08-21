//! Eén controleronde over de hele gegevensverzameling.
//!
//! # Waarom dit hier staat en niet in de bedieningsschil
//!
//! De ronde bestond twee keer: de opdrachtregel draaide veertien
//! beoordelingsfuncties, de grafische schil twee. Dat verschil was aan geen van
//! beide kanten te zien. Een gebruiker die in het venster op "controleronde"
//! klikte, kreeg een scherm dat er compleet uitzag en dat ook zo werd gelezen —
//! terwijl leveranciers, risicobeoordelingen, incidenten, effectbeoordelingen,
//! doorgiften en correcties er niet in zaten. Een lijst die zwijgt over wat zij
//! niet heeft nagekeken, is erger dan geen lijst: de lege plek vraagt om werk,
//! het gevulde vakje sust.
//!
//! Het verschil was ook niet toevallig ontstaan. Elke regel die erbij kwam,
//! moest op twee plaatsen worden aangesloten, en de tweede plaats werd
//! vergeten. Zolang die twee plaatsen bestaan, gebeurt dat opnieuw. Daarom
//! staat de ronde hier, en roepen beide schillen dezelfde functie aan.
//!
//! # De drempels komen uit het kennispakket
//!
//! Geen van de drempels in [`Drempels`] staat in de wet. De verordening noemt
//! geen getal bij "incidenteel", geen termijn voor herbeoordeling van een
//! effectbeoordeling en geen aandeel waarboven afwijken gewoonte heet. Ze staan
//! daarom in het kennispakket, waar een jurist ze kan bijstellen zonder de
//! programmacode te raken.
//!
//! De schil had ze hardgecodeerd staan. Met het meegeleverde pakket kwam dat
//! toevallig op hetzelfde neer; bij het eerste bijgestelde pakket niet meer, en
//! dan voert het venster een tweede waarheid naast de opdrachtregel zonder dat
//! iemand het merkt.

use chrono::{DateTime, Duration, Utc};

use dpofg_audit::Verificatierapport;
use dpofg_domain::{
    correctie::Correctie, doorgifte::Doorgifte, leverancier::Leverancier,
    risico::Risicobeoordeling, zorgplicht::Zorgplichtdossier, Dpia, Incident, Verwerking,
};

use crate::budget::Waarschuwingsbudget;
use crate::motor::{Bevinding, Niveau, Regelmotor};
use crate::regels::{
    beoordeel_budget, beoordeel_correcties, beoordeel_doorgifte, beoordeel_dpia,
    beoordeel_incident, beoordeel_leverancier, beoordeel_logboek, beoordeel_meldtermijn,
    beoordeel_oorzaakpatroon, beoordeel_raadplegingstermijn, beoordeel_risicobeoordeling,
    beoordeel_verwerkersmelding, beoordeel_verwerking, beoordeel_zorgplicht, pas_correcties_toe,
    Zorgplichtdrempels,
};

/// Over hoeveel dagen terug het patroon over incidenten heen wordt geteld.
///
/// Een kwartaal. Dit getal staat hier en niet in het kennispakket omdat het
/// geen norm is maar de reikwijdte van een telling: het patroon dat ORG-06
/// zoekt — dezelfde oorzaak die blijft terugkomen — heeft een venster nodig
/// waarbinnen "blijft terugkomen" betekenis heeft.
const KWARTAAL_DAGEN: i64 = 92;

/// Over hoeveel dagen terug de onderbrekingen voor het waarschuwingsbudget
/// worden geteld.
const WEEK_DAGEN: i64 = 7;

/// De drempels van één ronde, afgeleid uit het kennispakket.
///
/// De terugvalwaarden staan erbij zodat een uitgekleed pakket de controle niet
/// stilzwijgend uitzet. Een drempel die ontbreekt, hoort de bewaking niet af te
/// zetten maar terug te vallen op de waarde waarmee het product is ontworpen.
#[derive(Debug, Clone, Copy)]
pub struct Drempels {
    /// Na hoeveel maanden een effectbeoordeling om herbeoordeling vraagt.
    pub herbeoordeling_maanden: i64,
    /// Boven hoeveel toepassingen per jaar een uitzondering van artikel 49 niet
    /// meer incidenteel is.
    pub uitzondering_per_jaar: u32,
    /// Boven hoeveel uur de contractuele meldtermijn van een verwerker te lang is.
    pub verwerker_meldtermijn_uren: u32,
    /// Boven welk aandeel afwijkingen niet-herstel gewoonte wordt.
    pub afwijkingsaandeel_procent: u32,
    /// Hoeveel dagen vooruit een verlopende risicobeoordeling wordt gemeld.
    pub beoordelingshorizon_dagen: i64,
    /// Na hoeveel maanden de subverwerkerslijst opnieuw moet worden nagelopen.
    pub subverwerkers_maanden: i64,
    /// De vijf drempels waartegen de zorgplichtcontrolset wordt gemeten.
    pub zorgplicht: Zorgplichtdrempels,
}

impl Drempels {
    /// Leidt de drempels af uit het kennispakket.
    pub fn uit_pakket(pakket: &dpofg_content::Pakketinhoud) -> Self {
        let dagen = |code: &str, terugval: i64| {
            pakket
                .termijn(code)
                .ok()
                .filter(|t| t.eenheid == dpofg_terms::Eenheid::Kalenderdagen)
                .map(|t| i64::from(t.duur))
                .unwrap_or(terugval)
        };
        let maanden = |code: &str, terugval: i64| {
            pakket
                .termijn(code)
                .ok()
                .filter(|t| t.eenheid == dpofg_terms::Eenheid::Maanden)
                .map(|t| i64::from(t.duur))
                .unwrap_or(terugval)
        };
        let getal = |groep: &str, sleutel: &str, terugval: u32| {
            pakket
                .aanvullend
                .get(groep)
                .and_then(|v| v.get(sleutel))
                .and_then(|v| v.as_u64())
                .and_then(|v| u32::try_from(v).ok())
                .unwrap_or(terugval)
        };

        Self {
            herbeoordeling_maanden: maanden("INTERN-DPIA-HERBEOORDELING", 36),
            uitzondering_per_jaar: getal("doorgifte_uitzonderingsdrempel", "drempel", 2),
            verwerker_meldtermijn_uren: getal("verwerker_meldtermijndrempel", "drempel_uren", 48),
            afwijkingsaandeel_procent: getal(
                "zorgplicht_drempels",
                "afwijkingsaandeel_procent",
                50,
            ),
            beoordelingshorizon_dagen: dagen("INTERN-RISICOBEOORDELING-HORIZON", 60),
            subverwerkers_maanden: maanden("INTERN-SUBVERWERKERSCONTROLE", 12),
            zorgplicht: Zorgplichtdrempels {
                beoordelingstermijn_dagen: dagen("INTERN-ZORGPLICHT-BEOORDELINGSTERMIJN", 30),
                bewijshorizon_dagen: dagen("INTERN-ZORGPLICHT-BEWIJSHORIZON", 60),
                frequentiedrempel_maanden: getal(
                    "zorgplicht_drempels",
                    "frequentiedrempel_maanden",
                    12,
                ),
                bestuursvaststelling_maanden: maanden("INTERN-ZORGPLICHT-BESTUURSVASTSTELLING", 12),
                afwijkingsaandeel_procent: getal(
                    "zorgplicht_drempels",
                    "afwijkingsaandeel_procent",
                    20,
                ),
            },
        }
    }
}

/// De records waarover één ronde oordeelt.
///
/// De aanroeper laadt ze uit de kluis en geeft ze hier door. Dat is hetzelfde
/// patroon als bij de werkbak en de prognose: de laag die oordeelt kent de
/// opslag niet, zodat zij te beproeven is zonder kluis.
#[derive(Debug, Default)]
pub struct Ronde<'a> {
    pub verwerkingen: &'a [Verwerking],
    pub effectbeoordelingen: &'a [Dpia],
    pub doorgiften: &'a [Doorgifte],
    pub risicobeoordelingen: &'a [Risicobeoordeling],
    pub zorgplichtdossiers: &'a [Zorgplichtdossier],
    pub leveranciers: &'a [Leverancier],
    pub incidenten: &'a [Incident],
    pub correcties: &'a [Correctie],
    /// De uitkomst van de ketencontrole over het logboek.
    ///
    /// Optioneel omdat een aanroeper die geen kluis heeft — een test, een
    /// weergave die alleen over dossiers gaat — hem niet kan leveren. Wat er
    /// niet is, wordt niet beoordeeld; dat staat in [`Uitslag::niet_nagekeken`].
    pub logboek: Option<&'a Verificatierapport>,
    /// Het tijdstip van het laatste anker, nodig naast het rapport.
    ///
    /// `None` betekent hier iets anders dan bij [`Ronde::logboek`]: niet "niet
    /// nagekeken", maar "er is nog geen anker gezet". Dat onderscheid is het
    /// hele punt van regel SYS-06 — een keten zonder anker is niet gaaf maar
    /// onbewijsbaar, en dat hoort gemeld te worden in plaats van verzwegen.
    pub laatste_anker: Option<DateTime<Utc>>,
    /// Het waarschuwingsbudget, gevoed uit het logboek.
    pub budget: Option<&'a Waarschuwingsbudget>,
}

/// Wat één ronde heeft opgeleverd.
#[derive(Debug, Default)]
pub struct Uitslag {
    pub bevindingen: Vec<Bevinding>,
    /// Hoeveel dossiers er zijn nagekeken.
    pub beoordeeld: usize,
    /// Dossiers waarvan een termijn niet te berekenen was.
    ///
    /// Uitdrukkelijk niet stilzwijgend overgeslagen: een termijn die niet te
    /// berekenen is, is iets anders dan een termijn die in orde is. Zulke
    /// dossiers tellen ook niet als beoordeeld.
    pub onberekenbaar: Vec<String>,
    /// Onderdelen die deze ronde niet heeft nagekeken omdat de aanroeper ze
    /// niet heeft aangeleverd.
    ///
    /// Zonder deze lijst zou een ronde zonder logboek er hetzelfde uitzien als
    /// een ronde met een gaaf logboek.
    pub niet_nagekeken: Vec<String>,
}

/// Draait één controleronde.
///
/// Het peilmoment komt van buiten: de hele ronde rekent met één `nu`, zodat
/// twee regels nooit op verschillende klokken berusten.
pub fn beoordeel_ronde(
    motor: &Regelmotor,
    ronde: &Ronde<'_>,
    pakket: &dpofg_content::Pakketinhoud,
    drempels: &Drempels,
    nu: DateTime<Utc>,
) -> Uitslag {
    let mut uit = Uitslag::default();

    for v in ronde.verwerkingen {
        uit.bevindingen.extend(beoordeel_verwerking(motor, v, nu));
        uit.beoordeeld += 1;
    }

    for d in ronde.effectbeoordelingen {
        uit.bevindingen.extend(beoordeel_dpia(motor, d, drempels.herbeoordeling_maanden, nu));
        match raadplegingstermijn_van(pakket, d, nu) {
            Ok(Some(deadline)) => {
                uit.bevindingen.extend(beoordeel_raadplegingstermijn(motor, d, &deadline, nu));
                uit.beoordeeld += 1;
            }
            Ok(None) => uit.beoordeeld += 1,
            Err(fout) => uit.onberekenbaar.push(format!("{}: {fout}", d.kenmerk)),
        }
    }

    for d in ronde.doorgiften {
        uit.bevindingen.extend(beoordeel_doorgifte(motor, d, drempels.uitzondering_per_jaar, nu));
        uit.beoordeeld += 1;
    }

    for b in ronde.risicobeoordelingen {
        uit.bevindingen.extend(beoordeel_risicobeoordeling(
            motor,
            b,
            drempels.beoordelingshorizon_dagen,
            nu,
        ));
        uit.beoordeeld += 1;
    }

    // Regel ZRP-11 kijkt of de gekoppelde beoordeling bestaat, is vastgesteld
    // en nog geldt, en dat weet alleen het beoordelingsdossier zelf. Vandaar de
    // hele lijst en niet één dossier.
    for d in ronde.zorgplichtdossiers {
        uit.bevindingen.extend(beoordeel_zorgplicht(
            motor,
            d,
            ronde.risicobeoordelingen,
            drempels.zorgplicht,
            nu,
        ));
        uit.beoordeeld += 1;
    }

    for l in ronde.leveranciers {
        uit.bevindingen.extend(beoordeel_leverancier(
            motor,
            l,
            drempels.verwerker_meldtermijn_uren,
            drempels.subverwerkers_maanden,
            nu,
        ));
        uit.beoordeeld += 1;
    }

    for i in ronde.incidenten {
        uit.bevindingen.extend(beoordeel_incident(motor, i, nu));
        match meldtermijn_van(pakket, i) {
            Ok(Some(deadline)) => {
                uit.bevindingen.extend(beoordeel_meldtermijn(motor, i, &deadline, nu));
                uit.beoordeeld += 1;
            }
            Ok(None) => uit.beoordeeld += 1,
            Err(fout) => uit.onberekenbaar.push(format!("{}: {fout}", i.kenmerk)),
        }
        // De verwerker meldt aan de verantwoordelijke binnen zijn contractuele
        // termijn; dat is alleen na te rekenen als die leverancier bekend is.
        if let Some(verwerker_id) = i.verwerker_id {
            if let Some(l) = ronde.leveranciers.iter().find(|l| l.id == verwerker_id) {
                uit.bevindingen.extend(beoordeel_verwerkersmelding(motor, i, l, nu));
            }
        }
    }

    // Het patroon over incidenten heen: een kwartaal terug.
    let kwartaalgrens = nu - Duration::days(KWARTAAL_DAGEN);
    uit.bevindingen.extend(beoordeel_oorzaakpatroon(motor, ronde.incidenten, nu, kwartaalgrens));

    // Het systeem onder de dossiers: de keten en de klok.
    match ronde.logboek {
        Some(rapport) => {
            uit.bevindingen.extend(beoordeel_logboek(motor, rapport, ronde.laatste_anker, nu));
        }
        None => uit.niet_nagekeken.push("het ketenlogboek".into()),
    }

    match ronde.budget {
        Some(budget) => uit.bevindingen.extend(beoordeel_budget(motor, budget, nu)),
        None => uit.niet_nagekeken.push("het waarschuwingsbudget".into()),
    }

    // De correcties komen ná alle beoordelingen: zij gaan over de bevindingen
    // van deze ronde en niet over een dossier. Eerst wordt de correctieplicht
    // getoetst tegen het onbewerkte beeld — anders zou een lopende afwijking de
    // bevinding waarover zij gaat uit het zicht halen en zou COR-03 elke
    // afwijking als overbodig melden. Daarna pas worden de afwijkingen over de
    // bevindingen heen gelegd.
    if !ronde.correcties.is_empty()
        || uit.bevindingen.iter().any(|b| b.niveau == Niveau::Blokkerend)
    {
        let over_correcties = beoordeel_correcties(
            motor,
            ronde.correcties,
            &uit.bevindingen,
            drempels.afwijkingsaandeel_procent,
            nu,
        );
        pas_correcties_toe(&mut uit.bevindingen, ronde.correcties, nu);
        uit.bevindingen.extend(over_correcties);
        uit.beoordeeld += ronde.correcties.len();
    }

    uit
}

/// Het venster waarbinnen onderbrekingen voor het waarschuwingsbudget tellen.
///
/// Staat hier zodat beide schillen dezelfde grens hanteren bij het vullen van
/// het budget uit het logboek.
pub fn budgetvenster(nu: DateTime<Utc>) -> DateTime<Utc> {
    nu - Duration::days(WEEK_DAGEN)
}

/// De meldtermijn van één incident, of `None` wanneer de klok nog niet loopt.
///
/// Het anker is het moment van kennisname; zolang dat er niet is, is er niets te
/// rekenen en dus niets waaraan te herinneren valt. Een `Err` betekent iets
/// anders: er is wél een termijn maar hij is niet te berekenen, en dat mag niet
/// als "in orde" worden gelezen.
fn meldtermijn_van(
    pakket: &dpofg_content::Pakketinhoud,
    i: &Incident,
) -> Result<Option<dpofg_terms::Deadline>, String> {
    let Some(anker) = i.anker_meldklok() else { return Ok(None) };
    let soort = pakket.termijn("AVG-33-MELDING").map_err(|e| e.to_string())?;
    let kalender = pakket.kalender("NL").map_err(|e| e.to_string())?;
    let zone = dpofg_terms::tijdzone(dpofg_terms::TIJDZONE_NL).map_err(|e| e.to_string())?;
    Ok(Some(dpofg_terms::bereken(soort, anker, zone, kalender).map_err(|e| e.to_string())?))
}

/// De lopende raadplegingstermijn van één effectbeoordeling.
///
/// `Ok(None)` betekent: er loopt geen raadpleging, dus er valt niets te
/// beoordelen. Een `Err` betekent dat de termijn bestaat maar niet te berekenen
/// is. Vandaar geen `.ok()` op de berekening zelf.
fn raadplegingstermijn_van(
    pakket: &dpofg_content::Pakketinhoud,
    d: &Dpia,
    nu: DateTime<Utc>,
) -> Result<Option<dpofg_terms::Deadline>, String> {
    let Some(klok) = d.raadpleging.as_ref() else { return Ok(None) };
    let kalender = pakket.kalender("NL").map_err(|e| e.to_string())?;
    let zone = dpofg_terms::tijdzone(dpofg_terms::TIJDZONE_NL).map_err(|e| e.to_string())?;
    Ok(Some(klok.deadline_volledig(nu, zone, kalender).map_err(|e| e.to_string())?))
}
