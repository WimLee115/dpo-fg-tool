//! Opschorting en verlenging van een lopende termijn.
//!
//! # Waarom opschorten de uitzondering is
//!
//! Een opgeschorte termijn is een termijn die niemand meer ziet aftellen. Dat
//! is precies het mechanisme waarmee dossiers blijven liggen. Daarom:
//!
//! * Opschorten kan alleen bij termijnsoorten die daarvoor zijn aangemerkt.
//!   Een 72-uurstermijn kan niet worden opgeschort — die staat op `false` en
//!   dat is geen instelling.
//! * Elke opschorting heeft een verplichte grond en wordt vastgelegd.
//! * Een lopende opschorting is zichtbaar in de werkvoorraad, met de duur
//!   ervan, zodat een opschorting die te lang duurt zelf opvalt.
//!
//! # Verlenging is iets anders dan opschorting
//!
//! Verlengen is een recht dat de wet toekent en dat binnen de oorspronkelijke
//! termijn moet worden ingeroepen — wie te laat is, verliest het (randgeval
//! T-12). Opschorten is het stilzetten van een lopende klok. De motor houdt ze
//! gescheiden en toont ze apart.

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    berekening::{bereken, verschuif_kalenderdeadline, Deadline},
    kalender::Feestdagenkalender,
    soort::Termijnsoort,
    Resultaat, TermijnFout,
};

/// Eén periode waarin de termijn stilstond.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Opschorting {
    /// Wanneer de opschorting begon.
    pub van: DateTime<Utc>,
    /// Wanneer zij eindigde; `None` zolang zij loopt.
    pub tot: Option<DateTime<Utc>>,
    /// Waarom is opgeschort. Verplicht.
    pub grond: String,
    /// Wie de opschorting inriep.
    pub door: String,
}

impl Opschorting {
    /// Hoe lang deze opschorting heeft geduurd tot een peilmoment, in absolute
    /// tijd. Gebruikt bij urentermijnen.
    pub fn duur(&self, nu: DateTime<Utc>) -> Duration {
        let einde = self.tot.unwrap_or(nu);
        einde - self.van
    }

    /// Hoeveel hele kalenderdagen deze opschorting heeft geduurd, gemeten in de
    /// opgegeven tijdzone. Gebruikt bij kalendertermijnen.
    ///
    /// Meten in kalenderdagen in plaats van in uren is geen afronding maar de
    /// juiste maat: een kalendertermijn telt dagen, en een opschorting van
    /// "twee weken" over een zomertijdovergang is veertien dagen, ook al zijn
    /// het 335 uur.
    pub fn duur_in_dagen(&self, nu: DateTime<Utc>, zone: chrono_tz::Tz) -> i64 {
        let einde = self.tot.unwrap_or(nu);
        let van = self.van.with_timezone(&zone).date_naive();
        let tot = einde.with_timezone(&zone).date_naive();
        (tot - van).num_days().max(0)
    }

    pub fn loopt_nog(&self) -> bool {
        self.tot.is_none()
    }
}

/// De toestand van een lopende termijn.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Termijnstatus {
    /// De klok loopt.
    Loopt,
    /// De klok staat stil wegens een opschorting.
    Opgeschort,
    /// De termijn is verstreken.
    Verstreken,
    /// Het dossier is afgerond binnen de termijn.
    Afgerond,
}

/// Een termijn zoals die aan één dossier hangt, met haar geschiedenis.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LopendeTermijn {
    /// De onderliggende soort.
    pub soort: Termijnsoort,
    /// Het ankermoment.
    pub anker: DateTime<Utc>,
    /// De oorspronkelijk berekende deadline, vóór opschorting en verlenging.
    pub oorspronkelijk: Deadline,
    /// Alle opschortingen, in volgorde.
    pub opschortingen: Vec<Opschorting>,
    /// Hoeveel keer is verlengd.
    pub keer_verlengd: u32,
    /// De extra tijd die door verlenging is toegekend.
    pub verlengd_met: Option<Deadline>,
    /// Wanneer het dossier is afgerond, indien afgerond.
    pub afgerond_op: Option<DateTime<Utc>>,
}

impl LopendeTermijn {
    /// Start een termijn op een ankermoment.
    pub fn start(
        soort: Termijnsoort,
        anker: DateTime<Utc>,
        zone: chrono_tz::Tz,
        kalender: &Feestdagenkalender,
    ) -> Resultaat<Self> {
        let oorspronkelijk = bereken(&soort, anker, zone, kalender)?;
        Ok(Self {
            soort,
            anker,
            oorspronkelijk,
            opschortingen: Vec::new(),
            keer_verlengd: 0,
            verlengd_met: None,
            afgerond_op: None,
        })
    }

