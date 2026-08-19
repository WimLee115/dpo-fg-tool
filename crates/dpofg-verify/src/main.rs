//! `dpofg-verify` — de verificatiebinary voor toezichthouders en auditors.
//!
//! # Waarom dit een aparte binary is
//!
//! Een organisatie die een dossier aanlevert, zegt: *dit is niet gewijzigd.*
//! Die bewering is waardeloos als zij alleen te controleren is met software
//! van diezelfde organisatie. Daarom is dit een losse, apart ondertekende
//! binary die:
//!
//! * **alleen leest** — er is geen code in dit programma die iets schrijft;
//! * **de kluis niet nodig heeft** — hij controleert een aangeleverd dossier,
//!   niet het systeem waaruit het komt;
//! * **geen wachtwoord vraagt** — een dossier dat een wachtwoord vereist om te
//!   controleren, is geen dossier maar een belofte;
//! * **klein genoeg is om te lezen** — wie het niet vertrouwt, kan het nalopen.
//!
//! # Wat een geslaagde controle betekent, en wat niet
//!
//! Zij betekent: de aangeleverde stukken komen overeen met het manifest, en de
//! logboekketen is intern samenhangend. Zij betekent **niet** dat het dossier
//! volledig is, dat het de juiste stukken bevat, of dat de vastgelegde
//! tijdstippen kloppen. Dat laatste kan alleen met een anker dat buiten het
//! systeem is bewaard, en dat wordt hier expliciet gemeld.

#![forbid(unsafe_code)]

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use dpofg_audit::{Ankerstatus, Ketenregel};
use dpofg_report::OndertekendManifest;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "dpofg-verify",
    version,
    about = "Controleert een aangeleverd dossier van dpo-fg-tool.",
    long_about = "Leest uitsluitend. Vraagt geen wachtwoord en heeft de kluis niet nodig.\n\n\
                  Een geslaagde controle toont aan dat de stukken overeenkomen met het manifest \
                  en dat de logboekketen samenhangend is. Zij toont niet aan dat het dossier \
                  volledig is of dat de vastgelegde tijdstippen kloppen."
)]
struct Opdrachtregel {
    #[command(subcommand)]
    opdracht: Opdracht,

    /// De publieke installatiesleutel waarvan u een stuk verwacht, 64
    /// hexadecimale tekens. Herhaalbaar. Zonder deze vlag wordt de
    /// handtekening wel gecontroleerd maar de herkomst niet.
    ///
    /// Uitsluitend de sleutel zelf, geen bestandspad: een bestand uit dezelfde
    /// levering aanwijzen als bron van vertrouwen, is geen controle.
    #[arg(
        long = "sleutel",
        global = true,
        value_name = "64 HEX",
        value_parser = sleutel_uit_tekst
    )]
    sleutels: Vec<String>,
}

/// Aanvaardt uitsluitend 64 hexadecimale tekens; normaliseert naar kleine
/// letters zodat de vergelijking verderop letterlijk kan.
fn sleutel_uit_tekst(ruw: &str) -> Result<String, String> {
    let s = ruw.trim().to_ascii_lowercase();
    if s.len() != 64 || !s.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(format!(
            "een installatiesleutel bestaat uit 64 hexadecimale tekens; gekregen: {} teken(s)",
            s.chars().count()
        ));
    }
    Ok(s)
}

/// De uitkomst van een controle.
///
/// Drie uitkomsten en geen twee, omdat "het stuk is gewijzigd" en "het stuk is
/// van een andere installatie" verschillende gevolgen hebben voor wie het leest.
/// Code 3 kan alleen optreden wanneer `--sleutel` is meegegeven; een script dat
/// die vlag niet gebruikt, ziet nooit iets anders dan voorheen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Uitkomst {
    Geslaagd,
    Afwijking,
    VreemdeOndertekenaar,
}

impl Uitkomst {
    fn afsluitcode(self) -> i32 {
        match self {
            Self::Geslaagd => 0,
            Self::Afwijking => 2,
            Self::VreemdeOndertekenaar => 3,
        }
    }

