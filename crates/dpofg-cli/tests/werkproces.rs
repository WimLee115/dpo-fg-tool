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

    /// Zet een bestand naast de kluis en levert het pad.
    ///
    /// Nodig voor bewijsstukken: de tool neemt een bestand op en rekent de
    /// inhoudshash zelf uit, zodat de gebruiker er nooit een overtypt.
    fn bestand(&self, naam: &str, inhoud: &str) -> String {
        let pad = self.kluis.parent().unwrap().join(naam);
        std::fs::write(&pad, inhoud).unwrap();
        pad.display().to_string()
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

// --------------------------------------------------------------------------
// De installatiesleutel
// --------------------------------------------------------------------------

/// Een kluis is nooit zonder ondertekenidentiteit: er is geen moment waarop de
/// gebruiker eraan moet denken er een aan te maken.
#[test]
fn elke_kluis_heeft_meteen_een_installatiesleutel() {
    let p = Proef::nieuw();
    let status = p.moet("kluis status");
    let sleutel = p.moet("kluis sleutel");

    assert!(status.contains("installatiesleutel"));
    assert!(sleutel.contains("publieke sleutel"));

    let hex = sleutel_uit(&sleutel);
    assert_eq!(hex.len(), 64);
    // De afgekapte weergave in `kluis status` hoort bij dezelfde sleutel.
    assert!(status.contains(&hex[..16]), "status toont een andere sleutel:\n{status}");
}

/// De publieke sleutel is publiek. Er een wachtwoordzin voor laten intypen
/// kweekt de gewoonte om die zin overal in te typen.
#[test]
fn de_sleutel_is_te_tonen_zonder_wachtwoord() {
    let p = Proef::nieuw();
    let uit = Command::new(env!("CARGO_BIN_EXE_dpofg"))
        .args(["kluis", "sleutel"])
        .env("DPOFG_KLUIS", &p.kluis)
        .env("DPOFG_WACHTWOORD", "")
        .output()
        .unwrap();
    let tekst = String::from_utf8_lossy(&uit.stdout).to_string();

    assert!(uit.status.success(), "kreeg: {tekst}{}", String::from_utf8_lossy(&uit.stderr));
    assert_eq!(sleutel_uit(&tekst).len(), 64);
}

#[test]
fn de_sleutel_is_naar_een_bestand_te_schrijven() {
    let p = Proef::nieuw();
    let pad = p._map.path().join("sleutel.txt");
    p.moet(&format!("kluis sleutel --uitvoer {}", pad.display()));

    let inhoud = std::fs::read_to_string(&pad).unwrap();
    assert_eq!(inhoud.len(), 65, "64 tekens plus een regelovergang");
    assert!(inhoud.ends_with('\n'));
    assert_eq!(inhoud.trim(), sleutel_uit(&p.moet("kluis sleutel")));
}

/// De kern van deze bouwslag: één sleutel onder alles wat de deur uitgaat.
#[test]
fn dezelfde_installatiesleutel_ondertekent_anker_en_dossier() {
    let p = Proef::nieuw();
    p.moet("register nieuw 0412-K Verzuim --eigenaar P&O");

    let ankerpad = p._map.path().join("anker1.json");
    p.moet(&format!("logboek anker --bewaarplaats notulen --uitvoer {}", ankerpad.display()));
    let anker2pad = p._map.path().join("anker2.json");
    p.moet(&format!("logboek anker --bewaarplaats kluisje --uitvoer {}", anker2pad.display()));

    let map = p._map.path().join("uitvraag");
    p.moet(&format!(
        "dossier {} --aanleiding uitvraag --bestemd-voor toezichthouder --met-concepten",
        map.display()
    ));

    let eigen = sleutel_uit(&p.moet("kluis sleutel"));
    for (pad, veld) in [
        (ankerpad, "sleutel"),
        (anker2pad, "sleutel"),
        (map.join("manifest.json"), "ondertekenaar"),
    ] {
        let inhoud = std::fs::read_to_string(&pad).unwrap();
        let waarde: serde_json::Value = serde_json::from_str(&inhoud).unwrap();
        let gevonden = waarde
            .get(veld)
            .or_else(|| waarde.get("manifest").and_then(|_| waarde.get(veld)))
            .and_then(|v| v.as_str())
            .unwrap_or_else(|| panic!("{} heeft geen veld {veld}", pad.display()));
        assert_eq!(gevonden, eigen, "{} draagt een andere sleutel", pad.display());
    }
}

#[test]
fn de_installatiesleutel_overleeft_een_wachtwoordwissel() {
    let p = Proef::nieuw();
    let voor = sleutel_uit(&p.moet("kluis sleutel"));

    // `kluis wachtwoord` vraagt om de nieuwe zin; die komt uit de omgeving.
    let uit = Command::new(env!("CARGO_BIN_EXE_dpofg"))
        .args(["kluis", "wachtwoord"])
        .env("DPOFG_KLUIS", &p.kluis)
        .env("DPOFG_WACHTWOORD", WACHTWOORD)
        .env("DPOFG_GEBRUIKER", "a.devries")
        .output()
        .unwrap();
    assert!(
        uit.status.success(),
        "wachtwoord wijzigen faalde:\n{}{}",
        String::from_utf8_lossy(&uit.stdout),
        String::from_utf8_lossy(&uit.stderr)
    );

    assert_eq!(sleutel_uit(&p.moet("kluis sleutel")), voor);
    p.moet("register nieuw 0412-K Verzuim --eigenaar P&O");
    let ankerpad = p._map.path().join("anker.json");
    p.moet(&format!("logboek anker --uitvoer {}", ankerpad.display()));
    let waarde: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&ankerpad).unwrap()).unwrap();
    assert_eq!(waarde["sleutel"].as_str().unwrap(), voor);
}

#[test]
fn een_dossier_is_aan_de_installatie_toe_te_schrijven() {
    let p = Proef::nieuw();
    p.moet("register nieuw 0412-K Verzuim --eigenaar P&O");
    p.moet("logboek anker --bewaarplaats notulen");
    let map = p._map.path().join("uitvraag");
    p.moet(&format!(
        "dossier {} --aanleiding uitvraag --bestemd-voor toezichthouder --met-concepten",
        map.display()
    ));

    let eigen = sleutel_uit(&p.moet("kluis sleutel"));
    let uit = Command::new(verify_binary())
        .args(["dossier", map.join("manifest.json").to_str().unwrap(), "--sleutel", &eigen])
        .output()
        .unwrap();
    let tekst = String::from_utf8_lossy(&uit.stdout).to_string();

    assert_eq!(uit.status.code(), Some(0), "kreeg:\n{tekst}");
    assert!(tekst.contains("het manifest komt van die"), "kreeg:\n{tekst}");
    // De grens blijft benoemd: herkomst is geen inhoudelijk oordeel.
    assert!(tekst.contains("Dat toont niet aan dat de inhoud juist of volledig is"));
}

#[test]
fn een_dossier_van_een_andere_installatie_valt_door_de_mand() {
    let eerste = Proef::nieuw();
    let vreemde_sleutel = sleutel_uit(&eerste.moet("kluis sleutel"));

    let tweede = Proef::nieuw();
    tweede.moet("register nieuw 0412-K Verzuim --eigenaar P&O");
    tweede.moet("logboek anker --bewaarplaats notulen");
    let map = tweede._map.path().join("uitvraag");
    tweede.moet(&format!(
        "dossier {} --aanleiding uitvraag --bestemd-voor toezichthouder --met-concepten",
        map.display()
    ));

    let uit = Command::new(verify_binary())
        .args([
            "dossier",
            map.join("manifest.json").to_str().unwrap(),
            "--sleutel",
            &vreemde_sleutel,
        ])
        .output()
        .unwrap();
    let tekst = String::from_utf8_lossy(&uit.stdout).to_string();

    assert_eq!(uit.status.code(), Some(3), "een vreemde ondertekenaar hoort code 3:\n{tekst}");
    assert!(tekst.contains("ondertekend met een andere sleutel dan u hebt opgegeven"));
    // Een historisch feit hoort niet als manipulatie gelezen te worden.
    assert!(tekst.contains("wegwerpsleutel"));
    assert!(tekst.contains("komt van een andere installatie"));
}

/// De voorrangsregel: gewijzigde inhoud weegt zwaarder dan een vreemde sleutel.
#[test]
fn een_gewijzigd_stuk_weegt_zwaarder_dan_een_vreemde_sleutel() {
    let eerste = Proef::nieuw();
    let vreemde_sleutel = sleutel_uit(&eerste.moet("kluis sleutel"));

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
    std::fs::write(&logboekpad, format!("{inhoud} ")).unwrap();

    let uit = Command::new(verify_binary())
        .args([
            "dossier",
            map.join("manifest.json").to_str().unwrap(),
            "--sleutel",
            &vreemde_sleutel,
        ])
        .output()
        .unwrap();
    let tekst = String::from_utf8_lossy(&uit.stdout).to_string();

    assert_eq!(uit.status.code(), Some(2), "een gewijzigd stuk hoort code 2:\n{tekst}");
}

/// De uitgebrachte werkwijze mag niet regresseren: zonder `--sleutel` verandert
/// er niets aan de afsluitcode, en de eerlijke grens blijft staan.
#[test]
fn zonder_sleutel_meldt_de_controle_dat_er_niet_is_vastgezet() {
    let p = Proef::nieuw();
    p.moet("register nieuw 0412-K Verzuim --eigenaar P&O");
    p.moet("logboek anker --bewaarplaats notulen");
    let map = p._map.path().join("uitvraag");
    p.moet(&format!(
        "dossier {} --aanleiding uitvraag --bestemd-voor toezichthouder --met-concepten",
        map.display()
    ));

    let uit = Command::new(verify_binary())
        .args(["dossier", map.join("manifest.json").to_str().unwrap()])
        .output()
        .unwrap();
    let tekst = String::from_utf8_lossy(&uit.stdout).to_string();

    assert_eq!(uit.status.code(), Some(0));
    assert!(tekst.contains("Of die sleutel toebehoort aan wie u verwacht"));
    assert!(tekst.contains("--sleutel"));
}

#[test]
fn een_anker_is_aan_de_installatie_toe_te_schrijven() {
    let p = Proef::nieuw();
    p.moet("register nieuw 0412-K Verzuim --eigenaar P&O");
    let ankerpad = p._map.path().join("anker.json");
    p.moet(&format!("logboek anker --uitvoer {}", ankerpad.display()));
    let eigen = sleutel_uit(&p.moet("kluis sleutel"));

    let goed = Command::new(verify_binary())
        .args(["anker", ankerpad.to_str().unwrap(), "--sleutel", &eigen])
        .output()
        .unwrap();
    assert_eq!(goed.status.code(), Some(0));
    assert!(String::from_utf8_lossy(&goed.stdout).contains("komt van de installatie"));

    let vreemd = Command::new(verify_binary())
        .args(["anker", ankerpad.to_str().unwrap(), "--sleutel", &"a".repeat(64)])
        .output()
        .unwrap();
    assert_eq!(vreemd.status.code(), Some(3));
}

#[test]
fn een_onbruikbare_sleutel_wordt_geweigerd_voordat_er_iets_wordt_gecontroleerd() {
    let uit = Command::new(verify_binary())
        .args(["anker", "bestaat-niet.json", "--sleutel", "te-kort"])
        .output()
        .unwrap();
    let fouttekst = String::from_utf8_lossy(&uit.stderr).to_string();
    assert_eq!(uit.status.code(), Some(2), "clap meldt een onbruikbaar argument met code 2");
    assert!(fouttekst.contains("64 hexadecimale tekens"), "kreeg: {fouttekst}");
}

