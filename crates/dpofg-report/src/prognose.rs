//! De vervalprognose: welke eisen op een gekozen datum niet meer aantoonbaar
//! zijn, en waarom.
//!
//! # Waarom dit geen takenlijst is
//!
//! Een takenlijst zegt wat er moet gebeuren. Een vervalprognose zegt wat er
//! ophoudt te kloppen als er niets gebeurt, met de datum erbij. Dat is de
//! vorm waarin een bestuur een informatiebeveiligingsrisico kan wegen: niet
//! als kleur en niet als cijfer, maar als een datum waarop iets niet meer te
//! bewijzen is.
//!
//! # Waarom hier geen score staat
//!
//! Het plan noemt bij deze module een driefactorscore met drie veldnamen —
//! vaststelling, uitvoering en actualiteit — en geeft daar geen schaal,
//! geen weging en geen aggregatieregel bij. Een score bouwen zonder die drie
//! levert een getal op dat niet zegt waarop het is gebaseerd, en dat in een
//! bestuursstuk een eigen leven gaat leiden.
//!
//! Wat er wél is: dezelfde drie factoren als drie afzonderlijke tellingen per
//! eis. Vastgesteld, uitgevoerd en actueel zijn per eis met ja of nee te
//! beantwoorden, en "van de vijftien eisen zijn er twaalf vastgesteld, acht
//! uitgevoerd en zes actueel" draagt alle informatie die een gewogen getal
//! zou dragen, zonder de weging te verzinnen.
//!
//! # Wat er niet in zit
//!
//! Doorgifte-instrumenten kennen in dit model geen einddatum maar een status
//! die tegen het kennispakket wordt gecontroleerd; daar valt dus geen datum
//! uit af te leiden. Certificaten van leveranciers, mandaten en
//! mappingreviews bestaan nog niet als record. Die bronnen ontbreken hier, en
//! dat staat ook in de uitvoer: een prognose die zwijgt over wat zij niet
//! overziet, is een prognose die te weinig meldt.

use chrono::{DateTime, Datelike, TimeZone, Timelike, Utc};
use dpofg_domain::{
    dpia::Dpia,
    leverancier::Leverancier,
    risico::Risicobeoordeling,
    wpg::Wpgspoor,
    zorgplicht::{Bewijsrol, Toepassing, Zorgplichtdossier},
};
use serde::{Deserialize, Serialize};

/// Waardoor een eis op de peildatum niet meer aantoonbaar is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Vervaloorzaak {
    /// Het geldigheidsvenster van een bewijsstuk sluit.
    Bewijsstuk,
    /// De zelf vastgestelde uitvoeringstermijn verstrijkt.
    Frequentie,
    /// De bestuursvaststelling van het maatregelenpakket veroudert.
    Bestuursvaststelling,
    /// De risicobeoordeling verloopt.
    Risicobeoordeling,
    /// De subverwerkerslijst moet opnieuw worden nagelopen.
    Subverwerkerscontrole,
    /// De effectbeoordeling vraagt herbeoordeling.
    Effectbeoordeling,
    /// De externe audit onder de Wet politiegegevens verstrijkt.
    WpgAudit,
    /// De interne controle onder de Wet politiegegevens verstrijkt.
    WpgControle,
}

impl Vervaloorzaak {
    pub fn omschrijving(&self) -> &'static str {
        match self {
            Self::Bewijsstuk => "het bewijsstuk verloopt",
            Self::Frequentie => "de zelf vastgestelde uitvoeringstermijn verstrijkt",
            Self::Bestuursvaststelling => "de bestuursvaststelling veroudert",
            Self::Risicobeoordeling => "de risicobeoordeling verloopt",
            Self::Subverwerkerscontrole => "de subverwerkerslijst is dan te lang niet nagelopen",
            Self::Effectbeoordeling => "de effectbeoordeling vraagt herbeoordeling",
            Self::WpgAudit => "de externe audit verstrijkt",
            Self::WpgControle => "de interne controle verstrijkt",
        }
    }
}

/// Eén eis die op de peildatum niet meer aantoonbaar is.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Vervalpunt {
    /// Wat er niet meer te bewijzen is, in gewone taal.
    pub eis: String,
    pub grondslag: String,
    pub oorzaak: Vervaloorzaak,
    pub record_soort: String,
    pub record_kenmerk: String,
    /// Het onderdeel binnen het record, bijvoorbeeld een maatregelcode.
    pub onderdeel: Option<String>,
    pub eigenaar: Option<String>,
    pub vervalt_op: DateTime<Utc>,
}

impl Vervalpunt {
    pub fn dagen_tot_verval(&self, nu: DateTime<Utc>) -> i64 {
        (self.vervalt_op - nu).num_days()
    }

    /// Of het punt op de peildatum al is verstreken.
    pub fn is_verstreken(&self, nu: DateTime<Utc>) -> bool {
        self.vervalt_op <= nu
    }
}

/// De termijnen waarmee de prognose rekent.
///
/// Alle vijf komen uit het kennispakket. Een prognose die haar eigen
/// termijnen verzint, voert een tweede waarheid naast de termijnencatalogus.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Prognosetermijnen {
    pub bestuursvaststelling_maanden: u32,
    pub subverwerkerscontrole_maanden: u32,
    pub effectbeoordeling_maanden: u32,
    pub wpg_audit_maanden: u32,
    pub wpg_controle_maanden: u32,
}

