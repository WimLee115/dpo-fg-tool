//! Een startpakket om mee te beginnen.
//!
//! # Lees dit voordat u dit pakket in gebruik neemt
//!
//! Dit is een **vertrekpunt, geen bron van recht**. De waarden hierin zijn
//! samengesteld om de toepassing werkend te krijgen en om te laten zien welke
//! vorm het kennispakket heeft. Zij zijn niet door een jurist vastgesteld en
//! niet gecontroleerd tegen de geconsolideerde wettekst.
//!
//! Vóór gebruik in een echte organisatie hoort elk onderdeel te worden
//! geverifieerd tegen de bron: de geconsolideerde tekst op wetten.overheid.nl,
//! de tekst van de verordening op EUR-Lex, en de gepubliceerde besluiten van de
//! bevoegde autoriteit. Het veld `bron` bij elk onderdeel wijst aan waar die
//! controle moet plaatsvinden.
//!
//! De toepassing dwingt dit ook af: de consolidatiedatum van dit pakket staat
//! in elke export en elk auditdossier, zodat zichtbaar is op welke stand van de
//! inhoud een berekening berust.

use chrono::NaiveDate;
use dpofg_terms::{
    Aanvang, Eenheid, Feestdagenkalender, Rechtsstelsel, Termijnsoort, Verlengingsrecht,
};
use std::collections::{BTreeMap, BTreeSet};

use crate::pakket::{Doorgifteinstrument, Instrumentstatus, Pakketinhoud, Rechtsfeit};

fn d(j: i32, m: u32, dag: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(j, m, dag).expect("vaste, geldige datum")
}

/// Stelt het startpakket samen.
pub fn startpakket(consolidatiedatum: NaiveDate) -> Pakketinhoud {
    Pakketinhoud {
        code: "nl-start".into(),
        naam: "Startpakket Nederland — te verifiëren vóór gebruik".into(),
        versie: 1,
        versienaam: "0.1-start".into(),
        consolidatiedatum,
        jurisdictie: "NL".into(),
        minimaal_aanbevolen_programmaversie: "0.1.0".into(),
        termijnen: termijnen(),
        feestdagen: vec![feestdagen_nl()],
        rechtsfeiten: rechtsfeiten(),
        doorgifteinstrumenten: doorgifteinstrumenten(),
        aanvullend: aanvullend(),
    }
}

