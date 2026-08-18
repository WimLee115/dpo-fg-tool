//! Bewijs dat de afgeleide verplichtingen werken.
//!
//! Elk geval hier komt uit paragraaf 3.4 van het foutbestendigheidshoofdstuk:
//! de tool leidt de verplichting af uit een antwoord dat de gebruiker al heeft
//! gegeven, zodat hij de regel niet hoeft te kennen. Een test die faalt
//! betekent dat een gebruiker een verplichting kan missen zonder dat iets hem
//! tegenhoudt.

use chrono::{DateTime, TimeZone, Utc};
use dpofg_domain::{
    avg::{BijzondereCategorie, Grondslag, Rol, UitzonderingArtikel9},
    Bewaartermijn, Id, Motivering, Ontvanger, Overgenomen, Registerrapport, Status, Termijneenheid,
    Verwerking, Volledig,
};

fn nu() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 8, 18, 9, 0, 0).unwrap()
}

/// Een verwerking waarin alles van de vaste kern is ingevuld.
/// Vanaf hier wordt per test één antwoord toegevoegd om te zien wat er ontstaat.
fn basisverwerking() -> Verwerking {
    let mut v = Verwerking::nieuw(
        "0412-K",
        "Verzuimregistratie",
        Rol::Verwerkingsverantwoordelijke,
        "afdeling P&O",
        "u1",
        nu(),
    );
    v.doeleinden = vec!["uitvoering van de loondoorbetalingsplicht bij ziekte".into()];
    v.categorieen_betrokkenen = vec!["medewerkers".into()];
    v.categorieen_gegevens = vec!["naam".into(), "eerste ziektedag".into()];
    v.ontvangers = vec![Ontvanger {
        omschrijving: "leidinggevende".into(),
        is_verwerker: false,
        leverancier_id: None,
        buiten_eer: false,
    }];
    v.bewaartermijn = Some(Bewaartermijn::Vast {
        duur: 2,
        eenheid: Termijneenheid::Jaren,
        grondslag: "art. 52 Algemene wet inzake rijksbelastingen".into(),
        vanaf: "einde dienstverband".into(),
    });
    v.beveiligingsmaatregelen = Some("toegang op rolbasis, versleutelde opslag".into());
    v.grondslag = Some(Grondslag::WettelijkeVerplichting);
    v.wettelijke_bepaling = Some("art. 7:629 BW".into());
    v.grondslag_motivering = Some(
        Motivering::nieuw(
            "de werkgever is wettelijk verplicht het loon door te betalen",
            "u1",
            nu(),
        )
        .unwrap(),
    );
    v
}

fn ontbreekt(v: &Verwerking, veld: &str) -> bool {
    v.volledigheid().ontbreekt.iter().any(|o| o.veld == veld)
}

fn blokkeert(v: &Verwerking, veld: &str) -> bool {
    v.volledigheid().ontbreekt.iter().any(|o| o.veld == veld && o.blokkeert_vaststelling)
}

// --------------------------------------------------------------------------
// De vaste kern van artikel 30
// --------------------------------------------------------------------------

#[test]
fn leeg_concept_meldt_alles_wat_ontbreekt() {
    let v = Verwerking::nieuw("X", "X", Rol::Verwerkingsverantwoordelijke, "e", "u1", nu());
    let r = v.volledigheid();

    assert_eq!(v.status, Status::Concept, "een concept is een geldige toestand");
    assert!(!r.mag_vaststellen());
    for veld in [
        "verwerking.doeleinden",
        "verwerking.categorieen_betrokkenen",
        "verwerking.categorieen_gegevens",
        "verwerking.ontvangers",
        "verwerking.bewaartermijn",
        "verwerking.beveiligingsmaatregelen",
        "verwerking.grondslag",
    ] {
        assert!(ontbreekt(&v, veld), "{veld} hoort te ontbreken");
    }
}

#[test]
fn volledige_verwerking_kan_worden_vastgesteld() {
    let mut v = basisverwerking();
    assert!(v.volledigheid().is_volledig(), "kreeg: {:?}", v.volledigheid().ontbreekt);
    v.stel_vast("u2", nu()).unwrap();
    assert_eq!(v.status, Status::Vastgesteld);
    assert_eq!(v.herkomst.vastgesteld_door.as_deref(), Some("u2"));
}

