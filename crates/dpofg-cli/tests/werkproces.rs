//! Het werkproces van begin tot eind, door de bedieningsschil heen.
//!
//! Deze test draait de daadwerkelijke binary. Hij is trager dan de tests in de
//! lagen eronder, en dat is de bedoeling: hij controleert of alles samen werkt
//! en of de tegenspraak die het ontwerp belooft ook echt op het scherm komt.

use std::process::{Command, Output};
use tempfile::TempDir;

const WACHTWOORD: &str = "paard batterij niet vastzetten";

struct Proef {
    #[allow(dead_code)]
    _map: TempDir,
    kluis: std::path::PathBuf,
}

impl Proef {
    fn nieuw() -> Self {
        let map = TempDir::new().unwrap();
        let kluis = map.path().join("dossier.dpofg");
        let p = Self { _map: map, kluis };
        p.moet("kluis nieuw --licht");
        p
    }

    fn draai(&self, opdracht: &str) -> Output {
        let delen = shell_woorden(opdracht);
        Command::new(env!("CARGO_BIN_EXE_dpofg"))
            .args(&delen)
            .env("DPOFG_WACHTWOORD", WACHTWOORD)
            .env("DPOFG_KLUIS", &self.kluis)
            .env("DPOFG_GEBRUIKER", "a.devries")
            .output()
            .expect("de binary moet te starten zijn")
    }

    /// Draait een opdracht die moet slagen en levert de uitvoer.
    fn moet(&self, opdracht: &str) -> String {
        let uit = self.draai(opdracht);
        let tekst = format!(
            "{}{}",
            String::from_utf8_lossy(&uit.stdout),
            String::from_utf8_lossy(&uit.stderr)
        );
        assert!(uit.status.success(), "'{opdracht}' faalde:\n{tekst}");
        tekst
    }

    /// Draait een opdracht die moet falen en levert de uitvoer.
    fn moet_falen(&self, opdracht: &str) -> String {
        let uit = self.draai(opdracht);
        let tekst = format!(
            "{}{}",
            String::from_utf8_lossy(&uit.stdout),
            String::from_utf8_lossy(&uit.stderr)
        );
        assert!(!uit.status.success(), "'{opdracht}' had moeten falen:\n{tekst}");
        tekst
    }
}

/// Het pad naar `dpofg-verify`.
///
/// Cargo zet `CARGO_BIN_EXE_` alleen voor binaries van dezelfde crate, dus het
/// pad wordt afgeleid van de eigen binary: beide staan in dezelfde uitvoermap.
fn verify_binary() -> std::path::PathBuf {
    let eigen = std::path::PathBuf::from(env!("CARGO_BIN_EXE_dpofg"));
    let map = eigen.parent().expect("de binary staat in een map");
    let naam = if cfg!(windows) { "dpofg-verify.exe" } else { "dpofg-verify" };
    let pad = map.join(naam);
    assert!(
        pad.exists(),
        "{} bestaat niet; bouw eerst de hele werkruimte met 'cargo build --workspace'",
        pad.display()
    );
    pad
}

/// Splitst een opdrachtregel, met enkele aanhalingstekens als groepering.
fn shell_woorden(s: &str) -> Vec<String> {
    let mut uit = Vec::new();
    let mut huidig = String::new();
    let mut in_quote = false;
    for c in s.chars() {
        match c {
            '\'' => in_quote = !in_quote,
            ' ' if !in_quote => {
                if !huidig.is_empty() {
                    uit.push(std::mem::take(&mut huidig));
                }
            }
            _ => huidig.push(c),
        }
    }
    if !huidig.is_empty() {
        uit.push(huidig);
    }
    uit
}

// --------------------------------------------------------------------------
// De kluis
// --------------------------------------------------------------------------

#[test]
fn een_nieuwe_kluis_is_leeg_maar_bruikbaar() {
    let p = Proef::nieuw();
    let uit = p.moet("kluis status");
    assert!(uit.contains("algemeen"));
    assert!(uit.contains("vertrouwelijk"));
    assert!(uit.contains("De kluis is nog leeg"));
    // Zonder anker hoort daar een melding over te staan.
    assert!(uit.contains("nog geen anker"));
}

