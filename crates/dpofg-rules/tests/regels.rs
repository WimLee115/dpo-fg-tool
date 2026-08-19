//! De controleregels die over de samenhang waken.

use chrono::{DateTime, Duration, TimeZone, Utc};
use dpofg_domain::{
    avg::{BijzondereCategorie, Grondslag, Rol},
    incident::Herkomstkanaal,
    Aantasting, Bewaartermijn, Dpia, Id, Incident, Motivering, Ontvanger, Overgenomen,
    Restrisiconiveau, Risiconiveau, Status, Termijneenheid, Verwerking, Voortoets,
};
use dpofg_rules::{
    budget::Waarschuwingsbudget,
    motor::{Niveau, Ontvangerrol},
    regels::{
        beoordeel_budget, beoordeel_dpia, beoordeel_incident, beoordeel_logboek,
        beoordeel_meldtermijn, beoordeel_oorzaakpatroon, beoordeel_raadplegingstermijn,
        beoordeel_verwerking, catalogus, geimplementeerd, standaardmotor,
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

/// De andere richting, en de belangrijkste: elke regel die een evaluatie kán
/// afgeven, moet ook als geïmplementeerd worden gemeld.
///
/// Deze test bestaat omdat het mis is gegaan. GRO-04 en GRO-05 werden al
/// beoordeeld — de takken staan in `beoordeel_verwerking` — maar ontbraken in
/// `geimplementeerd()`, waardoor `dpofg controle --dekking` een lagere dekking
/// meldde dan er draaide. Een teller die onder zijn stand meldt is minder erg
/// dan een die erboven meldt, maar allebei zijn ze onbruikbaar als antwoord op
/// de vraag "wat bewaakt dit product".
///
/// De controle leest de broncode van de regelmodule. Dat is ongebruikelijk,
/// maar het alternatief — elke regel op een echte fixture laten aanslaan —
/// vraagt vijfenvijftig fixtures om één lijst te bewaken.
#[test]
fn elke_regel_die_kan_aanslaan_wordt_ook_als_dekking_gemeld() {
    const BRON: &str = include_str!("../src/regels.rs");

    // Alles vanaf de eerste evaluatiefunctie tot aan de tests: daarvóór staan
    // de definities, die dezelfde codes als tekst bevatten.
    let start = BRON.find("pub fn beoordeel_verwerking").expect("de evaluaties beginnen hier");
    let eind = BRON.find("\n#[cfg(test)]").unwrap_or(BRON.len());
    let evaluaties = &BRON[start..eind];

    let gemeld: Vec<&str> = geimplementeerd().to_vec();
    let mut ontbreekt = Vec::new();

    for regel in catalogus() {
        let letterlijk = format!("\"{}\"", regel.code);
        if evaluaties.contains(&letterlijk) && !gemeld.contains(&regel.code.as_str()) {
            ontbreekt.push(regel.code.clone());
        }
    }

    assert!(
        ontbreekt.is_empty(),
        "deze regels worden beoordeeld maar niet als dekking gemeld: {}",
        ontbreekt.join(", ")
    );
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
    assert_eq!(b.iter().find(|x| x.regelcode == "BEW-01").unwrap().niveau, Niveau::Blokkerend);
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
    let b = beoordeel_incident(&motor, &i, nu() + Duration::hours(13));
    assert!(codes(&b).contains(&"LEK-15"));
}

/// De koppeling is de uitkomst van het onderzoek, niet van de intake. Wie een
/// incident registreert weet vaak nog niet welke verwerking is geraakt; meteen
/// melden levert een bevinding op bij iemand die er op dat moment niets aan kan
/// doen.
#[test]
fn een_vers_incident_zonder_registerkoppeling_zwijgt() {
    let motor = standaardmotor();
    let mut i = incident();
    i.getroffen_verwerkingen.clear();
    let b = beoordeel_incident(&motor, &i, nu() + Duration::hours(1));
    assert!(!codes(&b).contains(&"LEK-15"), "binnen het respijt hoort de regel te zwijgen");
}

#[test]
fn een_gekoppeld_incident_blijft_stil() {
    let motor = standaardmotor();
    let i = incident();
    assert!(!i.getroffen_verwerkingen.is_empty(), "de fixture hoort een koppeling te hebben");
    let b = beoordeel_incident(&motor, &i, nu() + Duration::hours(48));
    assert!(!codes(&b).contains(&"LEK-15"));
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

// --------------------------------------------------------------------------
// De bewaartermijn zonder bron
// --------------------------------------------------------------------------

#[test]
fn een_bewaartermijn_zonder_bron_signaleert() {
    let motor = standaardmotor();
    let mut v = verwerking();
    v.bewaartermijn = Some(Bewaartermijn::Vast {
        duur: 2,
        eenheid: Termijneenheid::Jaren,
        grondslag: "  ".into(),
        vanaf: "einde dienstverband".into(),
    });
    let b = beoordeel_verwerking(&motor, &v, nu());
    let bev = b.iter().find(|x| x.regelcode == "BEW-04").expect("BEW-04 hoort aan te slaan");
    assert_eq!(bev.niveau, Niveau::Signalerend);
    assert!(bev.toelichting.contains("2 jaar"), "kreeg: {}", bev.toelichting);
}

#[test]
fn een_bewaartermijn_met_bron_zwijgt() {
    let motor = standaardmotor();
    let b = beoordeel_verwerking(&motor, &verwerking(), nu());
    assert!(!codes(&b).contains(&"BEW-04"), "de fixture heeft 'art. 52 AWR' als bron");
}

/// Een termijn die nog moet worden bepaald heeft geen bronveld; die is al
/// gedekt door BEW-01 en BEW-02, en twee keer melden op hetzelfde gat is ruis.
#[test]
fn een_nog_te_bepalen_termijn_levert_geen_bew_04() {
    let motor = standaardmotor();
    let mut v = verwerking();
    v.bewaartermijn = Some(Bewaartermijn::NogTeBepalen {
        motivering: motivering("wacht op de selectielijst"),
        uiterlijk_bepaald_op: nu() + Duration::days(30),
        eigenaar: "afdeling P&O".into(),
    });
    let b = beoordeel_verwerking(&motor, &v, nu());
    assert!(!codes(&b).contains(&"BEW-04"));
}

// --------------------------------------------------------------------------
// De meldtermijn die afloopt
// --------------------------------------------------------------------------

fn meldtermijn(verstrijkt: DateTime<Utc>) -> dpofg_terms::Deadline {
    dpofg_terms::Deadline {
        moment: verstrijkt,
        lokaal: "21-08-2026 11:00 (Europe/Amsterdam)".into(),
        tijdzone: "Europe/Amsterdam".into(),
        anker: verstrijkt - Duration::hours(72),
        code: "AVG-33-MELDING".into(),
        duur: "72 uur".into(),
        grondslag: "art. 33 lid 1 AVG".into(),
        verlenging: dpofg_terms::ToegepasteVerlenging::NietVanToepassingBijUren,
        verlengingsbepaling: "niet van toepassing".into(),
        verantwoording: "72 uur na kennisname".into(),
    }
}

#[test]
fn een_aflopende_meldtermijn_zonder_besluit_signaleert() {
    let motor = standaardmotor();
    let i = incident();
    let b = beoordeel_meldtermijn(&motor, &i, &meldtermijn(nu() + Duration::hours(7)), nu());
    let bev = b.first().expect("LEK-02 hoort aan te slaan");
    assert_eq!(bev.regelcode, "LEK-02");
    assert_eq!(bev.niveau, Niveau::Signalerend);
    assert_eq!(bev.ontvanger, Ontvangerrol::Functionaris);
    assert!(bev.toelichting.contains("7 uur"), "kreeg: {}", bev.toelichting);
}

/// Zonder ondergrens blijft de regel na het verstrijken eeuwig afgaan op een
/// incident waar niemand nog iets aan kan doen.
#[test]
fn een_verstreken_meldtermijn_zwijgt() {
    let motor = standaardmotor();
    let i = incident();
    let b = beoordeel_meldtermijn(&motor, &i, &meldtermijn(nu() - Duration::hours(8)), nu());
    assert!(b.is_empty());
}

#[test]
fn een_meldtermijn_ver_weg_zwijgt() {
    let motor = standaardmotor();
    let i = incident();
    let b = beoordeel_meldtermijn(&motor, &i, &meldtermijn(nu() + Duration::hours(40)), nu());
    assert!(b.is_empty());
}

#[test]
fn een_genomen_meldbesluit_maakt_de_herinnering_overbodig() {
    let motor = standaardmotor();
    let mut i = incident();
    i.gemeld_op = Some(nu());
    assert!(
        beoordeel_meldtermijn(&motor, &i, &meldtermijn(nu() + Duration::hours(7)), nu()).is_empty()
    );

    let mut i = incident();
    i.afgehandeld_op = Some(nu());
    assert!(
        beoordeel_meldtermijn(&motor, &i, &meldtermijn(nu() + Duration::hours(7)), nu()).is_empty()
    );
}

// --------------------------------------------------------------------------
// Het logboek zelf
// --------------------------------------------------------------------------

fn rapport(
    bevindingen: Vec<dpofg_audit::Bevinding>,
    ankerstatus: dpofg_audit::Ankerstatus,
) -> dpofg_audit::Verificatierapport {
    dpofg_audit::Verificatierapport {
        regels: 5,
        eerste_volgnummer: Some(1),
        laatste_volgnummer: Some(5),
        laatste_hash: Some("a".repeat(64)),
        periode: None,
        bevindingen,
        ankerstatus,
    }
}

fn ketenbevinding(volgnummer: u64, soort: dpofg_audit::Bevindingsoort) -> dpofg_audit::Bevinding {
    dpofg_audit::Bevinding { volgnummer, soort, omschrijving: "proefbevinding".into() }
}

#[test]
fn een_ongeschonden_logboek_zonder_anker_levert_niets_op() {
    let motor = standaardmotor();
    let b = beoordeel_logboek(
        &motor,
        &rapport(Vec::new(), dpofg_audit::Ankerstatus::GeenAnker),
        Some(nu() - Duration::hours(1)),
        nu(),
    );
    assert!(b.is_empty(), "geen anker is de normale toestand, geen bevinding");
}

#[test]
fn een_gebroken_keten_blokkeert() {
    use dpofg_audit::Bevindingsoort::*;
    let motor = standaardmotor();
    let b = beoordeel_logboek(
        &motor,
        &rapport(
            vec![ketenbevinding(3, OntbrekendeRegel), ketenbevinding(4, Ketenbreuk)],
            dpofg_audit::Ankerstatus::GeenAnker,
        ),
        None,
        nu(),
    );
    assert_eq!(b.len(), 2);
    for bev in &b {
        assert_eq!(bev.regelcode, "SYS-04");
        assert_eq!(bev.niveau, Niveau::Blokkerend);
        assert_eq!(bev.record_soort, "logboek");
    }
}

#[test]
fn een_ingekorte_keten_blokkeert() {
    let motor = standaardmotor();
    let b = beoordeel_logboek(
        &motor,
        &rapport(
            Vec::new(),
            dpofg_audit::Ankerstatus::KetenIsIngekort { anker_volgnummer: 9, keten_volgnummer: 5 },
        ),
        None,
        nu(),
    );
    assert_eq!(codes(&b), vec!["SYS-04"]);
}

#[test]
fn een_teruglopend_tijdstip_signaleert() {
    let motor = standaardmotor();
    let b = beoordeel_logboek(
        &motor,
        &rapport(
            vec![ketenbevinding(3, dpofg_audit::Bevindingsoort::TijdLooptTerug)],
            dpofg_audit::Ankerstatus::GeenAnker,
        ),
        None,
        nu(),
    );
    assert_eq!(codes(&b), vec!["SYS-10"]);
}

#[test]
fn een_klok_die_achterloopt_signaleert() {
    let motor = standaardmotor();
    let b = beoordeel_logboek(
        &motor,
        &rapport(Vec::new(), dpofg_audit::Ankerstatus::GeenAnker),
        Some(nu() + Duration::minutes(30)),
        nu(),
    );
    let bev = b.first().expect("SYS-10 hoort aan te slaan");
    assert_eq!(bev.regelcode, "SYS-10");
    assert!(bev.toelichting.contains("30 minuten"), "kreeg: {}", bev.toelichting);
}

// --------------------------------------------------------------------------
// Het waarschuwingsbudget
// --------------------------------------------------------------------------

#[test]
fn zes_onderbrekingen_in_een_week_melden_een_ontwerpfout() {
    let motor = standaardmotor();
    let mut budget = Waarschuwingsbudget::nieuw();
    for dag in 0..6 {
        budget.onderbreking("a.devries", nu() - Duration::days(dag));
    }
    let b = beoordeel_budget(&motor, &budget, nu());
    let bev = b.first().expect("SYS-06 hoort aan te slaan");
    assert_eq!(bev.regelcode, "SYS-06");
    assert_eq!(bev.niveau, Niveau::Rapporterend);
    assert!(bev.toelichting.contains('6') && bev.toelichting.contains('5'));
}

#[test]
fn vijf_onderbrekingen_blijven_binnen_het_budget() {
    let motor = standaardmotor();
    let mut budget = Waarschuwingsbudget::nieuw();
    for dag in 0..5 {
        budget.onderbreking("a.devries", nu() - Duration::days(dag));
    }
    assert!(beoordeel_budget(&motor, &budget, nu()).is_empty(), "vijf is precies de grens");
}

#[test]
fn onderbrekingen_van_verschillende_gebruikers_tellen_apart() {
    let motor = standaardmotor();
    let mut budget = Waarschuwingsbudget::nieuw();
    for dag in 0..3 {
        budget.onderbreking("a.devries", nu() - Duration::days(dag));
        budget.onderbreking("b.jansen", nu() - Duration::days(dag));
    }
    assert!(beoordeel_budget(&motor, &budget, nu()).is_empty());
}

// --------------------------------------------------------------------------
// De effectbeoordeling
// --------------------------------------------------------------------------

const HERBEOORDELING: i64 = 36;

fn dpia() -> Dpia {
    let mut d = Dpia::nieuw("DPIA-0412", "Verzuimregistratie", Id::nieuw(), "u1", nu());
    d.voortoets = Some(Voortoets::Vereist);
    d.voortoets_motivering = Some(motivering("twee criteria worden geraakt"));
    d.leg_beoordeling_vast(nu(), "A. de Vries", Some(true), nu()).unwrap();
    d.systematische_beschrijving = Some("verzuimregistratie voor loondoorbetaling".into());
    d.noodzaak_en_evenredigheid = Some("geen minder ingrijpend alternatief".into());
    d.risicos.push("onbevoegde inzage door collega's".into());
    d.maatregelen.push("toegang op rolbasis".into());
    d.stel_restrisico_vast(Restrisiconiveau::Laag, motivering("beperkte kring"), nu()).unwrap();
    d
}

#[test]
fn een_volledige_effectbeoordeling_levert_geen_bevindingen() {
    let motor = standaardmotor();
    assert!(beoordeel_dpia(&motor, &dpia(), HERBEOORDELING, nu()).is_empty());
}

#[test]
fn dpia_03_slaat_aan_bij_een_beoordeling_na_aanvang() {
    let motor = standaardmotor();
    let mut d = dpia();
    d.vooraf_uitgevoerd = Some(false);
    let b = beoordeel_dpia(&motor, &d, HERBEOORDELING, nu());
    let bev = b.iter().find(|x| x.regelcode == "DPIA-03").expect("DPIA-03 hoort aan te slaan");
    assert_eq!(bev.niveau, Niveau::Signalerend);
    assert!(bev.toelichting.contains("nadat de verwerking al liep"));
}

/// Bij een vrijwillige beoordeling is er geen moment waarvóór zij had moeten
/// plaatsvinden; de regel hoort dan te zwijgen.
#[test]
fn dpia_03_zwijgt_bij_een_vrijwillige_beoordeling() {
    let motor = standaardmotor();
    let mut d = dpia();
    d.voortoets = Some(Voortoets::Vrijwillig);
    d.vooraf_uitgevoerd = Some(false);
    assert!(!codes(&beoordeel_dpia(&motor, &d, HERBEOORDELING, nu())).contains(&"DPIA-03"));
}

/// Een onbeantwoorde vraag is geen "nee".
#[test]
fn dpia_03_zwijgt_zolang_de_vraag_niet_is_beantwoord() {
    let motor = standaardmotor();
    let mut d = dpia();
    d.vooraf_uitgevoerd = None;
    assert!(!codes(&beoordeel_dpia(&motor, &d, HERBEOORDELING, nu())).contains(&"DPIA-03"));
}

#[test]
fn dpia_06_slaat_aan_bij_hoog_restrisico_zonder_raadpleging() {
    let motor = standaardmotor();
    let mut d = dpia();
    d.restrisico = None;
    d.stel_restrisico_vast(Restrisiconiveau::Hoog, motivering("het risico blijft groot"), nu())
        .unwrap();
    let b = beoordeel_dpia(&motor, &d, HERBEOORDELING, nu());
    let bev = b.iter().find(|x| x.regelcode == "DPIA-06").expect("DPIA-06 hoort aan te slaan");
    assert!(bev.toelichting.contains("geen verzoek om voorafgaande raadpleging"));
}

#[test]
fn dpia_06_zwijgt_bij_een_gemiddeld_restrisico() {
    let motor = standaardmotor();
    let mut d = dpia();
    d.restrisico = None;
    d.stel_restrisico_vast(Restrisiconiveau::Gemiddeld, motivering("hanteerbaar"), nu()).unwrap();
    assert!(!codes(&beoordeel_dpia(&motor, &d, HERBEOORDELING, nu())).contains(&"DPIA-06"));
}

/// De kern van DPIA-06 in zijn tweede vorm: het verstrijken van de termijn is
/// geen goedkeuring. Die zin is de inhoud van de regel, niet de opsmuk.
#[test]
fn dpia_06_zegt_dat_stilzitten_geen_goedkeuring_is() {
    let motor = standaardmotor();
    let d = dpia();
    let deadline = raadplegingstermijn(nu() - Duration::days(1));
    let b = beoordeel_raadplegingstermijn(&motor, &d, &deadline, nu());
    let bev = b.first().expect("DPIA-06 hoort aan te slaan");
    assert_eq!(bev.regelcode, "DPIA-06");
    assert!(
        bev.toelichting.contains("Het verstrijken van deze termijn is geen goedkeuring"),
        "kreeg: {}",
        bev.toelichting
    );
}

#[test]
fn dpia_06_zwijgt_zolang_de_termijn_loopt() {
    let motor = standaardmotor();
    let d = dpia();
    let deadline = raadplegingstermijn(nu() + Duration::days(14));
    assert!(beoordeel_raadplegingstermijn(&motor, &d, &deadline, nu()).is_empty());
}

#[test]
fn dpia_06_zwijgt_zodra_er_advies_is() {
    let motor = standaardmotor();
    let mut d = dpia();
    let klok = lopende_raadpleging();
    d.dien_raadpleging_in(klok, nu()).unwrap();
    let later = nu() + Duration::days(3);
    d.leg_advies_vast(later, "AP-2026-1234", later).unwrap();

    let deadline = raadplegingstermijn(nu() - Duration::days(1));
    assert!(beoordeel_raadplegingstermijn(&motor, &d, &deadline, nu()).is_empty());
}

#[test]
fn dpia_07_slaat_aan_na_de_herbeoordelingstermijn() {
    let motor = standaardmotor();
    let mut d = dpia();
    d.status = Status::Vastgesteld;
    let later = nu() + Duration::days(40 * 30);
    let b = beoordeel_dpia(&motor, &d, HERBEOORDELING, later);
    let bev = b.iter().find(|x| x.regelcode == "DPIA-07").expect("DPIA-07 hoort aan te slaan");
    assert!(bev.toelichting.contains("maanden geleden"));
}

#[test]
fn dpia_07_zwijgt_binnen_de_herbeoordelingstermijn() {
    let motor = standaardmotor();
    let mut d = dpia();
    d.status = Status::Vastgesteld;
    let later = nu() + Duration::days(30 * 30);
    assert!(!codes(&beoordeel_dpia(&motor, &d, HERBEOORDELING, later)).contains(&"DPIA-07"));
}

/// De drempel komt uit het kennispakket; met een andere norm verschuift de
/// regel mee zonder dat er code verandert.
#[test]
fn de_herbeoordelingsdrempel_komt_van_buiten_de_regel() {
    let motor = standaardmotor();
    let mut d = dpia();
    d.status = Status::Vastgesteld;
    let later = nu() + Duration::days(13 * 30);
    assert!(!codes(&beoordeel_dpia(&motor, &d, 36, later)).contains(&"DPIA-07"));
    assert!(codes(&beoordeel_dpia(&motor, &d, 12, later)).contains(&"DPIA-07"));
}

#[test]
fn dpia_07_slaat_aan_wanneer_de_verwerking_is_gewijzigd() {
    let motor = standaardmotor();
    let mut d = dpia();
    d.status = Status::Vastgesteld;
    d.markeer_herziening_nodig("de criteria van registerregel 0412-K zijn gewijzigd", nu());

    let b = beoordeel_dpia(&motor, &d, HERBEOORDELING, nu());
    let bev = b.iter().find(|x| x.regelcode == "DPIA-07").expect("DPIA-07 hoort aan te slaan");
    assert!(
        bev.toelichting.contains("0412-K"),
        "de reden hoort in de melding: {}",
        bev.toelichting
    );
}

/// De reden staat in de herkomst, en die wordt bij elke bewerking overschreven.
/// Pakt de gebruiker de herziening op, dan mag de melding niet ineens
/// "beoordeling vastgelegd" als reden noemen onder de kop "Beoordeling
/// verouderd".
#[test]
fn dpia_07_blijft_waar_nadat_het_dossier_is_aangeraakt() {
    let motor = standaardmotor();
    let mut d = dpia();
    d.status = Status::Vastgesteld;
    d.markeer_herziening_nodig("de criteria van registerregel 0412-K zijn gewijzigd", nu());
    // De gebruiker pakt de herziening op en werkt een onderdeel bij.
    d.leg_beoordeling_vast(nu(), "A. de Vries", Some(true), nu()).unwrap();

    let b = beoordeel_dpia(&motor, &d, HERBEOORDELING, nu());
    let bev = b.iter().find(|x| x.regelcode == "DPIA-07").expect("DPIA-07 hoort aan te slaan");
    assert!(
        bev.toelichting.contains("de onderliggende verwerking is gewijzigd"),
        "kreeg: {}",
        bev.toelichting
    );
    assert!(
        !bev.toelichting.contains("beoordeling vastgelegd"),
        "de laatste bewerking is niet de reden van de herziening: {}",
        bev.toelichting
    );
}

/// De regel draagt twee vormen; de omschrijving en de grondslag horen ze
/// allebei te dekken.
#[test]
fn dpia_06_dekt_beide_vormen_in_zijn_omschrijving() {
    let motor = standaardmotor();
    let regel = motor.regel("DPIA-06").expect("DPIA-06 staat in de catalogus");
    assert!(regel.controleert.contains("zonder voorafgaande raadpleging"));
    assert!(regel.controleert.contains("verstreken"));
    assert!(regel.grondslag.contains("lid 2"), "de tweede vorm berust op lid 2");
}

fn lopende_raadpleging() -> dpofg_terms::LopendeTermijn {
    let pakket = dpofg_content::startpakket(nu().date_naive());
    let soort = pakket.termijn("AVG-36-RAADPLEGING").unwrap().clone();
    let kalender = pakket.kalender("NL").unwrap();
    let zone = dpofg_terms::tijdzone(dpofg_terms::TIJDZONE_NL).unwrap();
    dpofg_terms::LopendeTermijn::start(soort, nu(), zone, kalender).unwrap()
}

fn raadplegingstermijn(verstrijkt: DateTime<Utc>) -> dpofg_terms::Deadline {
    dpofg_terms::Deadline {
        moment: verstrijkt,
        lokaal: "14-10-2026 00:00 (Europe/Amsterdam)".into(),
        tijdzone: "Europe/Amsterdam".into(),
        anker: verstrijkt - Duration::weeks(8),
        code: "AVG-36-RAADPLEGING".into(),
        duur: "8 weken".into(),
        grondslag: "art. 36 lid 2 AVG".into(),
        verlenging: dpofg_terms::ToegepasteVerlenging::GeenNodig,
        verlengingsbepaling: "art. 36 lid 2, tweede en derde volzin, AVG".into(),
        verantwoording: "8 weken na ontvangst van het verzoek".into(),
    }
}
