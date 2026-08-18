//! Het kennispakket met de juridische inhoud.

use anyhow::Result;
use chrono::{DateTime, Utc};
use clap::Subcommand;
use dpofg_content::startpakket;
use std::path::PathBuf;

use crate::uitvoer::{kop, let_op, tabel, terzijde};

#[derive(Subcommand, Debug)]
pub enum Pakketopdracht {
    /// Toon het actieve kennispakket.
    Toon,
    /// Toon waarvoor het pakket een voorbehoud maakt.
    Voorbehoud,
}

pub fn draai(o: Pakketopdracht, _kluispad: Option<PathBuf>, nu: DateTime<Utc>) -> Result<()> {
    let pakket = startpakket(nu.date_naive());
    match o {
        Pakketopdracht::Toon => {
            kop("Kennispakket");
            let mut t = tabel(&["", ""]);
            t.add_row(vec!["code", &pakket.code]);
            t.add_row(vec!["naam", &pakket.naam]);
            t.add_row(vec!["versie", &pakket.versienaam]);
            t.add_row(vec![
                "geconsolideerd op",
                &pakket.consolidatiedatum.format("%d-%m-%Y").to_string(),
            ]);
            t.add_row(vec!["rechtsgebied", &pakket.jurisdictie]);
            t.add_row(vec!["termijnen", &pakket.termijnen.len().to_string()]);
            t.add_row(vec!["rechtsfeiten", &pakket.rechtsfeiten.len().to_string()]);
            t.add_row(vec![
                "doorgifte-instrumenten",
                &pakket.doorgifteinstrumenten.len().to_string(),
            ]);
            println!("{t}");

            let ouderdom = pakket.ouderdom_in_dagen(nu);
            if ouderdom > 180 {
                println!();
                let_op(&format!(
                    "De inhoud is {ouderdom} dagen geleden bijgewerkt. Controleer of er sindsdien \
                     wetgeving is gewijzigd."
                ));
            }

            let herbeoordeling = pakket.instrumenten_met_herbeoordeling();
            if !herbeoordeling.is_empty() {
                kop("Instrumenten die om herbeoordeling vragen");
                for i in herbeoordeling {
                    let_op(&format!("{} ({}) — {}", i.code, i.land_of_gebied, i.toelichting));
                }
            }

            println!();
            terzijde(
                "De consolidatiedatum gaat mee in elke export en elk dossier, zodat zichtbaar is \
                 op welke stand van het recht een berekening berust.",
            );
            Ok(())
        }
        Pakketopdracht::Voorbehoud => {
            kop("Voorbehoud bij dit kennispakket");
            let waarschuwing = pakket
                .aanvullend
                .get("waarschuwing")
                .ok_or_else(|| anyhow::anyhow!("dit pakket bevat geen voorbehoud"))?;

            println!("  {}", waarschuwing["strekking"].as_str().unwrap_or(""));
            kop("Te verifiëren vóór gebruik");
            if let Some(lijst) = waarschuwing["te_verifieren"].as_array() {
                for item in lijst {
                    println!("  • {}", item.as_str().unwrap_or(""));
                }
            }
            println!();
            terzijde(
                "Deze tekst staat er niet uit voorzichtigheid maar omdat de inhoud werkelijk \
                 niet is vastgesteld door een jurist. Een product dat juridische zekerheid \
                 suggereert die het niet heeft, is bij een inspectie erger dan geen product.",
            );
            Ok(())
        }
    }
}