/// Een logboek draagt geen handtekening; alleen een anker doet dat. Een
/// opgegeven sleutel zonder anker mag daarom niet groen melden.
#[test]
fn een_sleutel_zonder_anker_wordt_geweigerd() {
    let p = Proef::nieuw();
    p.moet("register nieuw 0412-K Verzuim --eigenaar P&O");
    p.moet("logboek anker --bewaarplaats notulen");
    let map = p._map.path().join("uitvraag");
    p.moet(&format!(
        "dossier {} --aanleiding uitvraag --bestemd-voor toezichthouder --met-concepten",
        map.display()
    ));

    let uit = Command::new(verify_binary())
        .args([
            "logboek",
            map.join("logboek.json").to_str().unwrap(),
            "--sleutel",
            &sleutel_uit(&p.moet("kluis sleutel")),
        ])
        .output()
        .unwrap();

    assert_eq!(uit.status.code(), Some(1), "zonder anker valt er niets te vergelijken");
    assert!(String::from_utf8_lossy(&uit.stderr).contains("--anker"));
}

#[test]
fn een_logboek_met_anker_is_aan_de_installatie_toe_te_schrijven() {
    let eerste = Proef::nieuw();
    let vreemde_sleutel = sleutel_uit(&eerste.moet("kluis sleutel"));

    let p = Proef::nieuw();
    p.moet("register nieuw 0412-K Verzuim --eigenaar P&O");
    p.moet("logboek anker --bewaarplaats notulen");
    let map = p._map.path().join("uitvraag");
    p.moet(&format!(
        "dossier {} --aanleiding uitvraag --bestemd-voor toezichthouder --met-concepten",
        map.display()
    ));
    let eigen = sleutel_uit(&p.moet("kluis sleutel"));

    let draai = |sleutel: &str| {
        Command::new(verify_binary())
            .args([
                "logboek",
                map.join("logboek.json").to_str().unwrap(),
                "--anker",
                map.join("anker.json").to_str().unwrap(),
                "--sleutel",
                sleutel,
            ])
            .output()
            .unwrap()
    };

    let goed = draai(&eigen);
    assert_eq!(goed.status.code(), Some(0), "{}", String::from_utf8_lossy(&goed.stdout));
    assert!(String::from_utf8_lossy(&goed.stdout).contains("komt van de installatie"));

    let vreemd = draai(&vreemde_sleutel);
    assert_eq!(vreemd.status.code(), Some(3));
    assert!(String::from_utf8_lossy(&vreemd.stdout).contains("staat niet in de lijst"));
}

/// Een gemanipuleerd dossier van de eigen installatie mag niet worden gemeld
/// als "van een andere sleutel": dat duwt de lezer naar de onschuldige
/// verklaring terwijl het stuk juist is gewijzigd.
#[test]
fn een_gewijzigd_manifest_van_de_eigen_sleutel_meldt_geen_vreemde_ondertekenaar() {
    let p = Proef::nieuw();
    p.moet("register nieuw 0412-K Verzuim --eigenaar P&O");
    p.moet("logboek anker --bewaarplaats notulen");
    let map = p._map.path().join("uitvraag");
    p.moet(&format!(
        "dossier {} --aanleiding uitvraag --bestemd-voor toezichthouder --met-concepten",
        map.display()
    ));

    let manifestpad = map.join("manifest.json");
    let inhoud = std::fs::read_to_string(&manifestpad).unwrap();
    std::fs::write(&manifestpad, inhoud.replacen("toezichthouder", "iemand anders", 1)).unwrap();

    let uit = Command::new(verify_binary())
        .args([
            "dossier",
            manifestpad.to_str().unwrap(),
            "--sleutel",
            &sleutel_uit(&p.moet("kluis sleutel")),
        ])
        .output()
        .unwrap();
    let tekst = String::from_utf8_lossy(&uit.stdout).to_string();

    assert_eq!(uit.status.code(), Some(2), "kreeg:\n{tekst}");
    assert!(tekst.contains("de handtekening klopt niet met de inhoud"));
    assert!(
        !tekst.contains("ondertekend met een andere sleutel"),
        "de melding spreekt zichzelf tegen:\n{tekst}"
    );
}

/// Een pad met een letter met een accent is geen bijzonder geval maar de
/// Nederlandse praktijk.
#[test]
fn een_pad_met_een_accent_laat_het_logboek_heel() {
    let p = Proef::nieuw();
    let map = p._map.path().join("dossiér-uitvraag");
    p.moet("register nieuw 0412-K Verzuim --eigenaar P&O");
    p.moet(&format!(
        "dossier {} --aanleiding uitvraag --bestemd-voor toezichthouder --met-concepten",
        map.display()
    ));
    let uit = p.moet("logboek toon --aantal 25");
    assert!(uit.contains("Logboek"));
}

// --------------------------------------------------------------------------
// De belangenafweging
// --------------------------------------------------------------------------

/// Een conclusie die aan de redenering voorafgaat, is geen conclusie.
#[test]
fn een_uitkomst_zonder_redenering_wordt_geweigerd() {
    let p = Proef::nieuw();
    p.moet("register nieuw 0412-K Cameratoezicht --eigenaar 'de facilitaire dienst'");
    p.moet("lia nieuw LIA-0412 'cameratoezicht bij de ingang' --verwerking 0412-K");

    let uit = p.moet_falen("lia uitkomst LIA-0412 --uitkomst weegt-op --door 'A. de Vries'");
    assert!(uit.contains("noodzakelijkheidstoets"), "kreeg:\n{uit}");
    assert!(uit.contains("redelijke verwachtingen"));
}

#[test]
fn een_complete_redenering_draagt_een_uitkomst() {
    let p = Proef::nieuw();
    p.moet("register nieuw 0412-K Cameratoezicht --eigenaar 'de facilitaire dienst'");
    p.moet("lia nieuw LIA-0412 'cameratoezicht bij de ingang' --verwerking 0412-K");
    p.moet("lia vul LIA-0412 --veld belang --waarde 'het voorkomen van diefstal uit het magazijn'");
    p.moet("lia vul LIA-0412 --veld noodzaak --waarde 'een sluitsysteem alleen bleek onvoldoende bij herhaalde inbraak'");
    p.moet("lia vul LIA-0412 --veld afweging --waarde 'de camera richt zich op de ingang en niet op werkplekken'");
    p.moet("lia vul LIA-0412 --veld verwachtingen --waarde 'bij een bedrijfsingang mag cameratoezicht worden verwacht'");

    let uit = p.moet("lia uitkomst LIA-0412 --uitkomst weegt-op --door 'A. de Vries'");
    assert!(uit.contains("het belang weegt op"));
    assert!(uit.contains("alle verplichte onderdelen"), "kreeg:\n{uit}");
}

/// Een uitkomst die op waarborgen berust, vergt waarborgen.
#[test]
fn met_waarborgen_zonder_waarborgen_wordt_geweigerd() {
    let p = Proef::nieuw();
    p.moet("register nieuw 0412-K Cameratoezicht --eigenaar 'de facilitaire dienst'");
    p.moet("lia nieuw LIA-0412 cameratoezicht --verwerking 0412-K");
    for (veld, waarde) in [
        ("belang", "het voorkomen van diefstal"),
        ("noodzaak", "een sluitsysteem bleek onvoldoende bij herhaalde inbraak"),
        ("afweging", "de camera richt zich op de ingang en niet op werkplekken"),
        ("verwachtingen", "bij een bedrijfsingang mag cameratoezicht worden verwacht"),
    ] {
        p.moet(&format!("lia vul LIA-0412 --veld {veld} --waarde '{waarde}'"));
    }

    let uit = p.moet_falen(
        "lia uitkomst LIA-0412 --uitkomst weegt-op-met-waarborgen --door 'A. de Vries'",
    );
    assert!(uit.contains("berust zij nergens op"), "kreeg:\n{uit}");

    p.moet("lia vul LIA-0412 --veld waarborg --waarde 'beelden worden na zeven dagen automatisch gewist'");
    p.moet("lia uitkomst LIA-0412 --uitkomst weegt-op-met-waarborgen --door 'A. de Vries'");
}

// --------------------------------------------------------------------------
// Doorgiften buiten de EER
// --------------------------------------------------------------------------

fn doorgifte_opzetten(p: &Proef) {
    p.moet("register nieuw 0412-K Verzuim --eigenaar P&O");
    p.moet("doorgifte nieuw EER-0412 'hosting bij een aanbieder' --verwerking 0412-K --ontvanger 'de hostingaanbieder' --land 'Verenigde Staten'");
}

/// Modelbepalingen zonder beoordeling zijn een handtekening onder een aanname.
#[test]
fn modelbepalingen_vragen_een_doorgiftebeoordeling() {
    let p = Proef::nieuw();
    doorgifte_opzetten(&p);

    let uit = p.moet("doorgifte instrument EER-0412 --soort modelbepalingen --code SCC-2021");
    assert!(uit.contains("handtekening onder een aanname"), "kreeg:\n{uit}");
    assert!(p.moet_falen("doorgifte vaststellen EER-0412").contains("ontvangstland"));

    p.moet("doorgifte beoordeling EER-0412 --uitkomst gelijkwaardig --door 'A. de Vries' --besluit-door 'de directie' --restrisico 'toegang door overheidsdiensten blijft theoretisch mogelijk'");
    p.moet("doorgifte vaststellen EER-0412");
}

/// Geen instrument is geen keuze maar een constatering.
#[test]
fn zonder_instrument_mag_de_doorgifte_niet() {
    let p = Proef::nieuw();
    doorgifte_opzetten(&p);
    p.moet("doorgifte instrument EER-0412 --soort geen");
    let uit = p.moet_falen("doorgifte vaststellen EER-0412");
    assert!(uit.contains("mag deze doorgifte niet plaatsvinden"), "kreeg:\n{uit}");
}

/// De uitzondering van artikel 49 vergt eerst een grond uit de limitatieve
/// opsomming.
#[test]
fn artikel_49_vergt_eerst_een_grond() {
    let p = Proef::nieuw();
    doorgifte_opzetten(&p);
    let uit = p.moet_falen("doorgifte instrument EER-0412 --soort artikel49");
    assert!(uit.contains("limitatief"), "kreeg:\n{uit}");

    let grond = p.moet("doorgifte artikel49 EER-0412 --grond 'uitdrukkelijke toestemming van de betrokkene' --toepassingen 5");
    assert!(grond.contains("geen uitzondering meer"));
    p.moet("doorgifte instrument EER-0412 --soort artikel49");
}

/// Een instrument kan verlopen zonder dat er in de organisatie iets gebeurt.
#[test]
fn de_instrumentcontrole_leest_het_kennispakket() {
    let p = Proef::nieuw();
    doorgifte_opzetten(&p);
    p.moet("doorgifte instrument EER-0412 --soort modelbepalingen --code SCC-2021");

    let instrumenten = p.moet("doorgifte instrumenten");
    assert!(instrumenten.contains("SCC-2021"));
    assert!(instrumenten.contains("geldig"));

    let uit = p.moet("doorgifte controleer EER-0412");
    assert!(uit.contains("SCC-2021 is geldig"), "kreeg:\n{uit}");
}

#[test]
fn een_onbekend_instrument_wordt_gemeld_en_niet_stil_overgeslagen() {
    let p = Proef::nieuw();
    doorgifte_opzetten(&p);
    p.moet("doorgifte instrument EER-0412 --soort modelbepalingen --code BESTAAT-NIET");
    let uit = p.moet("doorgifte controleer EER-0412");
    assert!(uit.contains("staat niet in het kennispakket"), "kreeg:\n{uit}");
}

// --------------------------------------------------------------------------
// Het Wpg-spoor
// --------------------------------------------------------------------------

