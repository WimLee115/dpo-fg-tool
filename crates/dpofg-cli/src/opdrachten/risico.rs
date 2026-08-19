//! De risicobeoordeling waarop de zorgplichtmaatregelen steunen.

use anyhow::Result;
use chrono::{DateTime, Utc};
use clap::Subcommand;
use dpofg_audit::Handeling;
use dpofg_domain::{
    risico::{Inschatting, Risicobeoordeling},
    Motivering, Volledig,
};
use dpofg_store::Kluis;
use std::path::PathBuf;

use crate::uitvoer::{blokkade, gelukt, kop, let_op, tabel, terzijde, voortgang};

const SOORT: &str = "risico";
const COMPARTIMENT: &str = "algemeen";

#[derive(Subcommand, Debug)]
pub enum Risicoopdracht {
    /// Toon alle risicobeoordelingen.
    Lijst,
    /// Begin een risicobeoordeling.
    Nieuw {
        /// Kenmerk van de beoordeling.
        kenmerk: String,
        /// Waarover de beoordeling gaat.
        #[arg(long)]
        reikwijdte: String,
        /// De gebruikte methode.
        #[arg(long)]
        methode: String,
        /// Waar die methode vandaan komt.
        #[arg(long, default_value = "eigen methodebeschrijving")]
        methode_bron: String,
        /// Wie de beoordeling heeft uitgevoerd.
        #[arg(long)]
        uitgevoerd_door: String,
        /// Wanneer. Standaard: nu.
        #[arg(long)]
        uitgevoerd_op: Option<String>,
        /// Tot wanneer de beoordeling geldt.
        #[arg(long)]
        geldig_tot: String,
    },
    /// Toon één beoordeling.
    Toon {
        /// Kenmerk van de beoordeling.
        kenmerk: String,
    },
    /// Leg vast welke bron is geraadpleegd.
    Bron {
        /// Kenmerk van de beoordeling.
        kenmerk: String,
        /// De aanduiding van de bron.
        #[arg(long)]
        aanduiding: String,
        /// Wat voor bron het is.
        #[arg(long, default_value = "publicatie")]
        soort: String,
        /// Wanneer hij is geraadpleegd. Standaard: nu.
        #[arg(long)]
        op: Option<String>,
    },
    /// Onderken een risico.
    Onderken {
        /// Kenmerk van de beoordeling.
        kenmerk: String,
        /// De code van het risico.
        #[arg(long)]
        code: String,
        /// Wat het risico is.
        #[arg(long)]
        omschrijving: String,
        /// Waardoor het zich kan verwezenlijken.
        #[arg(long)]
        oorzaak: String,
        /// Wat er dan gebeurt.
        #[arg(long)]
        gevolg: String,
        /// De ingeschatte waarschijnlijkheid.
        #[arg(long, value_enum)]
        waarschijnlijkheid: Schaal,
        /// De ingeschatte impact.
        #[arg(long, value_enum)]
        impact: Schaal,
    },
    /// Leg vast welke maatregelen een risico verkleinen, en wat er overblijft.
    Verklein {
        /// Kenmerk van de beoordeling.
        kenmerk: String,
        /// De code van het risico.
        #[arg(long)]
        code: String,
        /// Een maatregel, als aanduiding of maatregelcode. Herhaalbaar.
        #[arg(long = "maatregel")]
        maatregelen: Vec<String>,
        /// De waarschijnlijkheid die overblijft.
        #[arg(long, value_enum)]
        restwaarschijnlijkheid: Schaal,
        /// De impact die overblijft.
        #[arg(long, value_enum)]
        restimpact: Schaal,
    },
    /// Leg vast wie het restrisico aanvaardt.
    Aanvaarden {
        /// Kenmerk van de beoordeling.
        kenmerk: String,
        /// De code van het risico.
        #[arg(long)]
        code: String,
        /// De naam.
        #[arg(long)]
        door: String,
        /// De functie.
        #[arg(long)]
        functie: String,
        /// Degene die aanvaardt behoort tot het bestuur.
        #[arg(long)]
        bestuurder: bool,
        /// De onderbouwing.
        #[arg(long)]
        motivering: String,
    },
    /// Stel de beoordeling vast.
    Vaststellen {
        /// Kenmerk van de beoordeling.
        kenmerk: String,
    },
}