/// De dossiers waaruit de prognose wordt opgebouwd.
#[derive(Debug, Default)]
pub struct Bronnen<'a> {
    pub zorgplicht: &'a [Zorgplichtdossier],
    pub risicobeoordelingen: &'a [Risicobeoordeling],
    pub leveranciers: &'a [Leverancier],
    pub effectbeoordelingen: &'a [Dpia],
    pub wpgsporen: &'a [Wpgspoor],
}

/// Telt een aantal maanden bij een tijdstip op, met behoud van het tijdstip.
///
/// Gebruikt de kalenderrekenaar uit de termijnenmodule en niet dertig dagen
/// per maand: een prognose die de maand afrondt, verschuift bij elke stap en
/// wijst uiteindelijk de verkeerde week aan.
fn maanden_later(moment: DateTime<Utc>, maanden: u32) -> Option<DateTime<Utc>> {
    let datum = dpofg_terms::kalenderrekenen::tel_maanden_op(moment.date_naive(), maanden).ok()?;
    Utc.with_ymd_and_hms(
        datum.year(),
        datum.month(),
        datum.day(),
        moment.hour(),
        moment.minute(),
        moment.second(),
    )
    .single()
}

/// Stelt de prognose samen voor alles wat op of vóór de peildatum vervalt.
///
/// Punten die op het peilmoment `nu` al verstreken zijn, blijven staan: dat
/// zijn de eisen die vandaag al niet aantoonbaar zijn, en die weglaten uit een
/// prognose zou het beeld verbeteren zonder dat er iets is opgelost.
pub fn prognose(
    bronnen: &Bronnen<'_>,
    termijnen: Prognosetermijnen,
    peildatum: DateTime<Utc>,
) -> Vec<Vervalpunt> {
    let mut uit = Vec::new();
    verzamel_zorgplicht(bronnen.zorgplicht, termijnen, peildatum, &mut uit);
    verzamel_risicobeoordelingen(bronnen.risicobeoordelingen, peildatum, &mut uit);
    verzamel_leveranciers(bronnen.leveranciers, termijnen, peildatum, &mut uit);
    verzamel_effectbeoordelingen(bronnen.effectbeoordelingen, termijnen, peildatum, &mut uit);
    verzamel_wpg(bronnen.wpgsporen, termijnen, peildatum, &mut uit);
    uit.sort_by_key(|v| v.vervalt_op);
    uit
}

fn verzamel_zorgplicht(
    dossiers: &[Zorgplichtdossier],
    termijnen: Prognosetermijnen,
    peildatum: DateTime<Utc>,
    uit: &mut Vec<Vervalpunt>,
) {
    for d in dossiers {
        for m in &d.maatregelen {
            if !matches!(m.toepassing, Toepassing::Ingericht) {
                continue;
            }
            let eigenaar = m.eigenaar.as_ref().map(|e| format!("{} ({})", e.rol, e.persoon));

            // Het bewijsstuk dat de uitvoering onderbouwt.
            if let Some(b) = m
                .bewijs
                .iter()
                .filter(|b| b.rol == Bewijsrol::Uitvoering && !b.is_ingetrokken())
                .max_by_key(|b| b.geldig_tot)
            {
                if b.geldig_tot <= peildatum {
                    uit.push(Vervalpunt {
                        eis: format!("{}: {}", m.code, m.omschrijving),
                        grondslag: m.onderdeel.grondslag(),
                        oorzaak: Vervaloorzaak::Bewijsstuk,
                        record_soort: "zorgplicht".into(),
                        record_kenmerk: d.kenmerk.clone(),
                        onderdeel: Some(m.code.clone()),
                        eigenaar: eigenaar.clone(),
                        vervalt_op: b.geldig_tot,
                    });
                }
            }

            // De zelf vastgestelde uitvoeringstermijn.
            if let (Some(f), Some(laatste)) = (
                m.frequentie.as_ref(),
                m.bewijs
                    .iter()
                    .filter(|b| b.rol == Bewijsrol::Uitvoering && !b.is_ingetrokken())
                    .map(|b| b.geldig_van)
                    .max(),
            ) {
                if let Some(volgende) = maanden_later(laatste, f.maanden) {
                    if volgende <= peildatum {
                        uit.push(Vervalpunt {
                            eis: format!("{}: uitvoering elke {} maanden", m.code, f.maanden),
                            grondslag: "zelf vastgestelde termijn".into(),
                            oorzaak: Vervaloorzaak::Frequentie,
                            record_soort: "zorgplicht".into(),
                            record_kenmerk: d.kenmerk.clone(),
                            onderdeel: Some(m.code.clone()),
                            eigenaar: eigenaar.clone(),
                            vervalt_op: volgende,
                        });
                    }
                }
            }
        }

        if let Some(b) = &d.bestuursvaststelling {
            if let Some(volgende) = maanden_later(b.datum, termijnen.bestuursvaststelling_maanden) {
                if volgende <= peildatum {
                    uit.push(Vervalpunt {
                        eis: format!("{}: het bestuur stelt het maatregelenpakket vast", d.kenmerk),
                        grondslag: "art. 24 lid 1 Cyberbeveiligingswet".into(),
                        oorzaak: Vervaloorzaak::Bestuursvaststelling,
                        record_soort: "zorgplicht".into(),
                        record_kenmerk: d.kenmerk.clone(),
                        onderdeel: None,
                        eigenaar: Some("het bestuur".into()),
                        vervalt_op: volgende,
                    });
                }
            }
        }
    }
}

