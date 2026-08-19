//! Dossiers samenstellen voor een toezichthouder of auditor.
//!
//! Wat hier de deur uitgaat, wordt gecontroleerd met `dpofg-verify` — een
//! losse binary die de kluis niet nodig heeft en geen wachtwoord vraagt. Dat
//! is het hele punt: een bewering dat een dossier niet is gewijzigd, is
//! waardeloos als zij alleen te controleren is met software van degene die de
//! bewering doet.

use anyhow::Result;
use chrono::{DateTime, Utc};
use clap::Args;
use dpofg_audit::Handeling;
use dpofg_report::{Manifest, OndertekendManifest};
use std::path::PathBuf;

use crate::uitvoer::{gelukt, kop, let_op, tabel, terzijde};

#[derive(Args, Debug)]
pub struct Dossieropties {
    /// De map waarin het dossier wordt weggeschreven.
    pub map: PathBuf,
    /// Waarvoor het dossier wordt samengesteld.
    #[arg(long)]
    pub aanleiding: String,
    /// Voor wie het bestemd is.
    #[arg(long)]
    pub bestemd_voor: String,
    /// Welke soorten records worden opgenomen. Standaard: alle.
    #[arg(long)]
    pub soort: Vec<String>,
    /// Neem ook conceptrecords op.
    ///
    /// Standaard blijven concepten erbuiten, maar hun aantal wordt wél in het
    /// manifest vermeld: verzwijgen wat er ontbreekt is de snelste manier om
    /// vertrouwen in een dossier te verliezen.
    #[arg(long)]
    pub met_concepten: bool,
}