#[derive(clap::ValueEnum, Clone, Copy, Debug)]
pub enum Schaal {
    ZeerLaag,
    Laag,
    Gemiddeld,
    Hoog,
    ZeerHoog,
}

impl From<Schaal> for Inschatting {
    fn from(s: Schaal) -> Self {
        match s {
            Schaal::ZeerLaag => Inschatting::ZeerLaag,
            Schaal::Laag => Inschatting::Laag,
            Schaal::Gemiddeld => Inschatting::Gemiddeld,
            Schaal::Hoog => Inschatting::Hoog,
            Schaal::ZeerHoog => Inschatting::ZeerHoog,
        }
    }
}

pub fn draai(o: Risicoopdracht, kluispad: Option<PathBuf>, nu: DateTime<Utc>) -> Result<()> {
    let pad = super::kluispad(kluispad)?;
    let mut kluis = super::open_kluis(&pad, nu)?;
    match o {
        Risicoopdracht::Lijst => lijst(&kluis, nu),
        Risicoopdracht::Nieuw {
            kenmerk,
            reikwijdte,
            methode,
            methode_bron,
            uitgevoerd_door,
            uitgevoerd_op,
            geldig_tot,
        } => nieuw(
            &mut kluis,
            &kenmerk,
            &reikwijdte,
            &methode,
            &methode_bron,
            &uitgevoerd_door,
            uitgevoerd_op.as_deref(),
            &geldig_tot,
            nu,
        ),
        Risicoopdracht::Toon { kenmerk } => toon(&kluis, &kenmerk, nu),
        Risicoopdracht::Bron { kenmerk, aanduiding, soort, op } => {
            bron(&mut kluis, &kenmerk, &aanduiding, &soort, op.as_deref(), nu)
        }
        Risicoopdracht::Onderken {
            kenmerk,
            code,
            omschrijving,
            oorzaak,
            gevolg,
            waarschijnlijkheid,
            impact,
        } => onderken(
            &mut kluis,
            &kenmerk,
            &code,
            &omschrijving,
            &oorzaak,
            &gevolg,
            waarschijnlijkheid.into(),
            impact.into(),
            nu,
        ),
        Risicoopdracht::Verklein {
            kenmerk,
            code,
            maatregelen,
            restwaarschijnlijkheid,
            restimpact,
        } => verklein(
            &mut kluis,
            &kenmerk,
            &code,
            maatregelen,
            restwaarschijnlijkheid.into(),
            restimpact.into(),
            nu,
        ),
        Risicoopdracht::Aanvaarden { kenmerk, code, door, functie, bestuurder, motivering } => {
            aanvaarden(&mut kluis, &kenmerk, &code, &door, &functie, bestuurder, &motivering, nu)
        }
        Risicoopdracht::Vaststellen { kenmerk } => vaststellen(&mut kluis, &kenmerk, nu),
    }
}

pub fn zoek(kluis: &Kluis, kenmerk: &str) -> Result<Risicobeoordeling> {
    let kop = kluis
        .lijst(SOORT)?
        .into_iter()
        .find(|r| r.kenmerk.as_deref() == Some(kenmerk))
        .ok_or_else(|| anyhow::anyhow!("geen risicobeoordeling met kenmerk '{kenmerk}'"))?;
    Ok(kluis.laad(SOORT, &kop.id)?)
}

