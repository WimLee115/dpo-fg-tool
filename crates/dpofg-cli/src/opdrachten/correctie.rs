//! De correctieplicht: het besluit over een bevinding.

use anyhow::Result;
use chrono::{DateTime, Utc};
use clap::Subcommand;
use dpofg_audit::Handeling;
use dpofg_domain::{
    correctie::{Bevindingsleutel, Correctie, Correctiesoort},
    Motivering,
};
use dpofg_rules::regels::standaardmotor;
use dpofg_store::Kluis;
use std::path::PathBuf;

use crate::uitvoer::{blokkade, gelukt, kop, let_op, tabel, terzijde};

const SOORT: &str = "correctie";
const COMPARTIMENT: &str = "algemeen";

#[derive(Subcommand, Debug)]
pub enum Correctieopdracht {
    /// Toon alle correcties.
    Lijst {
        /// Toon ook de afgeronde correcties.
        #[arg(long)]
        alles: bool,
    },
    /// Leg vast wat er met een bevinding gebeurt.
    Nieuw {
        /// Kenmerk van de correctie.
        kenmerk: String,
        /// De regelcode van de bevinding.
        #[arg(long)]
        regel: String,
        /// De soort van het record waarop de bevinding slaat.
        #[arg(long)]
        soort: String,
        /// Het kenmerk van dat record.
        #[arg(long)]
        record: String,
        /// Wat er gaat gebeuren.
        #[arg(long, value_enum, default_value = "herstel")]
        aanpak: Soortkeuze,
        /// De rol die het oppakt.
        #[arg(long)]
        rol: String,
        /// Wie die rol vervult.
        #[arg(long)]
        persoon: String,
        /// Wanneer het klaar is, of tot wanneer wordt afgeweken.
        #[arg(long)]
        uiterlijk: String,
        /// Wat er gaat gebeuren, of waarom er wordt afgeweken.
        #[arg(long)]
        motivering: String,
        /// Wat de bevinding zegt. Standaard: de tekst uit de catalogus.
        #[arg(long)]
        bevinding: Option<String>,
    },
    /// Toon één correctie.
    Toon {
        /// Kenmerk van de correctie.
        kenmerk: String,
    },
    /// Verleng de afgesproken datum.
    Verlengen {
        /// Kenmerk van de correctie.
        kenmerk: String,
        /// De nieuwe datum.
        #[arg(long)]
        uiterlijk: String,
        /// Waarom er meer tijd nodig is.
        #[arg(long)]
        motivering: String,
    },
    /// Rond de correctie af.
    Afronden {
        /// Kenmerk van de correctie.
        kenmerk: String,
        /// Wat er is gebeurd.
        #[arg(long)]
        motivering: String,
    },
}

#[derive(clap::ValueEnum, Clone, Copy, Debug)]
pub enum Soortkeuze {
    /// De tekortkoming wordt weggenomen.
    Herstel,
    /// Er wordt tot de afgesproken datum bewust van afgeweken.
    Afwijking,
}

impl From<Soortkeuze> for Correctiesoort {
    fn from(k: Soortkeuze) -> Self {
        match k {
            Soortkeuze::Herstel => Correctiesoort::Herstel,
            Soortkeuze::Afwijking => Correctiesoort::Afwijking,
        }
    }
}

pub fn draai(o: Correctieopdracht, kluispad: Option<PathBuf>, nu: DateTime<Utc>) -> Result<()> {
    let pad = super::kluispad(kluispad)?;
    let mut kluis = super::open_kluis(&pad, nu)?;
    match o {
        Correctieopdracht::Lijst { alles } => lijst(&kluis, alles, nu),
        Correctieopdracht::Nieuw {
            kenmerk,
            regel,
            soort,
            record,
            aanpak,
            rol,
            persoon,
            uiterlijk,
            motivering,
            bevinding,
        } => nieuw(
            &mut kluis,
            &kenmerk,
            &regel,
            &soort,
            &record,
            aanpak.into(),
            &rol,
            &persoon,
            &uiterlijk,
            &motivering,
            bevinding.as_deref(),
            nu,
        ),
        Correctieopdracht::Toon { kenmerk } => toon(&kluis, &kenmerk, nu),
        Correctieopdracht::Verlengen { kenmerk, uiterlijk, motivering } => {
            verlengen(&mut kluis, &kenmerk, &uiterlijk, &motivering, nu)
        }
        Correctieopdracht::Afronden { kenmerk, motivering } => {
            afronden(&mut kluis, &kenmerk, &motivering, nu)
        }
    }
}