#[test]
fn vaststellen_faalt_met_een_bruikbare_melding() {
    let mut v = basisverwerking();
    v.bewaartermijn = None;
    let fout = v.stel_vast("u2", nu()).unwrap_err().to_string();

    assert!(fout.contains("bewaartermijn") || fout.contains("bewaard"), "kreeg: {fout}");
    assert!(fout.contains("art. 30 lid 1 onder f AVG"), "de grondslag hoort erbij: {fout}");
    assert_eq!(v.status, Status::Concept, "de status verandert niet bij een mislukte vaststelling");
}

// --------------------------------------------------------------------------
// Afgeleide verplichtingen
// --------------------------------------------------------------------------

/// Bijzondere gegevens aangevinkt, dus de uitzonderingsgrond van artikel 9
/// wordt verplicht.
#[test]
fn bijzondere_gegevens_roepen_de_uitzonderingsgrond_op() {
    let mut v = basisverwerking();
    assert!(v.volledigheid().is_volledig());

    v.bijzondere_categorieen.push(BijzondereCategorie::Gezondheidsgegevens);
    assert!(blokkeert(&v, "verwerking.uitzondering_artikel9"));
    assert!(v.stel_vast("u2", nu()).is_err());

    v.uitzondering_artikel9 = Some(UitzonderingArtikel9::UitdrukkelijkeToestemming);
    assert!(!ontbreekt(&v, "verwerking.uitzondering_artikel9"));
}

/// Een uitzondering die nationaal recht vereist, roept een tweede verplichting op.
#[test]
fn uitzondering_die_nationaal_recht_vereist_roept_een_tweede_eis_op() {
    let mut v = basisverwerking();
    v.bijzondere_categorieen.push(BijzondereCategorie::Gezondheidsgegevens);
    v.uitzondering_artikel9 = Some(UitzonderingArtikel9::Gezondheidszorg);

    assert!(blokkeert(&v, "verwerking.uitzondering_nationale_bepaling"));

    v.uitzondering_nationale_bepaling = Some("art. 30 lid 3 onder a UAVG".into());
    assert!(v.volledigheid().is_volledig(), "kreeg: {:?}", v.volledigheid().ontbreekt);
}

/// Een uitzondering die géén nationaal recht vereist, roept die eis niet op.
#[test]
fn uitzondering_zonder_nationale_eis_roept_niets_extras_op() {
    let mut v = basisverwerking();
    v.bijzondere_categorieen.push(BijzondereCategorie::Gezondheidsgegevens);
    v.uitzondering_artikel9 = Some(UitzonderingArtikel9::Rechtsvordering);
    assert!(!ontbreekt(&v, "verwerking.uitzondering_nationale_bepaling"));
}

/// Gerechtvaardigd belang, dus de belangenafweging wordt verplicht.
#[test]
fn gerechtvaardigd_belang_roept_de_belangenafweging_op() {
    let mut v = basisverwerking();
    v.grondslag = Some(Grondslag::GerechtvaardigdBelang);
    v.wettelijke_bepaling = None;

    assert!(blokkeert(&v, "verwerking.belangenafweging"));
    assert!(!ontbreekt(&v, "verwerking.wettelijke_bepaling"), "die eis hoort hier niet");

    v.belangenafweging_id = Some(Id::nieuw());
    assert!(v.volledigheid().is_volledig(), "kreeg: {:?}", v.volledigheid().ontbreekt);
}

/// Toestemming, dus het bewijs daarvan wordt verplicht.
#[test]
fn toestemming_roept_de_bewijsvoering_op() {
    let mut v = basisverwerking();
    v.grondslag = Some(Grondslag::Toestemming);
    v.wettelijke_bepaling = None;

    assert!(blokkeert(&v, "verwerking.toestemming"));
    v.toestemming_id = Some(Id::nieuw());
    assert!(v.volledigheid().is_volledig());
}

/// Wettelijke verplichting, dus de bepaling zelf wordt verplicht.
#[test]
fn wettelijke_verplichting_roept_de_bepaling_op() {
    let mut v = basisverwerking();
    v.wettelijke_bepaling = None;
    assert!(blokkeert(&v, "verwerking.wettelijke_bepaling"));
}

/// Het burgerservicenummer vereist een eigen wettelijke grondslag.
#[test]
fn burgerservicenummer_roept_een_eigen_grondslag_op() {
    let mut v = basisverwerking();
    v.burgerservicenummer = true;
    assert!(blokkeert(&v, "verwerking.bsn_grondslag"));

    v.bsn_grondslag =
        Some("art. 46 UAVG jo. de Wet algemene bepalingen burgerservicenummer".into());
    assert!(v.volledigheid().is_volledig());
}

