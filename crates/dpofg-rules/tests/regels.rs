//! De controleregels die over de samenhang waken.

use chrono::{DateTime, Duration, TimeZone, Utc};
use dpofg_domain::{
    avg::{BijzondereCategorie, Grondslag, Rol},
    incident::Herkomstkanaal,
    Aantasting, Bewaartermijn, Id, Incident, Motivering, Ontvanger, Overgenomen, Risiconiveau,
    Termijneenheid, Verwerking,
};
use dpofg_rules::{
    motor::{Niveau, Ontvangerrol},
    regels::{
        beoordeel_incident, beoordeel_oorzaakpatroon, beoordeel_verwerking, catalogus,
        geimplementeerd, standaardmotor,
    },
};

fn nu() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 8, 18, 9, 0, 0).unwrap()
}

fn motivering(tekst: &str) -> Motivering {
    Motivering::nieuw(tekst, "u1", nu()).unwrap()
}

fn verwerking() -> Verwerking {
    let mut v = Verwerking::nieuw(
        "0412-K",
        "Verzuimregistratie",
        Rol::Verwerkingsverantwoordelijke,
        "afdeling P&O",
        "u1",
        nu(),
    );
    v.doeleinden = vec!["loondoorbetaling bij ziekte".into()];
    v.categorieen_betrokkenen = vec!["medewerkers".into()];
    v.categorieen_gegevens = vec!["naam".into()];
    v.ontvangers = vec![Ontvanger {
        omschrijving: "leidinggevende".into(),
        is_verwerker: false,
        leverancier_id: None,
        buiten_eer: false,
    }];
    v.bewaartermijn = Some(Bewaartermijn::Vast {
        duur: 2,
        eenheid: Termijneenheid::Jaren,
        grondslag: "art. 52 AWR".into(),
        vanaf: "einde dienstverband".into(),
    });
    v.beveiligingsmaatregelen = Some("toegang op rolbasis".into());
    v.grondslag = Some(Grondslag::WettelijkeVerplichting);
    v.wettelijke_bepaling = Some("art. 7:629 BW".into());
    v.grondslag_motivering = Some(motivering("wettelijke loondoorbetalingsplicht bij ziekte"));
    v
}

fn incident() -> Incident {
    let mut i = Incident::nieuw(
        "2026-0041",
        "verkeerd geadresseerde brief",
        nu(),
        nu(),
        Herkomstkanaal::InternVastgesteld,
        "u1",
        "u1",
    );
    i.stel_kennisname_vast(nu(), None).unwrap();
    i.aantasting =
        Aantasting { vertrouwelijkheid: true, integriteit: false, beschikbaarheid: false };
    i.categorieen_gegevens = vec!["naam".into()];
    i.aantal_betrokkenen = Some(1);
    i.exfiltratie_uitgesloten = Some(true);
    i.getroffen_verwerkingen.push(Id::nieuw());
    i
}

fn codes(b: &[dpofg_rules::Bevinding]) -> Vec<&str> {
    b.iter().map(|x| x.regelcode.as_str()).collect()
}

// --------------------------------------------------------------------------
// De catalogus
// --------------------------------------------------------------------------

#[test]
fn de_catalogus_is_samenhangend() {
    let motor = standaardmotor();
    assert!(motor.aantal() >= 50, "de catalogus telt {} regels", motor.aantal());

    let mut gezien = std::collections::BTreeSet::new();
    for r in motor.alle() {
        assert!(gezien.insert(r.code.clone()), "dubbele code: {}", r.code);
        assert!(!r.naam.is_empty(), "{} mist een naam", r.code);
        assert!(!r.controleert.is_empty(), "{} beschrijft niet wat hij controleert", r.code);
        assert!(!r.grondslag.is_empty(), "{} mist een grondslag", r.code);
        assert!(r.controleert.len() > 20, "{} is te dun omschreven", r.code);
    }
}