pub fn draai(o: Dossieropties, kluispad: Option<PathBuf>, nu: DateTime<Utc>) -> Result<()> {
    let pad = super::kluispad(kluispad)?;
    let mut kluis = super::open_kluis(&pad, nu)?;

    std::fs::create_dir_all(&o.map)?;

    let rapport = kluis.verifieer_logboek()?;
    let pakket = dpofg_content::startpakket(nu.date_naive());

    let mut manifest = Manifest::nieuw(
        &o.aanleiding,
        &o.bestemd_voor,
        &super::actor().naam,
        nu,
        kluis.ketenstand().volgnummer,
        &kluis.ketenstand().hash,
        rapport.reikwijdte(),
        &pakket.code,
        &pakket.versienaam,
        pakket.consolidatiedatum,
    );
    if let Some(anker) = kluis.laatste_anker()? {
        manifest.anker_omschrijving = Some(format!(
            "regel {} op {}, bewaard in: {}",
            anker.volgnummer,
            crate::uitvoer::tijdstip(anker.tijdstip),
            anker.bewaarplaats.as_deref().unwrap_or("niet vermeld")
        ));
    }

    let soorten: Vec<String> = if o.soort.is_empty() {
        vec![
            "verwerking".into(),
            "dpia".into(),
            "incident".into(),
            "verzoek".into(),
            "woo".into(),
            "wpg".into(),
            "lia".into(),
            "doorgifte".into(),
            "leverancier".into(),
            "mapping".into(),
            "redactie".into(),
            "zorgplicht".into(),
            "risico".into(),
            "correctie".into(),
            "spiegel".into(),
        ]
    } else {
        o.soort.clone()
    };

    let mut opgenomen = 0usize;
    for soort in &soorten {
        let mut overgeslagen = 0usize;
        for k in kluis.lijst(soort)? {
            if k.status == "concept" && !o.met_concepten {
                overgeslagen += 1;
                continue;
            }
            // Het record wordt opgenomen zoals het is opgeslagen, ontsleuteld.
            let waarde: serde_json::Value = kluis.laad(soort, &k.id)?;
            let inhoud = serde_json::to_vec_pretty(&waarde)?;
            let naam =
                format!("{soort}-{}.json", k.kenmerk.clone().unwrap_or_else(|| k.id.clone()));
            std::fs::write(o.map.join(&naam), &inhoud)?;
            manifest.voeg_toe(&naam, soort, &k.id, k.versie, &inhoud);
            opgenomen += 1;
        }
        if overgeslagen > 0 {
            manifest.laat_weg(
                format!("{} met de status concept", meervoud(soort)),
                "concepten zijn nog niet vastgesteld en geven geen beeld van de werkelijkheid; \
                 hun aantal staat hier zodat zichtbaar is dat zij bestaan",
                overgeslagen,
            );
        }
    }

    // Het logboek gaat mee, want zonder logboek is de ketenstand in het
    // manifest niet na te rekenen.
    let logboek = serde_json::to_vec_pretty(&kluis.logboek()?)?;
    std::fs::write(o.map.join("logboek.json"), &logboek)?;
    manifest.voeg_toe("logboek.json", "logboek", "logboek", 1, &logboek);

    if let Some(anker) = kluis.laatste_anker()? {
        let ankerjson = serde_json::to_vec_pretty(&anker)?;
        std::fs::write(o.map.join("anker.json"), &ankerjson)?;
        manifest.voeg_toe("anker.json", "anker", "anker", 1, &ankerjson);
    }

    // Ondertekenen met de vaste installatiesleutel: dezelfde die onder elk
    // anker van deze kluis staat.
    let ondertekend = kluis.onderteken_met(|s| OndertekendManifest::onderteken(manifest, s))?;
    let manifestpad = o.map.join("manifest.json");
    std::fs::write(&manifestpad, serde_json::to_vec_pretty(&ondertekend)?)?;

    kluis.log(
        dpofg_audit::Gebeurtenis::nieuw(
            Handeling::DossierSamengesteld,
            super::actor(),
            nu,
            "dossier",
            o.map.display().to_string(),
            "algemeen",
            format!("dossier samengesteld voor {}", o.bestemd_voor),
        ),
        Some(o.aanleiding.clone()),
    )?;

    kop("Dossier samengesteld");
    let mut t = tabel(&["", ""]);
    t.add_row(vec!["map", &o.map.display().to_string()]);
    t.add_row(vec!["aanleiding", &o.aanleiding]);
    t.add_row(vec!["bestemd voor", &o.bestemd_voor]);
    t.add_row(vec!["stukken", &ondertekend.manifest.stukken.len().to_string()]);
    t.add_row(vec!["records", &opgenomen.to_string()]);
    t.add_row(vec!["ketenstand", &format!("regel {}", ondertekend.manifest.keten_volgnummer)]);
    println!("{t}");

    if !ondertekend.manifest.weggelaten.is_empty() {
        kop("Bewust weggelaten");
        for w in &ondertekend.manifest.weggelaten {
            println!("  • {} ({}×)", w.omschrijving, w.aantal);
            terzijde(&w.reden);
        }
    }

    kop("Reikwijdte die in het dossier staat");
    println!("  {}", ondertekend.manifest.reikwijdte);

    println!();
    gelukt(&format!("manifest weggeschreven naar {}", manifestpad.display()));
    println!();
    terzijde(&format!(
        "De ontvanger controleert dit met:\n  dpofg-verify dossier {} --sleutel {}",
        manifestpad.display(),
        ondertekend.ondertekenaar
    ));
    let_op(
        "Geef de sleutel langs een ánder kanaal door dan dit dossier. Het dossier bevat hem niet \
         als losstaande vermelding en hoort dat ook niet te doen: een sleutel die met het stuk \
         meekomt, toont niets aan. Publiceer hem op de website van de organisatie of in een \
         eerder ondertekend stuk; 'dpofg kluis sleutel' toont hem.",
    );
    Ok(())
}

/// Het meervoud van een recordsoort, voor gebruik in een zin.
///
/// "dpiaen" is geen woord; een tekst die in een dossier voor een toezichthouder
/// terechtkomt, hoort te lezen als Nederlands.
fn meervoud(soort: &str) -> &str {
    match soort {
        "verwerking" => "verwerkingen",
        "dpia" => "effectbeoordelingen",
        "incident" => "incidenten",
        "verzoek" => "verzoeken",
        andere => andere,
    }
}
