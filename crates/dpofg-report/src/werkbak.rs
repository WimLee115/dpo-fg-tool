//! De werkbak: één lijst met wat er openstaat, over alle regimes heen.
//!
//! # Waarom dit hier staat en niet in een scherm
//!
//! De werkbak is in het plan het beginpunt van de hele toepassing, maar er was
//! geen laag die hem kon voeden. `AfgeleideVerplichting` bestond uitsluitend
//! voor incidenten; elke andere dossiersoort rekende haar termijn uit in haar
//! eigen bedieningsmodule, waar niets anders erbij kon. Een werkbak als scherm
//! boven die laag zou een tweede motor zijn geworden, met alle kans dat de
//! twee uit elkaar lopen.
//!
//! Daarom staat de samenvoeging hier, en levert zij gewone gegevens op. De
//! opdrachtregel toont ze, een schil zou ze tonen, en een geplande taak kan ze
//! mailen — allemaal uit dezelfde bron.
//!
//! # Waarom niets hier iets afdoet
//!
//! Een regel verdwijnt uit de werkbak doordat het dossier verandert, nooit
//! doordat iemand hem afvinkt. Er is geen takenlijst met een eigen toestand:
//! die zou binnen een maand uit de pas lopen met de dossiers waarover zij
//! gaat, en dan is de vraag welke van de twee gold.
//!
//! # Wat er niet in staat
//!
//! Deze lijst kent de dossiersoorten die een termijn dragen voor een handeling
//! die nog moet gebeuren. Wat er verloopt zonder dat er iets moet — bewijs met
//! een geldigheidsvenster, een risicobeoordeling, een subverwerkerscontrole —
//! staat in de vervalprognose. Die grens staat ook in de uitvoer, want een
//! lege lijst die als "klaar" wordt gelezen is de duurste fout die een
//! werkvoorraad kan maken.

use chrono::{DateTime, Utc};
use dpofg_domain::{
    correctie::Correctie,
    klokken::{verplichtingen_uit_incident, Zorgplichtcontext},
    verzoek::Betrokkenenverzoek,
    woo::Wooverzoek,
    Incident,
};
use dpofg_terms::{Feestdagenkalender, LopendeTermijn};
use serde::{Deserialize, Serialize};

/// Hoe dringend een regel is.
///
/// De volgorde is vast en niet door de gebruiker om te draaien. Onherstelbaar
/// gaat vóór herstelbaar, en verstreken vóór aanstaand: een gemiste
/// meldtermijn is niet in te halen, een gemiste herzieningsdatum wel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Band {
    /// Onherstelbaar en al verstreken.
    OnherstelbaarVerstreken,
    /// Onherstelbaar, verloopt binnen een dag.
    OnherstelbaarVandaag,
    /// Onherstelbaar, verloopt binnen een week.
    OnherstelbaarDezeWeek,
    /// Herstelbaar en al verstreken.
    Verstreken,
    /// Loopt nog.
    Loopt,
    /// De klok is nog niet begonnen omdat het anker ontbreekt.
    WachtOpAnker,
}

impl Band {
    pub fn omschrijving(&self) -> &'static str {
        match self {
            Self::OnherstelbaarVerstreken => "onherstelbaar en verstreken",
            Self::OnherstelbaarVandaag => "onherstelbaar, verloopt vandaag",
            Self::OnherstelbaarDezeWeek => "onherstelbaar, verloopt deze week",
            Self::Verstreken => "verstreken",
            Self::Loopt => "loopt",
            Self::WachtOpAnker => "wacht op een anker",
        }
    }

    /// Bepaalt de band uit de deadline en de onherstelbaarheid.
    pub fn bepaal(deadline: Option<DateTime<Utc>>, onherstelbaar: bool, nu: DateTime<Utc>) -> Self {
        let Some(deadline) = deadline else {
            return Self::WachtOpAnker;
        };
        let uren = (deadline - nu).num_hours();
        match (onherstelbaar, uren) {
            (true, u) if u < 0 => Self::OnherstelbaarVerstreken,
            (true, u) if u <= 24 => Self::OnherstelbaarVandaag,
            (true, u) if u <= 24 * 7 => Self::OnherstelbaarDezeWeek,
            (false, u) if u < 0 => Self::Verstreken,
            _ => Self::Loopt,
        }
    }
}

