//! Het persoonlijke dossier van de functionaris.
//!
//! Een tweede kluisbestand met een eigen wachtwoordzin. De organisatie kan de
//! inhoud niet lezen; in haar kluis blijft alleen een hash achter waarmee
//! later is aan te tonen dát een record op een bepaald moment bestond.

use anyhow::Result;
use chrono::{DateTime, Utc};
use clap::Subcommand;
use dpofg_audit::Handeling;
use dpofg_domain::{
    fg::{
        spiegelstand, Aantastingsoort, Advies, Onafhankelijkheidsincident, Reactiestatus,
        Spiegeling, Spiegelstand, Tijdigheid,
    },
    Motivering,
};
use dpofg_store::{spiegelhash, Kluis, Spiegelregel, SPIEGELSOORT};
use std::path::PathBuf;

use crate::uitvoer::{blokkade, gelukt, kop, let_op, tabel, terzijde};

const COMPARTIMENT: &str = "fg-persoonlijk";

/// Het persoonlijke dossier heeft een eigen pad.
///
/// Uitdrukkelijk niet `--kluis`: dat wijst de kluis van de organisatie aan, en
/// die twee door elkaar halen is precies de vergissing die dit dossier
/// zinloos maakt. De opdrachten die beide kluizen nodig hebben, noemen ze
/// daarom apart.
#[derive(clap::Args, Debug)]
pub struct Fgargumenten {
    /// Waar uw persoonlijke dossier staat.
    #[arg(long, global = true)]
    pub dossier: Option<PathBuf>,
    #[command(subcommand)]
    pub opdracht: Fgopdracht,
}

#[derive(Subcommand, Debug)]
pub enum Fgopdracht {
    /// Maak het persoonlijke dossier aan, met een eigen wachtwoordzin.
    Nieuw {
        /// Waar het dossier komt te staan.
        pad: PathBuf,
        /// Sla de controle over of de zin ook de organisatiekluis opent.
        ///
        /// Alleen bedoeld voor het geval er geen organisatiekluis is.
        #[arg(long)]
        zonder_organisatiekluis: bool,
    },
    /// Toon wat er in het persoonlijke dossier staat.
    Lijst,
    /// Leg een uitgebracht advies vast.
    Advies {
        /// Kenmerk van het advies.
        kenmerk: String,
        /// Waarover het ging.
        #[arg(long)]
        onderwerp: String,
        /// Wie de vraag stelde.
        #[arg(long)]
        vraagsteller: String,
        /// Wat er is geadviseerd.
        #[arg(long)]
        advies: String,
        /// Aan wie het is uitgebracht.
        #[arg(long)]
        aan: String,
        /// Wanneer. Standaard: nu.
        #[arg(long)]
        op: Option<String>,
        /// Of u naar behoren en tijdig bent betrokken.
        #[arg(long, value_enum, default_value = "ja")]
        tijdig: Tijdigheidkeuze,
        /// Waaruit blijkt dat u niet tijdig bent betrokken.
        #[arg(long)]
        toelichting: Option<String>,
    },
    /// Leg vast wat het bestuur met een advies heeft gedaan.
    Reactie {
        /// Kenmerk van het advies.
        kenmerk: String,
        /// De uitkomst.
        #[arg(long, value_enum)]
        status: Reactiekeuze,
        /// Wie het besluit nam.
        #[arg(long)]
        beslisser: String,
        /// Wanneer. Standaard: nu.
        #[arg(long)]
        datum: Option<String>,
        /// De reden, verplicht bij niet of deels overnemen.
        #[arg(long)]
        motivering: Option<String>,
    },
    /// Leg een escalatiestap vast.
    Escaleren {
        /// Kenmerk van het advies.
        kenmerk: String,
        /// Naar wie is opgeschaald.
        #[arg(long)]
        niveau: String,
        /// Wanneer. Standaard: nu.
        #[arg(long)]
        datum: Option<String>,
        /// Wat het heeft opgeleverd.
        #[arg(long)]
        uitkomst: Option<String>,
    },
    /// Leg een gebeurtenis vast die uw onafhankelijkheid raakt.
    Onafhankelijkheid {
        /// Kenmerk van de gebeurtenis.
        kenmerk: String,
        /// Wat er is gebeurd.
        #[arg(long, value_enum)]
        soort: Soortkeuze,
        /// Wanneer. Standaard: nu.
        #[arg(long)]
        datum: Option<String>,
        /// Wat er feitelijk is gebeurd.
        #[arg(long)]
        omschrijving: String,
        /// Van wie het kwam.
        #[arg(long)]
        van: String,
        /// Wie het betrof. Standaard: uzelf.
        #[arg(long)]
        betrof: Option<String>,
    },
    /// Toon de zes gronden waarop de onafhankelijkheid rust.
    Gronden,
    /// Toon één record.
    Toon {
        /// Kenmerk van het advies of de gebeurtenis.
        kenmerk: String,
    },
    /// Leg de hash van een record vast in de kluis van de organisatie.
    ///
    /// De kluis van de organisatie is die van `--kluis`; het persoonlijke
    /// dossier dat van `--dossier`. Die twee hebben elk hun eigen
    /// wachtwoordzin en worden hier allebei geopend.
    Spiegelen {
        /// Kenmerk van het advies of de gebeurtenis.
        kenmerk: String,
    },
    /// Toon aan dat een record op een bepaald moment al bestond.
    Aantonen {
        /// Kenmerk van het advies of de gebeurtenis.
        kenmerk: String,
    },
}

