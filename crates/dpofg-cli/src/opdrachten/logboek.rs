//! Het ketenlogboek.

use anyhow::Result;
use chrono::{DateTime, Utc};
use clap::Subcommand;
use dpofg_audit::{Ankerstatus, Bevindingsoort};
use std::path::PathBuf;

use crate::uitvoer::{blokkade, gelukt, kop, let_op, tabel, terzijde};

#[derive(Subcommand, Debug)]
pub enum Logboekopdracht {
    /// Toon de laatste regels van het logboek.
    Toon {
        /// Hoeveel regels.
        #[arg(long, default_value = "25")]
        aantal: usize,
        /// Alleen de regels over dit onderwerp.
        #[arg(long)]
        onderwerp: Option<String>,
    },
    /// Controleer de keten op wijziging, verwijdering en afkapping.
    Verifieer,
    /// Plaats een anker op de huidige ketenstand.
    Anker {
        /// Waar het anker buiten het systeem wordt bewaard.
        #[arg(long)]
        bewaarplaats: Option<String>,
        /// Bestand waarin het anker wordt weggeschreven.
        #[arg(long)]
        uitvoer: Option<PathBuf>,
    },
}

pub fn draai(o: Logboekopdracht, kluispad: Option<PathBuf>, nu: DateTime<Utc>) -> Result<()> {
    let pad = super::kluispad(kluispad)?;
    let mut kluis = super::open_kluis(&pad, nu)?;
    match o {
        Logboekopdracht::Toon { aantal, onderwerp } => toon(&kluis, aantal, onderwerp),
        Logboekopdracht::Verifieer => verifieer(&kluis),
        Logboekopdracht::Anker { bewaarplaats, uitvoer } => {
            anker(&mut kluis, bewaarplaats, uitvoer, nu)
        }
    }
}

fn toon(kluis: &dpofg_store::Kluis, aantal: usize, onderwerp: Option<String>) -> Result<()> {
    let regels = match &onderwerp {
        Some(o) => {
            let (soort, id) = o.split_once(':').unwrap_or(("verwerking", o));
            kluis.logboek_van(soort, id)?
        }
        None => kluis.logboek()?,
    };

    kop("Logboek");
    let mut t = tabel(&["nr", "tijdstip", "handeling", "wie", "onderwerp", "omschrijving"]);
    for r in regels.iter().rev().take(aantal).rev() {
        t.add_row(vec![
            r.volgnummer.to_string(),
            r.gebeurtenis.tijdstip.format("%d-%m %H:%M").to_string(),
            format!("{:?}", r.gebeurtenis.handeling),
            r.gebeurtenis.actor.naam.clone(),
            format!(
                "{}/{}",
                r.gebeurtenis.onderwerp_soort,
                &r.gebeurtenis.onderwerp_id.chars().take(8).collect::<String>()
            ),
            r.gebeurtenis.omschrijving.clone(),
        ]);
    }
    println!("{t}");
    terzijde(&format!("{} regels in totaal", regels.len()));
    Ok(())
}

