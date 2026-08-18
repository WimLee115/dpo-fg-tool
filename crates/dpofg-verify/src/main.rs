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
        Ok(geslaagd) => {
            if !geslaagd {
                std::process::exit(2);
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

/// Levert `false` wanneer de controle inhoudelijk faalt, en `Err` wanneer er
/// iets misging bij het lezen. Dat onderscheid telt: een onleesbaar bestand is
/// iets anders dan een dossier dat niet klopt.
fn draai() -> Result<bool> {
    let args = Opdrachtregel::parse();
    match args.opdracht {
        Opdracht::Dossier { manifest, stukken } => dossier(&manifest, stukken),
        Opdracht::Logboek { regels, anker } => logboek(&regels, anker),
        Opdracht::Anker { bestand } => anker_alleen(&bestand),
    }
}

fn dossier(manifestpad: &PathBuf, stukkenmap: Option<PathBuf>) -> Result<bool> {
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

    let mut geslaagd = true;

    println!("\nHandtekening");
    match ondertekend.controleer() {
        Ok(()) => println!("  ✓ het manifest is niet gewijzigd na ondertekening"),
        Err(e) => {
            println!("  ✗ {e}");
            geslaagd = false;
        }
    }
    println!(
        "  ondertekenaar: {}",
        &ondertekend.ondertekenaar[..16.min(ondertekend.ondertekenaar.len())]
    );
    println!(
        "  Let op: deze controle toont aan dat de houder van deze sleutel het manifest heeft\n  \
         ondertekend. Of die sleutel toebehoort aan wie u verwacht, is een vraag die dit\n  \
         programma niet kan beantwoorden."
    );

    let map = stukkenmap.unwrap_or_else(|| {
        manifestpad.parent().map(|p| p.to_path_buf()).unwrap_or_else(|| PathBuf::from("."))
    });
    let mut stukken = Vec::new();
    for stuk in &m.stukken {
        let pad = map.join(&stuk.naam);
        match std::fs::read(&pad) {
            Ok(inhoud) => stukken.push((stuk.naam.clone(), inhoud)),
            Err(_) => {} // wordt hieronder als ontbrekend gemeld
        }
    }

    println!("\nStukken");
    let afwijkingen = ondertekend.controleer_stukken(&stukken);
    if afwijkingen.is_empty() {
        println!("  ✓ alle {} stukken komen overeen met het manifest", m.stukken.len());
    } else {
        geslaagd = false;
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
    if geslaagd {
        println!("UITKOMST: de controle is geslaagd.");
    } else {
        println!("UITKOMST: de controle is NIET geslaagd. Zie de punten hierboven.");
    }
    Ok(geslaagd)
}

fn logboek(regelpad: &PathBuf, ankerpad: Option<PathBuf>) -> Result<bool> {
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

    println!("\nReikwijdte");
    for regel in wikkel(&rapport.reikwijdte(), 76) {
        println!("  {regel}");
    }

    println!();
    if rapport.is_ongeschonden() {
        println!("UITKOMST: de controle is geslaagd.");
        Ok(true)
    } else {
        println!("UITKOMST: de controle is NIET geslaagd. Zie de punten hierboven.");
        Ok(false)
    }
}

fn anker_alleen(pad: &PathBuf) -> Result<bool> {
    let tekst = std::fs::read_to_string(pad)
        .with_context(|| format!("kon {} niet lezen", pad.display()))?;
    let anker: dpofg_audit::Anker =
        serde_json::from_str(&tekst).context("het anker is niet leesbaar")?;

    println!("Anker");
    println!("  kluis     : {}", anker.kluis_id);
    println!("  regel     : {}", anker.volgnummer);
    println!("  hash      : {}", anker.hash);
    println!("  tijdstip  : {}", anker.tijdstip.format("%d-%m-%Y %H:%M UTC"));
    if let Some(p) = &anker.bewaarplaats {
        println!("  bewaard in: {p}");
    }

    println!();
    match anker.controleer_handtekening() {
        Ok(()) => {
            println!("✓ de handtekening klopt: dit anker is niet gewijzigd.");
            println!(
                "\nLet op: het tijdstip hierboven komt van de machine die het anker maakte.\n\
                 Dat het anker vóór een bepaald moment bestond, blijkt niet uit dit bestand\n\
                 maar uit de plaats waar het buiten het systeem is bewaard."
            );
            Ok(true)
        }
        Err(e) => {
            println!("✗ {e}");
            Ok(false)
        }
    }
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
