//! Het verwerkingsregister.

use anyhow::Result;
use chrono::{DateTime, Utc};
use clap::Subcommand;
use dpofg_audit::Handeling;
use dpofg_domain::{
    avg::{BijzondereCategorie, Grondslag, Rol},
    Motivering, Ontvanger, Verwerking, Volledig,
};
use dpofg_store::Kluis;
use std::path::PathBuf;

use crate::uitvoer::{blokkade, gelukt, kop, let_op, tabel, terzijde, voortgang};

const SOORT: &str = "verwerking";

#[derive(Subcommand, Debug)]
pub enum Registeropdracht {
    /// Toon alle registerregels.
    Lijst {
        /// Toon alleen regels die nog niet volledig zijn.
        #[arg(long)]
        onvolledig: bool,
    },
    /// Maak een nieuwe registerregel aan.
    Nieuw {
        /// Kenmerk waaronder de regel bekend staat.
        kenmerk: String,
        /// Naam van de verwerking.
        naam: String,
        /// De rol waarin wordt verwerkt.
        #[arg(long, value_enum, default_value = "verantwoordelijke")]
        rol: Rolkeuze,
        /// De afdeling of persoon die eigenaar is.
        #[arg(long)]
        eigenaar: String,
    },
    /// Toon één registerregel met wat er nog ontbreekt.
    Toon {
        /// Het kenmerk van de regel.
        kenmerk: String,
    },
    /// Vul een veld van een registerregel.
    Vul {
        /// Het kenmerk van de regel.
        kenmerk: String,
        /// Het veld dat wordt gevuld.
        #[arg(long)]
        veld: String,
        /// De waarde. Meerdere waarden worden gescheiden door een puntkomma.
        #[arg(long)]
        waarde: String,
    },
    /// Stel een registerregel vast.
    Vaststellen {
        /// Het kenmerk van de regel.
        kenmerk: String,
    },
}

#[derive(clap::ValueEnum, Clone, Copy, Debug)]
pub enum Rolkeuze {
    Verantwoordelijke,
    Verwerker,
    Gezamenlijk,
}

impl From<Rolkeuze> for Rol {
    fn from(k: Rolkeuze) -> Self {
        match k {
            Rolkeuze::Verantwoordelijke => Rol::Verwerkingsverantwoordelijke,
            Rolkeuze::Verwerker => Rol::Verwerker,
            Rolkeuze::Gezamenlijk => Rol::GezamenlijkVerantwoordelijke,
        }
    }
}

pub fn draai(o: Registeropdracht, kluispad: Option<PathBuf>, nu: DateTime<Utc>) -> Result<()> {
    let pad = super::kluispad(kluispad)?;
    let mut kluis = super::open_kluis(&pad, nu)?;
    match o {
        Registeropdracht::Lijst { onvolledig } => lijst(&kluis, onvolledig),
        Registeropdracht::Nieuw { kenmerk, naam, rol, eigenaar } => {
            nieuw(&mut kluis, &kenmerk, &naam, rol.into(), &eigenaar, nu)
        }
        Registeropdracht::Toon { kenmerk } => toon(&kluis, &kenmerk),
        Registeropdracht::Vul { kenmerk, veld, waarde } => {
            vul(&mut kluis, &kenmerk, &veld, &waarde, nu)
        }
        Registeropdracht::Vaststellen { kenmerk } => vaststellen(&mut kluis, &kenmerk, nu),
    }
}

/// Zoekt een verwerking op kenmerk.
fn zoek(kluis: &Kluis, kenmerk: &str) -> Result<Verwerking> {
    let kop = kluis
        .lijst(SOORT)?
        .into_iter()
        .find(|r| r.kenmerk.as_deref() == Some(kenmerk))
        .ok_or_else(|| {
            anyhow::anyhow!("geen registerregel met kenmerk '{kenmerk}'")
        })?;
    Ok(kluis.laad(SOORT, &kop.id)?)
}

