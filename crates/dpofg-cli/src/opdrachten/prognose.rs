//! De vervalprognose en de drie factoren van aantoonbaarheid.

use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use clap::Args;
use dpofg_domain::{
    dpia::Dpia, leverancier::Leverancier, risico::Risicobeoordeling, wpg::Wpgspoor,
    zorgplicht::Zorgplichtdossier,
};
use dpofg_report::prognose::{
    aantoonbaarheid, prognose, Bronnen, Factortelling, Prognosetermijnen,
};
use dpofg_store::Kluis;
use std::path::PathBuf;

use crate::uitvoer::{blokkade, gelukt, kop, let_op, tabel, terzijde, voortgang};

#[derive(Args, Debug)]
pub struct Prognoseargumenten {
    /// De horizon in dagen. Herhaalbaar; standaard 30, 90 en 365.
    #[arg(long = "dagen", value_parser = clap::value_parser!(i64).range(1..=3650))]
    pub dagen: Vec<i64>,
    /// Toon per horizon elke eis, niet alleen de telling.
    #[arg(long)]
    pub uitgebreid: bool,
    /// Toon in plaats daarvan de drie factoren per eis.
    #[arg(long)]
    pub factoren: bool,
}

pub fn draai(o: Prognoseargumenten, kluispad: Option<PathBuf>, nu: DateTime<Utc>) -> Result<()> {
    let pad = super::kluispad(kluispad)?;
    let kluis = super::open_kluis(&pad, nu)?;

    let zorgplicht: Vec<Zorgplichtdossier> = laad(&kluis, "zorgplicht")?;
    if o.factoren {
        return toon_factoren(&zorgplicht, nu);
    }

    let beoordelingen: Vec<Risicobeoordeling> = laad(&kluis, "risico")?;
    let leveranciers: Vec<Leverancier> = laad(&kluis, "leverancier")?;
    let effectbeoordelingen: Vec<Dpia> = laad(&kluis, "dpia")?;
    let wpgsporen: Vec<Wpgspoor> = laad(&kluis, "wpg")?;

    let bronnen = Bronnen {
        zorgplicht: &zorgplicht,
        risicobeoordelingen: &beoordelingen,
        leveranciers: &leveranciers,
        effectbeoordelingen: &effectbeoordelingen,
        wpgsporen: &wpgsporen,
    };
    let termijnen = termijnen(nu);
    let horizonnen = if o.dagen.is_empty() { vec![30, 90, 365] } else { o.dagen.clone() };

    kop("Vervalprognose");
    terzijde(&format!(
        "peilmoment {} · {} dossiers doorzocht",
        nu.format("%d-%m-%Y %H:%M UTC"),
        zorgplicht.len()
            + beoordelingen.len()
            + leveranciers.len()
            + effectbeoordelingen.len()
            + wpgsporen.len()
    ));

    // Wat vandaag al niet aantoonbaar is, staat apart: dat is geen prognose
    // maar een stand van zaken, en de twee door elkaar halen laat een
    // achterstand als een aankomende gebeurtenis lezen.
    let verstreken = prognose(&bronnen, termijnen, nu);
    if !verstreken.is_empty() {
        kop("Vandaag al niet aantoonbaar");
        let mut t = tabel(&["eis", "oorzaak", "sinds", "eigenaar"]);
        for v in &verstreken {
            t.add_row(vec![
                v.eis.clone(),
                v.oorzaak.omschrijving().to_string(),
                v.vervalt_op.format("%d-%m-%Y").to_string(),
                v.eigenaar.clone().unwrap_or_else(|| "geen".into()),
            ]);
        }
        println!("{t}");
        blokkade(&format!("{} eis(en) zijn nu al niet te bewijzen", verstreken.len()));
    }

    let mut vorig = verstreken.len();
    for dagen in &horizonnen {
        let peildatum = nu + Duration::days(*dagen);
        let punten = prognose(&bronnen, termijnen, peildatum);
        let erbij = punten.len().saturating_sub(vorig);
        kop(&format!("Over {dagen} dagen — {}", peildatum.format("%d-%m-%Y")));
        if punten.is_empty() {
            gelukt("er verloopt niets binnen deze horizon");
        } else {
            terzijde(&format!(
                "{} eis(en) niet aantoonbaar, waarvan {erbij} erbij ten opzichte van de vorige \
                 horizon",
                punten.len()
            ));
            if o.uitgebreid {
                let mut t = tabel(&["eis", "oorzaak", "vervalt op", "over", "eigenaar"]);
                for v in punten.iter().filter(|v| !v.is_verstreken(nu)) {
                    t.add_row(vec![
                        v.eis.clone(),
                        v.oorzaak.omschrijving().to_string(),
                        v.vervalt_op.format("%d-%m-%Y").to_string(),
                        format!("{} dagen", v.dagen_tot_verval(nu)),
                        v.eigenaar.clone().unwrap_or_else(|| "geen".into()),
                    ]);
                }
                println!("{t}");
            } else {
                let mut t = tabel(&["oorzaak", "aantal"]);
                for oorzaak in oorzaken(&punten) {
                    let aantal = punten.iter().filter(|v| v.oorzaak == oorzaak).count();
                    t.add_row(vec![oorzaak.omschrijving().to_string(), aantal.to_string()]);
                }
                println!("{t}");
            }
        }
        vorig = punten.len();
    }

    if !o.uitgebreid {
        terzijde("gebruik --uitgebreid om per eis te zien wat er wanneer verloopt");
    }
    terzijde(
        "Dit is een lijst met eisen die onbewijsbaar worden, geen takenlijst en geen score. \
         Een bestuur weegt een informatiebeveiligingsrisico als datum, niet als kleur.",
    );
    let_op(
        "Deze prognose overziet niet alles: doorgifte-instrumenten kennen in dit model geen \
         einddatum maar een status, en certificaten, mandaten en mappingreviews bestaan nog \
         niet als eigen record. Wat hier niet staat, is daarmee niet in orde bevonden.",
    );
    Ok(())
}

