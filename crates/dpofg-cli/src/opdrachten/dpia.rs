//! De effectbeoordeling.
//!
//! De volgorde van de opdrachten volgt de volgorde van het werk: eerst de
//! voortoets (is dit nodig?), dan de beoordeling zelf, dan het restrisico, en
//! pas als dat hoog uitvalt de raadpleging van de toezichthouder. Elke
//! schrijfactie sluit af met de stand van de volledigheid, zodat de gebruiker
//! niet pas bij het vaststellen hoort wat er ontbreekt.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use clap::Subcommand;
use dpofg_audit::Handeling;
use dpofg_domain::{Dpia, Motivering, Restrisiconiveau, Status, Verwerking, Volledig, Voortoets};
use dpofg_store::Kluis;
use std::path::PathBuf;

use crate::uitvoer::{blokkade, duur, gelukt, kop, let_op, tabel, terzijde, voortgang};

const SOORT: &str = "dpia";
/// Het dossier draagt het restrisico-oordeel, het advies van de toezichthouder
/// en de namen van wie beoordeelde. Deze waarde gaat de versleuteling in en is
/// na het eerste weggeschreven dossier niet meer te wijzigen.
const COMPARTIMENT: &str = "vertrouwelijk";

/// De vaste opschortingsgrond van artikel 36 lid 2.
///
/// De verordening kent er precies één: de toezichthouder wacht op informatie
/// die zij heeft opgevraagd. Een vrij tekstveld zou uitnodigen tot een
/// verzonnen grond, en een opschorting op een verzonnen grond is een termijn
/// die stilstaat zonder dat daar iets tegenover staat.
const OPSCHORTINGSGROND: &str =
    "informatieverzoek van de toezichthouder (art. 36 lid 2, laatste volzin, AVG)";

#[derive(Subcommand, Debug)]
pub enum Dpiaopdracht {
    /// Toon alle effectbeoordelingen.
    Lijst {
        /// Toon alleen dossiers die nog niet volledig zijn.
        #[arg(long)]
        onvolledig: bool,
    },
    /// Maak een effectbeoordeling aan bij een registerregel.
    Nieuw {
        /// Kenmerk waaronder de beoordeling bekend staat.
        kenmerk: String,
        /// Korte omschrijving.
        omschrijving: String,
        /// Het kenmerk van de registerregel waarop de beoordeling ziet.
        #[arg(long)]
        verwerking: String,
    },
    /// Toon één effectbeoordeling met de klok en wat er nog ontbreekt.
    Toon {
        /// Het kenmerk van de beoordeling.
        kenmerk: String,
    },
    /// Leg vast of een effectbeoordeling nodig is.
    Voortoets {
        /// Het kenmerk van de beoordeling.
        kenmerk: String,
        /// De uitkomst.
        #[arg(long, value_enum)]
        uitkomst: Voortoetskeuze,
        /// Waarom wel of niet. Verplicht: juist een negatief besluit moet later
        /// te volgen zijn.
        #[arg(long)]
        motivering: String,
    },
    /// Leg vast wanneer en door wie is beoordeeld.
    Uitvoeren {
        /// Het kenmerk van de beoordeling.
        kenmerk: String,
        /// Tijdstip in de vorm 2026-08-19T09:00:00Z.
        datum: String,
        /// Wie de beoordeling heeft uitgevoerd.
        #[arg(long)]
        door: String,
        /// De gebruikte methode.
        #[arg(long)]
        methode: Option<String>,
        /// Of de beoordeling vóór de verwerking is uitgevoerd.
        #[arg(long)]
        vooraf: Option<bool>,
    },
    /// Vul een onderdeel van de beoordeling.
    Vul {
        /// Het kenmerk van de beoordeling.
        kenmerk: String,
        /// Welk onderdeel: systematische-beschrijving, noodzaak-en-evenredigheid,
        /// risico, maatregel, methode of advies-fg.
        #[arg(long)]
        veld: String,
        /// De waarde.
        #[arg(long)]
        waarde: String,
    },
    /// Weeg wat er ná de maatregelen aan risico overblijft.
    Restrisico {
        /// Het kenmerk van de beoordeling.
        kenmerk: String,
        /// Het niveau.
        #[arg(long, value_enum)]
        niveau: Restrisicokeuze,
        /// De weging. Verplicht: een niveau zonder weging zegt niets.
        #[arg(long)]
        motivering: String,
    },
    /// Leg vast dat de toezichthouder om voorafgaande raadpleging is gevraagd.
    Raadpleging {
        /// Het kenmerk van de beoordeling.
        kenmerk: String,
        /// Wanneer het verzoek is ingediend, in de vorm 2026-08-19T09:00:00Z.
        ingediend_op: String,
    },
    /// Schort de raadplegingstermijn op zolang de toezichthouder op informatie wacht.
    RaadplegingOpschorten {
        /// Het kenmerk van de beoordeling.
        kenmerk: String,
        /// Vanaf welk tijdstip.
        vanaf: String,
        /// Wat er is opgevraagd.
        #[arg(long)]
        opgevraagd: Option<String>,
    },
    /// Hervat de raadplegingstermijn.
    RaadplegingHervatten {
        /// Het kenmerk van de beoordeling.
        kenmerk: String,
        /// Vanaf welk tijdstip de termijn weer loopt.
        op: String,
    },
    /// Leg vast dat de toezichthouder de termijn heeft verlengd.
    RaadplegingVerlengen {
        /// Het kenmerk van de beoordeling.
        kenmerk: String,
        /// Wanneer het bericht van verlenging is ontvangen.
        bericht_op: String,
    },
    /// Leg het advies van de toezichthouder vast.
    Advies {
        /// Het kenmerk van de beoordeling.
        kenmerk: String,
        /// Wanneer het advies is ontvangen.
        ontvangen_op: String,
        /// Het kenmerk waaronder de toezichthouder het advies uitbracht.
        #[arg(long)]
        referentie: String,
    },
    /// Stel de effectbeoordeling vast.
    Vaststellen {
        /// Het kenmerk van de beoordeling.
        kenmerk: String,
    },
}