/// Strafrechtelijke gegevens vereisen een uitzonderingsgrond uit de UAVG.
#[test]
fn strafrechtelijke_gegevens_roepen_een_uitzondering_op() {
    let mut v = basisverwerking();
    v.strafrechtelijke_gegevens = true;
    assert!(blokkeert(&v, "verwerking.uitzondering_strafrechtelijk"));
}

/// Een verwerker koppelen roept de verwerkersovereenkomst op — per verwerker.
#[test]
fn elke_verwerker_vraagt_om_een_eigen_overeenkomst() {
    let mut v = basisverwerking();
    v.ontvangers.push(Ontvanger {
        omschrijving: "arbodienst".into(),
        is_verwerker: true,
        leverancier_id: Some(Id::nieuw()),
        buiten_eer: false,
    });
    v.ontvangers.push(Ontvanger {
        omschrijving: "salarisverwerker".into(),
        is_verwerker: true,
        leverancier_id: Some(Id::nieuw()),
        buiten_eer: false,
    });

    assert_eq!(v.aantal_verwerkers(), 2);
    assert!(blokkeert(&v, "verwerking.verwerkersovereenkomsten"));

    // Eén overeenkomst is niet genoeg voor twee verwerkers.
    v.verwerkersovereenkomsten.push(Id::nieuw());
    assert!(blokkeert(&v, "verwerking.verwerkersovereenkomsten"));
    let melding = v
        .volledigheid()
        .ontbreekt
        .iter()
        .find(|o| o.veld == "verwerking.verwerkersovereenkomsten")
        .unwrap()
        .omschrijving
        .clone();
    assert!(melding.contains("2"), "de melding hoort te tellen: {melding}");
    assert!(melding.contains("nu 1 gekoppeld"), "kreeg: {melding}");

    v.verwerkersovereenkomsten.push(Id::nieuw());
    assert!(!ontbreekt(&v, "verwerking.verwerkersovereenkomsten"));
}

/// Een ontvanger buiten de EER roept het doorgifte-instrument op.
#[test]
fn doorgifte_buiten_de_eer_vraagt_om_een_waarborg() {
    let mut v = basisverwerking();
    v.ontvangers.push(Ontvanger {
        omschrijving: "analysedienst".into(),
        is_verwerker: true,
        leverancier_id: Some(Id::nieuw()),
        buiten_eer: true,
    });
    v.verwerkersovereenkomsten.push(Id::nieuw());

    assert!(blokkeert(&v, "verwerking.doorgiften"));
    v.doorgiften.push(Id::nieuw());
    assert!(v.volledigheid().is_volledig(), "kreeg: {:?}", v.volledigheid().ontbreekt);
}

/// Gezamenlijke verantwoordelijkheid vraagt om een vastgelegde regeling.
#[test]
fn gezamenlijke_verantwoordelijkheid_vraagt_om_een_regeling() {
    let mut v = basisverwerking();
    v.rol = Rol::GezamenlijkVerantwoordelijke;
    assert!(blokkeert(&v, "verwerking.gezamenlijke_regeling"));
    assert_eq!(v.rol.registerschema(), "art. 30 lid 1 AVG");
}

/// Uitsluitend geautomatiseerde besluitvorming vraagt om een eigen dossier.
#[test]
fn geautomatiseerde_besluitvorming_vraagt_om_een_dossier() {
    let mut v = basisverwerking();
    v.uitsluitend_geautomatiseerde_besluitvorming = true;
    assert!(blokkeert(&v, "verwerking.geautomatiseerde_besluitvorming"));
}

// --------------------------------------------------------------------------
// De effectbeoordeling: signaleren, niet beslissen
// --------------------------------------------------------------------------

#[test]
fn criteria_voor_de_effectbeoordeling_worden_geteld_en_benoemd() {
    let mut v = basisverwerking();
    assert!(v.getelde_dpia_criteria().is_empty());
    assert!(!v.dpia_waarschijnlijk_verplicht());

    v.bijzondere_categorieen.push(BijzondereCategorie::Gezondheidsgegevens);
    v.uitzondering_artikel9 = Some(UitzonderingArtikel9::Rechtsvordering);
    assert_eq!(v.getelde_dpia_criteria().len(), 1);
    assert!(!v.dpia_waarschijnlijk_verplicht(), "één criterium is nog geen verplichting");

    v.minderjarigen = true;
    assert_eq!(v.getelde_dpia_criteria().len(), 2);
    assert!(v.dpia_waarschijnlijk_verplicht());
    assert!(ontbreekt(&v, "verwerking.dpia"));
}