fn termijnen() -> Vec<Termijnsoort> {
    vec![
        // --- AVG ---
        Termijnsoort::uren(
            "AVG-33-MELDING",
            "melding van een inbreuk aan de toezichthouder",
            72,
            "art. 33 lid 1 AVG",
        ),
        Termijnsoort::kalender(
            "AVG-12-3-VERZOEK",
            "afhandeling van een verzoek van een betrokkene",
            1,
            Eenheid::Maanden,
            Rechtsstelsel::Unierecht,
            Aanvang::VanafGebeurtenis,
            "art. 12 lid 3 AVG",
        )
        .met_verlenging(Verlengingsrecht {
            duur: 2,
            eenheid: Eenheid::Maanden,
            aantal_keer: 1,
            bericht_binnen_oorspronkelijke_termijn: true,
            grondslag: "art. 12 lid 3, tweede volzin, AVG".into(),
        }),
        Termijnsoort::kalender(
            "AVG-12-4-WEIGERING",
            "bericht bij niet honoreren van een verzoek",
            1,
            Eenheid::Maanden,
            Rechtsstelsel::Unierecht,
            Aanvang::VanafGebeurtenis,
            "art. 12 lid 4 AVG",
        ),
        Termijnsoort::kalender(
            "AVG-36-RAADPLEGING",
            "voorafgaande raadpleging van de toezichthouder",
            8,
            Eenheid::Weken,
            Rechtsstelsel::Unierecht,
            Aanvang::VanafGebeurtenis,
            "art. 36 lid 3 AVG",
        )
        .opschortbaar()
        .met_verlenging(Verlengingsrecht {
            duur: 6,
            eenheid: Eenheid::Weken,
            aantal_keer: 1,
            bericht_binnen_oorspronkelijke_termijn: false,
            grondslag: "art. 36 lid 3, tweede volzin, AVG".into(),
        }),
        // --- Zorgplichtketen ---
        // De duren hieronder zijn de gangbare waarden uit de meldketen. Zij
        // horen per entiteitstype te worden geverifieerd: er bestaan
        // entiteitstypen met een verkorte termijn (randgeval T-07).
        Termijnsoort::uren(
            "ZORG-WAARSCHUWING",
            "vroegtijdige waarschuwing bij een significant incident",
            24,
            "meldketen zorgplicht, eerste bericht",
        ),
        Termijnsoort::uren(
            "ZORG-MELDING",
            "incidentmelding bij een significant incident",
            72,
            "meldketen zorgplicht, incidentmelding",
        ),
        Termijnsoort::kalender(
            "ZORG-EINDRAPPORT",
            "eindrapport na een significant incident",
            1,
            Eenheid::Maanden,
            Rechtsstelsel::Unierecht,
            Aanvang::VanafGebeurtenis,
            "meldketen zorgplicht, eindrapport",
        ),
        // --- Bestuursrecht ---
        Termijnsoort::kalender(
            "AWB-6-7-BEZWAAR",
            "indienen van een bezwaarschrift",
            6,
            Eenheid::Weken,
            Rechtsstelsel::NationaalRecht,
            Aanvang::VanafDagNaGebeurtenis,
            "Awb art. 6:7 en 6:8",
        ),
        Termijnsoort::kalender(
            "AWB-6-7-BEROEP",
            "instellen van beroep",
            6,
            Eenheid::Weken,
            Rechtsstelsel::NationaalRecht,
            Aanvang::VanafDagNaGebeurtenis,
            "Awb art. 6:7 en 6:8",
        ),
        // --- Zelf vastgestelde termijnen ---
        Termijnsoort::kalender(
            "INTERN-REGISTERHERZIENING",
            "periodieke herziening van een registerregel",
            12,
            Eenheid::Maanden,
            Rechtsstelsel::ZelfGesteld,
            Aanvang::VanafGebeurtenis,
            "interne norm; geen wettelijke termijn",
        ),
        Termijnsoort::kalender(
            "INTERN-DPIA-HERBEOORDELING",
            "herbeoordeling van een effectbeoordeling",
            36,
            Eenheid::Maanden,
            Rechtsstelsel::ZelfGesteld,
            Aanvang::VanafGebeurtenis,
            "interne norm, aansluitend op de richtsnoeren",
        ),
        Termijnsoort::kalender(
            "INTERN-SUBVERWERKERSCONTROLE",
            "controle van de subverwerkerslijst",
            12,
            Eenheid::Maanden,
            Rechtsstelsel::ZelfGesteld,
            Aanvang::VanafGebeurtenis,
            "interne norm",
        ),
    ]
}

/// De Nederlandse algemeen erkende feestdagen.
///
/// Te verifiëren tegen de Algemene termijnenwet en de jaarlijkse bekendmaking.
/// De variabele dagen — Pasen, Hemelvaart, Pinksteren — zijn berekend en niet
/// overgenomen uit een gepubliceerde lijst; controleer ze vóór gebruik.
fn feestdagen_nl() -> Feestdagenkalender {
    let dagen: BTreeSet<NaiveDate> = [
        // 2026
        (2026, 1, 1),
        (2026, 4, 3),
        (2026, 4, 5),
        (2026, 4, 6),
        (2026, 4, 27),
        (2026, 5, 5),
        (2026, 5, 14),
        (2026, 5, 24),
        (2026, 5, 25),
        (2026, 12, 25),
        (2026, 12, 26),
        // 2027
        (2027, 1, 1),
        (2027, 3, 26),
        (2027, 3, 28),
        (2027, 3, 29),
        (2027, 4, 27),
        (2027, 5, 5),
        (2027, 5, 6),
        (2027, 5, 16),
        (2027, 5, 17),
        (2027, 12, 25),
        (2027, 12, 26),
    ]
    .into_iter()
    .map(|(j, m, dag)| d(j, m, dag))
    .collect();

    Feestdagenkalender {
        jurisdictie: "NL".into(),
        dekking_van: 2026,
        dekking_tot_en_met: 2027,
        bron: "startpakket; te verifiëren tegen de Algemene termijnenwet".into(),
        dagen,
    }
}

