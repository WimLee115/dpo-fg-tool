//! Datalekken en beveiligingsincidenten.

use anyhow::Result;
use chrono::{DateTime, Utc};
use clap::Subcommand;
use dpofg_audit::Handeling;
use dpofg_domain::{
    incident::Herkomstkanaal,
    klokken::{verplichtingen_uit_incident, Zorgplichtcontext},
    Aantasting, Incident, Motivering, Risiconiveau, Volledig,
};
use dpofg_store::Kluis;
use std::path::PathBuf;

use crate::uitvoer::{blokkade, duur, gelukt, kop, let_op, tabel, terzijde, voortgang};

const SOORT: &str = "incident";
const COMPARTIMENT: &str = "vertrouwelijk";

#[derive(Subcommand, Debug)]
pub enum Incidentopdracht {
    /// Registreer een incident. Doe dit als eerste, vul de rest daarna aan.
    Nieuw {
        /// Kenmerk van het incident.
        kenmerk: String,
        /// Korte omschrijving.
        omschrijving: String,
        /// Hoe het aan het licht kwam.
        #[arg(long, value_enum, default_value = "intern")]
        kanaal: Kanaalkeuze,
        /// Wanneer het eerste signaal binnenkwam, bijvoorbeeld
        /// 2026-08-18T09:00:00Z. Standaard: nu.
        ///
        /// Vaak ligt dit moment vóór de registratie: een melding die vrijdag
        /// binnenkwam en maandag wordt vastgelegd. Dat verschil is zelf een
        /// bevinding (LEK-03) en hoort dus vastgelegd te worden, niet gladgestreken.
        #[arg(long)]
        signaal: Option<String>,
    },
    /// Toon alle incidenten met hun klokken.
    Lijst,
    /// Toon één incident met zijn klokken en wat er nog ontbreekt.
    Toon {
        /// Het kenmerk van het incident.
        kenmerk: String,
    },
    /// Leg het moment van kennisname vast. Hierop start de meldklok.
    Kennisname {
        /// Het kenmerk van het incident.
        kenmerk: String,
        /// Tijdstip in de vorm 2026-08-18T09:20:00Z.
        tijdstip: String,
        /// Onderbouwing van de verificatieperiode tussen signaal en kennisname.
        #[arg(long)]
        onderbouwing: Option<String>,
    },
    /// Leg vast welke aspecten zijn aangetast.
    Aantasting {
        /// Het kenmerk van het incident.
        kenmerk: String,
        #[arg(long)]
        vertrouwelijkheid: bool,
        #[arg(long)]
        integriteit: bool,
        #[arg(long)]
        beschikbaarheid: bool,
    },
    /// Leg de feiten van het incident vast.
    ///
    /// Deze feiten bepalen welke waarborgen straks gelden bij het meldbesluit,
    /// en moeten er dus vóór dat besluit zijn.
    Feiten {
        /// Het kenmerk van het incident.
        kenmerk: String,
        /// Welke soorten gegevens het betreft, gescheiden door puntkomma's.
        #[arg(long)]
        gegevens: Option<String>,
        /// Het aantal betrokkenen.
        #[arg(long)]
        betrokkenen: Option<u64>,
        /// Merk het aantal aan als schatting.
        #[arg(long)]
        geschat: bool,
        /// Of gegevensuitvoer naar buiten is uit te sluiten.
        #[arg(long)]
        exfiltratie_uitgesloten: Option<bool>,
        /// Er zijn bijzondere persoonsgegevens in het spel.
        #[arg(long)]
        bijzondere_gegevens: bool,
        /// Er is een burgerservicenummer in het spel.
        #[arg(long)]
        bsn: bool,
        /// Er zijn financiële gegevens in het spel.
        #[arg(long)]
        financieel: bool,
    },
    /// Leg de risicoweging vast.
    Weging {
        /// Het kenmerk van het incident.
        kenmerk: String,
        /// De uitkomst.
        #[arg(long, value_enum)]
        uitkomst: Risicokeuze,
        /// De onderbouwing. Verplicht: de uitkomst zegt niets zonder de weging.
        #[arg(long)]
        motivering: String,
    },
    /// Besluit om niet te melden. Dit is de zwaarst beveiligde handeling.
    NietMelden {
        /// Het kenmerk van het incident.
        kenmerk: String,
        /// De onderbouwing.
        #[arg(long)]
        motivering: String,
        /// De tweede persoon die het besluit bevestigt.
        #[arg(long)]
        tweede_persoon: Option<String>,
        /// Afkoelperiode in uren, als er geen tweede persoon beschikbaar is.
        #[arg(long, default_value = "0")]
        afkoeluren: i64,
    },
}

