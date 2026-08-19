//! Doorgiften buiten de Europese Economische Ruimte.

use anyhow::Result;
use chrono::{DateTime, Utc};
use clap::Subcommand;
use dpofg_audit::Handeling;
use dpofg_domain::{
    doorgifte::{Beoordelingsuitkomst, Doorgifte, Doorgiftebeoordeling, Doorgifteinstrumentsoort},
    Motivering, Verwerking, Volledig,
};
use dpofg_store::Kluis;
use std::path::PathBuf;

use crate::uitvoer::{blokkade, gelukt, kop, let_op, tabel, terzijde, voortgang};

const SOORT: &str = "doorgifte";
const COMPARTIMENT: &str = "algemeen";

#[derive(Subcommand, Debug)]
pub enum Doorgifteopdracht {
    /// Toon alle doorgiften.
    Lijst,
    /// Registreer een doorgifte bij een registerregel.
    Nieuw {
        /// Kenmerk van de doorgifte.
        kenmerk: String,
        /// Waar het over gaat.
        omschrijving: String,
        /// Het kenmerk van de registerregel.
        #[arg(long)]
        verwerking: String,
        /// De ontvanger.
        #[arg(long)]
        ontvanger: String,
        /// Het land van de ontvanger.
        #[arg(long)]
        land: String,
    },
    /// Toon één doorgifte.
    Toon {
        /// Kenmerk van de doorgifte.
        kenmerk: String,
    },
    /// Toon de instrumenten uit het kennispakket, met hun status.
    Instrumenten,
    /// Wijs aan waarop de doorgifte berust.
    Instrument {
        /// Kenmerk van de doorgifte.
        kenmerk: String,
        /// Het instrument.
        #[arg(long, value_enum)]
        soort: Instrumentkeuze,
        /// De code uit het kennispakket, bijvoorbeeld SCC-2021.
        #[arg(long)]
        code: Option<String>,
    },
    /// Benoem de uitzondering van artikel 49 die wordt ingeroepen.
    Artikel49 {
        /// Kenmerk van de doorgifte.
        kenmerk: String,
        /// De grond uit de limitatieve opsomming van artikel 49 lid 1.
        #[arg(long)]
        grond: String,
        /// Hoe vaak zij dit jaar is toegepast.
        #[arg(long, default_value = "1")]
        toepassingen: u32,
    },
    /// Leg een aanvullende maatregel vast.
    Maatregel {
        /// Kenmerk van de doorgifte.
        kenmerk: String,
        /// De maatregel.
        #[arg(long)]
        omschrijving: String,
    },
    /// Leg de doorgiftebeoordeling vast.
    Beoordeling {
        /// Kenmerk van de doorgifte.
        kenmerk: String,
        /// De uitkomst.
        #[arg(long, value_enum)]
        uitkomst: Beoordelingkeuze,
        /// Wie de beoordeling uitvoerde.
        #[arg(long)]
        door: String,
        /// Wie het besluit nam.
        #[arg(long)]
        besluit_door: String,
        /// Het restrisico dat overblijft.
        #[arg(long)]
        restrisico: String,
    },
    /// Controleer het instrument tegen het kennispakket.
    Controleer {
        /// Kenmerk van de doorgifte. Zonder kenmerk: alle doorgiften.
        kenmerk: Option<String>,
    },
    /// Stel de doorgifte vast.
    Vaststellen {
        /// Kenmerk van de doorgifte.
        kenmerk: String,
    },
}

#[derive(clap::ValueEnum, Clone, Copy, Debug)]
pub enum Instrumentkeuze {
    Adequaatheidsbesluit,
    Modelbepalingen,
    Bedrijfsvoorschriften,
    Gedragscode,
    Certificering,
    Artikel49,
    Geen,
}

impl From<Instrumentkeuze> for Doorgifteinstrumentsoort {
    fn from(k: Instrumentkeuze) -> Self {
        match k {
            Instrumentkeuze::Adequaatheidsbesluit => Doorgifteinstrumentsoort::Adequaatheidsbesluit,
            Instrumentkeuze::Modelbepalingen => Doorgifteinstrumentsoort::Modelbepalingen,
            Instrumentkeuze::Bedrijfsvoorschriften => {
                Doorgifteinstrumentsoort::BindendeBedrijfsvoorschriften
            }
            Instrumentkeuze::Gedragscode => Doorgifteinstrumentsoort::Gedragscode,
            Instrumentkeuze::Certificering => Doorgifteinstrumentsoort::Certificering,
            Instrumentkeuze::Artikel49 => Doorgifteinstrumentsoort::Artikel49Uitzondering,
            Instrumentkeuze::Geen => Doorgifteinstrumentsoort::Geen,
        }
    }
}