#[derive(clap::ValueEnum, Clone, Copy, Debug)]
pub enum Tijdigheidkeuze {
    Ja,
    Deels,
    Nee,
}

impl From<Tijdigheidkeuze> for Tijdigheid {
    fn from(k: Tijdigheidkeuze) -> Self {
        match k {
            Tijdigheidkeuze::Ja => Tijdigheid::Ja,
            Tijdigheidkeuze::Deels => Tijdigheid::Deels,
            Tijdigheidkeuze::Nee => Tijdigheid::Nee,
        }
    }
}

#[derive(clap::ValueEnum, Clone, Copy, Debug)]
pub enum Reactiekeuze {
    Overgenomen,
    Deels,
    Niet,
    GeenReactie,
}

impl From<Reactiekeuze> for Reactiestatus {
    fn from(k: Reactiekeuze) -> Self {
        match k {
            Reactiekeuze::Overgenomen => Reactiestatus::Overgenomen,
            Reactiekeuze::Deels => Reactiestatus::Deels,
            Reactiekeuze::Niet => Reactiestatus::Niet,
            Reactiekeuze::GeenReactie => Reactiestatus::GeenReactie,
        }
    }
}

#[derive(clap::ValueEnum, Clone, Copy, Debug)]
pub enum Soortkeuze {
    Instructie,
    ToegangGeweigerd,
    CapaciteitGeweigerd,
    Belangenconflict,
    SanctieGedreigd,
    BeoordelingGekoppeld,
}

impl From<Soortkeuze> for Aantastingsoort {
    fn from(k: Soortkeuze) -> Self {
        match k {
            Soortkeuze::Instructie => Aantastingsoort::InstructieGegeven,
            Soortkeuze::ToegangGeweigerd => Aantastingsoort::ToegangGeweigerd,
            Soortkeuze::CapaciteitGeweigerd => Aantastingsoort::CapaciteitGeweigerd,
            Soortkeuze::Belangenconflict => Aantastingsoort::Belangenconflict,
            Soortkeuze::SanctieGedreigd => Aantastingsoort::SanctieGedreigd,
            Soortkeuze::BeoordelingGekoppeld => Aantastingsoort::BeoordelingGekoppeld,
        }
    }
}

/// Waar het persoonlijke dossier standaard staat.
fn dossierpad(meegegeven: Option<PathBuf>) -> Result<PathBuf> {
    if let Some(p) = meegegeven {
        return Ok(p);
    }
    Ok(super::standaardmap()?.join("fg-persoonlijk.dpofg"))
}

/// Opent het persoonlijke dossier met zijn eigen wachtwoordzin.
fn open_dossier(pad: &std::path::Path, nu: DateTime<Utc>) -> Result<Kluis> {
    if !pad.exists() {
        anyhow::bail!(
            "er staat geen persoonlijk dossier op {}. Maak er een aan met 'dpofg fg nieuw'",
            pad.display()
        );
    }
    let wachtwoord = crate::wachtwoord::vraag_met(
        "Wachtwoordzin van uw persoonlijke dossier",
        crate::wachtwoord::OMGEVINGSVARIABELE_FG,
    )?;
    let mut kluis = Kluis::openen(pad, &wachtwoord, nu)?;
    let namen: Vec<String> = kluis.compartimenten().iter().map(|s| s.to_string()).collect();
    for naam in namen {
        kluis.compartiment_ontgrendelen(&naam)?;
    }
    Ok(kluis)
}

