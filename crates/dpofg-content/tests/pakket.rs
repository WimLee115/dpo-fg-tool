//! Wat er bij het installeren van een kennispakket wordt tegengehouden.

use chrono::{NaiveDate, TimeZone, Utc};
use dpofg_content::{
    nieuw_uitgeverspaar, pakket::Instrumentstatus, startpakket, ContentFout, Kennispakket,
};

fn datum(j: i32, m: u32, d: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(j, m, d).unwrap()
}

fn pakket() -> dpofg_content::Pakketinhoud {
    startpakket(datum(2026, 8, 18))
}

#[test]
fn een_ondertekend_pakket_van_een_vertrouwde_uitgever_wordt_aanvaard() {
    let sleutel = nieuw_uitgeverspaar();
    let p = Kennispakket::onderteken(pakket(), &sleutel).unwrap();
    let vertrouwd = vec![p.uitgever.clone()];
    assert!(p.controleer(&vertrouwd).is_ok());
}

/// Een handtekening die alleen klopt met de bijgeleverde sleutel bewijst niets:
/// die sleutel komt uit hetzelfde bestand.
#[test]
fn een_pakket_van_een_onbekende_uitgever_wordt_geweigerd() {
    let p = Kennispakket::onderteken(pakket(), &nieuw_uitgeverspaar()).unwrap();
    let andere_uitgever = hex::encode(nieuw_uitgeverspaar().verifying_key().to_bytes());

    let fout = p.controleer(&[andere_uitgever]).unwrap_err();
    assert!(matches!(fout, ContentFout::OnbekendeUitgever { .. }));
    assert!(fout.to_string().contains("vertrouwde uitgever"));
}

#[test]
fn gewijzigde_inhoud_verbreekt_de_handtekening() {
    let sleutel = nieuw_uitgeverspaar();
    let mut p = Kennispakket::onderteken(pakket(), &sleutel).unwrap();
    let vertrouwd = vec![p.uitgever.clone()];

    // Verkort de 72-uurstermijn naar 24 uur: precies het soort wijziging dat
    // een organisatie te vroeg zou laten melden of juist te laat.
    let index = p.inhoud.termijnen.iter().position(|t| t.code == "AVG-33-MELDING").unwrap();
    p.inhoud.termijnen[index].duur = 24;

    let fout = p.controleer(&vertrouwd).unwrap_err();
    assert!(matches!(fout, ContentFout::OngeldigeHandtekening(_)));
    assert!(fout.to_string().contains("gewijzigd"));
}

#[test]
fn een_gewijzigde_consolidatiedatum_verbreekt_de_handtekening() {
    let sleutel = nieuw_uitgeverspaar();
    let mut p = Kennispakket::onderteken(pakket(), &sleutel).unwrap();
    let vertrouwd = vec![p.uitgever.clone()];
    p.inhoud.consolidatiedatum = datum(2027, 1, 1);
    assert!(p.controleer(&vertrouwd).is_err());
}

/// Terugrollen van juridische inhoud is een aanval, geen vergissing.
#[test]
fn terugrollen_naar_een_oudere_versie_wordt_geweigerd() {
    let sleutel = nieuw_uitgeverspaar();

    let mut nieuw = pakket();
    nieuw.versie = 5;
    nieuw.versienaam = "2026.5".into();

    let mut oud = pakket();
    oud.versie = 3;
    oud.versienaam = "2026.3".into();

    let oud_pakket = Kennispakket::onderteken(oud, &sleutel).unwrap();
    let fout = oud_pakket.controleer_volgorde(Some(&nieuw)).unwrap_err();

    assert!(matches!(fout, ContentFout::Terugrol { .. }));
    assert!(fout.to_string().contains("2026.3"));
    assert!(fout.to_string().contains("2026.5"));
    assert!(fout.to_string().contains("niet meer geldt"));
}

#[test]
fn dezelfde_versie_opnieuw_installeren_mag() {
    let sleutel = nieuw_uitgeverspaar();
    let huidig = pakket();
    let p = Kennispakket::onderteken(pakket(), &sleutel).unwrap();
    assert!(p.controleer_volgorde(Some(&huidig)).is_ok());
}

#[test]
fn een_pakket_met_een_andere_code_is_geen_terugrol() {
    let sleutel = nieuw_uitgeverspaar();
    let mut huidig = pakket();
    huidig.versie = 9;

    let mut ander = pakket();
    ander.code = "nl-zorg".into();
    ander.versie = 1;

    let p = Kennispakket::onderteken(ander, &sleutel).unwrap();
    assert!(p.controleer_volgorde(Some(&huidig)).is_ok());
}

