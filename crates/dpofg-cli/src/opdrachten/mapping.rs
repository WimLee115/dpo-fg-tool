//! Veldmapping: het register naast de werkelijkheid leggen.
//!
//! De invoer is bewust saai: een bestand met veldnamen, één per regel of als
//! kopregel met puntkomma's. Geen bestandsformaat om te ontleden dat morgen
//! weer anders is.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use clap::Subcommand;
use dpofg_audit::Handeling;
use dpofg_domain::{mapping::Mappingprofiel, Verwerking, Volledig};
use dpofg_store::Kluis;
use std::path::PathBuf;

use crate::uitvoer::{blokkade, gelukt, kop, let_op, tabel, terzijde, voortgang};

const SOORT: &str = "mapping";
const COMPARTIMENT: &str = "algemeen";

#[derive(Subcommand, Debug)]
pub enum Mappingopdracht {
    /// Toon alle mappingprofielen met de stand van de laatste vergelijking.
    Lijst,
    /// Maak een profiel voor één bronsysteem bij één registerregel.
    Nieuw {
        /// Kenmerk van het profiel.
        kenmerk: String,
        /// Waar het over gaat.
        omschrijving: String,
        /// Het bronsysteem.
        #[arg(long)]
        bron: String,
        /// Het kenmerk van de registerregel.
        #[arg(long)]
        verwerking: String,
    },
    /// Toon één profiel.
    Toon {
        /// Kenmerk van het profiel.
        kenmerk: String,
    },
    /// Koppel een veld uit het bronsysteem aan een categorie uit het register.
    Koppel {
        /// Kenmerk van het profiel.
        kenmerk: String,
        /// De veldnaam zoals het systeem hem kent.
        #[arg(long)]
        bronveld: String,
        /// De categorie zoals die in de registerregel staat.
        #[arg(long)]
        categorie: String,
    },
    /// Laat een veld bewust buiten beschouwing, met de reden erbij.
    Negeer {
        /// Kenmerk van het profiel.
        kenmerk: String,
        /// De veldnaam.
        #[arg(long)]
        bronveld: String,
        /// Waarom dit veld niet meedoet.
        #[arg(long)]
        reden: String,
    },
    /// Vergelijk een lijst veldnamen met de registerregel.
    Vergelijk {
        /// Kenmerk van het profiel.
        kenmerk: String,
        /// Bestand met veldnamen: één per regel, of één kopregel met
        /// puntkomma's of komma's ertussen.
        bestand: PathBuf,
    },
}

pub fn draai(o: Mappingopdracht, kluispad: Option<PathBuf>, nu: DateTime<Utc>) -> Result<()> {
    let pad = super::kluispad(kluispad)?;
    let mut kluis = super::open_kluis(&pad, nu)?;
    match o {
        Mappingopdracht::Lijst => lijst(&kluis),
        Mappingopdracht::Nieuw { kenmerk, omschrijving, bron, verwerking } => {
            nieuw(&mut kluis, &kenmerk, &omschrijving, &bron, &verwerking, nu)
        }
        Mappingopdracht::Toon { kenmerk } => toon(&kluis, &kenmerk),
        Mappingopdracht::Koppel { kenmerk, bronveld, categorie } => {
            koppel(&mut kluis, &kenmerk, &bronveld, &categorie, nu)
        }
        Mappingopdracht::Negeer { kenmerk, bronveld, reden } => {
            negeer(&mut kluis, &kenmerk, &bronveld, &reden, nu)
        }
        Mappingopdracht::Vergelijk { kenmerk, bestand } => {
            vergelijk(&mut kluis, &kenmerk, &bestand, nu)
        }
    }
}

fn zoek(kluis: &Kluis, kenmerk: &str) -> Result<Mappingprofiel> {
    let kop = kluis
        .lijst(SOORT)?
        .into_iter()
        .find(|r| r.kenmerk.as_deref() == Some(kenmerk))
        .ok_or_else(|| anyhow::anyhow!("geen mappingprofiel met kenmerk '{kenmerk}'"))?;
    Ok(kluis.laad(SOORT, &kop.id)?)
}

fn bewaar(
    kluis: &mut Kluis,
    p: &Mappingprofiel,
    handeling: Handeling,
    omschrijving: &str,
    nu: DateTime<Utc>,
) -> Result<()> {
    let actor = super::actor();
    kluis.bewaar(
        SOORT,
        &p.id.to_string(),
        COMPARTIMENT,
        p.status.omschrijving(),
        Some(&p.kenmerk),
        p,
        &actor,
        handeling,
        omschrijving,
        nu,
    )?;
    Ok(())
}

