//! De regelcatalogus.
//!
//! # Wanneer een regel blokkeert
//!
//! Blokkeren mag alleen bij een **objectief bepaalbaar feit dat tot een
//! overtreding leidt en dat de gebruiker zelf kan wegnemen**. Alle drie de
//! voorwaarden tellen:
//!
//! * *Objectief bepaalbaar* — geen oordeel. "Twee criteria geraakt, dus een
//!   effectbeoordeling verplicht" is een oordeel; "een verwerker zonder
//!   overeenkomst" is een feit.
//! * *Leidt tot een overtreding* — niet: is onhandig, of ziet er onaf uit.
//! * *De gebruiker kan het wegnemen* — een ingetrokken adequaatheidsbesluit
//!   kan hij niet ongedaan maken. Daar blokkeren betekent iemand straffen voor
//!   iets buiten zijn macht, en dat leert wegklikken.
//!
//! Bij twijfel: signalerend. Een regel kan altijd worden aangescherpt; een
//! gebruiker die heeft geleerd meldingen weg te klikken, leert dat niet af.
//!
//! De definities hieronder volgen paragraaf 5.2 van het
//! foutbestendigheidshoofdstuk. Niet elke regel heeft in deze uitgave al een
//! evaluatiefunctie: de regels waarvoor het onderliggende record nog niet
//! bestaat, staan wél in de catalogus maar draaien nog niet. Dat is expliciet
//! zichtbaar via [`Regelmotor::regels_zonder_evaluatie`], zodat het aantal
//! regels in de catalogus geen dekking suggereert die er niet is.

use chrono::{DateTime, Duration, Utc};
use dpofg_audit::{Ankerstatus, Bevindingsoort, Verificatierapport};
use dpofg_domain::{
    avg::Grondslag, Bewaartermijn, Incident, Meldbesluit, Risiconiveau, Status, Verwerking,
    Volledig,
};
use dpofg_terms::Deadline;

use crate::budget::Waarschuwingsbudget;

use crate::motor::{Bevinding, Niveau::*, Ontvangerrol::*, Regel, Regelmotor};

/// De volledige catalogus.
pub fn catalogus() -> Vec<Regel> {
    let mut uit = Vec::new();
    uit.extend(register());
    uit.extend(grondslag());
    uit.extend(bewaartermijn());
    uit.extend(verwerkers());
    uit.extend(doorgifte());
    uit.extend(datalekken());
    uit.extend(effectbeoordeling());
    uit.extend(organisatie());
    uit.extend(systeem());
    uit
}

/// Een motor met de volledige catalogus.
pub fn standaardmotor() -> Regelmotor {
    let mut motor = Regelmotor::nieuw();
    motor.registreer_alle(catalogus());
    motor
}

/// De codes waarvoor in deze uitgave een evaluatiefunctie bestaat.
///
/// Deze lijst is de bron van `dpofg controle --dekking`, en dus de plaats waar
/// het product zegt wat het bewaakt. Twee kanten daarvan worden bewaakt door
/// tests: geen code die hier staat mag ontbreken in de catalogus, en geen code
/// die een evaluatiefunctie kan afgeven mag hier ontbreken. Die tweede richting
/// is niet theoretisch — GRO-04 en GRO-05 draaiden en werden niet gemeld.
///
/// Een code hoort hier pas thuis wanneer de gegevens waarop hij oordeelt ook
/// daadwerkelijk in te vullen zijn. Bewaking certificeren die op producteigen
/// gegevens nooit kan aanslaan, is erger dan een lege plek: het lege vakje
/// vraagt om werk, het gevulde vakje sust.
pub fn geimplementeerd() -> &'static [&'static str] {
    &[
        "REG-01", "REG-02", "REG-03", "REG-04", "REG-05", "GRO-01", "GRO-02", "GRO-03", "GRO-04",
        "GRO-05", "BEW-01", "BEW-02", "BEW-04", "VWO-01", "EER-01", "DPIA-01", "LEK-01", "LEK-02",
        "LEK-03", "LEK-04", "LEK-06", "LEK-07", "LEK-08", "LEK-09", "LEK-12", "LEK-13", "LEK-15",
        "SYS-04", "SYS-06", "SYS-10",
    ]
}

// --------------------------------------------------------------------------
// De definities
// --------------------------------------------------------------------------