// --------------------------------------------------------------------------
// De inhoud opzoeken
// --------------------------------------------------------------------------

#[test]
fn termijnen_zijn_op_code_te_vinden() {
    let p = pakket();
    let melding = p.termijn("AVG-33-MELDING").unwrap();
    assert_eq!(melding.duur, 72);
    assert!(melding.eenheid.is_urentermijn());
    assert!(!melding.opschortbaar, "een meldtermijn is niet opschortbaar");

    let verzoek = p.termijn("AVG-12-3-VERZOEK").unwrap();
    assert!(verzoek.verlenging.is_some());
    assert!(
        verzoek.verlenging.as_ref().unwrap().bericht_binnen_oorspronkelijke_termijn,
        "het verlengingsbericht moet binnen de eerste maand"
    );

    assert!(p.termijn("BESTAAT-NIET").is_err());
}

#[test]
fn de_kalender_is_op_jurisdictie_te_vinden() {
    let p = pakket();
    let k = p.kalender("NL").unwrap();
    assert_eq!(k.dekking_van, 2026);
    assert!(k.is_feestdag(datum(2026, 12, 25)));
    assert!(p.kalender("BE").is_err());
}

#[test]
fn rechtsfeiten_zijn_op_code_te_vinden() {
    let p = pakket();
    assert_eq!(p.rechtsfeit("AVG-IWT").unwrap().datum, datum(2018, 5, 25));
    assert!(p.rechtsfeit("ONBEKEND").is_err());
}

#[test]
fn instrumenten_die_om_herbeoordeling_vragen_worden_opgesomd() {
    let mut p = pakket();
    assert!(p.instrumenten_met_herbeoordeling().is_empty());

    p.doorgifteinstrumenten[0].status = Instrumentstatus::Ingetrokken;
    assert_eq!(p.instrumenten_met_herbeoordeling().len(), 1);

    p.doorgifteinstrumenten[1].status = Instrumentstatus::OnderToetsing;
    assert_eq!(p.instrumenten_met_herbeoordeling().len(), 2);
}

// --------------------------------------------------------------------------
// Ouderdom
// --------------------------------------------------------------------------

#[test]
fn een_verouderd_pakket_wordt_gemeld_maar_blokkeert_niet() {
    let p = pakket();
    let nu = Utc.with_ymd_and_hms(2027, 8, 18, 9, 0, 0).unwrap();

    assert_eq!(p.ouderdom_in_dagen(nu), 365);
    assert!(p.controleer_ouderdom(nu, 400).is_ok());

    let fout = p.controleer_ouderdom(nu, 180).unwrap_err();
    assert!(matches!(fout, ContentFout::Verouderd { .. }));
    assert!(fout.to_string().contains("365 dagen geleden"));
    // De inhoud blijft bruikbaar; alleen de melding verschijnt.
    assert!(p.termijn("AVG-33-MELDING").is_ok());
}

// --------------------------------------------------------------------------
// Het startpakket draagt zijn eigen voorbehoud
// --------------------------------------------------------------------------

/// Het startpakket mag geen juridische zekerheid suggereren die het niet heeft.
#[test]
fn het_startpakket_waarschuwt_over_zichzelf() {
    let p = pakket();
    assert!(p.naam.contains("verifiëren"), "de naam hoort het voorbehoud te dragen");

    let waarschuwing = p.aanvullend.get("waarschuwing").expect("waarschuwing ontbreekt");
    let strekking = waarschuwing["strekking"].as_str().unwrap();
    assert!(strekking.contains("niet door een jurist vastgesteld"));
    assert!(strekking.contains("Verifieer"));

    let lijst = waarschuwing["te_verifieren"].as_array().unwrap();
    assert!(lijst.len() >= 5, "de lijst met te verifiëren onderdelen is te dun");
}

#[test]
fn elke_termijn_draagt_een_grondslag() {
    for t in pakket().termijnen {
        assert!(!t.grondslag.is_empty(), "{} mist een grondslag", t.code);
        assert!(!t.naam.is_empty(), "{} mist een naam", t.code);
        assert!(t.duur > 0, "{} heeft duur 0", t.code);
    }
}

#[test]
fn zelf_gestelde_termijnen_zijn_als_zodanig_gemarkeerd() {
    let p = pakket();
    let intern = p.termijn("INTERN-REGISTERHERZIENING").unwrap();
    assert_eq!(intern.stelsel, dpofg_terms::Rechtsstelsel::ZelfGesteld);
    assert!(intern.grondslag.contains("geen wettelijke termijn"));
}

