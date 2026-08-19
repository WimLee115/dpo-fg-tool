//! Redactieregie.
//!
//! De tool redigeert niet zelf. Zij wijst aan wát er weg moet, levert uit aan
//! een aangewezen extern hulpmiddel, en controleert het teruggeleverde bestand
//! voor zover zij dat eerlijk kan. Wat zij niet kan controleren, meldt zij als
//! niet gecontroleerd — en dat houdt de verstrekking tegen.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use clap::Subcommand;
use dpofg_audit::Handeling;
use dpofg_domain::{
    redactie::{
        zoek_in_bytes, Controlesoort, Controleuitkomst, Redactiecategorie, Redactieopdracht,
        Terugleescontrole,
    },
    Volledig,
};
use dpofg_store::Kluis;
use std::path::PathBuf;

use crate::uitvoer::{blokkade, gelukt, kop, let_op, tabel, terzijde, voortgang};

const SOORT: &str = "redactie";
const COMPARTIMENT: &str = "vertrouwelijk";

#[derive(Subcommand, Debug)]
pub enum Redactieopdrachtkeuze {
    /// Toon alle redactieopdrachten.
    Lijst,
    /// Maak een redactieopdracht bij een dossier.
    Nieuw {
        /// Kenmerk van de opdracht.
        kenmerk: String,
        /// Korte omschrijving.
        omschrijving: String,
        /// De soort van het dossier: verzoek of woo.
        #[arg(long, default_value = "verzoek")]
        dossier_soort: String,
        /// Het kenmerk van het dossier.
        #[arg(long)]
        dossier: String,
    },
    /// Toon één opdracht met de stand van de controles.
    Toon {
        /// Kenmerk van de opdracht.
        kenmerk: String,
    },
    /// Leg vast wat er uit de stukken moet verdwijnen.
    Profiel {
        /// Kenmerk van de opdracht.
        kenmerk: String,
        /// De categorie.
        #[arg(long, value_enum)]
        categorie: Categoriekeuze,
        /// De letterlijke waarde die weg moet. Verplicht bij een tekstuele
        /// categorie: daarop wordt straks gezocht.
        #[arg(long)]
        waarde: Option<String>,
        /// Waar het om gaat, in gewone taal.
        #[arg(long)]
        omschrijving: String,
    },
    /// Neem een stuk op in de opdracht.
    Stuk {
        /// Kenmerk van de opdracht.
        kenmerk: String,
        /// Het pad naar het stuk zoals het nu is.
        bestand: PathBuf,
    },
    /// Lever de stukken uit aan het externe redactiehulpmiddel.
    Uitleveren {
        /// Kenmerk van de opdracht.
        kenmerk: String,
        /// Welk hulpmiddel.
        #[arg(long)]
        hulpmiddel: String,
    },
    /// Neem een geredigeerd stuk terug en controleer de tekstlaag.
    Terugnemen {
        /// Kenmerk van de opdracht.
        kenmerk: String,
        /// De naam waaronder het stuk in de opdracht staat.
        #[arg(long)]
        stuk: String,
        /// Het pad naar het teruggeleverde bestand.
        bestand: PathBuf,
    },
    /// Leg een controle vast die buiten de tool is uitgevoerd.
    Controle {
        /// Kenmerk van de opdracht.
        kenmerk: String,
        /// De naam van het stuk.
        #[arg(long)]
        stuk: String,
        /// Welke controle.
        #[arg(long, value_enum)]
        soort: Controlesoortkeuze,
        /// De uitkomst.
        #[arg(long, value_enum)]
        uitkomst: Uitkomstkeuze,
        /// Bij een handmatige goedkeuring: de tweede persoon.
        #[arg(long)]
        tweede_persoon: Option<String>,
        /// Toelichting.
        #[arg(long)]
        toelichting: Option<String>,
    },
    /// Verstrek de stukken. Wordt geweigerd zolang een controle openstaat.
    Verstrekken {
        /// Kenmerk van de opdracht.
        kenmerk: String,
        /// Aan wie.
        #[arg(long)]
        aan: String,
    },
}

#[derive(clap::ValueEnum, Clone, Copy, Debug)]
pub enum Categoriekeuze {
    Bsn,
    Naam,
    Adres,
    Contactgegevens,
    Gezondheid,
    Strafrechtelijk,
    Financieel,
    Handtekening,
    Beeldmateriaal,
    Overig,
}