fn verifieer(kluis: &dpofg_store::Kluis) -> Result<()> {
    let rapport = kluis.verifieer_logboek()?;

    kop("Verificatie van het logboek");
    let mut t = tabel(&["", ""]);
    t.add_row(vec!["regels", &rapport.regels.to_string()]);
    if let Some((van, tot)) = rapport.periode {
        t.add_row(vec![
            "periode",
            &format!("{} tot {}", van.format("%d-%m-%Y %H:%M"), tot.format("%d-%m-%Y %H:%M")),
        ]);
    }
    if let Some(h) = &rapport.laatste_hash {
        t.add_row(vec!["laatste hash", &h[..16]]);
    }
    println!("{t}");

    if rapport.bevindingen.is_empty() {
        println!();
        gelukt("de keten is intern samenhangend: geen wijzigingen, geen ontbrekende regels");
    } else {
        kop("Bevindingen");
        for b in &rapport.bevindingen {
            let soort = match b.soort {
                Bevindingsoort::Ketenbreuk => "ketenbreuk",
                Bevindingsoort::OntbrekendeRegel => "ontbrekende regel",
                Bevindingsoort::DubbeleRegel => "dubbele regel",
                Bevindingsoort::InhoudGewijzigd => "inhoud gewijzigd",
                Bevindingsoort::TijdLooptTerug => "tijd loopt terug",
            };
            blokkade(&format!("regel {} — {soort}: {}", b.volgnummer, b.omschrijving));
        }
    }

    kop("Reikwijdte");
    // Deze zin hoort onder elk verificatierapport en mag niet worden afgezwakt.
    println!("  {}", rapport.reikwijdte());

    match &rapport.ankerstatus {
        Ankerstatus::GeenAnker => {
            println!();
            let_op(
                "Plaats een anker met 'dpofg logboek anker' en bewaar het buiten dit systeem. \
                 Zonder anker blijft afkappen van het logboek onzichtbaar.",
            );
        }
        Ankerstatus::KetenIsIngekort { .. } | Ankerstatus::HashWijktAf { .. } => {
            println!();
            blokkade("Het logboek wijkt af van wat het anker verklaart. Onderzoek dit.");
        }
        _ => {}
    }

    // Een aanwijzing, geen oordeel: dit raakt de samenhang van de keten niet.
    if let Some(anker) = kluis.laatste_anker()? {
        if !anker.sleutel.eq_ignore_ascii_case(kluis.installatiesleutel()) {
            println!();
            let_op(
                "Het laatste anker is met een andere sleutel ondertekend dan de huidige \
                 installatiesleutel. Ankers van vóór deze uitgave dragen een wegwerpsleutel en \
                 zijn daarom niet aan deze installatie toe te schrijven. Plaats een nieuw anker \
                 met 'dpofg logboek anker'.",
            );
        }
    }

    if !rapport.is_ongeschonden() {
        anyhow::bail!("het logboek is niet ongeschonden");
    }
    Ok(())
}

fn anker(
    kluis: &mut dpofg_store::Kluis,
    bewaarplaats: Option<String>,
    uitvoer: Option<PathBuf>,
    nu: DateTime<Utc>,
) -> Result<()> {
    if kluis.ketenstand().is_leeg() {
        anyhow::bail!("het logboek is leeg; er valt nog niets te ankeren");
    }

    // De stand eerst kopiëren: `onderteken_met` leent de kluis uit, en dan kan
    // de ketenstand er niet tegelijk uit worden gelezen.
    let stand = kluis.ketenstand().clone();
    let mut a = kluis.onderteken_met(|s| dpofg_audit::Anker::plaats(s, "kluis", &stand, nu))?;
    if let Some(p) = bewaarplaats {
        a = a.met_bewaarplaats(p);
    }
    kluis.anker_bewaren(&a)?;

    let json = serde_json::to_string_pretty(&a)?;
    kop("Anker geplaatst");
    let mut t = tabel(&["", ""]);
    t.add_row(vec!["regel", &a.volgnummer.to_string()]);
    t.add_row(vec!["hash", &a.hash[..16]]);
    t.add_row(vec!["tijdstip", &a.tijdstip.format("%d-%m-%Y %H:%M UTC").to_string()]);
    t.add_row(vec!["sleutel", &format!("{}…", &a.sleutel[..16])]);
    if let Some(p) = &a.bewaarplaats {
        t.add_row(vec!["bewaarplaats", p]);
    }
    println!("{t}");

    match uitvoer {
        Some(p) => {
            std::fs::write(&p, &json)?;
            gelukt(&format!("anker weggeschreven naar {}", p.display()));
        }
        None => {
            println!("\n{json}");
        }
    }

    println!();
    let_op(
        "Bewaar dit anker buiten dit systeem: in de notulen van een overleg, bij de accountant, \
         of op een andere machine. Een anker dat alleen in de kluis staat, verdwijnt samen met \
         de regels die het zou moeten beschermen.",
    );
    terzijde(
        "Dit anker is ondertekend met de vaste installatiesleutel van deze kluis; elk anker en \
         elk dossier draagt dezelfde. De ontvanger stelt de herkomst vast door die sleutel te \
         vergelijken met de sleutel die de organisatie langs een ander kanaal heeft gepubliceerd \
         — zonder die vergelijking is de herkomst nog steeds niet vastgesteld. Toon hem met \
         'dpofg kluis sleutel'.",
    );
    Ok(())
}