pub fn draai(a: Fgargumenten, kluispad: Option<PathBuf>, nu: DateTime<Utc>) -> Result<()> {
    let o = a.opdracht;
    if matches!(o, Fgopdracht::Gronden) {
        return gronden();
    }
    if let Fgopdracht::Nieuw { pad, zonder_organisatiekluis } = o {
        return nieuw(&pad, zonder_organisatiekluis, kluispad, nu);
    }

    let pad = dossierpad(a.dossier.clone())?;
    let mut dossier = open_dossier(&pad, nu)?;
    match o {
        Fgopdracht::Nieuw { .. } | Fgopdracht::Gronden => unreachable!("hierboven afgehandeld"),
        Fgopdracht::Lijst => lijst(&dossier),
        Fgopdracht::Advies {
            kenmerk,
            onderwerp,
            vraagsteller,
            advies,
            aan,
            op,
            tijdig,
            toelichting,
        } => nieuw_advies(
            &mut dossier,
            &kenmerk,
            &onderwerp,
            &vraagsteller,
            &advies,
            &aan,
            op.as_deref(),
            tijdig.into(),
            toelichting.as_deref(),
            nu,
        ),
        Fgopdracht::Reactie { kenmerk, status, beslisser, datum, motivering } => reactie(
            &mut dossier,
            &kenmerk,
            status.into(),
            &beslisser,
            datum.as_deref(),
            motivering.as_deref(),
            nu,
        ),
        Fgopdracht::Escaleren { kenmerk, niveau, datum, uitkomst } => {
            escaleren(&mut dossier, &kenmerk, &niveau, datum.as_deref(), uitkomst, nu)
        }
        Fgopdracht::Onafhankelijkheid { kenmerk, soort, datum, omschrijving, van, betrof } => {
            onafhankelijkheid(
                &mut dossier,
                &kenmerk,
                soort.into(),
                datum.as_deref(),
                &omschrijving,
                &van,
                betrof.as_deref(),
                nu,
            )
        }
        Fgopdracht::Toon { kenmerk } => toon(&dossier, &kenmerk),
        Fgopdracht::Spiegelen { kenmerk } => spiegelen(&mut dossier, &kenmerk, kluispad, nu),
        Fgopdracht::Aantonen { kenmerk } => aantonen(&dossier, &kenmerk, kluispad, nu),
    }
}

fn lees_tijdstip(tekst: &str) -> Result<DateTime<Utc>> {
    tekst.parse::<DateTime<chrono::FixedOffset>>().map(|t| t.with_timezone(&Utc)).map_err(|e| {
        anyhow::anyhow!(
            "kon '{tekst}' niet lezen als tijdstip ({e}). Gebruik de vorm 2026-08-19T09:00:00Z"
        )
    })
}

fn gronden() -> Result<()> {
    kop("Waarop uw onafhankelijkheid rust");
    let mut t = tabel(&["wat er gebeurt", "grondslag"]);
    for soort in Aantastingsoort::alle() {
        t.add_row(vec![soort.omschrijving().to_string(), soort.grondslag().to_string()]);
    }
    println!("{t}");
    terzijde(
        "Artikel 38 lid 3 AVG verbiedt u te ontslaan of te straffen voor de uitvoering van uw \
         taken. Die bescherming is waardeloos wanneer het bewijs ervan uitsluitend berust bij \
         degene tegen wie zij is gericht.",
    );
    Ok(())
}