fn verzamel_risicobeoordelingen(
    beoordelingen: &[Risicobeoordeling],
    peildatum: DateTime<Utc>,
    uit: &mut Vec<Vervalpunt>,
) {
    for b in beoordelingen {
        if b.geldig_tot <= peildatum {
            uit.push(Vervalpunt {
                eis: format!("{}: een geldige risicobeoordeling over {}", b.kenmerk, b.reikwijdte),
                grondslag: "art. 21 lid 1 Cyberbeveiligingswet".into(),
                oorzaak: Vervaloorzaak::Risicobeoordeling,
                record_soort: "risico".into(),
                record_kenmerk: b.kenmerk.clone(),
                onderdeel: None,
                eigenaar: Some(b.uitgevoerd_door.clone()),
                vervalt_op: b.geldig_tot,
            });
        }
    }
}

fn verzamel_leveranciers(
    leveranciers: &[Leverancier],
    termijnen: Prognosetermijnen,
    peildatum: DateTime<Utc>,
    uit: &mut Vec<Vervalpunt>,
) {
    for l in leveranciers {
        let Some(gecontroleerd) = l.subverwerkers_gecontroleerd_op else {
            continue;
        };
        let Some(volgende) = maanden_later(gecontroleerd, termijnen.subverwerkerscontrole_maanden)
        else {
            continue;
        };
        if volgende <= peildatum {
            uit.push(Vervalpunt {
                eis: format!("{}: een nagelopen subverwerkerslijst", l.naam),
                grondslag: "art. 28 lid 2 en lid 4 AVG".into(),
                oorzaak: Vervaloorzaak::Subverwerkerscontrole,
                record_soort: "leverancier".into(),
                record_kenmerk: l.kenmerk.clone(),
                onderdeel: None,
                eigenaar: None,
                vervalt_op: volgende,
            });
        }
    }
}

fn verzamel_effectbeoordelingen(
    beoordelingen: &[Dpia],
    termijnen: Prognosetermijnen,
    peildatum: DateTime<Utc>,
    uit: &mut Vec<Vervalpunt>,
) {
    for d in beoordelingen {
        let Some(datum) = d.datum else {
            continue;
        };
        let Some(volgende) = maanden_later(datum, termijnen.effectbeoordeling_maanden) else {
            continue;
        };
        if volgende <= peildatum {
            uit.push(Vervalpunt {
                eis: format!("{}: een actuele effectbeoordeling", d.kenmerk),
                grondslag: "art. 35 lid 11 AVG".into(),
                oorzaak: Vervaloorzaak::Effectbeoordeling,
                record_soort: "dpia".into(),
                record_kenmerk: d.kenmerk.clone(),
                onderdeel: None,
                eigenaar: d.uitgevoerd_door.clone(),
                vervalt_op: volgende,
            });
        }
    }
}

fn verzamel_wpg(
    sporen: &[Wpgspoor],
    termijnen: Prognosetermijnen,
    peildatum: DateTime<Utc>,
    uit: &mut Vec<Vervalpunt>,
) {
    for s in sporen {
        for (controle, maanden, oorzaak, eis, grondslag) in [
            (
                s.laatste_audit(),
                termijnen.wpg_audit_maanden,
                Vervaloorzaak::WpgAudit,
                "een externe audit",
                "art. 33 lid 3 Wet politiegegevens",
            ),
            (
                s.laatste_controle(),
                termijnen.wpg_controle_maanden,
                Vervaloorzaak::WpgControle,
                "een interne controle",
                "art. 33 lid 1 Wet politiegegevens",
            ),
        ] {
            let Some(c) = controle else {
                continue;
            };
            let Some(volgende) = maanden_later(c.uitgevoerd_op, maanden) else {
                continue;
            };
            if volgende <= peildatum {
                uit.push(Vervalpunt {
                    eis: format!("{}: {eis}", s.kenmerk),
                    grondslag: grondslag.into(),
                    oorzaak,
                    record_soort: "wpg".into(),
                    record_kenmerk: s.kenmerk.clone(),
                    onderdeel: None,
                    eigenaar: Some(c.uitvoerder.clone()),
                    vervalt_op: volgende,
                });
            }
        }
    }
}

/// Wat er in het bestuursstuk hoort te staan naast de prognose zelf.
///
/// Een prognose zonder deze context is een lijst datums zonder herkomst. Het
/// bestuur moet kunnen zien waarop zij berust: welke peildatum, welke versie
/// van de juridische inhoud, of het ketenlogboek klopt, en hoeveel van de
/// dossiers eronder nog concept zijn.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rapportcontext {
    pub peilmoment: DateTime<Utc>,
    pub samengesteld_door: String,
    pub bestemd_voor: String,
    pub kennispakket: String,
    pub consolidatiedatum: String,
    /// De reikwijdte van de ketenverificatie, letterlijk uit het rapport.
    pub ketenreikwijdte: String,
    /// Of de ketencontrole zonder bevindingen is doorlopen.
    pub keten_in_orde: bool,
    /// Hoeveel bronrecords nog de status concept dragen.
    pub concepten: usize,
    /// Hoeveel correcties er op dit moment lopen.
    pub lopende_correcties: usize,
}

