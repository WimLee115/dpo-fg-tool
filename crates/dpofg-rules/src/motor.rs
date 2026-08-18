//! De regelmotor.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Wat er gebeurt als een regel aanslaat.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Niveau {
    /// Verschijnt alleen in de periodieke rapportage.
    Rapporterend,
    /// Verschijnt in de werkvoorraad.
    Signalerend,
    /// De handeling gaat niet door.
    Blokkerend,
}

impl Niveau {
    pub fn omschrijving(&self) -> &'static str {
        match self {
            Self::Rapporterend => "rapporterend",
            Self::Signalerend => "signalerend",
            Self::Blokkerend => "blokkerend",
        }
    }

    /// Of dit niveau de gebruiker onderbreekt.
    ///
    /// Alleen blokkerende regels tellen mee voor het waarschuwingsbudget.
    pub fn onderbreekt(&self) -> bool {
        matches!(self, Self::Blokkerend)
    }
}

/// Naar wie een bevinding gaat.
///
/// Elke regel heeft een vaste ontvanger. Een bevinding zonder ontvanger belandt
/// op de stapel van de functionaris, en dat is precies hoe die stapel groeit
/// tot niemand er meer naar kijkt.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Ontvangerrol {
    Functionaris,
    Behandelaar,
    Contracteigenaar,
    Systeemeigenaar,
    SecurityOfficer,
    Directie,
    Beheerder,
}

impl Ontvangerrol {
    pub fn omschrijving(&self) -> &'static str {
        match self {
            Self::Functionaris => "functionaris voor gegevensbescherming",
            Self::Behandelaar => "behandelaar van het dossier",
            Self::Contracteigenaar => "contracteigenaar",
            Self::Systeemeigenaar => "systeemeigenaar",
            Self::SecurityOfficer => "security officer",
            Self::Directie => "directie",
            Self::Beheerder => "beheerder van de toepassing",
        }
    }
}

/// De definitie van één controleregel.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Regel {
    /// Vaste code, bijvoorbeeld `VWO-01`.
    pub code: String,
    /// De groep waartoe de regel behoort.
    pub groep: String,
    /// Korte naam.
    pub naam: String,
    /// Wat de regel controleert, in gewone taal.
    pub controleert: String,
    pub niveau: Niveau,
    pub ontvanger: Ontvangerrol,
    /// De bepaling waaruit de eis volgt.
    pub grondslag: String,
    /// Of deze regel een gemotiveerde afwijking toestaat.
    ///
    /// Elke ontsnapping wordt geteld: een stijgend aandeel afwijkingen betekent
    /// dat de regel verwordt tot een formaliteit, en dat is zelf een
    /// rapportageregel.
    pub afwijking_mogelijk: bool,
}

impl Regel {
    #[allow(clippy::too_many_arguments)]
    pub fn nieuw(
        code: &str,
        groep: &str,
        naam: &str,
        controleert: &str,
        niveau: Niveau,
        ontvanger: Ontvangerrol,
        grondslag: &str,
        afwijking_mogelijk: bool,
    ) -> Self {
        Self {
            code: code.into(),
            groep: groep.into(),
            naam: naam.into(),
            controleert: controleert.into(),
            niveau,
            ontvanger,
            grondslag: grondslag.into(),
            afwijking_mogelijk,
        }
    }
}

/// Eén treffer van een regel op één record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Bevinding {
    pub regelcode: String,
    pub niveau: Niveau,
    pub ontvanger: Ontvangerrol,
    /// Het record waarop de bevinding slaat.
    pub record_soort: String,
    pub record_id: String,
    /// Het kenmerk van het record, voor herkenbaarheid in een lijst.
    pub record_kenmerk: Option<String>,
    /// Wat er aan de hand is, toegespitst op dit record.
    pub toelichting: String,
    pub grondslag: String,
    /// Wanneer de bevinding is vastgesteld.
    pub vastgesteld_op: DateTime<Utc>,
    /// Of er een gemotiveerde afwijking is vastgelegd.
    pub afwijking: Option<Afwijking>,
}

/// Een gemotiveerde afwijking van een regel.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Afwijking {
    pub motivering: String,
    pub door: String,
    pub op: DateTime<Utc>,
    /// Tot wanneer de afwijking geldt. Een afwijking zonder einddatum wordt de
    /// nieuwe norm, en dat is hoe genormaliseerde deviatie ontstaat.
    pub geldig_tot: Option<DateTime<Utc>>,
}

impl Bevinding {
    /// Of deze bevinding op dit moment nog geldt.
    pub fn is_actief(&self, nu: DateTime<Utc>) -> bool {
        match &self.afwijking {
            None => true,
            Some(a) => a.geldig_tot.is_some_and(|t| nu > t),
        }
    }
}