#[test]
fn een_verkeerd_wachtwoord_geeft_een_duidelijke_melding() {
    let p = Proef::nieuw();
    let uit = Command::new(env!("CARGO_BIN_EXE_dpofg"))
        .args(["kluis", "status"])
        .env("DPOFG_WACHTWOORD", "een heel ander wachtwoord")
        .env("DPOFG_KLUIS", &p.kluis)
        .output()
        .unwrap();
    assert!(!uit.status.success());
    let tekst = String::from_utf8_lossy(&uit.stderr);
    assert!(tekst.contains("onjuiste sleutel of gewijzigde gegevens"), "kreeg: {tekst}");
}

// --------------------------------------------------------------------------
// Het register: de teller en de afgeleide verplichtingen
// --------------------------------------------------------------------------

#[test]
fn een_leeg_concept_meldt_wat_er_moet_gebeuren() {
    let p = Proef::nieuw();
    let uit = p.moet("register nieuw 0412-K Verzuimregistratie --eigenaar P&O");

    assert!(uit.contains("0 van de 8"), "de teller hoort op nul te staan: {uit}");
    assert!(uit.contains("art. 30 lid 1 onder f AVG"), "de grondslag hoort erbij");
    // De melding is voortgang, geen verwijt.
    assert!(!uit.contains("verplicht veld"));
    assert!(!uit.contains("fout"));
}

#[test]
fn de_teller_groeit_mee_met_de_gegeven_antwoorden() {
    let p = Proef::nieuw();
    p.moet("register nieuw 0412-K Verzuim --eigenaar P&O");

    let voor = p.moet("register vul 0412-K --veld doeleinden --waarde 'loondoorbetaling'");
    assert!(voor.contains("van de 8"), "kreeg: {voor}");

    // Een gerechtvaardigd belang roept de belangenafweging op: één verplicht
    // onderdeel erbij, zonder dat iemand daarom hoefde te vragen.
    let na = p.moet("register vul 0412-K --veld grondslag --waarde gerechtvaardigd-belang");
    assert!(na.contains("van de 9"), "kreeg: {na}");
    assert!(na.contains("belangenafweging"));
    assert!(na.contains("art. 6 lid 1 onder f AVG"));
}

#[test]
fn bijzondere_gegevens_roepen_de_uitzonderingsgrond_op() {
    let p = Proef::nieuw();
    p.moet("register nieuw 0412-K Verzuim --eigenaar P&O");
    let uit = p.moet("register vul 0412-K --veld bijzondere-gegevens --waarde gezondheid");
    assert!(uit.contains("uitzondering_artikel9"));
    assert!(uit.contains("art. 9 lid 1 en lid 2 AVG"));
}

#[test]
fn elke_verwerker_vraagt_om_een_eigen_overeenkomst() {
    let p = Proef::nieuw();
    p.moet("register nieuw 0412-K Verzuim --eigenaar P&O");
    p.moet("register vul 0412-K --veld ontvanger --waarde arbodienst:verwerker");
    let uit = p.moet("register vul 0412-K --veld ontvanger --waarde salaris:verwerker");
    assert!(uit.contains("alle 2 verwerkers"), "de melding hoort te tellen: {uit}");
    assert!(uit.contains("nu 0 gekoppeld"));
}

#[test]
fn een_ontvanger_buiten_de_eer_vraagt_om_een_waarborg() {
    let p = Proef::nieuw();
    p.moet("register nieuw 0412-K Verzuim --eigenaar P&O");
    let uit =
        p.moet("register vul 0412-K --veld ontvanger --waarde 'analyse:verwerker,buiten-eer'");
    assert!(uit.contains("buiten de EER"));
    assert!(uit.contains("hoofdstuk V AVG"));
}

#[test]
fn vaststellen_wordt_geweigerd_zolang_er_iets_ontbreekt() {
    let p = Proef::nieuw();
    p.moet("register nieuw 0412-K Verzuim --eigenaar P&O");
    let uit = p.moet_falen("register vaststellen 0412-K");
    assert!(uit.contains("nog niet volledig"));
    // En de weigering landt in het logboek, zodat later te zien is hoeveel
    // fouten het ontwerp heeft tegengehouden.
    let logboek = p.moet("logboek toon --aantal 5");
    assert!(logboek.contains("ControleGeblokkeerd"), "kreeg: {logboek}");
}