impl From<Categoriekeuze> for Redactiecategorie {
    fn from(k: Categoriekeuze) -> Self {
        match k {
            Categoriekeuze::Bsn => Redactiecategorie::Burgerservicenummer,
            Categoriekeuze::Naam => Redactiecategorie::Naam,
            Categoriekeuze::Adres => Redactiecategorie::Adres,
            Categoriekeuze::Contactgegevens => Redactiecategorie::Contactgegevens,
            Categoriekeuze::Gezondheid => Redactiecategorie::Gezondheidsgegevens,
            Categoriekeuze::Strafrechtelijk => Redactiecategorie::StrafrechtelijkeGegevens,
            Categoriekeuze::Financieel => Redactiecategorie::FinancieleGegevens,
            Categoriekeuze::Handtekening => Redactiecategorie::Handtekening,
            Categoriekeuze::Beeldmateriaal => Redactiecategorie::Beeldmateriaal,
            Categoriekeuze::Overig => Redactiecategorie::Overig,
        }
    }
}

#[derive(clap::ValueEnum, Clone, Copy, Debug)]
pub enum Controlesoortkeuze {
    Tekstlaag,
    Metagegevens,
    Beeldvergelijking,
    Handmatig,
}

impl From<Controlesoortkeuze> for Controlesoort {
    fn from(k: Controlesoortkeuze) -> Self {
        match k {
            Controlesoortkeuze::Tekstlaag => Controlesoort::Tekstlaag,
            Controlesoortkeuze::Metagegevens => Controlesoort::Metagegevens,
            Controlesoortkeuze::Beeldvergelijking => Controlesoort::Beeldvergelijking,
            Controlesoortkeuze::Handmatig => Controlesoort::Handmatig,
        }
    }
}

#[derive(clap::ValueEnum, Clone, Copy, Debug)]
pub enum Uitkomstkeuze {
    Geslaagd,
    Gefaald,
}

impl From<Uitkomstkeuze> for Controleuitkomst {
    fn from(k: Uitkomstkeuze) -> Self {
        match k {
            Uitkomstkeuze::Geslaagd => Controleuitkomst::Geslaagd,
            Uitkomstkeuze::Gefaald => Controleuitkomst::Gefaald,
        }
    }
}

pub fn draai(o: Redactieopdrachtkeuze, kluispad: Option<PathBuf>, nu: DateTime<Utc>) -> Result<()> {
    let pad = super::kluispad(kluispad)?;
    let mut kluis = super::open_kluis(&pad, nu)?;
    match o {
        Redactieopdrachtkeuze::Lijst => lijst(&kluis),
        Redactieopdrachtkeuze::Nieuw { kenmerk, omschrijving, dossier_soort, dossier } => {
            nieuw(&mut kluis, &kenmerk, &omschrijving, &dossier_soort, &dossier, nu)
        }
        Redactieopdrachtkeuze::Toon { kenmerk } => toon(&kluis, &kenmerk),
        Redactieopdrachtkeuze::Profiel { kenmerk, categorie, waarde, omschrijving } => {
            profiel(&mut kluis, &kenmerk, categorie.into(), waarde, &omschrijving, nu)
        }
        Redactieopdrachtkeuze::Stuk { kenmerk, bestand } => {
            stuk(&mut kluis, &kenmerk, &bestand, nu)
        }
        Redactieopdrachtkeuze::Uitleveren { kenmerk, hulpmiddel } => {
            uitleveren(&mut kluis, &kenmerk, &hulpmiddel, nu)
        }
        Redactieopdrachtkeuze::Terugnemen { kenmerk, stuk, bestand } => {
            terugnemen(&mut kluis, &kenmerk, &stuk, &bestand, nu)
        }
        Redactieopdrachtkeuze::Controle {
            kenmerk,
            stuk,
            soort,
            uitkomst,
            tweede_persoon,
            toelichting,
        } => controle(
            &mut kluis,
            &kenmerk,
            &stuk,
            soort.into(),
            uitkomst.into(),
            tweede_persoon,
            toelichting,
            nu,
        ),
        Redactieopdrachtkeuze::Verstrekken { kenmerk, aan } => {
            verstrekken(&mut kluis, &kenmerk, &aan, nu)
        }
    }
}

fn zoek(kluis: &Kluis, kenmerk: &str) -> Result<Redactieopdracht> {
    let kop = kluis
        .lijst(SOORT)?
        .into_iter()
        .find(|r| r.kenmerk.as_deref() == Some(kenmerk))
        .ok_or_else(|| anyhow::anyhow!("geen redactieopdracht met kenmerk '{kenmerk}'"))?;
    Ok(kluis.laad(SOORT, &kop.id)?)
}

