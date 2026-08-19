//! De belangenafweging bij een gerechtvaardigd belang.

use anyhow::Result;
use chrono::{DateTime, Utc};
use clap::Subcommand;
use dpofg_audit::Handeling;
use dpofg_domain::{
    belangenafweging::{Afwegingsuitkomst, Belangenafweging},
    Motivering, Verwerking, Volledig,
};
use dpofg_store::Kluis;
use std::path::PathBuf;

use crate::uitvoer::{blokkade, gelukt, kop, let_op, tabel, terzijde, voortgang};

const SOORT: &str = "lia";
const COMPARTIMENT: &str = "algemeen";

#[derive(Subcommand, Debug)]
pub enum Liaopdracht {
    /// Toon alle belangenafwegingen.
    Lijst,
    /// Maak een belangenafweging bij een registerregel.
    Nieuw {
        /// Kenmerk van de afweging.
        kenmerk: String,
        /// Waar het over gaat.
        omschrijving: String,
        /// Het kenmerk van de registerregel.
        #[arg(long)]
        verwerking: String,
    },
    /// Toon één afweging met wat er nog ontbreekt.
    Toon {
        /// Kenmerk van de afweging.
        kenmerk: String,
    },
    /// Vul een onderdeel van de redenering.
    Vul {
        /// Kenmerk van de afweging.
        kenmerk: String,
        /// Welk onderdeel: belang, noodzaak, afweging, verwachtingen of waarborg.
        #[arg(long)]
        veld: String,
        /// De waarde.
        #[arg(long)]
        waarde: String,
    },
    /// Leg de uitkomst vast. Kan pas als de redenering compleet is.
    Uitkomst {
        /// Kenmerk van de afweging.
        kenmerk: String,
        /// De uitkomst.
        #[arg(long, value_enum)]
        uitkomst: Uitkomstkeuze,
        /// Wie de afweging heeft gemaakt.
        #[arg(long)]
        door: String,
        /// Wanneer. Standaard: nu.
        #[arg(long)]
        op: Option<String>,
    },
}

// De drie varianten beginnen alle met hetzelfde woord, en dat is hier juist de
// bedoeling: de gebruiker typt `--uitkomst weegt-op` of `weegt-niet-op`, en dat
// leest als de zin die hij op papier zou schrijven.
#[allow(clippy::enum_variant_names)]
#[derive(clap::ValueEnum, Clone, Copy, Debug)]
pub enum Uitkomstkeuze {
    WeegtOp,
    WeegtOpMetWaarborgen,
    WeegtNietOp,
}

impl From<Uitkomstkeuze> for Afwegingsuitkomst {
    fn from(k: Uitkomstkeuze) -> Self {
        match k {
            Uitkomstkeuze::WeegtOp => Afwegingsuitkomst::BelangWeegtOp,
            Uitkomstkeuze::WeegtOpMetWaarborgen => Afwegingsuitkomst::BelangWeegtOpMetWaarborgen,
            Uitkomstkeuze::WeegtNietOp => Afwegingsuitkomst::BelangWeegtNietOp,
        }
    }
}

pub fn draai(o: Liaopdracht, kluispad: Option<PathBuf>, nu: DateTime<Utc>) -> Result<()> {
    let pad = super::kluispad(kluispad)?;
    let mut kluis = super::open_kluis(&pad, nu)?;
    match o {
        Liaopdracht::Lijst => lijst(&kluis),
        Liaopdracht::Nieuw { kenmerk, omschrijving, verwerking } => {
            nieuw(&mut kluis, &kenmerk, &omschrijving, &verwerking, nu)
        }
        Liaopdracht::Toon { kenmerk } => toon(&kluis, &kenmerk),
        Liaopdracht::Vul { kenmerk, veld, waarde } => vul(&mut kluis, &kenmerk, &veld, &waarde, nu),
        Liaopdracht::Uitkomst { kenmerk, uitkomst, door, op } => {
            leg_uitkomst_vast(&mut kluis, &kenmerk, uitkomst.into(), &door, op.as_deref(), nu)
        }
    }
}

fn zoek(kluis: &Kluis, kenmerk: &str) -> Result<Belangenafweging> {
    let kop = kluis
        .lijst(SOORT)?
        .into_iter()
        .find(|r| r.kenmerk.as_deref() == Some(kenmerk))
        .ok_or_else(|| anyhow::anyhow!("geen belangenafweging met kenmerk '{kenmerk}'"))?;
    Ok(kluis.laad(SOORT, &kop.id)?)
}