fn nieuw(
    pad: &std::path::Path,
    zonder_organisatiekluis: bool,
    kluispad: Option<PathBuf>,
    nu: DateTime<Utc>,
) -> Result<()> {
    if pad.exists() {
        anyhow::bail!("er staat al een bestand op {}", pad.display());
    }
    kop("Een persoonlijk dossier aanmaken");
    terzijde(
        "Dit dossier komt in een eigen bestand met een eigen wachtwoordzin. De organisatie kan \
         de inhoud niet lezen; in haar kluis blijft alleen een hash achter waarmee is aan te \
         tonen dát een record bestond.",
    );
    let_op(
        "Of deze constructie standhoudt tegenover eigendoms- en archiefaanspraken van de \
         organisatie, is niet vastgesteld. Leg dit vast in uw aanstellingsovereenkomst voordat \
         u erop vertrouwt; de keuze om dit dossier te voeren is aan u.",
    );

    let wachtwoord = crate::wachtwoord::vraag_nieuw_met(crate::wachtwoord::OMGEVINGSVARIABELE_FG)?;

    // De zin mag de kluis van de organisatie niet openen. Zou dat wel zo zijn,
    // dan is de scheiding schijn: wie die zin kent, leest beide.
    if !zonder_organisatiekluis {
        let orgpad = super::kluispad(kluispad)?;
        if orgpad.exists() && Kluis::openen(&orgpad, &wachtwoord, nu).is_ok() {
            anyhow::bail!(
                "deze wachtwoordzin opent ook de kluis van de organisatie. Daarmee zou iedereen \
                 die die zin kent uw persoonlijke dossier kunnen lezen, en dan beschermt het \
                 niets. Kies een andere zin"
            );
        }
    }

    let mut dossier =
        Kluis::aanmaken(pad, &wachtwoord, dpofg_crypto::kdf::KdfParameters::STANDAARD, nu)?;
    dossier.compartiment_aanmaken(
        COMPARTIMENT,
        "het persoonlijke dossier van de functionaris",
        nu,
    )?;

    gelukt(&format!("persoonlijk dossier aangemaakt op {}", pad.display()));
    terzijde(&format!(
        "voor geautomatiseerd gebruik leest de schil de zin uit {}",
        crate::wachtwoord::OMGEVINGSVARIABELE_FG
    ));
    let_op(
        "Er is geen herstelmogelijkheid. Raakt u deze zin kwijt, dan is dit dossier weg, en \
         niemand anders kan het openen — dat is precies waarom het bestaat.",
    );
    Ok(())
}

fn bewaar_advies(
    dossier: &mut Kluis,
    a: &Advies,
    handeling: Handeling,
    omschrijving: &str,
    nu: DateTime<Utc>,
) -> Result<()> {
    let actor = super::actor();
    dossier.bewaar(
        "advies",
        &a.id.to_string(),
        COMPARTIMENT,
        a.status.omschrijving(),
        Some(&a.kenmerk),
        a,
        &actor,
        handeling,
        omschrijving,
        nu,
    )?;
    Ok(())
}

fn zoek_advies(dossier: &Kluis, kenmerk: &str) -> Result<Option<Advies>> {
    let kop = dossier.lijst("advies")?.into_iter().find(|r| r.kenmerk.as_deref() == Some(kenmerk));
    match kop {
        Some(k) => Ok(Some(dossier.laad("advies", &k.id)?)),
        None => Ok(None),
    }
}

fn zoek_incident(dossier: &Kluis, kenmerk: &str) -> Result<Option<Onafhankelijkheidsincident>> {
    let kop = dossier
        .lijst("onafhankelijkheidsincident")?
        .into_iter()
        .find(|r| r.kenmerk.as_deref() == Some(kenmerk));
    match kop {
        Some(k) => Ok(Some(dossier.laad("onafhankelijkheidsincident", &k.id)?)),
        None => Ok(None),
    }
}