#[test]
fn een_volledige_regel_kan_worden_vastgesteld() {
    let p = Proef::nieuw();
    p.moet("register nieuw 0412-K Verzuim --eigenaar P&O");
    for opdracht in [
        "register vul 0412-K --veld doeleinden --waarde 'loondoorbetaling bij ziekte'",
        "register vul 0412-K --veld betrokkenen --waarde medewerkers",
        "register vul 0412-K --veld gegevens --waarde 'naam; eerste ziektedag'",
        "register vul 0412-K --veld ontvanger --waarde leidinggevende",
        "register vul 0412-K --veld beveiliging --waarde 'toegang op rolbasis'",
        "register vul 0412-K --veld bewaartermijn --waarde '2 jaar vanaf einde dienstverband | art. 52 AWR'",
        "register vul 0412-K --veld grondslag --waarde wettelijke-verplichting",
        "register vul 0412-K --veld wettelijke-bepaling --waarde 'art. 7:629 BW'",
        "register vul 0412-K --veld grondslag-motivering --waarde 'de werkgever moet het loon doorbetalen bij ziekte'",
    ] {
        p.moet(opdracht);
    }
    let uit = p.moet("register vaststellen 0412-K");
    assert!(uit.contains("vastgesteld"));
}

#[test]
fn een_onbekende_grondslag_wordt_uitgelegd_in_plaats_van_afgewezen() {
    let p = Proef::nieuw();
    p.moet("register nieuw 0412-K Verzuim --eigenaar P&O");
    let uit = p.moet_falen("register vul 0412-K --veld grondslag --waarde onzin");
    assert!(uit.contains("is geen grondslag"));
    assert!(uit.contains("gerechtvaardigd-belang"), "de melding hoort de opties te noemen");
}

// --------------------------------------------------------------------------
// Het incident
// --------------------------------------------------------------------------

#[test]
fn de_klokken_hangen_aan_verschillende_ankers() {
    let p = Proef::nieuw();
    p.moet("incident nieuw 2026-0041 onbevoegde-inzage --signaal 2026-01-05T09:00:00Z");
    p.moet("incident kennisname 2026-0041 2026-01-05T09:20:00Z");
    p.moet("incident aantasting 2026-0041 --vertrouwelijkheid");
    p.moet("incident weging 2026-0041 --uitkomst hoog-risico --motivering 'gezondheidsgegevens zijn ingezien door een onbevoegde'");

    let uit = p.moet("incident toon 2026-0041");
    assert!(uit.contains("AVG-33-MELDING"));
    assert!(uit.contains("AVG-34-MEDEDELING"));
    assert!(uit.contains("AVG-33-5-REGISTER"));
    assert!(uit.contains("vaststelling van een hoog risico"));
    assert!(uit.contains("kennisname door de organisatie"));
    assert!(uit.contains("ankers vallen niet samen"));
}

#[test]
fn kennisname_voor_het_signaal_wordt_geweigerd() {
    let p = Proef::nieuw();
    p.moet("incident nieuw 2026-0041 inzage --signaal 2026-01-05T12:00:00Z");
    let uit = p.moet_falen("incident kennisname 2026-0041 2026-01-05T09:00:00Z");
    assert!(uit.contains("vóór het eerste signaal"));
}

#[test]
fn een_besluit_dat_de_weging_tegenspreekt_wordt_geblokkeerd() {
    let p = Proef::nieuw();
    p.moet("incident nieuw 2026-0041 inzage --signaal 2026-01-05T09:00:00Z");
    p.moet("incident kennisname 2026-0041 2026-01-05T09:20:00Z");
    p.moet("incident aantasting 2026-0041 --vertrouwelijkheid");
    p.moet("incident weging 2026-0041 --uitkomst hoog-risico --motivering 'gezondheidsgegevens ingezien door onbevoegde'");

    let uit = p.moet_falen(
        "incident niet-melden 2026-0041 --motivering 'wij achten melding niet opportuun' --tweede-persoon b.jansen",
    );
    assert!(uit.contains("niet te rijmen met de weging"));
    assert!(uit.contains("het besluit is niet vastgelegd"));
}