/// Dat het regime niet geldt, is even goed een standpunt.
#[test]
fn niet_van_toepassing_sluit_het_wpg_spoor_met_een_motivering() {
    let p = Proef::nieuw();
    p.moet("wpg nieuw WPG-2026 'handhaving openbare orde'");
    let uit = p.moet("wpg toepasselijkheid WPG-2026 --motivering 'de organisatie verwerkt geen politiegegevens; de boa-taken zijn uitbesteed'");
    assert!(uit.contains("niet van toepassing"));
    assert!(uit.contains("alle verplichte onderdelen"), "kreeg:\n{uit}");
}

/// Een rapport opbergen zonder plan is de manier waarop een audit geen gevolg
/// krijgt.
#[test]
fn een_audit_met_bevindingen_vraagt_een_verbeterplan() {
    let p = Proef::nieuw();
    p.moet("wpg nieuw WPG-2026 'handhaving openbare orde'");
    p.moet("wpg toepasselijkheid WPG-2026 --van-toepassing --motivering 'de boa-taken worden in eigen beheer uitgevoerd'");
    p.moet("wpg controle WPG-2026 2026-03-01T09:00:00Z --door 'de interne auditdienst'");

    let audit = p.moet("wpg controle WPG-2026 2026-04-01T09:00:00Z --door 'een externe auditor' --extern-uitgevoerd --bevindingen 3 --rapport AUD-2026-01");
    assert!(audit.contains("3 bevinding(en)"));
    assert!(audit.contains("geen gevolg krijgt"));
    assert!(audit.contains("verbeterplan"));

    p.moet("wpg verbeterplan WPG-2026 --door 'de directie' --maatregel 'logging aanzetten | de teamleider | 2027-01-31'");
    let toon = p.moet("wpg toon WPG-2026");
    assert!(toon.contains("alle verplichte onderdelen"), "kreeg:\n{toon}");
}

#[test]
fn een_maatregel_zonder_eigenaar_wordt_geweigerd() {
    let p = Proef::nieuw();
    p.moet("wpg nieuw WPG-2026 'handhaving openbare orde'");
    let uit = p.moet_falen(
        "wpg verbeterplan WPG-2026 --door 'de directie' --maatregel 'logging aanzetten'",
    );
    assert!(uit.contains("eigenaar"), "kreeg:\n{uit}");
}

/// De norm komt uit het kennispakket en niet uit de code.
#[test]
fn een_verlopen_audit_wordt_gemeld_met_de_norm_erbij() {
    let p = Proef::nieuw();
    p.moet("wpg nieuw WPG-2026 'handhaving openbare orde'");
    p.moet("wpg toepasselijkheid WPG-2026 --van-toepassing --motivering 'de boa-taken worden in eigen beheer uitgevoerd'");
    p.moet("wpg controle WPG-2026 2021-01-01T09:00:00Z --door 'een externe auditor' --extern-uitgevoerd");

    let uit = p.moet("wpg toon WPG-2026");
    assert!(uit.contains("de norm is 48"), "kreeg:\n{uit}");
    assert!(uit.contains("art. 33 lid 3 Wet politiegegevens"));
}

// --------------------------------------------------------------------------
// De veldmapping
// --------------------------------------------------------------------------

fn mapping_opzetten(p: &Proef) {
    p.moet("register nieuw 0412-K Verzuim --eigenaar P&O");
    p.moet("register vul 0412-K --veld gegevens --waarde naam");
    p.moet("mapping nieuw MAP-0412 'verzuimsysteem' --bron verzuimsysteem --verwerking 0412-K");
    p.moet("mapping koppel MAP-0412 --bronveld achternaam --categorie naam");
}

/// De kernuitkomst: een veld dat niemand heeft aangewezen, is een verwerking
/// die niet in het register staat.
#[test]
fn een_nieuw_veld_in_het_systeem_valt_op() {
    let p = Proef::nieuw();
    mapping_opzetten(&p);

    let velden = p._map.path().join("velden.txt");
    std::fs::write(&velden, "achternaam\nbsn\n").unwrap();

    let uit = p.moet(&format!("mapping vergelijk MAP-0412 {}", velden.display()));
    assert!(uit.contains("Staat in het systeem, niet in het register"), "kreeg:\n{uit}");
    assert!(uit.contains("bsn"));
    assert!(uit.contains("verwerking die niet is vastgelegd"));
}

/// Een register dat te veel noemt is even onbetrouwbaar als een register dat te
/// weinig noemt.
#[test]
fn een_categorie_zonder_veld_valt_ook_op() {
    let p = Proef::nieuw();
    mapping_opzetten(&p);
    p.moet("register vul 0412-K --veld gegevens --waarde gezondheid");

    let velden = p._map.path().join("velden.txt");
    std::fs::write(&velden, "achternaam\n").unwrap();

    let uit = p.moet(&format!("mapping vergelijk MAP-0412 {}", velden.display()));
    assert!(uit.contains("Staat in het register, niet in het systeem"), "kreeg:\n{uit}");
    assert!(uit.contains("gezondheid"));
}

/// Een genegeerd veld zonder reden is een weggeklikte melding.
#[test]
fn negeren_vergt_een_reden_en_neemt_de_afwijking_weg() {
    let p = Proef::nieuw();
    mapping_opzetten(&p);

    let velden = p._map.path().join("velden.txt");
    std::fs::write(&velden, "achternaam\naangemaakt_op\n").unwrap();
    let voor = p.moet(&format!("mapping vergelijk MAP-0412 {}", velden.display()));
    assert!(voor.contains("aangemaakt_op"));

    p.moet("mapping negeer MAP-0412 --bronveld aangemaakt_op --reden 'technisch veld zonder persoonsgegevens'");
    let na = p.moet(&format!("mapping vergelijk MAP-0412 {}", velden.display()));
    assert!(na.contains("geen afwijkingen"), "kreeg:\n{na}");
    assert!(na.contains("buiten beschouwing"));
}

/// Een kopregel met puntkomma's is even goed invoer als één naam per regel.
#[test]
fn een_kopregel_met_scheidingstekens_wordt_ook_gelezen() {
    let p = Proef::nieuw();
    mapping_opzetten(&p);

    let velden = p._map.path().join("kop.csv");
    std::fs::write(&velden, "achternaam;bsn;afdeling\n").unwrap();
    let uit = p.moet(&format!("mapping vergelijk MAP-0412 {}", velden.display()));
    assert!(uit.contains("3 veldnaam/namen gelezen"), "kreeg:\n{uit}");
    assert!(uit.contains("bsn"));
    assert!(uit.contains("afdeling"));
}

// --------------------------------------------------------------------------
// De redactieregie
// --------------------------------------------------------------------------

/// Zet een redactieopdracht klaar met één stuk waarin een bsn staat.
fn redactie_opzetten(p: &Proef) -> std::path::PathBuf {
    p.moet(
        "verzoek nieuw VZ-2026-040 'inzageverzoek' --soort inzage --ontvangen 2026-06-01T09:00:00Z",
    );
    let bestand = p._map.path().join("bijlage-1.txt");
    std::fs::write(&bestand, "dossier van J. Jansen, bsn 123456782, afdeling P&O").unwrap();

    p.moet("redactie nieuw RED-2026-004 inzagebundel --dossier VZ-2026-040");
    p.moet("redactie profiel RED-2026-004 --categorie bsn --waarde 123456782 --omschrijving 'het bsn van een collega'");
    p.moet(&format!("redactie stuk RED-2026-004 {}", bestand.display()));
    p.moet("redactie uitleveren RED-2026-004 --hulpmiddel 'het aangewezen redactiehulpmiddel'");
    bestand
}

/// De klassieke fout: een zwart vlak over tekst die in de tekstlaag blijft
/// staan. Precies wat deze controle moet vinden.
#[test]
fn een_zwart_vlak_over_leesbare_tekst_valt_door_de_mand() {
    let p = Proef::nieuw();
    redactie_opzetten(&p);

    // Het "geredigeerde" bestand: er is iets veranderd, maar het bsn staat er nog.
    let terug = p._map.path().join("bijlage-1-geredigeerd.txt");
    std::fs::write(&terug, "dossier van [GELAKT], bsn 123456782, afdeling P&O").unwrap();

    let uit = p.moet(&format!(
        "redactie terugnemen RED-2026-004 --stuk bijlage-1.txt {}",
        terug.display()
    ));
    assert!(uit.contains("staat nog leesbaar in het bestand"), "kreeg:\n{uit}");
    assert!(uit.contains("123456782"));

    let geweigerd = p.moet_falen("redactie verstrekken RED-2026-004 --aan 'de betrokkene'");
    assert!(geweigerd.contains("tekstlaag"));
}

/// Invariant I28: geen verstrekking zonder geslaagde terugleescontrole. Ook
/// niet wanneer de tekstcontrole slaagt maar de rest niet is gedaan.
#[test]
fn een_geslaagde_tekstcontrole_alleen_is_niet_genoeg() {
    let p = Proef::nieuw();
    redactie_opzetten(&p);

    let terug = p._map.path().join("bijlage-1-geredigeerd.txt");
    std::fs::write(&terug, "dossier van [GELAKT], bsn [GELAKT], afdeling P&O").unwrap();
    let uit = p.moet(&format!(
        "redactie terugnemen RED-2026-004 --stuk bijlage-1.txt {}",
        terug.display()
    ));
    assert!(uit.contains("geen van de 1 waarden staat nog leesbaar"), "kreeg:\n{uit}");
    // De tool is eerlijk over wat zij niet heeft gezien.
    assert!(uit.contains("samengedrukte stroom"));
    assert!(uit.contains("niet gecontroleerd"));

    let geweigerd = p.moet_falen("redactie verstrekken RED-2026-004 --aan 'de betrokkene'");
    assert!(geweigerd.contains("metagegevens"), "kreeg:\n{geweigerd}");
    assert!(geweigerd.contains("beeldvergelijking"));

    p.moet("redactie controle RED-2026-004 --stuk bijlage-1.txt --soort metagegevens --uitkomst geslaagd");
    p.moet("redactie controle RED-2026-004 --stuk bijlage-1.txt --soort beeldvergelijking --uitkomst geslaagd");
    p.moet("redactie verstrekken RED-2026-004 --aan 'de betrokkene'");
}

/// Wie zijn eigen redactie goedkeurt, controleert niets.
#[test]
fn een_handmatige_goedkeuring_vergt_een_tweede_persoon() {
    let p = Proef::nieuw();
    redactie_opzetten(&p);
    let terug = p._map.path().join("bijlage-1-geredigeerd.txt");
    std::fs::write(&terug, "dossier van [GELAKT], bsn [GELAKT]").unwrap();
    p.moet(&format!("redactie terugnemen RED-2026-004 --stuk bijlage-1.txt {}", terug.display()));

    let zonder = p.moet_falen(
        "redactie controle RED-2026-004 --stuk bijlage-1.txt --soort handmatig --uitkomst geslaagd",
    );
    assert!(zonder.contains("tweede persoon"), "kreeg:\n{zonder}");

    p.moet("redactie controle RED-2026-004 --stuk bijlage-1.txt --soort handmatig --uitkomst geslaagd --tweede-persoon 'B. Jansen'");
    p.moet("redactie verstrekken RED-2026-004 --aan 'de betrokkene'");
}

/// Een categorie waarop de tool niet kan zoeken, wordt als zodanig gemeld.
#[test]
fn beeldmateriaal_wordt_niet_als_gecontroleerd_gepresenteerd() {
    let p = Proef::nieuw();
    p.moet(
        "verzoek nieuw VZ-2026-041 'inzageverzoek' --soort inzage --ontvangen 2026-06-01T09:00:00Z",
    );
    p.moet("redactie nieuw RED-2026-005 inzagebundel --dossier VZ-2026-041");

    let uit = p.moet("redactie profiel RED-2026-005 --categorie handtekening --omschrijving 'de handtekening onder de brief'");
    assert!(uit.contains("kan de tool niet zoeken"), "kreeg:\n{uit}");
    assert!(uit.contains("tweede persoon"));
}

