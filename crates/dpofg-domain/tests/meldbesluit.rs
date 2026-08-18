//! De zwaarst beveiligde beslissing in het product: wel of niet melden.
//!
//! Uit paragraaf 2.3 van het foutbestendigheidshoofdstuk. Elke test hier
//! beschrijft een manier waarop een datalek onterecht niet gemeld zou kunnen
//! worden, en de laag die dat tegenhoudt.

use chrono::{DateTime, Duration, TimeZone, Utc};
use dpofg_domain::{
    incident::Herkomstkanaal, Aantasting, DomeinFout, Incident, Meldbesluit, Motivering,
    Risiconiveau, Volledig,
};

fn t(dag: u32, uur: u32, minuut: u32) -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 8, dag, uur, minuut, 0).unwrap()
}

fn motivering(tekst: &str) -> Motivering {
    Motivering::nieuw(tekst, "u1", t(18, 10, 0)).unwrap()
}

/// Een incident dat zover is ingevuld dat het besluit genomen kan worden.
fn incident() -> Incident {
    let mut i = Incident::nieuw(
        "2026-0041",
        "verkeerd geadresseerde brief",
        t(18, 9, 0),
        t(18, 9, 30),
        Herkomstkanaal::InternVastgesteld,
        "u1",
        "u1",
    );
    i.stel_kennisname_vast(t(18, 9, 20), None).unwrap();
    i.aantasting =
        Aantasting { vertrouwelijkheid: true, integriteit: false, beschikbaarheid: false };
    i.categorieen_gegevens = vec!["naam".into(), "adres".into()];
    i.aantal_betrokkenen = Some(1);
    i.exfiltratie_uitgesloten = Some(false);
    i.risiconiveau = Some(Risiconiveau::GeenRisico);
    i.risicoweging = Some(motivering(
        "eenmalige brief aan een verkeerd adres, ontvanger heeft de brief ongeopend geretourneerd",
    ));
    i
}

// --------------------------------------------------------------------------
// De volgorde: eerst wegen, dan besluiten
// --------------------------------------------------------------------------

#[test]
fn zonder_weging_geen_besluit() {
    let mut i = incident();
    i.risiconiveau = None;
    let fout = i
        .besluit_niet_melden(
            motivering("het lijkt me niet nodig"),
            Some("u2".into()),
            t(18, 10, 0),
            Duration::zero(),
        )
        .unwrap_err();
    assert!(matches!(fout, DomeinFout::OngeldigeWaarde { .. }));
    assert!(fout.to_string().contains("weeg eerst het risico"));
}

/// Controleregel LEK-08: wie een risico vaststelt en vervolgens niet meldt,
/// spreekt zichzelf tegen. Dat wordt geblokkeerd, niet gesignaleerd.
#[test]
fn een_besluit_dat_de_weging_tegenspreekt_wordt_geblokkeerd() {
    for niveau in [Risiconiveau::Risico, Risiconiveau::HoogRisico] {
        let mut i = incident();
        i.risiconiveau = Some(niveau);
        let fout = i
            .besluit_niet_melden(
                motivering("wij achten melding niet opportuun in dit geval"),
                Some("u2".into()),
                t(18, 10, 0),
                Duration::zero(),
            )
            .unwrap_err();
        assert!(fout.to_string().contains("niet te rijmen met de weging"), "kreeg: {fout}");
    }
}

/// Controleregel LEK-09: het beschikbaarheidsaspect wordt in de praktijk het
/// vaakst vergeten.
#[test]
fn zonder_beoordeling_van_de_aantasting_geen_besluit() {
    let mut i = incident();
    i.aantasting =
        Aantasting { vertrouwelijkheid: false, integriteit: false, beschikbaarheid: false };
    let fout = i
        .besluit_niet_melden(
            motivering("er is niets aan de hand"),
            Some("u2".into()),
            t(18, 10, 0),
            Duration::zero(),
        )
        .unwrap_err();
    assert!(fout.to_string().contains("beschikbaarheid"), "kreeg: {fout}");
}

#[test]
fn zonder_antwoord_over_exfiltratie_geen_besluit() {
    let mut i = incident();
    i.exfiltratie_uitgesloten = None;
    let fout = i
        .besluit_niet_melden(
            motivering("wij denken dat het meevalt"),
            Some("u2".into()),
            t(18, 10, 0),
            Duration::zero(),
        )
        .unwrap_err();
    assert!(fout.to_string().contains("uit te sluiten"), "kreeg: {fout}");
}

// --------------------------------------------------------------------------
// De drie lagen onder "niet melden"
// --------------------------------------------------------------------------