#[derive(clap::ValueEnum, Clone, Copy, Debug)]
pub enum Kanaalkeuze {
    Intern,
    Verwerker,
    Betrokkene,
    Derde,
    Instantie,
}

impl From<Kanaalkeuze> for Herkomstkanaal {
    fn from(k: Kanaalkeuze) -> Self {
        match k {
            Kanaalkeuze::Intern => Herkomstkanaal::InternVastgesteld,
            Kanaalkeuze::Verwerker => Herkomstkanaal::MeldingVanVerwerker,
            Kanaalkeuze::Betrokkene => Herkomstkanaal::MeldingVanBetrokkene,
            Kanaalkeuze::Derde => Herkomstkanaal::MeldingVanDerde,
            Kanaalkeuze::Instantie => Herkomstkanaal::ExterneInstantie,
        }
    }
}

#[derive(clap::ValueEnum, Clone, Copy, Debug)]
pub enum Risicokeuze {
    GeenRisico,
    Risico,
    HoogRisico,
}

impl From<Risicokeuze> for Risiconiveau {
    fn from(k: Risicokeuze) -> Self {
        match k {
            Risicokeuze::GeenRisico => Risiconiveau::GeenRisico,
            Risicokeuze::Risico => Risiconiveau::Risico,
            Risicokeuze::HoogRisico => Risiconiveau::HoogRisico,
        }
    }
}

pub fn draai(o: Incidentopdracht, kluispad: Option<PathBuf>, nu: DateTime<Utc>) -> Result<()> {
    let pad = super::kluispad(kluispad)?;
    let mut kluis = super::open_kluis(&pad, nu)?;
    match o {
        Incidentopdracht::Nieuw { kenmerk, omschrijving, kanaal, signaal } => {
            let moment = match &signaal {
                None => nu,
                Some(s) => lees_tijdstip(s)?,
            };
            nieuw(&mut kluis, &kenmerk, &omschrijving, kanaal.into(), moment, nu)
        }
        Incidentopdracht::Lijst => lijst(&kluis, nu),
        Incidentopdracht::Toon { kenmerk } => toon(&kluis, &kenmerk, nu),
        Incidentopdracht::Kennisname { kenmerk, tijdstip, onderbouwing } => {
            kennisname(&mut kluis, &kenmerk, &tijdstip, onderbouwing, nu)
        }
        Incidentopdracht::Aantasting {
            kenmerk,
            vertrouwelijkheid,
            integriteit,
            beschikbaarheid,
        } => aantasting(&mut kluis, &kenmerk, vertrouwelijkheid, integriteit, beschikbaarheid, nu),
        Incidentopdracht::Feiten {
            kenmerk,
            gegevens,
            betrokkenen,
            geschat,
            exfiltratie_uitgesloten,
            bijzondere_gegevens,
            bsn,
            financieel,
        } => feiten(
            &mut kluis,
            &kenmerk,
            gegevens,
            betrokkenen,
            geschat,
            exfiltratie_uitgesloten,
            bijzondere_gegevens,
            bsn,
            financieel,
            nu,
        ),
        Incidentopdracht::Weging { kenmerk, uitkomst, motivering } => {
            weging(&mut kluis, &kenmerk, uitkomst.into(), &motivering, nu)
        }
        Incidentopdracht::NietMelden { kenmerk, motivering, tweede_persoon, afkoeluren } => {
            niet_melden(&mut kluis, &kenmerk, &motivering, tweede_persoon, afkoeluren, nu)
        }
    }
}