/// Eén regel in de werkbak.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Werkbakregel {
    pub record_soort: String,
    pub record_kenmerk: String,
    /// Wat er moet gebeuren, in gewone taal.
    pub wat: String,
    /// De bepaling waaruit de verplichting volgt.
    pub grondslag: String,
    /// Waarop de klok is verankerd, in gewone taal.
    pub anker: String,
    pub deadline: Option<DateTime<Utc>>,
    pub band: Band,
    /// Of een gemiste termijn onherstelbaar is.
    pub onherstelbaar: bool,
    /// Wie het moet doen, als dat uit het dossier blijkt.
    pub eigenaar: Option<String>,
    /// Het hoeveelste spoor van hoeveel, binnen hetzelfde dossier.
    ///
    /// Eén dossier levert net zoveel regels als het lopende klokken heeft. Wie
    /// die samentrekt tot één regel, laat de gebruiker denken dat het dossier
    /// klaar is zodra hij één spoor heeft afgehandeld.
    pub spoor: Option<Spoor>,
}

/// Het hoeveelste spoor van hoeveel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Spoor {
    pub nummer: usize,
    pub totaal: usize,
}

impl Werkbakregel {
    pub fn dagen_tot_deadline(&self, nu: DateTime<Utc>) -> Option<i64> {
        self.deadline.map(|d| (d - nu).num_days())
    }

    pub fn uren_tot_deadline(&self, nu: DateTime<Utc>) -> Option<i64> {
        self.deadline.map(|d| (d - nu).num_hours())
    }
}

/// De dossiers waaruit de werkbak wordt opgebouwd.
#[derive(Debug, Default)]
pub struct Bronnen<'a> {
    pub incidenten: &'a [Incident],
    pub verzoeken: &'a [Betrokkenenverzoek],
    pub wooverzoeken: &'a [Wooverzoek],
    pub correcties: &'a [Correctie],
}

/// Wat een termijn voor een verplichtingcode duurt, uit het kennispakket.
///
/// De werkbak rekent zelf geen termijnen uit. Zij vraagt de duur op en telt
/// die bij het anker; wie hier zelf getallen invult, voert een tweede waarheid
/// naast de termijnencatalogus.
pub trait Termijnbron {
    /// De duur in uren, en of een gemiste termijn onherstelbaar is.
    fn duur(&self, code: &str) -> Option<Termijnkenmerk>;
}

/// Wat de werkbak van een termijn moet weten.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Termijnkenmerk {
    pub uren: i64,
    pub omschrijving: String,
    pub grondslag: String,
    /// Of een gemiste termijn onherstelbaar is.
    ///
    /// Een meldtermijn is dat: te laat melden is niet in te halen. Een
    /// herzieningstermijn niet: die haal je alsnog.
    pub onherstelbaar: bool,
}

/// Waar de werkbak de kalender vandaan haalt.
///
/// Een deadline in maanden is niet met een optelling te bepalen: de
/// termijnenmotor kent maandeinden, feestdagen en de verlenging naar de
/// eerstvolgende werkdag. De werkbak rekent dus niet zelf maar vraagt het
/// daar op, en geeft `None` terug wanneer de motor het niet kan uitrekenen —
/// een verzonnen datum is erger dan een ontbrekende.
#[derive(Debug)]
pub struct Kalendercontext<'a> {
    pub zone: chrono_tz::Tz,
    pub kalender: &'a Feestdagenkalender,
}

impl Kalendercontext<'_> {
    fn deadline(
        &self,
        termijn: Option<&LopendeTermijn>,
        nu: DateTime<Utc>,
    ) -> Option<DateTime<Utc>> {
        termijn?.deadline_volledig(nu, self.zone, self.kalender).ok().map(|d| d.moment)
    }
}