fn bewaar(
    kluis: &mut Kluis,
    o: &Redactieopdracht,
    handeling: Handeling,
    omschrijving: &str,
    nu: DateTime<Utc>,
) -> Result<()> {
    let actor = super::actor();
    kluis.bewaar(
        SOORT,
        &o.id.to_string(),
        COMPARTIMENT,
        o.status.omschrijving(),
        Some(&o.kenmerk),
        o,
        &actor,
        handeling,
        omschrijving,
        nu,
    )?;
    Ok(())
}

fn hash_van(pad: &std::path::Path) -> Result<(String, Vec<u8>)> {
    let bytes = std::fs::read(pad).with_context(|| format!("kon {} niet lezen", pad.display()))?;
    Ok((blake3::hash(&bytes).to_hex().to_string(), bytes))
}

fn nieuw(
    kluis: &mut Kluis,
    kenmerk: &str,
    omschrijving: &str,
    dossier_soort: &str,
    dossier: &str,
    nu: DateTime<Utc>,
) -> Result<()> {
    if kluis.lijst(SOORT)?.iter().any(|r| r.kenmerk.as_deref() == Some(kenmerk)) {
        anyhow::bail!("er bestaat al een redactieopdracht met kenmerk '{kenmerk}'");
    }
    if !matches!(dossier_soort, "verzoek" | "woo") {
        anyhow::bail!("'{dossier_soort}' is geen dossiersoort. Kies uit: verzoek, woo");
    }
    if !kluis.lijst(dossier_soort)?.iter().any(|r| r.kenmerk.as_deref() == Some(dossier)) {
        anyhow::bail!("geen {dossier_soort} met kenmerk '{dossier}'");
    }

    let o = Redactieopdracht::nieuw(
        kenmerk,
        omschrijving,
        dossier_soort,
        dossier,
        &super::actor().id,
        nu,
    );
    bewaar(kluis, &o, Handeling::RecordAangemaakt, "redactieopdracht aangemaakt", nu)?;

    gelukt(&format!("redactieopdracht {kenmerk} aangemaakt bij {dossier}"));
    terzijde(
        "De tool redigeert niet zelf. Zij wijst aan wát er weg moet, levert uit aan een \
         aangewezen hulpmiddel, en controleert wat er terugkomt. Het zelf bouwen van een \
         redactiepijplijn zou de meest waarschijnlijke oorzaak van een datalek dóór de tool in \
         eigen beheer nemen.",
    );
    toon_ontbrekend(&o);
    Ok(())
}

fn profiel(
    kluis: &mut Kluis,
    kenmerk: &str,
    categorie: Redactiecategorie,
    waarde: Option<String>,
    omschrijving: &str,
    nu: DateTime<Utc>,
) -> Result<()> {
    let mut o = zoek(kluis, kenmerk)?;
    o.voeg_toe_aan_profiel(categorie, waarde, omschrijving, nu)?;
    bewaar(kluis, &o, Handeling::RecordGewijzigd, "profiel aangevuld", nu)?;

    gelukt(&format!("{} toegevoegd aan het profiel", categorie.omschrijving()));
    if !categorie.is_tekstueel() {
        let_op(
            "Hierop kan de tool niet zoeken: een handtekening of een foto staat niet als tekst \
             in het bestand. De controle daarop moet buiten de tool gebeuren en met een tweede \
             persoon worden vastgelegd.",
        );
    }
    toon_ontbrekend(&o);
    Ok(())
}

fn stuk(
    kluis: &mut Kluis,
    kenmerk: &str,
    bestand: &std::path::Path,
    nu: DateTime<Utc>,
) -> Result<()> {
    let mut o = zoek(kluis, kenmerk)?;
    let (hash, _) = hash_van(bestand)?;
    let naam = bestand
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| bestand.display().to_string());
    o.voeg_stuk_toe(&naam, &hash, nu)?;
    bewaar(kluis, &o, Handeling::RecordGewijzigd, &format!("stuk {naam} opgenomen"), nu)?;

    gelukt(&format!("{naam} opgenomen"));
    terzijde(&format!("hash van het origineel: {}", &hash[..16]));
    toon_ontbrekend(&o);
    Ok(())
}