fn zoek(kluis: &Kluis, kenmerk: &str) -> Result<Correctie> {
    let kop = kluis
        .lijst(SOORT)?
        .into_iter()
        .find(|r| r.kenmerk.as_deref() == Some(kenmerk))
        .ok_or_else(|| anyhow::anyhow!("geen correctie met kenmerk '{kenmerk}'"))?;
    Ok(kluis.laad(SOORT, &kop.id)?)
}

fn bewaar(
    kluis: &mut Kluis,
    c: &Correctie,
    handeling: Handeling,
    omschrijving: &str,
    nu: DateTime<Utc>,
) -> Result<()> {
    let actor = super::actor();
    kluis.bewaar(
        SOORT,
        &c.id.to_string(),
        COMPARTIMENT,
        c.status.omschrijving(),
        Some(&c.kenmerk),
        c,
        &actor,
        handeling,
        omschrijving,
        nu,
    )?;
    Ok(())
}

fn lees_tijdstip(tekst: &str) -> Result<DateTime<Utc>> {
    tekst.parse::<DateTime<chrono::FixedOffset>>().map(|t| t.with_timezone(&Utc)).map_err(|e| {
        anyhow::anyhow!(
            "kon '{tekst}' niet lezen als tijdstip ({e}). Gebruik de vorm 2026-12-01T00:00:00Z"
        )
    })
}

#[allow(clippy::too_many_arguments)]
fn nieuw(
    kluis: &mut Kluis,
    kenmerk: &str,
    regelcode: &str,
    record_soort: &str,
    record_kenmerk: &str,
    soort: Correctiesoort,
    rol: &str,
    persoon: &str,
    uiterlijk: &str,
    motivering: &str,
    bevindingstekst: Option<&str>,
    nu: DateTime<Utc>,
) -> Result<()> {
    if kluis.lijst(SOORT)?.iter().any(|r| r.kenmerk.as_deref() == Some(kenmerk)) {
        anyhow::bail!("er bestaat al een correctie met kenmerk '{kenmerk}'");
    }
    // Of van deze regel mag worden afgeweken, staat in de catalogus en niet in
    // het record. Een onbekende regelcode wordt hier geweigerd: een correctie
    // voor een regel die niet bestaat, kan nooit worden afgerond.
    let motor = standaardmotor();
    let regel = motor.regel(regelcode).ok_or_else(|| {
        anyhow::anyhow!(
            "'{regelcode}' staat niet in de regelcatalogus. Bekijk wat er is met \
             'dpofg controle --dekking'"
        )
    })?;
    let tekst = bevindingstekst.map(|t| t.to_string()).unwrap_or_else(|| regel.controleert.clone());
    let afwijking_mogelijk = regel.afwijking_mogelijk;

    let c = Correctie::nieuw(
        kenmerk,
        Bevindingsleutel::nieuw(regelcode, record_soort, record_kenmerk),
        tekst,
        soort,
        afwijking_mogelijk,
        rol,
        persoon,
        lees_tijdstip(uiterlijk)?,
        Motivering::nieuw(motivering, &super::actor().id, nu)?,
        &super::actor().id,
        nu,
    )?;
    bewaar(kluis, &c, Handeling::BesluitGenomen, "correctie vastgelegd", nu)?;

    gelukt(&format!(
        "{} over {} is belegd bij {rol} ({persoon})",
        c.soort.omschrijving(),
        c.bevinding.aanduiding()
    ));
    terzijde(&format!("uiterlijk {}", c.uiterlijk.format("%d-%m-%Y")));
    match soort {
        Correctiesoort::Herstel => terzijde(
            "Een herstelafspraak onderdrukt de bevinding niet: zolang de tekortkoming er is, \
             blijft zij in de controleronde staan, ook nu er iemand aan werkt.",
        ),
        Correctiesoort::Afwijking => let_op(
            "Tot de afgesproken datum wordt deze bevinding als afwijking getoond en niet meer \
             als openstaand punt. Daarna staat zij er weer, want een afwijking zonder einde \
             wordt de nieuwe norm.",
        ),
    }
    Ok(())
}

fn verlengen(
    kluis: &mut Kluis,
    kenmerk: &str,
    uiterlijk: &str,
    motivering: &str,
    nu: DateTime<Utc>,
) -> Result<()> {
    let mut c = zoek(kluis, kenmerk)?;
    let oud = c.uiterlijk;
    c.verleng(
        lees_tijdstip(uiterlijk)?,
        Motivering::nieuw(motivering, &super::actor().id, nu)?,
        nu,
    )?;
    bewaar(kluis, &c, Handeling::BesluitGenomen, "correctietermijn verlengd", nu)?;

    gelukt(&format!(
        "de termijn loopt van {} naar {}",
        oud.format("%d-%m-%Y"),
        c.uiterlijk.format("%d-%m-%Y")
    ));
    let_op(
        "Uitstel is een besluit en geen administratieve handeling. Een reeks verlengingen is \
         zelf het signaal dat de afspraak niet werkt.",
    );
    Ok(())
}