/// Laag 1: een motivering die niets zegt, wordt niet aangenomen.
#[test]
fn een_lege_motivering_wordt_niet_aangenomen() {
    assert!(Motivering::nieuw("nee", "u1", t(18, 10, 0)).is_err());
    assert!(Motivering::nieuw("", "u1", t(18, 10, 0)).is_err());
}

/// Laag 2 of 3: er moet ten minste één van beide zijn.
#[test]
fn zonder_tweede_persoon_en_zonder_afkoelperiode_wordt_geweigerd() {
    let mut i = incident();
    let fout = i
        .besluit_niet_melden(
            motivering("brief ongeopend retour ontvangen, geen kennisname door derden"),
            None,
            t(18, 10, 0),
            Duration::zero(),
        )
        .unwrap_err();
    assert!(matches!(fout, DomeinFout::TweedePersoonVereist { .. }));
    assert!(fout.to_string().contains("afkoelperiode"), "kreeg: {fout}");
}

/// Een eenpersoons-functie kan met de afkoelperiode werken. Dat is bewust een
/// zwakkere laag, en het gebruik ervan is te tellen.
#[test]
fn de_afkoelperiode_werkt_als_er_geen_tweede_persoon_is() {
    let mut i = incident();
    i.besluit_niet_melden(
        motivering("brief ongeopend retour ontvangen, geen kennisname door derden"),
        None,
        t(18, 10, 0),
        Duration::hours(24),
    )
    .unwrap();

    assert!(i.meldbesluit.is_niet_melden());
    assert!(!i.meldbesluit.is_definitief(t(18, 20, 0)), "binnen de afkoelperiode nog niet");
    assert!(i.meldbesluit.is_definitief(t(19, 11, 0)), "daarna wel");
}

/// Controleregel LEK-07: bij bijzondere gegevens, een burgerservicenummer of
/// financiële gegevens vervalt de afkoelperiode als alternatief.
#[test]
fn bij_gevoelige_gegevens_is_de_tweede_persoon_verplicht() {
    for (label, zet) in [
        ("bijzondere persoonsgegevens", 0),
        ("het burgerservicenummer", 1),
        ("financiële gegevens", 2),
    ] {
        let mut i = incident();
        match zet {
            0 => i.bijzondere_gegevens = true,
            1 => i.burgerservicenummer = true,
            _ => i.financiele_gegevens = true,
        }
        assert!(i.tweede_persoon_verplicht());

        // Ook met een ruime afkoelperiode wordt het geweigerd.
        let fout = i
            .besluit_niet_melden(
                motivering("wij achten het risico verwaarloosbaar gezien de omstandigheden"),
                None,
                t(18, 10, 0),
                Duration::days(7),
            )
            .unwrap_err();
        assert!(fout.to_string().contains(label), "kreeg: {fout}");
        assert!(fout.to_string().contains("volstaat een afkoelperiode niet"));

        // Met een tweede persoon mag het wel.
        let mut i2 = i.clone();
        i2.besluit_niet_melden(
            motivering("wij achten het risico verwaarloosbaar gezien de omstandigheden"),
            Some("u2".into()),
            t(18, 10, 0),
            Duration::zero(),
        )
        .unwrap();
        assert!(i2.meldbesluit.is_definitief(t(18, 10, 1)));
    }
}

/// Controleregel LEK-06: "geen risico" bij een grote groep vraagt om tegenspraak.
#[test]
fn grote_omvang_vraagt_om_tegenspraak() {
    let mut i = incident();
    assert!(!i.omvang_vereist_tegenspraak());
    i.aantal_betrokkenen = Some(251);
    assert!(i.omvang_vereist_tegenspraak());
}

// --------------------------------------------------------------------------
// Omkeren: de maat waaraan af te lezen is of de barrière werkt
// --------------------------------------------------------------------------

#[test]
fn een_besluit_kan_worden_omgekeerd() {
    let mut i = incident();
    i.besluit_niet_melden(
        motivering("brief ongeopend retour ontvangen, geen kennisname door derden"),
        Some("u2".into()),
        t(18, 10, 0),
        Duration::zero(),
    )
    .unwrap();

    i.keer_besluit_om(motivering(
        "bij nader inzien is niet vast te stellen of de brief geopend is geweest",
    ))
    .unwrap();
    assert!(matches!(i.meldbesluit, Meldbesluit::Melden { .. }));
}

#[test]
fn omkeren_kan_alleen_wat_er_ligt() {
    let mut i = incident();
    assert!(i.keer_besluit_om(motivering("zomaar een omkering zonder besluit")).is_err());
}