fn uitleveren(kluis: &mut Kluis, kenmerk: &str, hulpmiddel: &str, nu: DateTime<Utc>) -> Result<()> {
    let mut o = zoek(kluis, kenmerk)?;
    o.lever_uit(hulpmiddel, nu, nu)?;
    bewaar(kluis, &o, Handeling::ExportGemaakt, &format!("uitgeleverd aan {hulpmiddel}"), nu)?;

    gelukt(&format!("uitgeleverd aan {hulpmiddel}"));
    kop("Wat er weg moet");
    let mut t = tabel(&["categorie", "waarde", "omschrijving"]);
    for p in &o.profiel {
        t.add_row(vec![
            p.categorie.omschrijving().to_string(),
            p.waarde.clone().unwrap_or_else(|| "—".into()),
            p.omschrijving.clone(),
        ]);
    }
    println!("{t}");
    terzijde(
        "Neem het teruggeleverde bestand terug met 'dpofg redactie terugnemen'; de tool \
         controleert dan de tekstlaag.",
    );
    Ok(())
}

/// Neemt het geredigeerde bestand terug en voert meteen de enige controle uit
/// die dit programma eerlijk zelf kan doen.
fn terugnemen(
    kluis: &mut Kluis,
    kenmerk: &str,
    stuknaam: &str,
    bestand: &std::path::Path,
    nu: DateTime<Utc>,
) -> Result<()> {
    let mut o = zoek(kluis, kenmerk)?;
    let (hash, bytes) = hash_van(bestand)?;
    o.neem_terug(stuknaam, &hash, nu, nu)?;

    let waarden = o.te_zoeken_waarden();
    let bevindingen = zoek_in_bytes(&bytes, &waarden);
    let uitkomst =
        if bevindingen.is_empty() { Controleuitkomst::Geslaagd } else { Controleuitkomst::Gefaald };
    let aantal = waarden.len();

    o.leg_controle_vast(
        stuknaam,
        Terugleescontrole {
            soort: Controlesoort::Tekstlaag,
            uitkomst,
            uitgevoerd_op: nu,
            door: super::actor().id.clone(),
            bevindingen: bevindingen.clone(),
            tweede_persoon: None,
            toelichting: Some(format!(
                "{aantal} waarde(n) gezocht in de ruwe bytes; tekst in een samengedrukte stroom \
                 wordt hierdoor niet gevonden"
            )),
        },
    )?;

    bewaar(kluis, &o, Handeling::RecordGewijzigd, &format!("{stuknaam} teruggenomen"), nu)?;

    gelukt(&format!("{stuknaam} teruggenomen"));
    kop("Controle op de tekstlaag");
    if bevindingen.is_empty() {
        gelukt(&format!("geen van de {aantal} waarden staat nog leesbaar in het bestand"));
    } else {
        for b in &bevindingen {
            blokkade(b);
        }
    }
    let_op(
        "Deze controle zoekt in de ruwe bytes. Zij vindt de meest gemaakte fout — een zwart vlak \
         over tekst die in de tekstlaag blijft staan — en zij vindt géén tekst die in een \
         samengedrukte stroom zit. De metagegevens en het beeld zijn hiermee niet gecontroleerd.",
    );
    toon_ontbrekend(&o);
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn controle(
    kluis: &mut Kluis,
    kenmerk: &str,
    stuknaam: &str,
    soort: Controlesoort,
    uitkomst: Controleuitkomst,
    tweede_persoon: Option<String>,
    toelichting: Option<String>,
    nu: DateTime<Utc>,
) -> Result<()> {
    let mut o = zoek(kluis, kenmerk)?;
    o.leg_controle_vast(
        stuknaam,
        Terugleescontrole {
            soort,
            uitkomst,
            uitgevoerd_op: nu,
            door: super::actor().id.clone(),
            bevindingen: Vec::new(),
            tweede_persoon,
            toelichting,
        },
    )?;
    bewaar(
        kluis,
        &o,
        Handeling::RecordGewijzigd,
        &format!("{}: {}", soort.omschrijving(), uitkomst.omschrijving()),
        nu,
    )?;

    gelukt(&format!("{} vastgelegd als {}", soort.omschrijving(), uitkomst.omschrijving()));
    if soort.is_machinaal() {
        terzijde(
            "Deze controle kan de tool zelf; 'dpofg redactie terugnemen' voert haar uit op het \
             teruggeleverde bestand.",
        );
    }
    toon_ontbrekend(&o);
    Ok(())
}

fn verstrekken(kluis: &mut Kluis, kenmerk: &str, aan: &str, nu: DateTime<Utc>) -> Result<()> {
    let mut o = zoek(kluis, kenmerk)?;
    let actor = super::actor();
    let id = o.id.to_string();

    match o.verstrek(aan, nu, nu) {
        Ok(()) => {
            bewaar(kluis, &o, Handeling::DossierVerstrekt, &format!("verstrekt aan {aan}"), nu)?;
            gelukt(&format!("verstrekt aan {aan}"));
            terzijde(
                "De verstrekking staat in het logboek. Wat er is gecontroleerd en wat niet, staat \
                 per stuk in het dossier.",
            );
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
                    o.compartiment.naam(),
                    "verstrekking geweigerd: de terugleescontrole is niet geslaagd",
                ),
                Some(fout.to_string()),
            )?;
            kop("Verstrekken is geweigerd");
            toon_ontbrekend(&o);
            anyhow::bail!("{fout}")
        }
    }
}

