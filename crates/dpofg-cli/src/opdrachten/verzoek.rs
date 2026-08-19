//! Verzoeken van betrokkenen.
//!
//! De opdrachten volgen de volgorde van het werk: intake, de lezing van de
//! termijn, de klok, de vindplaatsen uit het register, de kennisgevingen aan
//! ontvangers, en pas dan de afhandeling. Elke schrijfactie sluit af met de
//! stand van de volledigheid.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use clap::Subcommand;
use dpofg_audit::Handeling;
use dpofg_domain::{
    verzoek::{
        Anonimiseringstoets, Betrokkenenverzoek, Termijnlezing, Verlengingsgrond, Verzoekkanaal,
        Verzoeksoort, Verzoekuitkomst, Vindplaatsuitkomst,
    },
    Motivering, Status, Verwerking, Volledig,
};
use dpofg_store::Kluis;
use std::path::PathBuf;

use crate::uitvoer::{blokkade, duur, gelukt, kop, let_op, tabel, terzijde, voortgang};

const SOORT: &str = "verzoek";
/// Een verzoekdossier draagt de identiteit van de betrokkene en wat er over hem
/// is vastgelegd. Deze waarde gaat de versleuteling in en is na het eerste
/// weggeschreven dossier niet meer te wijzigen.
const COMPARTIMENT: &str = "vertrouwelijk";

#[derive(Subcommand, Debug)]
pub enum Verzoekopdracht {
    /// Toon alle verzoeken met hun termijn.
    Lijst {
        /// Toon alleen verzoeken die nog niet volledig zijn.
        #[arg(long)]
        onvolledig: bool,
    },
    /// Registreer een verzoek. Doe dit als eerste, vul de rest daarna aan.
    Nieuw {
        /// Kenmerk waaronder het verzoek bekend staat.
        kenmerk: String,
        /// Korte omschrijving.
        omschrijving: String,
        /// Welk recht wordt ingeroepen.
        #[arg(long, value_enum)]
        soort: Soortkeuze,
        /// Langs welke weg het binnenkwam. Elk kanaal is geldig.
        #[arg(long, value_enum, default_value = "email")]
        kanaal: Kanaalkeuze,
        /// Wanneer het verzoek is ontvangen. Standaard: nu.
        #[arg(long)]
        ontvangen: Option<String>,
        /// Wie het verzoek behandelt.
        #[arg(long, default_value = "")]
        behandelaar: String,
    },
    /// Toon één verzoek met zijn termijn en wat er nog ontbreekt.
    Toon {
        /// Het kenmerk van het verzoek.
        kenmerk: String,
    },
    /// Leg vast wanneer de identiteit van de betrokkene is vastgesteld.
    Identiteit {
        /// Het kenmerk van het verzoek.
        kenmerk: String,
        /// Tijdstip in de vorm 2026-08-19T09:00:00Z.
        tijdstip: String,
    },
    /// Toon beide lezingen van de termijn, met hun bron.
    Lezingen,
    /// Kies vanaf welk moment de termijn loopt.
    Lezing {
        /// Het kenmerk van het verzoek.
        kenmerk: String,
        /// De gekozen lezing.
        #[arg(long, value_enum)]
        lezing: Lezingkeuzeoptie,
        /// Waarom deze lezing. Verplicht: het punt is omstreden.
        #[arg(long)]
        motivering: String,
    },
    /// Start de termijn van één maand op het gekozen anker.
    Termijn {
        /// Het kenmerk van het verzoek.
        kenmerk: String,
    },
    /// Leg de verlenging vast. Onbereikbaar zonder wettelijke grond.
    Verlengen {
        /// Het kenmerk van het verzoek.
        kenmerk: String,
        /// Wanneer de verlenging aan de betrokkene is medegedeeld.
        medegedeeld_op: String,
        /// De wettelijke grond. Er zijn er precies twee.
        #[arg(long, value_enum)]
        grond: Grondkeuze,
        /// De onderbouwing.
        #[arg(long)]
        motivering: String,
    },
    /// Leid uit het register af waar gegevens kunnen staan.
    Vindplaatsen {
        /// Het kenmerk van het verzoek.
        kenmerk: String,
        /// Neem ook registerregels met de status concept mee.
        #[arg(long)]
        met_concepten: bool,
    },
    /// Leg vast wat er op één vindplaats met de gegevens is gebeurd.
    Vindplaats {
        /// Het kenmerk van het verzoek.
        kenmerk: String,
        /// Het kenmerk van de registerregel.
        #[arg(long)]
        plaats: String,
        /// De uitkomst.
        #[arg(long, value_enum)]
        uitkomst: Uitkomstkeuze,
        /// Toelichting bij de uitkomst.
        #[arg(long)]
        toelichting: Option<String>,
        /// Bij 'geanonimiseerd': is de betrokkene niet meer uit de gegevens te lichten?
        #[arg(long)]
        singling_out_uitgesloten: bool,
        /// Bij 'geanonimiseerd': is koppeling met andere gegevens uitgesloten?
        #[arg(long)]
        koppelbaarheid_uitgesloten: bool,
        /// Bij 'geanonimiseerd': is afleiding van eigenschappen uitgesloten?
        #[arg(long)]
        afleidbaarheid_uitgesloten: bool,
        /// Bij 'geanonimiseerd': de onderbouwing van de toets.
        #[arg(long)]
        toets_motivering: Option<String>,
        /// Bij 'geanonimiseerd': de tweede persoon die de toets bevestigt.
        #[arg(long)]
        tweede_persoon: Option<String>,
    },
    /// Voeg een ontvanger toe die op grond van artikel 19 bericht moet krijgen.
    Ontvanger {
        /// Het kenmerk van het verzoek.
        kenmerk: String,
        /// De ontvanger.
        #[arg(long)]
        naam: String,
    },
    /// Leg vast dat een ontvanger is bericht, of waarom dat niet kan.
    Kennisgeving {
        /// Het kenmerk van het verzoek.
        kenmerk: String,
        /// De ontvanger.
        #[arg(long)]
        naam: String,
        /// Wanneer de kennisgeving is verzonden.
        #[arg(long)]
        verzonden: Option<String>,
        /// Op welke wijze.
        #[arg(long)]
        wijze: Option<String>,
        /// De kennisgeving is onmogelijk of kost onevenredig veel moeite.
        #[arg(long)]
        onmogelijk: bool,
        /// Waarom dat zo is. Verplicht bij --onmogelijk.
        #[arg(long)]
        motivering: Option<String>,
    },
    /// Leg het bericht van artikel 12 lid 4 vast bij een weigering.
    BerichtLid4 {
        /// Het kenmerk van het verzoek.
        kenmerk: String,
        /// Wanneer het bericht is verzonden.
        verzonden_op: String,
        /// De redenen die aan de betrokkene zijn gegeven.
        #[arg(long)]
        redenen: String,
        /// Het bericht noemt het klachtrecht bij de toezichthouder.
        #[arg(long)]
        klachtrecht: bool,
        /// Het bericht noemt de mogelijkheid van beroep bij de rechter.
        #[arg(long)]
        rechtsmiddel: bool,
    },
    /// Handel het verzoek af.
    Afhandelen {
        /// Het kenmerk van het verzoek.
        kenmerk: String,
        /// De uitkomst.
        #[arg(long, value_enum)]
        uitkomst: Verzoekuitkomstkeuze,
        /// Wanneer. Standaard: nu.
        #[arg(long)]
        op: Option<String>,
    },
}