/// Wat deze prognose niet overziet.
///
/// Staat in het bestuursstuk zelf en niet alleen in het manifest: wie het
/// stuk leest, hoort de grens te zien zonder een tweede bestand te openen.
pub const BUITEN_BEELD: [(&str, &str); 4] = [
    (
        "doorgifte-instrumenten",
        "die kennen in dit model geen einddatum maar een status die tegen het kennispakket \
         wordt gehouden; daar valt geen datum uit af te leiden",
    ),
    ("certificaten van leveranciers", "die bestaan nog niet als eigen record"),
    ("mandaten en volmachten", "die bestaan nog niet als eigen record"),
    ("reviews van de crosswalk naar normenkaders", "de crosswalk zelf bestaat nog niet"),
];

/// Stelt het bestuursstuk samen als leesbare tekst.
///
/// Geen kleur en geen cijfer. Wat een bestuur nodig heeft om te besluiten is
/// welke eis wanneer onbewijsbaar wordt en wie eraan werkt; de rest is
/// decoratie die de aandacht wegneemt van de datum.
pub fn bestuursstuk(
    context: &Rapportcontext,
    horizonnen: &[(i64, Vec<Vervalpunt>)],
    verstreken: &[Vervalpunt],
    factoren: &[Aantoonbaarheid],
) -> String {
    let mut uit = String::new();
    uit.push_str("# Vervalprognose\n\n");
    uit.push_str(&format!(
        "Peilmoment: {}\nSamengesteld door: {}\nBestemd voor: {}\n\n",
        context.peilmoment.format("%d-%m-%Y %H:%M UTC"),
        context.samengesteld_door,
        context.bestemd_voor
    ));
    uit.push_str(
        "Dit stuk noemt geen cijfer en geen kleur. Het noemt welke eisen op welke datum niet \
         meer met bewijs zijn te onderbouwen, en wie daarvoor is aangewezen.\n\n",
    );

    uit.push_str("## Waarop dit stuk berust\n\n");
    uit.push_str(&format!(
        "- Juridische inhoud: {} (bijgewerkt tot {})\n",
        context.kennispakket, context.consolidatiedatum
    ));
    uit.push_str(&format!("- Ketencontrole: {}\n", context.ketenreikwijdte));
    if !context.keten_in_orde {
        uit.push_str(
            "- **De ketencontrole is niet zonder bevindingen doorlopen.** Zolang dat zo is, \
             staat niet vast dat de gegevens onder deze prognose ongewijzigd zijn.\n",
        );
    }
    if context.concepten > 0 {
        uit.push_str(&format!(
            "- {} {} nog de status concept; wat daarin staat is niet vastgesteld en de \
             prognose erover zegt navenant minder\n",
            context.concepten,
            if context.concepten == 1 { "bronrecord draagt" } else { "bronrecords dragen" }
        ));
    }
    uit.push_str(&format!(
        "- {} {} vastgelegde correctie(s); wat daarvan is afgesproken staat in de bundel\n\n",
        if context.lopende_correcties == 1 { "Er loopt" } else { "Er lopen" },
        context.lopende_correcties
    ));

    if verstreken.is_empty() {
        uit.push_str("## Vandaag\n\nEr is op dit moment geen eis die niet te bewijzen is.\n\n");
    } else {
        uit.push_str(&format!("## Vandaag al niet aantoonbaar — {} eis(en)\n\n", verstreken.len()));
        uit.push_str(
            "Dit is geen prognose maar een stand van zaken. Deze eisen zijn nu niet te \
             bewijzen.\n\n",
        );
        uit.push_str(&tabel_van(verstreken, context.peilmoment, true));
    }

    for (dagen, punten) in horizonnen {
        let peildatum = context.peilmoment + chrono::Duration::days(*dagen);
        uit.push_str(&format!("## Over {dagen} dagen — {}\n\n", peildatum.format("%d-%m-%Y")));
        let nieuw: Vec<&Vervalpunt> =
            punten.iter().filter(|v| !v.is_verstreken(context.peilmoment)).collect();
        if nieuw.is_empty() {
            uit.push_str("Er verloopt niets binnen deze horizon.\n\n");
            continue;
        }
        uit.push_str(&format!(
            "{} eis(en) worden binnen deze horizon onbewijsbaar.\n\n",
            nieuw.len()
        ));
        let losse: Vec<Vervalpunt> = nieuw.into_iter().cloned().collect();
        uit.push_str(&tabel_van(&losse, context.peilmoment, false));
    }

    uit.push_str("## De drie factoren\n\n");
    if factoren.is_empty() {
        uit.push_str("Er is nog geen ingerichte maatregel om te wegen.\n\n");
    } else {
        let telling = Factortelling::van(factoren);
        uit.push_str(&format!(
            "| factor | wat de vraag is | aantal |\n|---|---|---|\n\
             | vastgesteld | ligt er een besluit of beleidsstuk dat de eis vastlegt | {} van de {} |\n\
             | uitgevoerd | ligt er bewijs van de uitvoering dat nu geldt | {} van de {} |\n\
             | actueel | is die uitvoering recent genoeg voor de eigen termijn | {} van de {} |\n\n",
            telling.vastgesteld,
            telling.totaal,
            telling.uitgevoerd,
            telling.totaal,
            telling.actueel,
            telling.totaal
        ));
        uit.push_str(
            "Drie tellingen en geen score. Een gewogen getal zou een schaal en een weging \
             veronderstellen die nergens is vastgesteld.\n\n",
        );
    }

    uit.push_str("## Wat deze prognose niet overziet\n\n");
    for (wat, waarom) in BUITEN_BEELD {
        uit.push_str(&format!("- **{wat}**: {waarom}\n"));
    }
    uit.push_str(
        "\nWat hier niet staat, is daarmee niet in orde bevonden. Het is buiten beeld.\n\n",
    );

    uit.push_str("## Voorbehoud\n\n");
    uit.push_str(
        "De juridische inhoud waarop dit stuk steunt is een vertrekpunt en geen bron van recht. \
         De termijnen en grondslagen zijn niet door een jurist vastgesteld en niet gecontroleerd \
         tegen de geconsolideerde wettekst. De handtekening onder het manifest zegt dat deze \
         bundel niet is gewijzigd sinds zij is samengesteld; zij zegt niets over de juistheid \
         van de inhoud.\n",
    );
    uit
}

