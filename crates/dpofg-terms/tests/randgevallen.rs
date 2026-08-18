//! De verplichte randgevallen uit paragraaf 10.2 van het projectplan.
//!
//! Elk geval draagt zijn nummer uit het plan. Deze tests zijn de bindende
//! specificatie van de termijnenmotor: wie het gedrag wil wijzigen, wijzigt
//! eerst het plan en pas daarna deze bestanden.
//!
//! Gevallen die niet op de rekenkern slaan maar op domeinlogica — welk anker
//! wordt gekozen, welke verplichtingen ontstaan — staan bij de betreffende
//! module en zijn hier met een verwijzing vermeld.

use chrono::{DateTime, NaiveDate, TimeZone, Utc};
use chrono_tz::Tz;
use dpofg_terms::{
    bereken, tijdzone, Aanvang, Eenheid, Feestdagenkalender, LopendeTermijn, Rechtsstelsel,
    Termijnsoort, ToegepasteVerlenging, Verlengingsrecht, TIJDZONE_NL,
};
use std::collections::BTreeSet;

// --------------------------------------------------------------------------
// Hulpstukken
// --------------------------------------------------------------------------

fn zone() -> Tz {
    tijdzone(TIJDZONE_NL).unwrap()
}

/// Feestdagenkalender voor de tests. In het product komt deze uit het
/// kennispakket; hier staat hij expliciet zodat de verwachtingen leesbaar zijn.
fn kalender() -> Feestdagenkalender {
    let dagen: BTreeSet<NaiveDate> = [
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
    .map(|(j, m, d)| NaiveDate::from_ymd_opt(j, m, d).unwrap())
    .collect();

    Feestdagenkalender {
        jurisdictie: "NL".into(),
        dekking_van: 2026,
        dekking_tot_en_met: 2027,
        bron: "testkalender bij de randgevallen".into(),
        dagen,
    }
}

/// Zet een lokaal tijdstip om naar UTC, zoals de gebruiker het invoert.
fn lokaal(j: i32, m: u32, d: u32, u: u32, min: u32) -> DateTime<Utc> {
    zone()
        .with_ymd_and_hms(j, m, d, u, min, 0)
        .single()
        .expect("eenduidig lokaal tijdstip")
        .with_timezone(&Utc)
}

/// De 72-uursmelding van artikel 33 AVG.
fn melding_72u() -> Termijnsoort {
    Termijnsoort::uren("AVG-33-MELDING", "melding datalek aan de toezichthouder", 72, "art. 33 lid 1 AVG")
}

/// De vroegtijdige waarschuwing van 24 uur uit de zorgplichtregelgeving.
fn waarschuwing_24u() -> Termijnsoort {
    Termijnsoort::uren("NIS-24-WAARSCHUWING", "vroegtijdige waarschuwing", 24, "meldketen, eerste bericht")
}

/// De maandtermijn van artikel 12 lid 3 AVG, met verlengingsrecht.
fn verzoek_1maand() -> Termijnsoort {
    Termijnsoort::kalender(
        "AVG-12-3-VERZOEK",
        "afhandeling verzoek van een betrokkene",
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
    })
}

/// De bezwaartermijn van zes weken.
fn bezwaar_6weken() -> Termijnsoort {
    Termijnsoort::kalender(
        "AWB-6-7-BEZWAAR",
        "indienen bezwaarschrift",
        6,
        Eenheid::Weken,
        Rechtsstelsel::NationaalRecht,
        Aanvang::VanafDagNaGebeurtenis,
        "Awb art. 6:7 en 6:8",
    )
}

// --------------------------------------------------------------------------
// T-01 tot en met T-03: urentermijnen
// --------------------------------------------------------------------------

/// T-01 — Kennisname vrijdag 16:40, 72-uursklok.
/// Verwacht: maandag 16:40, kalendertijd, geen verlenging voor het weekend.
#[test]
fn t01_zeventigtwee_uur_loopt_door_het_weekend() {
    let anker = lokaal(2026, 8, 21, 16, 40); // vrijdag
    let d = bereken(&melding_72u(), anker, zone(), &kalender()).unwrap();

    assert_eq!(d.moment, lokaal(2026, 8, 24, 16, 40), "maandag, zelfde tijdstip");
    assert_eq!(d.verlenging, ToegepasteVerlenging::NietVanToepassingBijUren);
    assert!(d.verantwoording.contains("zonder verlenging"));
    assert!(d.lokaal.starts_with("24-08-2026 16:40"));
}