/// Leest veldnamen: één per regel, of één kopregel met scheidingstekens.
fn lees_veldnamen(pad: &std::path::Path) -> Result<Vec<String>> {
    let tekst = std::fs::read_to_string(pad)
        .with_context(|| format!("kon {} niet lezen", pad.display()))?;
    let regels: Vec<&str> = tekst.lines().filter(|r| !r.trim().is_empty()).collect();
    if regels.is_empty() {
        anyhow::bail!("{} bevat geen veldnamen", pad.display());
    }
    // Eén regel met scheidingstekens is een kopregel; anders één naam per regel.
    if regels.len() == 1 && regels[0].contains([';', ',']) {
        return Ok(regels[0]
            .split([';', ','])
            .map(|v| v.trim().to_string())
            .filter(|v| !v.is_empty())
            .collect());
    }
    Ok(regels.iter().map(|r| r.trim().to_string()).collect())
}

fn nieuw(
    kluis: &mut Kluis,
    kenmerk: &str,
    omschrijving: &str,
    bron: &str,
    verwerkingkenmerk: &str,
    nu: DateTime<Utc>,
) -> Result<()> {
    if kluis.lijst(SOORT)?.iter().any(|r| r.kenmerk.as_deref() == Some(kenmerk)) {
        anyhow::bail!("er bestaat al een mappingprofiel met kenmerk '{kenmerk}'");
    }
    let kop_v = kluis
        .lijst("verwerking")?
        .into_iter()
        .find(|r| r.kenmerk.as_deref() == Some(verwerkingkenmerk))
        .ok_or_else(|| {
            anyhow::anyhow!(
                "geen registerregel met kenmerk '{verwerkingkenmerk}'. Bekijk de lijst met \
                 'dpofg register lijst'"
            )
        })?;
    let v: Verwerking = kluis.laad("verwerking", &kop_v.id)?;

    let p = Mappingprofiel::nieuw(
        kenmerk,
        omschrijving,
        bron,
        v.id,
        &v.kenmerk,
        &super::actor().id,
        nu,
    );
    bewaar(kluis, &p, Handeling::RecordAangemaakt, "mappingprofiel aangemaakt", nu)?;

    gelukt(&format!("mappingprofiel {kenmerk} aangemaakt voor {bron}"));
    if v.categorieen_gegevens.is_empty() {
        let_op(
            "De registerregel noemt nog geen categorieën gegevens. De vergelijking meldt dan \
             alles wat het systeem heeft als nieuw — wat klopt, maar weinig zegt.",
        );
    } else {
        kop("Categorieën in de registerregel");
        for c in &v.categorieen_gegevens {
            println!("  • {c}");
        }
    }
    toon_ontbrekend(&p);
    Ok(())
}

fn koppel(
    kluis: &mut Kluis,
    kenmerk: &str,
    bronveld: &str,
    categorie: &str,
    nu: DateTime<Utc>,
) -> Result<()> {
    let mut p = zoek(kluis, kenmerk)?;
    p.koppel(bronveld, categorie, nu)?;
    bewaar(kluis, &p, Handeling::RecordGewijzigd, &format!("{bronveld} gekoppeld"), nu)?;
    gelukt(&format!("'{bronveld}' hoort bij '{categorie}'"));
    toon_ontbrekend(&p);
    Ok(())
}

fn negeer(
    kluis: &mut Kluis,
    kenmerk: &str,
    bronveld: &str,
    reden: &str,
    nu: DateTime<Utc>,
) -> Result<()> {
    let mut p = zoek(kluis, kenmerk)?;
    p.negeer(bronveld, reden, nu)?;
    bewaar(
        kluis,
        &p,
        Handeling::ControleAfwijkingToegestaan,
        &format!("{bronveld} genegeerd"),
        nu,
    )?;
    gelukt(&format!("'{bronveld}' doet niet mee"));
    terzijde(reden);
    terzijde("De reden blijft staan en gaat mee in elke export; zo is later te zien wie wat heeft weggelaten en waarom.");
    toon_ontbrekend(&p);
    Ok(())
}

