//! De opslaglaag: versleuteling, versiegeschiedenis en het onuitschakelbare logboek.

use chrono::{DateTime, TimeZone, Utc};
use dpofg_audit::{Actor, Ankerstatus, Gebeurtenis, Handeling};
use dpofg_crypto::{kdf::KdfParameters, Wachtwoordzin};
use dpofg_store::{Kluis, StoreFout};
use serde::{Deserialize, Serialize};
use tempfile::TempDir;

const TEST: KdfParameters = KdfParameters::TEST_ONVEILIG;

fn t(uur: u32) -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 8, 18, uur, 0, 0).unwrap()
}

fn ww() -> Wachtwoordzin {
    Wachtwoordzin::nieuw("een voldoende lange wachtwoordzin")
}

fn actor() -> Actor {
    Actor::nieuw("u1", "A. de Vries", "fg")
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct Dossier {
    naam: String,
    bsn: String,
    aantekening: String,
}

fn dossier() -> Dossier {
    Dossier {
        naam: "J. Jansen".into(),
        bsn: "123456782".into(),
        aantekening: "verzoek om inzage ontvangen".into(),
    }
}

/// Maakt een kluis in een tijdelijke map. De map wordt opgeruimd zodra de
/// teruggegeven waarde wordt losgelaten.
fn nieuwe_kluis() -> (TempDir, Kluis) {
    let map = TempDir::new().unwrap();
    let pad = map.path().join("test.dpofg");
    let kluis = Kluis::aanmaken(&pad, &ww(), TEST, t(9)).unwrap();
    (map, kluis)
}

// --------------------------------------------------------------------------
// Aanmaken en openen
// --------------------------------------------------------------------------

#[test]
fn aanmaken_en_heropenen() {
    let map = TempDir::new().unwrap();
    let pad = map.path().join("test.dpofg");

    {
        let mut k = Kluis::aanmaken(&pad, &ww(), TEST, t(9)).unwrap();
        k.bewaar(
            "verzoek",
            "v1",
            "algemeen",
            "concept",
            Some("2026-0041"),
            &dossier(),
            &actor(),
            Handeling::RecordAangemaakt,
            "verzoek geregistreerd",
            t(9),
        )
        .unwrap();
    }

    let mut k = Kluis::openen(&pad, &ww(), t(10)).unwrap();
    k.compartiment_ontgrendelen("algemeen").unwrap();
    let terug: Dossier = k.laad("verzoek", "v1").unwrap();
    assert_eq!(terug, dossier());
}

#[test]
fn openen_met_verkeerd_wachtwoord_faalt() {
    let map = TempDir::new().unwrap();
    let pad = map.path().join("test.dpofg");
    Kluis::aanmaken(&pad, &ww(), TEST, t(9)).unwrap();

    let fout = Kluis::openen(&pad, &Wachtwoordzin::nieuw("een verkeerde wachtwoordzin"), t(10))
        .unwrap_err();
    assert!(matches!(fout, StoreFout::Crypto(_)));
    // De melding maakt geen onderscheid tussen een verkeerd wachtwoord en
    // gemanipuleerde gegevens.
    assert!(fout.to_string().contains("onjuiste sleutel of gewijzigde gegevens"));
}

#[test]
fn een_bestaand_bestand_wordt_niet_overschreven() {
    let map = TempDir::new().unwrap();
    let pad = map.path().join("test.dpofg");
    Kluis::aanmaken(&pad, &ww(), TEST, t(9)).unwrap();
    assert!(Kluis::aanmaken(&pad, &ww(), TEST, t(9)).is_err());
}

#[test]
fn een_vreemd_bestand_wordt_herkend() {
    let map = TempDir::new().unwrap();
    let pad = map.path().join("nietvanons.db");
    rusqlite::Connection::open(&pad).unwrap();
    let fout = Kluis::openen(&pad, &ww(), t(10)).unwrap_err();
    assert!(matches!(fout, StoreFout::GeenKluisbestand(_)));
}

// --------------------------------------------------------------------------
// Versleuteling
// --------------------------------------------------------------------------

/// De inhoud staat versleuteld op schijf; het bestand mag nergens de
/// klaartekst bevatten.
#[test]
fn het_bestand_bevat_de_gegevens_niet_in_leesbare_vorm() {
    let map = TempDir::new().unwrap();
    let pad = map.path().join("test.dpofg");
    {
        let mut k = Kluis::aanmaken(&pad, &ww(), TEST, t(9)).unwrap();
        k.bewaar(
            "verzoek",
            "v1",
            "algemeen",
            "concept",
            None,
            &dossier(),
            &actor(),
            Handeling::RecordAangemaakt,
            "verzoek geregistreerd",
            t(9),
        )
        .unwrap();
    }

    let ruw = std::fs::read(&pad).unwrap();
    for gevoelig in ["J. Jansen", "123456782", "verzoek om inzage ontvangen"] {
        assert!(
            !ruw.windows(gevoelig.len()).any(|w| w == gevoelig.as_bytes()),
            "'{gevoelig}' staat leesbaar in het bestand"
        );
    }
}

#[test]
fn een_gesloten_compartiment_is_onleesbaar() {
    let (_map, mut k) = nieuwe_kluis();
    k.compartiment_aanmaken("vertrouwelijk", "gevoelige dossiers", t(9)).unwrap();
    k.bewaar(
        "incident",
        "i1",
        "vertrouwelijk",
        "concept",
        None,
        &dossier(),
        &actor(),
        Handeling::RecordAangemaakt,
        "incident geregistreerd",
        t(9),
    )
    .unwrap();

    k.compartiment_vergrendelen("vertrouwelijk");
    let fout = k.laad::<Dossier>("incident", "i1").unwrap_err();
    assert!(matches!(fout, StoreFout::CompartimentGesloten(_)));
    assert!(fout.to_string().contains("niet ontgrendeld"));

    // De kop is wél zichtbaar: anders zou je niet eens zien dát er dossiers zijn.
    let lijst = k.lijst("incident").unwrap();
    assert_eq!(lijst.len(), 1);
    assert_eq!(lijst[0].compartiment, "vertrouwelijk");

    k.compartiment_ontgrendelen("vertrouwelijk").unwrap();
    assert_eq!(k.laad::<Dossier>("incident", "i1").unwrap(), dossier());
}

#[test]
fn een_onbekend_compartiment_wordt_gemeld() {
    let (_map, mut k) = nieuwe_kluis();
    let fout = k.compartiment_ontgrendelen("bestaatniet").unwrap_err();
    assert!(matches!(fout, StoreFout::OnbekendCompartiment(_)));
}

// --------------------------------------------------------------------------
// Versiegeschiedenis
// --------------------------------------------------------------------------

#[test]
fn niets_wordt_hard_overschreven() {
    let (_map, mut k) = nieuwe_kluis();

    let mut d = dossier();
    k.bewaar(
        "verzoek",
        "v1",
        "algemeen",
        "concept",
        None,
        &d,
        &actor(),
        Handeling::RecordAangemaakt,
        "aangemaakt",
        t(9),
    )
    .unwrap();

    d.aantekening = "identiteit vastgesteld".into();
    let v2 = k
        .bewaar(
            "verzoek",
            "v1",
            "algemeen",
            "concept",
            None,
            &d,
            &actor(),
            Handeling::RecordGewijzigd,
            "identiteit vastgesteld",
            t(10),
        )
        .unwrap();
    assert_eq!(v2, 2);

    d.aantekening = "afgehandeld".into();
    let v3 = k
        .bewaar(
            "verzoek",
            "v1",
            "algemeen",
            "vastgesteld",
            None,
            &d,
            &actor(),
            Handeling::RecordVastgesteld,
            "afgehandeld",
            t(11),
        )
        .unwrap();
    assert_eq!(v3, 3);

    assert_eq!(k.versies("v1").unwrap(), vec![1, 2]);
    let eerste: Dossier = k.laad_versie("verzoek", "v1", 1).unwrap();
    assert_eq!(eerste.aantekening, "verzoek om inzage ontvangen");
    let tweede: Dossier = k.laad_versie("verzoek", "v1", 2).unwrap();
    assert_eq!(tweede.aantekening, "identiteit vastgesteld");
    let huidig: Dossier = k.laad("verzoek", "v1").unwrap();
    assert_eq!(huidig.aantekening, "afgehandeld");
}

#[test]
fn de_aanmaakdatum_blijft_staan_bij_een_wijziging() {
    let (_map, mut k) = nieuwe_kluis();
    k.bewaar(
        "verzoek",
        "v1",
        "algemeen",
        "concept",
        None,
        &dossier(),
        &actor(),
        Handeling::RecordAangemaakt,
        "aangemaakt",
        t(9),
    )
    .unwrap();
    k.bewaar(
        "verzoek",
        "v1",
        "algemeen",
        "vastgesteld",
        None,
        &dossier(),
        &actor(),
        Handeling::RecordGewijzigd,
        "gewijzigd",
        t(14),
    )
    .unwrap();

    let kop = &k.lijst("verzoek").unwrap()[0];
    assert_eq!(kop.aangemaakt_op, t(9));
    assert_eq!(kop.gewijzigd_op, t(14));
    assert_eq!(kop.status, "vastgesteld");
}

// --------------------------------------------------------------------------
// Het logboek
// --------------------------------------------------------------------------

#[test]
fn elke_wijziging_landt_in_het_logboek() {
    let (_map, mut k) = nieuwe_kluis();
    let voor = k.ketenstand().volgnummer;

    k.bewaar(
        "verzoek",
        "v1",
        "algemeen",
        "concept",
        None,
        &dossier(),
        &actor(),
        Handeling::RecordAangemaakt,
        "aangemaakt",
        t(9),
    )
    .unwrap();
    k.bewaar(
        "verzoek",
        "v1",
        "algemeen",
        "vastgesteld",
        None,
        &dossier(),
        &actor(),
        Handeling::RecordVastgesteld,
        "vastgesteld",
        t(10),
    )
    .unwrap();

    assert_eq!(k.ketenstand().volgnummer, voor + 2);

    let regels = k.logboek_van("verzoek", "v1").unwrap();
    assert_eq!(regels.len(), 2);
    assert_eq!(regels[0].gebeurtenis.handeling, Handeling::RecordAangemaakt);
    assert_eq!(regels[1].gebeurtenis.handeling, Handeling::RecordVastgesteld);
    assert_eq!(regels[1].gebeurtenis.actor.naam, "A. de Vries");
}

#[test]
fn het_logboek_is_ongeschonden_na_gewoon_gebruik() {
    let (_map, mut k) = nieuwe_kluis();
    for n in 1..=5 {
        k.bewaar(
            "verzoek",
            &format!("v{n}"),
            "algemeen",
            "concept",
            None,
            &dossier(),
            &actor(),
            Handeling::RecordAangemaakt,
            "aangemaakt",
            t(9),
        )
        .unwrap();
    }
    let rapport = k.verifieer_logboek().unwrap();
    assert!(rapport.is_ongeschonden(), "kreeg: {:?}", rapport.bevindingen);
    assert!(rapport.reikwijdte().contains("geen anker"));
}

/// De database weigert zelf elke wijziging aan het logboek. Dat is de laatste
/// verdedigingslinie: ook wie de code omzeilt en rechtstreeks in het bestand
/// werkt, loopt hier tegenaan.
#[test]
fn het_logboek_weigert_wijziging_en_verwijdering() {
    let map = TempDir::new().unwrap();
    let pad = map.path().join("test.dpofg");
    {
        let mut k = Kluis::aanmaken(&pad, &ww(), TEST, t(9)).unwrap();
        k.bewaar(
            "verzoek",
            "v1",
            "algemeen",
            "concept",
            None,
            &dossier(),
            &actor(),
            Handeling::RecordAangemaakt,
            "aangemaakt",
            t(9),
        )
        .unwrap();
    }

    let conn = rusqlite::Connection::open(&pad).unwrap();
    let fout = conn
        .execute("UPDATE logboek SET handeling = 'vervalst' WHERE volgnummer = 1", [])
        .unwrap_err();
    assert!(fout.to_string().contains("append-only"), "kreeg: {fout}");

    let fout = conn.execute("DELETE FROM logboek WHERE volgnummer = 1", []).unwrap_err();
    assert!(fout.to_string().contains("append-only"), "kreeg: {fout}");
}

/// Een anker maakt afkappen van het logboek zichtbaar.
#[test]
fn een_anker_bevestigt_de_keten() {
    let (_map, mut k) = nieuwe_kluis();
    k.bewaar(
        "verzoek",
        "v1",
        "algemeen",
        "concept",
        None,
        &dossier(),
        &actor(),
        Handeling::RecordAangemaakt,
        "aangemaakt",
        t(9),
    )
    .unwrap();

    let sleutel = dpofg_audit::anker::nieuw_sleutelpaar();
    let anker = dpofg_audit::Anker::plaats(&sleutel, "kluis-1", k.ketenstand(), t(12))
        .unwrap()
        .met_bewaarplaats("notulen directieoverleg");
    k.anker_bewaren(&anker).unwrap();

    // Daarna gaat het werk door.
    k.bewaar(
        "verzoek",
        "v2",
        "algemeen",
        "concept",
        None,
        &dossier(),
        &actor(),
        Handeling::RecordAangemaakt,
        "aangemaakt",
        t(13),
    )
    .unwrap();

    let rapport = k.verifieer_logboek().unwrap();
    assert!(rapport.is_ongeschonden());
    assert!(matches!(rapport.ankerstatus, Ankerstatus::Bevestigd { .. }));
    assert!(rapport.reikwijdte().contains("bevestigd tot en met regel"));
    assert_eq!(
        k.laatste_anker().unwrap().unwrap().bewaarplaats.as_deref(),
        Some("notulen directieoverleg")
    );
}

// --------------------------------------------------------------------------
// Bijlagen
// --------------------------------------------------------------------------

#[test]
fn bijlagen_worden_inhoudsgeadresseerd_bewaard() {
    let (_map, mut k) = nieuwe_kluis();
    k.bewaar(
        "verzoek",
        "v1",
        "algemeen",
        "concept",
        None,
        &dossier(),
        &actor(),
        Handeling::RecordAangemaakt,
        "aangemaakt",
        t(9),
    )
    .unwrap();

    let inhoud = b"inhoud van de brief met persoonsgegevens";
    let hash = k.bijlage_toevoegen("v1", "algemeen", "brief.pdf", inhoud, &actor(), t(9)).unwrap();

    assert_eq!(k.bijlage_lezen(&hash).unwrap(), inhoud);

    // Dezelfde inhoud onder een andere naam levert dezelfde hash op en wordt
    // niet dubbel opgeslagen.
    let hash2 = k
        .bijlage_toevoegen("v1", "algemeen", "kopie van brief.pdf", inhoud, &actor(), t(10))
        .unwrap();
    assert_eq!(hash, hash2);
}

#[test]
fn een_bijlage_staat_versleuteld_op_schijf() {
    let map = TempDir::new().unwrap();
    let pad = map.path().join("test.dpofg");
    {
        let mut k = Kluis::aanmaken(&pad, &ww(), TEST, t(9)).unwrap();
        k.bewaar(
            "verzoek",
            "v1",
            "algemeen",
            "concept",
            None,
            &dossier(),
            &actor(),
            Handeling::RecordAangemaakt,
            "aangemaakt",
            t(9),
        )
        .unwrap();
        k.bijlage_toevoegen(
            "v1",
            "algemeen",
            "brief.pdf",
            b"strikt vertrouwelijke inhoud",
            &actor(),
            t(9),
        )
        .unwrap();
    }
    let ruw = std::fs::read(&pad).unwrap();
    let geheim = b"strikt vertrouwelijke inhoud";
    assert!(!ruw.windows(geheim.len()).any(|w| w == geheim));
}

// --------------------------------------------------------------------------
// Blinde index
// --------------------------------------------------------------------------

#[test]
fn zoeken_in_versleutelde_velden() {
    let (_map, mut k) = nieuwe_kluis();
    for (id, adres) in [("v1", "jan@example.nl"), ("v2", "piet@example.nl")] {
        k.bewaar(
            "verzoek",
            id,
            "algemeen",
            "concept",
            None,
            &dossier(),
            &actor(),
            Handeling::RecordAangemaakt,
            "aangemaakt",
            t(9),
        )
        .unwrap();
        k.indexeer(id, "algemeen", "betrokkene.emailadres", adres).unwrap();
    }

    let treffers = k.zoek_op_index("algemeen", "betrokkene.emailadres", "jan@example.nl").unwrap();
    assert_eq!(treffers, vec!["v1"]);

    // Normalisatie werkt door: hoofdletters en spaties maken niets uit.
    let treffers =
        k.zoek_op_index("algemeen", "betrokkene.emailadres", " JAN@Example.NL ").unwrap();
    assert_eq!(treffers, vec!["v1"]);

    assert!(k
        .zoek_op_index("algemeen", "betrokkene.emailadres", "kees@example.nl")
        .unwrap()
        .is_empty());
}

/// Een index op een veld met weinig mogelijke waarden wordt geweigerd: hij zou
/// verklappen welke records dezelfde waarde delen.
#[test]
fn een_index_op_een_veld_met_lage_variatie_wordt_geweigerd() {
    let (_map, mut k) = nieuwe_kluis();
    k.bewaar(
        "incident",
        "i1",
        "algemeen",
        "concept",
        None,
        &dossier(),
        &actor(),
        Handeling::RecordAangemaakt,
        "aangemaakt",
        t(9),
    )
    .unwrap();
    let fout = k.indexeer("i1", "algemeen", "incident.risiconiveau", "hoog").unwrap_err();
    assert!(fout.to_string().contains("dezelfde waarde delen"), "kreeg: {fout}");
}

// --------------------------------------------------------------------------
// Wachtwoord wijzigen
// --------------------------------------------------------------------------

#[test]
fn wachtwoord_wijzigen_herversleutelt_niets() {
    let map = TempDir::new().unwrap();
    let pad = map.path().join("test.dpofg");
    let nieuw = Wachtwoordzin::nieuw("een heel andere wachtwoordzin");

    {
        let mut k = Kluis::aanmaken(&pad, &ww(), TEST, t(9)).unwrap();
        k.bewaar(
            "verzoek",
            "v1",
            "algemeen",
            "concept",
            None,
            &dossier(),
            &actor(),
            Handeling::RecordAangemaakt,
            "aangemaakt",
            t(9),
        )
        .unwrap();
        k.wachtwoord_wijzigen(&nieuw, TEST, &actor(), t(10)).unwrap();
    }

    // Het oude wachtwoord werkt niet meer, het nieuwe wel, en de gegevens zijn
    // ongewijzigd.
    assert!(Kluis::openen(&pad, &ww(), t(11)).is_err());
    let mut k = Kluis::openen(&pad, &nieuw, t(11)).unwrap();
    k.compartiment_ontgrendelen("algemeen").unwrap();
    assert_eq!(k.laad::<Dossier>("verzoek", "v1").unwrap(), dossier());
}

// --------------------------------------------------------------------------
// Schemaversie
// --------------------------------------------------------------------------

/// Een kluis van een nieuwere uitgave wordt geweigerd in plaats van half
/// begrepen geopend.
#[test]
fn een_nieuwere_kluis_wordt_geweigerd() {
    let map = TempDir::new().unwrap();
    let pad = map.path().join("test.dpofg");
    Kluis::aanmaken(&pad, &ww(), TEST, t(9)).unwrap();

    {
        let conn = rusqlite::Connection::open(&pad).unwrap();
        conn.pragma_update(None, "user_version", dpofg_store::SCHEMAVERSIE + 5).unwrap();
    }

    let fout = Kluis::openen(&pad, &ww(), t(10)).unwrap_err();
    assert!(matches!(fout, StoreFout::KluisIsNieuwer { .. }));
    assert!(fout.to_string().contains("zou gegevens kunnen beschadigen"));
}

// --------------------------------------------------------------------------
// Ontbrekende records
// --------------------------------------------------------------------------

#[test]
fn een_onbekend_record_geeft_een_duidelijke_melding() {
    let (_map, k) = nieuwe_kluis();
    let fout = k.laad::<Dossier>("verzoek", "bestaatniet").unwrap_err();
    assert!(matches!(fout, StoreFout::NietGevonden { .. }));
    assert!(fout.to_string().contains("bestaatniet"));
}

#[test]
fn sluiten_landt_in_het_logboek() {
    let (_map, mut k) = nieuwe_kluis();
    let voor = k.ketenstand().volgnummer;
    k.bewaar(
        "verzoek",
        "v1",
        "algemeen",
        "concept",
        None,
        &dossier(),
        &actor(),
        Handeling::RecordAangemaakt,
        "aangemaakt",
        t(9),
    )
    .unwrap();
    let pad = k.pad().to_path_buf();
    k.sluiten(&actor(), t(18)).unwrap();

    let k = Kluis::openen(&pad, &ww(), t(19)).unwrap();
    let regels = k.logboek().unwrap();
    assert!(regels.iter().any(|r| r.gebeurtenis.handeling == Handeling::KluisGesloten));
    assert!(k.ketenstand().volgnummer > voor + 2);
}

#[test]
fn losse_logboekregels_kunnen_worden_toegevoegd() {
    let (_map, mut k) = nieuwe_kluis();
    let n = k
        .log(
            Gebeurtenis::nieuw(
                Handeling::ControleGeblokkeerd,
                actor(),
                t(9),
                "verwerking",
                "0412-K",
                "algemeen",
                "vaststellen geweigerd: bewaartermijn ontbreekt",
            ),
            Some("controleregel REG-03".into()),
        )
        .unwrap();
    assert_eq!(k.ketenstand().volgnummer, n);

    let regels = k.logboek_van("verwerking", "0412-K").unwrap();
    assert_eq!(regels.len(), 1);
    assert_eq!(regels[0].gebeurtenis.motivering.as_deref(), Some("controleregel REG-03"));
}