/// T-02 — Kennisname op een feestdag, urentermijn.
/// Verwacht: urentermijnen lopen door feestdagen heen.
#[test]
fn t02_urentermijn_loopt_door_feestdagen() {
    // Eerste kerstdag 2026 valt op een vrijdag; 26 december is zaterdag,
    // 27 december zondag.
    let anker = lokaal(2026, 12, 24, 15, 0);
    let d = bereken(&melding_72u(), anker, zone(), &kalender()).unwrap();
    assert_eq!(d.moment, lokaal(2026, 12, 27, 15, 0));
}

/// T-03 — Overgang naar zomertijd binnen het venster.
/// Verwacht: berekend in UTC; het venster blijft exact 72 uur, ook al staat de
/// lokale klok er een uur naast.
#[test]
fn t03_zomertijd_verandert_de_duur_niet() {
    // In 2026 gaat de zomertijd in op zondag 29 maart om 02:00 lokale tijd.
    let anker = lokaal(2026, 3, 27, 10, 0); // vrijdag, wintertijd (UTC+1)
    let d = bereken(&melding_72u(), anker, zone(), &kalender()).unwrap();

    // Exact 72 uur later in absolute tijd.
    assert_eq!(d.moment - anker, chrono::Duration::hours(72));
    // Op de lokale klok staat het 11:00 in plaats van 10:00: de klok is een uur
    // vooruit gezet, terwijl er in absolute tijd precies 72 uur is verstreken.
    // Wie in lokale tijd had gerekend, was een uur te laat geweest.
    assert!(d.lokaal.starts_with("30-03-2026 11:00"), "kreeg: {}", d.lokaal);
}

/// Spiegelbeeld van T-03: de overgang naar wintertijd.
#[test]
fn t03b_wintertijd_verandert_de_duur_evenmin() {
    // In 2026 eindigt de zomertijd op zondag 25 oktober om 03:00 lokale tijd.
    let anker = lokaal(2026, 10, 23, 10, 0);
    let d = bereken(&melding_72u(), anker, zone(), &kalender()).unwrap();
    assert_eq!(d.moment - anker, chrono::Duration::hours(72));
    assert!(d.lokaal.starts_with("26-10-2026 09:00"), "kreeg: {}", d.lokaal);
}

/// T-07 — Entiteitstype met een verkorte meldtermijn.
/// Verwacht: de klok is 24 uur, niet 72; de waarde komt uit het kennispakket.
#[test]
fn t07_verkorte_meldtermijn() {
    let anker = lokaal(2026, 8, 18, 10, 0);
    let d = bereken(&waarschuwing_24u(), anker, zone(), &kalender()).unwrap();
    assert_eq!(d.moment, lokaal(2026, 8, 19, 10, 0));
    assert_eq!(d.duur, "24 uur");
}

/// T-25 — Vroegtijdige waarschuwing en melding vallen samen op 24 uur.
/// Verwacht: twee verplichtingen op hetzelfde moment; de motor levert ze apart
/// zodat de interface één klok met twee verplichtingen kan tonen.
#[test]
fn t25_twee_verplichtingen_op_hetzelfde_moment() {
    let anker = lokaal(2026, 8, 18, 10, 0);
    let waarschuwing = bereken(&waarschuwing_24u(), anker, zone(), &kalender()).unwrap();
    let verkorte_melding =
        Termijnsoort::uren("NIS-24-MELDING", "incidentmelding, verkorte termijn", 24, "meldketen");
    let melding = bereken(&verkorte_melding, anker, zone(), &kalender()).unwrap();

    assert_eq!(waarschuwing.moment, melding.moment);
    assert_ne!(waarschuwing.code, melding.code, "twee verplichtingen, geen dubbeltelling");
}

// --------------------------------------------------------------------------
// T-21 tot en met T-23: kalendertermijnen
// --------------------------------------------------------------------------