fn afronden(kluis: &mut Kluis, kenmerk: &str, motivering: &str, nu: DateTime<Utc>) -> Result<()> {
    let mut c = zoek(kluis, kenmerk)?;
    let actor = super::actor();
    let te_laat = c.is_te_laat(nu);
    c.rond_af(&actor.naam, Motivering::nieuw(motivering, &actor.id, nu)?, nu)?;
    bewaar(kluis, &c, Handeling::RecordVastgesteld, "correctie afgerond", nu)?;

    gelukt(&format!("correctie {kenmerk} is afgerond"));
    if te_laat {
        terzijde(&format!(
            "de afspraak liep tot {}; dat de termijn is overschreden blijft in het logboek \
             staan",
            c.uiterlijk.format("%d-%m-%Y")
        ));
    }
    terzijde(
        "Of de tekortkoming werkelijk weg is, blijkt uit de volgende controleronde en niet uit \
         deze afronding.",
    );
    Ok(())
}

fn lijst(kluis: &Kluis, alles: bool, nu: DateTime<Utc>) -> Result<()> {
    let koppen = kluis.lijst(SOORT)?;
    kop("Correcties");
    if koppen.is_empty() {
        terzijde("Er staat nog geen correctie in de kluis.");
        return Ok(());
    }
    let mut t = tabel(&["kenmerk", "bevinding", "aanpak", "eigenaar", "uiterlijk", "stand"]);
    let mut getoond = 0usize;
    let mut verborgen = 0usize;
    for k in &koppen {
        let c: Correctie = kluis.laad(SOORT, &k.id)?;
        if c.is_afgerond() && !alles {
            verborgen += 1;
            continue;
        }
        let stand = if c.is_afgerond() {
            "afgerond".to_string()
        } else if c.is_te_laat(nu) {
            format!("{} dagen te laat", -c.dagen_tot_uiterlijk(nu))
        } else {
            format!("nog {} dagen", c.dagen_tot_uiterlijk(nu))
        };
        t.add_row(vec![
            c.kenmerk.clone(),
            c.bevinding.aanduiding(),
            c.soort.omschrijving().to_string(),
            format!("{} ({})", c.eigenaar_rol, c.eigenaar_persoon),
            c.uiterlijk.format("%d-%m-%Y").to_string(),
            stand,
        ]);
        getoond += 1;
    }
    if getoond == 0 {
        terzijde("er staat geen correctie open");
    } else {
        println!("{t}");
    }
    if verborgen > 0 {
        terzijde(&format!("{verborgen} afgeronde correctie(s) niet getoond; gebruik --alles"));
    }
    Ok(())
}

fn toon(kluis: &Kluis, kenmerk: &str, nu: DateTime<Utc>) -> Result<()> {
    let c = zoek(kluis, kenmerk)?;
    kop(&format!("Correctie {}", c.kenmerk));
    let mut t = tabel(&["", ""]);
    let aanduiding = c.bevinding.aanduiding();
    t.add_row(vec!["bevinding", &aanduiding]);
    t.add_row(vec!["wat die zei", &c.bevindingstekst]);
    t.add_row(vec!["aanpak", c.soort.omschrijving()]);
    let eigenaar = format!("{} ({})", c.eigenaar_rol, c.eigenaar_persoon);
    t.add_row(vec!["eigenaar", &eigenaar]);
    let uiterlijk = c.uiterlijk.format("%d-%m-%Y").to_string();
    t.add_row(vec!["uiterlijk", &uiterlijk]);
    t.add_row(vec!["status", c.status.omschrijving()]);
    println!("{t}");
    println!("  {}", c.aanpak.tekst);

    if let Some(a) = &c.afronding {
        kop("Afronding");
        let mut t = tabel(&["", ""]);
        let op = a.op.format("%d-%m-%Y").to_string();
        t.add_row(vec!["op", &op]);
        t.add_row(vec!["door", &a.door]);
        println!("{t}");
        println!("  {}", a.motivering.tekst);
    } else if c.is_te_laat(nu) {
        blokkade(&format!(
            "de afspraak liep tot {} en is {} dagen later nog niet afgerond",
            c.uiterlijk.format("%d-%m-%Y"),
            -c.dagen_tot_uiterlijk(nu)
        ));
    } else if c.onderdrukt(nu) {
        terzijde("deze afwijking loopt; de bevinding wordt tot de einddatum als afwijking getoond");
    }
    Ok(())
}