#[test]
fn niet_melden_vereist_een_tweede_laag() {
    let p = Proef::nieuw();
    p.moet("incident nieuw 2026-0041 brief --signaal 2026-01-05T09:00:00Z");
    p.moet("incident kennisname 2026-0041 2026-01-05T09:20:00Z");
    p.moet("incident aantasting 2026-0041 --vertrouwelijkheid");
    p.moet(
        "incident feiten 2026-0041 --gegevens naam --betrokkenen 1 --exfiltratie-uitgesloten true",
    );
    p.moet("incident weging 2026-0041 --uitkomst geen-risico --motivering 'brief ongeopend retour ontvangen van de postbezorger'");

    // Zonder tweede persoon en zonder afkoelperiode: geweigerd.
    let uit = p.moet_falen(
        "incident niet-melden 2026-0041 --motivering 'brief is ongeopend teruggekomen, geen kennisname door derden'",
    );
    assert!(uit.contains("afkoelperiode"));

    // Met een afkoelperiode mag het wel.
    let uit = p.moet(
        "incident niet-melden 2026-0041 --motivering 'brief is ongeopend teruggekomen, geen kennisname door derden' --afkoeluren 24",
    );
    assert!(uit.contains("besluit vastgelegd"));
    assert!(uit.contains("blijft de meldklok staan"), "kreeg: {uit}");
}

#[test]
fn een_late_registratie_blijft_zichtbaar() {
    let p = Proef::nieuw();
    let uit = p.moet("incident nieuw 2026-0041 inzage --signaal 2026-01-05T09:00:00Z");
    assert!(uit.contains("tussen het eerste signaal en deze registratie"));
    assert!(uit.contains("gladstrijken helpt niemand"));
}

#[test]
fn een_signaal_in_de_toekomst_wordt_geweigerd() {
    let p = Proef::nieuw();
    let uit = p.moet_falen("incident nieuw 2026-0041 inzage --signaal 2099-01-05T09:00:00Z");
    assert!(uit.contains("in de toekomst"));
}

// --------------------------------------------------------------------------
// De controleronde
// --------------------------------------------------------------------------

#[test]
fn de_controleronde_verdeelt_het_werk_per_rol() {
    let p = Proef::nieuw();
    p.moet("register nieuw 0412-K Verzuim --eigenaar P&O");
    p.moet("register vul 0412-K --veld ontvanger --waarde arbodienst:verwerker");
    p.moet("register vul 0412-K --veld grondslag --waarde gerechtvaardigd-belang");

    let uit = p.moet("controle");
    assert!(uit.contains("functionaris voor gegevensbescherming"));
    assert!(uit.contains("VWO-01"));
    assert!(uit.contains("GRO-01"));
    assert!(uit.contains("art. 28 lid 3 AVG"));
    assert!(uit.contains("regels gedraaid over"));
}

/// Het aantal regels in de catalogus mag geen dekking suggereren die er niet is.
#[test]
fn de_dekking_van_de_catalogus_is_opvraagbaar() {
    let p = Proef::nieuw();
    let uit = p.moet("controle --dekking");
    assert!(uit.contains("evaluatiefunctie"));
    assert!(uit.contains("Nog zonder evaluatie"));
    assert!(uit.contains("zegt niets over wat er werkelijk wordt bewaakt"));
}

// --------------------------------------------------------------------------
// Het logboek
// --------------------------------------------------------------------------

#[test]
fn het_logboek_meldt_zijn_eigen_reikwijdte() {
    let p = Proef::nieuw();
    p.moet("register nieuw 0412-K Verzuim --eigenaar P&O");

    let zonder_anker = p.moet("logboek verifieer");
    assert!(zonder_anker.contains("intern samenhangend"));
    assert!(
        zonder_anker.contains("niet vast te stellen of er aan het einde regels zijn verwijderd")
    );

    p.moet("logboek anker --bewaarplaats 'notulen directieoverleg'");

    let met_anker = p.moet("logboek verifieer");
    assert!(met_anker.contains("bevestigd tot en met regel"));
    assert!(met_anker.contains("niet uit te sluiten"), "de staart hoort benoemd te worden");
}