#[test]
fn de_groepen_zijn_herkenbaar() {
    let motor = standaardmotor();
    let groepen: Vec<String> = motor.groepen().into_iter().map(|(g, _)| g).collect();
    for verwacht in [
        "register",
        "grondslag",
        "bewaartermijn",
        "verwerkers",
        "doorgifte",
        "datalekken",
        "effectbeoordeling",
        "organisatie",
        "toepassing",
    ] {
        assert!(groepen.contains(&verwacht.to_string()), "groep {verwacht} ontbreekt");
    }
}

/// Het aantal regels in de catalogus mag geen dekking suggereren die er niet is.
#[test]
fn de_werkelijke_dekking_is_opvraagbaar() {
    let motor = standaardmotor();
    let zonder = motor.regels_zonder_evaluatie();

    assert!(!zonder.is_empty(), "er staan regels in de catalogus die nog niet draaien");
    assert!(motor.dekking() > 0.0 && motor.dekking() < 1.0);

    // Elke code die als geïmplementeerd staat aangemerkt, bestaat ook echt.
    for code in geimplementeerd() {
        assert!(motor.regel(code).is_some(), "{code} staat als geïmplementeerd maar bestaat niet");
    }
}

/// Blokkerende regels zijn in de minderheid. Wie alles blokkeert, leert mensen
/// wegklikken.
#[test]
fn blokkerende_regels_zijn_in_de_minderheid() {
    let alle = catalogus();
    let blokkerend = alle.iter().filter(|r| r.niveau == Niveau::Blokkerend).count();
    assert!(
        blokkerend * 2 < alle.len(),
        "{blokkerend} van de {} regels blokkeert; dat is te veel",
        alle.len()
    );
}

/// Elke regel heeft een ontvanger. Een bevinding zonder ontvanger belandt op de
/// stapel van de functionaris.
#[test]
fn niet_alles_gaat_naar_de_functionaris() {
    let alle = catalogus();
    let naar_fg = alle.iter().filter(|r| r.ontvanger == Ontvangerrol::Functionaris).count();
    assert!(
        naar_fg < alle.len(),
        "alle {naar_fg} regels gaan naar de functionaris; er is geen routering"
    );
    let rollen: std::collections::BTreeSet<_> = alle.iter().map(|r| r.ontvanger).collect();
    assert!(rollen.len() >= 5, "er worden maar {} rollen gebruikt", rollen.len());
}

// --------------------------------------------------------------------------
// De verwerking
// --------------------------------------------------------------------------

#[test]
fn een_volledige_verwerking_levert_geen_bevindingen() {
    let motor = standaardmotor();
    let mut v = verwerking();
    v.stel_vast("u2", nu()).unwrap();
    let b = beoordeel_verwerking(&motor, &v, nu());
    assert!(b.is_empty(), "kreeg: {:?}", codes(&b));
}

#[test]
fn een_vastgestelde_maar_onvolledige_regel_slaat_aan() {
    let motor = standaardmotor();
    let mut v = verwerking();
    v.stel_vast("u2", nu()).unwrap();
    // Na vaststelling verdwijnt de bewaartermijn, bijvoorbeeld door een import.
    v.bewaartermijn = None;

    let b = beoordeel_verwerking(&motor, &v, nu());
    assert!(codes(&b).contains(&"REG-01"));
    assert!(codes(&b).contains(&"BEW-01"));

    let reg01 = b.iter().find(|x| x.regelcode == "REG-01").unwrap();
    // Signalerend en niet blokkerend: de regel is al vastgesteld, dus er valt
    // niets meer tegen te houden. Wat er nog aan blokkeert, doet BEW-01.
    assert_eq!(reg01.niveau, Niveau::Signalerend);
    assert_eq!(
        b.iter().find(|x| x.regelcode == "BEW-01").unwrap().niveau,
        Niveau::Blokkerend
    );
    assert!(reg01.toelichting.contains("bewaartermijn"));
    assert_eq!(reg01.record_kenmerk.as_deref(), Some("0412-K"));
}