#[derive(clap::ValueEnum, Clone, Copy, Debug)]
pub enum Soortkeuze {
    Inzage,
    Rectificatie,
    Wissing,
    Beperking,
    Overdraagbaarheid,
    Bezwaar,
    GeautomatiseerdBesluit,
}

impl From<Soortkeuze> for Verzoeksoort {
    fn from(k: Soortkeuze) -> Self {
        match k {
            Soortkeuze::Inzage => Verzoeksoort::Inzage,
            Soortkeuze::Rectificatie => Verzoeksoort::Rectificatie,
            Soortkeuze::Wissing => Verzoeksoort::Wissing,
            Soortkeuze::Beperking => Verzoeksoort::Beperking,
            Soortkeuze::Overdraagbaarheid => Verzoeksoort::Overdraagbaarheid,
            Soortkeuze::Bezwaar => Verzoeksoort::Bezwaar,
            Soortkeuze::GeautomatiseerdBesluit => Verzoeksoort::GeautomatiseerdBesluit,
        }
    }
}

#[derive(clap::ValueEnum, Clone, Copy, Debug)]
pub enum Kanaalkeuze {
    Post,
    Email,
    Telefonisch,
    Balie,
    Portaal,
    SocialeMedia,
    Anders,
}

impl From<Kanaalkeuze> for Verzoekkanaal {
    fn from(k: Kanaalkeuze) -> Self {
        match k {
            Kanaalkeuze::Post => Verzoekkanaal::Post,
            Kanaalkeuze::Email => Verzoekkanaal::Email,
            Kanaalkeuze::Telefonisch => Verzoekkanaal::Telefonisch,
            Kanaalkeuze::Balie => Verzoekkanaal::Balie,
            Kanaalkeuze::Portaal => Verzoekkanaal::Portaal,
            Kanaalkeuze::SocialeMedia => Verzoekkanaal::SocialeMedia,
            Kanaalkeuze::Anders => Verzoekkanaal::Anders,
        }
    }
}

#[derive(clap::ValueEnum, Clone, Copy, Debug)]
pub enum Lezingkeuzeoptie {
    VanafOntvangst,
    VanafIdentiteit,
}

impl From<Lezingkeuzeoptie> for Termijnlezing {
    fn from(k: Lezingkeuzeoptie) -> Self {
        match k {
            Lezingkeuzeoptie::VanafOntvangst => Termijnlezing::VanafOntvangst,
            Lezingkeuzeoptie::VanafIdentiteit => Termijnlezing::VanafIdentiteitsvaststelling,
        }
    }
}