#[test]
fn elke_handeling_landt_in_het_logboek() {
    let p = Proef::nieuw();
    p.moet("register nieuw 0412-K Verzuim --eigenaar P&O");
    p.moet("register vul 0412-K --veld doeleinden --waarde loondoorbetaling");

    let uit = p.moet("logboek toon --aantal 10");
    assert!(uit.contains("RecordAangemaakt"));
    assert!(uit.contains("RecordGewijzigd"));
    assert!(uit.contains("a.devries"));
}

// --------------------------------------------------------------------------
// Termijnen en het kennispakket
// --------------------------------------------------------------------------

#[test]
fn een_termijn_draagt_haar_verantwoording() {
    let p = Proef::nieuw();
    let uit = p.moet("termijn AVG-33-MELDING --anker 2026-08-21T16:40:00+02:00");
    assert!(uit.contains("24-08-2026 16:40"), "72 uur later is maandag, zelfde tijdstip");
    assert!(uit.contains("zonder verlenging voor weekend of feestdag"));
    assert!(uit.contains("1182/71"));
    assert!(uit.contains("art. 33 lid 1 AVG"));
}

#[test]
fn een_maandtermijn_schuift_naar_de_eerstvolgende_werkdag() {
    let p = Proef::nieuw();
    // 15 februari 2026 is een zondag.
    let uit = p.moet("termijn AVG-12-3-VERZOEK --anker 2026-01-15T09:00:00+01:00");
    assert!(uit.contains("16-02-2026"), "kreeg: {uit}");
    assert!(uit.contains("doorgeschoven"));
}

#[test]
fn het_kennispakket_draagt_zijn_eigen_voorbehoud() {
    let p = Proef::nieuw();
    let uit = p.moet("pakket voorbehoud");
    assert!(uit.contains("niet door een jurist vastgesteld"));
    assert!(uit.contains("Verifieer"));
    assert!(uit.contains("erger dan geen product"));
}

#[test]
fn het_kennispakket_toont_zijn_consolidatiedatum() {
    let p = Proef::nieuw();
    let uit = p.moet("pakket toon");
    assert!(uit.contains("geconsolideerd op"));
    assert!(uit.contains("elke export"));
}

// --------------------------------------------------------------------------
// Het wachtwoord komt nooit uit een argument
// --------------------------------------------------------------------------

#[test]
fn er_bestaat_geen_wachtwoordvlag() {
    let uit = Command::new(env!("CARGO_BIN_EXE_dpofg")).args(["--help"]).output().unwrap();
    let tekst = String::from_utf8_lossy(&uit.stdout);
    assert!(!tekst.contains("--wachtwoord"), "een wachtwoordvlag lekt naar de proceslijst");
    assert!(tekst.contains("nooit als argument"));
}

/// De feiten bepalen welke waarborgen straks gelden, en dat wordt meteen
/// gezegd — niet pas op het moment dat het besluit wordt geweigerd.
#[test]
fn de_feiten_kondigen_de_waarborgen_aan() {
    let p = Proef::nieuw();
    p.moet("incident nieuw 2026-0041 inzage --signaal 2026-01-05T09:00:00Z");

    let uit = p.moet(
        "incident feiten 2026-0041 --gegevens 'personeelsdossiers' --betrokkenen 340 \
         --exfiltratie-uitgesloten false --bijzondere-gegevens",
    );
    assert!(uit.contains("bevestiging door een tweede persoon"));
    assert!(uit.contains("afkoelperiode volstaat dan niet"));
    assert!(uit.contains("340"), "de omvang hoort om tegenspraak te vragen");
}