/// De effectbeoordeling signaleert, maar houdt vaststellen niet tegen.
///
/// Reden: of een beoordeling verplicht is, is een oordeel dat bij een mens
/// hoort. De tool telt de criteria die zij kan afleiden en toont ze; zij
/// beslist niet. Blokkeren zou betekenen dat de tool zich een oordeel aanmeet
/// dat zij niet kan onderbouwen.
#[test]
fn de_effectbeoordeling_signaleert_maar_blokkeert_niet() {
    let mut v = basisverwerking();
    v.bijzondere_categorieen.push(BijzondereCategorie::Gezondheidsgegevens);
    v.uitzondering_artikel9 = Some(UitzonderingArtikel9::Rechtsvordering);
    v.minderjarigen = true;

    assert!(ontbreekt(&v, "verwerking.dpia"));
    assert!(!blokkeert(&v, "verwerking.dpia"));
    assert!(v.volledigheid().mag_vaststellen());
    v.stel_vast("u2", nu()).unwrap();
}

// --------------------------------------------------------------------------
// Overgenomen gegevens
// --------------------------------------------------------------------------

/// Een overgenomen regel draagt dat kenmerk zichtbaar mee tot iemand hem
/// verifieert.
#[test]
fn overgenomen_zonder_verificatie_blijft_zichtbaar() {
    let mut v = basisverwerking();
    v.overgenomen = Some(Overgenomen {
        bron: "werkblad register 2024".into(),
        overgenomen_op: nu(),
        geverifieerd_op: None,
        geverifieerd_door: None,
    });

    assert!(ontbreekt(&v, "verwerking.verificatie_overname"));
    assert!(!blokkeert(&v, "verwerking.verificatie_overname"));
    // Ook na vaststellen blijft het kenmerk staan.
    v.stel_vast("u2", nu()).unwrap();
    assert!(ontbreekt(&v, "verwerking.verificatie_overname"));

    v.overgenomen.as_mut().unwrap().geverifieerd_op = Some(nu());
    v.overgenomen.as_mut().unwrap().geverifieerd_door = Some("u2".into());
    assert!(!ontbreekt(&v, "verwerking.verificatie_overname"));
}

// --------------------------------------------------------------------------
// Bewaartermijn
// --------------------------------------------------------------------------

/// Uitstel van de bewaartermijn mag, maar blijft zichtbaar en heeft een eigenaar.
#[test]
fn uitgestelde_bewaartermijn_blijft_zichtbaar() {
    let mut v = basisverwerking();
    v.bewaartermijn = Some(Bewaartermijn::NogTeBepalen {
        motivering: Motivering::nieuw(
            "de archiefselectielijst wordt in het vierde kwartaal herzien",
            "u1",
            nu(),
        )
        .unwrap(),
        uiterlijk_bepaald_op: Utc.with_ymd_and_hms(2026, 12, 31, 23, 59, 59).unwrap(),
        eigenaar: "recordmanager".into(),
    });

    assert!(ontbreekt(&v, "verwerking.bewaartermijn"));
    assert!(!blokkeert(&v, "verwerking.bewaartermijn"), "uitstel mag, verzwijgen niet");
    v.stel_vast("u2", nu()).unwrap();

    let b = v.bewaartermijn.as_ref().unwrap();
    assert!(!b.is_vastgesteld());
    assert!(!b.uitstel_verlopen(nu()));
    assert!(b.uitstel_verlopen(Utc.with_ymd_and_hms(2027, 1, 2, 0, 0, 0).unwrap()));
}

// --------------------------------------------------------------------------
// Herziening
// --------------------------------------------------------------------------

#[test]
fn een_wijziging_elders_zet_de_verwerking_op_herzien() {
    let mut v = basisverwerking();
    v.stel_vast("u2", nu()).unwrap();
    assert_eq!(v.status, Status::Vastgesteld);

    v.markeer_herziening_nodig("adequaatheidsbesluit ingetrokken", nu());
    assert_eq!(v.status, Status::HerzieningNodig);
    assert!(v.status.is_actief(), "herziening nodig betekent niet buiten werking");
    assert!(v.herkomst.gewijzigd_door.contains("adequaatheidsbesluit"));
}

#[test]
fn een_concept_wordt_niet_op_herzien_gezet() {
    let mut v = basisverwerking();
    v.markeer_herziening_nodig("iets veranderde", nu());
    assert_eq!(v.status, Status::Concept, "een concept was al niet vastgesteld");
}