#[test]
fn het_pakket_overleeft_serialisatie() {
    let sleutel = nieuw_uitgeverspaar();
    let p = Kennispakket::onderteken(pakket(), &sleutel).unwrap();
    let vertrouwd = vec![p.uitgever.clone()];

    let json = serde_json::to_string(&p).unwrap();
    let terug: Kennispakket = serde_json::from_str(&json).unwrap();

    assert_eq!(p, terug);
    assert!(terug.controleer(&vertrouwd).is_ok(), "de handtekening moet serialisatie overleven");
}

/// De acht weken, de verlenging met zes weken, de berichttermijn en de
/// opschortingsgrond staan alle in artikel 36 lid 2. Lid 3 somt op welke
/// stukken bij het verzoek gaan. Deze tekst reist via de verantwoording mee naar
/// elk dossier, dus een verkeerde verwijzing komt bij een toezichthouder op
/// tafel.
#[test]
fn de_raadplegingstermijn_verwijst_naar_het_tweede_lid() {
    let p = pakket();
    let t = p.termijn("AVG-36-RAADPLEGING").unwrap();
    assert_eq!(t.grondslag, "art. 36 lid 2 AVG");
    let verlenging = t.verlenging.as_ref().expect("de termijn is verlengbaar");
    assert!(verlenging.grondslag.starts_with("art. 36 lid 2"), "kreeg: {}", verlenging.grondslag);
    assert!(t.opschortbaar, "art. 36 lid 2 kent een opschortingsgrond");
}

/// De kalender is een tikkende randvoorwaarde: valt een berekening buiten de
/// dekking, dan weigert de termijnenmotor te rekenen. Dat is juist gedrag, maar
/// het mag niet gebeuren op het moment dat iemand een termijn van acht weken
/// indient.
///
/// Deze test meet tegen de wandklok, en dat is hier uitdrukkelijk de bedoeling.
/// Hij mat eerder het verschil tussen twee vaste getallen uit het pakket zelf —
/// dekking_van en dekking_tot_en_met — en dat verschil verandert nooit. Daarmee
/// bleef de test tot in lengte van jaren groen terwijl de dekking onder hem
/// wegliep: in 2029 zou hij nog steeds "vier jaar dekking" melden over een
/// kalender die het lopende jaar nauwelijks nog haalt. Een test die zwijgt op
/// het moment dat hij zou moeten spreken, is erger dan geen test.
///
/// Wat hij nu doet, is een wekker zetten. Hij gaat af zodra de kalender minder
/// dan drie jaar vooruit reikt — ruim vóór het moment waarop een termijn van
/// acht weken tegen het jaareinde buiten de dekking valt, en met genoeg
/// aanloop om er een nieuwe kalender bij te zoeken. Gaat hij af, dan is dat
/// geen defect maar de melding waarvoor hij bestaat: het kennispakket moet
/// worden bijgewerkt.
#[test]
fn de_kalender_reikt_ver_genoeg_vooruit_vanaf_vandaag() {
    use chrono::Datelike;
    let kalender = pakket().kalender("NL").unwrap().clone();
    let dit_jaar = chrono::Utc::now().year();
    assert!(
        kalender.dekking_tot_en_met >= dit_jaar + 3,
        "de kalender dekt {} tot en met {}, en het is nu {}; dat is te kort voor een termijn \
         van acht weken die tegen het jaareinde wordt gestart. Werk de feestdagen in het \
         kennispakket bij",
        kalender.dekking_van,
        kalender.dekking_tot_en_met,
        dit_jaar
    );
}

/// Elk jaar in de dekking draagt de vaste feestdagen. Een jaar dat stilzwijgend
/// zonder Kerstmis in de kalender staat, levert een deadline op die een werkdag
/// te vroeg valt.
#[test]
fn elk_gedekt_jaar_draagt_de_vaste_feestdagen() {
    use chrono::NaiveDate;
    let kalender = pakket().kalender("NL").unwrap().clone();
    for jaar in kalender.dekking_van..=kalender.dekking_tot_en_met {
        for (maand, dag, naam) in [
            (1, 1, "Nieuwjaarsdag"),
            (4, 27, "Koningsdag"),
            (12, 25, "Eerste Kerstdag"),
            (12, 26, "Tweede Kerstdag"),
        ] {
            let datum = NaiveDate::from_ymd_opt(jaar, maand, dag).unwrap();
            // Koningsdag schuift naar 26 april wanneer 27 april op zondag valt.
            let alternatief = NaiveDate::from_ymd_opt(jaar, 4, 26).unwrap();
            let aanwezig = kalender.dagen.contains(&datum)
                || (maand == 4 && dag == 27 && kalender.dagen.contains(&alternatief));
            assert!(aanwezig, "{jaar} mist {naam}");
        }
    }
}