#[derive(clap::ValueEnum, Clone, Copy, Debug)]
pub enum Beoordelingkeuze {
    Gelijkwaardig,
    GelijkwaardigMetMaatregelen,
    NietGelijkwaardig,
}

impl From<Beoordelingkeuze> for Beoordelingsuitkomst {
    fn from(k: Beoordelingkeuze) -> Self {
        match k {
            Beoordelingkeuze::Gelijkwaardig => Beoordelingsuitkomst::Gelijkwaardig,
            Beoordelingkeuze::GelijkwaardigMetMaatregelen => {
                Beoordelingsuitkomst::GelijkwaardigMetMaatregelen
            }
            Beoordelingkeuze::NietGelijkwaardig => Beoordelingsuitkomst::NietGelijkwaardig,
        }
    }
}

pub fn draai(o: Doorgifteopdracht, kluispad: Option<PathBuf>, nu: DateTime<Utc>) -> Result<()> {
    if matches!(o, Doorgifteopdracht::Instrumenten) {
        return toon_instrumenten(nu);
    }
    let pad = super::kluispad(kluispad)?;
    let mut kluis = super::open_kluis(&pad, nu)?;
    match o {
        Doorgifteopdracht::Instrumenten => unreachable!("hierboven afgehandeld"),
        Doorgifteopdracht::Lijst => lijst(&kluis),
        Doorgifteopdracht::Nieuw { kenmerk, omschrijving, verwerking, ontvanger, land } => {
            nieuw(&mut kluis, &kenmerk, &omschrijving, &verwerking, &ontvanger, &land, nu)
        }
        Doorgifteopdracht::Toon { kenmerk } => toon(&kluis, &kenmerk),
        Doorgifteopdracht::Instrument { kenmerk, soort, code } => {
            instrument(&mut kluis, &kenmerk, soort.into(), code, nu)
        }
        Doorgifteopdracht::Artikel49 { kenmerk, grond, toepassingen } => {
            artikel49(&mut kluis, &kenmerk, &grond, toepassingen, nu)
        }
        Doorgifteopdracht::Maatregel { kenmerk, omschrijving } => {
            maatregel(&mut kluis, &kenmerk, &omschrijving, nu)
        }
        Doorgifteopdracht::Beoordeling { kenmerk, uitkomst, door, besluit_door, restrisico } => {
            beoordeling(
                &mut kluis,
                &kenmerk,
                uitkomst.into(),
                &door,
                &besluit_door,
                &restrisico,
                nu,
            )
        }
        Doorgifteopdracht::Controleer { kenmerk } => controleer(&mut kluis, kenmerk.as_deref(), nu),
        Doorgifteopdracht::Vaststellen { kenmerk } => vaststellen(&mut kluis, &kenmerk, nu),
    }
}

fn zoek(kluis: &Kluis, kenmerk: &str) -> Result<Doorgifte> {
    let kop = kluis
        .lijst(SOORT)?
        .into_iter()
        .find(|r| r.kenmerk.as_deref() == Some(kenmerk))
        .ok_or_else(|| anyhow::anyhow!("geen doorgifte met kenmerk '{kenmerk}'"))?;
    Ok(kluis.laad(SOORT, &kop.id)?)
}

fn bewaar(
    kluis: &mut Kluis,
    d: &Doorgifte,
    handeling: Handeling,
    omschrijving: &str,
    nu: DateTime<Utc>,
) -> Result<()> {
    let actor = super::actor();
    kluis.bewaar(
        SOORT,
        &d.id.to_string(),
        COMPARTIMENT,
        d.status.omschrijving(),
        Some(&d.kenmerk),
        d,
        &actor,
        handeling,
        omschrijving,
        nu,
    )?;
    Ok(())
}

