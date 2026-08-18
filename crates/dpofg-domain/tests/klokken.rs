//! De vijf klokken die uit één incident ontstaan.
//!
//! Randgevallen T-05, T-06, T-09, T-25, T-26 en T-31 uit het plan, plus de
//! regel dat een verplichting zichtbaar is vóórdat haar klok loopt.

use chrono::{DateTime, Duration, TimeZone, Utc};
use dpofg_domain::{
    incident::Herkomstkanaal,
    klokken::{
        besluit_past_bij_weging, getroffen_door_ankercorrectie, meldklok_vervalt,
        verplichtingen_bij_voortdurend_incident, verplichtingen_uit_incident, Ankertype,
        Verplichtingcode, Zorgplichtcontext,
    },
    Aantasting, Incident, Motivering, Risiconiveau,
};

fn t(dag: u32, uur: u32, minuut: u32) -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 8, dag, uur, minuut, 0).unwrap()
}

fn motivering(tekst: &str) -> Motivering {
    Motivering::nieuw(tekst, "u1", t(18, 10, 0)).unwrap()
}

fn incident() -> Incident {
    let mut i = Incident::nieuw(
        "2026-0041",
        "onbevoegde toegang tot een bestandsserver",
        t(18, 9, 0),
        t(18, 9, 30),
        Herkomstkanaal::InternVastgesteld,
        "u1",
        "u1",
    );
    i.stel_kennisname_vast(t(18, 9, 20), None).unwrap();
    i.aantasting =
        Aantasting { vertrouwelijkheid: true, integriteit: false, beschikbaarheid: false };
    i.categorieen_gegevens = vec!["personeelsdossiers".into()];
    i.aantal_betrokkenen = Some(340);
    i.exfiltratie_uitgesloten = Some(false);
    i
}

fn codes(v: &[dpofg_domain::AfgeleideVerplichting]) -> Vec<&str> {
    v.iter().map(|x| x.code.code()).collect()
}

// --------------------------------------------------------------------------
// Welke klokken ontstaan
// --------------------------------------------------------------------------

#[test]
fn een_gewoon_datalek_levert_twee_verplichtingen() {
    let i = incident();
    let v = verplichtingen_uit_incident(&i, Zorgplichtcontext::niet_van_toepassing());

    assert_eq!(
        codes(&v),
        vec![Verplichtingcode::AVG_MELDING, Verplichtingcode::AVG_INTERN_REGISTER]
    );
}

#[test]
fn een_hoog_risico_voegt_de_mededeling_toe() {
    let mut i = incident();
    i.risiconiveau = Some(Risiconiveau::HoogRisico);
    i.risicoweging = Some(motivering("personeelsdossiers met gezondheidsgegevens buitgemaakt"));

    let v = verplichtingen_uit_incident(&i, Zorgplichtcontext::niet_van_toepassing());
    assert!(codes(&v).contains(&Verplichtingcode::AVG_MEDEDELING));
}

/// De mededeling hangt aan de vaststelling van het hoge risico, niet aan de
/// kennisname. Wie beide op hetzelfde anker zet, laat de tweede klok te vroeg
/// aflopen.
#[test]
fn de_mededeling_hangt_aan_de_vaststelling_niet_aan_de_kennisname() {
    let mut i = incident();
    i.risiconiveau = Some(Risiconiveau::HoogRisico);
    i.risicoweging = Some(Motivering::nieuw(
        "na onderzoek blijkt dat gezondheidsgegevens zijn ingezien",
        "u1",
        t(19, 14, 0),
    )
    .unwrap());

    let v = verplichtingen_uit_incident(&i, Zorgplichtcontext::niet_van_toepassing());
    let mededeling = v
        .iter()
        .find(|x| x.code.code() == Verplichtingcode::AVG_MEDEDELING)
        .unwrap();

    assert_eq!(mededeling.ankertype, Ankertype::VaststellingHoogRisico);
    assert_eq!(mededeling.anker, Some(t(19, 14, 0)));
    assert_ne!(mededeling.anker, i.kennisname_op);
}

/// Ook wanneer er niet wordt gemeld, blijft de interne vastlegging staan — en
/// de reden die daarbij in beeld komt, zegt waaróm dat juist dan telt.
#[test]
fn niet_melden_laat_de_interne_vastlegging_staan() {
    let mut i = incident();
    i.risiconiveau = Some(Risiconiveau::GeenRisico);
    i.risicoweging = Some(motivering("toegang was beperkt tot een lege testmap"));
    i.besluit_niet_melden(
        motivering("de map bevatte geen persoonsgegevens, dit is vastgesteld uit de logging"),
        Some("u2".into()),
        t(18, 11, 0),
        Duration::zero(),
    )
    .unwrap();

    let v = verplichtingen_uit_incident(&i, Zorgplichtcontext::niet_van_toepassing());
    let intern = v
        .iter()
        .find(|x| x.code.code() == Verplichtingcode::AVG_INTERN_REGISTER)
        .unwrap();
    assert!(intern.reden.contains("enige verantwoording die overblijft"), "kreeg: {}", intern.reden);
}

