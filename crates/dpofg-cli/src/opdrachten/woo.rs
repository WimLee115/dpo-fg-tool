//! Verzoeken om informatie op grond van de Wet open overheid.
//!
//! Een eigen spoor naast het inzageverzoek: vier weken in plaats van een maand,
//! eigen weigeringsgronden en eigen rechtsbescherming. De opdracht `koppel`
//! legt de verwijzing naar het inzageverzoek wanneer één bericht beide bevatte.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use clap::Subcommand;
use dpofg_audit::Handeling;
use dpofg_domain::{
    verzoek::Betrokkenenverzoek,
    woo::{Weigeringsgrond, Woouitkomst, Wooverzoek},
    Motivering, Volledig,
};
use dpofg_store::Kluis;
use std::path::PathBuf;

use crate::uitvoer::{blokkade, duur, gelukt, kop, let_op, tabel, terzijde, voortgang};

const SOORT: &str = "woo";
const COMPARTIMENT: &str = "vertrouwelijk";

#[derive(Subcommand, Debug)]
pub enum Wooopdracht {
    /// Toon alle Woo-verzoeken met hun beslistermijn.
    Lijst,
    /// Registreer een verzoek om informatie.
    Nieuw {
        /// Kenmerk waaronder het verzoek bekend staat.
        kenmerk: String,
        /// Het onderwerp van het verzoek.
        onderwerp: String,
        /// Wie het verzoek indiende.
        #[arg(long, default_value = "niet vermeld")]
        verzoeker: String,
        /// Wanneer het is ontvangen. Standaard: nu.
        #[arg(long)]
        ontvangen: Option<String>,
    },
    /// Toon één verzoek met zijn termijn en wat er nog ontbreekt.
    Toon {
        /// Het kenmerk van het verzoek.
        kenmerk: String,
    },
    /// Start de beslistermijn van vier weken.
    Termijn {
        /// Het kenmerk van het verzoek.
        kenmerk: String,
    },
    /// Leg de verdaging van ten hoogste twee weken vast.
    Verdagen {
        /// Het kenmerk van het verzoek.
        kenmerk: String,
        /// Wanneer de verdaging is medegedeeld.
        medegedeeld_op: String,
        /// De schriftelijke motivering die de wet verlangt.
        #[arg(long)]
        motivering: String,
    },
    /// Voeg een belanghebbende derde toe.
    Belanghebbende {
        /// Het kenmerk van het verzoek.
        kenmerk: String,
        /// De belanghebbende.
        #[arg(long)]
        naam: String,
    },
    /// Leg vast dat een belanghebbende gelegenheid kreeg voor een zienswijze.
    Zienswijze {
        /// Het kenmerk van het verzoek.
        kenmerk: String,
        /// De belanghebbende.
        #[arg(long)]
        naam: String,
        /// Wanneer de gelegenheid is geboden.
        #[arg(long)]
        gevraagd: String,
        /// Wanneer een reactie binnenkwam.
        #[arg(long)]
        ontvangen: Option<String>,
        /// Wat de belanghebbende inbracht.
        #[arg(long)]
        standpunt: Option<String>,
    },
    /// Toon de weigeringsgronden van artikel 5.1, met hun lid.
    Gronden,
    /// Roep een weigeringsgrond in.
    Grond {
        /// Het kenmerk van het verzoek.
        kenmerk: String,
        /// De grond.
        #[arg(long, value_enum)]
        grond: Grondkeuze,
        /// Op welk document of onderdeel de grond ziet.
        #[arg(long)]
        betreft: String,
        /// De afweging tegen het belang van openbaarheid. Verplicht bij een
        /// relatieve grond.
        #[arg(long)]
        afweging: Option<String>,
    },
    /// Koppel dit verzoek aan een inzageverzoek uit hetzelfde bericht.
    Koppel {
        /// Het kenmerk van het Woo-verzoek.
        kenmerk: String,
        /// Het kenmerk van het betrokkenenverzoek.
        #[arg(long)]
        verzoek: String,
    },
    /// Neem het besluit op het verzoek.
    Besluit {
        /// Het kenmerk van het verzoek.
        kenmerk: String,
        /// De uitkomst.
        #[arg(long, value_enum)]
        uitkomst: Uitkomstkeuze,
        /// Wanneer. Standaard: nu.
        #[arg(long)]
        op: Option<String>,
    },
}