fn rechtsfeiten() -> Vec<Rechtsfeit> {
    vec![
        Rechtsfeit {
            code: "AVG-IWT".into(),
            omschrijving: "toepassing van de Algemene verordening gegevensbescherming".into(),
            datum: d(2018, 5, 25),
            bron: "Verordening (EU) 2016/679, art. 99 lid 2".into(),
        },
        Rechtsfeit {
            code: "CBW-IWT".into(),
            omschrijving: "inwerkingtreding van de Nederlandse cyberbeveiligingswetgeving".into(),
            datum: d(2026, 8, 15),
            bron: "te verifiëren in het Staatsblad".into(),
        },
    ]
}

fn doorgifteinstrumenten() -> Vec<Doorgifteinstrument> {
    vec![
        Doorgifteinstrument {
            code: "SCC-2021".into(),
            land_of_gebied: "alle derde landen".into(),
            besluit_ref: "Uitvoeringsbesluit (EU) 2021/914".into(),
            status: Instrumentstatus::Geldig,
            vastgesteld_op: d(2021, 6, 4),
            geldig_tot: None,
            geverifieerd_op: d(2026, 8, 18),
            toelichting: "modelcontractbepalingen; vereist een beoordeling van de doorgifte \
                          en zo nodig aanvullende maatregelen"
                .into(),
        },
        Doorgifteinstrument {
            code: "ART49".into(),
            land_of_gebied: "alle derde landen".into(),
            besluit_ref: "art. 49 AVG".into(),
            status: Instrumentstatus::Geldig,
            vastgesteld_op: d(2018, 5, 25),
            geldig_tot: None,
            geverifieerd_op: d(2026, 8, 18),
            toelichting: "uitzondering voor incidentele doorgiften; structureel gebruik is geen \
                          uitzondering meer en vraagt om een ander instrument"
                .into(),
        },
    ]
}

fn aanvullend() -> BTreeMap<String, serde_json::Value> {
    let mut uit = BTreeMap::new();
    uit.insert(
        "waarschuwing".into(),
        serde_json::json!({
            "strekking": "Dit is een startpakket. De inhoud is niet door een jurist vastgesteld \
                          en niet gecontroleerd tegen de geconsolideerde wettekst. Verifieer elk \
                          onderdeel tegen de bron voordat u hierop vertrouwt.",
            "te_verifieren": [
                "de duur en de grondslag van elke termijn",
                "de feestdagenkalender, in het bijzonder de variabele dagen",
                "de datums in de rechtsfeiten",
                "de status van elk doorgifte-instrument",
                "of voor uw entiteitstype een verkorte meldtermijn geldt"
            ]
        }),
    );
    uit.insert(
        "oorzaakcategorieen".into(),
        serde_json::json!([
            "verzending naar een verkeerde geadresseerde",
            "verkeerde bijlage meegestuurd",
            "onbevoegde inzage door een medewerker",
            "verlies of diefstal van gegevensdrager",
            "hacking, malware of phishing",
            "onbedoelde publicatie",
            "onjuiste verwijdering of vernietiging",
            "onjuiste instelling van een systeem",
            "verlies van beschikbaarheid door storing",
            "fout bij een verwerker"
        ]),
    );
    uit
}