fn register() -> Vec<Regel> {
    vec![
        // Signalerend en niet blokkerend: de regel is al vastgesteld; blokkeren heeft geen effect meer en zou het hele register tegenhouden.
        Regel::nieuw(
            "REG-01",
            "register",
            "Registerregel onvolledig",
            "een vastgestelde registerregel mist een van de onderdelen van artikel 30",
            Signalerend,
            Functionaris,
            "art. 30 AVG",
            false,
        ),
        Regel::nieuw(
            "REG-02",
            "register",
            "Register niet herzien",
            "een vastgestelde registerregel is langer dan twaalf maanden niet herzien",
            Signalerend,
            Functionaris,
            "art. 5 lid 2 AVG; interne norm",
            true,
        ),
        Regel::nieuw(
            "REG-03",
            "register",
            "Geërfd en niet geverifieerd",
            "een overgenomen registerregel is nog niet tegen de bron gecontroleerd",
            Signalerend,
            Functionaris,
            "art. 5 lid 2 AVG",
            true,
        ),
        Regel::nieuw(
            "REG-04",
            "register",
            "Concept blijft liggen",
            "een registerregel staat langer dan negentig dagen op concept",
            Signalerend,
            Functionaris,
            "interne norm",
            true,
        ),
        Regel::nieuw(
            "REG-05",
            "register",
            "Registerregel zonder eigenaar",
            "een registerregel heeft geen eigenaar buiten de functionaris",
            Signalerend,
            Directie,
            "art. 24 AVG",
            true,
        ),
        Regel::nieuw(
            "REG-06",
            "register",
            "Systeem zonder registerregel",
            "een systeem met persoonsgegevens komt in geen enkele registerregel voor",
            Signalerend,
            Functionaris,
            "art. 30 AVG",
            true,
        ),
        Regel::nieuw(
            "REG-07",
            "register",
            "Doel te ruim omschreven",
            "een doelomschrijving bevat uitsluitend algemene termen",
            Rapporterend,
            Functionaris,
            "art. 5 lid 1 onder b AVG",
            true,
        ),
    ]
}

fn grondslag() -> Vec<Regel> {
    vec![
        Regel::nieuw(
            "GRO-01",
            "grondslag",
            "Gerechtvaardigd belang zonder afweging",
            "grondslag f zonder vastgelegde belangenafweging",
            Blokkerend,
            Functionaris,
            "art. 6 lid 1 onder f AVG",
            false,
        ),
        Regel::nieuw(
            "GRO-02",
            "grondslag",
            "Toestemming zonder bewijs",
            "grondslag a zonder vastgelegde bewijsvorm en intrekkingsroute",
            Blokkerend,
            Functionaris,
            "art. 7 lid 1 en lid 3 AVG",
            false,
        ),
        Regel::nieuw(
            "GRO-03",
            "grondslag",
            "Bijzondere gegevens zonder uitzondering",
            "bijzondere categorieën zonder uitzonderingsgrond",
            Blokkerend,
            Functionaris,
            "art. 9 lid 2 AVG",
            false,
        ),
        Regel::nieuw(
            "GRO-04",
            "grondslag",
            "Wettelijke grondslag niet aangewezen",
            "grondslag c of e zonder aanwijsbare bepaling",
            Blokkerend,
            Functionaris,
            "art. 6 lid 3 AVG",
            false,
        ),
        Regel::nieuw(
            "GRO-05",
            "grondslag",
            "Burgerservicenummer zonder grondslag",
            "gebruik van het burgerservicenummer zonder wettelijke bepaling",
            Blokkerend,
            Functionaris,
            "art. 87 AVG en art. 46 UAVG",
            false,
        ),
    ]
}

fn bewaartermijn() -> Vec<Regel> {
    vec![
        Regel::nieuw(
            "BEW-01",
            "bewaartermijn",
            "Bewaartermijn ontbreekt",
            "een vastgestelde registerregel zonder bewaartermijn",
            Blokkerend,
            Functionaris,
            "art. 30 lid 1 onder f AVG",
            false,
        ),
        Regel::nieuw(
            "BEW-02",
            "bewaartermijn",
            "Uitstelafspraak verlopen",
            "een uitgestelde bewaartermijn is niet vastgesteld binnen de afgesproken datum",
            Signalerend,
            Functionaris,
            "art. 5 lid 1 onder e AVG",
            true,
        ),
        Regel::nieuw(
            "BEW-03",
            "bewaartermijn",
            "Schoningsopdracht niet uitgevoerd",
            "een schoningsopdracht is niet uitgevoerd binnen veertien dagen na de uitvoerdatum",
            Signalerend,
            Systeemeigenaar,
            "art. 5 lid 1 onder e AVG",
            true,
        ),
        Regel::nieuw(
            "BEW-04",
            "bewaartermijn",
            "Bewaartermijn zonder grondslag",
            "een bewaartermijn zonder aanwijsbare bron",
            Signalerend,
            Functionaris,
            "art. 5 lid 1 onder e AVG",
            true,
        ),
    ]
}