#[derive(clap::ValueEnum, Clone, Copy, Debug)]
pub enum Grondkeuze {
    EenheidVanDeKroon,
    VeiligheidVanDeStaat,
    BedrijfsEnFabricagegegevens,
    Persoonsgegevens,
    BetrekkingenMetAndereStaten,
    EconomischeBelangen,
    OpsporingEnVervolging,
    InspectieEnToezicht,
    PersoonlijkeLevenssfeer,
    Milieu,
    BeveiligingVanPersonenEnBedrijven,
    GoedFunctioneren,
}

impl From<Grondkeuze> for Weigeringsgrond {
    fn from(k: Grondkeuze) -> Self {
        match k {
            Grondkeuze::EenheidVanDeKroon => Weigeringsgrond::EenheidVanDeKroon,
            Grondkeuze::VeiligheidVanDeStaat => Weigeringsgrond::VeiligheidVanDeStaat,
            Grondkeuze::BedrijfsEnFabricagegegevens => Weigeringsgrond::BedrijfsEnFabricagegegevens,
            Grondkeuze::Persoonsgegevens => Weigeringsgrond::Persoonsgegevens,
            Grondkeuze::BetrekkingenMetAndereStaten => Weigeringsgrond::BetrekkingenMetAndereStaten,
            Grondkeuze::EconomischeBelangen => Weigeringsgrond::EconomischeBelangen,
            Grondkeuze::OpsporingEnVervolging => Weigeringsgrond::OpsporingEnVervolging,
            Grondkeuze::InspectieEnToezicht => Weigeringsgrond::InspectieEnToezicht,
            Grondkeuze::PersoonlijkeLevenssfeer => {
                Weigeringsgrond::EerbiedigingPersoonlijkeLevenssfeer
            }
            Grondkeuze::Milieu => Weigeringsgrond::BeschermingMilieu,
            Grondkeuze::BeveiligingVanPersonenEnBedrijven => {
                Weigeringsgrond::BeveiligingVanPersonenEnBedrijven
            }
            Grondkeuze::GoedFunctioneren => Weigeringsgrond::GoedFunctionerenVanDeStaat,
        }
    }
}

#[derive(clap::ValueEnum, Clone, Copy, Debug)]
pub enum Uitkomstkeuze {
    Openbaar,
    GedeeltelijkOpenbaar,
    Geweigerd,
    NietAanwezig,
}

impl From<Uitkomstkeuze> for Woouitkomst {
    fn from(k: Uitkomstkeuze) -> Self {
        match k {
            Uitkomstkeuze::Openbaar => Woouitkomst::Openbaar,
            Uitkomstkeuze::GedeeltelijkOpenbaar => Woouitkomst::GedeeltelijkOpenbaar,
            Uitkomstkeuze::Geweigerd => Woouitkomst::Geweigerd,
            Uitkomstkeuze::NietAanwezig => Woouitkomst::NietAanwezig,
        }
    }
}