// --------------------------------------------------------------------------
// Het register als geheel
// --------------------------------------------------------------------------

#[test]
fn registerrapport_wijst_aan_waar_het_structureel_misgaat() {
    let mut compleet = basisverwerking();
    compleet.stel_vast("u2", nu()).unwrap();

    let mut zonder_bewaartermijn = basisverwerking();
    zonder_bewaartermijn.bewaartermijn = None;

    let mut zonder_bewaartermijn_2 = basisverwerking();
    zonder_bewaartermijn_2.bewaartermijn = None;
    zonder_bewaartermijn_2.beveiligingsmaatregelen = None;

    let rapporten = vec![
        (compleet.status, compleet.volledigheid()),
        (zonder_bewaartermijn.status, zonder_bewaartermijn.volledigheid()),
        (zonder_bewaartermijn_2.status, zonder_bewaartermijn_2.volledigheid()),
    ];
    let r = Registerrapport::uit("verwerkingsregister", &rapporten);

    assert_eq!(r.totaal, 3);
    assert_eq!(r.vastgesteld, 1);
    assert_eq!(r.concept, 2);
    assert_eq!(r.volledig, 1);
    assert_eq!(r.geblokkeerd, 2);
    // De bewaartermijn ontbreekt het vaakst en staat daarom bovenaan.
    assert_eq!(r.ontbreekt_per_onderdeel[0].0, "verwerking.bewaartermijn");
    assert_eq!(r.ontbreekt_per_onderdeel[0].1, 2);
}

// --------------------------------------------------------------------------
// De teller is voortgang, geen verwijt
// --------------------------------------------------------------------------

#[test]
fn de_teller_groeit_mee_met_de_complexiteit() {
    let eenvoudig = basisverwerking();
    let eenvoudig_totaal = eenvoudig.volledigheid().verplicht;

    let mut ingewikkeld = basisverwerking();
    ingewikkeld.bijzondere_categorieen.push(BijzondereCategorie::Gezondheidsgegevens);
    ingewikkeld.uitzondering_artikel9 = Some(UitzonderingArtikel9::Gezondheidszorg);
    ingewikkeld.burgerservicenummer = true;
    ingewikkeld.ontvangers.push(Ontvanger {
        omschrijving: "arbodienst".into(),
        is_verwerker: true,
        leverancier_id: None,
        buiten_eer: false,
    });

    let ingewikkeld_totaal = ingewikkeld.volledigheid().verplicht;
    assert!(
        ingewikkeld_totaal > eenvoudig_totaal,
        "een verwerking met meer risico heeft meer aan te tonen: {eenvoudig_totaal} tegen {ingewikkeld_totaal}"
    );
}

#[test]
fn geen_enkele_melding_leest_als_een_verwijt() {
    let v = Verwerking::nieuw("X", "X", Rol::Verwerkingsverantwoordelijke, "e", "u1", nu());
    for o in v.volledigheid().ontbreekt {
        assert!(!o.omschrijving.contains("verplicht veld"), "kreeg: {}", o.omschrijving);
        assert!(!o.omschrijving.contains("fout"), "kreeg: {}", o.omschrijving);
        assert!(!o.grondslag.is_empty(), "{} mist een grondslag", o.veld);
        // Elke melding vertelt wat te doen, in de gebiedende wijs.
        assert!(
            o.omschrijving.len() > 15,
            "{} is te kort om iets uit te leggen: {}",
            o.veld,
            o.omschrijving
        );
    }
}

#[test]
fn verwerking_overleeft_serialisatie() {
    let mut v = basisverwerking();
    v.bijzondere_categorieen.push(BijzondereCategorie::Gezondheidsgegevens);
    v.uitzondering_artikel9 = Some(UitzonderingArtikel9::Gezondheidszorg);
    v.uitzondering_nationale_bepaling = Some("art. 30 lid 3 onder a UAVG".into());
    v.stel_vast("u2", nu()).unwrap();

    let json = serde_json::to_string(&v).unwrap();
    let terug: Verwerking = serde_json::from_str(&json).unwrap();
    assert_eq!(v, terug);
    assert_eq!(terug.volledigheid(), v.volledigheid());
}

// --------------------------------------------------------------------------
// Invarianten van de teller
// --------------------------------------------------------------------------