fn zoek(kluis: &Kluis, kenmerk: &str) -> Result<Incident> {
    let kop = kluis
        .lijst(SOORT)?
        .into_iter()
        .find(|r| r.kenmerk.as_deref() == Some(kenmerk))
        .ok_or_else(|| anyhow::anyhow!("geen incident met kenmerk '{kenmerk}'"))?;
    Ok(kluis.laad(SOORT, &kop.id)?)
}

fn bewaar(
    kluis: &mut Kluis,
    i: &Incident,
    handeling: Handeling,
    omschrijving: &str,
    nu: DateTime<Utc>,
) -> Result<()> {
    let actor = super::actor();
    kluis.bewaar(
        SOORT,
        &i.id.to_string(),
        COMPARTIMENT,
        i.status.omschrijving(),
        Some(&i.kenmerk),
        i,
        &actor,
        handeling,
        omschrijving,
        nu,
    )?;
    Ok(())
}

/// Leest een tijdstip uit de opdrachtregel.
fn lees_tijdstip(tekst: &str) -> Result<DateTime<Utc>> {
    tekst
        .parse::<DateTime<chrono::FixedOffset>>()
        .map(|t| t.with_timezone(&Utc))
        .map_err(|e| {
            anyhow::anyhow!(
                "kon '{tekst}' niet lezen als tijdstip ({e}). Gebruik de vorm \
                 2026-08-18T09:20:00Z of 2026-08-18T11:20:00+02:00"
            )
        })
}

fn nieuw(
    kluis: &mut Kluis,
    kenmerk: &str,
    omschrijving: &str,
    kanaal: Herkomstkanaal,
    signaal_op: DateTime<Utc>,
    nu: DateTime<Utc>,
) -> Result<()> {
    if kluis.lijst(SOORT)?.iter().any(|r| r.kenmerk.as_deref() == Some(kenmerk)) {
        anyhow::bail!("er bestaat al een incident met kenmerk '{kenmerk}'");
    }
    if signaal_op > nu {
        anyhow::bail!(
            "het signaalmoment ligt in de toekomst. Controleer het opgegeven tijdstip en \
             de tijdzone"
        );
    }
    let actor = super::actor();
    let i = Incident::nieuw(kenmerk, omschrijving, signaal_op, nu, kanaal, &actor.id, &actor.id);
    bewaar(kluis, &i, Handeling::RecordAangemaakt, "incident geregistreerd", nu)?;

    gelukt(&format!("incident '{kenmerk}' geregistreerd"));

    let vertraging = nu - signaal_op;
    if vertraging.num_hours() >= 4 {
        println!();
        let_op(&format!(
            "Er zat {} tussen het eerste signaal en deze registratie. Dat verschil blijft \
             vastgelegd en verschijnt in de controleronde; gladstrijken helpt niemand.",
            duur(vertraging)
        ));
    }

    println!();
    let_op(
        "Registreren is de eerste stap en mag ruw zijn. Het moment van registratie staat nu \
         vast; alles daarna is aanvullen.",
    );
    terzijde(
        "Leg als volgende het moment van kennisname vast: 'dpofg incident kennisname \
         <kenmerk> <tijdstip>'. Daarop start de meldklok.",
    );
    Ok(())
}

fn lijst(kluis: &Kluis, nu: DateTime<Utc>) -> Result<()> {
    let koppen = kluis.lijst(SOORT)?;
    if koppen.is_empty() {
        kop("Incidenten");
        terzijde("Er staan nog geen incidenten in de kluis.");
        return Ok(());
    }

    kop("Incidenten");
    let mut t = tabel(&["kenmerk", "omschrijving", "kennisname", "weging", "besluit", "volledig"]);
    for k in &koppen {
        let i: Incident = kluis.laad(SOORT, &k.id)?;
        let r = i.volledigheid();
        t.add_row(vec![
            i.kenmerk.clone(),
            i.omschrijving.chars().take(40).collect::<String>(),
            i.kennisname_op
                .map(|t| t.format("%d-%m %H:%M").to_string())
                .unwrap_or_else(|| "—".into()),
            i.risiconiveau
                .map(|r| match r {
                    Risiconiveau::GeenRisico => "geen risico",
                    Risiconiveau::Risico => "risico",
                    Risiconiveau::HoogRisico => "hoog risico",
                })
                .unwrap_or("—")
                .to_string(),
            besluittekst(&i, nu),
            format!("{}/{}", r.compleet, r.verplicht),
        ]);
    }
    println!("{t}");
    Ok(())
}