/// Stelt de werkbak samen.
pub fn werkbak(
    bronnen: &Bronnen<'_>,
    termijnen: &dyn Termijnbron,
    kalender: &Kalendercontext<'_>,
    nu: DateTime<Utc>,
) -> Vec<Werkbakregel> {
    let mut uit = Vec::new();
    uit.extend(uit_incidenten(bronnen.incidenten, termijnen, nu));
    uit.extend(uit_verzoeken(bronnen.verzoeken, kalender, nu));
    uit.extend(uit_woo(bronnen.wooverzoeken, kalender, nu));
    uit.extend(uit_correcties(bronnen.correcties, nu));

    // Eerst op band, dan op deadline. Binnen een band staat het meest
    // dringende bovenaan; regels zonder deadline sluiten de rij.
    uit.sort_by(|a, b| {
        a.band
            .cmp(&b.band)
            .then_with(|| match (a.deadline, b.deadline) {
                (Some(x), Some(y)) => x.cmp(&y),
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (None, None) => std::cmp::Ordering::Equal,
            })
            .then_with(|| a.record_kenmerk.cmp(&b.record_kenmerk))
    });
    uit
}

fn uit_incidenten(
    incidenten: &[Incident],
    termijnen: &dyn Termijnbron,
    nu: DateTime<Utc>,
) -> Vec<Werkbakregel> {
    let mut uit = Vec::new();
    for i in incidenten {
        // De zorgplichtcontext komt van buiten het incident. Zolang er geen
        // entiteitrecord is, kan de werkbak niet vaststellen of de meldketen
        // geldt; zij vraagt er dus niet naar en toont alleen het AVG-spoor.
        // Dat gat staat in de uitvoer vermeld.
        let alle = verplichtingen_uit_incident(i, Zorgplichtcontext::niet_van_toepassing());
        let open: Vec<_> = alle.iter().filter(|v| v.staat_open()).collect();
        let totaal = open.len();
        for (nummer, v) in open.iter().enumerate() {
            let kenmerk = termijnen.duur(v.code.code());
            let deadline = match (&kenmerk, v.anker) {
                (Some(k), Some(anker)) => Some(anker + chrono::Duration::hours(k.uren)),
                _ => None,
            };
            let onherstelbaar = kenmerk.as_ref().is_some_and(|k| k.onherstelbaar);
            uit.push(Werkbakregel {
                record_soort: "incident".into(),
                record_kenmerk: i.kenmerk.clone(),
                wat: kenmerk
                    .as_ref()
                    .map(|k| k.omschrijving.clone())
                    .unwrap_or_else(|| v.code.code().to_string()),
                grondslag: kenmerk
                    .as_ref()
                    .map(|k| k.grondslag.clone())
                    .unwrap_or_else(|| "niet in het kennispakket".into()),
                anker: format!("{} — {}", v.ankertype.omschrijving(), v.reden),
                deadline,
                band: Band::bepaal(deadline, onherstelbaar, nu),
                onherstelbaar,
                eigenaar: None,
                spoor: Some(Spoor { nummer: nummer + 1, totaal }),
            });
        }
    }
    uit
}

fn uit_verzoeken(
    verzoeken: &[Betrokkenenverzoek],
    kalender: &Kalendercontext<'_>,
    nu: DateTime<Utc>,
) -> Vec<Werkbakregel> {
    verzoeken
        .iter()
        .filter(|v| v.afgehandeld_op.is_none())
        .map(|v| {
            let deadline = kalender.deadline(v.termijn.as_ref(), nu);
            Werkbakregel {
                record_soort: "verzoek".into(),
                record_kenmerk: v.kenmerk.clone(),
                wat: format!("het verzoek afhandelen: {}", v.omschrijving),
                grondslag: "art. 12 lid 3 AVG".into(),
                anker: "ontvangst van het verzoek".into(),
                deadline,
                // Een gemiste maandtermijn is niet in te halen: de betrokkene
                // kan vanaf dat moment naar de toezichthouder.
                band: Band::bepaal(deadline, true, nu),
                onherstelbaar: true,
                eigenaar: Some(v.behandelaar.clone()),
                spoor: None,
            }
        })
        .collect()
}