/// Een teruggeleverd bestand dat niets is veranderd, betekent dat er niets is
/// gebeurd.
#[test]
fn een_onveranderd_bestand_valt_op() {
    let p = Proef::nieuw();
    let bestand = redactie_opzetten(&p);
    let uit = p.moet(&format!(
        "redactie terugnemen RED-2026-004 --stuk bijlage-1.txt {}",
        bestand.display()
    ));
    assert!(uit.contains("byte voor byte gelijk"), "kreeg:\n{uit}");
}

// --------------------------------------------------------------------------
// Het Woo-spoor
// --------------------------------------------------------------------------

#[test]
fn de_weigeringsgronden_scheiden_absoluut_van_relatief() {
    let p = Proef::nieuw();
    let uit = p.moet("woo gronden");
    assert!(uit.contains("de veiligheid van de Staat"));
    assert!(uit.contains("absoluut"));
    assert!(uit.contains("relatief"));
    assert!(uit.contains("art. 5.1 lid 1"));
    assert!(uit.contains("art. 5.1 lid 2"));
    assert!(uit.contains("verwijzing naar een wetsartikel"));
}

/// Vier weken, niet een maand. Een verzoek van 1 juni verstrijkt op 29 juni.
#[test]
fn de_beslistermijn_is_vier_weken_en_geen_maand() {
    let p = Proef::nieuw();
    p.moet("woo nieuw WOO-2026-003 'correspondentie over de aanbesteding' --ontvangen 2026-06-01T09:00:00Z");
    let uit = p.moet("woo termijn WOO-2026-003");
    assert!(uit.contains("4 weken"), "kreeg:\n{uit}");
    assert!(uit.contains("29-06-2026"), "kreeg:\n{uit}");
    assert!(uit.contains("Wet open overheid"));
}

/// Een relatieve grond zonder afweging is geen besluit maar een verwijzing.
#[test]
fn een_relatieve_grond_zonder_afweging_wordt_geweigerd() {
    let p = Proef::nieuw();
    p.moet("woo nieuw WOO-2026-004 aanbesteding --ontvangen 2026-06-01T09:00:00Z");
    p.moet("woo termijn WOO-2026-004");

    let uit =
        p.moet_falen("woo grond WOO-2026-004 --grond economische-belangen --betreft 'bijlage 3'");
    assert!(uit.contains("afgewogen"), "kreeg:\n{uit}");

    p.moet("woo grond WOO-2026-004 --grond economische-belangen --betreft 'bijlage 3' --afweging 'de onderhandelingspositie zou worden geschaad bij lopende gunning'");
}

/// Bij een absolute grond valt er niets af te wegen.
#[test]
fn een_absolute_grond_vergt_geen_afweging() {
    let p = Proef::nieuw();
    p.moet("woo nieuw WOO-2026-005 aanbesteding --ontvangen 2026-06-01T09:00:00Z");
    p.moet("woo grond WOO-2026-005 --grond veiligheid-van-de-staat --betreft 'bijlage 1'");
}

#[test]
fn een_besluit_wacht_op_de_belanghebbende_derde() {
    let p = Proef::nieuw();
    p.moet("woo nieuw WOO-2026-006 aanbesteding --ontvangen 2026-06-01T09:00:00Z");
    p.moet("woo termijn WOO-2026-006");
    p.moet("woo belanghebbende WOO-2026-006 --naam 'de aannemer'");

    let uit = p.moet_falen("woo besluit WOO-2026-006 --uitkomst openbaar");
    assert!(uit.contains("art. 4.4 lid 4"), "kreeg:\n{uit}");

    let zienswijze =
        p.moet("woo zienswijze WOO-2026-006 --naam 'de aannemer' --gevraagd 2026-06-05T09:00:00Z");
    assert!(zienswijze.contains("niet dat er wordt gereageerd"));
    p.moet("woo besluit WOO-2026-006 --uitkomst openbaar --op 2026-06-20T09:00:00Z");
}

#[test]
fn een_weigering_zonder_grond_is_geen_besluit() {
    let p = Proef::nieuw();
    p.moet("woo nieuw WOO-2026-007 aanbesteding --ontvangen 2026-06-01T09:00:00Z");
    p.moet("woo termijn WOO-2026-007");
    let uit = p.moet_falen("woo besluit WOO-2026-007 --uitkomst geweigerd");
    assert!(uit.contains("weigering zonder grond"));
}

/// Randgeval T-33: één bericht met beide verzoeken levert twee dossiers met
/// twee klokken en een onderlinge verwijzing.
#[test]
fn een_bericht_met_beide_verzoeken_levert_twee_dossiers_met_twee_klokken() {
    let p = Proef::nieuw();
    p.moet("verzoek nieuw VZ-2026-030 'inzage in het eigen dossier' --soort inzage --ontvangen 2026-06-01T09:00:00Z");
    p.moet("verzoek lezing VZ-2026-030 --lezing vanaf-ontvangst --motivering 'geen twijfel over de identiteit'");
    p.moet("verzoek termijn VZ-2026-030");
    p.moet("woo nieuw WOO-2026-008 'de aanbestedingsstukken' --ontvangen 2026-06-01T09:00:00Z");
    p.moet("woo termijn WOO-2026-008");

    let koppeling = p.moet("woo koppel WOO-2026-008 --verzoek VZ-2026-030");
    assert!(koppeling.contains("gekoppeld"));
    assert!(koppeling.contains("twee klokken"));

    // Dezelfde ontvangstdatum, twee verschillende deadlines.
    let woo = p.moet("woo toon WOO-2026-008");
    let verzoek = p.moet("verzoek toon VZ-2026-030");
    assert!(woo.contains("29-06-2026"), "de Woo-termijn is vier weken:\n{woo}");
    assert!(verzoek.contains("01-07-2026"), "de AVG-termijn is een maand:\n{verzoek}");
}

#[test]
fn koppelen_aan_een_onbekend_verzoek_wordt_geweigerd() {
    let p = Proef::nieuw();
    p.moet("woo nieuw WOO-2026-009 aanbesteding --ontvangen 2026-06-01T09:00:00Z");
    let uit = p.moet_falen("woo koppel WOO-2026-009 --verzoek bestaat-niet");
    assert!(uit.contains("geen betrokkenenverzoek"));
    assert!(uit.contains("verzoek lijst"));
}

// --------------------------------------------------------------------------
// Verzoeken van betrokkenen
// --------------------------------------------------------------------------

/// Een verzoek klaarzetten tot en met de lopende termijn.
fn verzoek_opzetten(p: &Proef, kenmerk: &str, soort: &str, ontvangen: &str) {
    p.moet(&format!(
        "verzoek nieuw {kenmerk} 'verzoek van een oud-medewerker' --soort {soort} --ontvangen {ontvangen}"
    ));
    p.moet(&format!(
        "verzoek lezing {kenmerk} --lezing vanaf-ontvangst --motivering 'de ruimste lezing; geen twijfel over de identiteit'"
    ));
    p.moet(&format!("verzoek termijn {kenmerk}"));
}

/// De omstreden lezing wordt aangeboden, niet gekozen.
#[test]
fn beide_lezingen_worden_getoond_met_hun_bron() {
    let p = Proef::nieuw();
    let uit = p.moet("verzoek lezingen");
    assert!(uit.contains("vanaf ontvangst van het verzoek"));
    assert!(uit.contains("vanaf vaststelling van de identiteit"));
    assert!(uit.contains("art. 12 lid 3"));
    assert!(uit.contains("art. 12 lid 6"));
    assert!(uit.contains("De tool kiest niet voor u"));
}

#[test]
fn een_inzageverzoek_doorloopt_het_hele_werkproces() {
    let p = Proef::nieuw();
    p.moet("register nieuw 0412-K Verzuim --eigenaar P&O");
    verzoek_opzetten(&p, "VZ-2026-014", "inzage", "2026-06-01T09:00:00Z");

    let vind = p.moet("verzoek vindplaatsen VZ-2026-014 --met-concepten");
    assert!(vind.contains("0412-K"));
    assert!(vind.contains("zo volledig als het register"));

    p.moet("verzoek vindplaats VZ-2026-014 --plaats 0412-K --uitkomst verstrekt");
    let af = p.moet("verzoek afhandelen VZ-2026-014 --uitkomst voldaan --op 2026-06-20T09:00:00Z");
    assert!(af.contains("afgehandeld: voldaan"));
}

/// Wat niet is doorzocht, is niet stilzwijgend leeg.
#[test]
fn afhandelen_wordt_geweigerd_zolang_een_vindplaats_openstaat() {
    let p = Proef::nieuw();
    p.moet("register nieuw 0412-K Verzuim --eigenaar P&O");
    verzoek_opzetten(&p, "VZ-2026-014", "inzage", "2026-06-01T09:00:00Z");
    p.moet("verzoek vindplaatsen VZ-2026-014 --met-concepten");

    let uit = p.moet_falen("verzoek afhandelen VZ-2026-014 --uitkomst voldaan");
    assert!(uit.contains("niet stilzwijgend leeg"), "kreeg:\n{uit}");
}

/// Invariant I18: elke ontvanger krijgt bericht, of er staat opgeschreven
/// waarom dat niet kan.
#[test]
fn een_gehonoreerde_rectificatie_bereikt_elke_ontvanger() {
    let p = Proef::nieuw();
    p.moet("register nieuw 0412-K Verzuim --eigenaar P&O");
    verzoek_opzetten(&p, "VZ-2026-015", "rectificatie", "2026-06-01T09:00:00Z");
    p.moet("verzoek vindplaatsen VZ-2026-015 --met-concepten");
    p.moet("verzoek vindplaats VZ-2026-015 --plaats 0412-K --uitkomst gerectificeerd");
    p.moet("verzoek ontvanger VZ-2026-015 --naam 'het pensioenfonds'");
    p.moet("verzoek ontvanger VZ-2026-015 --naam 'de arbodienst'");

    let geweigerd = p.moet_falen("verzoek afhandelen VZ-2026-015 --uitkomst voldaan");
    assert!(geweigerd.contains("art. 19 AVG"), "kreeg:\n{geweigerd}");

    p.moet("verzoek kennisgeving VZ-2026-015 --naam 'het pensioenfonds' --verzonden 2026-06-10T09:00:00Z --wijze e-mail");
    // De tweede ontvanger bestaat niet meer; dan hoort de reden er te staan.
    let zonder_reden =
        p.moet_falen("verzoek kennisgeving VZ-2026-015 --naam 'de arbodienst' --onmogelijk");
    assert!(zonder_reden.contains("onevenredig"));

    p.moet("verzoek kennisgeving VZ-2026-015 --naam 'de arbodienst' --onmogelijk --motivering 'de arbodienst is opgeheven en heeft geen rechtsopvolger'");
    p.moet("verzoek afhandelen VZ-2026-015 --uitkomst voldaan --op 2026-06-20T09:00:00Z");
}

/// Een weigering zonder klachtrecht en beroepsmogelijkheid is geen bericht in
/// de zin van artikel 12 lid 4.
#[test]
fn een_weigering_vergt_het_volledige_bericht_van_lid_vier() {
    let p = Proef::nieuw();
    p.moet("register nieuw 0412-K Verzuim --eigenaar P&O");
    verzoek_opzetten(&p, "VZ-2026-016", "wissing", "2026-06-01T09:00:00Z");
    p.moet("verzoek vindplaatsen VZ-2026-016 --met-concepten");
    p.moet("verzoek vindplaats VZ-2026-016 --plaats 0412-K --uitkomst geweigerd --toelichting 'wettelijke bewaarplicht'");

    let zonder = p.moet_falen("verzoek afhandelen VZ-2026-016 --uitkomst geweigerd");
    assert!(zonder.contains("art. 12 lid 4"));

    // Half bericht: wel de redenen, niet het klachtrecht.
    let half = p.moet("verzoek bericht-lid4 VZ-2026-016 2026-06-10T09:00:00Z --redenen 'wettelijke bewaarplicht op grond van art. 52 AWR' --rechtsmiddel");
    assert!(half.contains("klachtrecht"));
    assert!(p
        .moet_falen("verzoek afhandelen VZ-2026-016 --uitkomst geweigerd")
        .contains("klachtrecht"));

    p.moet("verzoek bericht-lid4 VZ-2026-016 2026-06-10T09:00:00Z --redenen 'wettelijke bewaarplicht op grond van art. 52 AWR' --klachtrecht --rechtsmiddel");
    p.moet("verzoek afhandelen VZ-2026-016 --uitkomst geweigerd --op 2026-06-20T09:00:00Z");
}