#[test]
fn een_register_dat_niet_wordt_herzien_slaat_aan() {
    let motor = standaardmotor();
    let mut v = verwerking();
    v.stel_vast("u2", nu()).unwrap();

    let veertien_maanden_later = Utc.with_ymd_and_hms(2027, 10, 18, 9, 0, 0).unwrap();
    let b = beoordeel_verwerking(&motor, &v, veertien_maanden_later);
    assert!(codes(&b).contains(&"REG-02"));

    let reg02 = b.iter().find(|x| x.regelcode == "REG-02").unwrap();
    assert_eq!(reg02.niveau, Niveau::Signalerend, "dit blokkeert niet, het signaleert");
}

#[test]
fn een_overgenomen_regel_blijft_zichtbaar() {
    let motor = standaardmotor();
    let mut v = verwerking();
    v.overgenomen = Some(Overgenomen {
        bron: "werkblad register 2024".into(),
        overgenomen_op: nu(),
        geverifieerd_op: None,
        geverifieerd_door: None,
    });
    let b = beoordeel_verwerking(&motor, &v, nu());
    assert!(codes(&b).contains(&"REG-03"));
    assert!(b.iter().find(|x| x.regelcode == "REG-03").unwrap().toelichting.contains("werkblad"));
}

#[test]
fn een_concept_dat_blijft_liggen_slaat_aan() {
    let motor = standaardmotor();
    let v = verwerking();
    let honderd_dagen_later = nu() + Duration::days(100);
    let b = beoordeel_verwerking(&motor, &v, honderd_dagen_later);
    assert!(codes(&b).contains(&"REG-04"));
}

#[test]
fn een_verwerker_zonder_overeenkomst_blokkeert() {
    let motor = standaardmotor();
    let mut v = verwerking();
    v.ontvangers.push(Ontvanger {
        omschrijving: "arbodienst".into(),
        is_verwerker: true,
        leverancier_id: None,
        buiten_eer: false,
    });
    let b = beoordeel_verwerking(&motor, &v, nu());
    let vwo = b.iter().find(|x| x.regelcode == "VWO-01").expect("VWO-01 ontbreekt");
    assert_eq!(vwo.niveau, Niveau::Blokkerend);
    assert_eq!(vwo.grondslag, "art. 28 lid 3 AVG");
}

#[test]
fn een_doorgifte_zonder_waarborg_blokkeert() {
    let motor = standaardmotor();
    let mut v = verwerking();
    v.ontvangers.push(Ontvanger {
        omschrijving: "analysedienst".into(),
        is_verwerker: false,
        leverancier_id: None,
        buiten_eer: true,
    });
    let b = beoordeel_verwerking(&motor, &v, nu());
    assert!(codes(&b).contains(&"EER-01"));
}

#[test]
fn een_gerechtvaardigd_belang_zonder_afweging_blokkeert() {
    let motor = standaardmotor();
    let mut v = verwerking();
    v.grondslag = Some(Grondslag::GerechtvaardigdBelang);
    v.wettelijke_bepaling = None;
    let b = beoordeel_verwerking(&motor, &v, nu());
    assert!(codes(&b).contains(&"GRO-01"));
    assert_eq!(b.iter().find(|x| x.regelcode == "GRO-01").unwrap().niveau, Niveau::Blokkerend);
}

#[test]
fn bijzondere_gegevens_zonder_uitzondering_blokkeren() {
    let motor = standaardmotor();
    let mut v = verwerking();
    v.bijzondere_categorieen.push(BijzondereCategorie::Gezondheidsgegevens);
    let b = beoordeel_verwerking(&motor, &v, nu());
    assert!(codes(&b).contains(&"GRO-03"));
}