fn uit_woo(
    verzoeken: &[Wooverzoek],
    kalender: &Kalendercontext<'_>,
    nu: DateTime<Utc>,
) -> Vec<Werkbakregel> {
    verzoeken
        .iter()
        .filter(|d| d.besluit_op.is_none())
        .map(|d| {
            let deadline = kalender.deadline(d.termijn.as_ref(), nu);
            Werkbakregel {
                record_soort: "woo".into(),
                record_kenmerk: d.kenmerk.clone(),
                wat: "beslissen op het verzoek om informatie".into(),
                grondslag: "art. 4.4 Wet open overheid".into(),
                anker: "ontvangst van het verzoek".into(),
                deadline,
                band: Band::bepaal(deadline, true, nu),
                onherstelbaar: true,
                eigenaar: None,
                spoor: None,
            }
        })
        .collect()
}

fn uit_correcties(correcties: &[Correctie], nu: DateTime<Utc>) -> Vec<Werkbakregel> {
    correcties
        .iter()
        .filter(|c| !c.is_afgerond())
        .map(|c| Werkbakregel {
            record_soort: "correctie".into(),
            record_kenmerk: c.kenmerk.clone(),
            wat: format!("{}: {}", c.soort.omschrijving(), c.bevinding.aanduiding()),
            grondslag: "interne norm; de correctieplicht volgt uit de verantwoordingsplicht".into(),
            anker: "de afgesproken einddatum".into(),
            deadline: Some(c.uiterlijk),
            // Een correctietermijn is zelf vastgesteld en dus in te halen; wat
            // erachter zit kan onherstelbaar zijn, maar dat staat daar.
            band: Band::bepaal(Some(c.uiterlijk), false, nu),
            onherstelbaar: false,
            eigenaar: Some(format!("{} ({})", c.eigenaar_rol, c.eigenaar_persoon)),
            spoor: None,
        })
        .collect()
}