/// Anonimiseren is een sterke bewering. Klopt zij niet, dan is er niets gewist.
#[test]
fn geanonimiseerd_valt_zonder_toets_terug_op_gepseudonimiseerd() {
    let p = Proef::nieuw();
    p.moet("register nieuw 0412-K Verzuim --eigenaar P&O");
    verzoek_opzetten(&p, "VZ-2026-017", "wissing", "2026-06-01T09:00:00Z");
    p.moet("verzoek vindplaatsen VZ-2026-017 --met-concepten");

    let uit = p.moet("verzoek vindplaats VZ-2026-017 --plaats 0412-K --uitkomst geanonimiseerd");
    assert!(uit.contains("nog persoonsgegevens"), "kreeg:\n{uit}");
    assert!(uit.contains("er niets gewist"));
}

/// Randgeval T-12: het verlengingsbericht moest binnen de eerste maand.
#[test]
fn een_verlenging_na_de_eerste_maand_wordt_geweigerd() {
    let p = Proef::nieuw();
    verzoek_opzetten(&p, "VZ-2026-018", "inzage", "2026-05-01T09:00:00Z");

    let te_laat = p.moet_falen(
        "verzoek verlengen VZ-2026-018 2026-06-15T09:00:00Z --grond complexiteit --motivering 'het verzoek raakt zeven systemen'",
    );
    assert!(te_laat.contains("binnen de oorspronkelijke termijn"), "kreeg:\n{te_laat}");

    let op_tijd = p.moet(
        "verzoek verlengen VZ-2026-018 2026-05-20T09:00:00Z --grond complexiteit --motivering 'het verzoek raakt zeven systemen'",
    );
    assert!(op_tijd.contains("de complexiteit van het verzoek"));
}

/// Randgevallen T-21 en T-22, achter elkaar. Een verzoek van 31 januari kan
/// niet op 31 februari verstrijken, dus klemt de maand op 28 februari 2026 —
/// en dat is een zaterdag, dus schuift de termijn door naar maandag 2 maart.
/// Twee regels op één datum: eerst de maandeindeklem, dan de verlenging naar de
/// eerstvolgende werkdag.
#[test]
fn de_maandtermijn_klemt_en_schuift_daarna_naar_een_werkdag() {
    let p = Proef::nieuw();
    verzoek_opzetten(&p, "VZ-2026-019", "inzage", "2026-01-31T09:00:00Z");
    let uit = p.moet("verzoek toon VZ-2026-019");
    assert!(uit.contains("02-03-2026"), "kreeg:\n{uit}");
}

/// De tweede lezing rekent vanaf een moment dat er dan wel moet zijn.
#[test]
fn de_tweede_lezing_vergt_een_vastgestelde_identiteit() {
    let p = Proef::nieuw();
    p.moet(
        "verzoek nieuw VZ-2026-020 'inzageverzoek' --soort inzage --ontvangen 2026-06-01T09:00:00Z",
    );

    let zonder = p.moet_falen(
        "verzoek lezing VZ-2026-020 --lezing vanaf-identiteit --motivering 'gerede twijfel over de identiteit'",
    );
    assert!(zonder.contains("leg dat moment eerst vast"));

    p.moet("verzoek identiteit VZ-2026-020 2026-06-05T09:00:00Z");
    p.moet("verzoek lezing VZ-2026-020 --lezing vanaf-identiteit --motivering 'gerede twijfel over de identiteit'");
    p.moet("verzoek termijn VZ-2026-020");

    let uit = p.moet("verzoek toon VZ-2026-020");
    // Verankerd op 5 juni, dus een maand later op 5 juli — een zondag, waardoor
    // de termijn doorschuift naar maandag 6 juli. Ankert de klok ten onrechte
    // op de ontvangstdatum van 1 juni, dan staat hier 1 juli.
    assert!(uit.contains("06-07-2026"), "de klok hoort op 5 juni te ankeren:\n{uit}");
}

// --------------------------------------------------------------------------
// De effectbeoordeling
// --------------------------------------------------------------------------

/// Een dossier bouwen dat klaar is om te worden vastgesteld.
fn dpia_opzetten(p: &Proef, kenmerk: &str) {
    p.moet("register nieuw 0412-K Verzuim --eigenaar P&O");
    p.moet(&format!("dpia nieuw {kenmerk} Verzuimregistratie --verwerking 0412-K"));
    p.moet(&format!(
        "dpia voortoets {kenmerk} --uitkomst vereist --motivering 'twee criteria worden geraakt: bijzondere gegevens en grootschaligheid'"
    ));
    p.moet(&format!("dpia uitvoeren {kenmerk} 2026-06-01T09:00:00Z --door 'A. de Vries' --methode WP248 --vooraf true"));
    p.moet(&format!("dpia vul {kenmerk} --veld systematische-beschrijving --waarde 'verzuimregistratie voor loondoorbetaling'"));
    p.moet(&format!("dpia vul {kenmerk} --veld noodzaak-en-evenredigheid --waarde 'geen minder ingrijpend alternatief beschikbaar'"));
    p.moet(&format!("dpia vul {kenmerk} --veld risico --waarde 'onbevoegde inzage door collega'"));
    p.moet(&format!("dpia vul {kenmerk} --veld maatregel --waarde 'toegang op rolbasis'"));
    p.moet(&format!(
        "dpia vul {kenmerk} --veld advies-fg --waarde 'de beoordeling is navolgbaar en volledig'"
    ));
}

#[test]
fn een_effectbeoordeling_koppelt_zich_aan_een_registerregel() {
    let p = Proef::nieuw();
    p.moet("register nieuw 0412-K Verzuim --eigenaar P&O");
    let uit = p.moet("dpia nieuw DPIA-0412 Verzuimregistratie --verwerking 0412-K");

    assert!(uit.contains("aangemaakt bij 0412-K"));
    // Het dossier vraagt eerst om de voortoets en niet meteen om alles.
    assert!(uit.contains("voortoets"));
    assert!(uit.contains("0 van de 2"), "kreeg:\n{uit}");

    // En het register meldt de ontbrekende verwijzing niet meer.
    let register = p.moet("register toon 0412-K");
    assert!(!register.contains("dpia —"), "de koppeling hoort nu te staan:\n{register}");
}

#[test]
fn koppelen_aan_een_onbekende_registerregel_wordt_geweigerd_bij_de_dpia() {
    let p = Proef::nieuw();
    let uit = p.moet_falen("dpia nieuw DPIA-0412 Verzuim --verwerking bestaat-niet");
    assert!(uit.contains("geen registerregel met kenmerk"));
    assert!(uit.contains("register lijst"));
}

/// Een gemotiveerd besluit dat er geen beoordeling nodig is, sluit het dossier.
/// Dat is geen ontsnapping: het besluit staat vast en gaat mee in elke export.
#[test]
fn de_voortoets_niet_nodig_sluit_het_dossier_met_twee_onderdelen() {
    let p = Proef::nieuw();
    p.moet("register nieuw 0412-K Verzuim --eigenaar P&O");
    p.moet("dpia nieuw DPIA-0412 Verzuimregistratie --verwerking 0412-K");

    let uit = p.moet(
        "dpia voortoets DPIA-0412 --uitkomst niet-nodig --motivering 'geen van de negen criteria wordt geraakt; kleinschalige verwerking zonder bijzondere gegevens'",
    );
    assert!(uit.contains("2 van de 2"), "kreeg:\n{uit}");
    assert!(uit.contains("alle verplichte onderdelen zijn ingevuld"));

    p.moet("dpia vaststellen DPIA-0412");
}

/// Een restrisico is per definitie wat er ná de maatregelen overblijft.
#[test]
fn een_restrisico_zonder_maatregelen_wordt_geweigerd() {
    let p = Proef::nieuw();
    p.moet("register nieuw 0412-K Verzuim --eigenaar P&O");
    p.moet("dpia nieuw DPIA-0412 Verzuimregistratie --verwerking 0412-K");
    p.moet(
        "dpia voortoets DPIA-0412 --uitkomst vereist --motivering 'twee criteria worden geraakt'",
    );
    p.moet("dpia uitvoeren DPIA-0412 2026-06-01T09:00:00Z --door 'A. de Vries'");
    p.moet("dpia vul DPIA-0412 --veld risico --waarde 'onbevoegde inzage'");

    let uit = p.moet_falen("dpia restrisico DPIA-0412 --niveau laag --motivering 'beperkte kring'");
    assert!(uit.contains("overblijft ná de maatregelen"), "kreeg:\n{uit}");
    assert!(uit.contains("art. 35 lid 7 onder d"));
}

/// De hele klok: indienen, opschorten, hervatten, verlengen, advies.
#[test]
fn de_raadplegingsklok_staat_stil_tijdens_een_informatieverzoek() {
    let p = Proef::nieuw();
    dpia_opzetten(&p, "DPIA-0412");
    p.moet("dpia restrisico DPIA-0412 --niveau hoog --motivering 'het risico blijft groot ondanks de maatregelen'");

    let ingediend = p.moet("dpia raadpleging DPIA-0412 2026-06-05T09:00:00Z");
    assert!(ingediend.contains("8 weken"));
    assert!(ingediend.contains("art. 36 lid 2 AVG"), "de grondslag hoort te kloppen:\n{ingediend}");
    assert!(ingediend.contains("geen goedkeuring"));

    let voor = p.moet("dpia toon DPIA-0412");
    let deadline_voor = regel_met(&voor, "verstrijkt");

    p.moet("dpia raadpleging-opschorten DPIA-0412 2026-06-12T09:00:00Z --opgevraagd 'de onderliggende risicoanalyse'");
    p.moet("dpia raadpleging-hervatten DPIA-0412 2026-06-26T09:00:00Z");

    let na = p.moet("dpia toon DPIA-0412");
    let deadline_na = regel_met(&na, "verstrijkt");
    assert_ne!(
        deadline_voor, deadline_na,
        "veertien dagen opschorting hoort de deadline te verschuiven"
    );
    assert!(na.contains("opgeschort van"));
    assert!(na.contains("informatieverzoek van de toezichthouder"));
}

#[test]
fn de_toezichthouder_kan_de_termijn_verlengen() {
    let p = Proef::nieuw();
    dpia_opzetten(&p, "DPIA-0412");
    p.moet("dpia restrisico DPIA-0412 --niveau hoog --motivering 'het risico blijft groot ondanks de maatregelen'");
    p.moet("dpia raadpleging DPIA-0412 2026-06-05T09:00:00Z");

    let uit = p.moet("dpia raadpleging-verlengen DPIA-0412 2026-06-20T09:00:00Z");
    assert!(uit.contains("verlenging vastgelegd"));
    assert!(uit.contains("1 keer"));

    // Een tweede verlenging kent de verordening niet.
    let tweede = p.moet_falen("dpia raadpleging-verlengen DPIA-0412 2026-06-25T09:00:00Z");
    assert!(tweede.contains("maximum"));
}