fn verwerkers() -> Vec<Regel> {
    vec![
        Regel::nieuw(
            "VWO-01",
            "verwerkers",
            "Verwerker zonder overeenkomst",
            "een verwerking met een verwerker zonder actieve verwerkersovereenkomst",
            Blokkerend,
            Functionaris,
            "art. 28 lid 3 AVG",
            false,
        ),
        // Signalerend en niet blokkerend: een contract in kaart brengen is werk in uitvoering, geen overtreding.
        Regel::nieuw(
            "VWO-02",
            "verwerkers",
            "Contracteisen niet gemapt",
            "een of meer eisen van artikel 28 lid 3 zonder vindplaats in het contract",
            Signalerend,
            Contracteigenaar,
            "art. 28 lid 3 AVG",
            false,
        ),
        Regel::nieuw(
            "VWO-04",
            "verwerkers",
            "Meldtermijn verwerker te lang",
            "de contractuele meldtermijn van de verwerker is langer dan achtenveertig uur",
            Signalerend,
            Functionaris,
            "art. 33 lid 2 AVG",
            true,
        ),
        Regel::nieuw(
            "VWO-09",
            "verwerkers",
            "Subverwerkerslijst niet gecontroleerd",
            "de laatste controle van de subverwerkerslijst is ouder dan twaalf maanden",
            Signalerend,
            Contracteigenaar,
            "art. 28 lid 2 en lid 4 AVG",
            true,
        ),
        Regel::nieuw(
            "VWO-13",
            "verwerkers",
            "Contract getekend na aanvang",
            "de ondertekendatum ligt na de startdatum van de verwerking",
            Rapporterend,
            Functionaris,
            "art. 28 lid 3 AVG",
            false,
        ),
    ]
}

fn doorgifte() -> Vec<Regel> {
    vec![
        Regel::nieuw(
            "EER-01",
            "doorgifte",
            "Doorgifte zonder waarborg",
            "een ontvanger buiten de EER zonder vastgelegd instrument",
            Blokkerend,
            Functionaris,
            "hoofdstuk V AVG",
            false,
        ),
        Regel::nieuw(
            "EER-02",
            "doorgifte",
            "Toegangsland onbekend",
            "alleen de opslaglocatie is ingevuld, niet vanwaaruit toegang bestaat",
            Blokkerend,
            Systeemeigenaar,
            "hoofdstuk V AVG",
            false,
        ),
        // Signalerend en niet blokkerend: of een beoordeling toereikend is, is een oordeel.
        Regel::nieuw(
            "EER-03",
            "doorgifte",
            "Modelbepalingen zonder beoordeling",
            "modelcontractbepalingen zonder afgeronde doorgiftebeoordeling",
            Signalerend,
            Functionaris,
            "art. 46 AVG",
            false,
        ),
        Regel::nieuw(
            "EER-06",
            "doorgifte",
            "Uitzondering structureel gebruikt",
            "een uitzondering van artikel 49 wordt vaker dan tweemaal per jaar toegepast",
            Signalerend,
            Functionaris,
            "art. 49 AVG",
            true,
        ),
        // Signalerend en niet blokkerend: de intrekking van een instrument ligt buiten de macht van de gebruiker.
        Regel::nieuw(
            "EER-07",
            "doorgifte",
            "Waarborg vervallen",
            "het instrument waarop deze doorgifte berust is ingetrokken of onder toetsing",
            Signalerend,
            Functionaris,
            "hoofdstuk V AVG",
            false,
        ),
    ]
}

fn effectbeoordeling() -> Vec<Regel> {
    vec![
        Regel::nieuw("DPIA-01", "effectbeoordeling", "Criteria bereikt zonder beoordeling",
            "twee of meer criteria geraakt zonder uitgevoerde effectbeoordeling of gemotiveerd besluit",
            Signalerend, Functionaris, "art. 35 lid 1 AVG", true),
        Regel::nieuw("DPIA-03", "effectbeoordeling", "Beoordeling na aanvang",
            "de effectbeoordeling is uitgevoerd nadat de verwerking al liep",
            Signalerend, Functionaris, "art. 35 lid 1 AVG", false),
        // Signalerend en niet blokkerend: of een restrisico hoog is en of raadpleging nodig is, is een oordeel.
        Regel::nieuw("DPIA-06", "effectbeoordeling", "Hoog restrisico zonder raadpleging",
            "een hoog restrisico zonder voorafgaande raadpleging",
            Signalerend, Functionaris, "art. 36 lid 1 AVG", false),
        Regel::nieuw("DPIA-07", "effectbeoordeling", "Beoordeling verouderd",
            "de effectbeoordeling is ouder dan zesendertig maanden of de verwerking is gewijzigd",
            Signalerend, Functionaris, "art. 35 lid 11 AVG", true),
    ]
}

