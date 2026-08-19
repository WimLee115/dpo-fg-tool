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
        versie: 2,
        versienaam: "0.2-start".into(),
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
            // Lid 2 en niet lid 3: de acht weken, de verlenging met zes weken,
            // de berichttermijn van één maand en de opschortingsgrond staan
            // alle in lid 2. Lid 3 somt op welke stukken bij het verzoek gaan.
            // Deze tekst reist via de verantwoording mee naar elk dossier, dus
            // een verkeerde verwijzing komt uiteindelijk bij een toezichthouder
            // op tafel.
            "art. 36 lid 2 AVG",
        )
        .opschortbaar()
        .met_verlenging(Verlengingsrecht {
            duur: 6,
            eenheid: Eenheid::Weken,
            aantal_keer: 1,
            bericht_binnen_oorspronkelijke_termijn: false,
            grondslag: "art. 36 lid 2, tweede en derde volzin, AVG".into(),
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
        // --- Woo: het informatieverzoek ---
        //
        // Een ander spoor dan het inzageverzoek: andere termijn, andere
        // weigeringsgronden, andere rechtsbescherming. Dat de twee scherp
        // gescheiden blijven is geen vormkwestie — wie de AVG-maandtermijn op
        // een Woo-verzoek loslaat, is vier weken te laat.
        Termijnsoort::kalender(
            "WOO-BESLISTERMIJN",
            "beslissing op een verzoek om informatie",
            4,
            Eenheid::Weken,
            Rechtsstelsel::NationaalRecht,
            Aanvang::VanafGebeurtenis,
            "art. 4.4 lid 1 Wet open overheid",
        )
        .met_verlenging(Verlengingsrecht {
            duur: 2,
            eenheid: Eenheid::Weken,
            aantal_keer: 1,
            // De verdaging moet binnen de oorspronkelijke termijn schriftelijk
            // en gemotiveerd worden medegedeeld; daarna is zij niet meer in te
            // roepen.
            bericht_binnen_oorspronkelijke_termijn: true,
            grondslag: "art. 4.4 lid 2 Wet open overheid".into(),
        }),
        Termijnsoort::kalender(
            "WOO-ZIENSWIJZE",
            "zienswijze van een belanghebbende derde",
            2,
            Eenheid::Weken,
            Rechtsstelsel::NationaalRecht,
            Aanvang::VanafGebeurtenis,
            "art. 4.4 lid 4 Wet open overheid",
        ),
        // --- Wet politiegegevens ---
        //
        // De audit is vierjaarlijks en de interne controle jaarlijks. Beide
        // worden hier als maandtermijn uitgedrukt en niet als lopende klok:
        // een deadline vier jaar vooruit valt buiten de dekking van de
        // feestdagenkalender, en dan zou de motor terecht weigeren te rekenen
        // over een termijn waarvan de einddatum er niet toe doet op de dag
        // nauwkeurig.
        Termijnsoort::kalender(
            "WPG-EXTERNE-AUDIT",
            "externe audit op de verwerking van politiegegevens",
            48,
            Eenheid::Maanden,
            Rechtsstelsel::NationaalRecht,
            Aanvang::VanafGebeurtenis,
            "art. 33 lid 3 Wet politiegegevens",
        ),
        Termijnsoort::kalender(
            "WPG-INTERNE-CONTROLE",
            "interne controle op de verwerking van politiegegevens",
            12,
            Eenheid::Maanden,
            Rechtsstelsel::NationaalRecht,
            Aanvang::VanafGebeurtenis,
            "art. 33 lid 1 Wet politiegegevens",
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
        // 2028
        (2028, 1, 1),
        (2028, 4, 14),
        (2028, 4, 16),
        (2028, 4, 17),
        (2028, 4, 27),
        (2028, 5, 5),
        (2028, 5, 25),
        (2028, 6, 4),
        (2028, 6, 5),
        (2028, 12, 25),
        (2028, 12, 26),
        // 2029
        (2029, 1, 1),
        (2029, 3, 30),
        (2029, 4, 1),
        (2029, 4, 2),
        (2029, 4, 27),
        (2029, 5, 5),
        (2029, 5, 10),
        (2029, 5, 20),
        (2029, 5, 21),
        (2029, 12, 25),
        (2029, 12, 26),
        // 2030
        (2030, 1, 1),
        (2030, 4, 19),
        (2030, 4, 21),
        (2030, 4, 22),
        (2030, 4, 27),
        (2030, 5, 5),
        (2030, 5, 30),
        (2030, 6, 9),
        (2030, 6, 10),
        (2030, 12, 25),
        (2030, 12, 26),
    ]
    .into_iter()
    .map(|(j, m, dag)| d(j, m, dag))
    .collect();

    Feestdagenkalender {
        jurisdictie: "NL".into(),
        dekking_van: 2026,
        dekking_tot_en_met: 2030,
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
                "of voor uw entiteitstype een verkorte meldtermijn geldt",
                "het lidnummer waarop de termijn voor de voorafgaande raadpleging berust",
                "de negen criteria voor een effectbeoordeling en de drempel van twee",
                "de zesendertig maanden voor de herbeoordeling van een effectbeoordeling",
                "de beslistermijn en de verdagingstermijn van de Wet open overheid",
                "de opsomming van weigeringsgronden en of die volledig is",
                "de frequentie van de audit en de interne controle onder de Wet politiegegevens",
                "de drempel waarboven een uitzondering van artikel 49 niet meer incidenteel is",
                "de drempel waarboven de meldtermijn van een verwerker te lang is"
            ]
        }),
    );
    uit.insert(
        "dpia_criteria_richtsnoeren".into(),
        serde_json::json!({
            "toelichting": "De criteria uit de richtsnoeren over de effectbeoordeling. Twee of \
                            meer geraakte criteria wijzen op een waarschijnlijk hoog risico. Dat \
                            is een aanwijzing en geen rekensom: één criterium kan volstaan en \
                            twee hoeven niet altijd te betekenen dat de beoordeling verplicht is.",
            "drempel": 2,
            "criteria": [
                { "nummer": 1, "naam": "evaluatie of scoretoekenning",
                  "voorbeeld": "profilering, kredietwaardigheid, gedragsvoorspelling" },
                { "nummer": 2, "naam": "geautomatiseerde besluitvorming met rechtsgevolg",
                  "voorbeeld": "een besluit dat uitsluitend door een systeem wordt genomen" },
                { "nummer": 3, "naam": "stelselmatige monitoring",
                  "voorbeeld": "cameratoezicht in de openbare ruimte, monitoring van medewerkers" },
                { "nummer": 4, "naam": "bijzondere of zeer persoonlijke gegevens",
                  "voorbeeld": "gezondheid, strafrechtelijke gegevens, financiële gegevens" },
                { "nummer": 5, "naam": "grootschalige verwerking",
                  "voorbeeld": "naar aantal betrokkenen, hoeveelheid gegevens, duur of bereik" },
                { "nummer": 6, "naam": "matching of samenvoeging van gegevensverzamelingen",
                  "voorbeeld": "koppeling van bestanden uit verschillende verwerkingen" },
                { "nummer": 7, "naam": "gegevens van kwetsbare betrokkenen",
                  "voorbeeld": "kinderen, patiënten, werknemers, asielzoekers" },
                { "nummer": 8, "naam": "nieuwe technologie of nieuw gebruik",
                  "voorbeeld": "gezichtsherkenning, gecombineerde sensoren" },
                { "nummer": 9, "naam": "het blokkeren van een recht, dienst of overeenkomst",
                  "voorbeeld": "een screening die toegang tot een dienst kan onthouden" }
            ]
        }),
    );
    uit.insert(
        "dpia_inhoudseisen".into(),
        serde_json::json!([
            { "onderdeel": "systematische_beschrijving",
              "bepaling": "art. 35 lid 7 onder a AVG",
              "eis": "een systematische beschrijving van de beoogde verwerkingen en de \
                      verwerkingsdoeleinden, waaronder, in voorkomend geval, het gerechtvaardigde \
                      belang dat door de verwerkingsverantwoordelijke wordt behartigd" },
            { "onderdeel": "noodzaak_en_evenredigheid",
              "bepaling": "art. 35 lid 7 onder b AVG",
              "eis": "een beoordeling van de noodzaak en de evenredigheid van de verwerkingen met \
                      betrekking tot de doeleinden" },
            { "onderdeel": "risicos",
              "bepaling": "art. 35 lid 7 onder c AVG",
              "eis": "een beoordeling van de risico's voor de rechten en vrijheden van betrokkenen" },
            { "onderdeel": "maatregelen",
              "bepaling": "art. 35 lid 7 onder d AVG",
              "eis": "de beoogde maatregelen om de risico's aan te pakken, waaronder waarborgen, \
                      veiligheidsmaatregelen en mechanismen om de bescherming van \
                      persoonsgegevens te verzekeren en aan te tonen dat aan deze verordening is \
                      voldaan" }
        ]),
    );
    uit.insert(
        "verwerker_meldtermijndrempel".into(),
        serde_json::json!({
            "toelichting": "Boven hoeveel uur de contractuele meldtermijn van een verwerker te \
                            lang is. De verordening noemt geen getal: artikel 33 lid 2 zegt dat \
                            de verwerker 'zonder onredelijke vertraging' meldt. Achtenveertig uur \
                            laat van de eigen tweeënzeventig uur nog een dag over om te wegen en \
                            te melden; wie meer weggeeft, geeft zijn eigen termijn weg.",
            "drempel_uren": 48,
            "bron": "art. 33 lid 2 AVG; het getal is niet aan de wettekst ontleend"
        }),
    );
    uit.insert(
        "doorgifte_uitzonderingsdrempel".into(),
        serde_json::json!({
            "toelichting": "Boven hoeveel toepassingen per jaar een uitzondering van artikel 49 \
                            niet meer incidenteel is. De verordening noemt geen getal; dit is een \
                            werkbare grens die past bij het woord 'incidenteel' en die per \
                            organisatie kan worden bijgesteld.",
            "drempel": 2,
            "bron": "art. 49 lid 1 AVG; het getal is niet aan de wettekst ontleend"
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
