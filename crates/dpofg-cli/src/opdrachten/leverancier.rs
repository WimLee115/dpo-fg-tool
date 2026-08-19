//! Leveranciers en verwerkersovereenkomsten.

use anyhow::Result;
use chrono::{DateTime, Utc};
use clap::Subcommand;
use dpofg_audit::Handeling;
use dpofg_domain::{
    leverancier::{Contracteis, Kritikaliteit, Leverancier},
    Motivering, Volledig,
};
use dpofg_store::Kluis;
use std::path::PathBuf;

use crate::uitvoer::{blokkade, gelukt, kop, let_op, tabel, terzijde, voortgang};

const SOORT: &str = "leverancier";
const COMPARTIMENT: &str = "algemeen";

#[derive(Subcommand, Debug)]
pub enum Leveranciersopdracht {
    /// Toon alle leveranciers met de stand van hun overeenkomst.
    Lijst,
    /// Registreer een leverancier.
    Nieuw {
        /// Kenmerk van de leverancier.
        kenmerk: String,
        /// De naam.
        naam: String,
        /// Het land van vestiging.
        #[arg(long, default_value = "Nederland")]
        land: String,
        /// Het KvK-nummer.
        #[arg(long)]
        kvk: Option<String>,
        /// Hoe belangrijk deze leverancier is.
        #[arg(long, value_enum)]
        kritikaliteit: Option<Kritikaliteitkeuze>,
    },
    /// Toon één leverancier.
    Toon {
        /// Kenmerk van de leverancier.
        kenmerk: String,
    },
    /// Toon de acht onderdelen van artikel 28 lid 3.
    Eisen,
    /// Leg de verwerkersovereenkomst vast.
    Overeenkomst {
        /// Kenmerk van de leverancier.
        kenmerk: String,
        /// Het kenmerk van de overeenkomst.
        #[arg(long)]
        contract: String,
        /// Wanneer zij is getekend.
        #[arg(long)]
        ondertekend: String,
        /// Wanneer de verwerking feitelijk begon.
        #[arg(long)]
        verwerking_begon: Option<String>,
        /// Binnen hoeveel uur de verwerker een inbreuk moet melden.
        #[arg(long)]
        meldtermijn_uren: Option<u32>,
    },
    /// Wijs aan wáár een onderdeel van artikel 28 lid 3 in het contract staat.
    Vindplaats {
        /// Kenmerk van de leverancier.
        kenmerk: String,
        /// Welk onderdeel.
        #[arg(long, value_enum)]
        eis: Eiskeuze,
        /// Artikel, bijlage of paragraaf.
        #[arg(long)]
        aanduiding: String,
        /// Toelichting.
        #[arg(long)]
        toelichting: Option<String>,
    },
    /// Voeg een subverwerker toe.
    Subverwerker {
        /// Kenmerk van de leverancier.
        kenmerk: String,
        /// De naam van de subverwerker.
        #[arg(long)]
        naam: String,
        /// Het land.
        #[arg(long)]
        land: String,
        /// Welke dienst hij levert.
        #[arg(long)]
        dienst: String,
    },
    /// Leg vast dat de subverwerkerslijst is nagelopen.
    Subverwerkerscontrole {
        /// Kenmerk van de leverancier.
        kenmerk: String,
        /// Wanneer. Standaard: nu.
        #[arg(long)]
        op: Option<String>,
    },
    /// Leg een besluit vast om deze leverancier te weren.
    Weren {
        /// Kenmerk van de leverancier.
        kenmerk: String,
        /// De reden.
        #[arg(long)]
        motivering: String,
    },
    /// Stel de leverancier vast.
    Vaststellen {
        /// Kenmerk van de leverancier.
        kenmerk: String,
    },
}

#[derive(clap::ValueEnum, Clone, Copy, Debug)]
pub enum Kritikaliteitkeuze {
    Laag,
    Gemiddeld,
    Hoog,
}

impl From<Kritikaliteitkeuze> for Kritikaliteit {
    fn from(k: Kritikaliteitkeuze) -> Self {
        match k {
            Kritikaliteitkeuze::Laag => Kritikaliteit::Laag,
            Kritikaliteitkeuze::Gemiddeld => Kritikaliteit::Gemiddeld,
            Kritikaliteitkeuze::Hoog => Kritikaliteit::Hoog,
        }
    }
}