fn bewaar(
    kluis: &mut Kluis,
    a: &Belangenafweging,
    handeling: Handeling,
    omschrijving: &str,
    nu: DateTime<Utc>,
) -> Result<()> {
    let actor = super::actor();
    kluis.bewaar(
        SOORT,
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

fn lees_tijdstip(tekst: &str) -> Result<DateTime<Utc>> {
    tekst.parse::<DateTime<chrono::FixedOffset>>().map(|t| t.with_timezone(&Utc)).map_err(|e| {
        anyhow::anyhow!(
            "kon '{tekst}' niet lezen als tijdstip ({e}). Gebruik de vorm 2026-08-19T09:00:00Z"
        )
    })
}

fn nieuw(
    kluis: &mut Kluis,
    kenmerk: &str,
    omschrijving: &str,
    verwerkingkenmerk: &str,
    nu: DateTime<Utc>,
) -> Result<()> {
    if kluis.lijst(SOORT)?.iter().any(|r| r.kenmerk.as_deref() == Some(kenmerk)) {
        anyhow::bail!("er bestaat al een belangenafweging met kenmerk '{kenmerk}'");
    }
    let kop_v = kluis
        .lijst("verwerking")?
        .into_iter()
        .find(|r| r.kenmerk.as_deref() == Some(verwerkingkenmerk))
        .ok_or_else(|| anyhow::anyhow!("geen registerregel met kenmerk '{verwerkingkenmerk}'"))?;
    let mut v: Verwerking = kluis.laad("verwerking", &kop_v.id)?;

    let a =
        Belangenafweging::nieuw(kenmerk, omschrijving, v.id, &v.kenmerk, &super::actor().id, nu);
    bewaar(kluis, &a, Handeling::RecordAangemaakt, "belangenafweging aangemaakt", nu)?;

    // De terugverwijzing in het register.
    if v.belangenafweging_id.is_none() {
        v.belangenafweging_id = Some(a.id);
        let actor = super::actor();
        kluis.bewaar(
            "verwerking",
            &v.id.to_string(),
            v.compartiment.naam(),
            v.status.omschrijving(),
            Some(&v.kenmerk),
            &v,
            &actor,
            Handeling::RecordGewijzigd,
            &format!("belangenafweging {kenmerk} gekoppeld"),
            nu,
        )?;
    }

    gelukt(&format!("belangenafweging {kenmerk} aangemaakt bij {verwerkingkenmerk}"));
    terzijde(
        "Artikel 6 lid 1 onder f is de enige grondslag die op een afweging rust in plaats van op \
         een feit. Wie haar niet opschrijft, heeft geen grondslag maar een bewering.",
    );
    toon_ontbrekend(&a);
    Ok(())
}

fn vul(
    kluis: &mut Kluis,
    kenmerk: &str,
    veld: &str,
    waarde: &str,
    nu: DateTime<Utc>,
) -> Result<()> {
    let mut a = zoek(kluis, kenmerk)?;
    let schoon = waarde.trim();
    if schoon.is_empty() {
        anyhow::bail!("'{veld}' mag niet leeg zijn");
    }
    let actor = super::actor();
    match veld {
        "belang" => a.gerechtvaardigd_belang = Some(schoon.to_string()),
        "noodzaak" => a.noodzakelijkheidstoets = Some(Motivering::nieuw(schoon, &actor.id, nu)?),
        "afweging" => a.afweging = Some(Motivering::nieuw(schoon, &actor.id, nu)?),
        "verwachtingen" => {
            a.redelijke_verwachtingen = Some(Motivering::nieuw(schoon, &actor.id, nu)?)
        }
        "waarborg" => a.waarborgen.push(schoon.to_string()),
        andere => anyhow::bail!(
            "'{andere}' is geen veld dat via deze route te vullen is. Beschikbaar: belang, \
             noodzaak, afweging, verwachtingen, waarborg"
        ),
    }
    a.herkomst.wijzig(format!("{veld} ingevuld"), nu);
    bewaar(kluis, &a, Handeling::RecordGewijzigd, &format!("{veld} ingevuld"), nu)?;

    gelukt(&format!("{veld} vastgelegd"));
    if veld == "noodzaak" {
        terzijde(
            "Kan het doel ook met minder gegevens of met een minder ingrijpend middel? Zo ja, \
             dan houdt de afweging hier op.",
        );
    }
    toon_ontbrekend(&a);
    Ok(())
}

fn leg_uitkomst_vast(
    kluis: &mut Kluis,
    kenmerk: &str,
    uitkomst: Afwegingsuitkomst,
    door: &str,
    op: Option<&str>,
    nu: DateTime<Utc>,
) -> Result<()> {
    let mut a = zoek(kluis, kenmerk)?;
    let moment = match op {
        Some(t) => lees_tijdstip(t)?,
        None => nu,
    };
    a.stel_uitkomst_vast(uitkomst, door, moment, nu)?;
    bewaar(
        kluis,
        &a,
        Handeling::BesluitGenomen,
        &format!("uitkomst: {}", uitkomst.omschrijving()),
        nu,
    )?;

    gelukt(&format!("uitkomst vastgelegd: {}", uitkomst.omschrijving()));
    if !uitkomst.draagt_de_grondslag() {
        let_op(
            "Deze verwerking kan niet op een gerechtvaardigd belang rusten. Kies een andere \
             grondslag of pas de verwerking aan; de afweging blijft staan als verantwoording van \
             dat besluit.",
        );
    }
    toon_ontbrekend(&a);
    Ok(())
}

fn lijst(kluis: &Kluis) -> Result<()> {
    let koppen = kluis.lijst(SOORT)?;
    if koppen.is_empty() {
        kop("Belangenafwegingen");
        terzijde("Er staan nog geen belangenafwegingen in de kluis.");
        return Ok(());
    }
    kop("Belangenafwegingen");
    let mut t = tabel(&["kenmerk", "registerregel", "uitkomst", "volledig"]);
    for k in &koppen {
        let a: Belangenafweging = kluis.laad(SOORT, &k.id)?;
        let r = a.volledigheid();
        t.add_row(vec![
            a.kenmerk.clone(),
            a.verwerking_kenmerk.clone(),
            a.uitkomst.map(|u| u.omschrijving().to_string()).unwrap_or_else(|| "—".into()),
            format!("{} van {}", r.compleet, r.verplicht),
        ]);
    }
    println!("{t}");
    Ok(())
}

fn toon(kluis: &Kluis, kenmerk: &str) -> Result<()> {
    let a = zoek(kluis, kenmerk)?;
    kop(&format!("Belangenafweging {}", a.kenmerk));
    let mut t = tabel(&["", ""]);
    t.add_row(vec!["omschrijving", &a.omschrijving]);
    t.add_row(vec!["registerregel", &a.verwerking_kenmerk]);
    t.add_row(vec!["status", a.status.omschrijving()]);
    if let Some(b) = &a.gerechtvaardigd_belang {
        t.add_row(vec!["belang", b]);
    }
    if let Some(u) = a.uitkomst {
        t.add_row(vec!["uitkomst", u.omschrijving()]);
    }
    if let Some(d) = &a.uitgevoerd_door {
        t.add_row(vec!["uitgevoerd door", d]);
    }
    println!("{t}");

    if !a.waarborgen.is_empty() {
        kop("Waarborgen");
        for w in &a.waarborgen {
            println!("  • {w}");
        }
        terzijde("Waarborgen kunnen de uitslag kantelen, maar zij vervangen de afweging niet.");
    }
    toon_ontbrekend(&a);
    Ok(())
}

fn toon_ontbrekend(a: &Belangenafweging) {
    let r = a.volledigheid();
    kop("Volledigheid");
    println!("  {}", voortgang(r.compleet, r.verplicht));
    if r.is_volledig() {
        println!();
        gelukt("alle verplichte onderdelen zijn ingevuld");
        return;
    }
    println!();
    for o in &r.ontbreekt {
        let veld = o.veld.trim_start_matches("lia.");
        if o.blokkeert_vaststelling {
            blokkade(&format!("{veld} — {}", o.omschrijving));
        } else {
            let_op(&format!("{veld} — {}", o.omschrijving));
        }
        terzijde(&o.grondslag);
    }
    println!();
    terzijde("■ houdt de uitkomst tegen · ▸ blijft zichtbaar maar blokkeert niet");
}