pub fn draai(o: Wooopdracht, kluispad: Option<PathBuf>, nu: DateTime<Utc>) -> Result<()> {
    if matches!(o, Wooopdracht::Gronden) {
        return toon_gronden();
    }

    let pad = super::kluispad(kluispad)?;
    let mut kluis = super::open_kluis(&pad, nu)?;
    match o {
        Wooopdracht::Gronden => unreachable!("hierboven afgehandeld"),
        Wooopdracht::Lijst => lijst(&kluis, nu),
        Wooopdracht::Nieuw { kenmerk, onderwerp, verzoeker, ontvangen } => {
            nieuw(&mut kluis, &kenmerk, &onderwerp, &verzoeker, ontvangen.as_deref(), nu)
        }
        Wooopdracht::Toon { kenmerk } => toon(&kluis, &kenmerk, nu),
        Wooopdracht::Termijn { kenmerk } => start_termijn(&mut kluis, &kenmerk, nu),
        Wooopdracht::Verdagen { kenmerk, medegedeeld_op, motivering } => {
            verdagen(&mut kluis, &kenmerk, &medegedeeld_op, &motivering, nu)
        }
        Wooopdracht::Belanghebbende { kenmerk, naam } => {
            belanghebbende(&mut kluis, &kenmerk, &naam, nu)
        }
        Wooopdracht::Zienswijze { kenmerk, naam, gevraagd, ontvangen, standpunt } => {
            zienswijze(&mut kluis, &kenmerk, &naam, &gevraagd, ontvangen.as_deref(), standpunt, nu)
        }
        Wooopdracht::Grond { kenmerk, grond, betreft, afweging } => {
            leg_grond_in(&mut kluis, &kenmerk, grond.into(), &betreft, afweging.as_deref(), nu)
        }
        Wooopdracht::Koppel { kenmerk, verzoek } => koppel(&mut kluis, &kenmerk, &verzoek, nu),
        Wooopdracht::Besluit { kenmerk, uitkomst, op } => {
            besluit(&mut kluis, &kenmerk, uitkomst.into(), op.as_deref(), nu)
        }
    }
}

fn zoek(kluis: &Kluis, kenmerk: &str) -> Result<Wooverzoek> {
    let kop = kluis
        .lijst(SOORT)?
        .into_iter()
        .find(|r| r.kenmerk.as_deref() == Some(kenmerk))
        .ok_or_else(|| anyhow::anyhow!("geen Woo-verzoek met kenmerk '{kenmerk}'"))?;
    Ok(kluis.laad(SOORT, &kop.id)?)
}