    /// De totale tijd die de klok heeft stilgestaan tot een peilmoment, in
    /// absolute tijd.
    pub fn totale_opschorting(&self, nu: DateTime<Utc>) -> Duration {
        self.opschortingen.iter().map(|o| o.duur(nu)).fold(Duration::zero(), |a, b| a + b)
    }

    /// De totale opschorting in hele kalenderdagen.
    pub fn totale_opschorting_in_dagen(&self, nu: DateTime<Utc>, zone: chrono_tz::Tz) -> i64 {
        self.opschortingen.iter().map(|o| o.duur_in_dagen(nu, zone)).sum()
    }

    /// De basisdeadline: oorspronkelijk, of de verlengde variant als er is
    /// verlengd. Zonder opschorting.
    fn basis(&self) -> &Deadline {
        self.verlengd_met.as_ref().unwrap_or(&self.oorspronkelijk)
    }

    /// De werkelijke deadline: de basis plus alle opschorting.
    ///
    /// Verlenging schuift de basis op; opschorting telt daar bovenop.
    /// Urentermijnen schuiven in absolute tijd, kalendertermijnen in hele
    /// kalenderdagen — zie [`Opschorting::duur_in_dagen`].
    pub fn deadline(
        &self,
        nu: DateTime<Utc>,
        zone: chrono_tz::Tz,
        kalender: &Feestdagenkalender,
    ) -> Resultaat<DateTime<Utc>> {
        let basis = self.basis();
        if self.opschortingen.is_empty() {
            return Ok(basis.moment);
        }
        if self.soort.eenheid.is_urentermijn() {
            return Ok(basis.moment + self.totale_opschorting(nu));
        }
        let dagen = self.totale_opschorting_in_dagen(nu, zone);
        if dagen <= 0 {
            return Ok(basis.moment);
        }
        let verschoven = verschuif_kalenderdeadline(
            &self.soort,
            basis,
            u32::try_from(dagen).map_err(|_| TermijnFout::DatumBuitenBereik)?,
            zone,
            kalender,
        )?;
        Ok(verschoven.moment)
    }

    /// De volledige deadline met verantwoording, inclusief opschorting.
    pub fn deadline_volledig(
        &self,
        nu: DateTime<Utc>,
        zone: chrono_tz::Tz,
        kalender: &Feestdagenkalender,
    ) -> Resultaat<Deadline> {
        let basis = self.basis();
        if self.opschortingen.is_empty() || self.soort.eenheid.is_urentermijn() {
            let mut d = basis.clone();
            d.moment = self.deadline(nu, zone, kalender)?;
            d.lokaal = d.moment.with_timezone(&zone).format("%d-%m-%Y %H:%M:%S %Z").to_string();
            return Ok(d);
        }
        let dagen = self.totale_opschorting_in_dagen(nu, zone);
        if dagen <= 0 {
            return Ok(basis.clone());
        }
        verschuif_kalenderdeadline(
            &self.soort,
            basis,
            u32::try_from(dagen).map_err(|_| TermijnFout::DatumBuitenBereik)?,
            zone,
            kalender,
        )
    }

    pub fn status(
        &self,
        nu: DateTime<Utc>,
        zone: chrono_tz::Tz,
        kalender: &Feestdagenkalender,
    ) -> Resultaat<Termijnstatus> {
        if self.afgerond_op.is_some() {
            return Ok(Termijnstatus::Afgerond);
        }
        if self.opschortingen.iter().any(|o| o.loopt_nog()) {
            return Ok(Termijnstatus::Opgeschort);
        }
        if nu > self.deadline(nu, zone, kalender)? {
            return Ok(Termijnstatus::Verstreken);
        }
        Ok(Termijnstatus::Loopt)
    }

    /// Schort de termijn op.
    pub fn schort_op(
        &mut self,
        vanaf: DateTime<Utc>,
        grond: impl Into<String>,
        door: impl Into<String>,
    ) -> Resultaat<()> {
        if let Some(op) = self.afgerond_op {
            return Err(TermijnFout::OpschortingNietToegestaan(format!(
                "de termijn is op {} afgerond; een afgesloten termijn schuift niet meer",
                op.to_rfc3339()
            )));
        }
        if !self.soort.opschortbaar {
            return Err(TermijnFout::OpschortingNietToegestaan(format!(
                "termijn {} ({}) is niet opschortbaar",
                self.soort.code, self.soort.naam
            )));
        }
        if self.opschortingen.iter().any(|o| o.loopt_nog()) {
            return Err(TermijnFout::OpschortingLooptAl);
        }
        if vanaf < self.anker {
            return Err(TermijnFout::OpschortingVoorAanvang {
                aanvang: self.anker.to_rfc3339(),
                opschorting: vanaf.to_rfc3339(),
            });
        }
        let grond = grond.into();
        if grond.trim().is_empty() {
            return Err(TermijnFout::OpschortingNietToegestaan(
                "een opschorting zonder grond wordt niet vastgelegd".into(),
            ));
        }
        self.opschortingen.push(Opschorting { van: vanaf, tot: None, grond, door: door.into() });
        Ok(())
    }