// --------------------------------------------------------------------------
// De zorgplichtketen
// --------------------------------------------------------------------------

#[test]
fn een_significant_incident_levert_vijf_klokken() {
    let mut i = incident();
    i.significant_vastgesteld_op = Some(t(18, 10, 0));
    i.risiconiveau = Some(Risiconiveau::HoogRisico);
    i.risicoweging = Some(motivering("personeelsdossiers met gezondheidsgegevens buitgemaakt"));

    let v = verplichtingen_uit_incident(
        &i,
        Zorgplichtcontext { valt_onder_meldketen: true, is_significant: true },
    );

    assert_eq!(v.len(), 6, "kreeg: {:?}", codes(&v));
    for code in [
        Verplichtingcode::AVG_MELDING,
        Verplichtingcode::AVG_MEDEDELING,
        Verplichtingcode::AVG_INTERN_REGISTER,
        Verplichtingcode::ZORG_WAARSCHUWING,
        Verplichtingcode::ZORG_MELDING,
        Verplichtingcode::ZORG_EINDRAPPORT,
    ] {
        assert!(codes(&v).contains(&code), "{code} ontbreekt");
    }
}

#[test]
fn zonder_significantie_geen_zorgplichtklokken() {
    let i = incident();
    let v = verplichtingen_uit_incident(
        &i,
        Zorgplichtcontext { valt_onder_meldketen: true, is_significant: false },
    );
    assert!(!codes(&v).contains(&Verplichtingcode::ZORG_WAARSCHUWING));
}

/// T-05: het eindrapport hangt aan de verzending van de melding, niet aan het
/// incident. Zolang er niet is gemeld, wacht de klok — en dat is zichtbaar.
#[test]
fn t05_het_eindrapport_wacht_op_de_verzending_van_de_melding() {
    let mut i = incident();
    i.significant_vastgesteld_op = Some(t(18, 10, 0));

    let v = verplichtingen_uit_incident(
        &i,
        Zorgplichtcontext { valt_onder_meldketen: true, is_significant: true },
    );
    let eind = v
        .iter()
        .find(|x| x.code.code() == Verplichtingcode::ZORG_EINDRAPPORT)
        .unwrap();

    assert_eq!(eind.ankertype, Ankertype::VerzendingMelding);
    assert!(eind.wacht_op_anker, "zonder verzending loopt de klok nog niet");
    assert!(eind.anker.is_none());

    // Zodra er is gemeld, gaat de klok lopen vanaf dát moment.
    i.gemeld_op = Some(t(19, 15, 0));
    let v2 = verplichtingen_uit_incident(
        &i,
        Zorgplichtcontext { valt_onder_meldketen: true, is_significant: true },
    );
    let eind2 = v2
        .iter()
        .find(|x| x.code.code() == Verplichtingcode::ZORG_EINDRAPPORT)
        .unwrap();
    assert!(!eind2.wacht_op_anker);
    assert_eq!(eind2.anker, Some(t(19, 15, 0)));
    assert_ne!(eind2.anker, i.kennisname_op, "niet het incident zelf");
}

/// T-06: het incident duurt voort op de eindrapportdatum.
#[test]
fn t06_een_voortdurend_incident_levert_een_voortgangsrapport() {
    let mut i = incident();
    i.gemeld_op = Some(t(19, 15, 0));

    let v = verplichtingen_bij_voortdurend_incident(&i, t(19, 15, 0) + Duration::days(30));
    assert_eq!(codes(&v), vec![Verplichtingcode::ZORG_VOORTGANG, Verplichtingcode::ZORG_EINDRAPPORT]);

    let nieuw_eind = &v[1];
    assert_eq!(nieuw_eind.ankertype, Ankertype::Afhandeling);
    assert!(nieuw_eind.wacht_op_anker);

    // Is het incident wél afgerond, dan ontstaat er niets extra's.
    i.afgehandeld_op = Some(t(20, 12, 0));
    assert!(verplichtingen_bij_voortdurend_incident(&i, t(20, 12, 0)).is_empty());
}

// --------------------------------------------------------------------------
// T-31: het anker bij een melding van een verwerker
// --------------------------------------------------------------------------

#[test]
fn t31_de_klok_hangt_aan_de_ontvangst_van_de_verwerkersmelding() {
    let mut i = Incident::nieuw(
        "2026-0042",
        "incident bij de salarisverwerker",
        t(20, 9, 0),
        t(20, 9, 15),
        Herkomstkanaal::MeldingVanVerwerker,
        "u1",
        "u1",
    );
    i.incident_bij_verwerker_op = Some(t(17, 3, 0));
    i.melding_verwerker_ontvangen_op = Some(t(20, 9, 0));
    i.stel_kennisname_vast(t(20, 9, 5), None).unwrap();

    let v = verplichtingen_uit_incident(&i, Zorgplichtcontext::niet_van_toepassing());
    let melding = v.iter().find(|x| x.code.code() == Verplichtingcode::AVG_MELDING).unwrap();

    assert_eq!(melding.ankertype, Ankertype::OntvangstVerwerkersmelding);
    assert_eq!(melding.anker, Some(t(20, 9, 0)));
    assert!(melding.reden.contains("melding van de verwerker"));
}