/// De uitkomst van een regelronde.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Regelrapport {
    pub gedraaid_op: DateTime<Utc>,
    pub regels_gedraaid: usize,
    pub records_beoordeeld: usize,
    pub bevindingen: Vec<Bevinding>,
}

impl Regelrapport {
    /// Bevindingen op een bepaald niveau.
    pub fn op_niveau(&self, niveau: Niveau) -> Vec<&Bevinding> {
        self.bevindingen.iter().filter(|b| b.niveau == niveau).collect()
    }

    /// Bevindingen voor een bepaalde rol.
    pub fn voor(&self, rol: Ontvangerrol) -> Vec<&Bevinding> {
        self.bevindingen.iter().filter(|b| b.ontvanger == rol).collect()
    }

    /// Hoe vaak elke regel aansloeg, aflopend gesorteerd.
    ///
    /// Dit is de lijst waarmee zichtbaar wordt wáár het structureel misgaat, in
    /// plaats van record voor record.
    pub fn per_regel(&self) -> Vec<(String, usize)> {
        let mut tellers: BTreeMap<String, usize> = BTreeMap::new();
        for b in &self.bevindingen {
            *tellers.entry(b.regelcode.clone()).or_default() += 1;
        }
        let mut uit: Vec<_> = tellers.into_iter().collect();
        uit.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
        uit
    }

    /// Of er blokkerende bevindingen zijn.
    pub fn heeft_blokkades(&self) -> bool {
        self.bevindingen.iter().any(|b| b.niveau == Niveau::Blokkerend)
    }

    /// Het aantal bevindingen dat de gebruiker daadwerkelijk onderbreekt.
    pub fn onderbrekingen(&self) -> usize {
        self.bevindingen.iter().filter(|b| b.niveau.onderbreekt()).count()
    }
}

/// De regelmotor: houdt de regeldefinities bij en verzamelt bevindingen.
#[derive(Debug, Default)]
pub struct Regelmotor {
    regels: BTreeMap<String, Regel>,
}

impl Regelmotor {
    pub fn nieuw() -> Self {
        Self::default()
    }

    /// Registreert een regel. Een dubbele code overschrijft de eerdere: de
    /// laatste definitie wint, zodat een kennispakket een regel kan aanscherpen.
    pub fn registreer(&mut self, regel: Regel) {
        self.regels.insert(regel.code.clone(), regel);
    }

    pub fn registreer_alle(&mut self, regels: impl IntoIterator<Item = Regel>) {
        for r in regels {
            self.registreer(r);
        }
    }

    pub fn regel(&self, code: &str) -> Option<&Regel> {
        self.regels.get(code)
    }

    pub fn aantal(&self) -> usize {
        self.regels.len()
    }

    pub fn alle(&self) -> impl Iterator<Item = &Regel> {
        self.regels.values()
    }

    /// De regels van één groep.
    pub fn groep(&self, groep: &str) -> Vec<&Regel> {
        self.regels.values().filter(|r| r.groep == groep).collect()
    }

    /// Alle groepen met hun aantal regels.
    pub fn groepen(&self) -> Vec<(String, usize)> {
        let mut tellers: BTreeMap<String, usize> = BTreeMap::new();
        for r in self.regels.values() {
            *tellers.entry(r.groep.clone()).or_default() += 1;
        }
        tellers.into_iter().collect()
    }

    /// Maakt een bevinding voor een regel.
    pub fn bevind(
        &self,
        code: &str,
        record_soort: &str,
        record_id: &str,
        record_kenmerk: Option<&str>,
        toelichting: impl Into<String>,
        nu: DateTime<Utc>,
    ) -> Option<Bevinding> {
        let regel = self.regels.get(code)?;
        Some(Bevinding {
            regelcode: regel.code.clone(),
            niveau: regel.niveau,
            ontvanger: regel.ontvanger,
            record_soort: record_soort.into(),
            record_id: record_id.into(),
            record_kenmerk: record_kenmerk.map(|s| s.into()),
            toelichting: toelichting.into(),
            grondslag: regel.grondslag.clone(),
            vastgesteld_op: nu,
            afwijking: None,
        })
    }

    /// Stelt een rapport samen uit losse bevindingen.
    pub fn rapporteer(
        &self,
        bevindingen: Vec<Bevinding>,
        records_beoordeeld: usize,
        nu: DateTime<Utc>,
    ) -> Regelrapport {
        Regelrapport {
            gedraaid_op: nu,
            regels_gedraaid: self.regels.len(),
            records_beoordeeld,
            bevindingen,
        }
    }
}
