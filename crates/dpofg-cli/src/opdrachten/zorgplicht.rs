//! De zorgplichtcontrolset van artikel 21 lid 3 van de Cyberbeveiligingswet.

use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use clap::Subcommand;
use dpofg_audit::Handeling;
use dpofg_domain::{
    zorgplicht::{
        Bestuursvaststelling, Bewijsaanwijzing, Bewijskracht, Bewijsrol, Kaderdefinitie,
        Maatregelstand, Niettoepassing, Toepassing, Zorgplichtdossier, Zorgplichtonderdeel,
    },
    Motivering, Volledig,
};
use dpofg_store::Kluis;
use std::path::PathBuf;

use crate::uitvoer::{blokkade, gelukt, kop, let_op, tabel, terzijde, voortgang};

const SOORT: &str = "zorgplicht";
const COMPARTIMENT: &str = "algemeen";

#[derive(Subcommand, Debug)]
pub enum Zorgplichtopdracht {
    /// Toon de tien onderdelen van artikel 21 lid 3.
    Onderdelen,
    /// Toon de normenkaders die het kennispakket bevat.
    Kaders,
    /// Leid een controlset af uit een normenkader.
    Afleiden {
        /// Kenmerk van het dossier.
        kenmerk: String,
        /// De naam van de entiteit.
        #[arg(long)]
        naam: String,
        /// Het kenmerk van het kader uit het kennispakket.
        #[arg(long, default_value = "CBB-ZORGPLICHT-A")]
        kader: String,
        /// De naam van de aangemelde functionaris.
        #[arg(long)]
        functionaris: String,
        /// Alleen bij een voorgeschreven kader: de regeling die het voorschrijft.
        #[arg(long)]
        regeling: Option<String>,
    },
    /// Toon alle zorgplichtdossiers.
    Lijst,
    /// Toon één dossier, met de stand per maatregel.
    Toon {
        /// Kenmerk van het dossier.
        kenmerk: String,
        /// Beperk tot één onderdeel, als letter a tot en met j.
        #[arg(long)]
        onder: Option<String>,
    },
    /// Wijs een eigenaar aan voor één maatregel.
    Eigenaar {
        /// Kenmerk van het dossier.
        kenmerk: String,
        /// De maatregelcode.
        #[arg(long)]
        maatregel: String,
        /// De rol die de maatregel uitvoert.
        #[arg(long)]
        rol: String,
        /// Wie die rol op dit moment vervult.
        #[arg(long)]
        persoon: String,
    },
    /// Leg vast dat een maatregel is ingericht.
    Inrichten {
        /// Kenmerk van het dossier.
        kenmerk: String,
        /// De maatregelcode.
        #[arg(long)]
        maatregel: String,
    },
    /// Leg vast dat een maatregel gemotiveerd niet wordt toegepast.
    NietToepassen {
        /// Kenmerk van het dossier.
        kenmerk: String,
        /// De maatregelcode.
        #[arg(long)]
        maatregel: String,
        /// De onderbouwing.
        #[arg(long)]
        motivering: String,
        /// Bij een voorgeschreven kader: de regeling die de afwijking toestaat.
        #[arg(long)]
        regeling: Option<String>,
        /// Bij een voorgeschreven kader: het artikel in die regeling.
        #[arg(long)]
        artikel: Option<String>,
    },
    /// Stel zelf vast hoe vaak een periodieke maatregel wordt uitgevoerd.
    Frequentie {
        /// Kenmerk van het dossier.
        kenmerk: String,
        /// De maatregelcode.
        #[arg(long)]
        maatregel: String,
        /// Het aantal maanden tussen twee uitvoeringen.
        #[arg(long)]
        maanden: u32,
        /// Wie de termijn heeft vastgesteld.
        #[arg(long)]
        door: String,
        /// Waarom die termijn passend is.
        #[arg(long)]
        motivering: String,
    },
    /// Wijs een bewijsstuk aan bij een maatregel.
    Bewijs {
        /// Kenmerk van het dossier.
        kenmerk: String,
        /// De maatregelcode.
        #[arg(long)]
        maatregel: String,
        /// Wat het stuk bewijst.
        #[arg(long, value_enum)]
        rol: Bewijsrolkeuze,
        /// Het bestand dat als bewijs dient.
        #[arg(long)]
        bestand: PathBuf,
        /// Wat dit stuk bewijst, in gewone taal.
        #[arg(long)]
        omschrijving: String,
        /// Vanaf wanneer het geldt. Standaard: nu.
        #[arg(long)]
        geldig_van: Option<String>,
        /// Tot wanneer het geldt.
        #[arg(long)]
        geldig_tot: String,
        /// Het stuk is door een ander getoetst, niet zelf opgesteld.
        #[arg(long)]
        geverifieerd: bool,
    },
    /// Trek een aangewezen bewijsstuk in.
    ///
    /// Intrekken is geen verwijderen: het stuk blijft in het dossier staan met
    /// de reden erbij, en telt vanaf dat moment niet meer mee.
    BewijsIntrekken {
        /// Kenmerk van het dossier.
        kenmerk: String,
        /// De maatregelcode.
        #[arg(long)]
        maatregel: String,
        /// De naam van het bestand dat wordt ingetrokken.
        #[arg(long)]
        bestand: String,
        /// Wat het stuk bewees.
        #[arg(long, value_enum)]
        rol: Bewijsrolkeuze,
        /// Waarom het wordt ingetrokken.
        #[arg(long)]
        motivering: String,
    },
    /// Koppel de risicobeoordeling waarop de controlset steunt.
    Risicobeoordeling {
        /// Kenmerk van het dossier.
        kenmerk: String,
        /// Het kenmerk van de risicobeoordeling.
        #[arg(long)]
        beoordeling: String,
    },
    /// Leg het bestuursbesluit vast waarmee het pakket is vastgesteld.
    Bestuursvaststelling {
        /// Kenmerk van het dossier.
        kenmerk: String,
        /// Wanneer het besluit is genomen. Standaard: nu.
        #[arg(long)]
        datum: Option<String>,
        /// Wat het bestuur heeft besloten.
        #[arg(long)]
        besluit: String,
        /// Wie erbij was. Herhaalbaar.
        #[arg(long = "aanwezige", required = true)]
        aanwezigen: Vec<String>,
        /// Het besluit of verslag als bestand.
        #[arg(long)]
        bestand: PathBuf,
        /// Tot wanneer het stuk als bewijs geldt.
        #[arg(long)]
        geldig_tot: String,
    },
    /// Leg een nieuwe aangemelde functionaris vast.
    Functionaris {
        /// Kenmerk van het dossier.
        kenmerk: String,
        /// De naam.
        #[arg(long)]
        naam: String,
    },
    /// Toon wat er binnen een aantal dagen niet meer aantoonbaar is.
    Vervalt {
        /// Kenmerk van het dossier.
        kenmerk: String,
        /// De horizon in dagen.
        #[arg(long, default_value = "90", value_parser = clap::value_parser!(i64).range(1..=3650))]
        dagen: i64,
    },
    /// Stel het dossier vast.
    Vaststellen {
        /// Kenmerk van het dossier.
        kenmerk: String,
    },
}