fn lijst(kluis: &Kluis, alleen_onvolledig: bool) -> Result<()> {
    let koppen = kluis.lijst(SOORT)?;
    if koppen.is_empty() {
        kop("Verwerkingsregister");
        terzijde("Er staan nog geen registerregels in de kluis.");
        terzijde("Voeg er een toe met 'dpofg register nieuw <kenmerk> <naam> --eigenaar <naam>'.");
        return Ok(());
    }

    let mut rapporten = Vec::new();
    let mut t = tabel(&["kenmerk", "naam", "status", "volledigheid", "ontbreekt"]);
    for k in &koppen {
        let v: Verwerking = kluis.laad(SOORT, &k.id)?;
        let r = v.volledigheid();
        rapporten.push((v.status, r.clone()));

        if alleen_onvolledig && r.is_volledig() {
            continue;
        }
        let ontbreekt = if r.is_volledig() {
            "—".to_string()
        } else {
            r.ontbreekt
                .iter()
                .map(|o| o.veld.trim_start_matches("verwerking.").to_string())
                .collect::<Vec<_>>()
                .join(", ")
        };
        t.add_row(vec![
            v.kenmerk.clone(),
            v.naam.clone(),
            v.status.omschrijving().to_string(),
            voortgang(r.compleet, r.verplicht),
            ontbreekt,
        ]);
    }

    kop("Verwerkingsregister");
    println!("{t}");

    // De tellers waarmee zichtbaar wordt waar het structureel misgaat.
    let register = dpofg_domain::Registerrapport::uit("verwerkingsregister", &rapporten);
    println!();
    let mut s = tabel(&["", "aantal"]);
    s.add_row(vec!["registerregels".to_string(), register.totaal.to_string()]);
    s.add_row(vec!["vastgesteld".to_string(), register.vastgesteld.to_string()]);
    s.add_row(vec!["concept".to_string(), register.concept.to_string()]);
    s.add_row(vec!["volledig".to_string(), register.volledig.to_string()]);
    s.add_row(vec![
        "kan niet worden vastgesteld".to_string(),
        register.geblokkeerd.to_string(),
    ]);
    println!("{s}");

    if !register.ontbreekt_per_onderdeel.is_empty() {
        kop("Wat er het vaakst ontbreekt");
        let mut o = tabel(&["onderdeel", "aantal regels"]);
        for (veld, aantal) in register.ontbreekt_per_onderdeel.iter().take(10) {
            o.add_row(vec![veld.trim_start_matches("verwerking.").to_string(), aantal.to_string()]);
        }
        println!("{o}");
        terzijde("Begin bovenaan: daar levert één ingreep de meeste winst op.");
    }
    Ok(())
}

fn nieuw(
    kluis: &mut Kluis,
    kenmerk: &str,
    naam: &str,
    rol: Rol,
    eigenaar: &str,
    nu: DateTime<Utc>,
) -> Result<()> {
    if kluis.lijst(SOORT)?.iter().any(|r| r.kenmerk.as_deref() == Some(kenmerk)) {
        anyhow::bail!("er bestaat al een registerregel met kenmerk '{kenmerk}'");
    }
    let actor = super::actor();
    let v = Verwerking::nieuw(kenmerk, naam, rol, eigenaar, &actor.id, nu);
    let id = v.id.to_string();

    kluis.bewaar(
        SOORT,
        &id,
        v.compartiment.naam(),
        "concept",
        Some(kenmerk),
        &v,
        &actor,
        Handeling::RecordAangemaakt,
        &format!("registerregel '{naam}' aangemaakt"),
        nu,
    )?;

    gelukt(&format!("registerregel '{kenmerk}' aangemaakt als concept"));
    toon_ontbrekend(&v);
    Ok(())
}

fn toon(kluis: &Kluis, kenmerk: &str) -> Result<()> {
    let v = zoek(kluis, kenmerk)?;

    kop(&format!("{} — {}", v.kenmerk, v.naam));
    let mut t = tabel(&["", ""]);
    t.add_row(vec!["status", v.status.omschrijving()]);
    t.add_row(vec!["rol", v.rol.omschrijving()]);
    t.add_row(vec!["registerschema", v.rol.registerschema()]);
    t.add_row(vec!["eigenaar", &v.eigenaar]);
    t.add_row(vec!["compartiment", v.compartiment.naam()]);
    if !v.doeleinden.is_empty() {
        t.add_row(vec!["doeleinden", &v.doeleinden.join("; ")]);
    }
    if let Some(g) = v.grondslag {
        t.add_row(vec![
            "grondslag",
            &format!("{} — {}", g.grondslagverwijzing(), g.omschrijving()),
        ]);
    }
    if !v.categorieen_betrokkenen.is_empty() {
        t.add_row(vec!["betrokkenen", &v.categorieen_betrokkenen.join("; ")]);
    }
    if !v.categorieen_gegevens.is_empty() {
        t.add_row(vec!["gegevens", &v.categorieen_gegevens.join("; ")]);
    }
    if !v.bijzondere_categorieen.is_empty() {
        t.add_row(vec![
            "bijzondere gegevens",
            &v.bijzondere_categorieen
                .iter()
                .map(|c| c.omschrijving())
                .collect::<Vec<_>>()
                .join("; "),
        ]);
    }
    if !v.ontvangers.is_empty() {
        t.add_row(vec![
            "ontvangers",
            &v.ontvangers
                .iter()
                .map(|o| {
                    let mut s = o.omschrijving.clone();
                    if o.is_verwerker {
                        s.push_str(" (verwerker)");
                    }
                    if o.buiten_eer {
                        s.push_str(" (buiten de EER)");
                    }
                    s
                })
                .collect::<Vec<_>>()
                .join("; "),
        ]);
    }
    println!("{t}");

    let criteria = v.getelde_dpia_criteria();
    if !criteria.is_empty() {
        kop("Criteria voor een effectbeoordeling");
        for c in &criteria {
            println!("  • {c}");
        }
        if v.dpia_waarschijnlijk_verplicht() {
            println!();
            let_op(&format!(
                "{} van de criteria zijn geraakt. Bij twee of meer is een effectbeoordeling in \
                 beginsel verplicht (art. 35 lid 1 AVG). De tool telt de criteria die zij uit het \
                 register kan afleiden; het oordeel blijft aan u.",
                criteria.len()
            ));
        }
    }

    toon_ontbrekend(&v);
    Ok(())
}

