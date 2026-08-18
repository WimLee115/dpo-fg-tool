//! De rekenkern: van ankermoment naar deadline.
//!
//! # De twee stelsels naast elkaar
//!
//! | | Urentermijn | Kalendertermijn |
//! |---|---|---|
//! | Rekent in | UTC | lokale kalenderdagen |
//! | Zomertijd | irrelevant; 72 uur blijft 72 uur | irrelevant; dagen blijven dagen |
//! | Weekend en feestdag | lopen door | verlengen naar de eerstvolgende werkdag |
//! | Eindigt op | een exact tijdstip | het einde van een dag |
//!
//! Die eerste kolom is de reden dat urentermijnen in UTC worden berekend: bij
//! de overgang naar zomertijd verspringt de lokale klok een uur, en wie in
//! lokale tijd rekent, levert een deadline op die een uur te vroeg of te laat
//! ligt. Bij een 72-uurstermijn is dat het verschil tussen een tijdige en een
//! te late melding.

use chrono::{DateTime, Duration, NaiveDate, NaiveTime, TimeZone, Utc};
use chrono_tz::Tz;
use serde::{Deserialize, Serialize};

use crate::{
    kalender::Feestdagenkalender,
    kalenderrekenen::{tel_dagen_op, tel_jaren_op, tel_maanden_op, tel_weken_op},
    soort::{Aanvang, Eenheid, Termijnsoort, ToegepasteVerlenging},
    Resultaat, TermijnFout,
};

/// Standaardtijdzone voor Nederlandse termijnen.
pub const TIJDZONE_NL: &str = "Europe/Amsterdam";

/// Een berekende deadline, inclusief de verantwoording van het rekenwerk.
///
/// De verantwoording is geen extraatje: eis 5 van het termijnrekenkundig
/// uitgangspunt schrijft voor dat de interface bij elke deadline toont welke
/// regel is toegepast en op welke bepaling die berust. Daarom draagt de uitkomst
/// die gegevens zelf mee en kan zij niet zonder worden getoond.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Deadline {
    /// Het exacte moment waarop de termijn verstrijkt, in UTC.
    pub moment: DateTime<Utc>,
    /// Datum en tijd in de tijdzone van het rechtsgebied, als tekst.
    pub lokaal: String,
    /// De gebruikte tijdzone.
    pub tijdzone: String,
    /// Het ankermoment waarop is gerekend.
    pub anker: DateTime<Utc>,
    /// De toegepaste termijnsoort.
    pub code: String,
    /// De duur in woorden, bijvoorbeeld "72 uur".
    pub duur: String,
    /// De bepaling waarop de termijn berust.
    pub grondslag: String,
    /// Of en hoe is verlengd wegens een niet-werkdag.
    pub verlenging: ToegepasteVerlenging,
    /// De bepaling waarop de verlengingsregel berust.
    pub verlengingsbepaling: String,
    /// De volledige rekenverantwoording in één zin, klaar voor weergave.
    pub verantwoording: String,
}

impl Deadline {
    /// Resterende tijd ten opzichte van een peilmoment.
    ///
    /// Negatief wanneer de termijn al is verstreken.
    pub fn resterend(&self, nu: DateTime<Utc>) -> Duration {
        self.moment - nu
    }

    /// Of de termijn op het peilmoment is verstreken.
    pub fn is_verstreken(&self, nu: DateTime<Utc>) -> bool {
        nu > self.moment
    }
}

/// Zoekt een tijdzone op.
pub fn tijdzone(naam: &str) -> Resultaat<Tz> {
    naam.parse::<Tz>().map_err(|_| TermijnFout::OnbekendeTijdzone(naam.to_string()))
}

/// Berekent de deadline van één termijn.
pub fn bereken(
    soort: &Termijnsoort,
    anker: DateTime<Utc>,
    zone: Tz,
    kalender: &Feestdagenkalender,
) -> Resultaat<Deadline> {
    if soort.duur == 0 {
        return Err(TermijnFout::OngeldigeDuur(format!(
            "termijn {} heeft duur 0",
            soort.code
        )));
    }

    if soort.eenheid.is_urentermijn() {
        bereken_uren(soort, anker, zone)
    } else {
        bereken_kalender(soort, anker, zone, kalender)
    }
}

