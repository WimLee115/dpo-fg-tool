//! Beheer van het kluisbestand.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use clap::Subcommand;
use dpofg_crypto::kdf::KdfParameters;
use dpofg_store::Kluis;
use std::path::PathBuf;

use crate::uitvoer::{gelukt, kop, let_op, tabel, terzijde};

#[derive(Subcommand, Debug)]
pub enum Kluisopdracht {
    /// Maak een nieuwe kluis aan.
    Nieuw {
        /// Zwaardere sleutelafleiding. Kost meer tijd bij het openen en maakt
        /// een offline raadaanval navenant duurder.
        #[arg(long)]
        zwaar: bool,
        /// Lichtere sleutelafleiding voor machines met weinig geheugen.
        /// De keuze wordt in het logboek vastgelegd.
        #[arg(long, conflicts_with = "zwaar")]
        licht: bool,
    },
    /// Toon de stand van de kluis.
    Status,
    /// Wijzig de wachtwoordzin. Er wordt niets herversleuteld.
    Wachtwoord,
    /// Maak een compartiment aan.
    Compartiment {
        /// Naam van het compartiment.
        naam: String,
        /// Waarvoor het compartiment bedoeld is.
        #[arg(long, default_value = "")]
        omschrijving: String,
    },
}

pub fn draai(o: Kluisopdracht, kluispad: Option<PathBuf>, nu: DateTime<Utc>) -> Result<()> {
    let pad = super::kluispad(kluispad)?;
    match o {
        Kluisopdracht::Nieuw { zwaar, licht } => nieuw(&pad, zwaar, licht, nu),
        Kluisopdracht::Status => status(&pad, nu),
        Kluisopdracht::Wachtwoord => wachtwoord(&pad, nu),
        Kluisopdracht::Compartiment { naam, omschrijving } => {
            compartiment(&pad, &naam, &omschrijving, nu)
        }
    }
}

fn nieuw(pad: &std::path::Path, zwaar: bool, licht: bool, nu: DateTime<Utc>) -> Result<()> {
    if pad.exists() {
        anyhow::bail!(
            "er staat al een kluis op {}. Verwijderen doet u zelf, en pas nadat u zeker weet \
             dat er een werkende reservekopie is",
            pad.display()
        );
    }
    if let Some(map) = pad.parent() {
        std::fs::create_dir_all(map)
            .with_context(|| format!("kon {} niet aanmaken", map.display()))?;
    }

    let params = match (zwaar, licht) {
        (true, _) => KdfParameters::ZWAAR,
        (_, true) => KdfParameters::LICHT,
        _ => KdfParameters::STANDAARD,
    };

    kop("Nieuwe kluis aanmaken");
    println!("Locatie: {}", pad.display());
    println!(
        "Sleutelafleiding: {} MiB geheugen, {} iteraties",
        params.geheugen_kib / 1024,
        params.iteraties
    );
    if licht {
        let_op(
            "Het lichte profiel biedt minder weerstand tegen een offline raadaanval. \
             Deze keuze wordt in het logboek vastgelegd.",
        );
    }

    let wachtwoord = crate::wachtwoord::vraag_nieuw()?;

    println!("\nSleutel afleiden…");
    let mut kluis = Kluis::aanmaken(pad, &wachtwoord, params, nu)?;
    kluis.compartiment_aanmaken(
        "vertrouwelijk",
        "incidenten, verzoeken en andere gevoelige dossiers",
        nu,
    )?;

    gelukt(&format!("kluis aangemaakt op {}", pad.display()));
    terzijde("Er zijn twee compartimenten: 'algemeen' en 'vertrouwelijk'.");
    println!();
    let_op(
        "Maak nu een reservekopie van dit bestand en bewaar die op een andere plaats. \
         Zonder reservekopie is een defecte schijf het einde van het dossier.",
    );
    let_op(
        "Er is geen herstelmogelijkheid voor de wachtwoordzin. Bewaar hem op een plaats \
         waar u er over een jaar nog bij kunt.",
    );
    Ok(())
}

fn status(pad: &std::path::Path, nu: DateTime<Utc>) -> Result<()> {
    let kluis = super::open_kluis(pad, nu)?;

    kop("Kluis");
    let mut t = tabel(&["", ""]);
    t.add_row(vec!["locatie", &kluis.pad().display().to_string()]);
    t.add_row(vec!["schemaversie", &dpofg_store::SCHEMAVERSIE.to_string()]);
    t.add_row(vec!["compartimenten", &kluis.compartimenten().join(", ")]);
    t.add_row(vec!["logboekregels", &kluis.ketenstand().volgnummer.to_string()]);
    if let Some(anker) = kluis.laatste_anker()? {
        t.add_row(vec![
            "laatste anker",
            &format!(
                "regel {} op {}",
                anker.volgnummer,
                anker.tijdstip.format("%d-%m-%Y %H:%M")
            ),
        ]);
    } else {
        t.add_row(vec!["laatste anker", "geen"]);
    }
    println!("{t}");

    kop("Inhoud");
    let mut t = tabel(&["soort", "aantal", "vastgesteld", "concept"]);
    for soort in ["verwerking", "incident", "verzoek"] {
        let lijst = kluis.lijst(soort)?;
        if lijst.is_empty() {
            continue;
        }
        let vastgesteld = lijst.iter().filter(|r| r.status == "vastgesteld").count();
        let concept = lijst.iter().filter(|r| r.status == "concept").count();
        t.add_row(vec![
            soort.to_string(),
            lijst.len().to_string(),
            vastgesteld.to_string(),
            concept.to_string(),
        ]);
    }
    if t.row_iter().count() == 0 {
        terzijde("De kluis is nog leeg.");
    } else {
        println!("{t}");
    }

    if kluis.laatste_anker()?.is_none() && kluis.ketenstand().volgnummer > 0 {
        println!();
        let_op(
            "Er is nog geen anker geplaatst. Zonder anker is niet vast te stellen of er aan het \
             einde van het logboek regels zijn verwijderd. Plaats er een met 'dpofg logboek anker'.",
        );
    }
    Ok(())
}

fn wachtwoord(pad: &std::path::Path, nu: DateTime<Utc>) -> Result<()> {
    let mut kluis = super::open_kluis(pad, nu)?;
    kop("Wachtwoordzin wijzigen");
    terzijde(
        "Alleen de wikkeling van de kluissleutel wordt vervangen. Geen enkele byte aan gegevens \
         wordt herversleuteld, dus dit kan niet halverwege stuklopen.",
    );
    let nieuw = crate::wachtwoord::vraag_nieuw()?;
    kluis.wachtwoord_wijzigen(&nieuw, KdfParameters::STANDAARD, &super::actor(), nu)?;
    gelukt("wachtwoordzin gewijzigd");
    Ok(())
}

fn compartiment(
    pad: &std::path::Path,
    naam: &str,
    omschrijving: &str,
    nu: DateTime<Utc>,
) -> Result<()> {
    let mut kluis = super::open_kluis(pad, nu)?;
    if kluis.compartimenten().contains(&naam) {
        anyhow::bail!("de kluis heeft al een compartiment '{naam}'");
    }
    kluis.compartiment_aanmaken(naam, omschrijving, nu)?;
    gelukt(&format!("compartiment '{naam}' aangemaakt"));
    terzijde(
        "De inhoud van dit compartiment krijgt een eigen sleutel. Wie die sleutel niet heeft, \
         ziet onleesbare bytes — ook bij een fout in de toegangscontrole.",
    );
    Ok(())
}