fn toon_instrumenten(nu: DateTime<Utc>) -> Result<()> {
    let pakket = dpofg_content::startpakket(nu.date_naive());
    kop("Doorgifte-instrumenten in het kennispakket");
    let mut t = tabel(&["code", "gebied", "status", "geverifieerd op"]);
    for i in &pakket.doorgifteinstrumenten {
        t.add_row(vec![
            i.code.clone(),
            i.land_of_gebied.clone(),
            format!("{:?}", i.status).to_lowercase(),
            crate::uitvoer::datum(i.geverifieerd_op).to_string(),
        ]);
    }
    println!("{t}");
    terzijde(
        "Een instrument kan worden ingetrokken of onder toetsing komen te staan zonder dat er in \
         de organisatie iets verandert. 'dpofg doorgifte controleer' houdt de doorgiften ertegen \
         aan.",
    );
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn nieuw(
    kluis: &mut Kluis,
    kenmerk: &str,
    omschrijving: &str,
    verwerkingkenmerk: &str,
    ontvanger: &str,
    land: &str,
    nu: DateTime<Utc>,
) -> Result<()> {
    if kluis.lijst(SOORT)?.iter().any(|r| r.kenmerk.as_deref() == Some(kenmerk)) {
        anyhow::bail!("er bestaat al een doorgifte met kenmerk '{kenmerk}'");
    }
    let kop_v = kluis
        .lijst("verwerking")?
        .into_iter()
        .find(|r| r.kenmerk.as_deref() == Some(verwerkingkenmerk))
        .ok_or_else(|| anyhow::anyhow!("geen registerregel met kenmerk '{verwerkingkenmerk}'"))?;
    let mut v: Verwerking = kluis.laad("verwerking", &kop_v.id)?;

    let d = Doorgifte::nieuw(
        kenmerk,
        omschrijving,
        v.id,
        &v.kenmerk,
        ontvanger,
        land,
        &super::actor().id,
        nu,
    );
    bewaar(kluis, &d, Handeling::RecordAangemaakt, "doorgifte geregistreerd", nu)?;

    if !v.doorgiften.contains(&d.id) {
        v.doorgiften.push(d.id);
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
            &format!("doorgifte {kenmerk} gekoppeld"),
            nu,
        )?;
    }

    gelukt(&format!("doorgifte {kenmerk} naar {land} geregistreerd"));
    terzijde("Het aanwijzen van een instrument is de makkelijke helft; de andere helft is dat het instrument iets waarmaakt.");
    toon_ontbrekend(&d);
    Ok(())
}

fn instrument(
    kluis: &mut Kluis,
    kenmerk: &str,
    soort: Doorgifteinstrumentsoort,
    code: Option<String>,
    nu: DateTime<Utc>,
) -> Result<()> {
    let mut d = zoek(kluis, kenmerk)?;
    d.kies_instrument(soort, code, nu)?;
    bewaar(
        kluis,
        &d,
        Handeling::RecordGewijzigd,
        &format!("instrument: {}", soort.omschrijving()),
        nu,
    )?;

    gelukt(&format!("{} aangewezen", soort.omschrijving()));
    terzijde(soort.grondslag());
    if soort.vraagt_beoordeling() {
        let_op(
            "Dit instrument vraagt een beoordeling van het recht en de praktijk in het \
             ontvangstland. Zonder die beoordeling is het contract een handtekening onder een \
             aanname; regel EER-03 signaleert dat.",
        );
    }
    toon_ontbrekend(&d);
    Ok(())
}

fn artikel49(
    kluis: &mut Kluis,
    kenmerk: &str,
    grond: &str,
    toepassingen: u32,
    nu: DateTime<Utc>,
) -> Result<()> {
    let mut d = zoek(kluis, kenmerk)?;
    d.artikel49_grond = Some(grond.to_string());
    d.artikel49_toepassingen_dit_jaar = toepassingen;
    d.herkomst.wijzig("uitzondering van artikel 49 vastgelegd", nu);
    bewaar(kluis, &d, Handeling::MotiveringVastgelegd, "artikel 49-grond vastgelegd", nu)?;

    gelukt(&format!("grond vastgelegd, {toepassingen} toepassing(en) dit jaar"));
    terzijde("art. 49 lid 1 AVG; de opsomming daar is limitatief");
    let_op(
        "De uitzonderingen van artikel 49 zijn er voor incidentele gevallen. Wie ze structureel \
         gebruikt, gebruikt geen uitzondering meer maar een instrument dat hij niet heeft.",
    );
    toon_ontbrekend(&d);
    Ok(())
}

