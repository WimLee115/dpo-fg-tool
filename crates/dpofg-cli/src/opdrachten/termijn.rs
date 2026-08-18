//! Termijnen berekenen.
//!
//! Deze opdracht bestaat ook los van een kluis: een functionaris die snel wil
//! weten wanneer een termijn afloopt, hoeft daarvoor geen dossier te openen.

use anyhow::Result;
use chrono::{DateTime, Utc};
use clap::Args;
use dpofg_content::startpakket;
use dpofg_terms::{bereken, tijdzone, ToegepasteVerlenging, TIJDZONE_NL};

use crate::uitvoer::{duur, kop, let_op, tabel, terzijde};

#[derive(Args, Debug)]
pub struct Termijnopties {
    /// De code van de termijn, bijvoorbeeld AVG-33-MELDING.
    /// Laat leeg om alle beschikbare termijnen te tonen.
    pub code: Option<String>,
    /// Het ankermoment, bijvoorbeeld 2026-08-21T16:40:00+02:00.
    /// Standaard: nu.
    #[arg(long)]
    pub anker: Option<String>,
    /// De tijdzone waarin het antwoord wordt weergegeven.
    #[arg(long, default_value = TIJDZONE_NL)]
    pub tijdzone: String,
}

pub fn draai(o: Termijnopties, nu: DateTime<Utc>) -> Result<()> {
    let pakket = startpakket(nu.date_naive());

    let Some(code) = o.code else {
        kop("Beschikbare termijnen");
        let mut t = tabel(&["code", "duur", "naam", "grondslag"]);
        for s in &pakket.termijnen {
            t.add_row(vec![
                s.code.clone(),
                s.duur_in_woorden(),
                s.naam.clone(),
                s.grondslag.clone(),
            ]);
        }
        println!("{t}");
        terzijde(&format!(
            "uit kennispakket '{}' versie {}, geconsolideerd op {}",
            pakket.code,
            pakket.versienaam,
            pakket.consolidatiedatum.format("%d-%m-%Y")
        ));
        return Ok(());
    };

    let soort = pakket.termijn(&code)?;
    let anker: DateTime<Utc> = match &o.anker {
        None => nu,
        Some(s) => s
            .parse::<DateTime<chrono::FixedOffset>>()
            .map(|t| t.with_timezone(&Utc))
            .map_err(|e| {
                anyhow::anyhow!(
                    "kon '{s}' niet lezen als tijdstip ({e}). Gebruik de vorm \
                     2026-08-21T16:40:00+02:00 of 2026-08-21T14:40:00Z"
                )
            })?,
    };

    let zone = tijdzone(&o.tijdzone)?;
    let kalender = pakket.kalender("NL")?;
    let deadline = bereken(soort, anker, zone, kalender)?;

    kop(&format!("{} — {}", soort.code, soort.naam));
    let mut t = tabel(&["", ""]);
    t.add_row(vec!["anker", &anker.with_timezone(&zone).format("%d-%m-%Y %H:%M %Z").to_string()]);
    t.add_row(vec!["duur", &deadline.duur]);
    t.add_row(vec!["verstrijkt", &deadline.lokaal]);
    t.add_row(vec!["grondslag", &deadline.grondslag]);
    println!("{t}");

    kop("Hoe dit is berekend");
    println!("  {}", deadline.verantwoording);
    println!();
    terzijde(&deadline.verlengingsbepaling);

    if let ToegepasteVerlenging::NaarEerstvolgendeWerkdag { van, naar } = &deadline.verlenging {
        println!();
        let_op(&format!(
            "De laatste dag viel op {van} en is doorgeschoven naar {naar}. Let op: dit geldt \
             niet voor termijnen in uren — die lopen door weekend en feestdag heen."
        ));
    }

    let resterend = deadline.resterend(nu);
    println!();
    if deadline.is_verstreken(nu) {
        crate::uitvoer::blokkade(&format!(
            "deze termijn is {} geleden verstreken",
            duur(resterend)
        ));
    } else if resterend.num_hours() < 24 {
        let_op(&format!("nog {}", duur(resterend)));
    } else {
        terzijde(&format!("nog {}", duur(resterend)));
    }

    println!();
    terzijde(&format!(
        "uit kennispakket '{}' versie {}, geconsolideerd op {}. \
         Verifieer de duur en de grondslag tegen de bron voordat u hierop vertrouwt.",
        pakket.code,
        pakket.versienaam,
        pakket.consolidatiedatum.format("%d-%m-%Y")
    ));
    Ok(())
}