    /// Een echte afwijking weegt zwaarder dan een vreemde ondertekenaar: wie
    /// een gewijzigd stuk in handen heeft, moet dát lezen en niet de mededeling
    /// dat het van iemand anders komt.
    fn samen(self, andere: Self) -> Self {
        match (self, andere) {
            (Self::Afwijking, _) | (_, Self::Afwijking) => Self::Afwijking,
            (Self::VreemdeOndertekenaar, _) | (_, Self::VreemdeOndertekenaar) => {
                Self::VreemdeOndertekenaar
            }
            _ => Self::Geslaagd,
        }
    }
}

#[derive(Subcommand, Debug)]
enum Opdracht {
    /// Controleer een dossiermanifest en de bijbehorende stukken.
    Dossier {
        /// Het manifestbestand.
        manifest: PathBuf,
        /// De map met de stukken. Standaard: de map van het manifest.
        #[arg(long)]
        stukken: Option<PathBuf>,
    },
    /// Controleer een uitgevoerd logboek, eventueel tegen een anker.
    Logboek {
        /// Het bestand met de logboekregels.
        regels: PathBuf,
        /// Het ankerbestand.
        #[arg(long)]
        anker: Option<PathBuf>,
    },
    /// Controleer een los anker op zijn handtekening.
    Anker {
        /// Het ankerbestand.
        bestand: PathBuf,
    },
}

fn main() {
    match draai() {
        Ok(uitkomst) => {
            let code = uitkomst.afsluitcode();
            if code != 0 {
                std::process::exit(code);
            }
        }
        Err(fout) => {
            eprintln!("fout: {fout}");
            let mut bron = fout.source();
            while let Some(b) = bron {
                eprintln!("  ← {b}");
                bron = b.source();
            }
            std::process::exit(1);
        }
    }
}

/// Levert een [`Uitkomst`] wanneer de controle is uitgevoerd, en `Err` wanneer
/// er iets misging bij het lezen. Dat onderscheid telt: een onleesbaar bestand
/// is iets anders dan een dossier dat niet klopt.
fn draai() -> Result<Uitkomst> {
    let args = Opdrachtregel::parse();
    let sleutels = args.sleutels;
    match args.opdracht {
        Opdracht::Dossier { manifest, stukken } => dossier(&manifest, stukken, &sleutels),
        Opdracht::Logboek { regels, anker } => logboek(&regels, anker, &sleutels),
        Opdracht::Anker { bestand } => anker_alleen(&bestand, &sleutels),
    }
}