fn lijst(kluis: &Kluis) -> Result<()> {
    let koppen = kluis.lijst(SOORT)?;
    if koppen.is_empty() {
        kop("Redactieopdrachten");
        terzijde("Er staan nog geen redactieopdrachten in de kluis.");
        return Ok(());
    }
    kop("Redactieopdrachten");
    let mut t = tabel(&["kenmerk", "dossier", "stukken", "mag verstrekt worden"]);
    for k in &koppen {
        let o: Redactieopdracht = kluis.laad(SOORT, &k.id)?;
        t.add_row(vec![
            o.kenmerk.clone(),
            format!("{} {}", o.dossier_soort, o.dossier_kenmerk),
            o.stukken.len().to_string(),
            if o.mag_verstrekken() { "ja" } else { "nee" }.to_string(),
        ]);
    }
    println!("{t}");
    Ok(())
}

fn toon(kluis: &Kluis, kenmerk: &str) -> Result<()> {
    let o = zoek(kluis, kenmerk)?;

    kop(&format!("Redactieopdracht {}", o.kenmerk));
    let mut t = tabel(&["", ""]);
    t.add_row(vec!["omschrijving", &o.omschrijving]);
    t.add_row(vec!["dossier", &format!("{} {}", o.dossier_soort, o.dossier_kenmerk)]);
    t.add_row(vec!["status", o.status.omschrijving()]);
    if let Some(h) = &o.hulpmiddel {
        t.add_row(vec!["hulpmiddel", h]);
    }
    if let Some(m) = o.verstrekt_op {
        t.add_row(vec!["verstrekt op", &m.format("%d-%m-%Y").to_string()]);
    }
    println!("{t}");

    if !o.profiel.is_empty() {
        kop("Wat er weg moet");
        let mut t = tabel(&["categorie", "waarde", "machinaal te controleren"]);
        for p in &o.profiel {
            t.add_row(vec![
                p.categorie.omschrijving().to_string(),
                p.waarde.clone().unwrap_or_else(|| "—".into()),
                if p.is_controleerbaar() { "ja" } else { "nee" }.to_string(),
            ]);
        }
        println!("{t}");
    }

    if !o.stukken.is_empty() {
        kop("Stukken en controles");
        let mut t = tabel(&["stuk", "tekstlaag", "metagegevens", "beeld", "handmatig"]);
        for s in &o.stukken {
            t.add_row(vec![
                s.naam.clone(),
                s.uitkomst_van(Controlesoort::Tekstlaag).omschrijving().to_string(),
                s.uitkomst_van(Controlesoort::Metagegevens).omschrijving().to_string(),
                s.uitkomst_van(Controlesoort::Beeldvergelijking).omschrijving().to_string(),
                if s.heeft_handmatige_bevestiging() { "bevestigd" } else { "—" }.to_string(),
            ]);
        }
        println!("{t}");
    }

    toon_ontbrekend(&o);
    Ok(())
}

fn toon_ontbrekend(o: &Redactieopdracht) {
    let r = o.volledigheid();
    kop("Volledigheid");
    println!("  {}", voortgang(r.compleet, r.verplicht));
    if r.is_volledig() {
        println!();
        gelukt("alle verplichte onderdelen zijn ingevuld");
        return;
    }
    println!();
    for x in &r.ontbreekt {
        let veld = x.veld.trim_start_matches("redactie.");
        if x.blokkeert_vaststelling {
            blokkade(&format!("{veld} — {}", x.omschrijving));
        } else {
            let_op(&format!("{veld} — {}", x.omschrijving));
        }
        terzijde(&x.grondslag);
    }
    println!();
    terzijde("■ houdt verstrekking tegen · ▸ blijft zichtbaar maar blokkeert niet");
}