#[test]
fn een_verlopen_uitstelafspraak_slaat_aan() {
    let motor = standaardmotor();
    let mut v = verwerking();
    v.bewaartermijn = Some(Bewaartermijn::NogTeBepalen {
        motivering: motivering("de selectielijst wordt in het vierde kwartaal herzien"),
        uiterlijk_bepaald_op: Utc.with_ymd_and_hms(2026, 6, 30, 23, 59, 59).unwrap(),
        eigenaar: "recordmanager".into(),
    });
    let b = beoordeel_verwerking(&motor, &v, nu());
    let bew = b.iter().find(|x| x.regelcode == "BEW-02").expect("BEW-02 ontbreekt");
    assert!(bew.toelichting.contains("recordmanager"));
    assert!(bew.toelichting.contains("30-06-2026"));
}

// --------------------------------------------------------------------------
// Het incident
// --------------------------------------------------------------------------

#[test]
fn een_net_geregistreerd_incident_slaat_weinig_aan() {
    let motor = standaardmotor();
    let i = incident();
    let b = beoordeel_incident(&motor, &i, nu());
    assert!(b.is_empty(), "kreeg: {:?}", codes(&b));
}

#[test]
fn een_incident_zonder_beoordeling_slaat_na_twaalf_uur_aan() {
    let motor = standaardmotor();
    let i = incident();
    let b = beoordeel_incident(&motor, &i, nu() + Duration::hours(13));
    assert!(codes(&b).contains(&"LEK-01"));
    assert!(b.iter().find(|x| x.regelcode == "LEK-01").unwrap().toelichting.contains("13 uur"));
}

#[test]
fn een_te_laat_geregistreerd_incident_blokkeert() {
    let motor = standaardmotor();
    let mut i = Incident::nieuw(
        "2026-0042",
        "laat geregistreerd",
        Utc.with_ymd_and_hms(2026, 8, 18, 8, 0, 0).unwrap(),
        Utc.with_ymd_and_hms(2026, 8, 18, 20, 0, 0).unwrap(),
        Herkomstkanaal::InternVastgesteld,
        "u1",
        "u1",
    );
    i.stel_kennisname_vast(Utc.with_ymd_and_hms(2026, 8, 18, 9, 0, 0).unwrap(), None).unwrap();
    i.aantasting =
        Aantasting { vertrouwelijkheid: true, integriteit: false, beschikbaarheid: false };
    i.getroffen_verwerkingen.push(Id::nieuw());

    let b = beoordeel_incident(&motor, &i, Utc.with_ymd_and_hms(2026, 8, 18, 21, 0, 0).unwrap());
    let lek03 = b.iter().find(|x| x.regelcode == "LEK-03").expect("LEK-03 ontbreekt");
    assert_eq!(lek03.niveau, Niveau::Blokkerend);
    assert!(lek03.toelichting.contains("11 uur"));
}

#[test]
fn geen_risico_bij_grote_omvang_slaat_aan() {
    let motor = standaardmotor();
    let mut i = incident();
    i.aantal_betrokkenen = Some(4000);
    i.risiconiveau = Some(Risiconiveau::GeenRisico);
    let b = beoordeel_incident(&motor, &i, nu());
    let lek06 = b.iter().find(|x| x.regelcode == "LEK-06").expect("LEK-06 ontbreekt");
    assert_eq!(lek06.niveau, Niveau::Signalerend);
    assert!(lek06.toelichting.contains("4000"));
}

#[test]
fn een_incident_zonder_beoordeelde_aantasting_blokkeert() {
    let motor = standaardmotor();
    let mut i = incident();
    i.aantasting =
        Aantasting { vertrouwelijkheid: false, integriteit: false, beschikbaarheid: false };
    let b = beoordeel_incident(&motor, &i, nu());
    assert!(codes(&b).contains(&"LEK-09"));
}

#[test]
fn een_incident_zonder_registerkoppeling_signaleert() {
    let motor = standaardmotor();
    let mut i = incident();
    i.getroffen_verwerkingen.clear();
    let b = beoordeel_incident(&motor, &i, nu());
    assert!(codes(&b).contains(&"LEK-15"));
}