fn vergelijk(
    kluis: &mut Kluis,
    kenmerk: &str,
    bestand: &std::path::Path,
    nu: DateTime<Utc>,
) -> Result<()> {
    let mut p = zoek(kluis, kenmerk)?;
    let velden = lees_veldnamen(bestand)?;

    let kop_v = kluis
        .lijst("verwerking")?
        .into_iter()
        .find(|r| r.id == p.verwerking_id.to_string())
        .ok_or_else(|| {
            anyhow::anyhow!("de registerregel {} bestaat niet meer", p.verwerking_kenmerk)
        })?;
    let v: Verwerking = kluis.laad("verwerking", &kop_v.id)?;

    let rapport = p.vergelijk(&velden, &v.categorieen_gegevens, nu);
    p.leg_rapport_vast(rapport.clone(), nu);
    bewaar(
        kluis,
        &p,
        Handeling::KetenGeverifieerd,
        &format!("vergelijking: {} afwijking(en)", rapport.aantal_afwijkingen()),
        nu,
    )?;

    kop(&format!("Vergelijking van {} met {}", p.bron, p.verwerking_kenmerk));
    terzijde(&format!("{} veldnaam/namen gelezen uit {}", velden.len(), bestand.display()));

    if !rapport.nieuw_in_bron.is_empty() {
        kop("Staat in het systeem, niet in het register");
        for veld in &rapport.nieuw_in_bron {
            blokkade(veld);
        }
        terzijde(
            "Een veld dat niemand heeft aangewezen, is een verwerking die niet is vastgelegd. \
             Koppel het aan een categorie, of leg met 'mapping negeer' vast waarom het niet \
             meedoet.",
        );
    }
    if !rapport.ontbreekt_in_bron.is_empty() {
        kop("Staat in het register, niet in het systeem");
        for categorie in &rapport.ontbreekt_in_bron {
            let_op(categorie);
        }
        terzijde(
            "Een register dat te veel noemt is even onbetrouwbaar als een register dat te weinig \
             noemt.",
        );
    }
    if !rapport.heeft_afwijkingen() {
        println!();
        gelukt(&format!("{} veld(en) bevestigd, geen afwijkingen", rapport.bevestigd.len()));
    }
    if !rapport.genegeerd.is_empty() {
        terzijde(&format!(
            "{} veld(en) bewust buiten beschouwing gelaten",
            rapport.genegeerd.len()
        ));
    }

    toon_ontbrekend(&p);
    Ok(())
}

fn lijst(kluis: &Kluis) -> Result<()> {
    let koppen = kluis.lijst(SOORT)?;
    if koppen.is_empty() {
        kop("Mappingprofielen");
        terzijde("Er staan nog geen mappingprofielen in de kluis.");
        return Ok(());
    }
    kop("Mappingprofielen");
    let mut t = tabel(&["kenmerk", "bron", "registerregel", "koppelingen", "laatste uitkomst"]);
    for k in &koppen {
        let p: Mappingprofiel = kluis.laad(SOORT, &k.id)?;
        let uitkomst = match &p.laatste_rapport {
            None => "nog niet vergeleken".to_string(),
            Some(r) if r.heeft_afwijkingen() => format!("{} afwijking(en)", r.aantal_afwijkingen()),
            Some(_) => "geen afwijkingen".to_string(),
        };
        t.add_row(vec![
            p.kenmerk.clone(),
            p.bron.clone(),
            p.verwerking_kenmerk.clone(),
            p.koppelingen.len().to_string(),
            uitkomst,
        ]);
    }
    println!("{t}");
    Ok(())
}

fn toon(kluis: &Kluis, kenmerk: &str) -> Result<()> {
    let p = zoek(kluis, kenmerk)?;

    kop(&format!("Mappingprofiel {}", p.kenmerk));
    let mut t = tabel(&["", ""]);
    t.add_row(vec!["omschrijving", &p.omschrijving]);
    t.add_row(vec!["bron", &p.bron]);
    t.add_row(vec!["registerregel", &p.verwerking_kenmerk]);
    println!("{t}");

    if !p.koppelingen.is_empty() {
        kop("Koppelingen");
        let mut t = tabel(&["bronveld", "categorie"]);
        for k in &p.koppelingen {
            t.add_row(vec![k.bronveld.clone(), k.categorie.clone()]);
        }
        println!("{t}");
    }
    if !p.genegeerd.is_empty() {
        kop("Bewust buiten beschouwing");
        let mut t = tabel(&["bronveld", "reden"]);
        for (veld, reden) in &p.genegeerd {
            t.add_row(vec![veld.clone(), reden.clone()]);
        }
        println!("{t}");
    }
    if let Some(r) = &p.laatste_rapport {
        kop("Laatste vergelijking");
        terzijde(&format!("uitgevoerd op {}", crate::uitvoer::tijdstip(r.uitgevoerd_op)));
        let mut t = tabel(&["", ""]);
        t.add_row(vec!["bevestigd", &r.bevestigd.len().to_string()]);
        t.add_row(vec!["nieuw in de bron", &r.nieuw_in_bron.len().to_string()]);
        t.add_row(vec!["ontbreekt in de bron", &r.ontbreekt_in_bron.len().to_string()]);
        println!("{t}");
    }

    toon_ontbrekend(&p);
    Ok(())
}

fn toon_ontbrekend(p: &Mappingprofiel) {
    let r = p.volledigheid();
    kop("Volledigheid");
    println!("  {}", voortgang(r.compleet, r.verplicht));
    if r.is_volledig() {
        println!();
        gelukt("het profiel is bij");
        return;
    }
    println!();
    for o in &r.ontbreekt {
        let veld = o.veld.trim_start_matches("mapping.");
        if o.blokkeert_vaststelling {
            blokkade(&format!("{veld} — {}", o.omschrijving));
        } else {
            let_op(&format!("{veld} — {}", o.omschrijving));
        }
        terzijde(&o.grondslag);
    }
}