#[derive(clap::ValueEnum, Clone, Copy, Debug)]
pub enum Grondkeuze {
    Complexiteit,
    AantalVerzoeken,
}

impl From<Grondkeuze> for Verlengingsgrond {
    fn from(k: Grondkeuze) -> Self {
        match k {
            Grondkeuze::Complexiteit => Verlengingsgrond::Complexiteit,
            Grondkeuze::AantalVerzoeken => Verlengingsgrond::AantalVerzoeken,
        }
    }
}

#[derive(clap::ValueEnum, Clone, Copy, Debug)]
pub enum Uitkomstkeuze {
    Verstrekt,
    Gerectificeerd,
    Verwijderd,
    Beperkt,
    Geanonimiseerd,
    Gepseudonimiseerd,
    NietAangetroffen,
    Geweigerd,
}

impl From<Uitkomstkeuze> for Vindplaatsuitkomst {
    fn from(k: Uitkomstkeuze) -> Self {
        match k {
            Uitkomstkeuze::Verstrekt => Vindplaatsuitkomst::Verstrekt,
            Uitkomstkeuze::Gerectificeerd => Vindplaatsuitkomst::Gerectificeerd,
            Uitkomstkeuze::Verwijderd => Vindplaatsuitkomst::Verwijderd,
            Uitkomstkeuze::Beperkt => Vindplaatsuitkomst::Beperkt,
            Uitkomstkeuze::Geanonimiseerd => Vindplaatsuitkomst::Geanonimiseerd,
            Uitkomstkeuze::Gepseudonimiseerd => Vindplaatsuitkomst::Gepseudonimiseerd,
            Uitkomstkeuze::NietAangetroffen => Vindplaatsuitkomst::NietAangetroffen,
            Uitkomstkeuze::Geweigerd => Vindplaatsuitkomst::Geweigerd,
        }
    }
}

#[derive(clap::ValueEnum, Clone, Copy, Debug)]
pub enum Verzoekuitkomstkeuze {
    Voldaan,
    DeelsVoldaan,
    Geweigerd,
}

impl From<Verzoekuitkomstkeuze> for Verzoekuitkomst {
    fn from(k: Verzoekuitkomstkeuze) -> Self {
        match k {
            Verzoekuitkomstkeuze::Voldaan => Verzoekuitkomst::Voldaan,
            Verzoekuitkomstkeuze::DeelsVoldaan => Verzoekuitkomst::DeelsVoldaan,
            Verzoekuitkomstkeuze::Geweigerd => Verzoekuitkomst::Geweigerd,
        }
    }
}

pub fn draai(o: Verzoekopdracht, kluispad: Option<PathBuf>, nu: DateTime<Utc>) -> Result<()> {
    // De lezingen zijn voorlichting en vergen geen kluis.
    if matches!(o, Verzoekopdracht::Lezingen) {
        return toon_lezingen();
    }

    let pad = super::kluispad(kluispad)?;
    let mut kluis = super::open_kluis(&pad, nu)?;
    match o {
        Verzoekopdracht::Lezingen => unreachable!("hierboven afgehandeld"),
        Verzoekopdracht::Lijst { onvolledig } => lijst(&kluis, onvolledig, nu),
        Verzoekopdracht::Nieuw { kenmerk, omschrijving, soort, kanaal, ontvangen, behandelaar } => {
            nieuw(
                &mut kluis,
                &kenmerk,
                &omschrijving,
                soort.into(),
                kanaal.into(),
                ontvangen.as_deref(),
                &behandelaar,
                nu,
            )
        }
        Verzoekopdracht::Toon { kenmerk } => toon(&kluis, &kenmerk, nu),
        Verzoekopdracht::Identiteit { kenmerk, tijdstip } => {
            identiteit(&mut kluis, &kenmerk, &tijdstip, nu)
        }
        Verzoekopdracht::Lezing { kenmerk, lezing, motivering } => {
            kies_lezing(&mut kluis, &kenmerk, lezing.into(), &motivering, nu)
        }
        Verzoekopdracht::Termijn { kenmerk } => start_termijn(&mut kluis, &kenmerk, nu),
        Verzoekopdracht::Verlengen { kenmerk, medegedeeld_op, grond, motivering } => {
            verlengen(&mut kluis, &kenmerk, &medegedeeld_op, grond.into(), &motivering, nu)
        }
        Verzoekopdracht::Vindplaatsen { kenmerk, met_concepten } => {
            vindplaatsen(&mut kluis, &kenmerk, met_concepten, nu)
        }
        Verzoekopdracht::Vindplaats {
            kenmerk,
            plaats,
            uitkomst,
            toelichting,
            singling_out_uitgesloten,
            koppelbaarheid_uitgesloten,
            afleidbaarheid_uitgesloten,
            toets_motivering,
            tweede_persoon,
        } => vindplaats(
            &mut kluis,
            &kenmerk,
            &plaats,
            uitkomst.into(),
            toelichting,
            Toetsinvoer {
                singling_out_uitgesloten,
                koppelbaarheid_uitgesloten,
                afleidbaarheid_uitgesloten,
                motivering: toets_motivering,
                tweede_persoon,
            },
            nu,
        ),
        Verzoekopdracht::Ontvanger { kenmerk, naam } => ontvanger(&mut kluis, &kenmerk, &naam, nu),
        Verzoekopdracht::Kennisgeving {
            kenmerk,
            naam,
            verzonden,
            wijze,
            onmogelijk,
            motivering,
        } => kennisgeving(
            &mut kluis,
            &kenmerk,
            &naam,
            verzonden.as_deref(),
            wijze,
            onmogelijk,
            motivering.as_deref(),
            nu,
        ),
        Verzoekopdracht::BerichtLid4 {
            kenmerk,
            verzonden_op,
            redenen,
            klachtrecht,
            rechtsmiddel,
        } => bericht_lid4(
            &mut kluis,
            &kenmerk,
            &verzonden_op,
            &redenen,
            klachtrecht,
            rechtsmiddel,
            nu,
        ),
        Verzoekopdracht::Afhandelen { kenmerk, uitkomst, op } => {
            afhandelen(&mut kluis, &kenmerk, uitkomst.into(), op.as_deref(), nu)
        }
    }
}