/// T-21 — Inzageverzoek ontvangen 15 januari, maandtermijn.
/// Verwacht: 15 februari; valt die op zaterdag, zondag of feestdag, dan de
/// eerstvolgende werkdag.
#[test]
fn t21_maandtermijn_met_verlenging_naar_werkdag() {
    let anker = lokaal(2026, 1, 15, 9, 30); // donderdag
    let d = bereken(&verzoek_1maand(), anker, zone(), &kalender()).unwrap();

    // 15 februari 2026 is een zondag, dus de termijn schuift naar maandag.
    assert!(d.lokaal.starts_with("16-02-2026 23:59:59"), "kreeg: {}", d.lokaal);
    assert_eq!(
        d.verlenging,
        ToegepasteVerlenging::NaarEerstvolgendeWerkdag {
            van: "2026-02-15".into(),
            naar: "2026-02-16".into()
        }
    );
    assert!(d.verantwoording.contains("Algemene termijnenwet") || d.verantwoording.contains("1182/71"));
}

/// T-21 zonder verlenging: een maandtermijn die op een werkdag eindigt.
#[test]
fn t21b_maandtermijn_zonder_verlenging() {
    let anker = lokaal(2026, 1, 12, 9, 30); // maandag
    let d = bereken(&verzoek_1maand(), anker, zone(), &kalender()).unwrap();
    // 12 februari 2026 is een donderdag.
    assert!(d.lokaal.starts_with("12-02-2026 23:59:59"), "kreeg: {}", d.lokaal);
    assert_eq!(d.verlenging, ToegepasteVerlenging::GeenNodig);
}

/// T-22 — Inzageverzoek ontvangen 31 januari.
/// Verwacht: 28 februari, respectievelijk 29 februari in een schrikkeljaar.
#[test]
fn t22_maandeindeklem() {
    let anker = lokaal(2026, 1, 31, 14, 0); // zaterdag
    let d = bereken(&verzoek_1maand(), anker, zone(), &kalender()).unwrap();
    // 28 februari 2026 is een zaterdag, dus verlengd naar maandag 2 maart.
    assert!(d.lokaal.starts_with("02-03-2026 23:59:59"), "kreeg: {}", d.lokaal);
    assert_eq!(
        d.verlenging,
        ToegepasteVerlenging::NaarEerstvolgendeWerkdag {
            van: "2026-02-28".into(),
            naar: "2026-03-02".into()
        }
    );
}

/// T-23 — Tweewekentermijn die eindigt op een feestdag wordt verlengd; de
/// urentermijnen niet.
#[test]
fn t23_dagtermijn_verlengt_urentermijn_niet() {
    let registratiewijziging = Termijnsoort::kalender(
        "REG-WIJZIGING",
        "melden registratiewijziging",
        2,
        Eenheid::Weken,
        Rechtsstelsel::NationaalRecht,
        Aanvang::VanafGebeurtenis,
        "registratieplicht",
    );
    // 11 december 2026 + 2 weken = 25 december, eerste kerstdag.
    let anker = lokaal(2026, 12, 11, 10, 0);
    let d = bereken(&registratiewijziging, anker, zone(), &kalender()).unwrap();
    assert!(d.lokaal.starts_with("28-12-2026 23:59:59"), "kreeg: {}", d.lokaal);
    assert!(matches!(d.verlenging, ToegepasteVerlenging::NaarEerstvolgendeWerkdag { .. }));

    // Dezelfde ankerdatum met een urentermijn: geen verlenging.
    let uren = bereken(&melding_72u(), lokaal(2026, 12, 24, 10, 0), zone(), &kalender()).unwrap();
    assert_eq!(uren.moment, lokaal(2026, 12, 27, 10, 0));
    assert_eq!(uren.verlenging, ToegepasteVerlenging::NietVanToepassingBijUren);
}