/// Urentermijn: rechttoe rechtaan in UTC, nooit verlengd.
fn bereken_uren(soort: &Termijnsoort, anker: DateTime<Utc>, zone: Tz) -> Resultaat<Deadline> {
    let moment = anker
        .checked_add_signed(Duration::hours(soort.duur as i64))
        .ok_or(TermijnFout::DatumBuitenBereik)?;

    let lokaal = moment.with_timezone(&zone);
    let duur = soort.duur_in_woorden();
    let verantwoording = format!(
        "{duur} in kalendertijd vanaf {}, zonder verlenging voor weekend of feestdag \
         (Verordening (EEG, Euratom) nr. 1182/71, art. 3 lid 1 en 2). Grondslag van de termijn: {}.",
        anker.with_timezone(&zone).format("%d-%m-%Y %H:%M %Z"),
        soort.grondslag
    );

    Ok(Deadline {
        moment,
        lokaal: lokaal.format("%d-%m-%Y %H:%M:%S %Z").to_string(),
        tijdzone: zone.name().to_string(),
        anker,
        code: soort.code.clone(),
        duur,
        grondslag: soort.grondslag.clone(),
        verlenging: ToegepasteVerlenging::NietVanToepassingBijUren,
        verlengingsbepaling:
            "Verordening (EEG, Euratom) nr. 1182/71, art. 3 lid 1 en 2: urentermijnen lopen door"
                .into(),
        verantwoording,
    })
}

/// Kalendertermijn: rekent in lokale kalenderdagen en eindigt aan het eind van
/// een dag, met verlenging naar de eerstvolgende werkdag.
fn bereken_kalender(
    soort: &Termijnsoort,
    anker: DateTime<Utc>,
    zone: Tz,
    kalender: &Feestdagenkalender,
) -> Resultaat<Deadline> {
    let lokaal_anker = anker.with_timezone(&zone);
    let gebeurtenisdag = lokaal_anker.date_naive();

    // De dag waarop de telling aangrijpt.
    let (referentiedag, aftrek) = match soort.aanvang {
        Aanvang::VanafGebeurtenis => (gebeurtenisdag, 0),
        // Awb art. 6:8: de termijn vangt aan met ingang van de dag ná de
        // gebeurtenis. De laatste dag ligt dan één dag eerder dan wanneer je
        // vanaf de referentiedag zou doortellen.
        Aanvang::VanafDagNaGebeurtenis => (
            gebeurtenisdag.succ_opt().ok_or(TermijnFout::DatumBuitenBereik)?,
            1,
        ),
    };

    let ruwe_einddag = match soort.eenheid {
        Eenheid::Klokuren => unreachable!("urentermijnen lopen via bereken_uren"),
        Eenheid::Kalenderdagen => tel_dagen_op(referentiedag, soort.duur)?,
        Eenheid::Weken => tel_weken_op(referentiedag, soort.duur)?,
        Eenheid::Maanden => tel_maanden_op(referentiedag, soort.duur)?,
        Eenheid::Jaren => tel_jaren_op(referentiedag, soort.duur)?,
        Eenheid::Werkdagen => {
            kalender.controleer_dekking(referentiedag)?;
            // Een werkdagentermijn eindigt per definitie op een werkdag; de
            // aftrek voor de dag-na-formulering geldt hier niet.
            return afronden(
                soort,
                anker,
                zone,
                kalender,
                kalender.tel_werkdagen_op(referentiedag, soort.duur)?,
                ToegepasteVerlenging::GeenNodig,
            );
        }
    };

    let einddag = if aftrek == 1 {
        ruwe_einddag.pred_opt().ok_or(TermijnFout::DatumBuitenBereik)?
    } else {
        ruwe_einddag
    };

    // De kalender moet de einddag dekken, ook wanneer er niet verlengd hoeft te
    // worden. Zonder deze controle zou een feestdag die buiten het
    // dekkingsvenster valt stilzwijgend als werkdag gelden, en zou de motor een
    // te vroege deadline melden zonder enig signaal.
    kalender.controleer_dekking(einddag)?;

    // Verlenging naar de eerstvolgende werkdag wanneer de laatste dag geen
    // werkdag is.
    let (definitieve_dag, verlenging) = if kalender.is_werkdag(einddag) {
        (einddag, ToegepasteVerlenging::GeenNodig)
    } else {
        let verschoven = kalender.eerstvolgende_werkdag(einddag)?;
        (
            verschoven,
            ToegepasteVerlenging::NaarEerstvolgendeWerkdag {
                van: einddag.to_string(),
                naar: verschoven.to_string(),
            },
        )
    };

    afronden(soort, anker, zone, kalender, definitieve_dag, verlenging)
}