#[derive(clap::ValueEnum, Clone, Copy, Debug)]
pub enum Eiskeuze {
    Instructies,
    Geheimhouding,
    Beveiliging,
    Subverwerkers,
    BijstandVerzoeken,
    BijstandVerplichtingen,
    WissenOfTeruggeven,
    AuditsEnInformatie,
}

impl From<Eiskeuze> for Contracteis {
    fn from(k: Eiskeuze) -> Self {
        match k {
            Eiskeuze::Instructies => Contracteis::Instructies,
            Eiskeuze::Geheimhouding => Contracteis::Geheimhouding,
            Eiskeuze::Beveiliging => Contracteis::Beveiliging,
            Eiskeuze::Subverwerkers => Contracteis::Subverwerkers,
            Eiskeuze::BijstandVerzoeken => Contracteis::BijstandVerzoeken,
            Eiskeuze::BijstandVerplichtingen => Contracteis::BijstandVerplichtingen,
            Eiskeuze::WissenOfTeruggeven => Contracteis::WissenOfTeruggeven,
            Eiskeuze::AuditsEnInformatie => Contracteis::AuditsEnInformatie,
        }
    }
}

pub fn draai(o: Leveranciersopdracht, kluispad: Option<PathBuf>, nu: DateTime<Utc>) -> Result<()> {
    if matches!(o, Leveranciersopdracht::Eisen) {
        return toon_eisen();
    }
    let pad = super::kluispad(kluispad)?;
    let mut kluis = super::open_kluis(&pad, nu)?;
    match o {
        Leveranciersopdracht::Eisen => unreachable!("hierboven afgehandeld"),
        Leveranciersopdracht::Lijst => lijst(&kluis, nu),
        Leveranciersopdracht::Nieuw { kenmerk, naam, land, kvk, kritikaliteit } => {
            nieuw(&mut kluis, &kenmerk, &naam, &land, kvk, kritikaliteit.map(Into::into), nu)
        }
        Leveranciersopdracht::Toon { kenmerk } => toon(&kluis, &kenmerk, nu),
        Leveranciersopdracht::Overeenkomst {
            kenmerk,
            contract,
            ondertekend,
            verwerking_begon,
            meldtermijn_uren,
        } => overeenkomst(
            &mut kluis,
            &kenmerk,
            &contract,
            &ondertekend,
            verwerking_begon.as_deref(),
            meldtermijn_uren,
            nu,
        ),
        Leveranciersopdracht::Vindplaats { kenmerk, eis, aanduiding, toelichting } => {
            vindplaats(&mut kluis, &kenmerk, eis.into(), &aanduiding, toelichting, nu)
        }
        Leveranciersopdracht::Subverwerker { kenmerk, naam, land, dienst } => {
            subverwerker(&mut kluis, &kenmerk, &naam, &land, &dienst, nu)
        }
        Leveranciersopdracht::Subverwerkerscontrole { kenmerk, op } => {
            subverwerkerscontrole(&mut kluis, &kenmerk, op.as_deref(), nu)
        }
        Leveranciersopdracht::Weren { kenmerk, motivering } => {
            weren(&mut kluis, &kenmerk, &motivering, nu)
        }
        Leveranciersopdracht::Vaststellen { kenmerk } => vaststellen(&mut kluis, &kenmerk, nu),
    }
}

fn zoek(kluis: &Kluis, kenmerk: &str) -> Result<Leverancier> {
    let kop = kluis
        .lijst(SOORT)?
        .into_iter()
        .find(|r| r.kenmerk.as_deref() == Some(kenmerk))
        .ok_or_else(|| anyhow::anyhow!("geen leverancier met kenmerk '{kenmerk}'"))?;
    Ok(kluis.laad(SOORT, &kop.id)?)
}