#[derive(clap::ValueEnum, Clone, Copy, Debug)]
pub enum Bewijsrolkeuze {
    Vaststelling,
    Uitvoering,
    Toetsing,
}

impl From<Bewijsrolkeuze> for Bewijsrol {
    fn from(k: Bewijsrolkeuze) -> Self {
        match k {
            Bewijsrolkeuze::Vaststelling => Bewijsrol::Vaststelling,
            Bewijsrolkeuze::Uitvoering => Bewijsrol::Uitvoering,
            Bewijsrolkeuze::Toetsing => Bewijsrol::Toetsing,
        }
    }
}

pub fn draai(o: Zorgplichtopdracht, kluispad: Option<PathBuf>, nu: DateTime<Utc>) -> Result<()> {
    match o {
        Zorgplichtopdracht::Onderdelen => return toon_onderdelen(),
        Zorgplichtopdracht::Kaders => return toon_kaders(nu),
        _ => {}
    }
    let pad = super::kluispad(kluispad)?;
    let mut kluis = super::open_kluis(&pad, nu)?;
    match o {
        Zorgplichtopdracht::Onderdelen | Zorgplichtopdracht::Kaders => {
            unreachable!("hierboven afgehandeld")
        }
        Zorgplichtopdracht::Afleiden { kenmerk, naam, kader, functionaris, regeling } => {
            afleiden(&mut kluis, &kenmerk, &naam, &kader, &functionaris, regeling, nu)
        }
        Zorgplichtopdracht::Lijst => lijst(&kluis, nu),
        Zorgplichtopdracht::Toon { kenmerk, onder } => toon(&kluis, &kenmerk, onder.as_deref(), nu),
        Zorgplichtopdracht::Eigenaar { kenmerk, maatregel, rol, persoon } => {
            eigenaar(&mut kluis, &kenmerk, &maatregel, &rol, &persoon, nu)
        }
        Zorgplichtopdracht::Inrichten { kenmerk, maatregel } => {
            inrichten(&mut kluis, &kenmerk, &maatregel, nu)
        }
        Zorgplichtopdracht::NietToepassen { kenmerk, maatregel, motivering, regeling, artikel } => {
            niet_toepassen(
                &mut kluis,
                &kenmerk,
                &maatregel,
                &motivering,
                regeling.as_deref(),
                artikel.as_deref(),
                nu,
            )
        }
        Zorgplichtopdracht::Frequentie { kenmerk, maatregel, maanden, door, motivering } => {
            frequentie(&mut kluis, &kenmerk, &maatregel, maanden, &door, &motivering, nu)
        }
        Zorgplichtopdracht::Bewijs {
            kenmerk,
            maatregel,
            rol,
            bestand,
            omschrijving,
            geldig_van,
            geldig_tot,
            geverifieerd,
        } => bewijs(
            &mut kluis,
            &kenmerk,
            &maatregel,
            rol.into(),
            &bestand,
            &omschrijving,
            geldig_van.as_deref(),
            &geldig_tot,
            geverifieerd,
            nu,
        ),
        Zorgplichtopdracht::BewijsIntrekken { kenmerk, maatregel, bestand, rol, motivering } => {
            bewijs_intrekken(
                &mut kluis,
                &kenmerk,
                &maatregel,
                &bestand,
                rol.into(),
                &motivering,
                nu,
            )
        }
        Zorgplichtopdracht::Risicobeoordeling { kenmerk, beoordeling } => {
            risicobeoordeling(&mut kluis, &kenmerk, &beoordeling, nu)
        }
        Zorgplichtopdracht::Bestuursvaststelling {
            kenmerk,
            datum,
            besluit,
            aanwezigen,
            bestand,
            geldig_tot,
        } => bestuursvaststelling(
            &mut kluis,
            &kenmerk,
            datum.as_deref(),
            &besluit,
            &aanwezigen,
            &bestand,
            &geldig_tot,
            nu,
        ),
        Zorgplichtopdracht::Functionaris { kenmerk, naam } => {
            functionaris(&mut kluis, &kenmerk, &naam, nu)
        }
        Zorgplichtopdracht::Vervalt { kenmerk, dagen } => vervalt(&kluis, &kenmerk, dagen, nu),
        Zorgplichtopdracht::Vaststellen { kenmerk } => vaststellen(&mut kluis, &kenmerk, nu),
    }
}