/// Verschuift een kalenderdeadline met een geheel aantal kalenderdagen.
///
/// Wordt gebruikt bij opschorting. Verschuiven gebeurt in **hele
/// kalenderdagen** en niet in absolute uren, want een kalendertermijn eindigt
/// aan het einde van een dag. Wie er absolute tijd bij optelt, komt na een
/// zomertijdovergang op 22:59 of 00:59 uit — een deadline die een uur te vroeg
/// of op de verkeerde dag ligt.
///
/// Na het verschuiven wordt opnieuw naar de eerstvolgende werkdag verlengd:
/// een opgeschoven deadline die op een zaterdag landt, schuift door.
pub fn verschuif_kalenderdeadline(
    soort: &Termijnsoort,
    basis: &Deadline,
    extra_dagen: u32,
    zone: Tz,
    kalender: &Feestdagenkalender,
) -> Resultaat<Deadline> {
    if soort.eenheid.is_urentermijn() {
        return Err(TermijnFout::OpschortingNietToegestaan(
            "een urentermijn wordt in absolute tijd verschoven, niet in kalenderdagen".into(),
        ));
    }
    // De dag waarop de basisdeadline eindigt, in lokale tijd.
    let basisdag = basis.moment.with_timezone(&zone).date_naive();
    let verschoven = tel_dagen_op(basisdag, extra_dagen)?;

    kalender.controleer_dekking(verschoven)?;
    let (definitieve_dag, verlenging) = if kalender.is_werkdag(verschoven) {
        (verschoven, ToegepasteVerlenging::GeenNodig)
    } else {
        let door = kalender.eerstvolgende_werkdag(verschoven)?;
        (
            door,
            ToegepasteVerlenging::NaarEerstvolgendeWerkdag {
                van: verschoven.to_string(),
                naar: door.to_string(),
            },
        )
    };
    afronden(soort, basis.anker, zone, kalender, definitieve_dag, verlenging)
}

/// Zet de laatste dag om in een exact moment en stelt de verantwoording samen.
fn afronden(
    soort: &Termijnsoort,
    anker: DateTime<Utc>,
    zone: Tz,
    _kalender: &Feestdagenkalender,
    dag: NaiveDate,
    verlenging: ToegepasteVerlenging,
) -> Resultaat<Deadline> {
    // De termijn eindigt bij het verstrijken van de laatste dag.
    let einde_dag = NaiveTime::from_hms_nano_opt(23, 59, 59, 999_999_999)
        .expect("vaste, geldige tijd");
    let naive = dag.and_time(einde_dag);

    let lokaal = match zone.from_local_datetime(&naive) {
        chrono::LocalResult::Single(t) => t,
        // Bij ambiguïteit — het uur dat bij de overgang naar wintertijd twee
        // keer voorkomt — geldt het latere moment. Dat is het gunstigste voor
        // degene die de termijn moet halen.
        chrono::LocalResult::Ambiguous(_, later) => later,
        chrono::LocalResult::None => {
            return Err(TermijnFout::TijdstipBestaatNiet(naive.to_string()))
        }
    };
    let moment = lokaal.with_timezone(&Utc);

    let duur = soort.duur_in_woorden();
    let aanvangstekst = match soort.aanvang {
        Aanvang::VanafGebeurtenis => "vanaf de dag van de gebeurtenis",
        Aanvang::VanafDagNaGebeurtenis => "met ingang van de dag ná de gebeurtenis",
    };
    let verlengingstekst = match &verlenging {
        ToegepasteVerlenging::GeenNodig => {
            "De laatste dag viel op een werkdag; verlenging was niet nodig.".to_string()
        }
        ToegepasteVerlenging::NietVanToepassingBijUren => {
            "Urentermijnen worden niet verlengd.".to_string()
        }
        ToegepasteVerlenging::NaarEerstvolgendeWerkdag { van, naar } => format!(
            "De laatste dag viel op {van}, geen werkdag; de termijn loopt af aan het einde van \
             {naar} ({}).",
            soort.stelsel.verlengingsbepaling()
        ),
    };
    let verantwoording = format!(
        "{duur} gerekend {aanvangstekst}, eindigend bij het verstrijken van de laatste dag. \
         {verlengingstekst} Grondslag van de termijn: {}.",
        soort.grondslag
    );

    Ok(Deadline {
        moment,
        lokaal: lokaal.format("%d-%m-%Y %H:%M:%S %Z").to_string(),
        tijdzone: zone.name().to_string(),
        anker,
        code: soort.code.clone(),
        duur,
        grondslag: soort.grondslag.clone(),
        verlenging,
        verlengingsbepaling: soort.stelsel.verlengingsbepaling().to_string(),
        verantwoording,
    })
}