fn datalekken() -> Vec<Regel> {
    vec![
        Regel::nieuw(
            "LEK-01",
            "datalekken",
            "Geen beoordeling binnen twaalf uur",
            "een geregistreerd incident zonder risicobeoordeling na twaalf uur",
            Signalerend,
            Behandelaar,
            "art. 33 lid 1 AVG",
            false,
        ),
        Regel::nieuw(
            "LEK-02",
            "datalekken",
            "Termijn nadert zonder besluit",
            "minder dan twaalf uur tot de meldtermijn zonder genomen besluit",
            Signalerend,
            Functionaris,
            "art. 33 lid 1 AVG",
            false,
        ),
        Regel::nieuw(
            "LEK-03",
            "datalekken",
            "Gat tussen kennisname en registratie",
            "meer dan vier uur tussen kennisname en registratie zonder toelichting",
            Blokkerend,
            Functionaris,
            "art. 33 lid 1 en lid 5 AVG",
            true,
        ),
        Regel::nieuw(
            "LEK-04",
            "datalekken",
            "Niet melden zonder tweede laag",
            "een besluit om niet te melden zonder tweede persoon en zonder afkoelperiode",
            Blokkerend,
            Functionaris,
            "art. 33 lid 1 AVG",
            false,
        ),
        Regel::nieuw(
            "LEK-06",
            "datalekken",
            "Geen risico bij grote omvang",
            "uitkomst geen risico bij meer dan tweehonderdvijftig betrokkenen",
            Signalerend,
            Functionaris,
            "art. 33 lid 1 AVG",
            true,
        ),
        Regel::nieuw(
            "LEK-07",
            "datalekken",
            "Geen risico bij gevoelige gegevens",
            "uitkomst geen risico terwijl bijzondere gegevens, een burgerservicenummer of \
             financiële gegevens betrokken zijn",
            Blokkerend,
            Functionaris,
            "art. 33 lid 1 jo. art. 9 AVG",
            false,
        ),
        Regel::nieuw(
            "LEK-08",
            "datalekken",
            "Besluit spreekt de weging tegen",
            "een risico vastgesteld en toch besloten niet te melden",
            Blokkerend,
            Functionaris,
            "art. 33 lid 1 AVG",
            false,
        ),
        Regel::nieuw(
            "LEK-09",
            "datalekken",
            "Beschikbaarheid niet beoordeeld",
            "geen van de drie aspecten van de inbreuk is beantwoord",
            Blokkerend,
            Behandelaar,
            "art. 4 onder 12 AVG",
            false,
        ),
        Regel::nieuw(
            "LEK-12",
            "datalekken",
            "Afgesloten zonder oorzaak of maatregel",
            "een afgesloten incident zonder oorzaakcategorie of zonder maatregel",
            Blokkerend,
            Functionaris,
            "art. 33 lid 5 AVG",
            false,
        ),
        Regel::nieuw(
            "LEK-13",
            "datalekken",
            "Herhaalde oorzaak",
            "dezelfde oorzaakcategorie meer dan driemaal per kwartaal",
            Rapporterend,
            Directie,
            "art. 32 AVG",
            false,
        ),
        Regel::nieuw(
            "LEK-15",
            "datalekken",
            "Lek zonder registerkoppeling",
            "een incident dat aan geen enkele verwerking is gekoppeld",
            Signalerend,
            Functionaris,
            "art. 30 en art. 33 AVG",
            true,
        ),
        Regel::nieuw(
            "LEK-16",
            "datalekken",
            "Verwerker meldde te laat",
            "de verwerker overschreed de contractuele meldtermijn",
            Rapporterend,
            Contracteigenaar,
            "art. 33 lid 2 AVG",
            false,
        ),
    ]
}

fn organisatie() -> Vec<Regel> {
    vec![
        // Signalerend en niet blokkerend: publicatie loopt vaak via een andere afdeling.
        Regel::nieuw(
            "ORG-02",
            "organisatie",
            "Contactgegevens verouderd",
            "de publicatie van de contactgegevens is ouder dan de rolwisseling",
            Signalerend,
            Functionaris,
            "art. 37 lid 7 AVG",
            true,
        ),
        Regel::nieuw(
            "ORG-03",
            "organisatie",
            "Geen plaatsvervanger",
            "de rol heeft geen vastgelegde achtervang",
            Signalerend,
            Directie,
            "art. 38 lid 2 AVG",
            true,
        ),
        // Signalerend en niet blokkerend: een rolconflict lost de functionaris niet zelf op; dit is een bestuursbesluit.
        Regel::nieuw(
            "ORG-04",
            "organisatie",
            "Rolconflict",
            "de functionaris combineert een functie uit de conflictlijst",
            Signalerend,
            Directie,
            "art. 38 lid 6 AVG",
            true,
        ),
        Regel::nieuw(
            "ORG-05",
            "organisatie",
            "Geen rapportage aan de leiding",
            "geen rapportage aan het hoogste leidinggevende niveau in twaalf maanden",
            Signalerend,
            Directie,
            "art. 38 lid 3 AVG",
            true,
        ),
        Regel::nieuw(
            "ORG-07",
            "organisatie",
            "Termijn valt in een afwezigheid",
            "een dossier met een termijn in een vastgelegde afwezigheidsperiode zonder overdracht",
            Blokkerend,
            Functionaris,
            "art. 12 lid 3 AVG",
            false,
        ),
        Regel::nieuw(
            "ORG-08",
            "organisatie",
            "Overdracht zonder terugleeslus",
            "een dossier is overgedragen zonder bevestiging door de ontvanger",
            Blokkerend,
            Behandelaar,
            "interne norm",
            false,
        ),
        // Signalerend en niet blokkerend: een toezegging aanvullen is werk, geen overtreding.
        Regel::nieuw(
            "ORG-12",
            "organisatie",
            "Toezegging zonder eigenaar",
            "een toezegging aan de toezichthouder zonder eigenaar of einddatum",
            Signalerend,
            Functionaris,
            "art. 31 AVG",
            false,
        ),
    ]
}