fn tabel_van(punten: &[Vervalpunt], nu: DateTime<Utc>, verstreken: bool) -> String {
    let mut uit =
        String::from("| eis | grondslag | oorzaak | eigenaar | datum |\n|---|---|---|---|---|\n");
    for v in punten {
        uit.push_str(&format!(
            "| {} | {} | {} | {} | {}{} |\n",
            v.eis,
            v.grondslag,
            v.oorzaak.omschrijving(),
            v.eigenaar.as_deref().unwrap_or("geen"),
            v.vervalt_op.format("%d-%m-%Y"),
            if verstreken {
                String::new()
            } else {
                format!(" (over {} dagen)", v.dagen_tot_verval(nu))
            }
        ));
    }
    uit.push('\n');
    uit
}

/// De drie factoren per eis, elk met ja of nee.
///
/// Dit is wat er van de driefactorscore overblijft wanneer de schaal en de
/// weging niet worden verzonnen: drie vragen die per eis te beantwoorden zijn,
/// en drie tellingen die samen alles dragen wat een gewogen getal zou dragen.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Aantoonbaarheid {
    pub record_kenmerk: String,
    pub onderdeel: String,
    pub omschrijving: String,
    /// Is er een besluit of beleidsstuk dat de eis vastlegt?
    pub vastgesteld: bool,
    /// Ligt er bewijs dat de eis is uitgevoerd, en geldt dat nu?
    pub uitgevoerd: bool,
    /// Is die uitvoering recent genoeg ten opzichte van de eigen termijn?
    ///
    /// Zonder eigen termijn is deze vraag niet te beantwoorden; dan telt hij
    /// als niet actueel, want een uitvoering zonder afgesproken herhaling
    /// veroudert zonder dat iemand het merkt.
    pub actueel: bool,
    pub eigenaar: Option<String>,
}

/// De telling van de drie factoren over een verzameling eisen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Factortelling {
    pub totaal: usize,
    pub vastgesteld: usize,
    pub uitgevoerd: usize,
    pub actueel: usize,
}

impl Factortelling {
    pub fn van(regels: &[Aantoonbaarheid]) -> Self {
        Self {
            totaal: regels.len(),
            vastgesteld: regels.iter().filter(|r| r.vastgesteld).count(),
            uitgevoerd: regels.iter().filter(|r| r.uitgevoerd).count(),
            actueel: regels.iter().filter(|r| r.actueel).count(),
        }
    }
}