/// Bij gevoelige gegevens vervalt de afkoelperiode als alternatief.
#[test]
fn bij_gevoelige_gegevens_helpt_een_afkoelperiode_niet() {
    let p = Proef::nieuw();
    p.moet("incident nieuw 2026-0041 inzage --signaal 2026-01-05T09:00:00Z");
    p.moet("incident kennisname 2026-0041 2026-01-05T09:20:00Z");
    p.moet("incident aantasting 2026-0041 --vertrouwelijkheid");
    p.moet("incident feiten 2026-0041 --gegevens dossiers --betrokkenen 5 --exfiltratie-uitgesloten true --bsn");
    p.moet("incident weging 2026-0041 --uitkomst geen-risico --motivering 'toegang was beperkt tot een lege map'");

    let uit = p.moet_falen(
        "incident niet-melden 2026-0041 --motivering 'de map bevatte geen gegevens, vastgesteld uit de logging' --afkoeluren 168",
    );
    assert!(uit.contains("burgerservicenummer"));
    assert!(uit.contains("volstaat een afkoelperiode niet"));
}

// --------------------------------------------------------------------------
// Het dossier en de controle door een derde
// --------------------------------------------------------------------------

/// De hele cirkel: samenstellen, en daarna controleren met de losse binary die
/// de kluis niet nodig heeft en geen wachtwoord vraagt.
#[test]
fn een_dossier_is_door_een_derde_te_controleren() {
    let p = Proef::nieuw();
    p.moet("register nieuw 0412-K Verzuim --eigenaar P&O");
    for opdracht in [
        "register vul 0412-K --veld doeleinden --waarde loondoorbetaling",
        "register vul 0412-K --veld betrokkenen --waarde medewerkers",
        "register vul 0412-K --veld gegevens --waarde naam",
        "register vul 0412-K --veld ontvanger --waarde leidinggevende",
        "register vul 0412-K --veld beveiliging --waarde 'toegang op rolbasis'",
        "register vul 0412-K --veld bewaartermijn --waarde '2 jaar vanaf einde dienstverband | art. 52 AWR'",
        "register vul 0412-K --veld grondslag --waarde wettelijke-verplichting",
        "register vul 0412-K --veld wettelijke-bepaling --waarde 'art. 7:629 BW'",
        "register vul 0412-K --veld grondslag-motivering --waarde 'wettelijke loondoorbetalingsplicht'",
    ] {
        p.moet(opdracht);
    }
    p.moet("register vaststellen 0412-K");
    p.moet("logboek anker --bewaarplaats notulen");

    let map = p._map.path().join("uitvraag");
    let uit = p.moet(&format!(
        "dossier {} --aanleiding 'uitvraag van 12 augustus' --bestemd-voor 'de toezichthouder'",
        map.display()
    ));
    assert!(uit.contains("Dossier samengesteld"));
    assert!(uit.contains("dpofg-verify dossier"));

    // De controle door een derde: geen kluis, geen wachtwoord.
    let controle = Command::new(verify_binary())
        .args(["dossier", map.join("manifest.json").to_str().unwrap()])
        .output()
        .unwrap();
    let tekst = String::from_utf8_lossy(&controle.stdout).to_string();

    assert!(controle.status.success(), "de controle hoort te slagen:\n{tekst}");
    assert!(tekst.contains("het manifest is niet gewijzigd na ondertekening"));
    assert!(tekst.contains("komen overeen met het manifest"));
    assert!(tekst.contains("UITKOMST: de controle is geslaagd"));
    // Het voorbehoud gaat mee en wordt niet weggelaten.
    assert!(tekst.contains("toont uit zichzelf niet aan op welk moment"));
    // En de eerlijke grens van de handtekening staat erbij.
    assert!(tekst.contains("Of die sleutel toebehoort aan wie u verwacht"));
}