#[derive(clap::ValueEnum, Clone, Copy, Debug)]
pub enum Voortoetskeuze {
    NietNodig,
    Vereist,
    Vrijwillig,
}

impl From<Voortoetskeuze> for Voortoets {
    fn from(k: Voortoetskeuze) -> Self {
        match k {
            Voortoetskeuze::NietNodig => Voortoets::NietNodig,
            Voortoetskeuze::Vereist => Voortoets::Vereist,
            Voortoetskeuze::Vrijwillig => Voortoets::Vrijwillig,
        }
    }
}

#[derive(clap::ValueEnum, Clone, Copy, Debug)]
pub enum Restrisicokeuze {
    Laag,
    Gemiddeld,
    Hoog,
}

impl From<Restrisicokeuze> for Restrisiconiveau {
    fn from(k: Restrisicokeuze) -> Self {
        match k {
            Restrisicokeuze::Laag => Restrisiconiveau::Laag,
            Restrisicokeuze::Gemiddeld => Restrisiconiveau::Gemiddeld,
            Restrisicokeuze::Hoog => Restrisiconiveau::Hoog,
        }
    }
}

pub fn draai(o: Dpiaopdracht, kluispad: Option<PathBuf>, nu: DateTime<Utc>) -> Result<()> {
    let pad = super::kluispad(kluispad)?;
    let mut kluis = super::open_kluis(&pad, nu)?;
    match o {
        Dpiaopdracht::Lijst { onvolledig } => lijst(&kluis, onvolledig),
        Dpiaopdracht::Nieuw { kenmerk, omschrijving, verwerking } => {
            nieuw(&mut kluis, &kenmerk, &omschrijving, &verwerking, nu)
        }
        Dpiaopdracht::Toon { kenmerk } => toon(&kluis, &kenmerk, nu),
        Dpiaopdracht::Voortoets { kenmerk, uitkomst, motivering } => {
            voortoets(&mut kluis, &kenmerk, uitkomst.into(), &motivering, nu)
        }
        Dpiaopdracht::Uitvoeren { kenmerk, datum, door, methode, vooraf } => {
            uitvoeren(&mut kluis, &kenmerk, &datum, &door, methode, vooraf, nu)
        }
        Dpiaopdracht::Vul { kenmerk, veld, waarde } => {
            vul(&mut kluis, &kenmerk, &veld, &waarde, nu)
        }
        Dpiaopdracht::Restrisico { kenmerk, niveau, motivering } => {
            restrisico(&mut kluis, &kenmerk, niveau.into(), &motivering, nu)
        }
        Dpiaopdracht::Raadpleging { kenmerk, ingediend_op } => {
            raadpleging(&mut kluis, &kenmerk, &ingediend_op, nu)
        }
        Dpiaopdracht::RaadplegingOpschorten { kenmerk, vanaf, opgevraagd } => {
            opschorten(&mut kluis, &kenmerk, &vanaf, opgevraagd.as_deref(), nu)
        }
        Dpiaopdracht::RaadplegingHervatten { kenmerk, op } => {
            hervatten(&mut kluis, &kenmerk, &op, nu)
        }
        Dpiaopdracht::RaadplegingVerlengen { kenmerk, bericht_op } => {
            verlengen(&mut kluis, &kenmerk, &bericht_op, nu)
        }
        Dpiaopdracht::Advies { kenmerk, ontvangen_op, referentie } => {
            advies(&mut kluis, &kenmerk, &ontvangen_op, &referentie, nu)
        }
        Dpiaopdracht::Vaststellen { kenmerk } => vaststellen(&mut kluis, &kenmerk, nu),
    }
}