#[test]
fn het_advies_van_de_toezichthouder_rondt_de_klok_af() {
    let p = Proef::nieuw();
    dpia_opzetten(&p, "DPIA-0412");
    p.moet("dpia restrisico DPIA-0412 --niveau hoog --motivering 'het risico blijft groot ondanks de maatregelen'");
    p.moet("dpia raadpleging DPIA-0412 2026-06-05T09:00:00Z");
    p.moet("dpia advies DPIA-0412 2026-07-15T09:00:00Z --referentie AP-2026-1234");

    let uit = p.moet("dpia toon DPIA-0412");
    assert!(uit.contains("afgerond met advies"));
    assert!(uit.contains("AP-2026-1234"));

    p.moet("dpia vaststellen DPIA-0412");
}

/// Wijzigt het risicoprofiel van de verwerking, dan is de beoordeling niet meer
/// vanzelfsprekend actueel (art. 35 lid 11 AVG).
#[test]
fn een_gewijzigde_verwerking_vraagt_om_herbeoordeling() {
    let p = Proef::nieuw();
    dpia_opzetten(&p, "DPIA-0412");
    p.moet(
        "dpia restrisico DPIA-0412 --niveau laag --motivering 'beperkte kring van geadresseerden'",
    );
    p.moet("dpia vaststellen DPIA-0412");

    let uit = p.moet("register vul 0412-K --veld bijzondere-gegevens --waarde gezondheid");
    assert!(uit.contains("DPIA-0412"), "de gekoppelde beoordeling hoort genoemd te worden:\n{uit}");
    assert!(uit.contains("herziening nodig"));
    assert!(uit.contains("art. 35 lid 11 AVG"));

    let dossier = p.moet("dpia toon DPIA-0412");
    assert!(dossier.contains("herziening nodig"));
}

#[test]
fn de_effectbeoordeling_komt_mee_in_een_dossier() {
    let p = Proef::nieuw();
    dpia_opzetten(&p, "DPIA-0412");
    p.moet(
        "dpia restrisico DPIA-0412 --niveau laag --motivering 'beperkte kring van geadresseerden'",
    );
    p.moet("dpia vaststellen DPIA-0412");
    p.moet("logboek anker --bewaarplaats notulen");

    let map = p._map.path().join("uitvraag");
    p.moet(&format!(
        "dossier {} --aanleiding uitvraag --bestemd-voor toezichthouder --met-concepten",
        map.display()
    ));
    assert!(map.join("dpia-DPIA-0412.json").exists(), "de beoordeling hoort in de bundel");

    // En de weglatingstekst leest als Nederlands.
    let manifest = std::fs::read_to_string(map.join("manifest.json")).unwrap();
    assert!(!manifest.contains("dpiaen"), "geen verzonnen meervoud in een dossier");
}

/// Een afgesloten termijn schuift niet meer. Zou dat wel kunnen, dan zou een
/// dossier dat al is uitgeleverd achteraf een andere einddatum krijgen.
#[test]
fn een_afgeronde_raadpleging_is_niet_meer_te_verschuiven() {
    let p = Proef::nieuw();
    dpia_opzetten(&p, "DPIA-0412");
    p.moet("dpia restrisico DPIA-0412 --niveau hoog --motivering 'het risico blijft groot ondanks de maatregelen'");
    p.moet("dpia raadpleging DPIA-0412 2026-06-05T09:00:00Z");
    p.moet("dpia advies DPIA-0412 2026-07-15T09:00:00Z --referentie AP-2026-1234");

    for opdracht in [
        "dpia raadpleging-opschorten DPIA-0412 2026-07-20T09:00:00Z",
        "dpia raadpleging-hervatten DPIA-0412 2026-07-25T09:00:00Z",
        "dpia raadpleging-verlengen DPIA-0412 2026-07-20T09:00:00Z",
    ] {
        let uit = p.moet_falen(opdracht);
        assert!(uit.contains("afgesloten termijn schuift niet meer"), "'{opdracht}' gaf:\n{uit}");
    }
}

/// Twee beoordelingen op één registerregel zou betekenen dat een
/// risicowijziging stilzwijgend aan één van beide voorbijgaat.
#[test]
fn een_registerregel_draagt_maar_een_effectbeoordeling() {
    let p = Proef::nieuw();
    p.moet("register nieuw 0412-K Verzuim --eigenaar P&O");
    p.moet("dpia nieuw DPIA-0412 Verzuimregistratie --verwerking 0412-K");

    let uit = p.moet_falen("dpia nieuw DPIA-0413 Nogeens --verwerking 0412-K");
    assert!(uit.contains("al gekoppeld aan effectbeoordeling DPIA-0412"), "kreeg:\n{uit}");
}

/// Een tijdstip in de toekomst is een invoerfout, en bij het advies zet hij
/// bovendien de bewaking van de termijn uit.
#[test]
fn tijdstippen_in_de_toekomst_worden_overal_geweigerd() {
    let p = Proef::nieuw();
    dpia_opzetten(&p, "DPIA-0412");
    p.moet("dpia restrisico DPIA-0412 --niveau hoog --motivering 'het risico blijft groot ondanks de maatregelen'");
    p.moet("dpia raadpleging DPIA-0412 2026-06-05T09:00:00Z");

    for (opdracht, woord) in [
        ("dpia raadpleging-hervatten DPIA-0412 2099-01-01T09:00:00Z", "toekomst"),
        ("dpia raadpleging-verlengen DPIA-0412 2099-01-01T09:00:00Z", "toekomst"),
        ("dpia advies DPIA-0412 2099-01-01T09:00:00Z --referentie AP-1", "toekomst"),
    ] {
        let uit = p.moet_falen(opdracht);
        assert!(uit.contains(woord), "'{opdracht}' gaf:\n{uit}");
    }
}

/// Haalt de eerste regel met een bepaald woord uit de uitvoer.
fn regel_met(uitvoer: &str, woord: &str) -> String {
    uitvoer
        .lines()
        .find(|r| r.contains(woord))
        .unwrap_or_else(|| panic!("geen regel met '{woord}' in:\n{uitvoer}"))
        .to_string()
}

// --------------------------------------------------------------------------
// De regelcatalogus: bevindingen moeten wegneembaar zijn
// --------------------------------------------------------------------------

/// De kern van een controleregel: hij moet weg te nemen zijn. Een bevinding
/// die blijft staan wat de gebruiker ook doet, leert hem meldingen wegklikken.
#[test]
fn een_incident_is_aan_een_registerregel_te_koppelen() {
    let p = Proef::nieuw();
    p.moet("register nieuw 0412-K Verzuim --eigenaar P&O");
    p.moet("incident nieuw 2026-0041 'onbevoegde inzage'");

    let uit = p.moet("incident koppel 2026-0041 --verwerking 0412-K");
    assert!(uit.contains("gekoppeld aan 0412-K"));

    // Nog eens koppelen is geen fout, maar verandert niets.
    let nogmaals = p.moet("incident koppel 2026-0041 --verwerking 0412-K");
    assert!(nogmaals.contains("was al gekoppeld"));

    let ontkoppeld = p.moet("incident ontkoppel 2026-0041 --verwerking 0412-K");
    assert!(ontkoppeld.contains("ontkoppeld van 0412-K"));
    assert!(ontkoppeld.contains("LEK-15"), "de regel die weer gaat spreken, hoort erbij");
}

#[test]
fn koppelen_aan_een_onbekende_registerregel_wordt_geweigerd() {
    let p = Proef::nieuw();
    p.moet("incident nieuw 2026-0041 'onbevoegde inzage'");
    let uit = p.moet_falen("incident koppel 2026-0041 --verwerking bestaat-niet");
    assert!(uit.contains("geen registerregel met kenmerk"));
    assert!(uit.contains("register lijst"), "de opdracht die verder helpt, hoort erbij");
}

#[test]
fn een_incident_is_af_te_ronden_met_oorzaak_en_maatregel() {
    let p = Proef::nieuw();
    p.moet("incident nieuw 2026-0041 'onbevoegde inzage'");

    let uit = p.moet(
        "incident afronden 2026-0041 --oorzaak menselijke-fout --maatregel 'vier-ogen bij verzending'",
    );
    assert!(uit.contains("menselijke-fout"));
    assert!(uit.contains("vier-ogen bij verzending"));
    assert!(uit.contains("LEK-13"), "waarvoor de oorzaakcategorie dient, hoort erbij");

    // Zonder maatregel: geen fout, wel de blokkerende regel benoemd.
    let p2 = Proef::nieuw();
    p2.moet("incident nieuw 2026-0042 'verkeerd geadresseerde brief'");
    let zonder = p2.moet("incident afronden 2026-0042 --oorzaak menselijke-fout");
    assert!(zonder.contains("geen maatregel"));
    assert!(zonder.contains("LEK-12"));
}

/// Een afhandelmoment in de toekomst is een invoerfout, geen planning.
#[test]
fn afronden_in_de_toekomst_wordt_geweigerd() {
    let p = Proef::nieuw();
    p.moet("incident nieuw 2026-0041 'onbevoegde inzage'");
    let uit = p.moet_falen(
        "incident afronden 2026-0041 --oorzaak menselijke-fout --afgehandeld 2099-01-01T00:00:00Z",
    );
    assert!(uit.contains("in de toekomst"));
}

/// 'Ja' werd stilzwijgend als nee gelezen. Daardoor legde de gebruiker een
/// burgerservicenummer aan dat niet werd vastgelegd, en zweeg de regel die
/// daarop bewaakt.
#[test]
fn een_ja_nee_veld_leest_meer_dan_een_schrijfwijze() {
    let p = Proef::nieuw();
    p.moet("register nieuw 0412-K Verzuim --eigenaar P&O");

    for schrijfwijze in ["ja", "Ja", "JA", "j", "true"] {
        p.moet(&format!("register vul 0412-K --veld bsn --waarde {schrijfwijze}"));
    }
    for schrijfwijze in ["nee", "Nee", "n", "false"] {
        p.moet(&format!("register vul 0412-K --veld bsn --waarde {schrijfwijze}"));
    }

    let uit = p.moet_falen("register vul 0412-K --veld bsn --waarde misschien");
    assert!(uit.contains("is geen antwoord"));
    assert!(uit.contains("ja of nee"));
}

/// Een leeg veld dat als ingevuld wordt vastgelegd, haalt de bevinding weg
/// zonder dat er iets is opgelost.
#[test]
fn een_leeg_verplicht_veld_wordt_geweigerd() {
    let p = Proef::nieuw();
    p.moet("register nieuw 0412-K Verzuim --eigenaar P&O");
    let uit = p.moet_falen("register vul 0412-K --veld wettelijke-bepaling --waarde '   '");
    assert!(uit.contains("mag niet leeg zijn"));
}

/// De dekkingsopdracht is de enige eerlijke bron over wat er wordt bewaakt.
#[test]
fn de_dekking_meldt_wat_er_werkelijk_draait() {
    let p = Proef::nieuw();
    let uit = p.moet("controle --dekking");
    assert!(uit.contains("van de 68 regels"));
    assert!(uit.contains("54 van de 68"), "kreeg:\n{uit}");
    // En wat er níet draait, staat er met naam bij.
    assert!(uit.contains("Nog zonder evaluatie"));
    assert!(uit.contains("SYS-09"), "kreeg:\n{uit}");
}

/// Haalt de 64 hexadecimale tekens uit de uitvoer van `kluis sleutel`.
fn sleutel_uit(uitvoer: &str) -> String {
    uitvoer
        .split(|c: char| !c.is_ascii_hexdigit())
        .find(|w| w.len() == 64 && w.chars().all(|c| c.is_ascii_hexdigit()))
        .unwrap_or_else(|| panic!("geen sleutel gevonden in:\n{uitvoer}"))
        .to_string()
}

// --------------------------------------------------------------------------
// Leveranciers en verwerkersovereenkomsten
// --------------------------------------------------------------------------

