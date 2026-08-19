//! De controleregels over de hele verzameling draaien.

use anyhow::Result;
use chrono::{DateTime, Utc};
use clap::Args;
use dpofg_audit::Handeling;
use dpofg_domain::{
    correctie::Correctie, doorgifte::Doorgifte, risico::Risicobeoordeling,
    zorgplicht::Zorgplichtdossier, Dpia, Incident, Leverancier, Verwerking,
};
use dpofg_rules::{
    budget::Waarschuwingsbudget,
    motor::{Niveau, Ontvangerrol},
    regels::{
        beoordeel_budget, beoordeel_correcties, beoordeel_doorgifte, beoordeel_dpia,
        beoordeel_incident, beoordeel_leverancier, beoordeel_logboek, beoordeel_meldtermijn,
        beoordeel_oorzaakpatroon, beoordeel_raadplegingstermijn, beoordeel_risicobeoordeling,
        beoordeel_verwerkersmelding, beoordeel_verwerking, beoordeel_zorgplicht,
        pas_correcties_toe, standaardmotor, Zorgplichtdrempels,
    },
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

    // Termijnen worden hier berekend en niet in de regels: de duren staan in
    // het kennispakket, en een regel die zijn eigen termijn uitrekent gaat een
    // tweede waarheid voeren naast de termijnenmotor.
    let pakket = dpofg_content::startpakket(nu.date_naive());

    let mut bevindingen = Vec::new();
    let mut beoordeeld = 0usize;
    let mut onberekenbaar: Vec<String> = Vec::new();

    for k in kluis.lijst("verwerking")? {
        let v: Verwerking = kluis.laad("verwerking", &k.id)?;
        bevindingen.extend(beoordeel_verwerking(&motor, &v, nu));
        beoordeeld += 1;
    }

    let herbeoordeling = herbeoordelingsdrempel(&pakket);
    for k in kluis.lijst("dpia")? {
        let d: Dpia = kluis.laad("dpia", &k.id)?;
        bevindingen.extend(beoordeel_dpia(&motor, &d, herbeoordeling, nu));
        match raadplegingstermijn_van(&pakket, &d, nu) {
            Ok(Some(deadline)) => {
                bevindingen.extend(beoordeel_raadplegingstermijn(&motor, &d, &deadline, nu));
                beoordeeld += 1;
            }
            Ok(None) => beoordeeld += 1,
            // Niet stilzwijgend overslaan: een termijn die niet te berekenen is,
            // is iets anders dan een termijn die in orde is. Het dossier telt
            // dan ook niet als beoordeeld.
            Err(fout) => onberekenbaar.push(format!("{}: {fout}", d.kenmerk)),
        }
    }

    let drempel = uitzonderingsdrempel(&pakket);
    for k in kluis.lijst("doorgifte")? {
        let d: Doorgifte = kluis.laad("doorgifte", &k.id)?;
        bevindingen.extend(beoordeel_doorgifte(&motor, &d, drempel, nu));
        beoordeeld += 1;
    }

    // De beoordelingen worden eerst geladen: regel ZRP-11 kijkt of de
    // gekoppelde beoordeling bestaat, is vastgesteld en nog geldt, en dat weet
    // alleen het beoordelingsdossier zelf.
    let beoordelingshorizon = beoordelingshorizon(&pakket);
    let mut beoordelingen: Vec<Risicobeoordeling> = Vec::new();
    for k in kluis.lijst("risico")? {
        let b: Risicobeoordeling = kluis.laad("risico", &k.id)?;
        bevindingen.extend(beoordeel_risicobeoordeling(&motor, &b, beoordelingshorizon, nu));
        beoordelingen.push(b);
        beoordeeld += 1;
    }

    let zorgplichtdrempels = zorgplichtdrempels(&pakket);
    for k in kluis.lijst("zorgplicht")? {
        let d: Zorgplichtdossier = kluis.laad("zorgplicht", &k.id)?;
        bevindingen.extend(beoordeel_zorgplicht(
            &motor,
            &d,
            &beoordelingen,
            zorgplichtdrempels,
            nu,
        ));
        beoordeeld += 1;
    }

    // De leveranciers worden eerst geladen: de incidentbeoordeling hieronder
    // heeft ze nodig om na te rekenen of de verwerker binnen zijn contractuele
    // termijn heeft gemeld.
    let meldtermijndrempel = meldtermijndrempel(&pakket);
    let subverwerkersdrempel = subverwerkersdrempel(&pakket);
    let mut leveranciers: Vec<Leverancier> = Vec::new();
    for k in kluis.lijst("leverancier")? {
        let l: Leverancier = kluis.laad("leverancier", &k.id)?;
        bevindingen.extend(beoordeel_leverancier(
            &motor,
            &l,
            meldtermijndrempel,
            subverwerkersdrempel,
            nu,
        ));
        leveranciers.push(l);
        beoordeeld += 1;
    }

    let mut incidenten = Vec::new();
    for k in kluis.lijst("incident")? {
        let i: Incident = kluis.laad("incident", &k.id)?;
        bevindingen.extend(beoordeel_incident(&motor, &i, nu));
        match meldtermijn_van(&pakket, &i) {
            Ok(Some(deadline)) => {
                bevindingen.extend(beoordeel_meldtermijn(&motor, &i, &deadline, nu));
                beoordeeld += 1;
            }
            Ok(None) => beoordeeld += 1,
            Err(fout) => onberekenbaar.push(format!("{}: {fout}", i.kenmerk)),
        }
        if let Some(verwerker_id) = i.verwerker_id {
            if let Some(l) = leveranciers.iter().find(|l| l.id == verwerker_id) {
                bevindingen.extend(beoordeel_verwerkersmelding(&motor, &i, l, nu));
            }
        }
        incidenten.push(i);
    }
    // Het patroon over incidenten heen: drie maanden terug.
    let kwartaalgrens = nu - chrono::Duration::days(92);
    bevindingen.extend(beoordeel_oorzaakpatroon(&motor, &incidenten, nu, kwartaalgrens));

    // Het systeem onder de dossiers: de keten en de klok.
    let verificatie = kluis.verifieer_logboek()?;
    bevindingen.extend(beoordeel_logboek(&motor, &verificatie, kluis.ketenstand().tijdstip, nu));

    // Het waarschuwingsbudget wordt gevoed uit het logboek en niet uit deze
    // ronde: een onderbreking is een moment waarop de gebruiker is
    // tegengehouden, geen regel in een rapport. Zou de rapportagelus zelf
    // tellen, dan overschrijden twee controlerondes op één middag het budget.
    let mut budget = Waarschuwingsbudget::nieuw();
    let weekgrens = nu - chrono::Duration::days(7);
    for regel in kluis.logboek()? {
        let g = &regel.gebeurtenis;
        if g.handeling == Handeling::ControleGeblokkeerd && g.tijdstip > weekgrens {
            budget.onderbreking(&g.actor.id, g.tijdstip);
        }
    }
    bevindingen.extend(beoordeel_budget(&motor, &budget, nu));

    // De correcties komen ná alle beoordelingen: zij gaan over de bevindingen
    // van deze ronde en niet over een dossier. Eerst wordt de correctieplicht
    // getoetst tegen het onbewerkte beeld — anders zou een lopende afwijking
    // de bevinding waarover zij gaat uit het zicht halen en zou COR-03 elke
    // afwijking als overbodig melden. Daarna pas worden de afwijkingen over de
    // bevindingen heen gelegd.
    let mut correcties: Vec<Correctie> = Vec::new();
    for k in kluis.lijst("correctie")? {
        correcties.push(kluis.laad("correctie", &k.id)?);
    }
    if !correcties.is_empty() || bevindingen.iter().any(|b| b.niveau == Niveau::Blokkerend) {
        let over_correcties =
            beoordeel_correcties(&motor, &correcties, &bevindingen, afwijkingsaandeel(&pakket), nu);
        pas_correcties_toe(&mut bevindingen, &correcties, nu);
        bevindingen.extend(over_correcties);
        beoordeeld += correcties.len();
    }

    let drempel: Niveau = o.vanaf.into();
    bevindingen.retain(|b| b.niveau >= drempel);
    if let Some(rol) = o.voor {
        let rol: Ontvangerrol = rol.into();
        bevindingen.retain(|b| b.ontvanger == rol);
    }

    let rapport = motor.rapporteer(bevindingen, beoordeeld, nu);

    if !onberekenbaar.is_empty() {
        kop("Niet beoordeeld");
        for regel in &onberekenbaar {
            blokkade(regel);
        }
        terzijde(
            "Deze dossiers dragen een termijn die met het huidige kennispakket niet te \
             berekenen is. Zij tellen niet mee in de ronde hieronder; zwijgen zou hier \
             betekenen dat een onberekenbare termijn als in orde geldt.",
        );
    }

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
            let omschrijving = motor.regel(code).map(|x| x.controleert.clone()).unwrap_or_default();
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

/// De meldtermijn van één incident, of `None` wanneer de klok nog niet loopt.
///
/// Het anker is het moment van kennisname; zolang dat er niet is, is er niets
/// te rekenen en dus niets waaraan te herinneren valt.
fn meldtermijn_van(
    pakket: &dpofg_content::Pakketinhoud,
    i: &Incident,
) -> Result<Option<dpofg_terms::Deadline>> {
    // `Ok(None)`: de klok loopt nog niet, er valt niets te rekenen. Een `Err`
    // betekent dat er wél een termijn is maar dat hij niet te berekenen is, en
    // dat is geen "in orde" — zie `raadplegingstermijn_van`.
    let Some(anker) = i.anker_meldklok() else { return Ok(None) };
    let soort = pakket.termijn("AVG-33-MELDING")?;
    let kalender = pakket.kalender("NL")?;
    let zone = dpofg_terms::tijdzone(dpofg_terms::TIJDZONE_NL)?;
    Ok(Some(dpofg_terms::bereken(soort, anker, zone, kalender)?))
}

/// De lopende raadplegingstermijn van één effectbeoordeling.
///
/// `Ok(None)` betekent: er loopt geen raadpleging, dus er valt niets te
/// beoordelen. Een `Err` betekent iets anders — de termijn bestaat maar is niet
/// te berekenen — en dat mag niet als "in orde" worden gelezen. Vandaar geen
/// `.ok()` op de berekening zelf.
fn raadplegingstermijn_van(
    pakket: &dpofg_content::Pakketinhoud,
    d: &Dpia,
    nu: DateTime<Utc>,
) -> Result<Option<dpofg_terms::Deadline>> {
    let Some(klok) = d.raadpleging.as_ref() else { return Ok(None) };
    let kalender = pakket.kalender("NL")?;
    let zone = dpofg_terms::tijdzone(dpofg_terms::TIJDZONE_NL)?;
    // Het peilmoment komt van buiten: de hele opdracht rekent met één `nu`,
    // zodat twee regels in dezelfde ronde nooit op verschillende klokken
    // berusten.
    Ok(Some(klok.deadline_volledig(nu, zone, kalender)?))
}

/// Na hoeveel maanden een effectbeoordeling om herbeoordeling vraagt.
///
/// Uit het kennispakket, zodat de norm bij te stellen is zonder de
/// programmacode te raken.
fn herbeoordelingsdrempel(pakket: &dpofg_content::Pakketinhoud) -> i64 {
    pakket
        .termijn("INTERN-DPIA-HERBEOORDELING")
        .ok()
        .filter(|t| t.eenheid == dpofg_terms::Eenheid::Maanden)
        .map(|t| i64::from(t.duur))
        .unwrap_or(36)
}

/// Boven hoeveel toepassingen per jaar een uitzondering van artikel 49 niet
/// meer incidenteel is.
///
/// Uit het kennispakket: de verordening noemt geen getal, dus het hoort op een
/// plaats te staan waar een jurist het kan bijstellen.
fn uitzonderingsdrempel(pakket: &dpofg_content::Pakketinhoud) -> u32 {
    pakket
        .aanvullend
        .get("doorgifte_uitzonderingsdrempel")
        .and_then(|v| v.get("drempel"))
        .and_then(|v| v.as_u64())
        .and_then(|v| u32::try_from(v).ok())
        .unwrap_or(2)
}

/// Boven hoeveel uur de contractuele meldtermijn van een verwerker te lang is.
fn meldtermijndrempel(pakket: &dpofg_content::Pakketinhoud) -> u32 {
    pakket
        .aanvullend
        .get("verwerker_meldtermijndrempel")
        .and_then(|v| v.get("drempel_uren"))
        .and_then(|v| v.as_u64())
        .and_then(|v| u32::try_from(v).ok())
        .unwrap_or(48)
}

/// Boven welk aandeel afwijkingen niet-herstel gewoonte wordt.
fn afwijkingsaandeel(pakket: &dpofg_content::Pakketinhoud) -> u32 {
    pakket
        .aanvullend
        .get("zorgplicht_drempels")
        .and_then(|v| v.get("afwijkingsaandeel_procent"))
        .and_then(|v| v.as_u64())
        .and_then(|v| u32::try_from(v).ok())
        .unwrap_or(50)
}

/// Hoeveel dagen vooruit een verlopende risicobeoordeling wordt gemeld.
fn beoordelingshorizon(pakket: &dpofg_content::Pakketinhoud) -> i64 {
    pakket
        .termijn("INTERN-RISICOBEOORDELING-HORIZON")
        .ok()
        .filter(|t| t.eenheid == dpofg_terms::Eenheid::Kalenderdagen)
        .map(|t| i64::from(t.duur))
        .unwrap_or(60)
}

/// De vijf drempels waartegen de zorgplichtcontrolset wordt gemeten.
///
/// Geen ervan staat in de wet. De termijnen komen uit de termijnencatalogus,
/// de twee kale getallen uit de aanvullende inhoud; de terugvalwaarden staan
/// hier zodat een uitgekleed pakket de controle niet stilzwijgend uitzet.
fn zorgplichtdrempels(pakket: &dpofg_content::Pakketinhoud) -> Zorgplichtdrempels {
    let dagen = |code: &str, terugval: i64| {
        pakket
            .termijn(code)
            .ok()
            .filter(|t| t.eenheid == dpofg_terms::Eenheid::Kalenderdagen)
            .map(|t| i64::from(t.duur))
            .unwrap_or(terugval)
    };
    let getal = |sleutel: &str, terugval: u32| {
        pakket
            .aanvullend
            .get("zorgplicht_drempels")
            .and_then(|v| v.get(sleutel))
            .and_then(|v| v.as_u64())
            .and_then(|v| u32::try_from(v).ok())
            .unwrap_or(terugval)
    };
    Zorgplichtdrempels {
        beoordelingstermijn_dagen: dagen("INTERN-ZORGPLICHT-BEOORDELINGSTERMIJN", 30),
        bewijshorizon_dagen: dagen("INTERN-ZORGPLICHT-BEWIJSHORIZON", 60),
        frequentiedrempel_maanden: getal("frequentiedrempel_maanden", 12),
        bestuursvaststelling_maanden: pakket
            .termijn("INTERN-ZORGPLICHT-BESTUURSVASTSTELLING")
            .ok()
            .filter(|t| t.eenheid == dpofg_terms::Eenheid::Maanden)
            .map(|t| i64::from(t.duur))
            .unwrap_or(12),
        afwijkingsaandeel_procent: getal("afwijkingsaandeel_procent", 20),
    }
}

/// Na hoeveel maanden de subverwerkerslijst opnieuw moet worden nagelopen.
fn subverwerkersdrempel(pakket: &dpofg_content::Pakketinhoud) -> i64 {
    pakket
        .termijn("INTERN-SUBVERWERKERSCONTROLE")
        .ok()
        .filter(|t| t.eenheid == dpofg_terms::Eenheid::Maanden)
        .map(|t| i64::from(t.duur))
        .unwrap_or(12)
}