/// De invoer voor de anonimiseringstoets, bij elkaar gehouden.
struct Toetsinvoer {
    singling_out_uitgesloten: bool,
    koppelbaarheid_uitgesloten: bool,
    afleidbaarheid_uitgesloten: bool,
    motivering: Option<String>,
    tweede_persoon: Option<String>,
}

fn zoek(kluis: &Kluis, kenmerk: &str) -> Result<Betrokkenenverzoek> {
    let kop = kluis
        .lijst(SOORT)?
        .into_iter()
        .find(|r| r.kenmerk.as_deref() == Some(kenmerk))
        .ok_or_else(|| anyhow::anyhow!("geen verzoek met kenmerk '{kenmerk}'"))?;
    Ok(kluis.laad(SOORT, &kop.id)?)
}

fn bewaar(
    kluis: &mut Kluis,
    v: &Betrokkenenverzoek,
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
            "kon '{tekst}' niet lezen als tijdstip ({e}). Gebruik de vorm \
             2026-08-19T09:00:00Z of 2026-08-19T11:00:00+02:00"
        )
    })
}

/// De twee lezingen naast elkaar, met hun bron.
///
/// Voorlichting en geen keuze: het projectplan bindt de tool eraan beide
/// lezingen aan te bieden in plaats van er één in de motor te bakken.
fn toon_lezingen() -> Result<()> {
    kop("Vanaf welk moment loopt de maand?");
    terzijde(
        "Hierover wordt verschillend gedacht. De tool kiest niet voor u; zij legt uw keuze met \
         motivering vast in het dossier, zodat zij later te volgen is.",
    );
    for lezing in Termijnlezing::alle() {
        println!();
        println!("  \x1b[1m{}\x1b[0m", lezing.omschrijving());
        terzijde(lezing.bron());
    }
    println!();
    let_op(
        "De ruimste lezing voor de betrokkene is tevens de veiligste voor de organisatie: wie \
         vanaf ontvangst rekent, kan nooit te laat zijn omdat hij van een later moment uitging.",
    );
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn nieuw(
    kluis: &mut Kluis,
    kenmerk: &str,
    omschrijving: &str,
    soort: Verzoeksoort,
    kanaal: Verzoekkanaal,
    ontvangen: Option<&str>,
    behandelaar: &str,
    nu: DateTime<Utc>,
) -> Result<()> {
    if kluis.lijst(SOORT)?.iter().any(|r| r.kenmerk.as_deref() == Some(kenmerk)) {
        anyhow::bail!("er bestaat al een verzoek met kenmerk '{kenmerk}'");
    }
    let ontvangen_op = match ontvangen {
        Some(t) => lees_tijdstip(t)?,
        None => nu,
    };
    if ontvangen_op > nu {
        anyhow::bail!("het verzoek zou in de toekomst zijn ontvangen; controleer het tijdstip");
    }

    let actor = super::actor();
    let wie = if behandelaar.trim().is_empty() { actor.naam.clone() } else { behandelaar.into() };
    let v = Betrokkenenverzoek::nieuw(
        kenmerk,
        omschrijving,
        soort,
        kanaal,
        ontvangen_op,
        wie,
        &actor.id,
        nu,
    );
    bewaar(kluis, &v, Handeling::RecordAangemaakt, "verzoek geregistreerd", nu)?;

    gelukt(&format!("verzoek {kenmerk} geregistreerd"));
    let mut t = tabel(&["", ""]);
    t.add_row(vec!["soort", soort.omschrijving()]);
    t.add_row(vec!["grondslag", soort.grondslag()]);
    t.add_row(vec!["kanaal", kanaal.omschrijving()]);
    t.add_row(vec!["ontvangen op", &crate::uitvoer::tijdstip(ontvangen_op).to_string()]);
    println!("{t}");

    if soort.vraagt_kennisgeving_aan_ontvangers() {
        let_op(
            "Wordt dit verzoek gehonoreerd, dan moet elke ontvanger van de gegevens bericht \
             krijgen (art. 19 AVG). Het verzoek is niet af te sluiten zolang er één openstaat \
             zonder bericht en zonder reden waarom dat niet kan.",
        );
    }
    terzijde(
        "Kies met 'dpofg verzoek lezing' vanaf welk moment de termijn loopt; \
         'dpofg verzoek lezingen' toont beide lezingen met hun bron.",
    );
    toon_ontbrekend(&v);
    Ok(())
}

fn identiteit(kluis: &mut Kluis, kenmerk: &str, tijdstip: &str, nu: DateTime<Utc>) -> Result<()> {
    let mut v = zoek(kluis, kenmerk)?;
    let moment = lees_tijdstip(tijdstip)?;
    v.stel_identiteit_vast(moment, nu)?;

    bewaar(kluis, &v, Handeling::RecordGewijzigd, "identiteit vastgesteld", nu)?;
    gelukt(&format!("identiteit vastgesteld op {}", crate::uitvoer::datum(moment)));
    toon_ontbrekend(&v);
    Ok(())
}

fn kies_lezing(
    kluis: &mut Kluis,
    kenmerk: &str,
    lezing: Termijnlezing,
    motivering: &str,
    nu: DateTime<Utc>,
) -> Result<()> {
    let mut v = zoek(kluis, kenmerk)?;
    let m = Motivering::nieuw(motivering, &super::actor().id, nu)?;
    v.kies_lezing(lezing, m, nu)?;

    bewaar(
        kluis,
        &v,
        Handeling::MotiveringVastgelegd,
        &format!("lezing van de termijn: {}", lezing.omschrijving()),
        nu,
    )?;
    gelukt(&format!("lezing vastgelegd: {}", lezing.omschrijving()));
    terzijde(lezing.bron());
    toon_ontbrekend(&v);
    Ok(())
}

fn start_termijn(kluis: &mut Kluis, kenmerk: &str, nu: DateTime<Utc>) -> Result<()> {
    let mut v = zoek(kluis, kenmerk)?;
    let Some(anker) = v.anker() else {
        anyhow::bail!(
            "kies eerst vanaf welk moment de termijn loopt met 'dpofg verzoek lezing'; die keuze \
             is omstreden en wordt daarom met motivering vastgelegd"
        );
    };

    let pakket = dpofg_content::startpakket(nu.date_naive());
    let soort = pakket
        .termijn("AVG-12-3-VERZOEK")
        .context("de verzoektermijn ontbreekt in het kennispakket")?
        .clone();
    let kalender = pakket.kalender("NL").context("de feestdagenkalender ontbreekt")?;
    let zone = dpofg_terms::tijdzone(dpofg_terms::TIJDZONE_NL)?;

    let klok = dpofg_terms::LopendeTermijn::start(soort, anker, zone, kalender).context(
        "de termijn kon niet worden berekend; de feestdagenkalender in het kennispakket reikt \
         niet ver genoeg",
    )?;
    let deadline = klok.deadline_volledig(nu, zone, kalender)?;

    v.start_termijn(klok, nu)?;
    v.termijn_pakket = Some(format!(
        "{} {}, geconsolideerd {}",
        pakket.code,
        pakket.versienaam,
        crate::uitvoer::datum(pakket.consolidatiedatum)
    ));

    bewaar(kluis, &v, Handeling::TermijnGestart, "termijn gestart", nu)?;

    gelukt("de termijn loopt");
    let mut t = tabel(&["", ""]);
    t.add_row(vec!["anker", &crate::uitvoer::datum(anker).to_string()]);
    t.add_row(vec!["duur", &deadline.duur]);
    t.add_row(vec!["verstrijkt", &deadline.lokaal]);
    t.add_row(vec!["grondslag", &deadline.grondslag]);
    println!("{t}");
    terzijde(&deadline.verantwoording);
    let_op(
        "Een verlenging moet binnen deze eerste maand aan de betrokkene worden medegedeeld. \
         Daarna is zij niet meer in te roepen; de tool weigert haar dan.",
    );
    toon_ontbrekend(&v);
    Ok(())
}

fn verlengen(
    kluis: &mut Kluis,
    kenmerk: &str,
    medegedeeld_op: &str,
    grond: Verlengingsgrond,
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

    // De motor weigert een mededeling die na de oorspronkelijke termijn komt;
    // het dossier eist daarbovenop een van de twee wettelijke gronden.
    let klok = v
        .termijn
        .as_mut()
        .ok_or_else(|| anyhow::anyhow!("de termijn van '{kenmerk}' loopt nog niet"))?;
    klok.verleng(moment, zone, kalender)?;

    let m = Motivering::nieuw(motivering, &super::actor().id, nu)?;
    v.leg_verlenging_vast(grond, moment, m, nu)?;

    let deadline = v
        .termijn
        .as_ref()
        .expect("de klok is hierboven gezet")
        .deadline_volledig(nu, zone, kalender)?;

    bewaar(
        kluis,
        &v,
        Handeling::TermijnVerlengd,
        &format!("verlengd: {}", grond.omschrijving()),
        nu,
    )?;

    gelukt(&format!("verlengd wegens {}", grond.omschrijving()));
    terzijde(&format!("verstrijkt nu op {}", deadline.lokaal));
    toon_ontbrekend(&v);
    Ok(())
}

/// Leidt de vindplaatsen af uit het verwerkingsregister.
fn vindplaatsen(
    kluis: &mut Kluis,
    kenmerk: &str,
    met_concepten: bool,
    nu: DateTime<Utc>,
) -> Result<()> {
    let mut v = zoek(kluis, kenmerk)?;

    let mut toegevoegd = Vec::new();
    let mut overgeslagen = 0usize;
    for k in kluis.lijst("verwerking")? {
        let w: Verwerking = kluis.laad("verwerking", &k.id)?;
        if w.status == Status::Concept && !met_concepten {
            overgeslagen += 1;
            continue;
        }
        // Al aanwezig is geen fout: de opdracht is bedoeld om herhaald te
        // draaien wanneer het register groeit.
        if v.vindplaatsen.iter().any(|p| p.verwerking_id == w.id) {
            continue;
        }
        v.voeg_vindplaats_toe(w.id, &w.kenmerk, &w.naam, nu)?;
        toegevoegd.push(w.kenmerk.clone());
    }

    if toegevoegd.is_empty() && overgeslagen == 0 {
        terzijde("Er zijn geen nieuwe vindplaatsen; de lijst was al bij.");
        toon_ontbrekend(&v);
        return Ok(());
    }

    bewaar(
        kluis,
        &v,
        Handeling::RecordGewijzigd,
        &format!("{} vindplaats(en) afgeleid uit het register", toegevoegd.len()),
        nu,
    )?;

    gelukt(&format!("{} vindplaats(en) toegevoegd", toegevoegd.len()));
    for naam in &toegevoegd {
        println!("  • {naam}");
    }
    if overgeslagen > 0 {
        println!();
        let_op(&format!(
            "{overgeslagen} registerregel(s) met de status concept zijn overgeslagen. Draai \
             opnieuw met --met-concepten als daar ook gegevens van deze betrokkene kunnen staan; \
             een verzoek beperken tot wat toevallig is vastgesteld, is de meest gemaakte fout."
        ));
    }
    terzijde(
        "Deze lijst komt uit het register en is dus zo volledig als het register. Wat er niet in \
         staat, wordt hier niet gevonden.",
    );
    toon_ontbrekend(&v);
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn vindplaats(
    kluis: &mut Kluis,
    kenmerk: &str,
    plaats: &str,
    uitkomst: Vindplaatsuitkomst,
    toelichting: Option<String>,
    toets: Toetsinvoer,
    nu: DateTime<Utc>,
) -> Result<()> {
    let mut v = zoek(kluis, kenmerk)?;

    let opgebouwd = match (&toets.motivering, &toets.tweede_persoon) {
        (Some(m), Some(p)) => Some(Anonimiseringstoets {
            singling_out_uitgesloten: toets.singling_out_uitgesloten,
            koppelbaarheid_uitgesloten: toets.koppelbaarheid_uitgesloten,
            afleidbaarheid_uitgesloten: toets.afleidbaarheid_uitgesloten,
            motivering: Motivering::nieuw(m, &super::actor().id, nu)?,
            bevestigd_door: p.clone(),
        }),
        _ => None,
    };

    let werkelijk = v.stel_vindplaats_vast(plaats, uitkomst, toelichting, opgebouwd, nu)?;
    bewaar(
        kluis,
        &v,
        Handeling::RecordGewijzigd,
        &format!("vindplaats {plaats}: {}", werkelijk.omschrijving()),
        nu,
    )?;

    gelukt(&format!("{plaats}: {}", werkelijk.omschrijving()));
    if uitkomst == Vindplaatsuitkomst::Geanonimiseerd
        && werkelijk == Vindplaatsuitkomst::Gepseudonimiseerd
    {
        let_op(
            "De uitkomst is vastgelegd als 'gepseudonimiseerd — nog persoonsgegevens'. \
             Anonimiseren is een sterke bewering: zij vergt een afgeronde toets op singling out, \
             koppelbaarheid én afleidbaarheid, met een tweede persoon. Klopt die bewering niet, \
             dan is er niets gewist en staan de gegevens er nog.",
        );
    }
    toon_ontbrekend(&v);
    Ok(())
}

fn ontvanger(kluis: &mut Kluis, kenmerk: &str, naam: &str, nu: DateTime<Utc>) -> Result<()> {
    let mut v = zoek(kluis, kenmerk)?;
    v.voeg_kennisgeving_toe(naam, nu)?;
    bewaar(kluis, &v, Handeling::RecordGewijzigd, &format!("ontvanger {naam} toegevoegd"), nu)?;
    gelukt(&format!("ontvanger {naam} toegevoegd"));
    terzijde(
        "art. 19 AVG: elke ontvanger krijgt bericht, tenzij dat onmogelijk of onevenredig is.",
    );
    toon_ontbrekend(&v);
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn kennisgeving(
    kluis: &mut Kluis,
    kenmerk: &str,
    naam: &str,
    verzonden: Option<&str>,
    wijze: Option<String>,
    onmogelijk: bool,
    motivering: Option<&str>,
    nu: DateTime<Utc>,
) -> Result<()> {
    let mut v = zoek(kluis, kenmerk)?;
    let moment = match verzonden {
        Some(t) => Some(lees_tijdstip(t)?),
        None => None,
    };
    let m = match motivering {
        Some(t) => Some(Motivering::nieuw(t, &super::actor().id, nu)?),
        None => None,
    };
    v.leg_kennisgeving_vast(naam, moment, wijze, onmogelijk, m, nu)?;

    bewaar(kluis, &v, Handeling::RecordGewijzigd, &format!("kennisgeving aan {naam}"), nu)?;
    gelukt(&format!("kennisgeving aan {naam} vastgelegd"));
    toon_ontbrekend(&v);
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn bericht_lid4(
    kluis: &mut Kluis,
    kenmerk: &str,
    verzonden_op: &str,
    redenen: &str,
    klachtrecht: bool,
    rechtsmiddel: bool,
    nu: DateTime<Utc>,
) -> Result<()> {
    let mut v = zoek(kluis, kenmerk)?;
    let moment = lees_tijdstip(verzonden_op)?;
    let m = Motivering::nieuw(redenen, &super::actor().id, nu)?;
    v.leg_bericht_lid4_vast(moment, klachtrecht, rechtsmiddel, m, nu)?;

    bewaar(kluis, &v, Handeling::MeldingVerzonden, "bericht art. 12 lid 4 verzonden", nu)?;
    gelukt(&format!("bericht vastgelegd, verzonden op {}", crate::uitvoer::datum(moment)));

    if !klachtrecht || !rechtsmiddel {
        blokkade(
            "Het bericht noemt niet zowel het klachtrecht bij de toezichthouder als de \
             mogelijkheid van beroep bij de rechter. Artikel 12 lid 4 vraagt allebei; het \
             verzoek is zo niet af te sluiten.",
        );
    }
    toon_ontbrekend(&v);
    Ok(())
}

fn afhandelen(
    kluis: &mut Kluis,
    kenmerk: &str,
    uitkomst: Verzoekuitkomst,
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

    match v.handel_af(uitkomst, moment, nu) {
        Ok(()) => {
            bewaar(kluis, &v, Handeling::RecordVastgesteld, "verzoek afgehandeld", nu)?;
            gelukt(&format!("verzoek {kenmerk} afgehandeld: {}", uitkomst.omschrijving()));
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
                    "afhandelen geweigerd: er staat nog iets open dat de betrokkene raakt",
                ),
                Some(fout.to_string()),
            )?;
            kop("Afhandelen is niet gelukt");
            toon_ontbrekend(&v);
            anyhow::bail!("{fout}")
        }
    }
}

fn lijst(kluis: &Kluis, alleen_onvolledig: bool, nu: DateTime<Utc>) -> Result<()> {
    let koppen = kluis.lijst(SOORT)?;
    if koppen.is_empty() {
        kop("Verzoeken van betrokkenen");
        terzijde("Er staan nog geen verzoeken in de kluis.");
        terzijde(
            "Registreer er een met 'dpofg verzoek nieuw <kenmerk> <omschrijving> --soort inzage'.",
        );
        return Ok(());
    }

    let pakket = dpofg_content::startpakket(nu.date_naive());
    let kalender = pakket.kalender("NL").ok();
    let zone = dpofg_terms::tijdzone(dpofg_terms::TIJDZONE_NL).ok();

    kop("Verzoeken van betrokkenen");
    let mut t = tabel(&["kenmerk", "soort", "ontvangen", "verstrijkt", "resterend", "volledig"]);
    for k in &koppen {
        let v: Betrokkenenverzoek = kluis.laad(SOORT, &k.id)?;
        let r = v.volledigheid();
        if alleen_onvolledig && r.is_volledig() {
            continue;
        }
        let (verstrijkt, resterend) = match (&v.termijn, kalender, zone) {
            (Some(klok), Some(kal), Some(z)) => match klok.deadline_volledig(nu, z, kal) {
                Ok(d) => {
                    let over = d.moment - nu;
                    let tekst = if v.afgehandeld_op.is_some() {
                        "afgehandeld".to_string()
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
            v.soort.omschrijving().to_string(),
            crate::uitvoer::datum(v.ontvangen_op).to_string(),
            verstrijkt,
            resterend,
            format!("{} van {}", r.compleet, r.verplicht),
        ]);
    }
    println!("{t}");
    Ok(())
}

fn toon(kluis: &Kluis, kenmerk: &str, nu: DateTime<Utc>) -> Result<()> {
    let v = zoek(kluis, kenmerk)?;

    kop(&format!("Verzoek {}", v.kenmerk));
    let mut t = tabel(&["", ""]);
    t.add_row(vec!["omschrijving", &v.omschrijving]);
    t.add_row(vec!["soort", v.soort.omschrijving()]);
    t.add_row(vec!["grondslag", v.soort.grondslag()]);
    t.add_row(vec!["kanaal", v.kanaal.omschrijving()]);
    t.add_row(vec!["ontvangen op", &crate::uitvoer::tijdstip(v.ontvangen_op).to_string()]);
    t.add_row(vec!["status", v.status.omschrijving()]);
    t.add_row(vec!["behandelaar", &v.behandelaar]);
    if let Some(m) = v.identiteit_geverifieerd_op {
        t.add_row(vec!["identiteit vastgesteld", &crate::uitvoer::datum(m).to_string()]);
    }
    if let Some(l) = &v.lezing {
        t.add_row(vec!["lezing", l.lezing.omschrijving()]);
    }
    if let Some(u) = v.uitkomst {
        t.add_row(vec!["uitkomst", u.omschrijving()]);
    }
    println!("{t}");

    if let Some(klok) = &v.termijn {
        let pakket = dpofg_content::startpakket(nu.date_naive());
        let kalender = pakket.kalender("NL").context("de feestdagenkalender ontbreekt")?;
        let zone = dpofg_terms::tijdzone(dpofg_terms::TIJDZONE_NL)?;

        kop("Termijn");
        let mut t = tabel(&["", ""]);
        t.add_row(vec!["anker", &crate::uitvoer::datum(klok.anker).to_string()]);
        match klok.deadline_volledig(nu, zone, kalender) {
            Ok(d) => {
                t.add_row(vec!["verstrijkt", &d.lokaal]);
                if v.afgehandeld_op.is_none() {
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
        if let Some(verlenging) = &v.verlenging {
            t.add_row(vec!["verlengd wegens", verlenging.grond.omschrijving()]);
            t.add_row(vec![
                "medegedeeld op",
                &crate::uitvoer::datum(verlenging.medegedeeld_op).to_string(),
            ]);
        }
        println!("{t}");
        if let Some(p) = &v.termijn_pakket {
            terzijde(&format!("gerekend op kennispakket {p}"));
        }
    }

    if !v.vindplaatsen.is_empty() {
        kop("Vindplaatsen");
        let mut t = tabel(&["registerregel", "omschrijving", "uitkomst"]);
        for p in &v.vindplaatsen {
            t.add_row(vec![
                p.kenmerk.clone(),
                p.omschrijving.clone(),
                p.uitkomst.map(|u| u.omschrijving().to_string()).unwrap_or_else(|| "—".into()),
            ]);
        }
        println!("{t}");
    }

    if !v.kennisgevingen.is_empty() {
        kop("Ontvangers (art. 19 AVG)");
        let mut t = tabel(&["ontvanger", "bericht", "reden bij onmogelijkheid"]);
        for k in &v.kennisgevingen {
            t.add_row(vec![
                k.ontvanger.clone(),
                k.verzonden_op.map(|m| crate::uitvoer::datum(m).to_string()).unwrap_or_else(|| {
                    if k.onmogelijk_of_onevenredig {
                        "—".into()
                    } else {
                        "nog niet".into()
                    }
                }),
                k.motivering.as_ref().map(|m| m.tekst.clone()).unwrap_or_default(),
            ]);
        }
        println!("{t}");
    }

    if let Some(b) = &v.bericht_lid4 {
        kop("Bericht art. 12 lid 4");
        let mut t = tabel(&["", ""]);
        t.add_row(vec!["verzonden op", &crate::uitvoer::datum(b.verzonden_op).to_string()]);
        t.add_row(vec!["noemt klachtrecht", if b.noemt_klachtrecht { "ja" } else { "nee" }]);
        t.add_row(vec![
            "noemt beroepsmogelijkheid",
            if b.noemt_rechtsmiddel { "ja" } else { "nee" },
        ]);
        println!("{t}");
    }

    toon_ontbrekend(&v);
    Ok(())
}

fn toon_ontbrekend(v: &Betrokkenenverzoek) {
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
        let veld = o.veld.trim_start_matches("verzoek.");
        if o.blokkeert_vaststelling {
            blokkade(&format!("{veld} — {}", o.omschrijving));
        } else {
            let_op(&format!("{veld} — {}", o.omschrijving));
        }
        terzijde(&o.grondslag);
    }
    println!();
    terzijde("■ houdt afhandelen tegen · ▸ blijft zichtbaar maar blokkeert niet");
}