fn besluittekst(i: &Incident, nu: DateTime<Utc>) -> String {
    use dpofg_domain::Meldbesluit::*;
    match &i.meldbesluit {
        NogTeNemen => "nog te nemen".into(),
        Melden { .. } => "melden".into(),
        NietMelden { afkoelperiode_tot, .. } => {
            if i.meldbesluit.is_definitief(nu) {
                "niet melden".into()
            } else {
                format!(
                    "niet melden (definitief {})",
                    afkoelperiode_tot
                        .map(|t| t.format("%d-%m %H:%M").to_string())
                        .unwrap_or_default()
                )
            }
        }
    }
}

fn toon(kluis: &Kluis, kenmerk: &str, nu: DateTime<Utc>) -> Result<()> {
    let i = zoek(kluis, kenmerk)?;

    kop(&format!("{} — {}", i.kenmerk, i.omschrijving));
    let mut t = tabel(&["", ""]);
    t.add_row(vec!["eerste signaal", &i.signaal_op.format("%d-%m-%Y %H:%M UTC").to_string()]);
    t.add_row(vec!["geregistreerd", &i.geregistreerd_op.format("%d-%m-%Y %H:%M UTC").to_string()]);
    match i.kennisname_op {
        Some(k) => t.add_row(vec!["kennisname", &k.format("%d-%m-%Y %H:%M UTC").to_string()]),
        None => t.add_row(vec!["kennisname", "nog niet vastgelegd"]),
    };
    t.add_row(vec![
        "aangetast",
        &aantastingstekst(&i.aantasting),
    ]);
    if let Some(n) = i.aantal_betrokkenen {
        t.add_row(vec!["betrokkenen", &n.to_string()]);
    }
    println!("{t}");

    // De verificatieperiode en de registratievertraging: de twee maten waarop
    // de 72 uur in de praktijk verdampt.
    if let Some(v) = i.verificatieduur() {
        terzijde(&format!("verificatieperiode tussen signaal en kennisname: {}", duur(v)));
    }
    if let Some(v) = i.registratievertraging() {
        if v.num_hours() > 4 {
            let_op(&format!(
                "Er zat {} tussen kennisname en registratie. Dat is de meest voorkomende \
                 manier waarop de meldtermijn verdampt.",
                duur(v)
            ));
        }
    }

    // De klokken.
    let verplichtingen =
        verplichtingen_uit_incident(&i, Zorgplichtcontext::niet_van_toepassing());
    kop("Klokken die uit dit incident volgen");
    let mut k = tabel(&["verplichting", "anker", "vanaf", "reden"]);
    for v in &verplichtingen {
        k.add_row(vec![
            v.code.code().to_string(),
            v.ankertype.omschrijving().to_string(),
            v.anker
                .map(|a| a.format("%d-%m-%Y %H:%M").to_string())
                .unwrap_or_else(|| "wacht op anker".into()),
            v.reden.clone(),
        ]);
    }
    println!("{k}");
    terzijde(
        "De ankers vallen niet samen. Wie één ankerveld gebruikt voor alle klokken, rekent er \
         minstens twee fout.",
    );

    let r = i.volledigheid();
    kop("Volledigheid");
    println!("  {}", voortgang(r.compleet, r.verplicht));
    if !r.is_volledig() {
        println!();
        for o in &r.ontbreekt {
            blokkade(&format!("{} — {}", o.veld.trim_start_matches("incident."), o.omschrijving));
            terzijde(&o.grondslag);
        }
    }
    let _ = nu;
    Ok(())
}

fn aantastingstekst(a: &Aantasting) -> String {
    let mut delen = Vec::new();
    if a.vertrouwelijkheid {
        delen.push("vertrouwelijkheid");
    }
    if a.integriteit {
        delen.push("integriteit");
    }
    if a.beschikbaarheid {
        delen.push("beschikbaarheid");
    }
    if delen.is_empty() {
        "nog niet beoordeeld".into()
    } else {
        delen.join(", ")
    }
}