fn toon_ontbrekend(v: &Verwerking) {
    let r = v.volledigheid();
    kop("Volledigheid");
    println!("  {}", voortgang(r.compleet, r.verplicht));
    if r.is_volledig() {
        println!();
        gelukt("alle verplichte onderdelen zijn ingevuld");
        return;
    }

    println!();
    for o in &r.ontbreekt {
        if o.blokkeert_vaststelling {
            blokkade(&format!("{} — {}", o.veld.trim_start_matches("verwerking."), o.omschrijving));
        } else {
            let_op(&format!("{} — {}", o.veld.trim_start_matches("verwerking."), o.omschrijving));
        }
        terzijde(&o.grondslag);
    }
    println!();
    terzijde("■ houdt vaststellen tegen · ▸ blijft zichtbaar maar blokkeert niet");
}

fn vul(
    kluis: &mut Kluis,
    kenmerk: &str,
    veld: &str,
    waarde: &str,
    nu: DateTime<Utc>,
) -> Result<()> {
    let mut v = zoek(kluis, kenmerk)?;
    let id = v.id.to_string();
    let lijst: Vec<String> =
        waarde.split(';').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();

    match veld {
        "doeleinden" => v.doeleinden = lijst,
        "betrokkenen" => v.categorieen_betrokkenen = lijst,
        "gegevens" => v.categorieen_gegevens = lijst,
        "beveiliging" => v.beveiligingsmaatregelen = Some(waarde.to_string()),
        "grondslag" => {
            v.grondslag = Some(match waarde {
                "toestemming" | "a" => Grondslag::Toestemming,
                "overeenkomst" | "b" => Grondslag::Overeenkomst,
                "wettelijke-verplichting" | "c" => Grondslag::WettelijkeVerplichting,
                "vitaal-belang" | "d" => Grondslag::VitaalBelang,
                "algemeen-belang" | "e" => Grondslag::AlgemeenBelang,
                "gerechtvaardigd-belang" | "f" => Grondslag::GerechtvaardigdBelang,
                andere => anyhow::bail!(
                    "'{andere}' is geen grondslag. Kies uit: toestemming, overeenkomst, \
                     wettelijke-verplichting, vitaal-belang, algemeen-belang, \
                     gerechtvaardigd-belang (of de letter a tot en met f)"
                ),
            })
        }
        "grondslag-motivering" => {
            v.grondslag_motivering = Some(Motivering::nieuw(waarde, &super::actor().id, nu)?)
        }
        "wettelijke-bepaling" => v.wettelijke_bepaling = Some(waarde.to_string()),
        "bewaartermijn" => {
            // Vorm: "7 jaar vanaf einde dienstverband | art. 52 AWR"
            let (termijn, grondslag) = waarde.split_once('|').ok_or_else(|| {
                anyhow::anyhow!(
                    "geef de bewaartermijn als '<duur> <eenheid> vanaf <gebeurtenis> | <grondslag>', \
                     bijvoorbeeld: '7 jaar vanaf einde dienstverband | art. 52 AWR'"
                )
            })?;
            let delen: Vec<&str> = termijn.split_whitespace().collect();
            if delen.len() < 4 || delen[2] != "vanaf" {
                anyhow::bail!(
                    "kon '{}' niet lezen; verwacht: '<duur> <eenheid> vanaf <gebeurtenis>'",
                    termijn.trim()
                );
            }
            let duur: u32 = delen[0].parse()?;
            let eenheid = match delen[1] {
                "dag" | "dagen" => dpofg_domain::Termijneenheid::Dagen,
                "maand" | "maanden" => dpofg_domain::Termijneenheid::Maanden,
                "jaar" | "jaren" => dpofg_domain::Termijneenheid::Jaren,
                andere => anyhow::bail!("'{andere}' is geen eenheid; kies dagen, maanden of jaren"),
            };
            v.bewaartermijn = Some(dpofg_domain::Bewaartermijn::Vast {
                duur,
                eenheid,
                grondslag: grondslag.trim().to_string(),
                vanaf: delen[3..].join(" "),
            });
        }
        "bijzondere-gegevens" => {
            v.bijzondere_categorieen = lijst
                .iter()
                .map(|s| match s.as_str() {
                    "gezondheid" => Ok(BijzondereCategorie::Gezondheidsgegevens),
                    "ras" => Ok(BijzondereCategorie::RasOfEtnischeAfkomst),
                    "politiek" => Ok(BijzondereCategorie::PolitiekeOpvattingen),
                    "religie" => Ok(BijzondereCategorie::ReligieuzeOfLevensbeschouwelijkeOvertuigingen),
                    "vakbond" => Ok(BijzondereCategorie::Vakbondslidmaatschap),
                    "genetisch" => Ok(BijzondereCategorie::GenetischeGegevens),
                    "biometrisch" => Ok(BijzondereCategorie::BiometrischeGegevensVoorIdentificatie),
                    "seksueel" => Ok(BijzondereCategorie::SeksueelGedragOfGerichtheid),
                    andere => Err(anyhow::anyhow!(
                        "'{andere}' is geen bijzondere categorie; kies uit gezondheid, ras, \
                         politiek, religie, vakbond, genetisch, biometrisch, seksueel"
                    )),
                })
                .collect::<Result<Vec<_>>>()?;
        }
        "ontvanger" => {
            // Vorm: "naam" of "naam:verwerker" of "naam:verwerker,buiten-eer"
            let (naam, kenmerken) = waarde.split_once(':').unwrap_or((waarde, ""));
            v.ontvangers.push(Ontvanger {
                omschrijving: naam.trim().to_string(),
                is_verwerker: kenmerken.contains("verwerker"),
                leverancier_id: None,
                buiten_eer: kenmerken.contains("buiten-eer"),
            });
        }
        "bsn" => {
            v.burgerservicenummer = waarde == "ja";
        }
        "bsn-grondslag" => v.bsn_grondslag = Some(waarde.to_string()),
        andere => anyhow::bail!(
            "'{andere}' is geen veld dat via deze route te vullen is. Beschikbaar: doeleinden, \
             betrokkenen, gegevens, grondslag, grondslag-motivering, wettelijke-bepaling, \
             bewaartermijn, beveiliging, bijzondere-gegevens, ontvanger, bsn, bsn-grondslag"
        ),
    }

    v.herkomst.wijzig(&super::actor().id, nu);
    let actor = super::actor();
    kluis.bewaar(
        SOORT,
        &id,
        v.compartiment.naam(),
        if v.status == dpofg_domain::Status::Vastgesteld { "vastgesteld" } else { "concept" },
        Some(&v.kenmerk),
        &v,
        &actor,
        Handeling::RecordGewijzigd,
        &format!("veld '{veld}' gevuld"),
        nu,
    )?;

    gelukt(&format!("'{veld}' vastgelegd"));
    toon_ontbrekend(&v);
    Ok(())
}