/// T-28 — Besluit bekendgemaakt op 3 september; bezwaartermijn zes weken vanaf
/// de dag ná bekendmaking.
#[test]
fn t28_bezwaartermijn_vangt_aan_de_dag_na_bekendmaking() {
    let anker = lokaal(2026, 9, 3, 11, 0); // donderdag
    let d = bereken(&bezwaar_6weken(), anker, zone(), &kalender()).unwrap();
    // Aanvang 4 september; zes weken later eindigt de termijn op 15 oktober.
    assert!(d.lokaal.starts_with("15-10-2026 23:59:59"), "kreeg: {}", d.lokaal);
    assert_eq!(d.verlenging, ToegepasteVerlenging::GeenNodig);
    assert!(d.verantwoording.contains("dag ná de gebeurtenis"));
}

/// Beide formuleringen van de aanvang leveren dezelfde laatste dag op.
///
/// Dit is een controle op de rekenkern zelf: "vanaf de gebeurtenis, N eenheden"
/// en "vanaf de dag erna, N eenheden, laatste dag" zijn twee manieren om
/// hetzelfde te zeggen. Wijkt dit af, dan zit er een telfout in.
#[test]
fn beide_aanvangsformuleringen_vallen_samen() {
    let anker = lokaal(2026, 9, 3, 11, 0);
    for (duur, eenheid) in
        [(6u32, Eenheid::Weken), (1, Eenheid::Maanden), (10, Eenheid::Kalenderdagen), (1, Eenheid::Jaren)]
    {
        let a = Termijnsoort::kalender(
            "A", "a", duur, eenheid, Rechtsstelsel::Unierecht, Aanvang::VanafGebeurtenis, "x",
        );
        let b = Termijnsoort::kalender(
            "B", "b", duur, eenheid, Rechtsstelsel::NationaalRecht,
            Aanvang::VanafDagNaGebeurtenis, "x",
        );
        let da = bereken(&a, anker, zone(), &kalender()).unwrap();
        let db = bereken(&b, anker, zone(), &kalender()).unwrap();
        assert_eq!(da.moment, db.moment, "afwijking bij {duur} {eenheid:?}");
    }
}

// --------------------------------------------------------------------------
// T-12: verlenging
// --------------------------------------------------------------------------

/// T-12 — Inzageverzoek ontvangen 15 januari, verlenging medegedeeld 20 februari.
/// Verwacht: geweigerd. De mededeling moest binnen de eerste maand, dus uiterlijk
/// 15 februari of de eerstvolgende werkdag.
#[test]
fn t12_te_late_verlenging_wordt_geweigerd() {
    let anker = lokaal(2026, 1, 15, 9, 30);
    let mut termijn =
        LopendeTermijn::start(verzoek_1maand(), anker, zone(), &kalender()).unwrap();

    let fout = termijn.verleng(lokaal(2026, 2, 20, 9, 0), zone(), &kalender()).unwrap_err();
    let tekst = fout.to_string();
    assert!(tekst.contains("binnen de oorspronkelijke termijn"), "kreeg: {tekst}");
    assert!(tekst.contains("16-02-2026"), "de uiterste datum hoort in de melding: {tekst}");
    assert_eq!(termijn.keer_verlengd, 0);
}

/// T-12 spiegelbeeld: op tijd verlengen mag wél en levert twee maanden extra.
#[test]
fn t12b_tijdige_verlenging_wordt_toegekend() {
    let anker = lokaal(2026, 1, 15, 9, 30);
    let mut termijn =
        LopendeTermijn::start(verzoek_1maand(), anker, zone(), &kalender()).unwrap();

    termijn.verleng(lokaal(2026, 2, 10, 9, 0), zone(), &kalender()).unwrap();
    assert_eq!(termijn.keer_verlengd, 1);

    let nieuw = termijn.verlengd_met.as_ref().unwrap();
    // 16 februari plus twee maanden is 16 april 2026, een donderdag.
    assert!(nieuw.lokaal.starts_with("16-04-2026"), "kreeg: {}", nieuw.lokaal);
}

/// Verlengen mag niet vaker dan de wet toestaat.
#[test]
fn verlengen_kan_niet_twee_keer() {
    let anker = lokaal(2026, 1, 15, 9, 30);
    let mut termijn =
        LopendeTermijn::start(verzoek_1maand(), anker, zone(), &kalender()).unwrap();
    termijn.verleng(lokaal(2026, 2, 10, 9, 0), zone(), &kalender()).unwrap();
    let fout = termijn.verleng(lokaal(2026, 2, 11, 9, 0), zone(), &kalender()).unwrap_err();
    assert!(fout.to_string().contains("maximum"));
}