/// Een gewijzigd stuk wordt op twee onafhankelijke manieren gevonden: door het
/// manifest en door de hashketen.
#[test]
fn een_gewijzigd_stuk_valt_door_de_mand() {
    let p = Proef::nieuw();
    p.moet("register nieuw 0412-K Verzuim --eigenaar P&O");
    p.moet("logboek anker --bewaarplaats notulen");

    let map = p._map.path().join("uitvraag");
    p.moet(&format!(
        "dossier {} --aanleiding uitvraag --bestemd-voor toezichthouder --met-concepten",
        map.display()
    ));

    // Iemand past de omschrijving van één logboekregel aan. Bewust een veld dat
    // het formaat niet breekt: anders zou de vervalsing al bij het inlezen
    // opvallen en zegt de test niets over de hashketen.
    let logboekpad = map.join("logboek.json");
    let inhoud = std::fs::read_to_string(&logboekpad).unwrap();
    assert!(inhoud.contains("kluis aangemaakt met schemaversie"));
    std::fs::write(
        &logboekpad,
        inhoud.replacen(
            "kluis aangemaakt met schemaversie",
            "alles in orde bevonden bij versie",
            1,
        ),
    )
    .unwrap();

    // 1. Het manifest ziet het.
    let uit = Command::new(verify_binary())
        .args(["dossier", map.join("manifest.json").to_str().unwrap()])
        .output()
        .unwrap();
    let tekst = String::from_utf8_lossy(&uit.stdout).to_string();
    assert!(!uit.status.success());
    assert!(tekst.contains("logboek.json' komt niet overeen met de hash"));

    // 2. En de hashketen ziet het ook, los van het manifest.
    let uit = Command::new(verify_binary())
        .args([
            "logboek",
            logboekpad.to_str().unwrap(),
            "--anker",
            map.join("anker.json").to_str().unwrap(),
        ])
        .output()
        .unwrap();
    let tekst = String::from_utf8_lossy(&uit.stdout).to_string();
    assert!(!uit.status.success());
    assert!(tekst.contains("de inhoud van regel"), "kreeg: {tekst}");
    assert!(tekst.contains("gewijzigd"));
}

/// Concepten blijven standaard buiten het dossier, maar hun aantal staat erin.
#[test]
fn wat_er_ontbreekt_staat_in_het_manifest() {
    let p = Proef::nieuw();
    p.moet("register nieuw 0412-K Verzuim --eigenaar P&O");

    let map = p._map.path().join("uitvraag");
    let uit = p.moet(&format!(
        "dossier {} --aanleiding uitvraag --bestemd-voor toezichthouder",
        map.display()
    ));
    assert!(uit.contains("Bewust weggelaten"));
    assert!(uit.contains("status concept"));

    let controle = Command::new(verify_binary())
        .args(["dossier", map.join("manifest.json").to_str().unwrap()])
        .output()
        .unwrap();
    let tekst = String::from_utf8_lossy(&controle.stdout).to_string();
    assert!(tekst.contains("Bewust weggelaten"), "de ontvanger hoort dit ook te zien");
}

/// Een vervalsing die het formaat zelf breekt, valt al bij het inlezen op — en
/// wordt onderscheiden van een dossier dat wél leesbaar is maar niet klopt.
#[test]
fn een_onleesbaar_bestand_is_iets_anders_dan_een_onjuist_dossier() {
    let p = Proef::nieuw();
    p.moet("register nieuw 0412-K Verzuim --eigenaar P&O");
    p.moet("logboek anker --bewaarplaats notulen");

    let map = p._map.path().join("uitvraag");
    p.moet(&format!(
        "dossier {} --aanleiding uitvraag --bestemd-voor toezichthouder --met-concepten",
        map.display()
    ));

    let logboekpad = map.join("logboek.json");
    let inhoud = std::fs::read_to_string(&logboekpad).unwrap();
    // Deze wijziging raakt de vaste woordenlijst van de handelingen.
    std::fs::write(&logboekpad, inhoud.replacen("record_aangemaakt", "record_goedgekeurd", 1))
        .unwrap();

    let uit = Command::new(verify_binary())
        .args(["logboek", logboekpad.to_str().unwrap()])
        .output()
        .unwrap();
    let fouttekst = String::from_utf8_lossy(&uit.stderr).to_string();

    // Afsluitcode 1 betekent: kon niet lezen. Afsluitcode 2 betekent: gelezen,
    // maar niet in orde. Dat onderscheid telt voor wie dit in een script draait.
    assert_eq!(uit.status.code(), Some(1), "onleesbaar hoort code 1 te geven");
    assert!(fouttekst.contains("niet leesbaar"), "kreeg: {fouttekst}");
    assert!(fouttekst.contains("unknown variant"), "de oorzaak hoort erbij: {fouttekst}");
}