// --------------------------------------------------------------------------
// Hulp
// --------------------------------------------------------------------------

fn zoek(kluis: &Kluis, kenmerk: &str) -> Result<Zorgplichtdossier> {
    let kop = kluis
        .lijst(SOORT)?
        .into_iter()
        .find(|r| r.kenmerk.as_deref() == Some(kenmerk))
        .ok_or_else(|| anyhow::anyhow!("geen zorgplichtdossier met kenmerk '{kenmerk}'"))?;
    Ok(kluis.laad(SOORT, &kop.id)?)
}

fn bewaar(
    kluis: &mut Kluis,
    d: &Zorgplichtdossier,
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
            "kon '{tekst}' niet lezen als tijdstip ({e}). Gebruik de vorm 2026-08-19T09:00:00Z"
        )
    })
}

/// De kaders uit het kennispakket, met de kaders die niet te lezen zijn.
///
/// Een onleesbaar kader stilzwijgend overslaan zou de melding "het pakket
/// bevat geen kader X" opleveren terwijl het er wél in staat, alleen met een
/// fout erin. Dat is een bewering over andermans inhoud die niet klopt, en de
/// beheerder van het pakket komt er dan nooit achter.
fn kaders(nu: DateTime<Utc>) -> (Vec<Kaderdefinitie>, Vec<(String, String)>) {
    let pakket = dpofg_content::startpakket(nu.date_naive());
    let mut goed = Vec::new();
    let mut stuk = Vec::new();
    for (sleutel, waarde) in
        pakket.aanvullend.iter().filter(|(sleutel, _)| sleutel.starts_with("zorgplicht_kader_"))
    {
        match serde_json::from_value::<Kaderdefinitie>(waarde.clone()) {
            Ok(k) => goed.push(k),
            Err(e) => stuk.push((sleutel.clone(), e.to_string())),
        }
    }
    (goed, stuk)
}

fn vaste_tekst(nu: DateTime<Utc>, sleutel: &str) -> Option<String> {
    dpofg_content::startpakket(nu.date_naive())
        .aanvullend
        .get("zorgplicht_teksten")
        .and_then(|v| v.get(sleutel))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

fn onderdeel_uit_letter(letter: &str) -> Option<Zorgplichtonderdeel> {
    Zorgplichtonderdeel::alle().into_iter().find(|o| o.letter() == letter.trim().to_lowercase())
}

/// Neemt een bestand op in de kluis en levert de bewijsaanwijzing.
///
/// De gebruiker typt nooit een hash over: hij wijst een bestand aan, dat gaat
/// versleuteld de kluis in en de inhoudshash komt hier vandaan.
#[allow(clippy::too_many_arguments)]
fn neem_bewijs_op(
    kluis: &mut Kluis,
    record_id: &str,
    bestand: &std::path::Path,
    rol: Bewijsrol,
    omschrijving: &str,
    geldig_van: DateTime<Utc>,
    geldig_tot: DateTime<Utc>,
    geverifieerd: bool,
    nu: DateTime<Utc>,
) -> Result<Bewijsaanwijzing> {
    let inhoud = std::fs::read(bestand)
        .map_err(|e| anyhow::anyhow!("kon '{}' niet lezen: {e}", bestand.display()))?;
    if inhoud.is_empty() {
        anyhow::bail!(
            "'{}' is leeg; een leeg bestand als bewijs aanwijzen maakt het dossier \
             onbetrouwbaar",
            bestand.display()
        );
    }
    let bestandsnaam = bestand
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| bestand.display().to_string());
    let actor = super::actor();
    let hash =
        kluis.bijlage_toevoegen(record_id, COMPARTIMENT, &bestandsnaam, &inhoud, &actor, nu)?;
    Ok(Bewijsaanwijzing {
        rol,
        omschrijving: omschrijving.to_string(),
        bijlagehash: hash,
        bestandsnaam,
        geldig_van,
        geldig_tot,
        bewijskracht: if geverifieerd {
            Bewijskracht::Geverifieerd
        } else {
            Bewijskracht::Zelfgerapporteerd
        },
        aangewezen_door: actor.naam.clone(),
        aangewezen_op: nu,
        ingetrokken: None,
    })
}