/// Een termijn zonder verlengingsrecht kan niet worden verlengd.
#[test]
fn urentermijn_kent_geen_verlenging() {
    let anker = lokaal(2026, 8, 18, 10, 0);
    let mut termijn = LopendeTermijn::start(melding_72u(), anker, zone(), &kalender()).unwrap();
    assert!(termijn.verleng(anker, zone(), &kalender()).is_err());
}

// --------------------------------------------------------------------------
// T-27: opschorting
// --------------------------------------------------------------------------

/// T-27 — Raadpleging ingediend, aanvullende informatie opgevraagd op dag 20.
/// Verwacht: de termijn wordt opgeschort tot ontvangst; verlenging is apart
/// zichtbaar en apart te motiveren.
#[test]
fn t27_opschorting_schuift_de_deadline_op() {
    let raadpleging = Termijnsoort::kalender(
        "AVG-36-RAADPLEGING",
        "voorafgaande raadpleging",
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
    });

    let anker = lokaal(2026, 3, 2, 9, 0); // maandag
    let mut termijn = LopendeTermijn::start(raadpleging, anker, zone(), &kalender()).unwrap();
    let zonder_opschorting = termijn.deadline(anker, zone(), &kalender()).unwrap();

    // Op dag 20 wordt aanvullende informatie gevraagd; twee weken later komt zij binnen.
    // De opschorting loopt dwars door de overgang naar zomertijd op 29 maart.
    termijn
        .schort_op(lokaal(2026, 3, 22, 9, 0), "aanvullende informatie opgevraagd", "u1")
        .unwrap();
    termijn.hervat(lokaal(2026, 4, 5, 9, 0)).unwrap();

    let nu = lokaal(2026, 4, 6, 9, 0);
    let met_opschorting = termijn.deadline(nu, zone(), &kalender()).unwrap();

    // Veertien kalenderdagen later, en nog steeds aan het einde van een dag.
    // In absolute tijd is dat 335 uur en niet 336, want de klok ging vooruit.
    assert_eq!(
        met_opschorting.with_timezone(&zone()).date_naive()
            - zonder_opschorting.with_timezone(&zone()).date_naive(),
        chrono::Duration::days(14),
        "de klok stond veertien kalenderdagen stil"
    );
    assert!(
        met_opschorting
            .with_timezone(&zone())
            .format("%H:%M:%S")
            .to_string()
            .starts_with("23:59:59"),
        "een kalendertermijn eindigt aan het einde van een dag, kreeg: {}",
        met_opschorting.with_timezone(&zone())
    );
    assert_eq!(termijn.opschortingen.len(), 1);
    assert_eq!(termijn.opschortingen[0].grond, "aanvullende informatie opgevraagd");

    // De verlenging telt daar bovenop en is apart zichtbaar.
    termijn.verleng(nu, zone(), &kalender()).unwrap();
    assert_eq!(termijn.keer_verlengd, 1);
    assert!(termijn.verlengd_met.is_some());
}

/// Een 72-uurstermijn kan niet worden opgeschort. Dat is geen instelling.
#[test]
fn zeventigtwee_uur_is_niet_opschortbaar() {
    let anker = lokaal(2026, 8, 18, 10, 0);
    let mut termijn = LopendeTermijn::start(melding_72u(), anker, zone(), &kalender()).unwrap();
    let fout = termijn.schort_op(anker, "onderzoek loopt nog", "u1").unwrap_err();
    assert!(fout.to_string().contains("niet opschortbaar"));
}

/// Opschorten zonder grond wordt geweigerd.
#[test]
fn opschorting_zonder_grond_wordt_geweigerd() {
    let soort = verzoek_1maand().opschortbaar();
    let anker = lokaal(2026, 1, 15, 9, 30);
    let mut termijn = LopendeTermijn::start(soort, anker, zone(), &kalender()).unwrap();
    assert!(termijn.schort_op(anker, "   ", "u1").is_err());
}