fn bewaar(
    kluis: &mut Kluis,
    b: &Risicobeoordeling,
    handeling: Handeling,
    omschrijving: &str,
    nu: DateTime<Utc>,
) -> Result<()> {
    let actor = super::actor();
    kluis.bewaar(
        SOORT,
        &b.id.to_string(),
        COMPARTIMENT,
        b.status.omschrijving(),
        Some(&b.kenmerk),
        b,
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

#[allow(clippy::too_many_arguments)]
fn nieuw(
    kluis: &mut Kluis,
    kenmerk: &str,
    reikwijdte: &str,
    methode: &str,
    methode_bron: &str,
    uitgevoerd_door: &str,
    uitgevoerd_op: Option<&str>,
    geldig_tot: &str,
    nu: DateTime<Utc>,
) -> Result<()> {
    if kluis.lijst(SOORT)?.iter().any(|r| r.kenmerk.as_deref() == Some(kenmerk)) {
        anyhow::bail!("er bestaat al een risicobeoordeling met kenmerk '{kenmerk}'");
    }
    let uitgevoerd = match uitgevoerd_op {
        Some(t) => lees_tijdstip(t)?,
        None => nu,
    };
    let tot = lees_tijdstip(geldig_tot)?;
    let b = Risicobeoordeling::nieuw(
        kenmerk,
        reikwijdte,
        methode,
        methode_bron,
        uitgevoerd_door,
        uitgevoerd,
        tot,
        &super::actor().id,
        nu,
    )?;
    bewaar(kluis, &b, Handeling::RecordAangemaakt, "risicobeoordeling begonnen", nu)?;

    gelukt(&format!("risicobeoordeling {kenmerk} begonnen"));
    terzijde(&format!("geldig tot {}", crate::uitvoer::datum(tot)));
    terzijde(
        "Er komt geen score uit deze beoordeling. Wat eruit komt is: welke risico's zijn \
         onderkend, welke maatregelen ze verkleinen, wat er overblijft en wie dat aanvaardt.",
    );
    toon_ontbrekend(&b);
    Ok(())
}

fn bron(
    kluis: &mut Kluis,
    kenmerk: &str,
    aanduiding: &str,
    soort: &str,
    op: Option<&str>,
    nu: DateTime<Utc>,
) -> Result<()> {
    let mut b = zoek(kluis, kenmerk)?;
    let moment = match op {
        Some(t) => lees_tijdstip(t)?,
        None => nu,
    };
    b.raadpleeg_bron(aanduiding, soort, moment, nu)?;
    bewaar(kluis, &b, Handeling::RecordGewijzigd, "bron geraadpleegd", nu)?;

    gelukt(&format!("{aanduiding} is als geraadpleegde bron vastgelegd"));
    terzijde(&format!("{} bron(nen) in deze beoordeling", b.bronnen.len()));
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn onderken(
    kluis: &mut Kluis,
    kenmerk: &str,
    code: &str,
    omschrijving: &str,
    oorzaak: &str,
    gevolg: &str,
    waarschijnlijkheid: Inschatting,
    impact: Inschatting,
    nu: DateTime<Utc>,
) -> Result<()> {
    let mut b = zoek(kluis, kenmerk)?;
    b.onderken(code, omschrijving, oorzaak, gevolg, waarschijnlijkheid, impact, nu)?;
    bewaar(kluis, &b, Handeling::RecordGewijzigd, &format!("risico {code} onderkend"), nu)?;

    let r = b.risico(code).expect("zojuist toegevoegd");
    gelukt(&format!("{code} onderkend, klasse {}", r.brutoklasse().omschrijving()));
    terzijde(
        "Het restrisico staat nu gelijk aan het risico. Lager wordt het alleen met een \
         maatregel erbij: een verlaging zonder maatregel is geen beoordeling maar een aanname.",
    );
    if r.vraagt_bestuur() {
        let_op(
            "Blijft dit restrisico hoog, dan aanvaardt het bestuur het, en niet iemand anders \
             namens het bestuur.",
        );
    }
    toon_ontbrekend(&b);
    Ok(())
}

fn verklein(
    kluis: &mut Kluis,
    kenmerk: &str,
    code: &str,
    maatregelen: Vec<String>,
    restwaarschijnlijkheid: Inschatting,
    restimpact: Inschatting,
    nu: DateTime<Utc>,
) -> Result<()> {
    let mut b = zoek(kluis, kenmerk)?;
    let was_aanvaard = b.risico(code).is_some_and(|r| r.aanvaarding.is_some());
    b.verklein(code, maatregelen, restwaarschijnlijkheid, restimpact, nu)?;
    bewaar(kluis, &b, Handeling::RecordGewijzigd, &format!("risico {code} verkleind"), nu)?;

    let r = b.risico(code).expect("zojuist gewijzigd");
    gelukt(&format!(
        "{code}: van {} naar {}",
        r.brutoklasse().omschrijving(),
        r.restklasse().omschrijving()
    ));
    if was_aanvaard {
        let_op(
            "De eerdere aanvaarding is vervallen: die ging over het restrisico zoals dat toen \
             was. Laat het gewijzigde restrisico opnieuw aanvaarden.",
        );
    }
    if r.vraagt_bestuur() {
        let_op("Dit restrisico is hoog; aanvaarding door het bestuur is nodig.");
    }
    toon_ontbrekend(&b);
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn aanvaarden(
    kluis: &mut Kluis,
    kenmerk: &str,
    code: &str,
    door: &str,
    functie: &str,
    bestuurder: bool,
    motivering: &str,
    nu: DateTime<Utc>,
) -> Result<()> {
    let mut b = zoek(kluis, kenmerk)?;
    let m = Motivering::nieuw(motivering, &super::actor().id, nu)?;
    b.aanvaard_restrisico(code, door, functie, bestuurder, m, nu)?;
    bewaar(kluis, &b, Handeling::BesluitGenomen, &format!("restrisico {code} aanvaard"), nu)?;

    gelukt(&format!("{door} ({functie}) aanvaardt het restrisico van {code}"));
    terzijde(
        "De tool dwingt af dát dit besluit wordt genomen en door wie; de afweging zelf laat \
         zij aan de organisatie.",
    );
    toon_ontbrekend(&b);
    Ok(())
}

fn vaststellen(kluis: &mut Kluis, kenmerk: &str, nu: DateTime<Utc>) -> Result<()> {
    let mut b = zoek(kluis, kenmerk)?;
    let actor = super::actor();
    let id = b.id.to_string();
    match b.stel_vast(&actor.naam, nu) {
        Ok(()) => {
            bewaar(kluis, &b, Handeling::RecordVastgesteld, "risicobeoordeling vastgesteld", nu)?;
            gelukt(&format!("risicobeoordeling {kenmerk} vastgesteld"));
            toon_klassen(&b);
            toon_ontbrekend(&b);
            Ok(())
        }
        Err(fout) => {
            kluis.log(
                dpofg_audit::Gebeurtenis::nieuw(
                    Handeling::ControleGeblokkeerd,
                    actor,
                    nu,
                    SOORT,
                    &id,
                    b.compartiment.naam(),
                    "vaststellen geweigerd: verplichte onderdelen ontbreken",
                ),
                Some(fout.to_string()),
            )?;
            kop("Vaststellen is niet gelukt");
            toon_ontbrekend(&b);
            anyhow::bail!("{fout}")
        }
    }
}

fn lijst(kluis: &Kluis, nu: DateTime<Utc>) -> Result<()> {
    let koppen = kluis.lijst(SOORT)?;
    kop("Risicobeoordelingen");
    if koppen.is_empty() {
        terzijde("Er staat nog geen risicobeoordeling in de kluis.");
        return Ok(());
    }
    let mut t = tabel(&["kenmerk", "reikwijdte", "status", "risico's", "geldig tot"]);
    for k in &koppen {
        let b: Risicobeoordeling = kluis.laad(SOORT, &k.id)?;
        let geldig = if b.is_verlopen(nu) {
            format!("{} (verlopen)", crate::uitvoer::datum(b.geldig_tot))
        } else {
            crate::uitvoer::datum(b.geldig_tot).to_string()
        };
        t.add_row(vec![
            b.kenmerk.clone(),
            b.reikwijdte.clone(),
            b.status.omschrijving().to_string(),
            b.risicos.len().to_string(),
            geldig,
        ]);
    }
    println!("{t}");
    Ok(())
}

fn toon(kluis: &Kluis, kenmerk: &str, nu: DateTime<Utc>) -> Result<()> {
    let b = zoek(kluis, kenmerk)?;
    kop(&format!("Risicobeoordeling {}", b.kenmerk));
    let mut t = tabel(&["", ""]);
    t.add_row(vec!["reikwijdte", &b.reikwijdte]);
    t.add_row(vec!["methode", &b.methode]);
    t.add_row(vec!["bron van de methode", &b.methode_bron]);
    t.add_row(vec!["uitgevoerd door", &b.uitgevoerd_door]);
    let uitgevoerd = crate::uitvoer::datum(b.uitgevoerd_op).to_string();
    t.add_row(vec!["uitgevoerd op", &uitgevoerd]);
    let tot = crate::uitvoer::datum(b.geldig_tot).to_string();
    t.add_row(vec!["geldig tot", &tot]);
    t.add_row(vec!["status", b.status.omschrijving()]);
    println!("{t}");
    if b.is_verlopen(nu) {
        blokkade("deze beoordeling is verlopen; de maatregelen eronder steunen op een beeld dat niet meer is getoetst");
    }

    if !b.bronnen.is_empty() {
        kop("Geraadpleegde bronnen");
        let mut t = tabel(&["aanduiding", "soort", "geraadpleegd op"]);
        for br in &b.bronnen {
            t.add_row(vec![
                br.aanduiding.clone(),
                br.soort.clone(),
                crate::uitvoer::datum(br.geraadpleegd_op).to_string(),
            ]);
        }
        println!("{t}");
    }

    kop("Onderkende risico's");
    if b.risicos.is_empty() {
        terzijde("nog geen");
    } else {
        let mut t = tabel(&[
            "code",
            "risico",
            "zonder maatregelen",
            "maatregelen",
            "restrisico",
            "aanvaard door",
        ]);
        for r in &b.risicos {
            t.add_row(vec![
                r.code.clone(),
                r.omschrijving.clone(),
                format!(
                    "{} / {} → {}",
                    r.waarschijnlijkheid.omschrijving(),
                    r.impact.omschrijving(),
                    r.brutoklasse().omschrijving()
                ),
                if r.maatregelen.is_empty() { "geen".into() } else { r.maatregelen.join(", ") },
                format!(
                    "{} / {} → {}",
                    r.restwaarschijnlijkheid.omschrijving(),
                    r.restimpact.omschrijving(),
                    r.restklasse().omschrijving()
                ),
                r.aanvaarding
                    .as_ref()
                    .map(|a| format!("{} ({})", a.door, a.functie))
                    .unwrap_or_else(|| "niemand".into()),
            ]);
        }
        println!("{t}");
        toon_klassen(&b);
    }

    toon_ontbrekend(&b);
    Ok(())
}

fn toon_klassen(b: &Risicobeoordeling) {
    kop("Wat er overblijft");
    let mut t = tabel(&["klasse", "aantal"]);
    for (klasse, aantal) in b.restklassen() {
        t.add_row(vec![klasse.omschrijving().to_string(), aantal.to_string()]);
    }
    println!("{t}");
    terzijde(
        "Twee inschattingen op een vijfpuntsschaal leveren geen getal op. De klasse dient één \
         doel: bepalen wie het restrisico mag aanvaarden.",
    );
}

fn toon_ontbrekend(b: &Risicobeoordeling) {
    let r = b.volledigheid();
    kop("Volledigheid");
    println!("  {}", voortgang(r.compleet, r.verplicht));
    if r.is_volledig() {
        println!();
        gelukt("alle verplichte onderdelen zijn ingevuld");
        return;
    }
    println!();
    for o in r.ontbreekt.iter().take(8) {
        let veld = o.veld.trim_start_matches("risico.");
        if o.blokkeert_vaststelling {
            blokkade(&format!("{veld} — {}", o.omschrijving));
        } else {
            let_op(&format!("{veld} — {}", o.omschrijving));
        }
        terzijde(&o.grondslag);
    }
    if r.ontbreekt.len() > 8 {
        terzijde(&format!("nog {} onderdelen, zie 'risico toon'", r.ontbreekt.len() - 8));
    }
    println!();
    terzijde("■ houdt vaststellen tegen · ▸ blijft zichtbaar maar blokkeert niet");
}