fn systeem() -> Vec<Regel> {
    vec![
        Regel::nieuw(
            "SYS-04",
            "toepassing",
            "Ketenbreuk in het logboek",
            "de integriteitscontrole van het logboek faalt",
            Blokkerend,
            Beheerder,
            "art. 5 lid 2 AVG",
            false,
        ),
        Regel::nieuw(
            "SYS-05",
            "toepassing",
            "Signaalregel wordt genegeerd",
            "een signalerende regel wordt in een kwartaal meer dan tachtig procent genegeerd",
            Rapporterend,
            Beheerder,
            "interne norm",
            false,
        ),
        Regel::nieuw(
            "SYS-06",
            "toepassing",
            "Waarschuwingsbudget overschreden",
            "meer dan vijf onderbrekende meldingen per gebruiker per week",
            Rapporterend,
            Beheerder,
            "interne norm; behandeld als ontwerpdefect",
            false,
        ),
        Regel::nieuw(
            "SYS-08",
            "toepassing",
            "Onafgeronde onomkeerbare handeling",
            "een onomkeerbare handeling is voorbereid maar niet bevestigd of teruggedraaid",
            Blokkerend,
            Behandelaar,
            "interne norm",
            false,
        ),
        Regel::nieuw(
            "SYS-09",
            "toepassing",
            "Termijnmodule niet geverifieerd",
            "de testgevallen van de termijnmodule zijn na een bijwerking niet groen",
            Blokkerend,
            Beheerder,
            "interne norm",
            false,
        ),
        Regel::nieuw(
            "SYS-10",
            "toepassing",
            "Klokafwijking",
            "de systeemklok wijkt af van de monotone referentie",
            Signalerend,
            Beheerder,
            "art. 5 lid 2 AVG",
            false,
        ),
    ]
}

// --------------------------------------------------------------------------
// De evaluatie
// --------------------------------------------------------------------------