fn bewaar(
    kluis: &mut Kluis,
    l: &Leverancier,
    handeling: Handeling,
    omschrijving: &str,
    nu: DateTime<Utc>,
) -> Result<()> {
    let actor = super::actor();
    kluis.bewaar(
        SOORT,
        &l.id.to_string(),
        COMPARTIMENT,
        l.status.omschrijving(),
        Some(&l.kenmerk),
        l,
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

fn drempel_uren(nu: DateTime<Utc>) -> u32 {
    dpofg_content::startpakket(nu.date_naive())
        .aanvullend
        .get("verwerker_meldtermijndrempel")
        .and_then(|v| v.get("drempel_uren"))
        .and_then(|v| v.as_u64())
        .and_then(|v| u32::try_from(v).ok())
        .unwrap_or(48)
}

fn toon_eisen() -> Result<()> {
    kop("De acht onderdelen van artikel 28 lid 3");
    let mut t = tabel(&["", "de verwerker", "grondslag"]);
    for eis in Contracteis::alle() {
        t.add_row(vec![eis.letter().to_string(), eis.omschrijving().to_string(), eis.grondslag()]);
    }
    println!("{t}");
    terzijde(
        "Elk onderdeel vraagt een vindplaats: artikel, bijlage of paragraaf. Een vinkje zegt \
         alleen dat iemand ooit dacht dat het geregeld was; bij een uitvraag moet worden \
         aangewezen wáár het staat.",
    );
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn nieuw(
    kluis: &mut Kluis,
    kenmerk: &str,
    naam: &str,
    land: &str,
    kvk: Option<String>,
    kritikaliteit: Option<Kritikaliteit>,
    nu: DateTime<Utc>,
) -> Result<()> {
    if kluis.lijst(SOORT)?.iter().any(|r| r.kenmerk.as_deref() == Some(kenmerk)) {
        anyhow::bail!("er bestaat al een leverancier met kenmerk '{kenmerk}'");
    }
    let mut l = Leverancier::nieuw(kenmerk, naam, land, &super::actor().id, nu);
    l.kvk_nummer = kvk;
    l.kritikaliteit = kritikaliteit;
    bewaar(kluis, &l, Handeling::RecordAangemaakt, "leverancier geregistreerd", nu)?;

    gelukt(&format!("leverancier {kenmerk} geregistreerd"));
    terzijde("Leg de verwerkersovereenkomst vast met 'dpofg leverancier overeenkomst'; 'leverancier eisen' toont wat erin moet staan.");
    toon_ontbrekend(&l);
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn overeenkomst(
    kluis: &mut Kluis,
    kenmerk: &str,
    contract: &str,
    ondertekend: &str,
    verwerking_begon: Option<&str>,
    meldtermijn_uren: Option<u32>,
    nu: DateTime<Utc>,
) -> Result<()> {
    let mut l = zoek(kluis, kenmerk)?;
    let getekend = lees_tijdstip(ondertekend)?;
    let begon = match verwerking_begon {
        Some(t) => Some(lees_tijdstip(t)?),
        None => None,
    };
    l.leg_overeenkomst_vast(contract, getekend, begon, meldtermijn_uren, nu)?;
    bewaar(
        kluis,
        &l,
        Handeling::RecordGewijzigd,
        &format!("overeenkomst {contract} vastgelegd"),
        nu,
    )?;

    gelukt(&format!("overeenkomst {contract} vastgelegd"));
    let o = l.overeenkomst.as_ref().expect("zojuist gezet");
    if o.getekend_na_aanvang() {
        let_op(
            "De overeenkomst is getekend nadat de verwerking al liep. Die periode is niet gedekt; \
             dat is een feit dat blijft staan en regel VWO-13 maakt het zichtbaar.",
        );
    }
    let drempel = drempel_uren(nu);
    if l.meldtermijn_te_lang(drempel) {
        let_op(&format!(
            "De verwerker krijgt meer dan {drempel} uur om te melden. Van de eigen termijn van \
             tweeënzeventig uur blijft dan te weinig over om te wegen en te melden."
        ));
    }
    toon_ontbrekend(&l);
    Ok(())
}

fn vindplaats(
    kluis: &mut Kluis,
    kenmerk: &str,
    eis: Contracteis,
    aanduiding: &str,
    toelichting: Option<String>,
    nu: DateTime<Utc>,
) -> Result<()> {
    let mut l = zoek(kluis, kenmerk)?;
    l.wijs_vindplaats_aan(eis, aanduiding, toelichting, nu)?;
    bewaar(kluis, &l, Handeling::RecordGewijzigd, &format!("vindplaats {}", eis.letter()), nu)?;

    gelukt(&format!("onderdeel {} staat in {aanduiding}", eis.letter()));
    terzijde(&eis.grondslag());
    let resterend = l.overeenkomst.as_ref().map(|o| o.eisen_zonder_vindplaats().len()).unwrap_or(8);
    if resterend > 0 {
        terzijde(&format!("nog {resterend} onderdeel/onderdelen zonder vindplaats"));
    }
    toon_ontbrekend(&l);
    Ok(())
}

fn subverwerker(
    kluis: &mut Kluis,
    kenmerk: &str,
    naam: &str,
    land: &str,
    dienst: &str,
    nu: DateTime<Utc>,
) -> Result<()> {
    let mut l = zoek(kluis, kenmerk)?;
    l.voeg_subverwerker_toe(naam, land, dienst, nu)?;
    bewaar(kluis, &l, Handeling::RecordGewijzigd, &format!("subverwerker {naam} toegevoegd"), nu)?;

    gelukt(&format!("subverwerker {naam} ({land}) toegevoegd"));
    terzijde(
        "Toevoegen is geen controle: de datum van de laatste controle blijft staan. Loop de lijst \
         na met 'dpofg leverancier subverwerkerscontrole'.",
    );
    toon_ontbrekend(&l);
    Ok(())
}

fn subverwerkerscontrole(
    kluis: &mut Kluis,
    kenmerk: &str,
    op: Option<&str>,
    nu: DateTime<Utc>,
) -> Result<()> {
    let mut l = zoek(kluis, kenmerk)?;
    let moment = match op {
        Some(t) => lees_tijdstip(t)?,
        None => nu,
    };
    l.controleer_subverwerkers(moment, nu)?;
    bewaar(kluis, &l, Handeling::RecordGewijzigd, "subverwerkerslijst nagelopen", nu)?;

    gelukt(&format!("subverwerkerslijst nagelopen op {}", moment.format("%d-%m-%Y")));
    terzijde(&format!("{} subverwerker(s) in de lijst", l.subverwerkers.len()));
    toon_ontbrekend(&l);
    Ok(())
}

fn weren(kluis: &mut Kluis, kenmerk: &str, motivering: &str, nu: DateTime<Utc>) -> Result<()> {
    let mut l = zoek(kluis, kenmerk)?;
    l.weringsbesluit = Some(Motivering::nieuw(motivering, &super::actor().id, nu)?);
    l.herkomst.wijzig("weringsbesluit vastgelegd", nu);
    bewaar(kluis, &l, Handeling::BesluitGenomen, "weringsbesluit vastgelegd", nu)?;

    gelukt(&format!("weringsbesluit voor {} vastgelegd", l.naam));
    let_op(
        "Het besluit staat vast; de uitvoering ervan niet. Leg vast wanneer de leverancier \
         daadwerkelijk is vervangen, anders blijft de verwerking lopen bij een partij die is \
         geweerd.",
    );
    Ok(())
}

fn vaststellen(kluis: &mut Kluis, kenmerk: &str, nu: DateTime<Utc>) -> Result<()> {
    let mut l = zoek(kluis, kenmerk)?;
    let actor = super::actor();
    let id = l.id.to_string();
    match l.stel_vast(&actor.naam, nu) {
        Ok(()) => {
            bewaar(kluis, &l, Handeling::RecordVastgesteld, "leverancier vastgesteld", nu)?;
            gelukt(&format!("leverancier {kenmerk} vastgesteld"));
            toon_ontbrekend(&l);
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
                    l.compartiment.naam(),
                    "vaststellen geweigerd: verplichte onderdelen ontbreken",
                ),
                Some(fout.to_string()),
            )?;
            kop("Vaststellen is niet gelukt");
            toon_ontbrekend(&l);
            anyhow::bail!("{fout}")
        }
    }
}

fn lijst(kluis: &Kluis, nu: DateTime<Utc>) -> Result<()> {
    let koppen = kluis.lijst(SOORT)?;
    if koppen.is_empty() {
        kop("Leveranciers");
        terzijde("Er staan nog geen leveranciers in de kluis.");
        return Ok(());
    }
    kop("Leveranciers");
    let mut t =
        tabel(&["kenmerk", "naam", "land", "overeenkomst", "zonder vindplaats", "meldtermijn"]);
    for k in &koppen {
        let l: Leverancier = kluis.laad(SOORT, &k.id)?;
        let (contract, zonder, termijn) = match &l.overeenkomst {
            None => ("geen".to_string(), "—".to_string(), "—".to_string()),
            Some(o) => (
                o.kenmerk.clone(),
                o.eisen_zonder_vindplaats().len().to_string(),
                o.meldtermijn_uren
                    .map(|u| format!("{u} uur"))
                    .unwrap_or_else(|| "niet afgesproken".into()),
            ),
        };
        t.add_row(vec![
            l.kenmerk.clone(),
            l.naam.clone(),
            l.land.clone(),
            contract,
            zonder,
            termijn,
        ]);
    }
    println!("{t}");
    terzijde(&format!("norm: een verwerker meldt binnen {} uur", drempel_uren(nu)));
    Ok(())
}

fn toon(kluis: &Kluis, kenmerk: &str, nu: DateTime<Utc>) -> Result<()> {
    let l = zoek(kluis, kenmerk)?;
    kop(&format!("Leverancier {}", l.kenmerk));
    let mut t = tabel(&["", ""]);
    t.add_row(vec!["naam", &l.naam]);
    t.add_row(vec!["land", &l.land]);
    t.add_row(vec!["status", l.status.omschrijving()]);
    if let Some(k) = l.kritikaliteit {
        t.add_row(vec!["kritikaliteit", k.omschrijving()]);
    }
    if let Some(k) = &l.kvk_nummer {
        t.add_row(vec!["KvK", k]);
    }
    println!("{t}");

    if let Some(o) = &l.overeenkomst {
        kop("Verwerkersovereenkomst");
        let mut t = tabel(&["", ""]);
        t.add_row(vec!["kenmerk", &o.kenmerk]);
        t.add_row(vec!["ondertekend op", &o.ondertekend_op.format("%d-%m-%Y").to_string()]);
        if let Some(b) = o.verwerking_begon_op {
            t.add_row(vec!["verwerking begon op", &b.format("%d-%m-%Y").to_string()]);
        }
        t.add_row(vec![
            "meldtermijn",
            &o.meldtermijn_uren
                .map(|u| format!("{u} uur"))
                .unwrap_or_else(|| "niet afgesproken".into()),
        ]);
        println!("{t}");
        if o.getekend_na_aanvang() {
            blokkade("de overeenkomst is getekend nadat de verwerking al liep; die periode is niet gedekt");
            terzijde("art. 28 lid 3 AVG");
        }
        if l.meldtermijn_te_lang(drempel_uren(nu)) {
            let_op("de contractuele meldtermijn is langer dan de norm");
            terzijde("art. 33 lid 2 AVG");
        }

        kop("Artikel 28 lid 3");
        let mut t = tabel(&["", "de verwerker", "vindplaats"]);
        for eis in Contracteis::alle() {
            let plaats = o
                .vindplaatsen
                .iter()
                .find(|v| v.eis == eis)
                .map(|v| v.aanduiding.clone())
                .unwrap_or_else(|| "—".into());
            t.add_row(vec![eis.letter().to_string(), eis.omschrijving().to_string(), plaats]);
        }
        println!("{t}");
    }

    if !l.subverwerkers.is_empty() {
        kop("Subverwerkers");
        let mut t = tabel(&["naam", "land", "dienst"]);
        for s in &l.subverwerkers {
            t.add_row(vec![s.naam.clone(), s.land.clone(), s.dienst.clone()]);
        }
        println!("{t}");
    }
    if let Some(m) = l.maanden_sinds_subverwerkerscontrole(nu) {
        terzijde(&format!("lijst {m} maanden geleden nagelopen"));
    }
    if let Some(w) = &l.weringsbesluit {
        kop("Weringsbesluit");
        println!("  {}", w.tekst);
    }

    toon_ontbrekend(&l);
    Ok(())
}

fn toon_ontbrekend(l: &Leverancier) {
    let r = l.volledigheid();
    kop("Volledigheid");
    println!("  {}", voortgang(r.compleet, r.verplicht));
    if r.is_volledig() {
        println!();
        gelukt("alle verplichte onderdelen zijn ingevuld");
        return;
    }
    println!();
    for o in &r.ontbreekt {
        let veld = o.veld.trim_start_matches("leverancier.");
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