fn leverancier_opzetten(p: &Proef) {
    p.moet("leverancier nieuw LEV-014 'Zorgdossier BV' --kritikaliteit hoog");
    p.moet(
        "leverancier overeenkomst LEV-014 --contract VWO-2026-014 \
         --ondertekend 2026-01-15T09:00:00Z --meldtermijn-uren 24",
    );
}

fn alle_vindplaatsen(p: &Proef) {
    for eis in [
        "instructies",
        "geheimhouding",
        "beveiliging",
        "subverwerkers",
        "bijstand-verzoeken",
        "bijstand-verplichtingen",
        "wissen-of-teruggeven",
        "audits-en-informatie",
    ] {
        p.moet(&format!("leverancier vindplaats LEV-014 --eis {eis} --aanduiding 'artikel 7'"));
    }
}

/// De acht onderdelen van artikel 28 lid 3 staan in het product, niet in een
/// bijlage die de gebruiker zelf moet opzoeken.
#[test]
fn de_acht_onderdelen_staan_met_letter_en_grondslag_in_beeld() {
    let p = Proef::nieuw();
    let uit = p.moet("leverancier eisen");
    for letter in ["a", "b", "c", "d", "e", "f", "g", "h"] {
        assert!(uit.contains(&format!("art. 28 lid 3 onder {letter} AVG")), "kreeg:\n{uit}");
    }
}

/// Een vinkje zegt alleen dat iemand ooit dacht dat het geregeld was. Bij een
/// uitvraag moet worden aangewezen wáár het staat.
#[test]
fn een_onderdeel_afvinken_zonder_vindplaats_kan_niet() {
    let p = Proef::nieuw();
    leverancier_opzetten(&p);

    let uit = p.moet_falen("leverancier vindplaats LEV-014 --eis beveiliging --aanduiding '   '");
    assert!(uit.contains("een vinkje zegt alleen"), "kreeg:\n{uit}");

    let uit = p.moet_falen("leverancier vaststellen LEV-014");
    assert!(uit.contains("wijs aan waar in het contract staat"), "kreeg:\n{uit}");
}

#[test]
fn met_acht_vindplaatsen_is_de_leverancier_vast_te_stellen() {
    let p = Proef::nieuw();
    leverancier_opzetten(&p);
    alle_vindplaatsen(&p);
    p.moet("leverancier subverwerkerscontrole LEV-014");

    let uit = p.moet("leverancier vaststellen LEV-014");
    assert!(uit.contains("vastgesteld"), "kreeg:\n{uit}");
}

/// Zonder afgesproken meldtermijn is er niets waaraan een melding valt te
/// toetsen. Dat blokkeert, en niet met een signaaltje.
#[test]
fn zonder_meldtermijn_blokkeert_het_vaststellen() {
    let p = Proef::nieuw();
    p.moet("leverancier nieuw LEV-015 'Rekencentrum BV' --kritikaliteit gemiddeld");
    p.moet(
        "leverancier overeenkomst LEV-015 --contract VWO-2026-015 \
         --ondertekend 2026-01-15T09:00:00Z",
    );
    for eis in [
        "instructies",
        "geheimhouding",
        "beveiliging",
        "subverwerkers",
        "bijstand-verzoeken",
        "bijstand-verplichtingen",
        "wissen-of-teruggeven",
        "audits-en-informatie",
    ] {
        p.moet(&format!("leverancier vindplaats LEV-015 --eis {eis} --aanduiding 'artikel 7'"));
    }
    let uit = p.moet_falen("leverancier vaststellen LEV-015");
    assert!(uit.contains("binnen hoeveel uur"), "kreeg:\n{uit}");
}

/// Een contract dat na de aanvang is getekend, dekt de periode ervóór niet.
/// Dat is een feit dat blijft staan; de tool maakt het zichtbaar en telt de
/// ongedekte dagen.
#[test]
fn tekenen_na_aanvang_wordt_gemeld_met_het_aantal_ongedekte_dagen() {
    let p = Proef::nieuw();
    p.moet("leverancier nieuw LEV-016 'Laatkomer BV'");
    let uit = p.moet(
        "leverancier overeenkomst LEV-016 --contract VWO-2026-016 \
         --ondertekend 2026-03-01T09:00:00Z --verwerking-begon 2026-01-01T09:00:00Z \
         --meldtermijn-uren 24",
    );
    assert!(uit.contains("nadat de verwerking al liep"), "kreeg:\n{uit}");

    // Rapporterend, dus buiten de standaarddrempel: het is een feit om te
    // melden, geen gebrek dat vandaag nog te herstellen valt.
    assert!(!p.moet("controle").contains("VWO-13"));
    let uit = p.moet("controle --vanaf rapporterend");
    assert!(uit.contains("VWO-13"), "kreeg:\n{uit}");
    assert!(uit.contains("59 dagen"), "de ongedekte periode hoort geteld te worden:\n{uit}");
}

/// Toevoegen is niet nalopen. Wie dat door elkaar haalt, denkt jaren op orde
/// te zijn omdat er ooit iemand een naam heeft ingetypt.
#[test]
fn een_subverwerker_toevoegen_geldt_niet_als_controle() {
    let p = Proef::nieuw();
    leverancier_opzetten(&p);
    let uit = p.moet(
        "leverancier subverwerker LEV-014 --naam 'het rekencentrum' --land Duitsland \
         --dienst opslag",
    );
    assert!(uit.contains("Toevoegen is geen controle"), "kreeg:\n{uit}");

    let uit = p.moet("controle");
    assert!(uit.contains("VWO-09"), "kreeg:\n{uit}");
    assert!(uit.contains("nooit nagelopen"), "kreeg:\n{uit}");

    p.moet("leverancier subverwerkerscontrole LEV-014");
    assert!(!p.moet("controle").contains("VWO-09"));
}

/// De drempel voor de meldtermijn komt uit het kennispakket en staat niet in
/// de programmacode.
#[test]
fn een_te_lange_meldtermijn_wordt_gesignaleerd() {
    let p = Proef::nieuw();
    p.moet("leverancier nieuw LEV-017 'Trage Verwerker BV'");
    let uit = p.moet(
        "leverancier overeenkomst LEV-017 --contract VWO-2026-017 \
         --ondertekend 2026-01-15T09:00:00Z --meldtermijn-uren 96",
    );
    assert!(uit.contains("tweeënzeventig uur"), "kreeg:\n{uit}");

    let uit = p.moet("controle");
    assert!(uit.contains("VWO-04"), "kreeg:\n{uit}");
}

/// LEK-16 rekent na wat er feitelijk gebeurde tegen wat er is afgesproken. De
/// regel is alleen bereikbaar als de bedieningsschil beide tijdstippen kan
/// vastleggen; anders is het een regel die nooit aanslaat.
#[test]
fn een_verwerker_die_te_laat_meldt_is_na_te_rekenen() {
    let p = Proef::nieuw();
    leverancier_opzetten(&p);
    alle_vindplaatsen(&p);
    p.moet("leverancier subverwerkerscontrole LEV-014");
    p.moet("incident nieuw 2026-0041 'melding van de verwerker' --kanaal verwerker");

    let uit = p.moet(
        "incident verwerker 2026-0041 --leverancier LEV-014 \
         --opgetreden 2026-06-01T09:00:00Z --gemeld 2026-06-03T09:00:00Z",
    );
    assert!(uit.contains("48 uur"), "de werkelijke duur hoort in beeld:\n{uit}");
    assert!(uit.contains("24 uur"), "de afgesproken termijn hoort in beeld:\n{uit}");

    let uit = p.moet("controle --vanaf rapporterend");
    assert!(uit.contains("LEK-16"), "kreeg:\n{uit}");
    assert!(uit.contains("Zorgdossier BV"), "de naam van de verwerker hoort erbij:\n{uit}");
}

/// Twee tijdstippen die zijn verwisseld, leveren een negatieve duur op. Dat is
/// geen bevinding maar een invoerfout, en die hoort te worden geweigerd.
#[test]
fn een_melding_van_voor_het_incident_wordt_geweigerd() {
    let p = Proef::nieuw();
    leverancier_opzetten(&p);
    p.moet("incident nieuw 2026-0042 'melding van de verwerker' --kanaal verwerker");

    let uit = p.moet_falen(
        "incident verwerker 2026-0042 --leverancier LEV-014 \
         --opgetreden 2026-06-03T09:00:00Z --gemeld 2026-06-01T09:00:00Z",
    );
    assert!(uit.contains("verwisseld"), "kreeg:\n{uit}");
}

#[test]
fn een_onbekende_leverancier_levert_een_bruikbare_fout() {
    let p = Proef::nieuw();
    p.moet("incident nieuw 2026-0043 'melding van de verwerker' --kanaal verwerker");
    let uit = p.moet_falen("incident verwerker 2026-0043 --leverancier LEV-999");
    assert!(uit.contains("dpofg leverancier lijst"), "kreeg:\n{uit}");
}

// --------------------------------------------------------------------------
// De zorgplichtcontrolset
// --------------------------------------------------------------------------

fn controlset_opzetten(p: &Proef) {
    p.moet(
        "zorgplicht afleiden ZRP-2026 --naam 'Gemeente Voorbeeld' \
         --functionaris 'A. de Vries'",
    );
}

/// De tien onderdelen staan in het product, met de juiste lidverwijzing. De
/// richtlijn nummert ze als lid 2, de Nederlandse wet als lid 3.
#[test]
fn de_tien_onderdelen_verwijzen_naar_het_derde_lid() {
    let p = Proef::nieuw();
    let uit = p.moet("zorgplicht onderdelen");
    for letter in ["a", "b", "c", "d", "e", "f", "g", "h", "i", "j"] {
        assert!(
            uit.contains(&format!("art. 21 lid 3 onder {letter} Cyberbeveiligingswet")),
            "kreeg:\n{uit}"
        );
    }
    assert!(!uit.contains("lid 2 onder"), "de verwijzing van de richtlijn hoort er niet te staan");
}

/// De controlset wordt afgeleid en niet samengesteld: er is geen opdracht om
/// er een maatregel bij te zetten of uit te halen.
#[test]
fn de_controlset_wordt_in_een_keer_afgeleid() {
    let p = Proef::nieuw();
    let uit = p.moet(
        "zorgplicht afleiden ZRP-2026 --naam 'Gemeente Voorbeeld' \
         --functionaris 'A. de Vries'",
    );
    assert!(uit.contains("afgeleid en niet samengesteld"), "kreeg:\n{uit}");
    assert!(uit.contains("niet tegen de bron geverifieerd"), "kreeg:\n{uit}");

    let uit = p.moet("zorgplicht toon ZRP-2026");
    for letter in ["a", "b", "c", "d", "e", "f", "g", "h", "i", "j"] {
        assert!(uit.contains(&format!("│ {letter} ")), "onderdeel {letter} ontbreekt:\n{uit}");
    }
}

/// Een kader dat niet in het pakket zit, is niet te kiezen. Anders ontstaat een
/// leeg dossier dat er compleet uitziet omdat het nul maatregelen heeft.
#[test]
fn een_kader_dat_er_niet_is_wordt_niet_afgeleid() {
    let p = Proef::nieuw();
    let uit = p.moet_falen(
        "zorgplicht afleiden ZRP-2026 --naam Gemeente --kader UV-VARIANT-B \
         --functionaris 'A. de Vries'",
    );
    assert!(uit.contains("dpofg zorgplicht kaders"), "kreeg:\n{uit}");
}