#[allow(clippy::too_many_arguments)]
fn nieuw_advies(
    dossier: &mut Kluis,
    kenmerk: &str,
    onderwerp: &str,
    vraagsteller: &str,
    advies: &str,
    aan: &str,
    op: Option<&str>,
    tijdig: Tijdigheid,
    toelichting: Option<&str>,
    nu: DateTime<Utc>,
) -> Result<()> {
    if zoek_advies(dossier, kenmerk)?.is_some() {
        anyhow::bail!("er bestaat al een advies met kenmerk '{kenmerk}'");
    }
    let moment = match op {
        Some(t) => lees_tijdstip(t)?,
        None => nu,
    };
    let toelichting = match toelichting {
        Some(t) => Some(Motivering::nieuw(t, &super::actor().id, nu)?),
        None => None,
    };
    let a = Advies::nieuw(
        kenmerk,
        onderwerp,
        vraagsteller,
        advies,
        aan,
        moment,
        tijdig,
        toelichting,
        &super::actor().id,
        nu,
    )?;
    bewaar_advies(dossier, &a, Handeling::RecordAangemaakt, "advies vastgelegd", nu)?;

    gelukt(&format!("advies {kenmerk} vastgelegd"));
    if tijdig.vraagt_toelichting() {
        terzijde("art. 38 lid 1 AVG — de toelichting staat erbij");
    }
    terzijde(
        "Leg de hash vast in de kluis van de organisatie met 'dpofg fg spiegelen'; anders \
         berust het tijdstip uitsluitend op uw eigen opgave.",
    );
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn reactie(
    dossier: &mut Kluis,
    kenmerk: &str,
    status: Reactiestatus,
    beslisser: &str,
    datum: Option<&str>,
    motivering: Option<&str>,
    nu: DateTime<Utc>,
) -> Result<()> {
    let mut a = zoek_advies(dossier, kenmerk)?
        .ok_or_else(|| anyhow::anyhow!("geen advies met kenmerk '{kenmerk}'"))?;
    let moment = match datum {
        Some(t) => lees_tijdstip(t)?,
        None => nu,
    };
    let m = match motivering {
        Some(t) => Some(Motivering::nieuw(t, &super::actor().id, nu)?),
        None => None,
    };
    a.leg_reactie_vast(status, beslisser, moment, m, nu)?;
    bewaar_advies(dossier, &a, Handeling::RecordGewijzigd, "bestuursreactie vastgelegd", nu)?;

    gelukt(&format!("{kenmerk}: {}", status.omschrijving()));
    terzijde(&format!("{} dagen tussen advies en reactie", a.dagen_tot_reactie(nu)));
    if status == Reactiestatus::GeenReactie {
        let_op(
            "Het uitblijven van een reactie is zelf een feit. Overweeg te escaleren en dat vast \
             te leggen; een advies dat nergens landt, is later alleen aantoonbaar met de \
             stappen die u hebt gezet.",
        );
    }
    Ok(())
}

fn escaleren(
    dossier: &mut Kluis,
    kenmerk: &str,
    niveau: &str,
    datum: Option<&str>,
    uitkomst: Option<String>,
    nu: DateTime<Utc>,
) -> Result<()> {
    let mut a = zoek_advies(dossier, kenmerk)?
        .ok_or_else(|| anyhow::anyhow!("geen advies met kenmerk '{kenmerk}'"))?;
    let moment = match datum {
        Some(t) => lees_tijdstip(t)?,
        None => nu,
    };
    a.escaleer(niveau, moment, uitkomst, nu)?;
    bewaar_advies(dossier, &a, Handeling::RecordGewijzigd, "escalatiestap vastgelegd", nu)?;

    gelukt(&format!("opgeschaald naar {niveau}"));
    terzijde(&format!("{} escalatiestap(pen) bij dit advies", a.escalatie.len()));
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn onafhankelijkheid(
    dossier: &mut Kluis,
    kenmerk: &str,
    soort: Aantastingsoort,
    datum: Option<&str>,
    omschrijving: &str,
    van: &str,
    betrof: Option<&str>,
    nu: DateTime<Utc>,
) -> Result<()> {
    if zoek_incident(dossier, kenmerk)?.is_some() {
        anyhow::bail!("er bestaat al een gebeurtenis met kenmerk '{kenmerk}'");
    }
    let moment = match datum {
        Some(t) => lees_tijdstip(t)?,
        None => nu,
    };
    let actor = super::actor();
    let i = Onafhankelijkheidsincident::nieuw(
        kenmerk,
        soort,
        moment,
        omschrijving,
        betrof.unwrap_or(&actor.naam),
        van,
        &actor.id,
        nu,
    )?;
    dossier.bewaar(
        "onafhankelijkheidsincident",
        &i.id.to_string(),
        COMPARTIMENT,
        i.status.omschrijving(),
        Some(&i.kenmerk),
        &i,
        &actor,
        Handeling::RecordAangemaakt,
        "onafhankelijkheidsincident vastgelegd",
        nu,
    )?;

    gelukt(&format!("{kenmerk} vastgelegd: {}", soort.omschrijving()));
    terzijde(soort.grondslag());
    terzijde(
        "Leg de hash vast in de kluis van de organisatie met 'dpofg fg spiegelen'; dat is wat \
         het tijdstip later toetsbaar maakt.",
    );
    Ok(())
}

fn lijst(dossier: &Kluis) -> Result<()> {
    let adviezen = dossier.lijst("advies")?;
    let incidenten = dossier.lijst("onafhankelijkheidsincident")?;

    kop("Adviezen");
    if adviezen.is_empty() {
        terzijde("nog geen");
    } else {
        let mut t = tabel(&["kenmerk", "onderwerp", "aan", "uitgebracht", "reactie"]);
        for k in &adviezen {
            let a: Advies = dossier.laad("advies", &k.id)?;
            t.add_row(vec![
                a.kenmerk.clone(),
                a.onderwerp.clone(),
                a.uitgebracht_aan.clone(),
                a.uitgebracht_op.format("%d-%m-%Y").to_string(),
                a.bestuursreactie
                    .as_ref()
                    .map(|r| r.status.omschrijving().to_string())
                    .unwrap_or_else(|| "nog geen".into()),
            ]);
        }
        println!("{t}");
    }

    kop("Onafhankelijkheid");
    if incidenten.is_empty() {
        terzijde("nog geen");
    } else {
        let mut t = tabel(&["kenmerk", "wat er gebeurde", "van", "datum", "opvolging"]);
        for k in &incidenten {
            let i: Onafhankelijkheidsincident =
                dossier.laad("onafhankelijkheidsincident", &k.id)?;
            t.add_row(vec![
                i.kenmerk.clone(),
                i.soort.omschrijving().to_string(),
                i.van.clone(),
                i.datum.format("%d-%m-%Y").to_string(),
                i.opvolging.clone().unwrap_or_else(|| "nog geen".into()),
            ]);
        }
        println!("{t}");
    }
    Ok(())
}

fn toon(dossier: &Kluis, kenmerk: &str) -> Result<()> {
    if let Some(a) = zoek_advies(dossier, kenmerk)? {
        kop(&format!("Advies {}", a.kenmerk));
        let mut t = tabel(&["", ""]);
        t.add_row(vec!["onderwerp", &a.onderwerp]);
        t.add_row(vec!["gevraagd door", &a.vraagsteller]);
        t.add_row(vec!["uitgebracht aan", &a.uitgebracht_aan]);
        let op = a.uitgebracht_op.format("%d-%m-%Y").to_string();
        t.add_row(vec!["uitgebracht op", &op]);
        t.add_row(vec!["betrokkenheid", a.tijdig_betrokken.omschrijving()]);
        println!("{t}");
        println!("  {}", a.adviestekst);
        if let Some(m) = &a.tijdigheidstoelichting {
            terzijde(&format!("over de betrokkenheid: {}", m.tekst));
        }

        if let Some(r) = &a.bestuursreactie {
            kop("Reactie van het bestuur");
            let mut t = tabel(&["", ""]);
            t.add_row(vec!["uitkomst", r.status.omschrijving()]);
            t.add_row(vec!["door", &r.beslisser]);
            let d = r.datum.format("%d-%m-%Y").to_string();
            t.add_row(vec!["op", &d]);
            println!("{t}");
            if let Some(m) = &r.motivering {
                println!("  {}", m.tekst);
            }
        } else {
            let_op("er is nog geen reactie vastgelegd");
        }

        if !a.escalatie.is_empty() {
            kop("Escalatie");
            let mut t = tabel(&["naar", "datum", "uitkomst"]);
            for e in &a.escalatie {
                t.add_row(vec![
                    e.niveau.clone(),
                    e.datum.format("%d-%m-%Y").to_string(),
                    e.uitkomst.clone().unwrap_or_else(|| "nog geen".into()),
                ]);
            }
            println!("{t}");
        }
        return Ok(());
    }

    let i = zoek_incident(dossier, kenmerk)?.ok_or_else(|| {
        anyhow::anyhow!("geen advies en geen gebeurtenis met kenmerk '{kenmerk}'")
    })?;
    kop(&format!("Onafhankelijkheid {}", i.kenmerk));
    let mut t = tabel(&["", ""]);
    t.add_row(vec!["wat er gebeurde", i.soort.omschrijving()]);
    t.add_row(vec!["grondslag", i.soort.grondslag()]);
    let d = i.datum.format("%d-%m-%Y").to_string();
    t.add_row(vec!["datum", &d]);
    t.add_row(vec!["van", &i.van]);
    t.add_row(vec!["betrof", &i.betrokken_functionaris]);
    println!("{t}");
    println!("  {}", i.omschrijving);
    match &i.opvolging {
        Some(o) => terzijde(&format!("opvolging: {o}")),
        None => let_op("er is nog geen opvolging vastgelegd"),
    }
    Ok(())
}

/// Bepaalt de soort en de hash van een record uit het persoonlijke dossier.
fn hash_van(dossier: &Kluis, kenmerk: &str) -> Result<(String, String)> {
    if let Some(a) = zoek_advies(dossier, kenmerk)? {
        return Ok(("advies".into(), spiegelhash("advies", &a.spiegelbaar())?));
    }
    let i = zoek_incident(dossier, kenmerk)?.ok_or_else(|| {
        anyhow::anyhow!("geen advies en geen gebeurtenis met kenmerk '{kenmerk}'")
    })?;
    Ok((
        "onafhankelijkheidsincident".into(),
        spiegelhash("onafhankelijkheidsincident", &i.spiegelbaar())?,
    ))
}

/// Legt in het persoonlijke dossier vast dat er is gespiegeld.
fn noteer_spiegeling(
    dossier: &mut Kluis,
    kenmerk: &str,
    hash: &str,
    nu: DateTime<Utc>,
) -> Result<()> {
    let spiegeling = Spiegeling { hash: hash.to_string(), op: nu };
    if let Some(mut a) = zoek_advies(dossier, kenmerk)? {
        // Geen herkomst.wijzig: spiegelen verandert niets aan het advies zelf.
        a.spiegelingen.push(spiegeling);
        return bewaar_advies(dossier, &a, Handeling::RecordGewijzigd, "gespiegeld", nu);
    }
    let mut i = zoek_incident(dossier, kenmerk)?
        .ok_or_else(|| anyhow::anyhow!("geen record met kenmerk '{kenmerk}'"))?;
    i.spiegelingen.push(spiegeling);
    let actor = super::actor();
    dossier.bewaar(
        "onafhankelijkheidsincident",
        &i.id.to_string(),
        COMPARTIMENT,
        i.status.omschrijving(),
        Some(&i.kenmerk),
        &i,
        &actor,
        Handeling::RecordGewijzigd,
        "gespiegeld",
        nu,
    )?;
    Ok(())
}

fn spiegelen(
    dossier: &mut Kluis,
    kenmerk: &str,
    kluispad: Option<PathBuf>,
    nu: DateTime<Utc>,
) -> Result<()> {
    let (soort, hash) = hash_van(dossier, kenmerk)?;
    let orgpad = super::kluispad(kluispad)?;
    let mut kluis = super::open_kluis(&orgpad, nu)?;

    if kluis.lijst(SPIEGELSOORT)?.iter().any(|k| k.id == hash) {
        anyhow::bail!(
            "deze hash staat al in de kluis van de organisatie; het record is sinds het \
             spiegelen niet gewijzigd"
        );
    }
    let regel = Spiegelregel { hash: hash.clone(), soort: soort.clone(), vastgelegd_op: nu };
    kluis.bewaar(
        SPIEGELSOORT,
        &hash,
        "algemeen",
        "vastgesteld",
        None,
        &regel,
        &super::actor(),
        Handeling::RecordAangemaakt,
        &format!("hash van een {soort} uit het persoonlijke dossier vastgelegd"),
        nu,
    )?;

    // De spiegeling ook in het eigen dossier vastleggen. Anders is later niet
    // te zien dát er ooit is gespiegeld, en dan zijn "nooit gespiegeld" en "na
    // het spiegelen gewijzigd" niet uit elkaar te houden.
    noteer_spiegeling(dossier, kenmerk, &hash, nu)?;

    gelukt("de hash staat in de kluis van de organisatie");
    let mut t = tabel(&["", ""]);
    t.add_row(vec!["soort", &soort]);
    t.add_row(vec!["hash", &hash]);
    let volgnummer = kluis.ketenstand().volgnummer.to_string();
    t.add_row(vec!["ketenregel", &volgnummer]);
    println!("{t}");
    terzijde(
        "In de kluis van de organisatie staat uitsluitend deze hash. Geen kenmerk, geen \
         onderwerp, geen tekst: wat er is geadviseerd blijft in uw dossier.",
    );
    let_op(
        "Wijzigt u het record hierna nog, dan komt de hash niet meer overeen en is er geen \
         bewijs. Spiegel opnieuw wanneer u iets aanvult.",
    );
    Ok(())
}

/// De spiegelstand van een record uit het persoonlijke dossier.
fn spiegelingen_van(dossier: &Kluis, kenmerk: &str) -> Result<Spiegelstand> {
    if let Some(a) = zoek_advies(dossier, kenmerk)? {
        let hash = spiegelhash("advies", &a.spiegelbaar())?;
        return Ok(spiegelstand(&a.spiegelingen, &hash));
    }
    let i = zoek_incident(dossier, kenmerk)?
        .ok_or_else(|| anyhow::anyhow!("geen record met kenmerk '{kenmerk}'"))?;
    let hash = spiegelhash("onafhankelijkheidsincident", &i.spiegelbaar())?;
    Ok(spiegelstand(&i.spiegelingen, &hash))
}

fn aantonen(
    dossier: &Kluis,
    kenmerk: &str,
    kluispad: Option<PathBuf>,
    nu: DateTime<Utc>,
) -> Result<()> {
    let (soort, hash) = hash_van(dossier, kenmerk)?;
    let orgpad = super::kluispad(kluispad)?;
    let kluis = super::open_kluis(&orgpad, nu)?;

    kop(&format!("Bestaan aantonen van {kenmerk}"));
    let mut t = tabel(&["", ""]);
    t.add_row(vec!["soort", &soort]);
    t.add_row(vec!["hash nu", &hash]);
    println!("{t}");

    let treffer = kluis.lijst(SPIEGELSOORT)?.into_iter().find(|k| k.id == hash);
    let Some(k) = treffer else {
        // Nooit gespiegeld en na het spiegelen gewijzigd zijn twee heel
        // verschillende antwoorden; die door elkaar halen laat de gebruiker in
        // het ongewisse over de vraag waar het bewijs is gebleven.
        match spiegelingen_van(dossier, kenmerk)? {
            Spiegelstand::NooitGespiegeld => {
                blokkade(
                    "dit record is nooit gespiegeld; het tijdstip berust uitsluitend op uw \
                     eigen opgave",
                );
                terzijde("spiegel het met 'dpofg fg spiegelen'; dat dateert vanaf vandaag");
            }
            Spiegelstand::Gewijzigd { laatste, aantal } => {
                blokkade(&format!(
                    "dit record is sinds de laatste spiegeling van {} gewijzigd; de huidige \
                     inhoud is daarmee niet aangetoond",
                    laatste.format("%d-%m-%Y")
                ));
                terzijde(&format!(
                    "er {} van dit record in de kluis van de organisatie; {} aan dát er toen \
                     iets was, niet wat erin stond",
                    if aantal == 1 {
                        "staat 1 eerdere spiegeling".to_string()
                    } else {
                        format!("staan {aantal} eerdere spiegelingen")
                    },
                    if aantal == 1 { "die toont" } else { "die tonen" }
                ));
                terzijde("spiegel opnieuw om de huidige inhoud vast te leggen");
            }
            Spiegelstand::Sluitend { .. } => blokkade(
                "uw dossier zegt dat dit record is gespiegeld, maar de bijbehorende regel \
                 staat niet in de kluis van de organisatie. Die is daar verwijderd of het \
                 betreft een andere kluis",
            ),
        }
        // De uitleg staat hierboven; deze regel is er voor de afsluitcode.
        anyhow::bail!("het bestaan is niet aangetoond");
    };
    let regel: Spiegelregel = kluis.laad(SPIEGELSOORT, &k.id)?;

    gelukt(&format!(
        "dit record bestond op {} en is sindsdien niet gewijzigd",
        regel.vastgelegd_op.format("%d-%m-%Y %H:%M UTC")
    ));
    let ketenrapport = kluis.verifieer_logboek()?;
    terzijde(&ketenrapport.reikwijdte());
    if !ketenrapport.bevindingen.is_empty() {
        blokkade(
            "het ketenlogboek van de organisatie is niet zonder bevindingen doorlopen; het \
             tijdstip rust daarmee op een keten die zelf ter discussie staat",
        );
    }
    terzijde(
        "Wat dit aantoont: de inhoud van uw record levert dezelfde hash op als die welke op dat \
         moment in het ketenlogboek van de organisatie is vastgelegd. Wat het niet aantoont: \
         dat de inhoud juist is.",
    );
    Ok(())
}