fn dossier(
    manifestpad: &PathBuf,
    stukkenmap: Option<PathBuf>,
    sleutels: &[String],
) -> Result<Uitkomst> {
    let tekst = std::fs::read_to_string(manifestpad)
        .with_context(|| format!("kon {} niet lezen", manifestpad.display()))?;
    let ondertekend: OndertekendManifest =
        serde_json::from_str(&tekst).context("het manifest is niet leesbaar")?;
    let m = &ondertekend.manifest;

    println!("Dossier");
    println!("  aanleiding        : {}", m.aanleiding);
    println!("  bestemd voor      : {}", m.bestemd_voor);
    println!(
        "  samengesteld      : {} door {}",
        m.samengesteld_op.format("%d-%m-%Y %H:%M UTC"),
        m.samengesteld_door
    );
    println!("  stukken           : {}", m.stukken.len());
    println!("  ketenstand        : regel {}", m.keten_volgnummer);
    println!(
        "  juridische inhoud : {} versie {}, geconsolideerd op {}",
        m.kennispakket_code,
        m.kennispakket_versie,
        m.kennispakket_consolidatiedatum.format("%d-%m-%Y")
    );
    println!("  programmaversie   : {}", m.programmaversie);

    let mut uitkomst = Uitkomst::Geslaagd;

    println!("\nHandtekening");
    match ondertekend.controleer() {
        Ok(()) => println!("  ✓ het manifest is niet gewijzigd na ondertekening"),
        Err(e) => {
            println!("  ✗ {e}");
            uitkomst = uitkomst.samen(Uitkomst::Afwijking);
        }
    }
    println!("  ondertekenaar: {}", ondertekend.ondertekenaar);

    if sleutels.is_empty() {
        println!(
            "  Let op: deze controle toont aan dat de houder van deze sleutel het manifest heeft\n  \
             ondertekend. Of die sleutel toebehoort aan wie u verwacht, is een vraag die dit\n  \
             programma niet kan beantwoorden. Geef de gepubliceerde sleutel mee met --sleutel om\n  \
             dat wél vast te stellen."
        );
    } else if sleutels.iter().any(|k| k.eq_ignore_ascii_case(&ondertekend.ondertekenaar)) {
        // Alleen de sleutel vergelijken, niet de handtekening: die is hierboven
        // al beoordeeld. Zouden we hier `controleer_ondertekenaar` gebruiken,
        // dan meldde een gemanipuleerd manifest van de júiste installatie dat
        // het "van een andere sleutel" komt — en dat duwt de lezer naar de
        // onschuldige verklaring terwijl het stuk is gewijzigd.
        println!(
            "  ✓ deze sleutel is de sleutel die u hebt opgegeven; het manifest komt van die\n  \
             installatie. Dat toont niet aan dat de inhoud juist of volledig is."
        );
    } else {
        println!(
            "  ✗ ondertekend met een andere sleutel dan u hebt opgegeven. Bestanden van vóór de\n  \
             uitgave met vast sleutelbeheer dragen een wegwerpsleutel; die kunnen nooit\n  \
             overeenkomen."
        );
        uitkomst = uitkomst.samen(Uitkomst::VreemdeOndertekenaar);
    }

    if let Err(e) = ondertekend.controleer_voorbehoud() {
        println!("  ✗ {e}");
        uitkomst = uitkomst.samen(Uitkomst::Afwijking);
    }

    let map = stukkenmap.unwrap_or_else(|| {
        manifestpad.parent().map(|p| p.to_path_buf()).unwrap_or_else(|| PathBuf::from("."))
    });
    let mut stukken = Vec::new();
    for stuk in &m.stukken {
        let pad = map.join(&stuk.naam);
        // Een stuk dat niet te lezen is, wordt hieronder als ontbrekend gemeld;
        // stilzwijgend overslaan zou het verschil verdoezelen tussen een stuk
        // dat er niet is en een stuk dat er wel is maar niet klopt.
        if let Ok(inhoud) = std::fs::read(&pad) {
            stukken.push((stuk.naam.clone(), inhoud));
        }
    }

    println!("\nStukken");
    let afwijkingen = ondertekend.controleer_stukken(&stukken);
    if afwijkingen.is_empty() {
        println!("  ✓ alle {} stukken komen overeen met het manifest", m.stukken.len());
    } else {
        uitkomst = uitkomst.samen(Uitkomst::Afwijking);
        for a in &afwijkingen {
            println!("  ✗ {a}");
        }
    }

    if !m.weggelaten.is_empty() {
        println!("\nBewust weggelaten");
        for w in &m.weggelaten {
            println!("  • {} ({}×) — {}", w.omschrijving, w.aantal, w.reden);
        }
    }

    println!("\nReikwijdte volgens het dossier");
    for regel in m.reikwijdte.split(". ") {
        if !regel.trim().is_empty() {
            println!("  {}.", regel.trim().trim_end_matches('.'));
        }
    }

    println!("\nVoorbehoud");
    for regel in wikkel(&m.voorbehoud, 76) {
        println!("  {regel}");
    }

    println!();
    match uitkomst {
        Uitkomst::Geslaagd => println!("UITKOMST: de controle is geslaagd."),
        Uitkomst::VreemdeOndertekenaar => println!(
            "UITKOMST: het dossier is niet gewijzigd, maar komt van een andere installatie dan \
             de sleutel die u hebt opgegeven."
        ),
        Uitkomst::Afwijking => {
            println!("UITKOMST: de controle is NIET geslaagd. Zie de punten hierboven.")
        }
    }
    Ok(uitkomst)
}