/// Beoordeelt één verwerking tegen de regels die daarop van toepassing zijn.
pub fn beoordeel_verwerking(
    motor: &Regelmotor,
    v: &Verwerking,
    nu: DateTime<Utc>,
) -> Vec<Bevinding> {
    let mut uit = Vec::new();
    let kenmerk = Some(v.kenmerk.as_str());
    let id = v.id.to_string();

    let mut voeg = |code: &str, toelichting: String| {
        if let Some(b) = motor.bevind(code, "verwerking", &id, kenmerk, toelichting, nu) {
            uit.push(b);
        }
    };

    let volledigheid = v.volledigheid();

    // REG-01: een vastgestelde regel die niet volledig is.
    if v.status.is_actief() && !volledigheid.is_volledig() {
        voeg(
            "REG-01",
            format!(
                "vastgesteld, maar {} ontbreken nog: {}",
                volledigheid.ontbreekt.len(),
                volledigheid
                    .ontbreekt
                    .iter()
                    .map(|o| o.veld.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        );
    }

    // REG-02: langer dan twaalf maanden niet herzien.
    if v.status == Status::Vastgesteld {
        if let Some(maanden) = v.herkomst.maanden_sinds_herziening(nu) {
            if maanden >= 12 {
                voeg("REG-02", format!("laatst herzien {maanden} maanden geleden"));
            }
        }
    }

    // REG-03: overgenomen en niet geverifieerd.
    if let Some(o) = &v.overgenomen {
        if !o.is_geverifieerd() {
            voeg("REG-03", format!("overgenomen uit {} en nog niet geverifieerd", o.bron));
        }
    }

    // REG-04: concept dat blijft liggen.
    if v.status == Status::Concept && (nu - v.herkomst.aangemaakt_op) > Duration::days(90) {
        voeg(
            "REG-04",
            format!("staat {} dagen op concept", (nu - v.herkomst.aangemaakt_op).num_days()),
        );
    }

    // REG-05: geen eigenaar buiten de functionaris.
    if v.eigenaar.trim().is_empty() {
        voeg("REG-05", "er is geen eigenaar aangewezen".into());
    }

    // GRO-01 tot en met GRO-03 en GRO-05 volgen uit de volledigheidscontrole,
    // maar krijgen hier een eigen code zodat ze apart telbaar zijn in de
    // rapportage — daar draait de gap-analyse op.
    for o in &volledigheid.ontbreekt {
        match o.veld.as_str() {
            "verwerking.belangenafweging" => voeg("GRO-01", o.omschrijving.clone()),
            "verwerking.toestemming" => voeg("GRO-02", o.omschrijving.clone()),
            "verwerking.uitzondering_artikel9" => voeg("GRO-03", o.omschrijving.clone()),
            "verwerking.bsn_grondslag" => voeg("GRO-05", o.omschrijving.clone()),
            "verwerking.wettelijke_bepaling" => voeg("GRO-04", o.omschrijving.clone()),
            "verwerking.bewaartermijn" => voeg("BEW-01", o.omschrijving.clone()),
            "verwerking.verwerkersovereenkomsten" => voeg("VWO-01", o.omschrijving.clone()),
            "verwerking.doorgiften" => voeg("EER-01", o.omschrijving.clone()),
            "verwerking.dpia" => voeg("DPIA-01", o.omschrijving.clone()),
            _ => {}
        }
    }

    // BEW-02: uitstelafspraak verlopen.
    if let Some(b) = &v.bewaartermijn {
        if b.uitstel_verlopen(nu) {
            if let Bewaartermijn::NogTeBepalen { uiterlijk_bepaald_op, eigenaar, .. } = b {
                voeg(
                    "BEW-02",
                    format!(
                        "de bewaartermijn zou uiterlijk {} zijn vastgesteld door {eigenaar}",
                        uiterlijk_bepaald_op.format("%d-%m-%Y")
                    ),
                );
            }
        }
    }

    // BEW-04: een vastgestelde bewaartermijn zonder aanwijsbare bron.
    //
    // Dit gat kent de volledigheidscontrole niet: die kijkt of er een termijn
    // ís, niet of er iets onder ligt. "twee jaar" zonder bron is een getal dat
    // niemand kan verdedigen bij een uitvraag. Wat de bron inhoudelijk waard
    // is, wordt hier niet beoordeeld — dat is een oordeel, geen feit.
    match &v.bewaartermijn {
        Some(Bewaartermijn::Vast { duur, eenheid, grondslag, .. })
            if grondslag.trim().is_empty() =>
        {
            let eenheidstekst = if *duur == 1 { eenheid.enkelvoud() } else { eenheid.meervoud() };
            voeg(
                "BEW-04",
                format!("bewaartermijn van {duur} {eenheidstekst} zonder aanwijsbare bron"),
            );
        }
        Some(Bewaartermijn::ZolangToestand { toestand, grondslag, .. })
            if grondslag.trim().is_empty() =>
        {
            voeg("BEW-04", format!("bewaren zolang '{toestand}' duurt, zonder aanwijsbare bron"));
        }
        // NogTeBepalen heeft geen grondslagveld en is al gedekt door BEW-01 en
        // BEW-02; twee keer melden op hetzelfde gat is dubbele ruis.
        _ => {}
    }

    uit
}

/// Beoordeelt één incident.
pub fn beoordeel_incident(motor: &Regelmotor, i: &Incident, nu: DateTime<Utc>) -> Vec<Bevinding> {
    let mut uit = Vec::new();
    let kenmerk = Some(i.kenmerk.as_str());
    let id = i.id.to_string();

    let mut voeg = |code: &str, toelichting: String| {
        if let Some(b) = motor.bevind(code, "incident", &id, kenmerk, toelichting, nu) {
            uit.push(b);
        }
    };

    // LEK-09: geen enkel aspect beoordeeld.
    if !i.aantasting.is_aangetast() {
        voeg(
            "LEK-09",
            "vertrouwelijkheid, integriteit en beschikbaarheid zijn geen van drieën beantwoord"
                .into(),
        );
    }

    // LEK-01: geen beoordeling binnen twaalf uur.
    if i.risiconiveau.is_none() && (nu - i.geregistreerd_op) > Duration::hours(12) {
        voeg(
            "LEK-01",
            format!(
                "geregistreerd {} uur geleden, nog geen risicobeoordeling",
                (nu - i.geregistreerd_op).num_hours()
            ),
        );
    }

    // LEK-03: gat tussen kennisname en registratie.
    if let Some(vertraging) = i.registratievertraging() {
        if vertraging > Duration::hours(4) && i.verificatie_onderbouwing.is_none() {
            voeg(
                "LEK-03",
                format!(
                    "{} uur tussen kennisname en registratie, zonder toelichting",
                    vertraging.num_hours()
                ),
            );
        }
    }

    // LEK-04: niet melden zonder tweede laag.
    if let Meldbesluit::NietMelden { tweede_persoon, afkoelperiode_tot, .. } = &i.meldbesluit {
        if tweede_persoon.is_none() && afkoelperiode_tot.is_none() {
            voeg(
                "LEK-04",
                "besloten niet te melden zonder tweede persoon en zonder afkoelperiode".into(),
            );
        }
    }

    // LEK-06: geen risico bij grote omvang.
    if i.risiconiveau == Some(Risiconiveau::GeenRisico) && i.omvang_vereist_tegenspraak() {
        voeg(
            "LEK-06",
            format!("uitkomst geen risico bij {} betrokkenen", i.aantal_betrokkenen.unwrap_or(0)),
        );
    }

    // LEK-07: geen risico bij gevoelige gegevens.
    if i.risiconiveau == Some(Risiconiveau::GeenRisico) && i.tweede_persoon_verplicht() {
        let tweede_aanwezig =
            matches!(&i.meldbesluit, Meldbesluit::NietMelden { tweede_persoon: Some(_), .. });
        if !tweede_aanwezig && i.meldbesluit.is_niet_melden() {
            voeg(
                "LEK-07",
                "uitkomst geen risico terwijl er gevoelige gegevens in het spel zijn, zonder \
                 bevestiging door een tweede persoon"
                    .into(),
            );
        }
    }

    // LEK-08: besluit spreekt de weging tegen.
    if !dpofg_domain::klokken::besluit_past_bij_weging(i) && i.meldbesluit.is_genomen() {
        voeg(
            "LEK-08",
            format!(
                "de weging kwam uit op '{}' maar er is besloten niet te melden",
                i.risiconiveau.map(|r| r.omschrijving()).unwrap_or("nog niet gewogen")
            ),
        );
    }

    // LEK-12: afgesloten zonder oorzaak of maatregel.
    if i.afgehandeld_op.is_some() {
        if i.oorzaakcategorie.is_none() {
            voeg("LEK-12", "afgesloten zonder oorzaakcategorie".into());
        }
        if i.zonder_maatregel() {
            voeg("LEK-12", "afgesloten zonder enige maatregel".into());
        }
    }

    // LEK-15: geen koppeling aan een verwerking, na een respijt van twaalf uur.
    //
    // De koppeling is de uitkomst van het onderzoek en niet van de intake. Wie
    // een incident registreert weet vaak nog niet welke verwerking is geraakt;
    // meteen melden levert dan een bevinding op bij iemand die er op dat moment
    // niets aan kan doen. Dezelfde orde van grootte als LEK-01.
    if i.getroffen_verwerkingen.is_empty() && (nu - i.geregistreerd_op) > Duration::hours(12) {
        voeg("LEK-15", "het incident is aan geen enkele verwerking gekoppeld".into());
    }

    uit
}

/// Beoordeelt de meldtermijn van één incident (LEK-02).
///
/// De termijn wordt niet hier berekend maar meegegeven. Dat is met opzet: de
/// tweeënzeventig uur staat in het kennispakket en niet in de programmacode, en
/// een regel die zijn eigen termijn uitrekent, gaat een tweede waarheid voeren
/// naast de termijnenmotor.
///
/// De ondergrens is niet optioneel. Zonder `resterend > 0` blijft de regel na
/// het verstrijken eeuwig afgaan op een incident waar niemand nog iets aan kan
/// doen, en dat is precies hoe een gebruiker leert meldingen weg te klikken.
pub fn beoordeel_meldtermijn(
    motor: &Regelmotor,
    i: &Incident,
    meldtermijn: &Deadline,
    nu: DateTime<Utc>,
) -> Vec<Bevinding> {
    // Al besloten, al gemeld of al afgehandeld: dan valt er niets meer aan te
    // herinneren.
    if !matches!(i.meldbesluit, Meldbesluit::NogTeNemen)
        || i.gemeld_op.is_some()
        || i.afgehandeld_op.is_some()
    {
        return Vec::new();
    }

    let resterend = meldtermijn.resterend(nu);
    if resterend <= Duration::zero() || resterend > VENSTER_MELDTERMIJN {
        return Vec::new();
    }

    motor
        .bevind(
            "LEK-02",
            "incident",
            &i.id.to_string(),
            Some(&i.kenmerk),
            format!(
                "nog {} uur tot het einde van de meldtermijn van {} (verstrijkt {}), meldbesluit nog niet genomen",
                resterend.num_hours(),
                meldtermijn.duur,
                meldtermijn.lokaal
            ),
            nu,
        )
        .into_iter()
        .collect()
}

/// Hoe lang vóór het verstrijken van de meldtermijn LEK-02 aanslaat.
const VENSTER_MELDTERMIJN: Duration = Duration::hours(12);

/// Beoordeelt de staat van het ketenlogboek (SYS-04 en SYS-10).
///
/// Deze regels gaan niet over de inhoud van een dossier maar over het systeem
/// eronder. Ze staan daarom los van de recordbeoordelingen en krijgen
/// `record_soort` "logboek".
pub fn beoordeel_logboek(
    motor: &Regelmotor,
    rapport: &Verificatierapport,
    ketenstand_tijdstip: Option<DateTime<Utc>>,
    nu: DateTime<Utc>,
) -> Vec<Bevinding> {
    let mut uit = Vec::new();
    let mut voeg = |code: &str, id: String, toelichting: String| {
        if let Some(b) = motor.bevind(code, "logboek", &id, None, toelichting, nu) {
            uit.push(b);
        }
    };

    for b in &rapport.bevindingen {
        let code = match b.soort {
            Bevindingsoort::TijdLooptTerug => "SYS-10",
            Bevindingsoort::Ketenbreuk
            | Bevindingsoort::OntbrekendeRegel
            | Bevindingsoort::DubbeleRegel
            | Bevindingsoort::InhoudGewijzigd => "SYS-04",
        };
        voeg(code, b.volgnummer.to_string(), format!("regel {}: {}", b.volgnummer, b.omschrijving));
    }

    // Het anker. `GeenAnker` is uitdrukkelijk géén bevinding: dat is de normale
    // toestand van elke kluis waarvan nog geen anker buiten het systeem is
    // bewaard, en de schil behandelt het al als advies. `AnkerOngeldig` evenmin
    // — dat zegt iets over het anker, niet over de keten.
    match &rapport.ankerstatus {
        Ankerstatus::KetenIsIngekort { anker_volgnummer, keten_volgnummer } => voeg(
            "SYS-04",
            anker_volgnummer.to_string(),
            format!(
                "het anker verklaart regel {anker_volgnummer}, de keten eindigt bij {keten_volgnummer}"
            ),
        ),
        Ankerstatus::HashWijktAf { volgnummer, .. } => voeg(
            "SYS-04",
            volgnummer.to_string(),
            format!("op ankerpositie {volgnummer} wijkt de hash af van wat het anker verklaart"),
        ),
        _ => {}
    }

    // SYS-10: de klok staat vóór het laatst vastgelegde tijdstip. Uitsluitend
    // vastleggingstijdstippen tellen mee; een incident dat achteraf wordt
    // geregistreerd heeft een tijdstip in het verleden en dat is normaal.
    if let Some(t) = ketenstand_tijdstip {
        if nu < t {
            voeg(
                "SYS-10",
                "klok".into(),
                format!(
                    "de klok van deze machine loopt {} minuten achter op het laatst vastgelegde tijdstip ({})",
                    (t - nu).num_minutes(),
                    t.format("%d-%m-%Y %H:%M UTC")
                ),
            );
        }
    }

    uit
}

/// Beoordeelt het waarschuwingsbudget (SYS-06).
///
/// De drempel komt uit [`crate::budget`] zelf en niet uit deze regel: het
/// budget is de norm, de regel maakt hem alleen zichtbaar.
pub fn beoordeel_budget(
    motor: &Regelmotor,
    budget: &Waarschuwingsbudget,
    nu: DateTime<Utc>,
) -> Vec<Bevinding> {
    let mut uit = Vec::new();
    for stand in budget.overschrijdingen(nu) {
        if let Some(melding) = stand.defectmelding() {
            if let Some(b) =
                motor.bevind("SYS-06", "gebruiker", &stand.gebruiker, None, melding, nu)
            {
                uit.push(b);
            }
        }
    }
    uit
}

/// Zoekt het patroon van herhaalde oorzaken (LEK-13).
///
/// Deze regel kijkt over incidenten heen en kan daarom niet per record worden
/// beoordeeld. Dat is precies waarom de regels hier los staan van de
/// volledigheidscontrole.
pub fn beoordeel_oorzaakpatroon(
    motor: &Regelmotor,
    incidenten: &[Incident],
    nu: DateTime<Utc>,
    kwartaalgrens: DateTime<Utc>,
) -> Vec<Bevinding> {
    let mut tellers: std::collections::BTreeMap<&str, Vec<&Incident>> = Default::default();
    for i in incidenten {
        if i.geregistreerd_op < kwartaalgrens {
            continue;
        }
        if let Some(o) = &i.oorzaakcategorie {
            tellers.entry(o.as_str()).or_default().push(i);
        }
    }

    let mut uit = Vec::new();
    for (oorzaak, mut groep) in tellers {
        // Op het oudste incident landen, en niet op het eerste in de
        // aangeleverde volgorde: de opslaglaag levert op laatst gewijzigd, dus
        // anders wisselt de bevinding van record zonder dat er iets verandert.
        groep.sort_by_key(|i| (i.geregistreerd_op, i.kenmerk.as_str()));
        if groep.len() > 3 {
            if let Some(b) = motor.bevind(
                "LEK-13",
                "incident",
                &groep[0].id.to_string(),
                Some(&groep[0].kenmerk),
                format!(
                    "'{oorzaak}' kwam dit kwartaal {} keer voor: {}",
                    groep.len(),
                    groep.iter().map(|i| i.kenmerk.as_str()).collect::<Vec<_>>().join(", ")
                ),
                nu,
            ) {
                uit.push(b);
            }
        }
    }
    uit
}

impl Regelmotor {
    /// De regels die in de catalogus staan maar nog geen evaluatiefunctie hebben.
    ///
    /// Bewust een publieke functie: het aantal regels in de catalogus mag geen
    /// dekking suggereren die er niet is. Wie wil weten wat de toepassing
    /// werkelijk bewaakt, vraagt dit op.
    pub fn regels_zonder_evaluatie(&self) -> Vec<&Regel> {
        let klaar = geimplementeerd();
        self.alle().filter(|r| !klaar.contains(&r.code.as_str())).collect()
    }

    /// Het aandeel van de catalogus dat daadwerkelijk draait.
    pub fn dekking(&self) -> f64 {
        if self.aantal() == 0 {
            return 0.0;
        }
        let klaar = geimplementeerd().len().min(self.aantal());
        klaar as f64 / self.aantal() as f64
    }
}

// Zodat de opsomming van grondslagen ook echt wordt gebruikt en niet stilzwijgend
// verdwijnt bij een wijziging.
#[allow(dead_code)]
fn grondslagen_die_extra_eisen_stellen() -> Vec<Grondslag> {
    Grondslag::alle()
        .into_iter()
        .filter(|g| {
            g.vereist_belangenafweging()
                || g.vereist_toestemmingsbewijs()
                || g.vereist_wettelijke_bepaling()
        })
        .collect()
}