// --------------------------------------------------------------------------
// De ankers: randgevallen T-08 en T-31
// --------------------------------------------------------------------------

/// T-08: kennisname vóór het optreden van het incident is een invoerfout.
#[test]
fn t08_kennisname_voor_het_incident_wordt_geweigerd() {
    let mut i = incident();
    i.opgetreden_op = Some(t(18, 12, 0));
    let fout = i.stel_kennisname_vast(t(18, 9, 0), None).unwrap_err();
    assert!(matches!(fout, DomeinFout::OnmogelijkTijdstip { .. }));
    assert!(fout.to_string().contains("verwisseld"), "de melding hoort te helpen: {fout}");
}

#[test]
fn kennisname_voor_het_eerste_signaal_wordt_geweigerd() {
    let mut i = incident();
    assert!(i.stel_kennisname_vast(t(18, 8, 0), None).is_err());
}

/// T-31: bij een melding van een verwerker start de klok bij ontvangst van die
/// melding, niet bij het optreden bij de verwerker. Beide worden vastgelegd.
#[test]
fn t31_de_klok_start_bij_ontvangst_van_de_verwerkersmelding() {
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

    assert_eq!(i.anker_meldklok(), Some(t(20, 9, 0)), "het anker is de ontvangst van de melding");
    assert_eq!(
        i.incident_bij_verwerker_op,
        Some(t(17, 3, 0)),
        "het eerdere moment blijft vastgelegd, maar telt niet"
    );
}

#[test]
fn bij_een_intern_incident_is_kennisname_het_anker() {
    let i = incident();
    assert_eq!(i.anker_meldklok(), Some(t(18, 9, 20)));
}

// --------------------------------------------------------------------------
// De verificatieperiode en de registratievertraging
// --------------------------------------------------------------------------

#[test]
fn de_verificatieperiode_is_meetbaar() {
    let i = incident();
    assert_eq!(i.verificatieduur(), Some(Duration::minutes(20)));
}

/// Controleregel LEK-03: het gat tussen kennisname en registratie.
#[test]
fn de_registratievertraging_is_meetbaar() {
    let i = incident();
    assert_eq!(i.registratievertraging(), Some(Duration::minutes(10)));

    let mut traag = Incident::nieuw(
        "2026-0043",
        "laat geregistreerd",
        t(18, 8, 0),
        t(18, 20, 0),
        Herkomstkanaal::InternVastgesteld,
        "u1",
        "u1",
    );
    traag.stel_kennisname_vast(t(18, 9, 0), None).unwrap();
    assert!(traag.registratievertraging().unwrap() > Duration::hours(4));
}

// --------------------------------------------------------------------------
// Volledigheid van het dossier
// --------------------------------------------------------------------------

#[test]
fn een_incident_is_pas_af_met_oorzaak_en_maatregel() {
    let mut i = incident();
    i.besluit_niet_melden(
        motivering("brief ongeopend retour ontvangen, geen kennisname door derden"),
        Some("u2".into()),
        t(18, 10, 0),
        Duration::zero(),
    )
    .unwrap();

    let r = i.volledigheid();
    assert!(!r.mag_vaststellen());
    let velden: Vec<_> = r.ontbreekt.iter().map(|o| o.veld.as_str()).collect();
    assert!(velden.contains(&"incident.oorzaakcategorie"));
    assert!(velden.contains(&"incident.maatregelen"));

    i.oorzaakcategorie = Some("verzending naar verkeerde geadresseerde".into());
    i.maatregelen.push(dpofg_domain::Id::nieuw());
    assert!(i.volledigheid().is_volledig(), "kreeg: {:?}", i.volledigheid().ontbreekt);
}

#[test]
fn een_leeg_incident_meldt_alles_wat_ontbreekt() {
    let i = Incident::nieuw(
        "X",
        "X",
        t(18, 9, 0),
        t(18, 9, 0),
        Herkomstkanaal::InternVastgesteld,
        "u1",
        "u1",
    );
    let r = i.volledigheid();
    assert_eq!(r.compleet, 0, "kreeg: {}", r.teller());
    assert_eq!(r.ontbreekt.len(), 10);
    for o in &r.ontbreekt {
        assert!(!o.grondslag.is_empty(), "{} mist een grondslag", o.veld);
    }
}

#[test]
fn het_incident_valt_standaard_in_het_vertrouwelijke_compartiment() {
    let i = incident();
    assert_eq!(i.compartiment.naam(), "vertrouwelijk");
}