/// Bepaalt de drie factoren per ingerichte maatregel.
///
/// Maatregelen die gemotiveerd niet worden toegepast, tellen niet mee: daar is
/// niets uit te voeren en niets aan te tonen. Dat zij bestaan, staat in het
/// zorgplichtdossier zelf.
pub fn aantoonbaarheid(dossiers: &[Zorgplichtdossier], nu: DateTime<Utc>) -> Vec<Aantoonbaarheid> {
    let mut uit = Vec::new();
    for d in dossiers {
        for m in &d.maatregelen {
            if !matches!(m.toepassing, Toepassing::Ingericht) {
                continue;
            }
            let vastgesteld =
                m.bewijs.iter().any(|b| b.rol == Bewijsrol::Vaststelling && b.telt_mee(nu));
            let uitgevoerd = m.geldig_uitvoeringsbewijs(nu).is_some();
            let actueel = match (m.frequentie.as_ref(), m.maanden_sinds_uitvoering(nu)) {
                (Some(f), Some(maanden)) => uitgevoerd && maanden <= i64::from(f.maanden),
                _ => false,
            };
            uit.push(Aantoonbaarheid {
                record_kenmerk: d.kenmerk.clone(),
                onderdeel: m.code.clone(),
                omschrijving: m.omschrijving.clone(),
                vastgesteld,
                uitgevoerd,
                actueel,
                eigenaar: m.eigenaar.as_ref().map(|e| format!("{} ({})", e.rol, e.persoon)),
            });
        }
    }
    uit
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, NaiveDate, TimeZone};
    use dpofg_domain::{
        risico::Inschatting,
        zorgplicht::{
            Bestuursvaststelling, Bewijsaanwijzing, Bewijskracht, Kaderdefinitie, Kadermaatregel,
            Niettoepassingsvorm, Raamwerkvariant, Zorgplichtonderdeel,
        },
        Motivering,
    };

    fn nu() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 8, 19, 9, 0, 0).unwrap()
    }

    fn termijnen() -> Prognosetermijnen {
        Prognosetermijnen {
            bestuursvaststelling_maanden: 12,
            subverwerkerscontrole_maanden: 12,
            effectbeoordeling_maanden: 36,
            wpg_audit_maanden: 48,
            wpg_controle_maanden: 12,
        }
    }

    fn kader() -> Kaderdefinitie {
        Kaderdefinitie {
            kenmerk: "CBB-ZORGPLICHT-A".into(),
            variant: Raamwerkvariant::A,
            versie: "2026-08-01".into(),
            bron: "Cyberbeveiligingsbesluit".into(),
            geverifieerd_op: NaiveDate::from_ymd_opt(2026, 8, 1),
            toelichting: None,
            maatregelen: Zorgplichtonderdeel::alle()
                .into_iter()
                .map(|o| Kadermaatregel {
                    code: format!("CBB-{}", o.letter()),
                    onderdeel: o,
                    normvindplaats: "art. 6 Cbb".into(),
                    omschrijving: format!("maatregelen voor {}", o.omschrijving()),
                    periodiek: false,
                    niettoepassingsvorm: Niettoepassingsvorm::EigenMotivering,
                    externe_toetsing_verwacht: false,
                })
                .collect(),
        }
    }

    fn bewijs(rol: Bewijsrol, van: DateTime<Utc>, tot: DateTime<Utc>) -> Bewijsaanwijzing {
        Bewijsaanwijzing {
            rol,
            omschrijving: "uitdraai".into(),
            bijlagehash: "a".repeat(64),
            bestandsnaam: "uitdraai.pdf".into(),
            geldig_van: van,
            geldig_tot: tot,
            bewijskracht: Bewijskracht::Zelfgerapporteerd,
            aangewezen_door: "u1".into(),
            aangewezen_op: nu(),
            ingetrokken: None,
        }
    }

    fn dossier() -> Zorgplichtdossier {
        let mut d = Zorgplichtdossier::leid_af(
            "ZRP-2026",
            "Gemeente",
            "A. de Vries",
            &kader(),
            None,
            "u1",
            nu(),
        )
        .unwrap();
        d.wijs_eigenaar_toe("CBB-a", "de beheerder", "J. Jansen", nu()).unwrap();
        d.richt_in("CBB-a", nu()).unwrap();
        d
    }

    /// De maandrekening loopt over de kalender en niet over dertig dagen:
    /// een prognose die de maand afrondt, wijst uiteindelijk de verkeerde week
    /// aan.
    #[test]
    fn de_maandrekening_volgt_de_kalender() {
        let januari = Utc.with_ymd_and_hms(2026, 1, 31, 9, 0, 0).unwrap();
        let later = maanden_later(januari, 1).unwrap();
        assert_eq!(later.format("%d-%m-%Y").to_string(), "28-02-2026");
        // En het tijdstip blijft staan.
        assert_eq!(later.format("%H:%M").to_string(), "09:00");
    }

    #[test]
    fn een_verlopend_bewijsstuk_komt_in_de_prognose() {
        let mut d = dossier();
        d.wijs_bewijs_aan(
            "CBB-a",
            bewijs(Bewijsrol::Uitvoering, nu(), nu() + Duration::days(45)),
            nu(),
        )
        .unwrap();
        let dossiers = [d];
        let bronnen = Bronnen { zorgplicht: &dossiers, ..Default::default() };

        assert!(prognose(&bronnen, termijnen(), nu() + Duration::days(30)).is_empty());
        let punten = prognose(&bronnen, termijnen(), nu() + Duration::days(90));
        assert_eq!(punten.len(), 1);
        assert_eq!(punten[0].oorzaak, Vervaloorzaak::Bewijsstuk);
        assert_eq!(punten[0].onderdeel.as_deref(), Some("CBB-a"));
        assert_eq!(punten[0].eigenaar.as_deref(), Some("de beheerder (J. Jansen)"));
    }

    /// Een maatregel kan op twee manieren omvallen: het stuk verloopt, of de
    /// eigen termijn verstrijkt. Dat zijn twee punten met twee datums.
    #[test]
    fn bewijs_en_frequentie_leveren_elk_een_eigen_punt() {
        let mut k = kader();
        k.maatregelen[0].periodiek = true;
        let mut d =
            Zorgplichtdossier::leid_af("ZRP-2026", "Gemeente", "A. de Vries", &k, None, "u1", nu())
                .unwrap();
        d.wijs_eigenaar_toe("CBB-a", "de beheerder", "J. Jansen", nu()).unwrap();
        d.richt_in("CBB-a", nu()).unwrap();
        d.stel_frequentie_vast(
            "CBB-a",
            6,
            "de directie",
            Motivering::nieuw("halfjaarlijks is passend", "u1", nu()).unwrap(),
            nu(),
        )
        .unwrap();
        d.wijs_bewijs_aan(
            "CBB-a",
            bewijs(Bewijsrol::Uitvoering, nu(), nu() + Duration::days(400)),
            nu(),
        )
        .unwrap();

        let dossiers = [d];
        let bronnen = Bronnen { zorgplicht: &dossiers, ..Default::default() };
        let punten = prognose(&bronnen, termijnen(), nu() + Duration::days(365));
        let oorzaken: Vec<_> = punten.iter().map(|v| v.oorzaak).collect();
        assert!(oorzaken.contains(&Vervaloorzaak::Frequentie), "kreeg: {oorzaken:?}");
        // De frequentie verstrijkt eerder dan het stuk; de lijst is gesorteerd.
        assert_eq!(punten[0].oorzaak, Vervaloorzaak::Frequentie);
    }

    /// Een maatregel die gemotiveerd niet wordt toegepast, valt niet om: er is
    /// niets uit te voeren.
    #[test]
    fn een_niet_toegepaste_maatregel_staat_niet_in_de_prognose() {
        let mut d = dossier();
        d.pas_niet_toe(
            "CBB-a",
            dpofg_domain::zorgplicht::Niettoepassing::EigenMotivering(
                Motivering::nieuw("dit past niet bij onze omvang", "u1", nu()).unwrap(),
            ),
            nu(),
        )
        .unwrap();
        let dossiers = [d];
        let bronnen = Bronnen { zorgplicht: &dossiers, ..Default::default() };
        assert!(prognose(&bronnen, termijnen(), nu() + Duration::days(365)).is_empty());
    }

    #[test]
    fn de_bestuursvaststelling_veroudert_na_de_termijn_uit_het_pakket() {
        let mut d = dossier();
        d.leg_bestuursvaststelling_vast(
            Bestuursvaststelling {
                datum: nu() - Duration::days(300),
                besluittekst: "vastgesteld".into(),
                goedgekeurde_kaderversie: "2026-08-01".into(),
                aanwezigen: vec!["de directie".into()],
                bewijs: bewijs(
                    Bewijsrol::Vaststelling,
                    nu() - Duration::days(300),
                    nu() + Duration::days(300),
                ),
            },
            nu(),
        )
        .unwrap();
        let dossiers = [d];
        let bronnen = Bronnen { zorgplicht: &dossiers, ..Default::default() };

        let punten = prognose(&bronnen, termijnen(), nu() + Duration::days(90));
        let bestuur = punten
            .iter()
            .find(|v| v.oorzaak == Vervaloorzaak::Bestuursvaststelling)
            .expect("de bestuursvaststelling hoort erin te staan");
        assert_eq!(bestuur.eigenaar.as_deref(), Some("het bestuur"));
    }

    /// Wat vandaag al verstreken is, blijft in de prognose staan. Weglaten zou
    /// het beeld verbeteren zonder dat er iets is opgelost.
    #[test]
    fn wat_al_verstreken_is_verdwijnt_niet_uit_het_beeld() {
        let mut b = Risicobeoordeling::nieuw(
            "RIS-2025",
            "de hele organisatie",
            "scenarioanalyse",
            "eigen",
            "de security officer",
            nu() - Duration::days(400),
            nu() - Duration::days(30),
            "u1",
            nu(),
        )
        .unwrap();
        b.onderken(
            "R-01",
            "uitval",
            "een storing",
            "de dienst ligt stil",
            Inschatting::Laag,
            Inschatting::Laag,
            nu(),
        )
        .unwrap();
        let beoordelingen = [b];
        let bronnen = Bronnen { risicobeoordelingen: &beoordelingen, ..Default::default() };

        let punten = prognose(&bronnen, termijnen(), nu());
        assert_eq!(punten.len(), 1);
        assert!(punten[0].is_verstreken(nu()));
        assert_eq!(punten[0].oorzaak, Vervaloorzaak::Risicobeoordeling);
    }

    fn context() -> Rapportcontext {
        Rapportcontext {
            peilmoment: nu(),
            samengesteld_door: "a.devries".into(),
            bestemd_voor: "de directie".into(),
            kennispakket: "nl-start 0.3-start".into(),
            consolidatiedatum: "2026-08-19".into(),
            ketenreikwijdte: "De keten van 12 regels is intern samenhangend.".into(),
            keten_in_orde: true,
            concepten: 0,
            lopende_correcties: 0,
        }
    }

    /// Het stuk noemt geen cijfer en geen kleur, en zegt dat ook.
    #[test]
    fn het_bestuursstuk_noemt_geen_score() {
        let tekst = bestuursstuk(&context(), &[], &[], &[]);
        assert!(tekst.contains("geen cijfer en geen kleur"));
        assert!(tekst.contains("Peilmoment"));
        assert!(tekst.contains("nl-start 0.3-start"));
    }

    /// Wat buiten beeld blijft, staat in het stuk zelf en niet alleen in het
    /// manifest: wie het stuk leest, hoort de grens te zien zonder een tweede
    /// bestand te openen.
    #[test]
    fn het_bestuursstuk_noemt_wat_buiten_beeld_blijft() {
        let tekst = bestuursstuk(&context(), &[], &[], &[]);
        for (wat, _) in BUITEN_BEELD {
            assert!(tekst.contains(wat), "'{wat}' hoort in het stuk te staan");
        }
        assert!(tekst.contains("niet in orde bevonden"));
        assert!(tekst.contains("Voorbehoud"));
    }

    /// Een bestuursstuk dat op een gebroken keten rust, hoort dat te zeggen.
    #[test]
    fn een_gebroken_keten_staat_vetgedrukt_in_het_stuk() {
        let mut c = context();
        c.keten_in_orde = false;
        let tekst = bestuursstuk(&c, &[], &[], &[]);
        assert!(tekst.contains("niet zonder bevindingen doorlopen"), "kreeg:\n{tekst}");
        assert!(tekst.contains("staat niet vast dat de gegevens"));
    }

    /// Wat vandaag al omvalt, staat als stand van zaken en niet als prognose.
    #[test]
    fn het_stuk_scheidt_de_stand_van_zaken_van_de_prognose() {
        let mut d = dossier();
        d.wijs_bewijs_aan(
            "CBB-a",
            bewijs(Bewijsrol::Uitvoering, nu() - Duration::days(10), nu() + Duration::days(45)),
            nu(),
        )
        .unwrap();
        let dossiers = [d];
        let bronnen = Bronnen { zorgplicht: &dossiers, ..Default::default() };
        let over90 = prognose(&bronnen, termijnen(), nu() + Duration::days(90));

        let tekst = bestuursstuk(&context(), &[(90, over90)], &[], &[]);
        assert!(tekst.contains("Er is op dit moment geen eis die niet te bewijzen is"));
        assert!(tekst.contains("Over 90 dagen"));
        assert!(tekst.contains("CBB-a"));
        assert!(tekst.contains("de beheerder (J. Jansen)"));
        assert!(tekst.contains("over 45 dagen"), "de doorlooptijd hoort erbij:\n{tekst}");
    }

    #[test]
    fn de_drie_factoren_staan_als_telling_in_het_stuk() {
        let mut d = dossier();
        d.wijs_bewijs_aan(
            "CBB-a",
            bewijs(Bewijsrol::Vaststelling, nu(), nu() + Duration::days(300)),
            nu(),
        )
        .unwrap();
        let dossiers = [d];
        let tekst = bestuursstuk(&context(), &[], &[], &aantoonbaarheid(&dossiers, nu()));
        assert!(tekst.contains("1 van de 1"), "kreeg:\n{tekst}");
        assert!(tekst.contains("0 van de 1"));
        assert!(tekst.contains("geen score"));
    }

    /// Enkelvoud en meervoud: een stuk dat "1 bronrecords dragen" schrijft,
    /// leest als machinetaal en dat ondermijnt het vertrouwen erin.
    #[test]
    fn het_stuk_telt_in_goed_nederlands() {
        let mut c = context();
        c.concepten = 1;
        c.lopende_correcties = 1;
        let tekst = bestuursstuk(&c, &[], &[], &[]);
        assert!(tekst.contains("1 bronrecord draagt"), "kreeg:\n{tekst}");
        assert!(tekst.contains("Er loopt 1"), "kreeg:\n{tekst}");

        c.concepten = 3;
        c.lopende_correcties = 2;
        let tekst = bestuursstuk(&c, &[], &[], &[]);
        assert!(tekst.contains("3 bronrecords dragen"));
        assert!(tekst.contains("Er lopen 2"));
    }
    /// De drie factoren zijn drie vragen met ja of nee, en geen score.
    #[test]
    fn de_drie_factoren_worden_apart_geteld() {
        let mut k = kader();
        k.maatregelen[0].periodiek = true;
        let mut d =
            Zorgplichtdossier::leid_af("ZRP-2026", "Gemeente", "A. de Vries", &k, None, "u1", nu())
                .unwrap();
        d.wijs_eigenaar_toe("CBB-a", "de beheerder", "J. Jansen", nu()).unwrap();
        d.richt_in("CBB-a", nu()).unwrap();

        // Niets vastgelegd: alle drie nee.
        let dossiers = [d.clone()];
        let telling = Factortelling::van(&aantoonbaarheid(&dossiers, nu()));
        assert_eq!(telling.totaal, 1);
        assert_eq!((telling.vastgesteld, telling.uitgevoerd, telling.actueel), (0, 0, 0));

        // Beleid erbij: vastgesteld ja, de rest nee.
        d.wijs_bewijs_aan(
            "CBB-a",
            bewijs(Bewijsrol::Vaststelling, nu(), nu() + Duration::days(300)),
            nu(),
        )
        .unwrap();
        let dossiers = [d.clone()];
        let telling = Factortelling::van(&aantoonbaarheid(&dossiers, nu()));
        assert_eq!((telling.vastgesteld, telling.uitgevoerd, telling.actueel), (1, 0, 0));

        // Uitvoering erbij, maar zonder eigen termijn is actueel niet te
        // beantwoorden en telt dus als nee.
        let mut uitvoering = bewijs(Bewijsrol::Uitvoering, nu(), nu() + Duration::days(300));
        uitvoering.bijlagehash = "b".repeat(64);
        d.wijs_bewijs_aan("CBB-a", uitvoering, nu()).unwrap();
        let dossiers = [d.clone()];
        let telling = Factortelling::van(&aantoonbaarheid(&dossiers, nu()));
        assert_eq!((telling.vastgesteld, telling.uitgevoerd, telling.actueel), (1, 1, 0));

        // Met een eigen termijn erbij wordt actueel wél beantwoordbaar.
        d.stel_frequentie_vast(
            "CBB-a",
            12,
            "de directie",
            Motivering::nieuw("jaarlijks is passend", "u1", nu()).unwrap(),
            nu(),
        )
        .unwrap();
        let dossiers = [d];
        let telling = Factortelling::van(&aantoonbaarheid(&dossiers, nu()));
        assert_eq!((telling.vastgesteld, telling.uitgevoerd, telling.actueel), (1, 1, 1));
    }
}
