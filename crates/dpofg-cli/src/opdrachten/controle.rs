//! De controleregels over de hele verzameling draaien.

use anyhow::Result;
use chrono::{DateTime, Utc};
use clap::Args;
use dpofg_domain::{Incident, Verwerking};
use dpofg_rules::{
    motor::{Niveau, Ontvangerrol},
    regels::{beoordeel_incident, beoordeel_oorzaakpatroon, beoordeel_verwerking, standaardmotor},
};
use std::path::PathBuf;

use crate::uitvoer::{blokkade, gelukt, kop, let_op, tabel, terzijde};

#[derive(Args, Debug)]
pub struct Controleopties {
    /// Toon alleen bevindingen voor deze rol.
    #[arg(long, value_enum)]
    pub voor: Option<Rolkeuze>,
    /// Toon alleen bevindingen op dit niveau of hoger.
    #[arg(long, value_enum, default_value = "signalerend")]
    pub vanaf: Niveaukeuze,
    /// Toon welke regels in de catalogus nog geen evaluatie hebben.
    #[arg(long)]
    pub dekking: bool,
}

#[derive(clap::ValueEnum, Clone, Copy, Debug)]
pub enum Rolkeuze {
    Functionaris,
    Behandelaar,
    Contracteigenaar,
    Systeemeigenaar,
    SecurityOfficer,
    Directie,
    Beheerder,
}

impl From<Rolkeuze> for Ontvangerrol {
    fn from(k: Rolkeuze) -> Self {
        match k {
            Rolkeuze::Functionaris => Ontvangerrol::Functionaris,
            Rolkeuze::Behandelaar => Ontvangerrol::Behandelaar,
            Rolkeuze::Contracteigenaar => Ontvangerrol::Contracteigenaar,
            Rolkeuze::Systeemeigenaar => Ontvangerrol::Systeemeigenaar,
            Rolkeuze::SecurityOfficer => Ontvangerrol::SecurityOfficer,
            Rolkeuze::Directie => Ontvangerrol::Directie,
            Rolkeuze::Beheerder => Ontvangerrol::Beheerder,
        }
    }
}

#[derive(clap::ValueEnum, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Niveaukeuze {
    Rapporterend,
    Signalerend,
    Blokkerend,
}

impl From<Niveaukeuze> for Niveau {
    fn from(k: Niveaukeuze) -> Self {
        match k {
            Niveaukeuze::Rapporterend => Niveau::Rapporterend,
            Niveaukeuze::Signalerend => Niveau::Signalerend,
            Niveaukeuze::Blokkerend => Niveau::Blokkerend,
        }
    }
}

pub fn draai(o: Controleopties, kluispad: Option<PathBuf>, nu: DateTime<Utc>) -> Result<()> {
    let motor = standaardmotor();

    if o.dekking {
        return toon_dekking(&motor);
    }

    let pad = super::kluispad(kluispad)?;
    let kluis = super::open_kluis(&pad, nu)?;

    let mut bevindingen = Vec::new();
    let mut beoordeeld = 0usize;

    for k in kluis.lijst("verwerking")? {
        let v: Verwerking = kluis.laad("verwerking", &k.id)?;
        bevindingen.extend(beoordeel_verwerking(&motor, &v, nu));
        beoordeeld += 1;
    }

    let mut incidenten = Vec::new();
    for k in kluis.lijst("incident")? {
        let i: Incident = kluis.laad("incident", &k.id)?;
        bevindingen.extend(beoordeel_incident(&motor, &i, nu));
        incidenten.push(i);
        beoordeeld += 1;
    }
    // Het patroon over incidenten heen: drie maanden terug.
    let kwartaalgrens = nu - chrono::Duration::days(92);
    bevindingen.extend(beoordeel_oorzaakpatroon(&motor, &incidenten, nu, kwartaalgrens));

    let drempel: Niveau = o.vanaf.into();
    bevindingen.retain(|b| b.niveau >= drempel);
    if let Some(rol) = o.voor {
        let rol: Ontvangerrol = rol.into();
        bevindingen.retain(|b| b.ontvanger == rol);
    }

    let rapport = motor.rapporteer(bevindingen, beoordeeld, nu);

    kop("Controleronde");
    terzijde(&format!(
        "{} regels gedraaid over {} dossiers op {}",
        rapport.regels_gedraaid,
        rapport.records_beoordeeld,
        nu.format("%d-%m-%Y %H:%M")
    ));

    if rapport.bevindingen.is_empty() {
        println!();
        gelukt("geen bevindingen op dit niveau");
        return Ok(());
    }

    // Per ontvanger, want dat is hoe het werk wordt verdeeld.
    for rol in [
        Ontvangerrol::Functionaris,
        Ontvangerrol::Behandelaar,
        Ontvangerrol::Contracteigenaar,
        Ontvangerrol::Systeemeigenaar,
        Ontvangerrol::SecurityOfficer,
        Ontvangerrol::Directie,
        Ontvangerrol::Beheerder,
    ] {
        let voor_rol = rapport.voor(rol);
        if voor_rol.is_empty() {
            continue;
        }
        kop(&format!("Voor de {}", rol.omschrijving()));
        for b in voor_rol {
            let regel = format!(
                "[{}] {} — {}",
                b.regelcode,
                b.record_kenmerk.as_deref().unwrap_or(&b.record_id),
                b.toelichting
            );
            match b.niveau {
                Niveau::Blokkerend => blokkade(&regel),
                Niveau::Signalerend => let_op(&regel),
                Niveau::Rapporterend => println!("  {regel}"),
            }
            terzijde(&b.grondslag);
        }
    }

    kop("Samenvatting");
    let mut t = tabel(&["niveau", "aantal"]);
    for n in [Niveau::Blokkerend, Niveau::Signalerend, Niveau::Rapporterend] {
        let aantal = rapport.op_niveau(n).len();
        if aantal > 0 {
            t.add_row(vec![n.omschrijving().to_string(), aantal.to_string()]);
        }
    }
    println!("{t}");

    let per_regel = rapport.per_regel();
    if per_regel.len() > 1 {
        kop("Meest voorkomend");
        let mut r = tabel(&["regel", "aantal", "wat de regel controleert"]);
        for (code, aantal) in per_regel.iter().take(5) {
            let omschrijving =
                motor.regel(code).map(|x| x.controleert.clone()).unwrap_or_default();
            r.add_row(vec![code.clone(), aantal.to_string(), omschrijving]);
        }
        println!("{r}");
        terzijde("Begin bovenaan: daar levert één ingreep de meeste winst op.");
    }

    Ok(())
}

fn toon_dekking(motor: &dpofg_rules::Regelmotor) -> Result<()> {
    kop("Dekking van de regelcatalogus");
    println!(
        "  {} van de {} regels heeft een evaluatiefunctie ({:.0}%)",
        (motor.dekking() * motor.aantal() as f64).round() as usize,
        motor.aantal(),
        motor.dekking() * 100.0
    );
    println!();
    terzijde(
        "Het aantal regels in de catalogus zegt niets over wat er werkelijk wordt bewaakt. \
         Daarom staat hieronder wat er nog niet draait.",
    );

    kop("Nog zonder evaluatie");
    let mut t = tabel(&["regel", "groep", "wat hij zou controleren"]);
    for r in motor.regels_zonder_evaluatie() {
        t.add_row(vec![r.code.clone(), r.groep.clone(), r.controleert.clone()]);
    }
    println!("{t}");
    Ok(())
}