// --------------------------------------------------------------------------
// T-09: correctie van het ankertijdstip
// --------------------------------------------------------------------------

#[test]
fn t09_een_ankercorrectie_raakt_alle_klokken_op_dat_anker() {
    let mut i = incident();
    i.significant_vastgesteld_op = Some(t(18, 10, 0));
    let v = verplichtingen_uit_incident(
        &i,
        Zorgplichtcontext { valt_onder_meldketen: true, is_significant: true },
    );

    let op_kennisname = getroffen_door_ankercorrectie(&v, Ankertype::Kennisname);
    assert_eq!(op_kennisname.len(), 2, "melding en interne vastlegging hangen aan de kennisname");

    let op_significantie = getroffen_door_ankercorrectie(&v, Ankertype::VaststellingSignificant);
    assert_eq!(op_significantie.len(), 2, "waarschuwing en incidentmelding");

    // De ankers zijn gescheiden: één correctie sleept niet alles mee.
    assert_ne!(op_kennisname.len() + op_significantie.len(), v.len());
}

// --------------------------------------------------------------------------
// Wanneer de meldklok verdwijnt
// --------------------------------------------------------------------------

/// Een besluit binnen de afkoelperiode laat de klok staan: als het omslaat,
/// moet de oorspronkelijke termijn nog haalbaar zijn.
#[test]
fn de_meldklok_verdwijnt_pas_als_het_besluit_vaststaat() {
    let mut i = incident();
    i.risiconiveau = Some(Risiconiveau::GeenRisico);
    i.risicoweging = Some(motivering("toegang was beperkt tot een lege testmap"));
    i.besluit_niet_melden(
        motivering("de map bevatte geen persoonsgegevens, vastgesteld uit de logging"),
        None,
        t(18, 11, 0),
        Duration::hours(12),
    )
    .unwrap();

    assert!(!meldklok_vervalt(&i.meldbesluit, t(18, 20, 0)), "binnen de afkoelperiode blijft hij staan");
    assert!(meldklok_vervalt(&i.meldbesluit, t(19, 0, 0)), "daarna vervalt hij");
}

#[test]
fn bij_melden_vervalt_de_klok_niet_maar_wordt_hij_gehaald() {
    let mut i = incident();
    i.risiconiveau = Some(Risiconiveau::Risico);
    i.meldbesluit = dpofg_domain::Meldbesluit::Melden { motivering: motivering("er is een risico voor betrokkenen") };
    assert!(!meldklok_vervalt(&i.meldbesluit, t(20, 0, 0)));
}

// --------------------------------------------------------------------------
// Samenhang tussen weging en besluit
// --------------------------------------------------------------------------

#[test]
fn weging_en_besluit_moeten_te_rijmen_zijn() {
    let mut i = incident();

    // Nog geen besluit genomen: er valt niets tegen te spreken.
    assert!(besluit_past_bij_weging(&i), "zonder besluit is er geen tegenstrijdigheid");

    i.risiconiveau = Some(Risiconiveau::GeenRisico);
    assert!(besluit_past_bij_weging(&i));

    i.risicoweging = Some(motivering("toegang was beperkt tot een lege testmap"));
    i.besluit_niet_melden(
        motivering("de map bevatte geen persoonsgegevens, vastgesteld uit de logging"),
        Some("u2".into()),
        t(18, 11, 0),
        Duration::zero(),
    )
    .unwrap();
    assert!(besluit_past_bij_weging(&i));

    // Melden mag altijd, ook bij "geen risico": voorzichtigheid is geen fout.
    i.meldbesluit = dpofg_domain::Meldbesluit::Melden {
        motivering: motivering("bij nader inzien melden wij toch, uit voorzorg"),
    };
    assert!(besluit_past_bij_weging(&i));
}

// --------------------------------------------------------------------------
// De reden staat er altijd bij
// --------------------------------------------------------------------------

/// Elke afgeleide verplichting vertelt welk antwoord haar heeft opgeroepen.
#[test]
fn elke_verplichting_vertelt_waarom_zij_bestaat() {
    let mut i = incident();
    i.significant_vastgesteld_op = Some(t(18, 10, 0));
    i.risiconiveau = Some(Risiconiveau::HoogRisico);
    i.risicoweging = Some(motivering("personeelsdossiers met gezondheidsgegevens buitgemaakt"));

    let v = verplichtingen_uit_incident(
        &i,
        Zorgplichtcontext { valt_onder_meldketen: true, is_significant: true },
    );
    for x in &v {
        assert!(x.reden.len() > 20, "{} heeft een te dunne reden: {}", x.code.code(), x.reden);
        assert!(!x.ankertype.omschrijving().is_empty());
    }
}