/// Twee opschortingen tegelijk kan niet.
#[test]
fn dubbele_opschorting_wordt_geweigerd() {
    let soort = verzoek_1maand().opschortbaar();
    let anker = lokaal(2026, 1, 15, 9, 30);
    let mut termijn = LopendeTermijn::start(soort, anker, zone(), &kalender()).unwrap();
    termijn.schort_op(lokaal(2026, 1, 20, 9, 0), "identiteit onduidelijk", "u1").unwrap();
    assert!(termijn.schort_op(lokaal(2026, 1, 22, 9, 0), "nog iets", "u1").is_err());
}

// --------------------------------------------------------------------------
// Algemene eisen aan de motor
// --------------------------------------------------------------------------

/// Eis 5 van het termijnrekenkundig uitgangspunt: elke deadline draagt haar
/// verantwoording mee.
#[test]
fn elke_deadline_draagt_haar_verantwoording() {
    let gevallen = [melding_72u(), verzoek_1maand(), bezwaar_6weken(), waarschuwing_24u()];
    let anker = lokaal(2026, 8, 18, 10, 0);
    for soort in gevallen {
        let d = bereken(&soort, anker, zone(), &kalender()).unwrap();
        assert!(!d.verantwoording.is_empty(), "{} mist verantwoording", soort.code);
        assert!(
            d.verantwoording.contains(&soort.grondslag),
            "{} noemt zijn grondslag niet",
            soort.code
        );
        assert!(!d.verlengingsbepaling.is_empty(), "{} mist verlengingsbepaling", soort.code);
        assert!(!d.lokaal.is_empty());
        assert_eq!(d.tijdzone, TIJDZONE_NL);
    }
}

/// Eis 4: valt een berekening buiten het dekkingsvenster van de kalender, dan
/// faalt zij zichtbaar in plaats van te gokken.
#[test]
fn buiten_het_dekkingsvenster_wordt_niet_gegokt() {
    let smalle_kalender = Feestdagenkalender::leeg("NL", 2026, 2026);
    let maandtermijn = verzoek_1maand();

    // Binnen de dekking: gewoon rekenen.
    assert!(bereken(&maandtermijn, lokaal(2026, 3, 2, 9, 0), zone(), &smalle_kalender).is_ok());

    // Over de grens heen: de einddag valt in 2027 en dat jaar kent de kalender
    // niet. De motor weet dan niet of 2 januari 2027 een feestdag is en mag
    // geen deadline afgeven.
    let uitkomst = bereken(&maandtermijn, lokaal(2026, 12, 2, 9, 0), zone(), &smalle_kalender);
    assert!(uitkomst.is_err(), "buiten dekking hoort te falen");
    assert!(uitkomst.unwrap_err().to_string().contains("kennispakket"));
}

/// Een termijn met duur nul bestaat niet.
#[test]
fn duur_nul_wordt_geweigerd() {
    let leeg = Termijnsoort::uren("LEEG", "leeg", 0, "geen");
    assert!(bereken(&leeg, lokaal(2026, 8, 18, 10, 0), zone(), &kalender()).is_err());
}

/// Het halen van een termijn is vast te stellen.
#[test]
fn gehaald_en_niet_gehaald() {
    let anker = lokaal(2026, 8, 18, 10, 0);

    let mut op_tijd = LopendeTermijn::start(melding_72u(), anker, zone(), &kalender()).unwrap();
    op_tijd.rond_af(lokaal(2026, 8, 20, 9, 0));
    assert_eq!(op_tijd.is_gehaald(zone(), &kalender()).unwrap(), Some(true));

    let mut te_laat = LopendeTermijn::start(melding_72u(), anker, zone(), &kalender()).unwrap();
    te_laat.rond_af(lokaal(2026, 8, 22, 9, 0));
    assert_eq!(te_laat.is_gehaald(zone(), &kalender()).unwrap(), Some(false));

    let loopt_nog = LopendeTermijn::start(melding_72u(), anker, zone(), &kalender()).unwrap();
    assert_eq!(loopt_nog.is_gehaald(zone(), &kalender()).unwrap(), None);
}