fn bewaar(
    kluis: &mut Kluis,
    v: &Wooverzoek,
    handeling: Handeling,
    omschrijving: &str,
    nu: DateTime<Utc>,
) -> Result<()> {
    let actor = super::actor();
    kluis.bewaar(
        SOORT,
        &v.id.to_string(),
        COMPARTIMENT,
        v.status.omschrijving(),
        Some(&v.kenmerk),
        v,
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

fn toon_gronden() -> Result<()> {
    kop("Weigeringsgronden van artikel 5.1 Wet open overheid");
    let mut t = tabel(&["grond", "soort", "grondslag"]);
    for g in Weigeringsgrond::alle() {
        t.add_row(vec![
            g.omschrijving().to_string(),
            if g.is_relatief() { "relatief" } else { "absoluut" }.to_string(),
            g.grondslag().to_string(),
        ]);
    }
    println!("{t}");
    terzijde(
        "Bij een absolute grond valt er niets af te wegen. Bij een relatieve grond is de afweging \
         tegen het belang van openbaarheid de kern van het besluit; zonder die afweging is er \
         geen besluit maar een verwijzing naar een wetsartikel.",
    );
    Ok(())
}

fn nieuw(
    kluis: &mut Kluis,
    kenmerk: &str,
    onderwerp: &str,
    verzoeker: &str,
    ontvangen: Option<&str>,
    nu: DateTime<Utc>,
) -> Result<()> {
    if kluis.lijst(SOORT)?.iter().any(|r| r.kenmerk.as_deref() == Some(kenmerk)) {
        anyhow::bail!("er bestaat al een Woo-verzoek met kenmerk '{kenmerk}'");
    }
    let ontvangen_op = match ontvangen {
        Some(t) => lees_tijdstip(t)?,
        None => nu,
    };
    if ontvangen_op > nu {
        anyhow::bail!("het verzoek zou in de toekomst zijn ontvangen; controleer het tijdstip");
    }

    let v = Wooverzoek::nieuw(kenmerk, onderwerp, verzoeker, ontvangen_op, &super::actor().id, nu);
    bewaar(kluis, &v, Handeling::RecordAangemaakt, "Woo-verzoek geregistreerd", nu)?;

    gelukt(&format!("Woo-verzoek {kenmerk} geregistreerd"));
    let_op(
        "Dit is een ánder spoor dan een inzageverzoek: vier weken in plaats van een maand, eigen \
         weigeringsgronden en bezwaar en beroep bij de bestuursrechter. Bevatte hetzelfde bericht \
         ook een verzoek van een betrokkene over zichzelf, maak daar dan een eigen dossier voor \
         en koppel de twee met 'dpofg woo koppel'.",
    );
    toon_ontbrekend(&v);
    Ok(())
}

fn start_termijn(kluis: &mut Kluis, kenmerk: &str, nu: DateTime<Utc>) -> Result<()> {
    let mut v = zoek(kluis, kenmerk)?;

    let pakket = dpofg_content::startpakket(nu.date_naive());
    let soort = pakket
        .termijn("WOO-BESLISTERMIJN")
        .context("de beslistermijn ontbreekt in het kennispakket")?
        .clone();
    let kalender = pakket.kalender("NL").context("de feestdagenkalender ontbreekt")?;
    let zone = dpofg_terms::tijdzone(dpofg_terms::TIJDZONE_NL)?;

    let klok = dpofg_terms::LopendeTermijn::start(soort, v.ontvangen_op, zone, kalender)
        .context("de beslistermijn kon niet worden berekend")?;
    let deadline = klok.deadline_volledig(nu, zone, kalender)?;

    v.start_termijn(klok, nu)?;
    v.termijn_pakket = Some(format!(
        "{} {}, geconsolideerd {}",
        pakket.code,
        pakket.versienaam,
        pakket.consolidatiedatum.format("%d-%m-%Y")
    ));

    bewaar(kluis, &v, Handeling::TermijnGestart, "beslistermijn gestart", nu)?;

    gelukt("de beslistermijn loopt");
    let mut t = tabel(&["", ""]);
    t.add_row(vec!["duur", &deadline.duur]);
    t.add_row(vec!["verstrijkt", &deadline.lokaal]);
    t.add_row(vec!["grondslag", &deadline.grondslag]);
    println!("{t}");
    terzijde(&deadline.verantwoording);
    let_op(
        "Een verdaging van ten hoogste twee weken moet binnen deze termijn schriftelijk en \
         gemotiveerd worden medegedeeld. Daarna is zij niet meer in te roepen.",
    );
    toon_ontbrekend(&v);
    Ok(())
}

fn verdagen(
    kluis: &mut Kluis,
    kenmerk: &str,
    medegedeeld_op: &str,
    motivering: &str,
    nu: DateTime<Utc>,
) -> Result<()> {
    let mut v = zoek(kluis, kenmerk)?;
    let moment = lees_tijdstip(medegedeeld_op)?;
    if moment > nu {
        anyhow::bail!("de mededeling zou in de toekomst zijn verzonden; controleer het tijdstip");
    }

    let pakket = dpofg_content::startpakket(nu.date_naive());
    let kalender = pakket.kalender("NL").context("de feestdagenkalender ontbreekt")?;
    let zone = dpofg_terms::tijdzone(dpofg_terms::TIJDZONE_NL)?;

    let klok = v
        .termijn
        .as_mut()
        .ok_or_else(|| anyhow::anyhow!("de beslistermijn van '{kenmerk}' loopt nog niet"))?;
    klok.verleng(moment, zone, kalender)?;

    let m = Motivering::nieuw(motivering, &super::actor().id, nu)?;
    v.leg_verdaging_vast(moment, m, nu)?;

    let deadline = v
        .termijn
        .as_ref()
        .expect("de klok is hierboven gezet")
        .deadline_volledig(nu, zone, kalender)?;

    bewaar(kluis, &v, Handeling::TermijnVerlengd, "beslistermijn verdaagd", nu)?;
    gelukt("verdaging vastgelegd");
    terzijde(&format!("verstrijkt nu op {}", deadline.lokaal));
    toon_ontbrekend(&v);
    Ok(())
}

fn belanghebbende(kluis: &mut Kluis, kenmerk: &str, naam: &str, nu: DateTime<Utc>) -> Result<()> {
    let mut v = zoek(kluis, kenmerk)?;
    v.voeg_belanghebbende_toe(naam, nu)?;
    bewaar(
        kluis,
        &v,
        Handeling::RecordGewijzigd,
        &format!("belanghebbende {naam} toegevoegd"),
        nu,
    )?;
    gelukt(&format!("belanghebbende {naam} toegevoegd"));
    terzijde("art. 4.4 lid 4 Wet open overheid: de belanghebbende krijgt gelegenheid tot een zienswijze.");
    toon_ontbrekend(&v);
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn zienswijze(
    kluis: &mut Kluis,
    kenmerk: &str,
    naam: &str,
    gevraagd: &str,
    ontvangen: Option<&str>,
    standpunt: Option<String>,
    nu: DateTime<Utc>,
) -> Result<()> {
    let mut v = zoek(kluis, kenmerk)?;
    let gevraagd_op = lees_tijdstip(gevraagd)?;
    let ontvangen_op = match ontvangen {
        Some(t) => Some(lees_tijdstip(t)?),
        None => None,
    };
    v.leg_zienswijze_vast(naam, gevraagd_op, ontvangen_op, standpunt, nu)?;

    bewaar(kluis, &v, Handeling::RecordGewijzigd, &format!("zienswijze van {naam}"), nu)?;
    gelukt(&format!("zienswijze van {naam} vastgelegd"));
    if ontvangen_op.is_none() {
        terzijde(
            "Er is nog geen reactie. Dat houdt het besluit niet tegen: de wet vraagt dat de \
             gelegenheid wordt geboden, niet dat er wordt gereageerd.",
        );
    }
    toon_ontbrekend(&v);
    Ok(())
}

fn leg_grond_in(
    kluis: &mut Kluis,
    kenmerk: &str,
    grond: Weigeringsgrond,
    betreft: &str,
    afweging: Option<&str>,
    nu: DateTime<Utc>,
) -> Result<()> {
    let mut v = zoek(kluis, kenmerk)?;
    let m = match afweging {
        Some(t) => Some(Motivering::nieuw(t, &super::actor().id, nu)?),
        None => None,
    };
    v.roep_grond_in(grond, betreft, m, nu)?;

    bewaar(
        kluis,
        &v,
        Handeling::MotiveringVastgelegd,
        &format!("grond: {}", grond.omschrijving()),
        nu,
    )?;
    gelukt(&format!("{} ingeroepen voor {betreft}", grond.omschrijving()));
    terzijde(grond.grondslag());
    toon_ontbrekend(&v);
    Ok(())
}

/// Randgeval T-33: één bericht, twee dossiers, één onderlinge verwijzing.
fn koppel(kluis: &mut Kluis, kenmerk: &str, verzoekkenmerk: &str, nu: DateTime<Utc>) -> Result<()> {
    let mut v = zoek(kluis, kenmerk)?;
    let kop = kluis
        .lijst("verzoek")?
        .into_iter()
        .find(|r| r.kenmerk.as_deref() == Some(verzoekkenmerk))
        .ok_or_else(|| {
            anyhow::anyhow!(
                "geen betrokkenenverzoek met kenmerk '{verzoekkenmerk}'. Bekijk de lijst met \
                 'dpofg verzoek lijst'"
            )
        })?;
    let ander: Betrokkenenverzoek = kluis.laad("verzoek", &kop.id)?;

    v.gerelateerd_verzoek_id = Some(ander.id);
    v.herkomst.wijzig(format!("gekoppeld aan verzoek {verzoekkenmerk}"), nu);
    bewaar(kluis, &v, Handeling::RecordGewijzigd, &format!("gekoppeld aan {verzoekkenmerk}"), nu)?;

    gelukt(&format!("{kenmerk} en {verzoekkenmerk} zijn aan elkaar gekoppeld"));
    terzijde(
        "Twee dossiers met twee klokken, en dat blijft zo. Samenvoegen zou betekenen dat één van \
         beide termijnen wordt genegeerd — de Woo-termijn is vier weken, die van het \
         inzageverzoek een maand.",
    );
    Ok(())
}

fn besluit(
    kluis: &mut Kluis,
    kenmerk: &str,
    uitkomst: Woouitkomst,
    op: Option<&str>,
    nu: DateTime<Utc>,
) -> Result<()> {
    let mut v = zoek(kluis, kenmerk)?;
    let moment = match op {
        Some(t) => lees_tijdstip(t)?,
        None => nu,
    };
    let actor = super::actor();
    let id = v.id.to_string();

    match v.neem_besluit(uitkomst, moment, nu) {
        Ok(()) => {
            bewaar(kluis, &v, Handeling::BesluitGenomen, "besluit genomen", nu)?;
            gelukt(&format!("besluit: {}", uitkomst.omschrijving()));
            terzijde(
                "Tegen dit besluit staat bezwaar open, en daarna beroep bij de bestuursrechter. \
                 Dat is een andere rechtsgang dan bij een inzageverzoek.",
            );
            toon_ontbrekend(&v);
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
                    v.compartiment.naam(),
                    "besluit geweigerd: er staat nog iets open",
                ),
                Some(fout.to_string()),
            )?;
            kop("Het besluit is niet vastgelegd");
            toon_ontbrekend(&v);
            anyhow::bail!("{fout}")
        }
    }
}

fn lijst(kluis: &Kluis, nu: DateTime<Utc>) -> Result<()> {
    let koppen = kluis.lijst(SOORT)?;
    if koppen.is_empty() {
        kop("Woo-verzoeken");
        terzijde("Er staan nog geen Woo-verzoeken in de kluis.");
        return Ok(());
    }

    let pakket = dpofg_content::startpakket(nu.date_naive());
    let kalender = pakket.kalender("NL").ok();
    let zone = dpofg_terms::tijdzone(dpofg_terms::TIJDZONE_NL).ok();

    kop("Woo-verzoeken");
    let mut t =
        tabel(&["kenmerk", "onderwerp", "ontvangen", "verstrijkt", "resterend", "uitkomst"]);
    for k in &koppen {
        let v: Wooverzoek = kluis.laad(SOORT, &k.id)?;
        let (verstrijkt, resterend) = match (&v.termijn, kalender, zone) {
            (Some(klok), Some(kal), Some(z)) => match klok.deadline_volledig(nu, z, kal) {
                Ok(d) => {
                    let over = d.moment - nu;
                    let tekst = if v.besluit_op.is_some() {
                        "besloten".to_string()
                    } else if over > chrono::Duration::zero() {
                        duur(over)
                    } else {
                        "verstreken".to_string()
                    };
                    (d.lokaal.clone(), tekst)
                }
                Err(_) => ("niet te berekenen".into(), "—".into()),
            },
            _ => ("nog niet gestart".into(), "—".into()),
        };
        t.add_row(vec![
            v.kenmerk.clone(),
            v.onderwerp.clone(),
            v.ontvangen_op.format("%d-%m-%Y").to_string(),
            verstrijkt,
            resterend,
            v.uitkomst.map(|u| u.omschrijving().to_string()).unwrap_or_else(|| "—".into()),
        ]);
    }
    println!("{t}");
    Ok(())
}

fn toon(kluis: &Kluis, kenmerk: &str, nu: DateTime<Utc>) -> Result<()> {
    let v = zoek(kluis, kenmerk)?;

    kop(&format!("Woo-verzoek {}", v.kenmerk));
    let mut t = tabel(&["", ""]);
    t.add_row(vec!["onderwerp", &v.onderwerp]);
    t.add_row(vec!["verzoeker", &v.verzoeker]);
    t.add_row(vec!["ontvangen op", &v.ontvangen_op.format("%d-%m-%Y %H:%M UTC").to_string()]);
    t.add_row(vec!["status", v.status.omschrijving()]);
    if let Some(u) = v.uitkomst {
        t.add_row(vec!["uitkomst", u.omschrijving()]);
    }
    if v.gerelateerd_verzoek_id.is_some() {
        t.add_row(vec!["gekoppeld", "aan een betrokkenenverzoek uit hetzelfde bericht"]);
    }
    println!("{t}");

    if let Some(klok) = &v.termijn {
        let pakket = dpofg_content::startpakket(nu.date_naive());
        let kalender = pakket.kalender("NL").context("de feestdagenkalender ontbreekt")?;
        let zone = dpofg_terms::tijdzone(dpofg_terms::TIJDZONE_NL)?;

        kop("Beslistermijn");
        let mut t = tabel(&["", ""]);
        match klok.deadline_volledig(nu, zone, kalender) {
            Ok(d) => {
                t.add_row(vec!["verstrijkt", &d.lokaal]);
                if v.besluit_op.is_none() {
                    let over = d.moment - nu;
                    let tekst = if over > chrono::Duration::zero() {
                        duur(over)
                    } else {
                        "verstreken".into()
                    };
                    t.add_row(vec!["resterend", &tekst]);
                }
            }
            Err(e) => {
                t.add_row(vec!["verstrijkt", &format!("niet te berekenen: {e}")]);
            }
        }
        if let Some(m) = v.verdaging_medegedeeld_op {
            t.add_row(vec!["verdaagd, medegedeeld", &m.format("%d-%m-%Y").to_string()]);
        }
        println!("{t}");
    }

    if !v.zienswijzen.is_empty() {
        kop("Belanghebbenden");
        let mut t = tabel(&["belanghebbende", "gelegenheid geboden", "reactie"]);
        for z in &v.zienswijzen {
            t.add_row(vec![
                z.belanghebbende.clone(),
                z.gevraagd_op
                    .map(|m| m.format("%d-%m-%Y").to_string())
                    .unwrap_or_else(|| "nog niet".into()),
                z.standpunt.clone().unwrap_or_else(|| "geen".into()),
            ]);
        }
        println!("{t}");
    }

    if !v.gronden.is_empty() {
        kop("Ingeroepen weigeringsgronden");
        for g in &v.gronden {
            let soort = if g.grond.is_relatief() { "relatief" } else { "absoluut" };
            println!("  • {} ({soort}) — {}", g.grond.omschrijving(), g.betreft);
            terzijde(g.grond.grondslag());
            if g.grond.is_relatief() && g.afweging.is_none() {
                blokkade("de afweging tegen het belang van openbaarheid ontbreekt");
            }
        }
    }

    toon_ontbrekend(&v);
    Ok(())
}

fn toon_ontbrekend(v: &Wooverzoek) {
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
        let veld = o.veld.trim_start_matches("woo.");
        if o.blokkeert_vaststelling {
            blokkade(&format!("{veld} — {}", o.omschrijving));
        } else {
            let_op(&format!("{veld} — {}", o.omschrijving));
        }
        terzijde(&o.grondslag);
    }
    println!();
    terzijde("■ houdt het besluit tegen · ▸ blijft zichtbaar maar blokkeert niet");
}