fn kennisname(
    kluis: &mut Kluis,
    kenmerk: &str,
    tijdstip: &str,
    onderbouwing: Option<String>,
    nu: DateTime<Utc>,
) -> Result<()> {
    let mut i = zoek(kluis, kenmerk)?;
    let moment = lees_tijdstip(tijdstip)?;

    let motivering = match onderbouwing {
        Some(t) => Some(Motivering::nieuw(t, &super::actor().id, nu)?),
        None => None,
    };
    i.stel_kennisname_vast(moment, motivering)?;
    bewaar(kluis, &i, Handeling::TermijnGestart, "moment van kennisname vastgelegd", nu)?;

    gelukt(&format!(
        "kennisname vastgelegd op {}",
        moment.format("%d-%m-%Y %H:%M UTC")
    ));
    let verplichtingen =
        verplichtingen_uit_incident(&i, Zorgplichtcontext::niet_van_toepassing());
    println!();
    println!("Hierdoor zijn deze klokken gaan lopen:");
    for v in &verplichtingen {
        if !v.wacht_op_anker {
            println!("  • {} — vanaf {}", v.code.code(), v.ankertype.omschrijving());
        }
    }
    Ok(())
}

fn aantasting(
    kluis: &mut Kluis,
    kenmerk: &str,
    vertrouwelijkheid: bool,
    integriteit: bool,
    beschikbaarheid: bool,
    nu: DateTime<Utc>,
) -> Result<()> {
    let mut i = zoek(kluis, kenmerk)?;
    i.aantasting = Aantasting { vertrouwelijkheid, integriteit, beschikbaarheid };
    if !i.aantasting.is_aangetast() {
        let_op(
            "Geen van de drie aspecten is aangevinkt. Let op het beschikbaarheidsaspect: \
             ook verlies van toegang tot gegevens is een inbreuk (art. 4 onder 12 AVG).",
        );
    }
    bewaar(kluis, &i, Handeling::RecordGewijzigd, "aantasting beoordeeld", nu)?;
    gelukt(&format!("aantasting vastgelegd: {}", aantastingstekst(&i.aantasting)));
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn feiten(
    kluis: &mut Kluis,
    kenmerk: &str,
    gegevens: Option<String>,
    betrokkenen: Option<u64>,
    geschat: bool,
    exfiltratie_uitgesloten: Option<bool>,
    bijzondere_gegevens: bool,
    bsn: bool,
    financieel: bool,
    nu: DateTime<Utc>,
) -> Result<()> {
    let mut i = zoek(kluis, kenmerk)?;

    if let Some(g) = gegevens {
        i.categorieen_gegevens =
            g.split(';').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
    }
    if let Some(n) = betrokkenen {
        i.aantal_betrokkenen = Some(n);
        i.aantal_betrokkenen_geschat = geschat;
    }
    if let Some(e) = exfiltratie_uitgesloten {
        i.exfiltratie_uitgesloten = Some(e);
    }
    if bijzondere_gegevens {
        i.bijzondere_gegevens = true;
    }
    if bsn {
        i.burgerservicenummer = true;
    }
    if financieel {
        i.financiele_gegevens = true;
    }

    bewaar(kluis, &i, Handeling::RecordGewijzigd, "feiten vastgelegd", nu)?;
    gelukt("feiten vastgelegd");

    // Meteen laten zien wat deze feiten straks betekenen voor het besluit.
    // Dat is het verschil tussen een tool die registreert en een tool die helpt.
    if i.tweede_persoon_verplicht() {
        println!();
        let_op(
            "Door deze gegevens vereist een besluit om niet te melden straks bevestiging door \
             een tweede persoon. Een afkoelperiode volstaat dan niet.",
        );
    }
    if i.omvang_vereist_tegenspraak() {
        println!();
        let_op(&format!(
            "Bij {} betrokkenen vraagt de uitkomst 'geen risico' om tegenspraak van iemand anders.",
            i.aantal_betrokkenen.unwrap_or(0)
        ));
    }
    if i.aantal_betrokkenen_geschat {
        terzijde(
            "Het aantal is als schatting aangemerkt; die aanduiding gaat mee in de melding.",
        );
    }

    let r = i.volledigheid();
    println!();
    println!("  {}", voortgang(r.compleet, r.verplicht));
    Ok(())
}

fn weging(
    kluis: &mut Kluis,
    kenmerk: &str,
    uitkomst: Risiconiveau,
    motivering: &str,
    nu: DateTime<Utc>,
) -> Result<()> {
    let mut i = zoek(kluis, kenmerk)?;
    i.risiconiveau = Some(uitkomst);
    i.risicoweging = Some(Motivering::nieuw(motivering, &super::actor().id, nu)?);
    bewaar(kluis, &i, Handeling::BesluitGenomen, "risicoweging vastgelegd", nu)?;

    gelukt(&format!("weging vastgelegd: {}", uitkomst.omschrijving()));

    if uitkomst.leidt_tot_melding() {
        println!();
        let_op("Deze uitkomst leidt tot een melding aan de toezichthouder (art. 33 lid 1 AVG).");
    }
    if uitkomst.leidt_tot_mededeling() {
        let_op(
            "Daarnaast ontstaat de plicht de betrokkenen te informeren (art. 34 lid 1 AVG). \
             Die klok hangt aan dit moment van vaststelling, niet aan de kennisname.",
        );
    }
    if uitkomst == Risiconiveau::GeenRisico {
        if i.omvang_vereist_tegenspraak() {
            println!();
            let_op(&format!(
                "'Geen risico' bij {} betrokkenen. Laat iemand anders hier naar kijken voordat u \
                 het besluit neemt.",
                i.aantal_betrokkenen.unwrap_or(0)
            ));
        }
        if i.tweede_persoon_verplicht() {
            let_op(
                "Er zijn gevoelige gegevens in het spel. Een besluit om niet te melden vereist \
                 dan bevestiging door een tweede persoon; een afkoelperiode volstaat niet.",
            );
        }
    }
    Ok(())
}

fn niet_melden(
    kluis: &mut Kluis,
    kenmerk: &str,
    motivering: &str,
    tweede_persoon: Option<String>,
    afkoeluren: i64,
    nu: DateTime<Utc>,
) -> Result<()> {
    let mut i = zoek(kluis, kenmerk)?;
    let m = Motivering::nieuw(motivering, &super::actor().id, nu)?;

    kop("Besluit om niet te melden");
    terzijde(
        "Dit is de enige beslissing in dit product waarvan de gevolgen niet met een herstelknop \
         terug te draaien zijn. De 72 uur komt niet terug.",
    );

    match i.besluit_niet_melden(m, tweede_persoon.clone(), nu, chrono::Duration::hours(afkoeluren))
    {
        Ok(()) => {
            bewaar(kluis, &i, Handeling::BesluitGenomen, "besloten niet te melden", nu)?;
            println!();
            gelukt("besluit vastgelegd");
            if let Some(p) = tweede_persoon {
                terzijde(&format!("bevestigd door {p}"));
            }
            if afkoeluren > 0 {
                terzijde(&format!(
                    "het besluit wordt definitief over {afkoeluren} uur; tot dan blijft de \
                     meldklok staan zodat een omkering nog binnen de termijn past"
                ));
            }
            println!();
            let_op(
                "De vastlegging in het interne register blijft verplicht (art. 33 lid 5 AVG). \
                 Juist bij niet melden is dat de enige verantwoording die overblijft.",
            );
            Ok(())
        }
        Err(fout) => {
            let actor = super::actor();
            kluis.log(
                dpofg_audit::Gebeurtenis::nieuw(
                    Handeling::ControleGeblokkeerd,
                    actor,
                    nu,
                    SOORT,
                    i.id.to_string(),
                    COMPARTIMENT,
                    "besluit om niet te melden geweigerd",
                ),
                Some(fout.to_string()),
            )?;
            println!();
            blokkade(&fout.to_string());
            anyhow::bail!("het besluit is niet vastgelegd")
        }
    }
}