fn vaststellen(kluis: &mut Kluis, kenmerk: &str, nu: DateTime<Utc>) -> Result<()> {
    let mut v = zoek(kluis, kenmerk)?;
    let id = v.id.to_string();
    let actor = super::actor();

    match v.stel_vast(&actor.id, nu) {
        Ok(()) => {
            kluis.bewaar(
                SOORT,
                &id,
                v.compartiment.naam(),
                "vastgesteld",
                Some(&v.kenmerk),
                &v,
                &actor,
                Handeling::RecordVastgesteld,
                &format!("registerregel '{}' vastgesteld", v.naam),
                nu,
            )?;
            gelukt(&format!("registerregel '{kenmerk}' vastgesteld"));
            let r = v.volledigheid();
            if !r.is_volledig() {
                println!();
                let_op(&format!(
                    "Er staan nog {} onderdelen open die vaststellen niet tegenhouden. \
                     Zij blijven zichtbaar in het register en in elke export.",
                    r.ontbreekt.len()
                ));
            }
            Ok(())
        }
        Err(fout) => {
            // De blokkade landt in het logboek: zo is later te zien hoeveel
            // fouten het ontwerp heeft tegengehouden.
            kluis.log(
                dpofg_audit::Gebeurtenis::nieuw(
                    Handeling::ControleGeblokkeerd,
                    actor,
                    nu,
                    SOORT,
                    &id,
                    v.compartiment.naam(),
                    "vaststellen geweigerd: verplichte onderdelen ontbreken",
                ),
                Some(fout.to_string()),
            )?;
            kop("Vaststellen is niet gelukt");
            toon_ontbrekend(&v);
            anyhow::bail!("de registerregel is nog niet volledig")
        }
    }
}
