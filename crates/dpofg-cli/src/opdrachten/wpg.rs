//! Het spoor van de Wet politiegegevens.

use anyhow::Result;
use chrono::{DateTime, Utc};
use clap::Subcommand;
use dpofg_audit::Handeling;
use dpofg_domain::{
    wpg::{Controle, Maatregel, Wpgspoor},
    Motivering, Volledig,
};
use dpofg_store::Kluis;
use std::path::PathBuf;

use crate::uitvoer::{blokkade, gelukt, kop, let_op, tabel, terzijde, voortgang};

const SOORT: &str = "wpg";
const COMPARTIMENT: &str = "vertrouwelijk";

#[derive(Subcommand, Debug)]
pub enum Wpgopdracht {
    /// Toon de Wpg-sporen met de stand van de cyclus.
    Lijst,
    /// Maak een Wpg-spoor aan.
    Nieuw {
        /// Kenmerk van het spoor.
        kenmerk: String,
        /// Waar het over gaat.
        omschrijving: String,
    },
    /// Toon één spoor.
    Toon {
        /// Kenmerk van het spoor.
        kenmerk: String,
    },
    /// Beoordeel of het regime van toepassing is.
    Toepasselijkheid {
        /// Kenmerk van het spoor.
        kenmerk: String,
        /// Geldt het regime?
        #[arg(long)]
        van_toepassing: bool,
        /// Waarom wel of niet. Verplicht: ook een ontkennend antwoord is een
        /// standpunt dat te verantwoorden moet zijn.
        #[arg(long)]
        motivering: String,
    },
    /// Leg een uitgevoerde controle of audit vast.
    Controle {
        /// Kenmerk van het spoor.
        kenmerk: String,
        /// Wanneer de controle is uitgevoerd.
        uitgevoerd_op: String,
        /// Wie hem uitvoerde.
        #[arg(long)]
        door: String,
        /// Het betreft de vierjaarlijkse externe audit.
        #[arg(long)]
        extern_uitgevoerd: bool,
        /// Hoeveel bevindingen de controle opleverde.
        #[arg(long, default_value = "0")]
        bevindingen: usize,
        /// Het kenmerk waaronder het rapport is opgeborgen.
        #[arg(long)]
        rapport: Option<String>,
    },
    /// Stel het verbeterplan vast.
    Verbeterplan {
        /// Kenmerk van het spoor.
        kenmerk: String,
        /// Wie het plan vaststelde.
        #[arg(long)]
        door: String,
        /// Een maatregel in de vorm "omschrijving | eigenaar | 2027-01-31".
        /// Herhaalbaar.
        #[arg(long = "maatregel", required = true)]
        maatregelen: Vec<String>,
    },
    /// Rond één maatregel af.
    Maatregel {
        /// Kenmerk van het spoor.
        kenmerk: String,
        /// De omschrijving van de maatregel.
        #[arg(long)]
        omschrijving: String,
        /// Wanneer hij is afgerond. Standaard: nu.
        #[arg(long)]
        op: Option<String>,
    },
}

pub fn draai(o: Wpgopdracht, kluispad: Option<PathBuf>, nu: DateTime<Utc>) -> Result<()> {
    let pad = super::kluispad(kluispad)?;
    let mut kluis = super::open_kluis(&pad, nu)?;
    match o {
        Wpgopdracht::Lijst => lijst(&kluis, nu),
        Wpgopdracht::Nieuw { kenmerk, omschrijving } => {
            nieuw(&mut kluis, &kenmerk, &omschrijving, nu)
        }
        Wpgopdracht::Toon { kenmerk } => toon(&kluis, &kenmerk, nu),
        Wpgopdracht::Toepasselijkheid { kenmerk, van_toepassing, motivering } => {
            toepasselijkheid(&mut kluis, &kenmerk, van_toepassing, &motivering, nu)
        }
        Wpgopdracht::Controle {
            kenmerk,
            uitgevoerd_op,
            door,
            extern_uitgevoerd,
            bevindingen,
            rapport,
        } => controle(
            &mut kluis,
            &kenmerk,
            &uitgevoerd_op,
            &door,
            extern_uitgevoerd,
            bevindingen,
            rapport,
            nu,
        ),
        Wpgopdracht::Verbeterplan { kenmerk, door, maatregelen } => {
            verbeterplan(&mut kluis, &kenmerk, &door, &maatregelen, nu)
        }
        Wpgopdracht::Maatregel { kenmerk, omschrijving, op } => {
            maatregel_af(&mut kluis, &kenmerk, &omschrijving, op.as_deref(), nu)
        }
    }
}