// --------------------------------------------------------------------------
// De opdrachten
// --------------------------------------------------------------------------

fn toon_onderdelen() -> Result<()> {
    kop("De tien onderdelen van artikel 21 lid 3");
    let mut t = tabel(&["", "onderwerp", "grondslag"]);
    for o in Zorgplichtonderdeel::alle() {
        t.add_row(vec![o.letter().to_string(), o.omschrijving().to_string(), o.grondslag()]);
    }
    println!("{t}");
    terzijde(
        "De omschrijvingen zijn samenvattingen en geen citaten. De Nederlandse wet nummert \
         anders dan de richtlijn: waar NIS2 spreekt van artikel 21 lid 2, staat het in de \
         Cyberbeveiligingswet in lid 3.",
    );
    Ok(())
}

fn toon_kaders(nu: DateTime<Utc>) -> Result<()> {
    kop("Normenkaders in het kennispakket");
    let (gevonden, stuk) = kaders(nu);
    if gevonden.is_empty() && stuk.is_empty() {
        terzijde("het kennispakket bevat geen normenkader voor de zorgplicht");
        return Ok(());
    }
    let mut t = tabel(&["kenmerk", "variant", "versie", "maatregelen", "geverifieerd"]);
    for k in &gevonden {
        t.add_row(vec![
            k.kenmerk.clone(),
            k.variant.letter().to_string(),
            k.versie.clone(),
            k.maatregelen.len().to_string(),
            k.geverifieerd_op.map(|d| d.to_string()).unwrap_or_else(|| "nee".into()),
        ]);
    }
    println!("{t}");
    for k in &gevonden {
        terzijde(&format!("{}: {}", k.kenmerk, k.bron));
        if k.geverifieerd_op.is_none() {
            if let Some(tekst) = vaste_tekst(nu, "bij_ongeverifieerd_kader") {
                let_op(&tekst);
            }
        }
    }
    for (sleutel, fout) in &stuk {
        blokkade(&format!("'{sleutel}' staat in het pakket maar is niet te lezen: {fout}"));
    }
    terzijde(
        "Wat er niet staat, is niet te kiezen. Een variant aanklikken waarvoor geen inhoud \
         bestaat, zou een leeg dossier opleveren dat er compleet uitziet.",
    );
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn afleiden(
    kluis: &mut Kluis,
    kenmerk: &str,
    naam: &str,
    kaderkenmerk: &str,
    functionaris: &str,
    regeling: Option<String>,
    nu: DateTime<Utc>,
) -> Result<()> {
    if kluis.lijst(SOORT)?.iter().any(|r| r.kenmerk.as_deref() == Some(kenmerk)) {
        anyhow::bail!("er bestaat al een zorgplichtdossier met kenmerk '{kenmerk}'");
    }
    let (beschikbaar, stuk) = kaders(nu);
    if !stuk.is_empty() {
        for (sleutel, fout) in &stuk {
            let_op(&format!("'{sleutel}' staat in het pakket maar is niet te lezen: {fout}"));
        }
    }
    let kader = beschikbaar.iter().find(|k| k.kenmerk == kaderkenmerk).ok_or_else(|| {
        anyhow::anyhow!(
            "het kennispakket bevat geen leesbaar kader '{kaderkenmerk}'. Bekijk wat er is met \
             'dpofg zorgplicht kaders'"
        )
    })?;

    let d = Zorgplichtdossier::leid_af(
        kenmerk,
        naam,
        functionaris,
        kader,
        regeling,
        &super::actor().id,
        nu,
    )?;
    bewaar(kluis, &d, Handeling::RecordAangemaakt, "controlset afgeleid", nu)?;

    gelukt(&format!(
        "{} maatregelen afgeleid uit {} ({})",
        d.maatregelen.len(),
        d.kaderkenmerk,
        d.kaderversie
    ));
    terzijde(
        "De set is afgeleid en niet samengesteld: er is geen opdracht om er een maatregel bij \
         te zetten of uit te halen. Wie de norm wil wijzigen, wijzigt het kennispakket.",
    );
    if d.kader_geverifieerd_op.is_none() {
        if let Some(tekst) = vaste_tekst(nu, "bij_ongeverifieerd_kader") {
            let_op(&tekst);
        }
    }
    toon_ontbrekend(&d);
    Ok(())
}

fn eigenaar(
    kluis: &mut Kluis,
    kenmerk: &str,
    code: &str,
    rol: &str,
    persoon: &str,
    nu: DateTime<Utc>,
) -> Result<()> {
    let mut d = zoek(kluis, kenmerk)?;
    d.wijs_eigenaar_toe(code, rol, persoon, nu)?;
    bewaar(kluis, &d, Handeling::RecordGewijzigd, &format!("eigenaar van {code}"), nu)?;
    gelukt(&format!("{code} is belegd bij {rol} ({persoon})"));
    toon_ontbrekend(&d);
    Ok(())
}

fn inrichten(kluis: &mut Kluis, kenmerk: &str, code: &str, nu: DateTime<Utc>) -> Result<()> {
    let mut d = zoek(kluis, kenmerk)?;
    d.richt_in(code, nu)?;
    bewaar(kluis, &d, Handeling::RecordGewijzigd, &format!("{code} ingericht"), nu)?;

    gelukt(&format!("{code} staat op ingericht"));
    let stand = d.maatregel(code).map(|m| m.stand(nu));
    if stand == Some(Maatregelstand::VastgesteldNietAantoonbaar) {
        let_op(
            "Ingericht is nog niet aantoonbaar. Wijs een bewijsstuk van de uitvoering aan met \
             'dpofg zorgplicht bewijs'; tot dan blijft deze maatregel vastgesteld en niet \
             aantoonbaar.",
        );
    }
    toon_ontbrekend(&d);
    Ok(())
}

fn niet_toepassen(
    kluis: &mut Kluis,
    kenmerk: &str,
    code: &str,
    motivering: &str,
    regeling: Option<&str>,
    artikel: Option<&str>,
    nu: DateTime<Utc>,
) -> Result<()> {
    let mut d = zoek(kluis, kenmerk)?;
    let m = Motivering::nieuw(motivering, &super::actor().id, nu)?;
    let keuze = match (regeling, artikel) {
        (None, None) => Niettoepassing::EigenMotivering(m),
        (Some(r), Some(a)) => Niettoepassing::GrondslagInRegeling {
            regeling: r.to_string(),
            artikel: a.to_string(),
            motivering: m,
        },
        _ => anyhow::bail!(
            "noem de regeling en het artikel samen; een regeling zonder artikelaanduiding is \
             niet na te lopen"
        ),
    };
    d.pas_niet_toe(code, keuze, nu)?;
    bewaar(kluis, &d, Handeling::BesluitGenomen, &format!("{code} niet toegepast"), nu)?;

    gelukt(&format!("{code} staat op gemotiveerd niet toegepast"));
    terzijde(
        "Dit is geen tekortkoming die verdwijnt, maar een besluit dat blijft staan. Het telt \
         mee in het aandeel niet-toegepaste maatregelen dat aan de directie wordt gemeld.",
    );
    toon_ontbrekend(&d);
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn frequentie(
    kluis: &mut Kluis,
    kenmerk: &str,
    code: &str,
    maanden: u32,
    door: &str,
    motivering: &str,
    nu: DateTime<Utc>,
) -> Result<()> {
    let mut d = zoek(kluis, kenmerk)?;
    let m = Motivering::nieuw(motivering, &super::actor().id, nu)?;
    d.stel_frequentie_vast(code, maanden, door, m, nu)?;
    bewaar(kluis, &d, Handeling::RecordGewijzigd, &format!("frequentie van {code}"), nu)?;

    gelukt(&format!("{code} wordt eens per {maanden} maanden uitgevoerd"));
    terzijde(
        "De wet noemt hier geen termijn. Deze is zelf vastgesteld; daarom staan er een naam en \
         een motivering bij, en daarom is hij later te herzien.",
    );
    toon_ontbrekend(&d);
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn bewijs(
    kluis: &mut Kluis,
    kenmerk: &str,
    code: &str,
    rol: Bewijsrol,
    bestand: &std::path::Path,
    omschrijving: &str,
    geldig_van: Option<&str>,
    geldig_tot: &str,
    geverifieerd: bool,
    nu: DateTime<Utc>,
) -> Result<()> {
    let mut d = zoek(kluis, kenmerk)?;
    let van = match geldig_van {
        Some(t) => lees_tijdstip(t)?,
        None => nu,
    };
    let tot = lees_tijdstip(geldig_tot)?;
    let id = d.id.to_string();
    // Eerst keuren, dan pas opslaan. Het bestand gaat versleuteld de kluis in
    // en die handeling hangt in de keten; die is niet terug te draaien. Een
    // typefout in de maatregelcode zou anders een bijlage achterlaten die aan
    // geen enkele maatregel hangt en die niemand meer weg kan halen.
    d.keur_bewijs(code, rol, omschrijving, van, tot, nu)?;
    let aanwijzing =
        neem_bewijs_op(kluis, &id, bestand, rol, omschrijving, van, tot, geverifieerd, nu)?;
    let hash = aanwijzing.bijlagehash.clone();
    d.wijs_bewijs_aan(code, aanwijzing, nu)?;
    bewaar(kluis, &d, Handeling::BijlageToegevoegd, &format!("bewijs bij {code}"), nu)?;

    gelukt(&format!("{} opgenomen als bewijs bij {code}", bestand.display()));
    terzijde(&format!("inhoudshash {}…", &hash[..16]));
    let m = d.maatregel(code).expect("zojuist gewijzigd");
    terzijde(&format!("stand van {code}: {}", m.stand(nu).omschrijving()));
    if rol != Bewijsrol::Uitvoering && m.stand(nu) == Maatregelstand::VastgesteldNietAantoonbaar {
        let_op(
            "Dit stuk bewijst niet dat de maatregel is uitgevoerd. Alleen bewijs met de rol \
             uitvoering maakt een maatregel aantoonbaar.",
        );
    }
    if let Some(tekst) = vaste_tekst(nu, "bij_elke_uitvoer") {
        terzijde(&tekst);
    }
    toon_ontbrekend(&d);
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn bewijs_intrekken(
    kluis: &mut Kluis,
    kenmerk: &str,
    code: &str,
    bestand: &str,
    rol: Bewijsrol,
    motivering: &str,
    nu: DateTime<Utc>,
) -> Result<()> {
    let mut d = zoek(kluis, kenmerk)?;
    let actor = super::actor();
    let m = Motivering::nieuw(motivering, &actor.id, nu)?;
    let was_vastgesteld = d.status == dpofg_domain::Status::Vastgesteld;
    let gedaald = d.trek_bewijs_in(code, bestand, rol, &actor.naam, m, nu)?;
    bewaar(kluis, &d, Handeling::RecordGewijzigd, &format!("bewijs bij {code} ingetrokken"), nu)?;

    gelukt(&format!("'{bestand}' telt niet meer mee bij {code}"));
    terzijde(
        "Het stuk blijft in het dossier staan met de reden erbij. Intrekken is geen \
         verwijderen: wat ooit is aangewezen, kan de grond zijn geweest onder een rapport of \
         een vaststelling.",
    );
    let stand = d.maatregel(code).map(|m| m.stand(nu));
    if let Some(stand) = stand {
        terzijde(&format!("stand van {code}: {}", stand.omschrijving()));
    }
    if gedaald && was_vastgesteld {
        blokkade(
            "Het dossier staat hierdoor op herziening nodig: de vaststelling rustte mede op \
             dit stuk.",
        );
    } else if gedaald {
        let_op("Deze maatregel is hierdoor niet langer aantoonbaar.");
    }
    toon_ontbrekend(&d);
    Ok(())
}

fn risicobeoordeling(
    kluis: &mut Kluis,
    kenmerk: &str,
    beoordelingskenmerk: &str,
    nu: DateTime<Utc>,
) -> Result<()> {
    let mut d = zoek(kluis, kenmerk)?;
    let b = super::risico::zoek(kluis, beoordelingskenmerk).map_err(|_| {
        anyhow::anyhow!(
            "geen risicobeoordeling met kenmerk '{beoordelingskenmerk}'. Maak er een met \
             'dpofg risico nieuw' of bekijk de lijst met 'dpofg risico lijst'"
        )
    })?;
    d.koppel_risicobeoordeling(&b.kenmerk, b.id, nu)?;
    bewaar(kluis, &d, Handeling::RecordGewijzigd, "risicobeoordeling gekoppeld", nu)?;

    gelukt(&format!("{} is gekoppeld aan beoordeling {}", d.kenmerk, b.kenmerk));
    terzijde(&format!("reikwijdte: {}", b.reikwijdte));
    terzijde(&format!("geldig tot {}", b.geldig_tot.format("%d-%m-%Y")));
    if b.is_verlopen(nu) {
        let_op("Deze beoordeling is verlopen; regel ZRP-11 blijft dat melden.");
    }
    if b.status != dpofg_domain::Status::Vastgesteld {
        let_op(
            "Deze beoordeling is nog niet vastgesteld. Een controlset kan niet steunen op een \
             beoordeling die zelf nog concept is.",
        );
    }
    toon_ontbrekend(&d);
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn bestuursvaststelling(
    kluis: &mut Kluis,
    kenmerk: &str,
    datum: Option<&str>,
    besluit: &str,
    aanwezigen: &[String],
    bestand: &std::path::Path,
    geldig_tot: &str,
    nu: DateTime<Utc>,
) -> Result<()> {
    let mut d = zoek(kluis, kenmerk)?;
    let moment = match datum {
        Some(t) => lees_tijdstip(t)?,
        None => nu,
    };
    let tot = lees_tijdstip(geldig_tot)?;
    let id = d.id.to_string();
    let versie = d.kaderversie.clone();
    let aanwijzing = neem_bewijs_op(
        kluis,
        &id,
        bestand,
        Bewijsrol::Vaststelling,
        "besluit van het bestuur",
        moment,
        tot,
        false,
        nu,
    )?;
    d.leg_bestuursvaststelling_vast(
        Bestuursvaststelling {
            datum: moment,
            besluittekst: besluit.to_string(),
            // De kaderversie wordt door de tool ingevuld en niet door de
            // gebruiker: die zou hem kunnen overtypen, en dan gaat de
            // goedkeuring stilzwijgend over een andere versie.
            goedgekeurde_kaderversie: versie.clone(),
            aanwezigen: aanwezigen.to_vec(),
            bewijs: aanwijzing,
        },
        nu,
    )?;
    bewaar(kluis, &d, Handeling::BesluitGenomen, "bestuursvaststelling vastgelegd", nu)?;

    gelukt(&format!("het bestuur stelde kaderversie {versie} vast"));
    terzijde(&format!("{} aanwezige(n)", aanwezigen.len()));
    toon_ontbrekend(&d);
    Ok(())
}

fn functionaris(kluis: &mut Kluis, kenmerk: &str, naam: &str, nu: DateTime<Utc>) -> Result<()> {
    let mut d = zoek(kluis, kenmerk)?;
    d.wijzig_aangemelde_functionaris(naam, nu)?;
    bewaar(kluis, &d, Handeling::RecordGewijzigd, "aangemelde functionaris gewijzigd", nu)?;

    gelukt(&format!("{naam} staat vastgelegd als de aangemelde functionaris"));
    let conflicten: Vec<String> = d.eigenaarsconflicten().iter().map(|m| m.code.clone()).collect();
    if conflicten.is_empty() {
        return Ok(());
    }
    blokkade(&format!(
        "{naam} is eigenaar van {}; toezicht op het eigen werk is geen toezicht",
        conflicten.join(", ")
    ));
    terzijde("art. 38 lid 6 AVG · regel ZRP-02 houdt dit zichtbaar tot het is belegd");
    Ok(())
}

fn vervalt(kluis: &Kluis, kenmerk: &str, dagen: i64, nu: DateTime<Utc>) -> Result<()> {
    let d = zoek(kluis, kenmerk)?;
    let peildatum = nu + Duration::days(dagen);
    kop(&format!("Niet meer aantoonbaar op {}", peildatum.format("%d-%m-%Y")));
    let regels = d.vervalt_voor(peildatum, nu);
    if regels.is_empty() {
        gelukt(&format!("binnen {dagen} dagen vervalt er geen uitvoeringsbewijs"));
    } else {
        let mut t = tabel(&["maatregel", "", "eigenaar", "vervalt op"]);
        for r in &regels {
            t.add_row(vec![
                r.code.clone(),
                r.onderdeel.letter().to_string(),
                r.eigenaar
                    .as_ref()
                    .map(|e| format!("{} ({})", e.rol, e.persoon))
                    .unwrap_or_else(|| "geen".into()),
                r.vervalt_op.format("%d-%m-%Y").to_string(),
            ]);
        }
        println!("{t}");
        terzijde(&format!(
            "{} maatregel(en) vallen terug op vastgesteld, niet aantoonbaar",
            regels.len()
        ));
    }
    terzijde(
        "Dit is een lijst met datums en geen prognose met een cijfer: wat er omvalt en wanneer, \
         zonder een score die suggereert hoe erg dat is.",
    );
    Ok(())
}

fn vaststellen(kluis: &mut Kluis, kenmerk: &str, nu: DateTime<Utc>) -> Result<()> {
    let mut d = zoek(kluis, kenmerk)?;
    let actor = super::actor();
    let id = d.id.to_string();
    match d.stel_vast(&actor.naam, nu) {
        Ok(()) => {
            bewaar(kluis, &d, Handeling::RecordVastgesteld, "zorgplichtdossier vastgesteld", nu)?;
            gelukt(&format!("zorgplichtdossier {kenmerk} vastgesteld"));
            toon_standen(&d, nu);
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

fn lijst(kluis: &Kluis, nu: DateTime<Utc>) -> Result<()> {
    let koppen = kluis.lijst(SOORT)?;
    kop("Zorgplichtdossiers");
    if koppen.is_empty() {
        terzijde("Er staat nog geen zorgplichtdossier in de kluis.");
        return Ok(());
    }
    let mut t = tabel(&["kenmerk", "entiteit", "kader", "maatregelen", "aantoonbaar"]);
    for k in &koppen {
        let d: Zorgplichtdossier = kluis.laad(SOORT, &k.id)?;
        let aantoonbaar =
            d.maatregelen.iter().filter(|m| m.stand(nu) == Maatregelstand::Aantoonbaar).count();
        t.add_row(vec![
            d.kenmerk.clone(),
            d.naam.clone(),
            format!("{} ({})", d.kaderkenmerk, d.variant.letter()),
            d.maatregelen.len().to_string(),
            format!("{aantoonbaar} van de {}", d.maatregelen.len()),
        ]);
    }
    println!("{t}");
    Ok(())
}

fn toon(kluis: &Kluis, kenmerk: &str, onder: Option<&str>, nu: DateTime<Utc>) -> Result<()> {
    let d = zoek(kluis, kenmerk)?;
    let filter = match onder {
        None => None,
        Some(l) => Some(onderdeel_uit_letter(l).ok_or_else(|| {
            anyhow::anyhow!("'{l}' is geen onderdeel; gebruik een letter a tot en met j")
        })?),
    };

    kop(&format!("Zorgplichtdossier {}", d.kenmerk));
    let mut t = tabel(&["", ""]);
    t.add_row(vec!["entiteit", &d.naam]);
    t.add_row(vec!["status", d.status.omschrijving()]);
    t.add_row(vec!["kader", &d.kaderkenmerk]);
    t.add_row(vec!["variant", d.variant.omschrijving()]);
    t.add_row(vec!["kaderversie", &d.kaderversie]);
    t.add_row(vec!["bron", &d.kaderbron]);
    let geverifieerd = d
        .kader_geverifieerd_op
        .map(|x| x.to_string())
        .unwrap_or_else(|| "niet tegen de bron gehouden".into());
    t.add_row(vec!["kader geverifieerd", &geverifieerd]);
    t.add_row(vec!["aangemelde functionaris", &d.aangemelde_functionaris]);
    println!("{t}");

    kop("Maatregelen");
    let mut t = tabel(&[
        "",
        "maatregel",
        "vindplaats",
        "stand",
        "toepassing",
        "eigenaar",
        "bewijs geldig tot",
    ]);
    for m in d.maatregelen.iter().filter(|m| filter.is_none_or(|o| m.onderdeel == o)) {
        let ingetrokken = m.bewijs.iter().filter(|b| b.is_ingetrokken()).count();
        let bewijs = m
            .geldig_uitvoeringsbewijs(nu)
            .map(|b| b.geldig_tot.format("%d-%m-%Y").to_string())
            .unwrap_or_else(|| "—".into());
        let bewijs =
            if ingetrokken > 0 { format!("{bewijs} ({ingetrokken} ingetrokken)") } else { bewijs };
        t.add_row(vec![
            m.onderdeel.letter().to_string(),
            m.code.clone(),
            m.normvindplaats.clone(),
            m.stand(nu).omschrijving().to_string(),
            m.toepassing.omschrijving(),
            m.eigenaar.as_ref().map(|e| e.rol.clone()).unwrap_or_else(|| "geen".into()),
            bewijs,
        ]);
    }
    println!("{t}");
    toon_standen(&d, nu);

    if let Some(k) = &d.risicobeoordeling {
        kop("Risicobeoordeling");
        let mut t = tabel(&["", ""]);
        t.add_row(vec!["kenmerk", &k.kenmerk]);
        let gekoppeld = k.gekoppeld_op.format("%d-%m-%Y").to_string();
        t.add_row(vec!["gekoppeld op", &gekoppeld]);
        println!("{t}");
        terzijde("bekijk de beoordeling met 'dpofg risico toon'");
    }

    if let Some(b) = &d.bestuursvaststelling {
        kop("Bestuursvaststelling");
        let mut t = tabel(&["", ""]);
        let datum = b.datum.format("%d-%m-%Y").to_string();
        t.add_row(vec!["datum", &datum]);
        t.add_row(vec!["kaderversie", &b.goedgekeurde_kaderversie]);
        let aanwezigen = b.aanwezigen.join(", ");
        t.add_row(vec!["aanwezigen", &aanwezigen]);
        println!("{t}");
        println!("  {}", b.besluittekst);
    }

    if let Some(tekst) = vaste_tekst(nu, "bij_elke_uitvoer") {
        terzijde(&tekst);
    }
    toon_ontbrekend(&d);
    Ok(())
}

fn toon_standen(d: &Zorgplichtdossier, nu: DateTime<Utc>) {
    kop("Stand van de maatregelen");
    let mut t = tabel(&["stand", "aantal"]);
    for (stand, aantal) in d.standen(nu) {
        t.add_row(vec![stand.omschrijving().to_string(), aantal.to_string()]);
    }
    println!("{t}");
    let niet_toegepast = d
        .maatregelen
        .iter()
        .filter(|m| matches!(m.toepassing, Toepassing::NietToegepast(_)))
        .count();
    if niet_toegepast > 0 {
        terzijde(&format!(
            "{niet_toegepast} van de {} maatregelen waar het kader afwijken toestaat, worden \
             gemotiveerd niet toegepast",
            d.aantal_afwijkbaar()
        ));
    }
    terzijde(
        "Er staat hier geen score en geen percentage naleving. Aantoonbaar betekent: er ligt \
         een bewijsstuk van de uitvoering waarvan het geldigheidsvenster nu openstaat.",
    );
}

fn toon_ontbrekend(d: &Zorgplichtdossier) {
    let r = d.volledigheid();
    kop("Volledigheid");
    println!("  {}", voortgang(r.compleet, r.verplicht));
    if r.is_volledig() {
        println!();
        gelukt("alle verplichte onderdelen zijn ingevuld");
        return;
    }
    println!();
    // Bij tien onderdelen met elk drie mogelijke gaten wordt de lijst lang;
    // een scherm vol identieke regels leert wegkijken. Daarom per onderdeel
    // een regel en de rest geteld.
    let blokkades = r.blokkades();
    let getoond = blokkades.iter().take(6);
    for o in getoond {
        let veld = o.veld.trim_start_matches("zorgplicht.");
        blokkade(&format!("{veld} — {}", o.omschrijving));
        terzijde(&o.grondslag);
    }
    if blokkades.len() > 6 {
        terzijde(&format!(
            "nog {} blokkerende onderdelen, zie 'zorgplicht toon'",
            blokkades.len() - 6
        ));
    }
    let signalen: Vec<_> = r.ontbreekt.iter().filter(|o| !o.blokkeert_vaststelling).collect();
    if !signalen.is_empty() {
        let_op(&format!(
            "{} maatregel(en) zijn ingericht zonder bewijs van de uitvoering",
            signalen.len()
        ));
        terzijde("art. 5 lid 2 AVG; art. 6 lid 4 Cyberbeveiligingsbesluit");
    }
    println!();
    terzijde("■ houdt vaststellen tegen · ▸ blijft zichtbaar maar blokkeert niet");
}