fn maatregel(
    kluis: &mut Kluis,
    kenmerk: &str,
    omschrijving: &str,
    nu: DateTime<Utc>,
) -> Result<()> {
    let mut d = zoek(kluis, kenmerk)?;
    d.aanvullende_maatregelen.push(omschrijving.to_string());
    d.herkomst.wijzig("aanvullende maatregel vastgelegd", nu);
    bewaar(kluis, &d, Handeling::RecordGewijzigd, "aanvullende maatregel", nu)?;
    gelukt(&format!("maatregel vastgelegd: {omschrijving}"));
    toon_ontbrekend(&d);
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn beoordeling(
    kluis: &mut Kluis,
    kenmerk: &str,
    uitkomst: Beoordelingsuitkomst,
    door: &str,
    besluit_door: &str,
    restrisico: &str,
    nu: DateTime<Utc>,
) -> Result<()> {
    let mut d = zoek(kluis, kenmerk)?;
    let m = Motivering::nieuw(restrisico, &super::actor().id, nu)?;
    d.leg_beoordeling_vast(
        Doorgiftebeoordeling {
            datum: nu,
            uitvoerder: door.to_string(),
            rechtsontwikkelingen_geraadpleegd_op: nu,
            uitkomst,
            restrisico: m,
            besluit_door: besluit_door.to_string(),
        },
        nu,
    )?;
    bewaar(
        kluis,
        &d,
        Handeling::BesluitGenomen,
        &format!("beoordeling: {}", uitkomst.omschrijving()),
        nu,
    )?;

    gelukt(&format!("beoordeling vastgelegd: {}", uitkomst.omschrijving()));
    if !uitkomst.draagt_de_doorgifte() {
        let_op(
            "Het beschermingsniveau is ook met maatregelen niet gelijkwaardig. Deze doorgifte kan \
             niet doorgaan op dit instrument; de beoordeling blijft staan als verantwoording van \
             dat besluit.",
        );
    }
    toon_ontbrekend(&d);
    Ok(())
}

/// Houdt de doorgiften tegen de status van het instrument in het kennispakket.
fn controleer(kluis: &mut Kluis, kenmerk: Option<&str>, nu: DateTime<Utc>) -> Result<()> {
    let pakket = dpofg_content::startpakket(nu.date_naive());
    let koppen = kluis.lijst(SOORT)?;
    let doelen: Vec<_> = match kenmerk {
        Some(k) => koppen.into_iter().filter(|r| r.kenmerk.as_deref() == Some(k)).collect(),
        None => koppen,
    };
    if doelen.is_empty() {
        anyhow::bail!("er is geen doorgifte om te controleren");
    }

    kop("Controle van de instrumenten");
    let mut gewijzigd = 0usize;
    for k in doelen {
        let mut d: Doorgifte = kluis.laad(SOORT, &k.id)?;
        let Some(code) = d.instrument_code.clone() else {
            terzijde(&format!("{}: geen instrumentcode, niets te controleren", d.kenmerk));
            continue;
        };
        let Some(i) = pakket.doorgifteinstrumenten.iter().find(|i| i.code == code) else {
            let_op(&format!("{}: instrument '{code}' staat niet in het kennispakket", d.kenmerk));
            continue;
        };
        let status = format!("{:?}", i.status).to_lowercase();
        let herbeoordeling = i.status.vereist_herbeoordeling();
        d.controleer_instrument(&status, herbeoordeling, nu);
        bewaar(
            kluis,
            &d,
            Handeling::KetenGeverifieerd,
            &format!("instrument {code}: {status}"),
            nu,
        )?;
        gewijzigd += 1;

        if herbeoordeling {
            blokkade(&format!("{}: instrument {code} staat op '{status}'", d.kenmerk));
            terzijde("hoofdstuk V AVG; regel EER-07 signaleert dit tot de doorgifte opnieuw is beoordeeld");
        } else {
            gelukt(&format!("{}: instrument {code} is {status}", d.kenmerk));
        }
    }
    terzijde(&format!(
        "{gewijzigd} doorgifte(n) gecontroleerd tegen kennispakket {} {}",
        pakket.code, pakket.versienaam
    ));
    Ok(())
}

fn vaststellen(kluis: &mut Kluis, kenmerk: &str, nu: DateTime<Utc>) -> Result<()> {
    let mut d = zoek(kluis, kenmerk)?;
    let actor = super::actor();
    let id = d.id.to_string();
    match d.stel_vast(&actor.naam, nu) {
        Ok(()) => {
            bewaar(kluis, &d, Handeling::RecordVastgesteld, "doorgifte vastgesteld", nu)?;
            gelukt(&format!("doorgifte {kenmerk} vastgesteld"));
            toon_ontbrekend(&d);
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
                    d.compartiment.naam(),
                    "vaststellen geweigerd: verplichte onderdelen ontbreken",
                ),
                Some(fout.to_string()),
            )?;
            kop("Vaststellen is niet gelukt");
            toon_ontbrekend(&d);
            anyhow::bail!("{fout}")
        }
    }
}