fn logboek(regelpad: &PathBuf, ankerpad: Option<PathBuf>, sleutels: &[String]) -> Result<Uitkomst> {
    // Een opgegeven sleutel zonder anker is een vergelijking zonder tegenpartij.
    // Groen melden zou hier het gevaarlijkst zijn: wie dit in een script zet,
    // leest het als "de herkomst is vastgesteld".
    if !sleutels.is_empty() && ankerpad.is_none() {
        anyhow::bail!(
            "er is een sleutel opgegeven maar geen anker. In een logboek staat geen \
             handtekening; alleen een anker draagt er een. Geef het ankerbestand mee met \
             --anker, of laat --sleutel weg."
        );
    }

    let tekst = std::fs::read_to_string(regelpad)
        .with_context(|| format!("kon {} niet lezen", regelpad.display()))?;
    let regels: Vec<Ketenregel> =
        serde_json::from_str(&tekst).context("het logboek is niet leesbaar")?;

    let anker = match &ankerpad {
        None => None,
        Some(p) => {
            let t = std::fs::read_to_string(p)
                .with_context(|| format!("kon {} niet lezen", p.display()))?;
            Some(
                serde_json::from_str::<dpofg_audit::Anker>(&t)
                    .context("het anker is niet leesbaar")?,
            )
        }
    };

    // De pin vóór de ketenverificatie: een anker van een vreemde installatie
    // hoort als zodanig gemeld te worden en niet eerst als geldig bevonden.
    let mut uitkomst = Uitkomst::Geslaagd;
    let mut vreemd = None;
    if let (Some(a), false) = (anker.as_ref(), sleutels.is_empty()) {
        // Uitsluitend de sleutelvergelijking: een gebroken handtekening meldt
        // `verifieer` hieronder al als `AnkerOngeldig`, en die melding twee keer
        // afdrukken met verschillende bewoordingen helpt niemand.
        if !sleutels.iter().any(|k| k.eq_ignore_ascii_case(&a.sleutel)) {
            vreemd = Some(a.sleutel.to_ascii_lowercase());
            uitkomst = uitkomst.samen(Uitkomst::VreemdeOndertekenaar);
        }
    }

    let rapport = dpofg_audit::verifieer(&regels, anker.as_ref())?;

    println!("Logboek");
    println!("  regels : {}", rapport.regels);
    if let Some((van, tot)) = rapport.periode {
        println!(
            "  periode: {} tot {}",
            van.format("%d-%m-%Y %H:%M"),
            tot.format("%d-%m-%Y %H:%M")
        );
    }

    println!("\nBevindingen");
    if rapport.bevindingen.is_empty() {
        println!("  ✓ geen wijzigingen, geen ontbrekende regels, geen dubbele volgnummers");
    } else {
        for b in &rapport.bevindingen {
            println!("  ✗ regel {}: {}", b.volgnummer, b.omschrijving);
        }
    }

    println!("\nAnker");
    match &rapport.ankerstatus {
        Ankerstatus::GeenAnker => println!("  geen anker meegegeven"),
        Ankerstatus::Bevestigd { volgnummer, regels_sinds_anker } => println!(
            "  ✓ bevestigd tot en met regel {volgnummer}; {regels_sinds_anker} regels daarna \
             rusten alleen op de keten"
        ),
        Ankerstatus::KetenIsIngekort { anker_volgnummer, keten_volgnummer } => println!(
            "  ✗ het anker verklaart regel {anker_volgnummer}, de keten eindigt bij \
             {keten_volgnummer}: er zijn regels verwijderd"
        ),
        Ankerstatus::HashWijktAf { volgnummer, .. } => println!(
            "  ✗ op ankerpositie {volgnummer} wijkt de hash af: de inhoud is na het ankeren \
             gewijzigd"
        ),
        Ankerstatus::AnkerOngeldig(reden) => println!("  ✗ het anker is niet bruikbaar: {reden}"),
    }
    match (&vreemd, anker.is_some(), sleutels.is_empty()) {
        (Some(sleutel), _, _) => println!(
            "  ✗ dit anker is ondertekend met sleutel {sleutel}; die staat niet in de lijst \
             sleutels waarmee u vergelijkt.\n    Ankers van vóór de uitgave met vast \
             sleutelbeheer dragen een wegwerpsleutel; die kunnen nooit overeenkomen."
        ),
        (None, true, false) => {
            println!("  ✓ het anker komt van de installatie waarvan u de sleutel hebt opgegeven")
        }
        (None, true, true) => println!(
            "  Let op: van wie dit anker komt, is hiermee niet vastgesteld. Geef de gepubliceerde \
             sleutel mee met --sleutel."
        ),
        _ => {}
    }

    println!("\nReikwijdte");
    for regel in wikkel(&rapport.reikwijdte(), 76) {
        println!("  {regel}");
    }

    if !rapport.is_ongeschonden() {
        uitkomst = uitkomst.samen(Uitkomst::Afwijking);
    }

    println!();
    match uitkomst {
        Uitkomst::Geslaagd => println!("UITKOMST: de controle is geslaagd."),
        Uitkomst::VreemdeOndertekenaar => println!(
            "UITKOMST: de keten is samenhangend, maar het anker komt van een andere installatie \
             dan de sleutel die u hebt opgegeven."
        ),
        Uitkomst::Afwijking => {
            println!("UITKOMST: de controle is NIET geslaagd. Zie de punten hierboven.")
        }
    }
    Ok(uitkomst)
}