fn zoek(kluis: &Kluis, kenmerk: &str) -> Result<Dpia> {
    let kop = kluis
        .lijst(SOORT)?
        .into_iter()
        .find(|r| r.kenmerk.as_deref() == Some(kenmerk))
        .ok_or_else(|| anyhow::anyhow!("geen effectbeoordeling met kenmerk '{kenmerk}'"))?;
    Ok(kluis.laad(SOORT, &kop.id)?)
}

fn bewaar(
    kluis: &mut Kluis,
    d: &Dpia,
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

fn lees_tijdstip(tekst: &str) -> Result<DateTime<Utc>> {
    tekst.parse::<DateTime<chrono::FixedOffset>>().map(|t| t.with_timezone(&Utc)).map_err(|e| {
        anyhow::anyhow!(
            "kon '{tekst}' niet lezen als tijdstip ({e}). Gebruik de vorm \
             2026-08-19T09:00:00Z of 2026-08-19T11:00:00+02:00"
        )
    })
}

fn lijst(kluis: &Kluis, alleen_onvolledig: bool) -> Result<()> {
    let koppen = kluis.lijst(SOORT)?;
    if koppen.is_empty() {
        kop("Effectbeoordelingen");
        terzijde("Er staan nog geen effectbeoordelingen in de kluis.");
        terzijde(
            "Maak er een aan met 'dpofg dpia nieuw <kenmerk> <omschrijving> \
             --verwerking <registerkenmerk>'.",
        );
        return Ok(());
    }

    kop("Effectbeoordelingen");
    let mut t =
        tabel(&["kenmerk", "omschrijving", "status", "voortoets", "restrisico", "volledig"]);
    for k in &koppen {
        let d: Dpia = kluis.laad(SOORT, &k.id)?;
        let r = d.volledigheid();
        if alleen_onvolledig && r.is_volledig() {
            continue;
        }
        t.add_row(vec![
            d.kenmerk.clone(),
            d.omschrijving.clone(),
            d.status.omschrijving().to_string(),
            d.voortoets.map(|v| v.omschrijving().to_string()).unwrap_or_else(|| "—".into()),
            d.restrisico
                .as_ref()
                .map(|x| x.niveau.omschrijving().to_string())
                .unwrap_or_else(|| "—".into()),
            format!("{} van {}", r.compleet, r.verplicht),
        ]);
    }
    println!("{t}");
    Ok(())
}

fn nieuw(
    kluis: &mut Kluis,
    kenmerk: &str,
    omschrijving: &str,
    verwerkingkenmerk: &str,
    nu: DateTime<Utc>,
) -> Result<()> {
    if kluis.lijst(SOORT)?.iter().any(|r| r.kenmerk.as_deref() == Some(kenmerk)) {
        anyhow::bail!("er bestaat al een effectbeoordeling met kenmerk '{kenmerk}'");
    }

    // Eerst opzoeken, dan pas schrijven: afbreken op een typefout mag geen half
    // aangelegd dossier achterlaten.
    let kop_verwerking = kluis
        .lijst("verwerking")?
        .into_iter()
        .find(|r| r.kenmerk.as_deref() == Some(verwerkingkenmerk))
        .ok_or_else(|| {
            anyhow::anyhow!(
                "geen registerregel met kenmerk '{verwerkingkenmerk}'. Bekijk de lijst met \
                 'dpofg register lijst'"
            )
        })?;
    let mut v: Verwerking = kluis.laad("verwerking", &kop_verwerking.id)?;

    // Twee effectbeoordelingen op één registerregel zou betekenen dat de
    // terugverwijzing er maar één kan aanwijzen, en dat een risicowijziging
    // stilzwijgend aan de andere voorbijgaat.
    if let Some(bestaand) = v.dpia_id {
        if let Some(k) = kluis.lijst(SOORT)?.into_iter().find(|r| r.id == bestaand.to_string()) {
            let ander: Dpia = kluis.laad(SOORT, &k.id)?;
            if ander.status != Status::Ingetrokken {
                anyhow::bail!(
                    "registerregel {verwerkingkenmerk} is al gekoppeld aan effectbeoordeling {}. \
                     Werk die bij met 'dpofg dpia toon {}', of trek haar in voordat u een \
                     nieuwe aanmaakt",
                    ander.kenmerk,
                    ander.kenmerk
                );
            }
        }
    }

    let actor = super::actor();
    let d = Dpia::nieuw(kenmerk, omschrijving, v.id, &actor.id, nu);
    bewaar(kluis, &d, Handeling::RecordAangemaakt, "effectbeoordeling aangemaakt", nu)?;

    // De terugverwijzing in het register. Er is geen schrijfactie over twee
    // dossiers heen; valt deze weg, dan blijft 'verwerking.dpia' gemeld en is
    // de toestand zichtbaar in plaats van stil. 'dpia vaststellen' herstelt hem.
    v.dpia_id = Some(d.id);
    kluis.bewaar(
        "verwerking",
        &v.id.to_string(),
        v.compartiment.naam(),
        v.status.omschrijving(),
        Some(&v.kenmerk),
        &v,
        &actor,
        Handeling::RecordGewijzigd,
        &format!("effectbeoordeling {kenmerk} gekoppeld"),
        nu,
    )?;

    gelukt(&format!("effectbeoordeling {kenmerk} aangemaakt bij {verwerkingkenmerk}"));

    let criteria = v.getelde_dpia_criteria();
    if criteria.is_empty() {
        terzijde(
            "Uit het register volgt geen enkel criterium voor deze verwerking. Dat maakt een \
             beoordeling niet zinloos — de tool ziet alleen wat er in het register staat.",
        );
    } else {
        kop("Criteria die uit het register volgen");
        for c in &criteria {
            println!("  • {c}");
        }
        terzijde(
            "De tool telt de criteria die zij kan afleiden; of een beoordeling nodig is, \
             beantwoordt u met 'dpofg dpia voortoets'.",
        );
    }

    toon_ontbrekend(&d);
    Ok(())
}

fn voortoets(
    kluis: &mut Kluis,
    kenmerk: &str,
    uitkomst: Voortoets,
    motivering: &str,
    nu: DateTime<Utc>,
) -> Result<()> {
    let mut d = zoek(kluis, kenmerk)?;
    d.voortoets_motivering = Some(Motivering::nieuw(motivering, &super::actor().id, nu)?);
    d.voortoets = Some(uitkomst);
    d.herkomst.wijzig("voortoets vastgelegd", nu);

    bewaar(
        kluis,
        &d,
        Handeling::MotiveringVastgelegd,
        &format!("voortoets: {}", uitkomst.omschrijving()),
        nu,
    )?;
    gelukt(&format!("voortoets vastgelegd: {}", uitkomst.omschrijving()));

    if uitkomst == Voortoets::NietNodig {
        terzijde(
            "Het dossier is hiermee compleet. De motivering blijft staan en gaat mee in elke \
             export: een negatief besluit hoort even goed te verantwoorden te zijn als een \
             uitgevoerde beoordeling.",
        );
    }
    toon_ontbrekend(&d);
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn uitvoeren(
    kluis: &mut Kluis,
    kenmerk: &str,
    datum: &str,
    door: &str,
    methode: Option<String>,
    vooraf: Option<bool>,
    nu: DateTime<Utc>,
) -> Result<()> {
    let mut d = zoek(kluis, kenmerk)?;
    let moment = lees_tijdstip(datum)?;
    d.leg_beoordeling_vast(moment, door, vooraf, nu)?;
    if let Some(m) = methode {
        d.methode = Some(m);
    }

    bewaar(kluis, &d, Handeling::RecordGewijzigd, "beoordeling vastgelegd", nu)?;
    gelukt(&format!("beoordeling vastgelegd op {}", crate::uitvoer::datum(moment)));

    if d.vooraf_uitgevoerd == Some(false) {
        let_op(
            "De beoordeling is uitgevoerd nadat de verwerking al liep. Dat is een feit dat blijft \
             staan; regel DPIA-03 maakt het zichtbaar. Het wordt niet weggepoetst en het is geen \
             reden om de datum aan te passen.",
        );
    }
    toon_ontbrekend(&d);
    Ok(())
}

fn vul(
    kluis: &mut Kluis,
    kenmerk: &str,
    veld: &str,
    waarde: &str,
    nu: DateTime<Utc>,
) -> Result<()> {
    let mut d = zoek(kluis, kenmerk)?;
    let schoon = waarde.trim();
    if schoon.is_empty() {
        anyhow::bail!(
            "'{veld}' mag niet leeg zijn. Een leeg veld dat als ingevuld wordt vastgelegd, \
             laat de volledigheidscontrole zwijgen zonder dat er iets is vastgelegd"
        );
    }

    match veld {
        "systematische-beschrijving" => d.systematische_beschrijving = Some(schoon.to_string()),
        "noodzaak-en-evenredigheid" => d.noodzaak_en_evenredigheid = Some(schoon.to_string()),
        "risico" => d.risicos.push(schoon.to_string()),
        "maatregel" => d.maatregelen.push(schoon.to_string()),
        "methode" => d.methode = Some(schoon.to_string()),
        "advies-fg" => {
            d.advies_functionaris = Some(Motivering::nieuw(schoon, &super::actor().id, nu)?)
        }
        andere => anyhow::bail!(
            "'{andere}' is geen veld dat via deze route te vullen is. Beschikbaar: \
             systematische-beschrijving, noodzaak-en-evenredigheid, risico, maatregel, methode, \
             advies-fg"
        ),
    }
    d.herkomst.wijzig(format!("{veld} ingevuld"), nu);

    bewaar(kluis, &d, Handeling::RecordGewijzigd, &format!("{veld} ingevuld"), nu)?;
    gelukt(&format!("{veld} vastgelegd"));

    // Is er al gewogen, dan berustte die weging op minder maatregelen.
    if (veld == "maatregel" || veld == "risico") && d.restrisico.is_some() {
        let gewogen = d.restrisico.as_ref().map(|r| r.gewogen_maatregelen).unwrap_or(0);
        if d.maatregelen.len() != gewogen {
            let_op(&format!(
                "Het restrisico is eerder gewogen tegen {gewogen} maatregel(en); er staan er nu \
                 {}. Weeg opnieuw met 'dpofg dpia restrisico' als het oordeel verandert.",
                d.maatregelen.len()
            ));
        }
    }
    toon_ontbrekend(&d);
    Ok(())
}

fn restrisico(
    kluis: &mut Kluis,
    kenmerk: &str,
    niveau: Restrisiconiveau,
    motivering: &str,
    nu: DateTime<Utc>,
) -> Result<()> {
    let mut d = zoek(kluis, kenmerk)?;
    let m = Motivering::nieuw(motivering, &super::actor().id, nu)?;
    d.stel_restrisico_vast(niveau, m, nu)?;

    bewaar(
        kluis,
        &d,
        Handeling::MotiveringVastgelegd,
        &format!("restrisico: {}", niveau.omschrijving()),
        nu,
    )?;
    gelukt(&format!("restrisico vastgesteld: {}", niveau.omschrijving()));

    if d.raadpleging_nodig() {
        println!();
        let_op(
            "Bij een hoog restrisico raadpleegt u de toezichthouder vóórdat u met de verwerking \
             begint (art. 36 lid 1 AVG). Leg het verzoek vast met 'dpofg dpia raadpleging'; \
             daarmee gaat de termijn van acht weken lopen.",
        );
    }
    toon_ontbrekend(&d);
    Ok(())
}

/// Haalt de raadplegingstermijn uit het kennispakket en start hem.
fn raadpleging(
    kluis: &mut Kluis,
    kenmerk: &str,
    ingediend_op: &str,
    nu: DateTime<Utc>,
) -> Result<()> {
    let mut d = zoek(kluis, kenmerk)?;
    let moment = lees_tijdstip(ingediend_op)?;
    if moment > nu {
        anyhow::bail!("het verzoek zou in de toekomst zijn ingediend; controleer het tijdstip");
    }

    let pakket = dpofg_content::startpakket(nu.date_naive());
    let soort = pakket
        .termijn("AVG-36-RAADPLEGING")
        .context("de raadplegingstermijn ontbreekt in het kennispakket")?
        .clone();
    let kalender =
        pakket.kalender("NL").context("de feestdagenkalender ontbreekt in het kennispakket")?;
    let zone = dpofg_terms::tijdzone(dpofg_terms::TIJDZONE_NL)?;

    let klok = dpofg_terms::LopendeTermijn::start(soort, moment, zone, kalender).context(
        "de raadplegingstermijn kon niet worden berekend; de feestdagenkalender in het \
         kennispakket reikt niet ver genoeg. 'dpofg pakket voorbehoud' toont wat er te \
         verifiëren valt",
    )?;
    let deadline = klok.deadline_volledig(nu, zone, kalender)?;

    d.dien_raadpleging_in(klok, nu)?;
    d.raadpleging_pakket = Some(format!(
        "{} {}, geconsolideerd {}",
        pakket.code,
        pakket.versienaam,
        crate::uitvoer::datum(pakket.consolidatiedatum)
    ));

    bewaar(kluis, &d, Handeling::TermijnGestart, "verzoek om voorafgaande raadpleging", nu)?;

    gelukt(&format!("verzoek ingediend op {}", crate::uitvoer::datum(moment)));
    kop("De termijn van de toezichthouder");
    let mut t = tabel(&["", ""]);
    t.add_row(vec!["duur", &deadline.duur]);
    t.add_row(vec!["verstrijkt", &deadline.lokaal]);
    t.add_row(vec!["grondslag", &deadline.grondslag]);
    println!("{t}");
    terzijde(&deadline.verantwoording);
    terzijde(
        "Deze termijn is die van de toezichthouder, niet van u. Verstrijkt hij zonder antwoord, \
         dan is dat geen goedkeuring.",
    );
    Ok(())
}

fn opschorten(
    kluis: &mut Kluis,
    kenmerk: &str,
    vanaf: &str,
    opgevraagd: Option<&str>,
    nu: DateTime<Utc>,
) -> Result<()> {
    let mut d = zoek(kluis, kenmerk)?;
    let moment = lees_tijdstip(vanaf)?;
    let grond = match opgevraagd {
        Some(wat) if !wat.trim().is_empty() => format!("{OPSCHORTINGSGROND}: {}", wat.trim()),
        _ => OPSCHORTINGSGROND.to_string(),
    };

    let actor = super::actor();
    let klok = d
        .raadpleging
        .as_mut()
        .ok_or_else(|| anyhow::anyhow!("er loopt geen raadpleging voor '{kenmerk}'"))?;
    klok.schort_op(moment, &grond, &actor.id)?;
    d.herkomst.wijzig("raadplegingstermijn opgeschort", nu);

    bewaar(kluis, &d, Handeling::TermijnGestuit, &grond, nu)?;
    gelukt(&format!("termijn opgeschort vanaf {}", crate::uitvoer::datum(moment)));
    terzijde(&grond);
    terzijde(
        "De opschorting wordt in hele kalenderdagen verrekend, zodat een overgang naar of van \
         zomertijd de einddatum niet verschuift.",
    );
    Ok(())
}

fn hervatten(kluis: &mut Kluis, kenmerk: &str, op: &str, nu: DateTime<Utc>) -> Result<()> {
    let mut d = zoek(kluis, kenmerk)?;
    let moment = lees_tijdstip(op)?;
    if moment > nu {
        anyhow::bail!("het hervatten zou in de toekomst liggen; controleer het tijdstip");
    }

    let klok = d
        .raadpleging
        .as_mut()
        .ok_or_else(|| anyhow::anyhow!("er loopt geen raadpleging voor '{kenmerk}'"))?;
    klok.hervat(moment)?;
    d.herkomst.wijzig("raadplegingstermijn hervat", nu);

    // Eerst rekenen, dan pas wegschrijven. Faalt de berekening — bijvoorbeeld
    // omdat de nieuwe einddatum buiten de feestdagenkalender valt — dan blijft
    // het dossier zoals het was, in plaats van dat er een klok in de kluis
    // belandt die niemand meer kan tonen.
    let deadline = bereken_deadline(&d, nu)?;

    bewaar(kluis, &d, Handeling::TermijnHervat, "raadplegingstermijn hervat", nu)?;
    gelukt(&format!("termijn hervat op {}", crate::uitvoer::datum(moment)));
    terzijde(&format!("verstrijkt nu op {}", deadline.lokaal));
    toon_klok(&d, nu)?;
    Ok(())
}

fn verlengen(kluis: &mut Kluis, kenmerk: &str, bericht_op: &str, nu: DateTime<Utc>) -> Result<()> {
    let mut d = zoek(kluis, kenmerk)?;
    let moment = lees_tijdstip(bericht_op)?;
    if moment > nu {
        anyhow::bail!("het bericht zou in de toekomst zijn ontvangen; controleer het tijdstip");
    }

    let pakket = dpofg_content::startpakket(nu.date_naive());
    let kalender = pakket.kalender("NL").context("de feestdagenkalender ontbreekt")?;
    let zone = dpofg_terms::tijdzone(dpofg_terms::TIJDZONE_NL)?;

    let klok = d
        .raadpleging
        .as_mut()
        .ok_or_else(|| anyhow::anyhow!("er loopt geen raadpleging voor '{kenmerk}'"))?;
    klok.verleng(moment, zone, kalender)?;
    d.herkomst.wijzig("raadplegingstermijn verlengd", nu);

    // Zie `hervatten`: eerst rekenen, dan pas wegschrijven.
    let deadline = bereken_deadline(&d, nu)?;

    bewaar(kluis, &d, Handeling::TermijnVerlengd, "raadplegingstermijn verlengd", nu)?;
    gelukt("verlenging vastgelegd");
    terzijde(&format!("verstrijkt nu op {}", deadline.lokaal));
    toon_klok(&d, nu)?;
    Ok(())
}

/// Berekent de einddatum van de lopende raadplegingstermijn.
fn bereken_deadline(d: &Dpia, nu: DateTime<Utc>) -> Result<dpofg_terms::Deadline> {
    let klok = d
        .raadpleging
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("er loopt geen raadpleging voor dit dossier"))?;
    let pakket = dpofg_content::startpakket(nu.date_naive());
    let kalender = pakket.kalender("NL").context("de feestdagenkalender ontbreekt")?;
    let zone = dpofg_terms::tijdzone(dpofg_terms::TIJDZONE_NL)?;
    Ok(klok.deadline_volledig(nu, zone, kalender)?)
}

fn advies(
    kluis: &mut Kluis,
    kenmerk: &str,
    ontvangen_op: &str,
    referentie: &str,
    nu: DateTime<Utc>,
) -> Result<()> {
    let mut d = zoek(kluis, kenmerk)?;
    let moment = lees_tijdstip(ontvangen_op)?;
    d.leg_advies_vast(moment, referentie, nu)?;

    bewaar(kluis, &d, Handeling::BesluitGenomen, &format!("advies {referentie} vastgelegd"), nu)?;
    gelukt(&format!("advies {referentie} vastgelegd op {}", crate::uitvoer::datum(moment)));
    toon_ontbrekend(&d);
    Ok(())
}

fn vaststellen(kluis: &mut Kluis, kenmerk: &str, nu: DateTime<Utc>) -> Result<()> {
    let mut d = zoek(kluis, kenmerk)?;
    let actor = super::actor();
    let id = d.id.to_string();

    match d.stel_vast(&actor.naam, nu) {
        Ok(()) => {
            bewaar(kluis, &d, Handeling::RecordVastgesteld, "effectbeoordeling vastgesteld", nu)?;

            // De terugverwijzing idempotent herstellen: is die bij het aanmaken
            // weggevallen, dan komt hij hier alsnog goed.
            herstel_koppeling(kluis, &d, nu)?;

            gelukt(&format!("effectbeoordeling {kenmerk} vastgesteld"));
            let r = d.volledigheid();
            if !r.ontbreekt.is_empty() {
                println!();
                let_op(&format!(
                    "Er staan nog {} onderdelen open die vaststellen niet tegenhouden. Zij \
                     blijven zichtbaar in het dossier en in elke export.",
                    r.ontbreekt.len()
                ));
            }
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

/// Zet de verwijzing vanuit het register terug als die ontbreekt.
fn herstel_koppeling(kluis: &mut Kluis, d: &Dpia, nu: DateTime<Utc>) -> Result<()> {
    let Some(kop) =
        kluis.lijst("verwerking")?.into_iter().find(|r| r.id == d.verwerking_id.to_string())
    else {
        return Ok(());
    };
    let mut v: Verwerking = kluis.laad("verwerking", &kop.id)?;
    match v.dpia_id {
        Some(id) if id == d.id => return Ok(()),
        // Nooit een bestaande koppeling overschrijven: dan zou het ene dossier
        // het andere stilzwijgend van zijn registerregel afhalen, en zou de
        // herzieningsmelding bij een wijziging naar het verkeerde dossier gaan.
        Some(_) => anyhow::bail!(
            "registerregel {} is al gekoppeld aan een andere effectbeoordeling; koppel die \
             eerst los",
            v.kenmerk
        ),
        None => {}
    }
    v.dpia_id = Some(d.id);
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
        &format!("effectbeoordeling {} gekoppeld", d.kenmerk),
        nu,
    )?;
    Ok(())
}

/// Markeert de effectbeoordeling bij een registerregel als te herzien.
///
/// Staat hier en niet in `register.rs`, zodat de kennis over dit dossier in
/// deze module blijft.
pub fn herziening_nodig_na_registerwijziging(
    kluis: &mut Kluis,
    v: &Verwerking,
    reden: &str,
    nu: DateTime<Utc>,
) -> Result<Vec<String>> {
    // Over de dossiers lopen en niet over de terugverwijzing: die wijst er maar
    // één aan, en een dossier dat niet is teruggekoppeld hoort even goed te
    // worden gemarkeerd.
    let mut gemarkeerd = Vec::new();
    for kop in kluis.lijst(SOORT)? {
        let d: Dpia = kluis.laad(SOORT, &kop.id)?;
        if d.verwerking_id != v.id || d.status != Status::Vastgesteld {
            continue;
        }
        let mut d = d;
        d.markeer_herziening_nodig(reden, nu);
        bewaar(kluis, &d, Handeling::RecordGewijzigd, "herziening nodig na registerwijziging", nu)?;
        gemarkeerd.push(d.kenmerk);
    }
    Ok(gemarkeerd)
}

fn toon(kluis: &Kluis, kenmerk: &str, nu: DateTime<Utc>) -> Result<()> {
    let d = zoek(kluis, kenmerk)?;

    kop(&format!("Effectbeoordeling {}", d.kenmerk));
    let mut t = tabel(&["", ""]);
    t.add_row(vec!["omschrijving", &d.omschrijving]);
    t.add_row(vec!["status", d.status.omschrijving()]);
    t.add_row(vec![
        "voortoets",
        d.voortoets.map(|v| v.omschrijving()).unwrap_or("nog niet beantwoord"),
    ]);
    if let Some(datum) = d.datum {
        t.add_row(vec!["beoordeeld op", &crate::uitvoer::datum(datum).to_string()]);
    }
    if let Some(door) = &d.uitgevoerd_door {
        t.add_row(vec!["uitgevoerd door", door]);
    }
    if let Some(m) = &d.methode {
        t.add_row(vec!["methode", m]);
    }
    if let Some(r) = &d.restrisico {
        t.add_row(vec![
            "restrisico",
            &format!(
                "{} (gewogen tegen {} maatregelen)",
                r.niveau.omschrijving(),
                r.gewogen_maatregelen
            ),
        ]);
    }
    println!("{t}");

    if !d.risicos.is_empty() {
        kop("Risico's voor rechten en vrijheden");
        for r in &d.risicos {
            println!("  • {r}");
        }
        terzijde("art. 35 lid 7 onder c AVG");
    }
    if !d.maatregelen.is_empty() {
        kop("Maatregelen");
        for m in &d.maatregelen {
            println!("  • {m}");
        }
        terzijde("art. 35 lid 7 onder d AVG");
    }

    toon_klok(&d, nu)?;
    toon_ontbrekend(&d);
    Ok(())
}

fn toon_klok(d: &Dpia, nu: DateTime<Utc>) -> Result<()> {
    let Some(klok) = &d.raadpleging else { return Ok(()) };

    let pakket = dpofg_content::startpakket(nu.date_naive());
    let kalender = pakket.kalender("NL").context("de feestdagenkalender ontbreekt")?;
    let zone = dpofg_terms::tijdzone(dpofg_terms::TIJDZONE_NL)?;

    // Een klok die niet te berekenen is, mag het tonen van het dossier niet
    // tegenhouden: dan zou een dossier onleesbaar worden door iets wat er
    // uitsluitend omheen zit.
    let deadline = klok.deadline_volledig(nu, zone, kalender);
    let status = klok.status(nu, zone, kalender);

    kop("Voorafgaande raadpleging");
    let mut t = tabel(&["", ""]);
    t.add_row(vec!["ingediend op", &crate::uitvoer::datum(klok.anker).to_string()]);
    t.add_row(vec![
        "stand",
        match &status {
            Ok(dpofg_terms::Termijnstatus::Loopt) => "loopt",
            Ok(dpofg_terms::Termijnstatus::Opgeschort) => "opgeschort",
            Ok(dpofg_terms::Termijnstatus::Verstreken) => "verstreken zonder advies",
            Ok(dpofg_terms::Termijnstatus::Afgerond) => "afgerond met advies",
            Err(_) => "niet te bepalen",
        },
    ]);
    match &deadline {
        Ok(d) => t.add_row(vec!["verstrijkt", &d.lokaal]),
        Err(e) => t.add_row(vec!["verstrijkt", &format!("niet te berekenen: {e}")]),
    };
    if let (Ok(d), None) = (&deadline, klok.afgerond_op) {
        let resterend = d.moment - nu;
        if resterend > chrono::Duration::zero() {
            t.add_row(vec!["resterend", &duur(resterend)]);
        }
    }
    if klok.keer_verlengd > 0 {
        t.add_row(vec!["verlengd", &format!("{} keer", klok.keer_verlengd)]);
    }
    if let Some(op) = klok.afgerond_op {
        t.add_row(vec!["advies ontvangen", &crate::uitvoer::datum(op).to_string()]);
    }
    if let Some(r) = &d.advies_referentie {
        t.add_row(vec!["kenmerk advies", r]);
    }
    println!("{t}");

    for o in &klok.opschortingen {
        let tot = o
            .tot
            .map(|t| crate::uitvoer::datum(t).to_string())
            .unwrap_or_else(|| "loopt nog".into());
        println!("  opgeschort van {} tot {}", crate::uitvoer::datum(o.van), tot);
        terzijde(&o.grond);
    }

    match &deadline {
        Ok(d) => terzijde(&d.verantwoording),
        Err(_) => let_op(
            "De einddatum is met het huidige kennispakket niet te berekenen; werk het \
             kennispakket bij. 'dpofg pakket voorbehoud' toont wat er te verifiëren valt.",
        ),
    }
    if let Some(p) = &d.raadpleging_pakket {
        terzijde(&format!("gerekend op kennispakket {p}"));
    }
    Ok(())
}

fn toon_ontbrekend(d: &Dpia) {
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
        let veld = o.veld.trim_start_matches("dpia.");
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