fn lijst(kluis: &Kluis) -> Result<()> {
    let koppen = kluis.lijst(SOORT)?;
    if koppen.is_empty() {
        kop("Doorgiften");
        terzijde("Er staan nog geen doorgiften in de kluis.");
        return Ok(());
    }
    kop("Doorgiften");
    let mut t = tabel(&["kenmerk", "land", "instrument", "beoordeeld", "status"]);
    for k in &koppen {
        let d: Doorgifte = kluis.laad(SOORT, &k.id)?;
        t.add_row(vec![
            d.kenmerk.clone(),
            d.ontvangerland.clone(),
            d.instrument.map(|i| i.omschrijving().to_string()).unwrap_or_else(|| "—".into()),
            if d.beoordeling.is_some() {
                "ja"
            } else if d.mist_beoordeling() {
                "nee"
            } else {
                "n.v.t."
            }
            .to_string(),
            d.status.omschrijving().to_string(),
        ]);
    }
    println!("{t}");
    Ok(())
}

fn toon(kluis: &Kluis, kenmerk: &str) -> Result<()> {
    let d = zoek(kluis, kenmerk)?;
    kop(&format!("Doorgifte {}", d.kenmerk));
    let mut t = tabel(&["", ""]);
    t.add_row(vec!["omschrijving", &d.omschrijving]);
    t.add_row(vec!["registerregel", &d.verwerking_kenmerk]);
    t.add_row(vec!["ontvanger", &d.ontvanger]);
    t.add_row(vec!["land", &d.ontvangerland]);
    t.add_row(vec!["status", d.status.omschrijving()]);
    if let Some(i) = d.instrument {
        t.add_row(vec!["instrument", i.omschrijving()]);
        t.add_row(vec!["grondslag", i.grondslag()]);
    }
    if let Some(c) = &d.instrument_code {
        t.add_row(vec!["code", c]);
    }
    if let Some(s) = &d.instrument_status_bij_controle {
        t.add_row(vec!["status bij laatste controle", s]);
    }
    println!("{t}");

    if let Some(b) = &d.beoordeling {
        kop("Doorgiftebeoordeling");
        let mut t = tabel(&["", ""]);
        t.add_row(vec!["uitkomst", b.uitkomst.omschrijving()]);
        t.add_row(vec!["uitgevoerd door", &b.uitvoerder]);
        t.add_row(vec!["besluit door", &b.besluit_door]);
        t.add_row(vec![
            "rechtsontwikkelingen geraadpleegd",
            &crate::uitvoer::datum(b.rechtsontwikkelingen_geraadpleegd_op).to_string(),
        ]);
        println!("{t}");
        terzijde(&b.restrisico.tekst);
    }
    if !d.aanvullende_maatregelen.is_empty() {
        kop("Aanvullende maatregelen");
        for m in &d.aanvullende_maatregelen {
            println!("  • {m}");
        }
    }
    if let Some(g) = &d.artikel49_grond {
        kop("Uitzondering van artikel 49");
        println!("  {g}");
        terzijde(&format!("dit jaar {} keer toegepast", d.artikel49_toepassingen_dit_jaar));
    }
    toon_ontbrekend(&d);
    Ok(())
}

fn toon_ontbrekend(d: &Doorgifte) {
    let r = d.volledigheid();
    kop("Volledigheid");
    println!("  {}", voortgang(r.compleet, r.verplicht));
    if r.is_volledig() {
        println!();
        gelukt("alle verplichte onderdelen zijn ingevuld");
        return;
    }
    println!();
    for o in &r.ontbreekt {
        let veld = o.veld.trim_start_matches("doorgifte.");
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
