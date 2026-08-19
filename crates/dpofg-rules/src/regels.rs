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
    avg::Grondslag,
    risico::Risicobeoordeling,
    zorgplicht::{
        Bewijsaanwijzing, Bewijskracht, Bewijsrol, Maatregelstand, Toepassing, Zorgplichtdossier,
    },
    Bewaartermijn, Doorgifte, Dpia, Incident, Leverancier, Meldbesluit, Risiconiveau, Status,
    Verwerking, Volledig, Voortoets,
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
    uit.extend(zorgplicht());
    uit.extend(risicobeoordeling());
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
        "DPIA-03", "DPIA-06", "DPIA-07", "EER-03", "EER-06", "EER-07", "VWO-02", "VWO-04",
        "VWO-09", "VWO-13", "LEK-16", "ZRP-01", "ZRP-02", "ZRP-03", "ZRP-04", "ZRP-05", "ZRP-06",
        "ZRP-07", "ZRP-08", "ZRP-09", "ZRP-10", "ZRP-11", "ZRP-12", "ZRP-13", "RIS-01", "RIS-02",
        "RIS-03", "RIS-04", "RIS-05", "RIS-06", "SYS-04", "SYS-06", "SYS-10",
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
        //
        // Twee vormen onder één code, net als DPIA-07: geen verzoek ingediend
        // waar dat wel moest, en een verzoek waarvan de termijn zonder advies
        // is verstreken. De omschrijving dekt ze allebei, want de titel en de
        // grondslag reizen mee met elke bevinding.
        Regel::nieuw("DPIA-06", "effectbeoordeling", "Voorafgaande raadpleging niet op orde",
            "een hoog restrisico zonder voorafgaande raadpleging, of een raadplegingstermijn die zonder advies is verstreken",
            Signalerend, Functionaris, "art. 36 lid 1 en lid 2 AVG", false),
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

/// De zorgplichtcontrolset van artikel 21 lid 3 van de Cyberbeveiligingswet.
///
/// Eén blokkerende regel op dertien. Dat is met opzet: het inrichten van een
/// controlset is werk dat maanden duurt, en een blokkerende bevinding die al
/// die tijd in ieder overzicht staat, leert de gebruiker wegklikken. Wat
/// blokkeert, blokkeert op de plaats waar het thuishoort — bij het vaststellen
/// van het dossier zelf.
///
/// De uitzondering is ZRP-02. Een functionaris die eigenaar is van een
/// maatregel waarop hij toezicht houdt, is geen werk in uitvoering maar een
/// rolconflict, en dat conflict ontstaat buiten het dossier om: bij een
/// rolwissel. Zolang de tool die wissel niet mag weigeren, moet zij hem
/// zichtbaar maken.
fn zorgplicht() -> Vec<Regel> {
    vec![
        Regel::nieuw(
            "ZRP-01",
            "zorgplicht",
            "Maatregel zonder eigenaar",
            "een maatregel uit de controlset zonder rol met bezetting",
            Signalerend,
            Directie,
            "art. 6 lid 4 Cyberbeveiligingsbesluit",
            false,
        ),
        Regel::nieuw(
            "ZRP-02",
            "zorgplicht",
            "Eigenaar is de aangemelde functionaris",
            "de functionaris is eigenaar van een maatregel waarop hij toezicht houdt",
            Blokkerend,
            Directie,
            "art. 38 lid 6 AVG",
            false,
        ),
        Regel::nieuw(
            "ZRP-03",
            "zorgplicht",
            "Maatregel niet beoordeeld",
            "een maatregel staat na de beoordelingstermijn nog op nog niet beoordeeld",
            Signalerend,
            SecurityOfficer,
            "art. 21 lid 3 Cyberbeveiligingswet; de termijn is zelf vastgesteld",
            true,
        ),
        Regel::nieuw(
            "ZRP-04",
            "zorgplicht",
            "Ingericht maar niet aantoonbaar",
            "een ingerichte maatregel zonder bewijs van de uitvoering dat nu geldt",
            Signalerend,
            SecurityOfficer,
            "art. 6 lid 4 Cyberbeveiligingsbesluit",
            true,
        ),
        Regel::nieuw(
            "ZRP-05",
            "zorgplicht",
            "Bewijs verloopt binnen de horizon",
            "een bewijsstuk waarvan het geldigheidsvenster binnenkort sluit",
            Signalerend,
            SecurityOfficer,
            "art. 6 lid 4 Cyberbeveiligingsbesluit",
            true,
        ),
        Regel::nieuw(
            "ZRP-06",
            "zorgplicht",
            "Periodieke maatregel zonder frequentie",
            "een maatregel die het kader periodiek noemt zonder zelf vastgestelde termijn",
            Signalerend,
            SecurityOfficer,
            "zelf vastgestelde termijn; de wet noemt hier geen frequentie",
            false,
        ),
        Regel::nieuw(
            "ZRP-07",
            "zorgplicht",
            "Frequentie langer dan de norm",
            "een zelf vastgestelde uitvoeringsfrequentie boven de drempel uit het pakket",
            Signalerend,
            SecurityOfficer,
            "interne norm; geen wettelijke drempel",
            true,
        ),
        Regel::nieuw(
            "ZRP-08",
            "zorgplicht",
            "Uitvoering achter op de eigen frequentie",
            "de laatste uitvoering ligt langer geleden dan de zelf vastgestelde termijn",
            Signalerend,
            SecurityOfficer,
            "zelf vastgestelde termijn",
            true,
        ),
        Regel::nieuw(
            "ZRP-09",
            "zorgplicht",
            "Geen bestuursvaststelling",
            "het maatregelenpakket is niet door het bestuur vastgesteld",
            Signalerend,
            Directie,
            "art. 24 lid 1 Cyberbeveiligingswet",
            false,
        ),
        Regel::nieuw(
            "ZRP-10",
            "zorgplicht",
            "Bestuursvaststelling verouderd",
            "de bestuursvaststelling is ouder dan de zelf vastgestelde termijn",
            Signalerend,
            Directie,
            "art. 24 lid 1 Cyberbeveiligingswet",
            true,
        ),
        Regel::nieuw(
            "ZRP-11",
            "zorgplicht",
            "Risicobeoordeling ontbreekt of is verlopen",
            "geen geldige risicobeoordeling onder de controlset",
            Signalerend,
            Directie,
            "art. 21 lid 1 en 2 Cyberbeveiligingswet",
            false,
        ),
        Regel::nieuw(
            "ZRP-12",
            "zorgplicht",
            "Zelfgerapporteerd bewijs waar toetsing wordt verwacht",
            "het kader verwacht externe toetsing en het bewijs berust op de eigen verklaring",
            Signalerend,
            Functionaris,
            "art. 6 lid 4 Cyberbeveiligingsbesluit",
            true,
        ),
        Regel::nieuw(
            "ZRP-13",
            "zorgplicht",
            "Niet-toepassing is gewoonte geworden",
            "van de maatregelen waar afwijken mag, wordt een te groot deel niet toegepast",
            Rapporterend,
            Directie,
            "interne norm; geen wettelijke drempel",
            true,
        ),
    ]
}