fn zoek(kluis: &Kluis, kenmerk: &str) -> Result<Wpgspoor> {
    let kop = kluis
        .lijst(SOORT)?
        .into_iter()
        .find(|r| r.kenmerk.as_deref() == Some(kenmerk))
        .ok_or_else(|| anyhow::anyhow!("geen Wpg-spoor met kenmerk '{kenmerk}'"))?;
    Ok(kluis.laad(SOORT, &kop.id)?)
}

fn bewaar(
    kluis: &mut Kluis,
    s: &Wpgspoor,
    handeling: Handeling,
    omschrijving: &str,
    nu: DateTime<Utc>,
) -> Result<()> {
    let actor = super::actor();
    kluis.bewaar(
        SOORT,
        &s.id.to_string(),
        COMPARTIMENT,
        s.status.omschrijving(),
        Some(&s.kenmerk),
        s,
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

/// Leest een datum in de vorm 2027-01-31 als het einde van die dag.
fn lees_datum(tekst: &str) -> Result<DateTime<Utc>> {
    let datum: chrono::NaiveDate = tekst.trim().parse().map_err(|e| {
        anyhow::anyhow!("kon '{tekst}' niet lezen als datum ({e}). Gebruik de vorm 2027-01-31")
    })?;
    Ok(datum.and_hms_opt(23, 59, 59).expect("geldige tijd").and_utc())
}

fn drempels(nu: DateTime<Utc>) -> (i64, i64) {
    let pakket = dpofg_content::startpakket(nu.date_naive());
    let audit = pakket.termijn("WPG-EXTERNE-AUDIT").map(|t| i64::from(t.duur)).unwrap_or(48);
    let controle = pakket.termijn("WPG-INTERNE-CONTROLE").map(|t| i64::from(t.duur)).unwrap_or(12);
    (audit, controle)
}

fn nieuw(kluis: &mut Kluis, kenmerk: &str, omschrijving: &str, nu: DateTime<Utc>) -> Result<()> {
    if kluis.lijst(SOORT)?.iter().any(|r| r.kenmerk.as_deref() == Some(kenmerk)) {
        anyhow::bail!("er bestaat al een Wpg-spoor met kenmerk '{kenmerk}'");
    }
    let s = Wpgspoor::nieuw(kenmerk, omschrijving, &super::actor().id, nu);
    bewaar(kluis, &s, Handeling::RecordAangemaakt, "Wpg-spoor aangemaakt", nu)?;
    gelukt(&format!("Wpg-spoor {kenmerk} aangemaakt"));
    terzijde(
        "De kern van dit regime is een cyclus die vanzelf doortikt: jaarlijks intern controleren, \
         vierjaarlijks extern laten auditen. Wie hem niet bijhoudt, ontdekt bij de eerstvolgende \
         audit dat de vorige vier jaar geleden was.",
    );
    toon_ontbrekend(&s, nu);
    Ok(())
}

fn toepasselijkheid(
    kluis: &mut Kluis,
    kenmerk: &str,
    van_toepassing: bool,
    motivering: &str,
    nu: DateTime<Utc>,
) -> Result<()> {
    let mut s = zoek(kluis, kenmerk)?;
    let m = Motivering::nieuw(motivering, &super::actor().id, nu)?;
    s.stel_toepasselijkheid_vast(van_toepassing, m, nu)?;
    bewaar(kluis, &s, Handeling::MotiveringVastgelegd, "toepasselijkheid vastgesteld", nu)?;
    gelukt(if van_toepassing {
        "het regime is van toepassing"
    } else {
        "het regime is niet van toepassing"
    });
    if !van_toepassing {
        terzijde("De motivering blijft staan en gaat mee in elke export: een ontkennend antwoord hoort even goed te verantwoorden te zijn.");
    }
    toon_ontbrekend(&s, nu);
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn controle(
    kluis: &mut Kluis,
    kenmerk: &str,
    uitgevoerd_op: &str,
    door: &str,
    extern_uitgevoerd: bool,
    bevindingen: usize,
    rapport: Option<String>,
    nu: DateTime<Utc>,
) -> Result<()> {
    let mut s = zoek(kluis, kenmerk)?;
    let moment = lees_tijdstip(uitgevoerd_op)?;
    s.leg_controle_vast(
        extern_uitgevoerd,
        Controle {
            uitgevoerd_op: moment,
            uitvoerder: door.to_string(),
            rapport_kenmerk: rapport,
            bevindingen,
            toelichting: None,
        },
        nu,
    )?;
    let soort = if extern_uitgevoerd { "externe audit" } else { "interne controle" };
    bewaar(kluis, &s, Handeling::KetenGeverifieerd, soort, nu)?;

    gelukt(&format!("{soort} vastgelegd op {}", moment.format("%d-%m-%Y")));
    if bevindingen > 0 {
        let_op(&format!(
            "{bevindingen} bevinding(en). Een rapport opbergen zonder plan is de meest \
             voorkomende manier waarop een audit geen gevolg krijgt; stel een verbeterplan vast \
             met een eigenaar en een einddatum per maatregel."
        ));
    }
    toon_ontbrekend(&s, nu);
    Ok(())
}

fn verbeterplan(
    kluis: &mut Kluis,
    kenmerk: &str,
    door: &str,
    ruw: &[String],
    nu: DateTime<Utc>,
) -> Result<()> {
    let mut s = zoek(kluis, kenmerk)?;
    let mut maatregelen = Vec::new();
    for regel in ruw {
        let delen: Vec<&str> = regel.split('|').map(|d| d.trim()).collect();
        if delen.len() != 3 {
            anyhow::bail!(
                "geef een maatregel als '<omschrijving> | <eigenaar> | <datum>', bijvoorbeeld: \
                 'logging aanzetten | de teamleider | 2027-01-31'. Gekregen: '{regel}'"
            );
        }
        maatregelen.push(Maatregel {
            omschrijving: delen[0].to_string(),
            eigenaar: delen[1].to_string(),
            gereed_uiterlijk: lees_datum(delen[2])?,
            afgerond_op: None,
        });
    }
    s.stel_verbeterplan_vast(door, maatregelen, nu, nu)?;
    bewaar(kluis, &s, Handeling::BesluitGenomen, "verbeterplan vastgesteld", nu)?;

    gelukt("verbeterplan vastgesteld");
    let plan = s.verbeterplan.as_ref().expect("zojuist gezet");
    let mut t = tabel(&["maatregel", "eigenaar", "gereed uiterlijk"]);
    for m in &plan.maatregelen {
        t.add_row(vec![
            m.omschrijving.clone(),
            m.eigenaar.clone(),
            m.gereed_uiterlijk.format("%d-%m-%Y").to_string(),
        ]);
    }
    println!("{t}");
    toon_ontbrekend(&s, nu);
    Ok(())
}

fn maatregel_af(
    kluis: &mut Kluis,
    kenmerk: &str,
    omschrijving: &str,
    op: Option<&str>,
    nu: DateTime<Utc>,
) -> Result<()> {
    let mut s = zoek(kluis, kenmerk)?;
    let moment = match op {
        Some(t) => lees_tijdstip(t)?,
        None => nu,
    };
    s.rond_maatregel_af(omschrijving, moment, nu)?;
    bewaar(
        kluis,
        &s,
        Handeling::RecordGewijzigd,
        &format!("maatregel '{omschrijving}' afgerond"),
        nu,
    )?;
    gelukt(&format!("'{omschrijving}' afgerond"));
    let openstaand = s.verbeterplan.as_ref().map(|p| p.openstaand()).unwrap_or(0);
    terzijde(&format!("{openstaand} maatregel(en) staan nog open"));
    toon_ontbrekend(&s, nu);
    Ok(())
}

fn lijst(kluis: &Kluis, nu: DateTime<Utc>) -> Result<()> {
    let koppen = kluis.lijst(SOORT)?;
    if koppen.is_empty() {
        kop("Wpg-sporen");
        terzijde("Er staan nog geen Wpg-sporen in de kluis.");
        return Ok(());
    }
    let (audit_drempel, controle_drempel) = drempels(nu);
    kop("Wpg-sporen");
    let mut t =
        tabel(&["kenmerk", "van toepassing", "laatste controle", "laatste audit", "openstaand"]);
    for k in &koppen {
        let s: Wpgspoor = kluis.laad(SOORT, &k.id)?;
        let controle = s
            .maanden_sinds_controle(nu)
            .map(|m| format!("{m} maanden geleden"))
            .unwrap_or_else(|| "geen".into());
        let audit = s
            .maanden_sinds_audit(nu)
            .map(|m| format!("{m} maanden geleden"))
            .unwrap_or_else(|| "geen".into());
        t.add_row(vec![
            s.kenmerk.clone(),
            match s.van_toepassing {
                Some(true) => "ja",
                Some(false) => "nee",
                None => "—",
            }
            .to_string(),
            controle,
            audit,
            s.verbeterplan
                .as_ref()
                .map(|p| p.openstaand().to_string())
                .unwrap_or_else(|| "—".into()),
        ]);
    }
    println!("{t}");
    terzijde(&format!("norm: audit elke {audit_drempel} maanden, interne controle elke {controle_drempel} maanden"));
    Ok(())
}

fn toon(kluis: &Kluis, kenmerk: &str, nu: DateTime<Utc>) -> Result<()> {
    let s = zoek(kluis, kenmerk)?;
    let (audit_drempel, controle_drempel) = drempels(nu);

    kop(&format!("Wpg-spoor {}", s.kenmerk));
    let mut t = tabel(&["", ""]);
    t.add_row(vec!["omschrijving", &s.omschrijving]);
    t.add_row(vec!["status", s.status.omschrijving()]);
    t.add_row(vec![
        "van toepassing",
        match s.van_toepassing {
            Some(true) => "ja",
            Some(false) => "nee",
            None => "nog niet beoordeeld",
        },
    ]);
    println!("{t}");

    if !s.interne_controles.is_empty() || !s.externe_audits.is_empty() {
        kop("Cyclus");
        let mut t = tabel(&["soort", "uitgevoerd op", "door", "bevindingen"]);
        for (soort, c) in s
            .interne_controles
            .iter()
            .map(|c| ("intern", c))
            .chain(s.externe_audits.iter().map(|c| ("extern", c)))
        {
            t.add_row(vec![
                soort.to_string(),
                c.uitgevoerd_op.format("%d-%m-%Y").to_string(),
                c.uitvoerder.clone(),
                c.bevindingen.to_string(),
            ]);
        }
        println!("{t}");

        if let Some(m) = s.maanden_sinds_audit(nu) {
            if m >= audit_drempel {
                blokkade(&format!(
                    "de laatste externe audit was {m} maanden geleden; de norm is {audit_drempel}"
                ));
                terzijde("art. 33 lid 3 Wet politiegegevens");
            }
        }
        if let Some(m) = s.maanden_sinds_controle(nu) {
            if m >= controle_drempel {
                let_op(&format!("de laatste interne controle was {m} maanden geleden; de norm is {controle_drempel}"));
                terzijde("art. 33 lid 1 Wet politiegegevens");
            }
        }
    }

    if let Some(plan) = &s.verbeterplan {
        kop("Verbeterplan");
        let mut t = tabel(&["maatregel", "eigenaar", "gereed uiterlijk", "stand"]);
        for m in &plan.maatregelen {
            let stand = match m.afgerond_op {
                Some(d) => format!("afgerond {}", d.format("%d-%m-%Y")),
                None if m.is_verlopen(nu) => "verlopen".to_string(),
                None => "open".to_string(),
            };
            t.add_row(vec![
                m.omschrijving.clone(),
                m.eigenaar.clone(),
                m.gereed_uiterlijk.format("%d-%m-%Y").to_string(),
                stand,
            ]);
        }
        println!("{t}");
        for m in plan.verlopen(nu) {
            blokkade(&format!("'{}' is over de einddatum en niet afgerond", m.omschrijving));
        }
    }

    toon_ontbrekend(&s, nu);
    Ok(())
}

fn toon_ontbrekend(s: &Wpgspoor, _nu: DateTime<Utc>) {
    let r = s.volledigheid();
    kop("Volledigheid");
    println!("  {}", voortgang(r.compleet, r.verplicht));
    if r.is_volledig() {
        println!();
        gelukt("alle verplichte onderdelen zijn ingevuld");
        return;
    }
    println!();
    for o in &r.ontbreekt {
        let veld = o.veld.trim_start_matches("wpg.");
        if o.blokkeert_vaststelling {
            blokkade(&format!("{veld} — {}", o.omschrijving));
        } else {
            let_op(&format!("{veld} — {}", o.omschrijving));
        }
        terzijde(&o.grondslag);
    }
    println!();
    terzijde("■ houdt vaststellen tegen · ▸ blijft zichtbaar maar blokkeert niet");
}