#[test]
fn afsluiten_zonder_oorzaak_of_maatregel_blokkeert() {
    let motor = standaardmotor();
    let mut i = incident();
    i.afgehandeld_op = Some(nu());
    let b = beoordeel_incident(&motor, &i, nu());
    let lek12: Vec<_> = b.iter().filter(|x| x.regelcode == "LEK-12").collect();
    assert_eq!(lek12.len(), 2, "zowel de oorzaak als de maatregel ontbreekt");
}

/// LEK-13 kijkt over incidenten heen; dat is precies waarom deze regels los
/// staan van de volledigheidscontrole per record.
#[test]
fn een_herhaalde_oorzaak_wordt_gezien() {
    let motor = standaardmotor();
    let kwartaalgrens = Utc.with_ymd_and_hms(2026, 7, 1, 0, 0, 0).unwrap();

    let mut incidenten = Vec::new();
    for n in 1..=4 {
        let mut i = incident();
        i.kenmerk = format!("2026-004{n}");
        i.oorzaakcategorie = Some("verzending naar een verkeerde geadresseerde".into());
        incidenten.push(i);
    }
    // Een afwijkende oorzaak telt niet mee in de groep.
    let mut anders = incident();
    anders.kenmerk = "2026-0050".into();
    anders.oorzaakcategorie = Some("verlies van gegevensdrager".into());
    incidenten.push(anders);

    let b = beoordeel_oorzaakpatroon(&motor, &incidenten, nu(), kwartaalgrens);
    assert_eq!(b.len(), 1);
    assert_eq!(b[0].regelcode, "LEK-13");
    assert_eq!(b[0].niveau, Niveau::Rapporterend, "een patroon is geen blokkade");
    assert_eq!(b[0].ontvanger, Ontvangerrol::Directie);
    assert!(b[0].toelichting.contains("4 keer"));
}

#[test]
fn drie_keer_dezelfde_oorzaak_is_nog_geen_patroon() {
    let motor = standaardmotor();
    let kwartaalgrens = Utc.with_ymd_and_hms(2026, 7, 1, 0, 0, 0).unwrap();
    let incidenten: Vec<_> = (1..=3)
        .map(|n| {
            let mut i = incident();
            i.kenmerk = format!("2026-004{n}");
            i.oorzaakcategorie = Some("verkeerde geadresseerde".into());
            i
        })
        .collect();
    assert!(beoordeel_oorzaakpatroon(&motor, &incidenten, nu(), kwartaalgrens).is_empty());
}

// --------------------------------------------------------------------------
// Het rapport
// --------------------------------------------------------------------------

#[test]
fn het_rapport_wijst_aan_waar_het_structureel_misgaat() {
    let motor = standaardmotor();
    let mut bevindingen = Vec::new();

    for n in 1..=3 {
        let mut v = verwerking();
        v.kenmerk = format!("041{n}-K");
        v.bewaartermijn = None;
        v.stel_vast("u2", nu()).ok();
        v.status = dpofg_domain::Status::Vastgesteld;
        bevindingen.extend(beoordeel_verwerking(&motor, &v, nu()));
    }

    let rapport = motor.rapporteer(bevindingen, 3, nu());
    assert!(rapport.heeft_blokkades());
    let per_regel = rapport.per_regel();
    assert_eq!(per_regel[0].1, 3, "de meest voorkomende bevinding staat bovenaan");
    assert!(!rapport.voor(Ontvangerrol::Functionaris).is_empty());
    assert!(rapport.op_niveau(Niveau::Blokkerend).len() >= 3);
    assert!(rapport.onderbrekingen() >= 3);
}

#[test]
fn een_leeg_rapport_is_hanteerbaar() {
    let motor = standaardmotor();
    let rapport = motor.rapporteer(Vec::new(), 0, nu());
    assert!(!rapport.heeft_blokkades());
    assert!(rapport.per_regel().is_empty());
    assert_eq!(rapport.onderbrekingen(), 0);
    assert_eq!(rapport.regels_gedraaid, motor.aantal());
}