/// De teller mag nooit meer compleet melden dan er werkelijk is ingevuld.
///
/// Deze invariant heeft al één fout gevangen: de motivering werd alleen geteld
/// wanneer de grondslag al was gekozen, waardoor een leeg record "1 van de 8"
/// meldde. Precies de misleiding die dit mechanisme moet voorkomen.
#[test]
fn de_teller_klopt_bij_elke_combinatie() {
    let leeg = Verwerking::nieuw("X", "X", Rol::Verwerkingsverantwoordelijke, "e", "u1", nu());
    let r = leeg.volledigheid();
    assert_eq!(r.compleet, 0, "een leeg record heeft niets compleet, kreeg: {}", r.teller());
    assert_eq!(r.percentage(), 0);

    // Elke combinatie van antwoorden moet een sluitende telling opleveren.
    for bijzonder in [false, true] {
        for verwerker in [false, true] {
            for buiten_eer in [false, true] {
                for grondslag in Grondslag::alle() {
                    let mut v = Verwerking::nieuw(
                        "X",
                        "X",
                        Rol::Verwerkingsverantwoordelijke,
                        "e",
                        "u1",
                        nu(),
                    );
                    v.grondslag = Some(grondslag);
                    if bijzonder {
                        v.bijzondere_categorieen.push(BijzondereCategorie::Gezondheidsgegevens);
                        v.uitzondering_artikel9 = Some(UitzonderingArtikel9::Gezondheidszorg);
                    }
                    if verwerker || buiten_eer {
                        v.ontvangers.push(Ontvanger {
                            omschrijving: "derde".into(),
                            is_verwerker: verwerker,
                            leverancier_id: None,
                            buiten_eer,
                        });
                    }
                    let r = v.volledigheid();
                    assert!(
                        r.ontbreekt.len() <= r.verplicht,
                        "er ontbreken {} onderdelen van {} verplichte (grondslag {:?}, bijzonder {bijzonder}, \
                         verwerker {verwerker}, buiten EER {buiten_eer})",
                        r.ontbreekt.len(),
                        r.verplicht,
                        grondslag
                    );
                    assert!(r.percentage() <= 100);
                }
            }
        }
    }
}

/// Een volledig ingevulde verwerking meldt honderd procent, in elke variant.
#[test]
fn een_volledige_verwerking_meldt_honderd_procent() {
    let mut v = basisverwerking();
    assert_eq!(v.volledigheid().percentage(), 100, "kreeg: {:?}", v.volledigheid().ontbreekt);

    // Ook met alle complicaties erbij.
    v.bijzondere_categorieen.push(BijzondereCategorie::Gezondheidsgegevens);
    v.uitzondering_artikel9 = Some(UitzonderingArtikel9::Gezondheidszorg);
    v.uitzondering_nationale_bepaling = Some("art. 30 lid 3 onder a UAVG".into());
    v.burgerservicenummer = true;
    v.bsn_grondslag = Some("art. 46 UAVG".into());
    v.strafrechtelijke_gegevens = true;
    v.uitzondering_strafrechtelijk = Some("art. 33 UAVG".into());
    v.ontvangers.push(Ontvanger {
        omschrijving: "arbodienst".into(),
        is_verwerker: true,
        leverancier_id: None,
        buiten_eer: true,
    });
    v.verwerkersovereenkomsten.push(Id::nieuw());
    v.doorgiften.push(Id::nieuw());
    v.dpia_id = Some(Id::nieuw());

    let r = v.volledigheid();
    assert_eq!(r.percentage(), 100, "kreeg: {:?}", r.ontbreekt);
    assert!(r.mag_vaststellen());
}

/// Geen enkel ontbrekend onderdeel wordt twee keer gemeld.
#[test]
fn geen_dubbele_meldingen() {
    let mut v = Verwerking::nieuw("X", "X", Rol::GezamenlijkVerantwoordelijke, "e", "u1", nu());
    v.grondslag = Some(Grondslag::GerechtvaardigdBelang);
    v.bijzondere_categorieen.push(BijzondereCategorie::Gezondheidsgegevens);
    v.burgerservicenummer = true;
    v.strafrechtelijke_gegevens = true;
    v.uitsluitend_geautomatiseerde_besluitvorming = true;
    v.ontvangers.push(Ontvanger {
        omschrijving: "derde".into(),
        is_verwerker: true,
        leverancier_id: None,
        buiten_eer: true,
    });

    let velden: Vec<_> = v.volledigheid().ontbreekt.iter().map(|o| o.veld.clone()).collect();
    let uniek: std::collections::BTreeSet<_> = velden.iter().collect();
    assert_eq!(velden.len(), uniek.len(), "dubbele meldingen: {velden:?}");
}
