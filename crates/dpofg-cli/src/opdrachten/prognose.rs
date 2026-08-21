//! De vervalprognose en de drie factoren van aantoonbaarheid.

use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use clap::Args;
use dpofg_audit::Handeling;
use dpofg_domain::{
    correctie::Correctie, dpia::Dpia, leverancier::Leverancier, risico::Risicobeoordeling,
    wpg::Wpgspoor, zorgplicht::Zorgplichtdossier,
};
use dpofg_report::{
    prognose::{
        aantoonbaarheid, bestuursstuk, prognose, Bronnen, Factortelling, Prognosetermijnen,
        Rapportcontext, Vervalpunt, BUITEN_BEELD,
    },
    Manifest, OndertekendManifest,
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
    /// Schrijf een ondertekend bestuursstuk naar deze map.
    #[arg(long)]
    pub export: Option<PathBuf>,
    /// Voor wie het bestuursstuk bestemd is. Verplicht bij --export.
    #[arg(long)]
    pub bestemd_voor: Option<String>,
    /// Waarvoor het stuk wordt samengesteld.
    #[arg(long, default_value = "periodieke bestuursrapportage")]
    pub aanleiding: String,
}

pub fn draai(o: Prognoseargumenten, kluispad: Option<PathBuf>, nu: DateTime<Utc>) -> Result<()> {
    let pad = super::kluispad(kluispad)?;
    let mut kluis = super::open_kluis(&pad, nu)?;

    let zorgplicht: Vec<Zorgplichtdossier> = super::laad(&kluis, "zorgplicht")?;
    if o.factoren {
        return toon_factoren(&zorgplicht, nu);
    }

    let beoordelingen: Vec<Risicobeoordeling> = super::laad(&kluis, "risico")?;
    let leveranciers: Vec<Leverancier> = super::laad(&kluis, "leverancier")?;
    let effectbeoordelingen: Vec<Dpia> = super::laad(&kluis, "dpia")?;
    let wpgsporen: Vec<Wpgspoor> = super::laad(&kluis, "wpg")?;

    let bronnen = Bronnen {
        zorgplicht: &zorgplicht,
        risicobeoordelingen: &beoordelingen,
        leveranciers: &leveranciers,
        effectbeoordelingen: &effectbeoordelingen,
        wpgsporen: &wpgsporen,
    };
    let termijnen = termijnen(nu);
    let horizonnen = if o.dagen.is_empty() { vec![30, 90, 365] } else { o.dagen.clone() };

    if let Some(map) = o.export.clone() {
        let bestemd_voor = o.bestemd_voor.clone().ok_or_else(|| {
            anyhow::anyhow!(
                "noem met --bestemd-voor voor wie dit stuk is. Een bestuursstuk zonder \
                 geadresseerde is later niet te herleiden tot de vergadering waarin het lag"
            )
        })?;
        return exporteer(
            &mut kluis,
            &map,
            &o.aanleiding,
            &bestemd_voor,
            &bronnen,
            &zorgplicht,
            termijnen,
            &horizonnen,
            nu,
        );
    }

    kop("Vervalprognose");
    terzijde(&format!(
        "peilmoment {} · {} dossiers doorzocht",
        crate::uitvoer::tijdstip(nu),
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
                crate::uitvoer::datum(v.vervalt_op).to_string(),
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
        kop(&format!("Over {dagen} dagen — {}", crate::uitvoer::datum(peildatum)));
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
                        crate::uitvoer::datum(v.vervalt_op).to_string(),
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

/// Schrijft het bestuursstuk met een ondertekend manifest.
///
/// Gebruikt hetzelfde manifest als `dpofg dossier`, en daarmee dezelfde
/// verificatiebinary. Een tweede bundelformaat zou een tweede verifier
/// vragen, en een verifier van de leverancier die het formaat van de
/// leverancier controleert, is geen onafhankelijke verificatie.
#[allow(clippy::too_many_arguments)]
fn exporteer(
    kluis: &mut Kluis,
    map: &std::path::Path,
    aanleiding: &str,
    bestemd_voor: &str,
    bronnen: &Bronnen<'_>,
    zorgplicht: &[Zorgplichtdossier],
    termijnen: Prognosetermijnen,
    horizonnen: &[i64],
    nu: DateTime<Utc>,
) -> Result<()> {
    std::fs::create_dir_all(map)?;
    let ketenrapport = kluis.verifieer_logboek()?;
    let pakket = dpofg_content::startpakket(nu.date_naive());
    let correcties: Vec<Correctie> = super::laad(kluis, "correctie")?;
    let lopend: Vec<&Correctie> = correcties.iter().filter(|c| !c.is_afgerond()).collect();

    let verstreken = prognose(bronnen, termijnen, nu);
    let per_horizon: Vec<(i64, Vec<Vervalpunt>)> = horizonnen
        .iter()
        .map(|d| (*d, prognose(bronnen, termijnen, nu + Duration::days(*d))))
        .collect();
    let factoren = aantoonbaarheid(zorgplicht, nu);

    let concepten = tel_concepten(kluis)?;
    let context = Rapportcontext {
        peilmoment: nu,
        samengesteld_door: super::actor().naam.clone(),
        bestemd_voor: bestemd_voor.to_string(),
        kennispakket: format!("{} {}", pakket.code, pakket.versienaam),
        consolidatiedatum: pakket.consolidatiedatum.to_string(),
        ketenreikwijdte: ketenrapport.reikwijdte(),
        keten_in_orde: ketenrapport.bevindingen.is_empty(),
        concepten,
        lopende_correcties: lopend.len(),
    };

    let mut manifest = Manifest::nieuw(
        aanleiding,
        bestemd_voor,
        &super::actor().naam,
        nu,
        kluis.ketenstand().volgnummer,
        &kluis.ketenstand().hash,
        ketenrapport.reikwijdte(),
        &pakket.code,
        &pakket.versienaam,
        pakket.consolidatiedatum,
    );

    let tekst = bestuursstuk(&context, &per_horizon, &verstreken, &factoren);
    schrijf(map, &mut manifest, "vervalprognose.md", "bestuursstuk", tekst.as_bytes())?;

    let punten = serde_json::to_vec_pretty(&serde_json::json!({
        "peilmoment": nu,
        "verstreken": verstreken,
        "per_horizon": per_horizon
            .iter()
            .map(|(d, p)| serde_json::json!({ "dagen": d, "punten": p }))
            .collect::<Vec<_>>(),
    }))?;
    schrijf(map, &mut manifest, "vervalprognose.json", "prognose", &punten)?;

    let f = serde_json::to_vec_pretty(&factoren)?;
    schrijf(map, &mut manifest, "factoren.json", "aantoonbaarheid", &f)?;

    let c = serde_json::to_vec_pretty(&lopend)?;
    schrijf(map, &mut manifest, "correcties.json", "correctie", &c)?;

    // Het logboek gaat mee: zonder logboek is de ketenstand in het manifest
    // niet na te rekenen.
    let logboek = serde_json::to_vec_pretty(&kluis.logboek()?)?;
    schrijf(map, &mut manifest, "logboek.json", "logboek", &logboek)?;
    if let Some(anker) = kluis.laatste_anker()? {
        let a = serde_json::to_vec_pretty(&anker)?;
        schrijf(map, &mut manifest, "anker.json", "anker", &a)?;
    }

    // Wat buiten beeld blijft, staat in het manifest én in het stuk zelf.
    for (wat, waarom) in BUITEN_BEELD {
        manifest.laat_weg(format!("verval van {wat}"), waarom, 0);
    }
    if concepten > 0 {
        manifest.laat_weg(
            "records met de status concept",
            "die zijn niet vastgesteld; zij tellen wel mee in de prognose, want een termijn \
             loopt ook over een concept, maar wat erin staat is niet vastgelegd",
            concepten,
        );
    }

    let ondertekend = kluis.onderteken_met(|s| OndertekendManifest::onderteken(manifest, s))?;
    std::fs::write(map.join("manifest.json"), serde_json::to_vec_pretty(&ondertekend)?)?;

    kluis.log(
        dpofg_audit::Gebeurtenis::nieuw(
            Handeling::DossierSamengesteld,
            super::actor(),
            nu,
            "prognose",
            map.display().to_string(),
            "algemeen",
            format!("vervalprognose samengesteld voor {bestemd_voor}"),
        ),
        Some(aanleiding.to_string()),
    )?;

    kop("Bestuursstuk samengesteld");
    let mut t = tabel(&["", ""]);
    let m = map.display().to_string();
    t.add_row(vec!["map", &m]);
    t.add_row(vec!["bestemd voor", bestemd_voor]);
    let aantal = verstreken.len().to_string();
    t.add_row(vec!["nu al niet aantoonbaar", &aantal]);
    let langste = per_horizon.last().map(|(_, p)| p.len()).unwrap_or(0).to_string();
    t.add_row(vec!["binnen de langste horizon", &langste]);
    println!("{t}");
    gelukt("het manifest is ondertekend met de installatiesleutel van deze kluis");
    terzijde("controleer de bundel met 'dpofg-verify dossier <map>'");
    if !context.keten_in_orde {
        blokkade(
            "de ketencontrole is niet zonder bevindingen doorlopen; dat staat ook in het stuk \
             zelf, want een bestuursstuk dat op een gebroken keten rust, hoort dat te zeggen",
        );
    }
    let_op(
        "De handtekening zegt dat deze bundel niet is gewijzigd sinds zij is samengesteld. Zij \
         zegt niets over de juistheid van de juridische inhoud waarop de prognose steunt.",
    );
    Ok(())
}

fn schrijf(
    map: &std::path::Path,
    manifest: &mut Manifest,
    naam: &str,
    soort: &str,
    inhoud: &[u8],
) -> Result<()> {
    std::fs::write(map.join(naam), inhoud)?;
    manifest.voeg_toe(naam, soort, naam, 1, inhoud);
    Ok(())
}

/// Hoeveel bronrecords er nog de status concept dragen.
fn tel_concepten(kluis: &Kluis) -> Result<usize> {
    let mut uit = 0;
    for soort in ["zorgplicht", "risico", "leverancier", "dpia", "wpg"] {
        uit += kluis.lijst(soort)?.iter().filter(|k| k.status == "concept").count();
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