/// De risicobeoordeling als zelfstandig artefact.
///
/// Eén blokkerende regel: een hoog restrisico dat niemand heeft aanvaard.
/// Dat is geen werk in uitvoering maar een besluit dat niet is genomen, en
/// zolang het niet is genomen, staat de organisatie bloot aan iets waarvan
/// niemand heeft gezegd dat het aanvaardbaar is.
fn risicobeoordeling() -> Vec<Regel> {
    vec![
        Regel::nieuw(
            "RIS-01",
            "risicobeoordeling",
            "Beoordeling verlopen",
            "de geldigheidsduur van de risicobeoordeling is verstreken",
            Signalerend,
            SecurityOfficer,
            "art. 21 lid 1 Cyberbeveiligingswet",
            true,
        ),
        Regel::nieuw(
            "RIS-02",
            "risicobeoordeling",
            "Beoordeling verloopt binnen de horizon",
            "de geldigheidsduur verstrijkt binnen de termijn uit het kennispakket",
            Signalerend,
            SecurityOfficer,
            "interne norm; geen wettelijke termijn",
            true,
        ),
        Regel::nieuw(
            "RIS-03",
            "risicobeoordeling",
            "Hoog restrisico niet aanvaard",
            "een restrisico van klasse hoog zonder vastgelegde aanvaarding",
            Blokkerend,
            Directie,
            "art. 24 lid 1 Cyberbeveiligingswet",
            false,
        ),
        Regel::nieuw(
            "RIS-04",
            "risicobeoordeling",
            "Risico zonder maatregel",
            "een onderkend risico waarbij geen enkele maatregel is genoemd",
            Signalerend,
            SecurityOfficer,
            "art. 21 lid 1 Cyberbeveiligingswet",
            true,
        ),
        Regel::nieuw(
            "RIS-05",
            "risicobeoordeling",
            "Geen bron geraadpleegd",
            "een beoordeling die uitsluitend op het eigen beeld berust",
            Signalerend,
            SecurityOfficer,
            "interne norm; methodische onderbouwing",
            true,
        ),
        Regel::nieuw(
            "RIS-06",
            "risicobeoordeling",
            "Aanvaarding ouder dan de beoordeling",
            "een restrisico is aanvaard vóór de laatste wijziging van de beoordeling",
            Signalerend,
            Directie,
            "interne norm",
            true,
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

/// Beoordeelt één effectbeoordeling (DPIA-03, DPIA-06 en DPIA-07).
///
/// De herbeoordelingsdrempel komt uit het kennispakket en niet uit deze code:
/// zesendertig maanden is een norm, en normen horen in de inhoud te staan waar
/// een jurist ze kan bijstellen.
pub fn beoordeel_dpia(
    motor: &Regelmotor,
    d: &Dpia,
    herbeoordeling_maanden: i64,
    nu: DateTime<Utc>,
) -> Vec<Bevinding> {
    let mut uit = Vec::new();
    let kenmerk = Some(d.kenmerk.as_str());
    let id = d.id.to_string();

    let mut voeg = |code: &str, toelichting: String| {
        if let Some(b) = motor.bevind(code, "dpia", &id, kenmerk, toelichting, nu) {
            uit.push(b);
        }
    };

    // DPIA-03: de beoordeling is uitgevoerd nadat de verwerking al liep.
    //
    // Alleen bij een vereiste beoordeling: bij een vrijwillige is er geen
    // moment waarvóór zij had moeten plaatsvinden. `None` slaat niet aan — dat
    // is een onbeantwoorde vraag, geen "nee".
    if d.voortoets == Some(Voortoets::Vereist) && d.vooraf_uitgevoerd == Some(false) {
        let wanneer = d
            .datum
            .map(|t| t.format("%d-%m-%Y").to_string())
            .unwrap_or_else(|| "onbekende datum".into());
        voeg(
            "DPIA-03",
            format!(
                "de beoordeling van {wanneer} is uitgevoerd nadat de verwerking al liep; artikel 35 lid 1 vraagt haar vóór de verwerking"
            ),
        );
    }

    // DPIA-06, eerste vorm: hoog restrisico en geen verzoek ingediend.
    if d.raadpleging_nodig() && d.raadpleging.is_none() {
        voeg(
            "DPIA-06",
            "het restrisico is als hoog beoordeeld en er is geen verzoek om voorafgaande raadpleging ingediend"
                .into(),
        );
    }

    // DPIA-07, eerste tak: de beoordeling is verouderd.
    if d.status.is_actief() {
        if let Some(maanden) = d.maanden_sinds_beoordeling(nu) {
            if maanden >= herbeoordeling_maanden {
                voeg("DPIA-07", format!("laatst beoordeeld {maanden} maanden geleden"));
            }
        }
    }

    // DPIA-07, tweede tak: de onderliggende verwerking is gewijzigd.
    if d.status == Status::HerzieningNodig {
        // De reden staat in de herkomst, maar alleen zolang niemand het dossier
        // sindsdien heeft aangeraakt: `wijzig` overschrijft dat veld bij elke
        // bewerking. De prefix "systeem: " wordt uitsluitend door
        // `markeer_herziening_nodig` geschreven en is dus een betrouwbare toets.
        let toelichting = d
            .herkomst
            .gewijzigd_door
            .strip_prefix("systeem: ")
            .map(|reden| format!("de onderliggende verwerking is gewijzigd: {reden}"))
            .unwrap_or_else(|| {
                "de onderliggende verwerking is gewijzigd; het dossier staat op herziening nodig"
                    .into()
            });
        voeg("DPIA-07", toelichting);
    }

    uit
}

/// Beoordeelt de lopende raadplegingstermijn van één effectbeoordeling (DPIA-06).
///
/// Anders dan bij de meldtermijn is er géén herinnering vóór het verstrijken:
/// zolang de termijn van de toezichthouder loopt, valt er voor de organisatie
/// niets te doen. Wat er wél toe doet is het moment waarop hij verstrijkt zonder
/// antwoord — want stilzitten van de toezichthouder is geen goedkeuring.
pub fn beoordeel_raadplegingstermijn(
    motor: &Regelmotor,
    d: &Dpia,
    deadline: &Deadline,
    nu: DateTime<Utc>,
) -> Vec<Bevinding> {
    if d.advies_ontvangen_op().is_some() || nu <= deadline.moment {
        return Vec::new();
    }

    motor
        .bevind(
            "DPIA-06",
            "dpia",
            &d.id.to_string(),
            Some(&d.kenmerk),
            format!(
                "de termijn van {} voor de voorafgaande raadpleging is op {} verstreken zonder dat er advies van de toezichthouder is vastgelegd. Het verstrijken van deze termijn is geen goedkeuring: de verordening verbindt aan stilzitten van de toezichthouder geen instemming.",
                deadline.duur, deadline.lokaal
            ),
            nu,
        )
        .into_iter()
        .collect()
}

/// Beoordeelt één doorgifte (EER-03, EER-06 en EER-07).
///
/// De drempel voor structureel gebruik komt uit het kennispakket: hoeveel
/// "incidenteel" is, hoort een jurist te bepalen en niet deze code.
pub fn beoordeel_doorgifte(
    motor: &Regelmotor,
    d: &Doorgifte,
    uitzonderingsdrempel: u32,
    nu: DateTime<Utc>,
) -> Vec<Bevinding> {
    let mut uit = Vec::new();
    let kenmerk = Some(d.kenmerk.as_str());
    let id = d.id.to_string();

    let mut voeg = |code: &str, toelichting: String| {
        if let Some(b) = motor.bevind(code, "doorgifte", &id, kenmerk, toelichting, nu) {
            uit.push(b);
        }
    };

    // EER-03: een instrument van artikel 46 zonder afgeronde beoordeling.
    if d.mist_beoordeling() {
        voeg(
            "EER-03",
            format!(
                "{} naar {} zonder beoordeling van het recht en de praktijk in het ontvangstland",
                d.instrument.map(|i| i.omschrijving()).unwrap_or("het instrument"),
                d.ontvangerland
            ),
        );
    }

    // EER-06: een uitzondering die structureel wordt gebruikt.
    if d.gebruikt_uitzondering_structureel(uitzonderingsdrempel) {
        voeg(
            "EER-06",
            format!(
                "de uitzondering is dit jaar {} keer toegepast; boven {} is het geen uitzondering meer maar een instrument dat ontbreekt",
                d.artikel49_toepassingen_dit_jaar, uitzonderingsdrempel
            ),
        );
    }

    // EER-07: het instrument waarop de doorgifte berust is niet meer geldig.
    //
    // De status komt uit het kennispakket en wordt bij de controle op de
    // doorgifte vastgelegd; hier wordt alleen gelezen wat daar staat.
    if d.status == Status::HerzieningNodig {
        let status = d.instrument_status_bij_controle.as_deref().unwrap_or("onbekend");
        voeg(
            "EER-07",
            format!(
                "het instrument {} staat op '{status}'; de waarborg waarop deze doorgifte rust \
                 is er niet meer of staat ter discussie",
                d.instrument_code.as_deref().unwrap_or("waarop deze doorgifte berust")
            ),
        );
    }

    uit
}

/// Beoordeelt één leverancier (VWO-02, VWO-04, VWO-09 en VWO-13).
///
/// Beide drempels komen uit het kennispakket: hoeveel uur een verwerker mag
/// nemen om te melden en hoe vaak de subverwerkerslijst moet worden nagelopen,
/// zijn normen en horen niet in deze code te staan.
pub fn beoordeel_leverancier(
    motor: &Regelmotor,
    l: &Leverancier,
    meldtermijndrempel_uren: u32,
    subverwerkersdrempel_maanden: i64,
    nu: DateTime<Utc>,
) -> Vec<Bevinding> {
    let mut uit = Vec::new();
    let kenmerk = Some(l.kenmerk.as_str());
    let id = l.id.to_string();

    let mut voeg = |code: &str, toelichting: String| {
        if let Some(b) = motor.bevind(code, "leverancier", &id, kenmerk, toelichting, nu) {
            uit.push(b);
        }
    };

    let Some(overeenkomst) = &l.overeenkomst else {
        // Zonder overeenkomst valt er niets aan het contract te toetsen; dat de
        // overeenkomst ontbreekt, meldt de volledigheidscontrole al.
        return uit;
    };

    // VWO-02: onderdelen van artikel 28 lid 3 zonder vindplaats.
    let zonder = overeenkomst.eisen_zonder_vindplaats();
    if !zonder.is_empty() {
        voeg(
            "VWO-02",
            format!(
                "{} van de acht onderdelen van artikel 28 lid 3 hebben geen vindplaats in het \
                 contract: {}",
                zonder.len(),
                zonder.iter().map(|e| e.letter()).collect::<Vec<_>>().join(", ")
            ),
        );
    }

    // VWO-04: de contractuele meldtermijn is te lang.
    //
    // Is er in het geheel geen termijn afgesproken, dan zwijgt deze regel. Dat
    // gat blokkeert het vaststellen van de leverancier al, en een tweede
    // melding over hetzelfde gemis leert de gebruiker alleen maar wegklikken.
    if l.meldtermijn_te_lang(meldtermijndrempel_uren) {
        let uren = overeenkomst.meldtermijn_uren.unwrap_or_default();
        voeg(
            "VWO-04",
            format!(
                "de verwerker heeft {uren} uur om te melden; boven {meldtermijndrempel_uren} uur \
                 blijft er van de eigen termijn van tweeënzeventig uur te weinig over"
            ),
        );
    }

    // VWO-09: de subverwerkerslijst is te lang geleden nagelopen — of nooit.
    //
    // Nooit nagelopen is geen mildere variant van te lang geleden; het is de
    // ernstigere. De volledigheidscontrole meldt het gemis wel, maar alleen
    // signalerend, zodat een leverancier zonder één controle toch kan worden
    // vastgesteld. Zou deze regel dan zwijgen, dan verdween de enige plaats
    // waar het daarna nog zichtbaar was.
    match l.maanden_sinds_subverwerkerscontrole(nu) {
        None => voeg(
            "VWO-09",
            "de subverwerkerslijst is nooit nagelopen; er is dus niet vastgesteld wie er \
             achter deze verwerker meewerkt"
                .to_string(),
        ),
        Some(maanden) if maanden >= subverwerkersdrempel_maanden => voeg(
            "VWO-09",
            format!("de subverwerkerslijst is {maanden} maanden geleden voor het laatst nagelopen"),
        ),
        Some(_) => {}
    }

    // VWO-13: het contract is getekend nadat de verwerking al liep.
    if overeenkomst.getekend_na_aanvang() {
        let start = overeenkomst
            .verwerking_begon_op
            .map(|d| d.format("%d-%m-%Y").to_string())
            .unwrap_or_else(|| "onbekend".into());
        let dagen = overeenkomst
            .verwerking_begon_op
            .map(|d| (overeenkomst.ondertekend_op - d).num_days())
            .unwrap_or_default();
        voeg(
            "VWO-13",
            format!(
                "de verwerking begon op {start} en de overeenkomst is getekend op {}; die \
                 {dagen} dagen zijn niet gedekt",
                overeenkomst.ondertekend_op.format("%d-%m-%Y")
            ),
        );
    }

    uit
}

/// Drempels waartegen de zorgplichtcontrolset wordt gemeten.
///
/// Vijf getallen die de wet geen van alle noemt. Ze komen daarom uit het
/// kennispakket en worden hier meegegeven; een regel die zijn eigen norm
/// verzint, gaat een tweede waarheid voeren naast het pakket.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Zorgplichtdrempels {
    /// Na hoeveel dagen een nog niet beoordeelde maatregel wordt gemeld.
    pub beoordelingstermijn_dagen: i64,
    /// Hoeveel dagen vooruit verlopend bewijs wordt gemeld.
    pub bewijshorizon_dagen: i64,
    /// Boven hoeveel maanden een zelf vastgestelde frequentie wordt gemeld.
    pub frequentiedrempel_maanden: u32,
    /// Na hoeveel maanden de bestuursvaststelling als verouderd geldt.
    pub bestuursvaststelling_maanden: i64,
    /// Boven welk percentage niet-toepassing van uitzondering gewoonte wordt.
    pub afwijkingsaandeel_procent: u32,
}

/// Beoordeelt één zorgplichtdossier (ZRP-01 tot en met ZRP-13).
///
/// De stand per maatregel wordt hier niet opnieuw uitgerekend maar opgevraagd:
/// `Maatregelstand` is een berekening in het domein en er hoort maar één plaats
/// te zijn waar wordt bepaald wanneer iets aantoonbaar is.
pub fn beoordeel_zorgplicht(
    motor: &Regelmotor,
    d: &Zorgplichtdossier,
    beoordelingen: &[Risicobeoordeling],
    drempels: Zorgplichtdrempels,
    nu: DateTime<Utc>,
) -> Vec<Bevinding> {
    let mut uit = Vec::new();
    let kenmerk = Some(d.kenmerk.as_str());
    let id = d.id.to_string();

    let mut voeg = |code: &str, toelichting: String| {
        if let Some(b) = motor.bevind(code, "zorgplicht", &id, kenmerk, toelichting, nu) {
            uit.push(b);
        }
    };

    // ZRP-02 eerst: een rolconflict weegt zwaarder dan een open eind.
    let conflicten = d.eigenaarsconflicten();
    if !conflicten.is_empty() {
        let codes: Vec<_> = conflicten.iter().map(|m| m.code.as_str()).collect();
        voeg(
            "ZRP-02",
            format!(
                "{} is aangemeld als functionaris en tegelijk eigenaar van {}; toezicht op het \
                 eigen werk is geen toezicht",
                d.aangemelde_functionaris,
                codes.join(", ")
            ),
        );
    }

    let zonder_eigenaar: Vec<_> =
        d.maatregelen.iter().filter(|m| m.eigenaar.is_none()).map(|m| m.code.as_str()).collect();
    if !zonder_eigenaar.is_empty() {
        voeg(
            "ZRP-01",
            format!(
                "{} van de {} maatregelen hebben geen eigenaar: {}",
                zonder_eigenaar.len(),
                d.maatregelen.len(),
                zonder_eigenaar.join(", ")
            ),
        );
    }

    let dagen_sinds_afleiden = (nu - d.herkomst.aangemaakt_op).num_days();
    if dagen_sinds_afleiden >= drempels.beoordelingstermijn_dagen {
        let onbeoordeeld: Vec<_> = d
            .maatregelen
            .iter()
            .filter(|m| matches!(m.toepassing, Toepassing::NogNietBeoordeeld))
            .map(|m| m.code.as_str())
            .collect();
        if !onbeoordeeld.is_empty() {
            voeg(
                "ZRP-03",
                format!(
                    "{} maatregelen staan na {dagen_sinds_afleiden} dagen nog op nog niet \
                     beoordeeld: {}",
                    onbeoordeeld.len(),
                    onbeoordeeld.join(", ")
                ),
            );
        }
    }

    // Alles hieronder telt per dossier en niet per maatregel. Vijftien
    // bevindingen die alle vijftien hetzelfde zeggen, zijn geen vijftien keer
    // zoveel informatie; ze zijn de manier waarop een gebruiker leert
    // wegklikken. Regel SYS-06 rekent dat elders af als ontwerpdefect, dus het
    // hoort hier niet te ontstaan.
    let codes = |v: &[&str]| v.join(", ");

    let niet_aantoonbaar: Vec<&str> = d
        .maatregelen
        .iter()
        .filter(|m| m.stand(nu) == Maatregelstand::VastgesteldNietAantoonbaar)
        .map(|m| m.code.as_str())
        .collect();
    if !niet_aantoonbaar.is_empty() {
        voeg(
            "ZRP-04",
            format!(
                "{} maatregelen zijn ingericht zonder bewijs van de uitvoering dat nu geldt: {}",
                niet_aantoonbaar.len(),
                codes(&niet_aantoonbaar)
            ),
        );
    }

    let zonder_frequentie: Vec<&str> = d
        .maatregelen
        .iter()
        .filter(|m| m.periodiek && m.frequentie.is_none())
        .map(|m| m.code.as_str())
        .collect();
    if !zonder_frequentie.is_empty() {
        voeg(
            "ZRP-06",
            format!(
                "{} maatregelen worden door het kader periodiek genoemd zonder dat er een \
                 uitvoeringstermijn is vastgesteld: {}",
                zonder_frequentie.len(),
                codes(&zonder_frequentie)
            ),
        );
    }

    let te_ruim: Vec<String> = d
        .maatregelen
        .iter()
        .filter_map(|m| {
            let f = m.frequentie.as_ref()?;
            (f.maanden > drempels.frequentiedrempel_maanden)
                .then(|| format!("{} ({} maanden)", m.code, f.maanden))
        })
        .collect();
    if !te_ruim.is_empty() {
        voeg(
            "ZRP-07",
            format!(
                "een uitvoeringstermijn boven {} maanden laat een maatregel daartussen te lang \
                 onbeproefd: {}",
                drempels.frequentiedrempel_maanden,
                te_ruim.join(", ")
            ),
        );
    }

    let achterstallig: Vec<String> = d
        .maatregelen
        .iter()
        .filter_map(|m| {
            let f = m.frequentie.as_ref()?;
            let maanden = m.maanden_sinds_uitvoering(nu)?;
            (maanden > i64::from(f.maanden))
                .then(|| format!("{} ({maanden} maanden geleden, termijn {})", m.code, f.maanden))
        })
        .collect();
    if !achterstallig.is_empty() {
        voeg(
            "ZRP-08",
            format!(
                "de laatste uitvoering ligt langer geleden dan de eigen termijn bij: {}",
                achterstallig.join(", ")
            ),
        );
    }

    // Meet op uitvoeringsbewijs, want daar verwijst de toelichting naar. Zou
    // deze tak op elk bewijsstuk aanslaan en de vervallijst alleen op
    // uitvoering, dan stuurt een alarm de gebruiker naar een lege lijst — en
    // een alarm dat naar niets wijst, is de snelste weg naar wegklikken.
    let vervallend: Vec<&Bewijsaanwijzing> = d
        .maatregelen
        .iter()
        .filter_map(|m| m.geldig_uitvoeringsbewijs(nu))
        .filter(|b| b.dagen_tot_verval(nu) <= drempels.bewijshorizon_dagen)
        .collect();
    if let Some(eerste) = vervallend.iter().min_by_key(|b| b.geldig_tot) {
        voeg(
            "ZRP-05",
            format!(
                "bij {} maatregel(en) verloopt het uitvoeringsbewijs binnen {} dagen; het \
                 eerste op {}. Bekijk de lijst met 'dpofg zorgplicht vervalt'",
                vervallend.len(),
                drempels.bewijshorizon_dagen,
                eerste.geldig_tot.format("%d-%m-%Y")
            ),
        );
    }

    // Kijkt naar wat vandaag geldt, en alleen naar toetsingsbewijs. Overal
    // elders in deze module telt verlopen bewijs als geen bewijs; deed deze
    // tak dat niet, dan zou één auditverklaring uit het verleden de regel voor
    // altijd het zwijgen opleggen — en er is geen opdracht om bewijs weer weg
    // te halen.
    let alleen_zelfgerapporteerd: Vec<&str> = d
        .maatregelen
        .iter()
        .filter(|m| m.externe_toetsing_verwacht)
        .filter(|m| {
            let geldig: Vec<_> = m
                .bewijs
                .iter()
                .filter(|b| b.rol == Bewijsrol::Toetsing && b.geldt_op(nu))
                .collect();
            geldig.is_empty()
                || geldig.iter().all(|b| b.bewijskracht == Bewijskracht::Zelfgerapporteerd)
        })
        .filter(|m| !m.bewijs.is_empty())
        .map(|m| m.code.as_str())
        .collect();
    if !alleen_zelfgerapporteerd.is_empty() {
        voeg(
            "ZRP-12",
            format!(
                "het kader verwacht toetsing door een ander bij {}; daar ligt op dit moment \
                 geen geldige verklaring van een ander",
                codes(&alleen_zelfgerapporteerd)
            ),
        );
    }

    match &d.bestuursvaststelling {
        None => voeg(
            "ZRP-09",
            format!("kaderversie {} is niet door het bestuur vastgesteld", d.kaderversie),
        ),
        Some(b) => {
            let maanden = b.maanden_oud(nu);
            if maanden >= drempels.bestuursvaststelling_maanden {
                voeg(
                    "ZRP-10",
                    format!(
                        "het bestuur stelde het maatregelenpakket {maanden} maanden geleden \
                         vast, op {}",
                        b.datum.format("%d-%m-%Y")
                    ),
                );
            }
        }
    }

    match &d.risicobeoordeling {
        None => voeg(
            "ZRP-11",
            "er is geen risicobeoordeling aan deze controlset gekoppeld; zonder beoordeling is \
             niet te zeggen of dit de passende maatregelen zijn"
                .to_string(),
        ),
        Some(k) => match beoordelingen.iter().find(|b| b.id == k.id) {
            None => voeg(
                "ZRP-11",
                format!(
                    "de gekoppelde beoordeling {} staat niet in deze kluis; de koppeling wijst \
                     naar iets wat er niet is",
                    k.kenmerk
                ),
            ),
            Some(b) if b.is_verlopen(nu) => voeg(
                "ZRP-11",
                format!(
                    "de gekoppelde beoordeling {} is op {} verlopen",
                    b.kenmerk,
                    b.geldig_tot.format("%d-%m-%Y")
                ),
            ),
            Some(b) if b.status != Status::Vastgesteld => voeg(
                "ZRP-11",
                format!(
                    "de gekoppelde beoordeling {} is nog niet vastgesteld; een controlset kan \
                     niet steunen op een beoordeling die zelf nog concept is",
                    b.kenmerk
                ),
            ),
            Some(_) => {}
        },
    }

    if let Some(aandeel) = d.aandeel_niet_toegepast() {
        if aandeel > drempels.afwijkingsaandeel_procent {
            voeg(
                "ZRP-13",
                format!(
                    "van de {} maatregelen waar het kader afwijken toestaat, wordt {aandeel} \
                     procent gemotiveerd niet toegepast; boven {} procent is afwijken geen \
                     uitzondering meer",
                    d.aantal_afwijkbaar(),
                    drempels.afwijkingsaandeel_procent
                ),
            );
        }
    }
    uit
}

/// Beoordeelt één risicobeoordeling (RIS-01 tot en met RIS-06).
pub fn beoordeel_risicobeoordeling(
    motor: &Regelmotor,
    b: &Risicobeoordeling,
    horizon_dagen: i64,
    nu: DateTime<Utc>,
) -> Vec<Bevinding> {
    let mut uit = Vec::new();
    let kenmerk = Some(b.kenmerk.as_str());
    let id = b.id.to_string();

    let mut voeg = |code: &str, toelichting: String| {
        if let Some(bev) = motor.bevind(code, "risico", &id, kenmerk, toelichting, nu) {
            uit.push(bev);
        }
    };

    if b.is_verlopen(nu) {
        voeg(
            "RIS-01",
            format!(
                "de beoordeling van {} is op {} verlopen; de maatregelen eronder steunen op \\
                 een beeld dat niet meer is getoetst",
                b.uitgevoerd_op.format("%d-%m-%Y"),
                b.geldig_tot.format("%d-%m-%Y")
            ),
        );
    } else if b.dagen_tot_verval(nu) <= horizon_dagen {
        voeg(
            "RIS-02",
            format!(
                "de beoordeling verloopt over {} dagen, op {}",
                b.dagen_tot_verval(nu),
                b.geldig_tot.format("%d-%m-%Y")
            ),
        );
    }

    let hoog: Vec<&str> = b
        .risicos
        .iter()
        .filter(|r| r.vraagt_bestuur() && r.aanvaarding.is_none())
        .map(|r| r.code.as_str())
        .collect();
    if !hoog.is_empty() {
        voeg(
            "RIS-03",
            format!("het restrisico van {} is hoog en door niemand aanvaard", hoog.join(", ")),
        );
    }

    let zonder: Vec<&str> = b.zonder_maatregel().iter().map(|r| r.code.as_str()).collect();
    if !zonder.is_empty() {
        voeg(
            "RIS-04",
            format!(
                "bij {} is geen enkele maatregel genoemd; het restrisico is daarmee gelijk aan \\
                 het risico",
                zonder.join(", ")
            ),
        );
    }

    if b.bronnen.is_empty() {
        voeg(
            "RIS-05",
            "er is geen enkele bron vastgelegd; een beoordeling die alleen op het eigen beeld \\
             berust, ziet wat de organisatie al wist"
                .to_string(),
        );
    }

    // De aanvaarding gaat over het restrisico zoals dat gold toen zij werd
    // gegeven. Het domein laat een aanvaarding vervallen zodra het restrisico
    // wijzigt, maar een beoordeling kan ook op andere gronden zijn bijgewerkt;
    // dan is het besluit ouder dan het stuk waarover het gaat.
    let verouderd: Vec<&str> = b
        .risicos
        .iter()
        .filter(|r| r.aanvaarding.as_ref().is_some_and(|a| a.op < b.herkomst.gewijzigd_op))
        .map(|r| r.code.as_str())
        .collect();
    if !verouderd.is_empty() && b.status == Status::Vastgesteld {
        voeg(
            "RIS-06",
            format!(
                "de aanvaarding van {} dateert van vóór de laatste wijziging van deze \\
                 beoordeling",
                verouderd.join(", ")
            ),
        );
    }

    uit
}

/// Beoordeelt of de verwerker binnen zijn contractuele termijn heeft gemeld
/// (LEK-16).
///
/// De termijn komt van de leverancier waaraan het incident hangt; deze functie
/// rekent alleen na. Is er geen termijn afgesproken, dan valt er niets te
/// overschrijden — dat gat meldt de volledigheidscontrole van de leverancier.
pub fn beoordeel_verwerkersmelding(
    motor: &Regelmotor,
    i: &Incident,
    l: &Leverancier,
    nu: DateTime<Utc>,
) -> Vec<Bevinding> {
    let Some(termijn) = l.overeenkomst.as_ref().and_then(|o| o.meldtermijn_uren) else {
        return Vec::new();
    };
    let (Some(opgetreden), Some(ontvangen)) =
        (i.incident_bij_verwerker_op, i.melding_verwerker_ontvangen_op)
    else {
        return Vec::new();
    };

    let verstreken = (ontvangen - opgetreden).num_hours();
    if verstreken <= i64::from(termijn) {
        return Vec::new();
    }

    motor
        .bevind(
            "LEK-16",
            "incident",
            &i.id.to_string(),
            Some(&i.kenmerk),
            format!(
                "{} meldde na {verstreken} uur; de overeenkomst geeft {termijn} uur. Die \
                 overschrijding komt in mindering op de eigen termijn van tweeënzeventig uur",
                l.naam
            ),
            nu,
        )
        .into_iter()
        .collect()
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