/// Inrichten maakt niets aantoonbaar. Dat verschil is de kern van deze module.
#[test]
fn inrichten_is_geen_bewijs() {
    let p = Proef::nieuw();
    controlset_opzetten(&p);
    p.moet("zorgplicht eigenaar ZRP-2026 --maatregel CBB-13 --rol 'de systeembeheerder' --persoon 'J. Jansen'");

    let uit = p.moet("zorgplicht inrichten ZRP-2026 --maatregel CBB-13");
    assert!(uit.contains("Ingericht is nog niet aantoonbaar"), "kreeg:\n{uit}");

    let uit = p.moet("controle");
    assert!(uit.contains("ZRP-04"), "kreeg:\n{uit}");
    assert!(uit.contains("zonder bewijs van de uitvoering"), "kreeg:\n{uit}");

    let bewijs = p.bestand("uitdraai.txt", "sleutelbeheer, uitdraai van 1 juli 2026");
    let uit = p.moet(&format!(
        "zorgplicht bewijs ZRP-2026 --maatregel CBB-13 --rol uitvoering --bestand {bewijs} \
         --omschrijving 'uitdraai van het sleutelbeheer' --geldig-tot 2027-06-01T00:00:00Z"
    ));
    assert!(uit.contains("stand van CBB-13: aantoonbaar"), "kreeg:\n{uit}");
    assert!(!p.moet("controle").contains("ZRP-04"));
}

/// De gebruiker typt nooit een hash over: hij wijst een bestand aan en de tool
/// rekent de inhoudshash zelf uit.
#[test]
fn bewijs_is_een_bestand_en_geen_verwijzing() {
    let p = Proef::nieuw();
    controlset_opzetten(&p);
    let leeg = p.bestand("leeg.txt", "");
    let uit = p.moet_falen(&format!(
        "zorgplicht bewijs ZRP-2026 --maatregel CBB-13 --rol uitvoering --bestand {leeg} \
         --omschrijving 'niets' --geldig-tot 2027-06-01T00:00:00Z"
    ));
    assert!(uit.contains("leeg bestand als bewijs"), "kreeg:\n{uit}");

    let uit = p.moet_falen(
        "zorgplicht bewijs ZRP-2026 --maatregel CBB-13 --rol uitvoering \
         --bestand /bestaat/niet.pdf --omschrijving 'iets' --geldig-tot 2027-06-01T00:00:00Z",
    );
    assert!(uit.contains("niet lezen"), "kreeg:\n{uit}");
}

/// Bewijs zonder einddatum houdt een dossier voor altijd groen. Dat kan niet:
/// de einddatum is verplicht en verlopen bewijs aanwijzen wordt geweigerd.
#[test]
fn verlopen_bewijs_aanwijzen_kan_niet() {
    let p = Proef::nieuw();
    controlset_opzetten(&p);
    let bewijs = p.bestand("oud.txt", "uitdraai van 2020");
    let uit = p.moet_falen(&format!(
        "zorgplicht bewijs ZRP-2026 --maatregel CBB-13 --rol uitvoering --bestand {bewijs} \
         --omschrijving 'oude uitdraai' --geldig-van 2020-01-01T00:00:00Z \
         --geldig-tot 2021-01-01T00:00:00Z"
    ));
    assert!(uit.contains("nu al verlopen"), "kreeg:\n{uit}");
}

/// De aangemelde functionaris houdt geen toezicht op zijn eigen werk. Bij
/// toewijzing wordt dat geweigerd; bij een rolwissel is dat niet mogelijk en
/// wordt het een blokkerende bevinding.
#[test]
fn de_functionaris_kan_geen_maatregel_bezitten() {
    let p = Proef::nieuw();
    controlset_opzetten(&p);
    let uit = p.moet_falen(
        "zorgplicht eigenaar ZRP-2026 --maatregel CBB-13 --rol 'de beheerder' \
         --persoon 'A. de Vries'",
    );
    assert!(uit.contains("art. 38 lid 6 AVG"), "kreeg:\n{uit}");

    p.moet("zorgplicht eigenaar ZRP-2026 --maatregel CBB-13 --rol 'de beheerder' --persoon 'J. Jansen'");
    let uit = p.moet("zorgplicht functionaris ZRP-2026 --naam 'J. Jansen'");
    assert!(uit.contains("toezicht op het eigen werk"), "kreeg:\n{uit}");

    let uit = p.moet("controle");
    assert!(uit.contains("ZRP-02"), "kreeg:\n{uit}");
}

/// Een onvoorwaardelijke eis is geen keuze die met een motivering te maken is.
/// Het kader bepaalt dat, niet de gebruiker.
#[test]
fn het_kader_bepaalt_of_afwijken_mag() {
    let p = Proef::nieuw();
    controlset_opzetten(&p);

    let uit = p.moet_falen(
        "zorgplicht niet-toepassen ZRP-2026 --maatregel CBB-06 \
         --motivering 'dit past niet bij onze omvang en middelen'",
    );
    assert!(uit.contains("onvoorwaardelijk geformuleerd"), "kreeg:\n{uit}");

    // CBB-13-E draagt in het pakket wél een voorbehoud.
    let uit = p.moet(
        "zorgplicht niet-toepassen ZRP-2026 --maatregel CBB-13-E \
         --motivering 'de gegevens verlaten het eigen netwerk niet'",
    );
    assert!(uit.contains("besluit dat blijft staan"), "kreeg:\n{uit}");
}

/// Zonder risicobeoordeling en zonder bestuursbesluit is de zorgplicht niet
/// aantoonbaar, hoeveel maatregelen er ook zijn ingericht.
#[test]
fn vaststellen_vergt_de_beoordeling_en_het_bestuursbesluit() {
    let p = Proef::nieuw();
    controlset_opzetten(&p);
    let uit = p.moet_falen("zorgplicht vaststellen ZRP-2026");
    assert!(uit.contains("zorgplicht.risicobeoordeling"), "kreeg:\n{uit}");
    assert!(uit.contains("zorgplicht.bestuursvaststelling"), "kreeg:\n{uit}");
}

/// De vervallijst noemt wat er omvalt en wanneer. Geen score, geen kleur.
#[test]
fn de_vervallijst_toont_datums_en_geen_cijfer() {
    let p = Proef::nieuw();
    controlset_opzetten(&p);
    p.moet("zorgplicht eigenaar ZRP-2026 --maatregel CBB-13 --rol 'de beheerder' --persoon 'J. Jansen'");
    p.moet("zorgplicht inrichten ZRP-2026 --maatregel CBB-13");
    let bewijs = p.bestand("sleutelbeheer.txt", "uitdraai van het sleutelbeheer");
    p.moet(&format!(
        "zorgplicht bewijs ZRP-2026 --maatregel CBB-13 --rol uitvoering --bestand {bewijs} \
         --omschrijving 'uitdraai van het sleutelbeheer' --geldig-tot 2026-10-01T00:00:00Z"
    ));

    let uit = p.moet("zorgplicht vervalt ZRP-2026 --dagen 7");
    assert!(uit.contains("vervalt er geen uitvoeringsbewijs"), "kreeg:\n{uit}");

    let uit = p.moet("zorgplicht vervalt ZRP-2026 --dagen 365");
    assert!(uit.contains("CBB-13"), "kreeg:\n{uit}");
    assert!(uit.contains("01-10-2026"), "kreeg:\n{uit}");
    assert!(uit.contains("lijst met datums en geen prognose"), "kreeg:\n{uit}");
}

/// Vijftien maatregelen mogen niet vijftien bevindingen opleveren die alle
/// vijftien hetzelfde zeggen; dat is hoe een gebruiker leert wegklikken.
#[test]
fn een_verse_controlset_levert_een_leesbare_controleronde() {
    let p = Proef::nieuw();
    controlset_opzetten(&p);
    let uit = p.moet("controle --vanaf rapporterend");
    let aantal = uit.matches("[ZRP-").count();
    assert!(aantal <= 6, "{aantal} bevindingen voor één vers dossier:\n{uit}");
}

/// Het hele werkproces met het meegeleverde kennispakket, tot en met
/// vaststellen. Deze test bestaat omdat de kaderverificatie eerst blokkerend
/// was: het product leverde daarmee een werkproces dat met de eigen inhoud
/// nooit was af te maken, en geen enkele test merkte dat op.
#[test]
fn de_controlset_is_met_het_meegeleverde_pakket_af_te_maken() {
    let p = Proef::nieuw();
    controlset_opzetten(&p);

    let codes = [
        "CBB-06",
        "CBB-07",
        "CBB-08",
        "CBB-09",
        "CBB-10",
        "CBB-11",
        "CBB-17",
        "CBB-18",
        "CBB-12",
        "CBB-13",
        "CBB-13-E",
        "CBB-14",
        "CBB-15",
        "CBB-16",
        "CBB-15-MFA",
    ];
    for code in codes {
        p.moet(&format!(
            "zorgplicht eigenaar ZRP-2026 --maatregel {code} --rol 'de systeembeheerder' \
             --persoon 'J. Jansen'"
        ));
        p.moet(&format!("zorgplicht inrichten ZRP-2026 --maatregel {code}"));
    }
    for code in [
        "CBB-06", "CBB-07", "CBB-08", "CBB-09", "CBB-10", "CBB-11", "CBB-18", "CBB-12", "CBB-15",
        "CBB-16",
    ] {
        p.moet(&format!(
            "zorgplicht frequentie ZRP-2026 --maatregel {code} --maanden 12 \
             --door 'de directie' --motivering 'jaarlijks sluit aan op de planning en control'"
        ));
    }
    let verslag = p.bestand("risicobeoordeling.txt", "verslag van de risicobeoordeling");
    p.moet(&format!(
        "zorgplicht risicobeoordeling ZRP-2026 --methode scenarioanalyse \
         --scope 'de hele organisatie' --uitgevoerd-door 'de security officer' \
         --geldig-tot 2027-06-01T00:00:00Z --bestand {verslag} \
         --omschrijving 'verslag van de jaarlijkse beoordeling'"
    ));
    let besluit = p.bestand("besluit.txt", "besluit van het bestuur");
    p.moet(&format!(
        "zorgplicht bestuursvaststelling ZRP-2026 --besluit 'het maatregelenpakket is \
         vastgesteld' --aanwezige 'de directie' --aanwezige 'de security officer' \
         --bestand {besluit} --geldig-tot 2027-06-01T00:00:00Z"
    ));

    let uit = p.moet("zorgplicht vaststellen ZRP-2026");
    assert!(uit.contains("vastgesteld"), "kreeg:\n{uit}");

    // En toch is geen enkele maatregel aantoonbaar. Dat hoort te blijven staan.
    assert!(uit.contains("vastgesteld, niet aantoonbaar"), "kreeg:\n{uit}");
    let uit = p.moet("controle");
    assert!(uit.contains("ZRP-04"), "kreeg:\n{uit}");
    // Het kader is niet tegen de bron gehouden; dat blokkeert niet en verdwijnt
    // ook niet.
    let uit = p.moet("zorgplicht toon ZRP-2026");
    assert!(uit.contains("niet tegen de bron"), "kreeg:\n{uit}");
}

/// Een onvoorwaardelijke eis kan niet worden afgeweken, dus het aandeel meet
/// over de maatregelen waar dat wél mag. Meet het over de hele set, dan kan
/// ZRP-13 met dit kader nooit aanslaan.
#[test]
fn het_aandeel_afwijkingen_meet_over_wat_afwijkbaar_is() {
    let p = Proef::nieuw();
    controlset_opzetten(&p);
    p.moet(
        "zorgplicht niet-toepassen ZRP-2026 --maatregel CBB-13-E \
         --motivering 'de gegevens verlaten het eigen netwerk niet'",
    );
    p.moet(
        "zorgplicht niet-toepassen ZRP-2026 --maatregel CBB-15-MFA \
         --motivering 'er is geen toegang van buiten het eigen netwerk'",
    );

    let uit = p.moet("controle --vanaf rapporterend");
    assert!(uit.contains("ZRP-13"), "kreeg:\n{uit}");
    assert!(uit.contains("100 procent"), "kreeg:\n{uit}");
}