    /// Hervat de termijn.
    pub fn hervat(&mut self, op: DateTime<Utc>) -> Resultaat<()> {
        if let Some(op) = self.afgerond_op {
            return Err(TermijnFout::OpschortingNietToegestaan(format!(
                "de termijn is op {} afgerond; een afgesloten termijn schuift niet meer",
                op.to_rfc3339()
            )));
        }
        let lopend = self
            .opschortingen
            .iter_mut()
            .find(|o| o.loopt_nog())
            .ok_or(TermijnFout::GeenLopendeOpschorting)?;
        if op < lopend.van {
            return Err(TermijnFout::OpschortingLooptTerug {
                van: lopend.van.to_rfc3339(),
                tot: op.to_rfc3339(),
            });
        }
        lopend.tot = Some(op);
        Ok(())
    }

    /// Roept het verlengingsrecht in.
    ///
    /// Faalt wanneer de wet geen verlenging kent, wanneer het maximum is
    /// bereikt, of wanneer het bericht te laat komt (randgeval T-12).
    pub fn verleng(
        &mut self,
        bericht_op: DateTime<Utc>,
        zone: chrono_tz::Tz,
        kalender: &Feestdagenkalender,
    ) -> Resultaat<()> {
        if let Some(op) = self.afgerond_op {
            return Err(TermijnFout::OpschortingNietToegestaan(format!(
                "de termijn is op {} afgerond; een afgesloten termijn schuift niet meer",
                op.to_rfc3339()
            )));
        }
        let recht = self.soort.verlenging.clone().ok_or_else(|| {
            TermijnFout::OpschortingNietToegestaan(format!(
                "termijn {} kent geen verlengingsrecht",
                self.soort.code
            ))
        })?;
        if self.keer_verlengd >= recht.aantal_keer {
            return Err(TermijnFout::OpschortingNietToegestaan(format!(
                "termijn {} is al {} keer verlengd; het maximum is {}",
                self.soort.code, self.keer_verlengd, recht.aantal_keer
            )));
        }
        if recht.bericht_binnen_oorspronkelijke_termijn && bericht_op > self.oorspronkelijk.moment {
            return Err(TermijnFout::OpschortingNietToegestaan(format!(
                "het bericht van verlenging moest binnen de oorspronkelijke termijn zijn verzonden, \
                 uiterlijk {}; het is verzonden op {}. Grondslag: {}",
                self.oorspronkelijk.lokaal,
                bericht_op.with_timezone(&zone).format("%d-%m-%Y %H:%M %Z"),
                recht.grondslag
            )));
        }

        // De verlenging telt door vanaf de bestaande deadline.
        let basis = self.basis().moment;
        let verlengsoort = Termijnsoort {
            code: format!("{}-VERLENGING", self.soort.code),
            naam: format!("verlenging van {}", self.soort.naam),
            duur: recht.duur,
            eenheid: recht.eenheid,
            stelsel: self.soort.stelsel,
            aanvang: crate::soort::Aanvang::VanafGebeurtenis,
            grondslag: recht.grondslag.clone(),
            opschortbaar: false,
            verlenging: None,
        };
        self.verlengd_met = Some(bereken(&verlengsoort, basis, zone, kalender)?);
        self.keer_verlengd += 1;
        Ok(())
    }

    /// Rondt het dossier af.
    ///
    /// Een opschorting die op dat moment nog loopt, wordt meteen beëindigd.
    /// Zou dat niet gebeuren, dan zou de berekende einddatum van een afgesloten
    /// dossier elke dag verder opschuiven — ook in een dossier dat al bij een
    /// toezichthouder ligt.
    pub fn rond_af(&mut self, op: DateTime<Utc>) {
        for o in self.opschortingen.iter_mut().filter(|o| o.tot.is_none()) {
            o.tot = Some(op);
        }
        self.afgerond_op = Some(op);
    }

    /// Of de termijn is gehaald. `None` zolang het dossier loopt.
    pub fn is_gehaald(
        &self,
        zone: chrono_tz::Tz,
        kalender: &Feestdagenkalender,
    ) -> Resultaat<Option<bool>> {
        match self.afgerond_op {
            None => Ok(None),
            Some(op) => Ok(Some(op <= self.deadline(op, zone, kalender)?)),
        }
    }
}