/// Wat er níet in de werkbak staat.
///
/// Hoort onder elke weergave. Een lege lijst die als "klaar" wordt gelezen is
/// de duurste fout die een werkvoorraad kan maken, en het enige tegengif is
/// dat er staat wat er buiten valt.
pub const NIET_IN_DE_LIJST: [(&str, &str); 4] = [
    (
        "wat verloopt zonder dat er iets moet",
        "bewijs met een geldigheidsvenster, risicobeoordelingen en subverwerkerscontroles staan \
         in 'dpofg prognose'",
    ),
    (
        "de meldketen van de zorgplicht",
        "die geldt alleen voor aangewezen entiteiten, en er is nog geen entiteitrecord waaruit \
         dat blijkt; het AVG-spoor van een incident staat er wel in",
    ),
    (
        "onvolledige dossiers zonder termijn",
        "wat er per dossier ontbreekt, staat bij het dossier zelf en in 'dpofg controle'",
    ),
    (
        "wat nog niet is vastgelegd",
        "een verplichting die uit een niet-geregistreerd feit volgt, kan hier per definitie niet \
         staan",
    ),
];

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, TimeZone};
    use dpofg_domain::incident::{Herkomstkanaal, Zorgtrap};

    fn nu() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 8, 19, 9, 0, 0).unwrap()
    }

    struct Vasttermijn;

    impl Termijnbron for Vasttermijn {
        fn duur(&self, code: &str) -> Option<Termijnkenmerk> {
            match code {
                "AVG-33-MELDING" => Some(Termijnkenmerk {
                    uren: 72,
                    omschrijving: "de inbreuk melden aan de toezichthouder".into(),
                    grondslag: "art. 33 lid 1 AVG".into(),
                    onherstelbaar: true,
                }),
                "AVG-33-5-REGISTER" => Some(Termijnkenmerk {
                    uren: 24 * 30,
                    omschrijving: "de inbreuk intern vastleggen".into(),
                    grondslag: "art. 33 lid 5 AVG".into(),
                    onherstelbaar: false,
                }),
                _ => None,
            }
        }
    }

    /// Een lege kalender: de tests hier gaan over incidenten en correcties,
    /// die op klokuren rekenen en dus geen feestdagen nodig hebben.
    fn kalender() -> Kalendercontext<'static> {
        static LEEG: std::sync::OnceLock<Feestdagenkalender> = std::sync::OnceLock::new();
        Kalendercontext {
            zone: chrono_tz::Europe::Amsterdam,
            kalender: LEEG.get_or_init(|| Feestdagenkalender::leeg("NL", 2026, 2030)),
        }
    }

    fn incident() -> Incident {
        let mut i = Incident::nieuw(
            "2026-0041",
            "verkeerd geadresseerde brief",
            nu() - Duration::hours(4),
            nu() - Duration::hours(4),
            Herkomstkanaal::InternVastgesteld,
            "u1",
            "u1",
        );
        i.stel_kennisname_vast(nu() - Duration::hours(4), None).unwrap();
        i
    }

    /// Eén dossier levert net zoveel regels als het lopende klokken heeft.
    #[test]
    fn een_incident_levert_een_regel_per_open_verplichting() {
        let incidenten = [incident()];
        let bronnen = Bronnen { incidenten: &incidenten, ..Default::default() };
        let regels = werkbak(&bronnen, &Vasttermijn, &kalender(), nu());

        assert_eq!(regels.len(), 2, "melding en interne vastlegging");
        assert!(regels.iter().all(|r| r.spoor.is_some_and(|s| s.totaal == 2)));
        assert!(regels[0].grondslag.contains("art. 33 lid 1"));
    }

    /// De kern: een verplichting verdwijnt doordat het dossier verandert.
    #[test]
    fn een_verzonden_melding_haalt_de_regel_uit_de_lijst() {
        let mut i = incident();
        let bronnen = Bronnen { incidenten: std::slice::from_ref(&i), ..Default::default() };
        assert_eq!(werkbak(&bronnen, &Vasttermijn, &kalender(), nu()).len(), 2);

        i.leg_melding_vast(nu(), Some("AP-2026-441".into()), nu()).unwrap();
        let incidenten = [i];
        let bronnen = Bronnen { incidenten: &incidenten, ..Default::default() };
        let regels = werkbak(&bronnen, &Vasttermijn, &kalender(), nu());
        assert_eq!(regels.len(), 1);
        assert!(regels[0].grondslag.contains("art. 33 lid 5"));
        // En het spoor telt mee met wat er nog openstaat.
        assert_eq!(regels[0].spoor.unwrap().totaal, 1);
    }

    /// Onherstelbaar gaat vóór herstelbaar, en verstreken vóór aanstaand.
    #[test]
    fn de_volgorde_zet_het_onherstelbare_bovenaan() {
        let incidenten = [incident()];
        let bronnen = Bronnen { incidenten: &incidenten, ..Default::default() };

        // Vier uur na kennisname is er nog 68 uur te gaan, en dat valt binnen
        // de week: een onherstelbare klok van tweeënzeventig uur is vanaf het
        // eerste moment dringend, en de banden horen dat te laten zien.
        let regels = werkbak(&bronnen, &Vasttermijn, &kalender(), nu());
        assert_eq!(regels[0].band, Band::OnherstelbaarDezeWeek);
        assert!(regels[0].grondslag.contains("art. 33 lid 1"));
        // De interne vastlegging is herstelbaar en staat daaronder.
        assert_eq!(regels[1].band, Band::Loopt);

        // Vlak voor het verstrijken staat hij bovenaan in de dagband.
        let laat = werkbak(&bronnen, &Vasttermijn, &kalender(), nu() + Duration::hours(60));
        assert_eq!(laat[0].band, Band::OnherstelbaarVandaag);
        assert!(laat[0].grondslag.contains("art. 33 lid 1"));

        // En na afloop in de verstreken band, nog steeds bovenaan.
        let verstreken = werkbak(&bronnen, &Vasttermijn, &kalender(), nu() + Duration::hours(100));
        assert_eq!(verstreken[0].band, Band::OnherstelbaarVerstreken);
    }

    /// Een verplichting zonder anker verdwijnt niet; zij zakt naar onderen met
    /// de reden erbij.
    #[test]
    fn een_verplichting_zonder_anker_blijft_zichtbaar() {
        let mut i = incident();
        i.kennisname_op = None;
        let incidenten = [i];
        let bronnen = Bronnen { incidenten: &incidenten, ..Default::default() };
        let regels = werkbak(&bronnen, &Vasttermijn, &kalender(), nu());

        assert!(regels.iter().all(|r| r.band == Band::WachtOpAnker));
        assert!(regels.iter().all(|r| r.deadline.is_none()));
        assert!(regels[0].anker.contains("kennisname"));
    }

    /// Een code die het kennispakket niet kent, levert geen deadline op en
    /// verdwijnt niet: dat gat hoort zichtbaar te zijn.
    #[test]
    fn een_onbekende_termijncode_verbergt_de_verplichting_niet() {
        let mut i = incident();
        i.risiconiveau = Some(dpofg_domain::Risiconiveau::HoogRisico);
        i.risicoweging = Some(dpofg_domain::Motivering::nieuw("hoog risico", "u1", nu()).unwrap());
        let incidenten = [i];
        let bronnen = Bronnen { incidenten: &incidenten, ..Default::default() };
        let regels = werkbak(&bronnen, &Vasttermijn, &kalender(), nu());

        let mededeling = regels
            .iter()
            .find(|r| r.wat.contains("AVG-34"))
            .expect("de mededeling hoort in de lijst te staan");
        assert!(mededeling.grondslag.contains("niet in het kennispakket"));
        assert_eq!(mededeling.band, Band::WachtOpAnker);
    }

    /// De zorgplichtketen kan worden afgedaan, trap voor trap.
    #[test]
    fn de_zorgplichtketen_is_af_te_doen() {
        let mut i = incident();
        i.leg_zorgverzending_vast(Zorgtrap::Waarschuwing, nu(), nu()).unwrap();
        assert!(i.zorgketen.waarschuwing_op.is_some());
        assert!(i.zorgketen.melding_op.is_none());
    }

    #[test]
    fn een_afgeronde_correctie_staat_niet_in_de_lijst() {
        use dpofg_domain::correctie::{Bevindingsleutel, Correctiesoort};
        let mut c = Correctie::nieuw(
            "COR-001",
            Bevindingsleutel::nieuw("ZRP-04", "zorgplicht", "ZRP-2026"),
            "een tekortkoming",
            Correctiesoort::Herstel,
            false,
            "de security officer",
            "J. Jansen",
            nu() + Duration::days(30),
            dpofg_domain::Motivering::nieuw("wij pakken dit op bij de kwartaalronde", "u1", nu())
                .unwrap(),
            "u1",
            nu(),
        )
        .unwrap();

        let lijst = [c.clone()];
        let bronnen = Bronnen { correcties: &lijst, ..Default::default() };
        assert_eq!(werkbak(&bronnen, &Vasttermijn, &kalender(), nu()).len(), 1);

        c.rond_af(
            "A. de Vries",
            dpofg_domain::Motivering::nieuw("de uitdraaien zijn aangeleverd", "u1", nu()).unwrap(),
            nu(),
        )
        .unwrap();
        let lijst = [c];
        let bronnen = Bronnen { correcties: &lijst, ..Default::default() };
        assert!(werkbak(&bronnen, &Vasttermijn, &kalender(), nu()).is_empty());
    }

    /// Wat er buiten valt, staat erbij. Een lege lijst die als "klaar" wordt
    /// gelezen is de duurste fout die een werkvoorraad kan maken.
    #[test]
    fn wat_er_niet_in_staat_is_benoemd() {
        assert_eq!(NIET_IN_DE_LIJST.len(), 4);
        assert!(NIET_IN_DE_LIJST.iter().any(|(w, _)| w.contains("verloopt")));
        assert!(NIET_IN_DE_LIJST.iter().any(|(_, u)| u.contains("dpofg prognose")));
    }
}