fn laad<T: serde::de::DeserializeOwned>(kluis: &Kluis, soort: &str) -> Result<Vec<T>> {
    let mut uit = Vec::new();
    for k in kluis.lijst(soort)? {
        uit.push(kluis.laad(soort, &k.id)?);
    }
    Ok(uit)
}

fn oorzaken(
    punten: &[dpofg_report::prognose::Vervalpunt],
) -> Vec<dpofg_report::prognose::Vervaloorzaak> {
    let mut uit: Vec<_> = punten.iter().map(|v| v.oorzaak).collect();
    uit.sort_unstable();
    uit.dedup();
    uit
}

fn toon_factoren(dossiers: &[Zorgplichtdossier], nu: DateTime<Utc>) -> Result<()> {
    let regels = aantoonbaarheid(dossiers, nu);
    kop("De drie factoren van aantoonbaarheid");
    if regels.is_empty() {
        terzijde("er is nog geen ingerichte maatregel om te wegen");
        return Ok(());
    }

    let telling = Factortelling::van(&regels);
    let mut t = tabel(&["factor", "wat de vraag is", "aantal"]);
    t.add_row(vec![
        "vastgesteld".to_string(),
        "ligt er een besluit of beleidsstuk dat de eis vastlegt".to_string(),
        format!("{} van de {}", telling.vastgesteld, telling.totaal),
    ]);
    t.add_row(vec![
        "uitgevoerd".to_string(),
        "ligt er bewijs van de uitvoering dat nu geldt".to_string(),
        format!("{} van de {}", telling.uitgevoerd, telling.totaal),
    ]);
    t.add_row(vec![
        "actueel".to_string(),
        "is die uitvoering recent genoeg voor de eigen termijn".to_string(),
        format!("{} van de {}", telling.actueel, telling.totaal),
    ]);
    println!("{t}");
    println!();
    println!("  vastgesteld  {}", voortgang(telling.vastgesteld, telling.totaal));
    println!("  uitgevoerd   {}", voortgang(telling.uitgevoerd, telling.totaal));
    println!("  actueel      {}", voortgang(telling.actueel, telling.totaal));
    println!();

    kop("Per eis");
    let mut t = tabel(&["dossier", "eis", "vastgesteld", "uitgevoerd", "actueel", "eigenaar"]);
    for r in &regels {
        let vink = |ja: bool| if ja { "ja" } else { "nee" }.to_string();
        t.add_row(vec![
            r.record_kenmerk.clone(),
            r.onderdeel.clone(),
            vink(r.vastgesteld),
            vink(r.uitgevoerd),
            vink(r.actueel),
            r.eigenaar.clone().unwrap_or_else(|| "geen".into()),
        ]);
    }
    println!("{t}");

    terzijde(
        "Drie tellingen en geen score. Het plan noemt hier een driefactorscore maar geeft geen \
         schaal, geen weging en geen aggregatieregel; een getal dat niet zegt waarop het is \
         gebaseerd, gaat in een bestuursstuk een eigen leven leiden.",
    );
    Ok(())
}

/// De termijnen waarmee de prognose rekent, uit het kennispakket.
fn termijnen(nu: DateTime<Utc>) -> Prognosetermijnen {
    let pakket = dpofg_content::startpakket(nu.date_naive());
    let maanden = |code: &str, terugval: u32| {
        pakket
            .termijn(code)
            .ok()
            .filter(|t| t.eenheid == dpofg_terms::Eenheid::Maanden)
            .map(|t| t.duur)
            .unwrap_or(terugval)
    };
    Prognosetermijnen {
        bestuursvaststelling_maanden: maanden("INTERN-ZORGPLICHT-BESTUURSVASTSTELLING", 12),
        subverwerkerscontrole_maanden: maanden("INTERN-SUBVERWERKERSCONTROLE", 12),
        effectbeoordeling_maanden: maanden("INTERN-DPIA-HERBEOORDELING", 36),
        wpg_audit_maanden: maanden("WPG-EXTERNE-AUDIT", 48),
        wpg_controle_maanden: maanden("WPG-INTERNE-CONTROLE", 12),
    }
}