fn anker_alleen(pad: &PathBuf, sleutels: &[String]) -> Result<Uitkomst> {
    let tekst = std::fs::read_to_string(pad)
        .with_context(|| format!("kon {} niet lezen", pad.display()))?;
    let anker: dpofg_audit::Anker =
        serde_json::from_str(&tekst).context("het anker is niet leesbaar")?;

    println!("Anker");
    println!("  kluis     : {}", anker.kluis_id);
    println!("  regel     : {}", anker.volgnummer);
    println!("  hash      : {}", anker.hash);
    println!("  tijdstip  : {}", anker.tijdstip.format("%d-%m-%Y %H:%M UTC"));
    println!("  sleutel   : {}", anker.sleutel);
    if let Some(p) = &anker.bewaarplaats {
        println!("  bewaard in: {p}");
    }

    println!();
    let mut uitkomst = Uitkomst::Geslaagd;
    if !sleutels.is_empty() {
        if let Err(e) = anker.controleer_ondertekenaar(sleutels) {
            if matches!(e, dpofg_audit::AuditFout::OnbekendeOndertekenaar { .. }) {
                println!(
                    "✗ {e}\n  Ankers van vóór de uitgave met vast sleutelbeheer dragen een \
                     wegwerpsleutel; die kunnen nooit overeenkomen."
                );
                uitkomst = uitkomst.samen(Uitkomst::VreemdeOndertekenaar);
            }
        } else {
            println!("✓ dit anker komt van de installatie waarvan u de sleutel hebt opgegeven.");
        }
    }

    match anker.controleer_handtekening() {
        Ok(()) => {
            println!("✓ de handtekening klopt: dit anker is niet gewijzigd.");
            if sleutels.is_empty() {
                println!(
                    "  Van wie dit anker komt, is hiermee niet vastgesteld. Geef de gepubliceerde \
                     sleutel mee met --sleutel."
                );
            }
            println!(
                "\nLet op: het tijdstip hierboven komt van de machine die het anker maakte.\n\
                 Dat het anker vóór een bepaald moment bestond, blijkt niet uit dit bestand\n\
                 maar uit de plaats waar het buiten het systeem is bewaard."
            );
        }
        Err(e) => {
            println!("✗ {e}");
            uitkomst = uitkomst.samen(Uitkomst::Afwijking);
        }
    }
    Ok(uitkomst)
}

/// Breekt tekst af op woordgrenzen.
fn wikkel(tekst: &str, breedte: usize) -> Vec<String> {
    let mut uit = Vec::new();
    let mut regel = String::new();
    for woord in tekst.split_whitespace() {
        if !regel.is_empty() && regel.chars().count() + 1 + woord.chars().count() > breedte {
            uit.push(std::mem::take(&mut regel));
        }
        if !regel.is_empty() {
            regel.push(' ');
        }
        regel.push_str(woord);
    }
    if !regel.is_empty() {
        uit.push(regel);
    }
    uit
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wikkelen_breekt_op_woordgrenzen() {
        let r = wikkel("een tekst die langer is dan de breedte toestaat", 20);
        assert!(r.len() > 1);
        for regel in &r {
            assert!(regel.chars().count() <= 20, "te lange regel: {regel}");
            assert!(!regel.starts_with(' '));
        }
        assert_eq!(r.join(" "), "een tekst die langer is dan de breedte toestaat");
    }

    #[test]
    fn wikkelen_verliest_geen_woorden() {
        let tekst = dpofg_report::VOORBEHOUD;
        let r = wikkel(tekst, 76);
        assert_eq!(r.join(" ").split_whitespace().count(), tekst.split_whitespace().count());
    }
}
