//! Veldmapping: het register naast de werkelijkheid leggen.
//!
//! # Het probleem dat dit oplost
//!
//! Een register is een momentopname. Systemen veranderen: er komt een veld bij,
//! er wordt een kolom hernoemd, een koppeling levert ineens meer op dan
//! afgesproken. Het register merkt daar niets van, want niemand kijkt.
//!
//! Deze module legt een lijst veldnamen uit een systeem naast de categorieën
//! gegevens die in de registerregel staan, en meldt het verschil. Wat er nieuw
//! bij staat, is een verwerking die niet is vastgelegd. Wat er ontbreekt, is
//! een registerregel die meer belooft dan het systeem bevat — ook dat is een
//! afwijking, want een register dat te veel noemt, is even onbetrouwbaar als
//! een register dat te weinig noemt.
//!
//! # Waarom één generieke mapping en niet drie importeurs
//!
//! De vorige planversie kende drie afzonderlijke driftimporteurs, elk voor een
//! eigen bron. Dat is drie keer hetzelfde onderhoud voor hetzelfde probleem.
//! Hier is er één profiel dat per bron wordt bewaard: welke veldnaam hoort bij
//! welke categorie, en welke velden doen niet mee. De invoer is bewust zo saai
//! mogelijk — een lijst veldnamen — zodat er geen bestandsformaat hoeft te
//! worden ontleed dat morgen weer anders is.
//!
//! # Wat het niet doet
//!
//! Het leest geen systemen uit en legt geen verbinding. De lijst veldnamen komt
//! van de beheerder van het systeem; de tool vergelijkt.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

use crate::{
    basis::{Compartiment, Herkomst, Id, Status},
    error::{DomeinFout, Resultaat},
    volledigheid::{Ontbrekend, Volledig},
};

/// Eén koppeling tussen een veld in het bronsysteem en het register.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Veldkoppeling {
    /// De veldnaam zoals het systeem hem kent.
    pub bronveld: String,
    /// De categorie gegevens zoals die in de registerregel staat.
    pub categorie: String,
}

/// Wat één vergelijking opleverde.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Verschilrapport {
    pub uitgevoerd_op: DateTime<Utc>,
    /// Velden die het systeem heeft en die nergens op zijn afgebeeld.
    ///
    /// Dit is de belangrijkste uitkomst: een veld dat niemand heeft aangewezen
    /// is een verwerking die niet in het register staat.
    pub nieuw_in_bron: Vec<String>,
    /// Categorieën uit het register waarvoor geen enkel bronveld is gevonden.
    pub ontbreekt_in_bron: Vec<String>,
    /// Koppelingen die aan beide kanten bestaan.
    pub bevestigd: Vec<String>,
    /// Velden die bewust buiten beschouwing blijven.
    pub genegeerd: Vec<String>,
}

impl Verschilrapport {
    /// Of er iets is dat aandacht vraagt.
    pub fn heeft_afwijkingen(&self) -> bool {
        !self.nieuw_in_bron.is_empty() || !self.ontbreekt_in_bron.is_empty()
    }

    pub fn aantal_afwijkingen(&self) -> usize {
        self.nieuw_in_bron.len() + self.ontbreekt_in_bron.len()
    }
}

/// Een bewaard mappingprofiel voor één bronsysteem.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Mappingprofiel {
    pub id: Id,
    pub kenmerk: String,
    pub omschrijving: String,
    pub status: Status,
    pub compartiment: Compartiment,
    pub herkomst: Herkomst,

    /// Het systeem waaruit de veldnamen komen.
    pub bron: String,
    /// De registerregel waartegen wordt vergeleken.
    pub verwerking_id: Id,
    pub verwerking_kenmerk: String,

    pub koppelingen: Vec<Veldkoppeling>,
    /// Velden die bewust niet meedoen, met de reden erbij.
    pub genegeerd: Vec<(String, String)>,

    /// De uitkomst van de laatste vergelijking.
    pub laatste_rapport: Option<Verschilrapport>,
}

impl Mappingprofiel {
    pub fn nieuw(
        kenmerk: impl Into<String>,
        omschrijving: impl Into<String>,
        bron: impl Into<String>,
        verwerking_id: Id,
        verwerking_kenmerk: impl Into<String>,
        door: impl Into<String>,
        op: DateTime<Utc>,
    ) -> Self {
        Self {
            id: Id::nieuw(),
            kenmerk: kenmerk.into(),
            omschrijving: omschrijving.into(),
            status: Status::Concept,
            // Een mappingprofiel bevat veldnamen, geen gegevens. Het algemene
            // compartiment volstaat, en dat scheelt een ontgrendeling bij het
            // werk waarvoor het bedoeld is: periodiek vergelijken.
            compartiment: Compartiment::algemeen(),
            herkomst: Herkomst::nieuw(door, op),
            bron: bron.into(),
            verwerking_id,
            verwerking_kenmerk: verwerking_kenmerk.into(),
            koppelingen: Vec::new(),
            genegeerd: Vec::new(),
            laatste_rapport: None,
        }
    }

    /// Koppelt een veld uit het bronsysteem aan een categorie in het register.
    pub fn koppel(
        &mut self,
        bronveld: impl Into<String>,
        categorie: impl Into<String>,
        op: DateTime<Utc>,
    ) -> Resultaat<()> {
        let bronveld = bronveld.into();
        let categorie = categorie.into();
        if bronveld.trim().is_empty() || categorie.trim().is_empty() {
            return Err(DomeinFout::OngeldigeWaarde {
                veld: "mapping.koppeling".into(),
                reden: "een koppeling vergt zowel een veldnaam als een categorie".into(),
            });
        }
        if self.genegeerd.iter().any(|(v, _)| v == &bronveld) {
            return Err(DomeinFout::OngeldigeWaarde {
                veld: "mapping.koppeling".into(),
                reden: format!(
                    "'{bronveld}' staat als genegeerd in dit profiel; haal het daar eerst weg"
                ),
            });
        }
        if let Some(bestaand) = self.koppelingen.iter_mut().find(|k| k.bronveld == bronveld) {
            bestaand.categorie = categorie;
        } else {
            self.koppelingen.push(Veldkoppeling { bronveld, categorie });
        }
        self.herkomst.wijzig("koppeling vastgelegd", op);
        Ok(())
    }

    /// Laat een veld bewust buiten beschouwing.
    ///
    /// Met een reden, altijd. Een genegeerd veld zonder reden is een veld dat
    /// iemand ooit heeft weggeklikt, en dat is precies wat deze vergelijking
    /// moet voorkomen.
    pub fn negeer(
        &mut self,
        bronveld: impl Into<String>,
        reden: impl Into<String>,
        op: DateTime<Utc>,
    ) -> Resultaat<()> {
        let bronveld = bronveld.into();
        let reden = reden.into();
        if reden.trim().is_empty() {
            return Err(DomeinFout::OngeldigeWaarde {
                veld: "mapping.genegeerd".into(),
                reden: format!(
                    "schrijf op waarom '{bronveld}' niet meedoet; zonder reden is dit een \
                     weggeklikte melding en geen besluit"
                ),
            });
        }
        self.koppelingen.retain(|k| k.bronveld != bronveld);
        if let Some(bestaand) = self.genegeerd.iter_mut().find(|(v, _)| v == &bronveld) {
            bestaand.1 = reden;
        } else {
            self.genegeerd.push((bronveld, reden));
        }
        self.herkomst.wijzig("veld genegeerd", op);
        Ok(())
    }

    /// Vergelijkt een lijst veldnamen uit het bronsysteem met het register.
    ///
    /// De categorieën komen van de aanroeper: dit type kent het register niet,
    /// en dat scheelt een afhankelijkheid die er niet hoeft te zijn.
    pub fn vergelijk(
        &self,
        bronvelden: &[String],
        registercategorieen: &[String],
        nu: DateTime<Utc>,
    ) -> Verschilrapport {
        let genegeerd: BTreeSet<&str> = self.genegeerd.iter().map(|(v, _)| v.as_str()).collect();
        let gekoppeld: BTreeSet<&str> =
            self.koppelingen.iter().map(|k| k.bronveld.as_str()).collect();

        let mut nieuw_in_bron = Vec::new();
        let mut bevestigd = Vec::new();
        let mut gezien_genegeerd = Vec::new();
        for veld in bronvelden {
            let veld = veld.trim();
            if veld.is_empty() {
                continue;
            }
            if genegeerd.contains(veld) {
                gezien_genegeerd.push(veld.to_string());
            } else if gekoppeld.contains(veld) {
                bevestigd.push(veld.to_string());
            } else {
                nieuw_in_bron.push(veld.to_string());
            }
        }

        // Welke categorieën heeft de bron werkelijk laten zien?
        let aanwezige_categorieen: BTreeSet<&str> = self
            .koppelingen
            .iter()
            .filter(|k| bevestigd.iter().any(|b| b == &k.bronveld))
            .map(|k| k.categorie.as_str())
            .collect();

        let ontbreekt_in_bron: Vec<String> = registercategorieen
            .iter()
            .map(|c| c.trim())
            .filter(|c| !c.is_empty() && !aanwezige_categorieen.contains(c))
            .map(String::from)
            .collect();

        nieuw_in_bron.sort();
        nieuw_in_bron.dedup();
        bevestigd.sort();
        bevestigd.dedup();
        gezien_genegeerd.sort();
        gezien_genegeerd.dedup();

        Verschilrapport {
            uitgevoerd_op: nu,
            nieuw_in_bron,
            ontbreekt_in_bron,
            bevestigd,
            genegeerd: gezien_genegeerd,
        }
    }

    /// Bewaart de uitkomst van een vergelijking.
    pub fn leg_rapport_vast(&mut self, rapport: Verschilrapport, op: DateTime<Utc>) {
        self.herkomst.wijzig(
            format!("vergelijking uitgevoerd: {} afwijking(en)", rapport.aantal_afwijkingen()),
            op,
        );
        self.laatste_rapport = Some(rapport);
    }
}

impl Volledig for Mappingprofiel {
    fn soortnaam(&self) -> &'static str {
        "mappingprofiel"
    }

    fn aantal_verplichte_onderdelen(&self) -> usize {
        // Ten minste één koppeling en één uitgevoerde vergelijking.
        2
    }

    fn ontbrekende_onderdelen(&self) -> Vec<Ontbrekend> {
        let mut uit = Vec::new();
        if self.koppelingen.is_empty() {
            uit.push(Ontbrekend::blokkerend(
                "mapping.koppelingen",
                "koppel ten minste één veld uit het bronsysteem aan een categorie uit het register",
                "art. 30 lid 1 onder c AVG; interne norm",
            ));
        }
        match &self.laatste_rapport {
            None => uit.push(Ontbrekend::blokkerend(
                "mapping.vergelijking",
                "voer een vergelijking uit tegen een actuele lijst veldnamen",
                "art. 5 lid 2 AVG; verantwoordingsplicht",
            )),
            Some(r) if r.heeft_afwijkingen() => uit.push(Ontbrekend::signalerend(
                "mapping.afwijkingen",
                format!(
                    "de laatste vergelijking leverde {} afwijking(en) op; werk het register bij \
                     of leg vast waarom het veld niet meedoet",
                    r.aantal_afwijkingen()
                ),
                "art. 30 AVG",
            )),
            Some(_) => {}
        }
        uit
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn nu() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 8, 19, 9, 0, 0).unwrap()
    }

    fn profiel() -> Mappingprofiel {
        Mappingprofiel::nieuw(
            "MAP-0412",
            "verzuimsysteem naast de registerregel",
            "verzuimsysteem",
            Id::nieuw(),
            "0412-K",
            "u1",
            nu(),
        )
    }

    fn regels(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| s.to_string()).collect()
    }

    /// De kernuitkomst: een veld dat niemand heeft aangewezen, is een
    /// verwerking die niet in het register staat.
    #[test]
    fn een_nieuw_veld_in_de_bron_valt_op() {
        let mut p = profiel();
        p.koppel("achternaam", "naam", nu()).unwrap();

        let r = p.vergelijk(&regels(&["achternaam", "bsn"]), &regels(&["naam"]), nu());
        assert_eq!(r.nieuw_in_bron, vec!["bsn".to_string()]);
        assert!(r.ontbreekt_in_bron.is_empty());
        assert!(r.heeft_afwijkingen());
    }

    /// Een register dat te veel noemt, is even onbetrouwbaar als een register
    /// dat te weinig noemt.
    #[test]
    fn een_categorie_zonder_veld_in_de_bron_valt_ook_op() {
        let mut p = profiel();
        p.koppel("achternaam", "naam", nu()).unwrap();

        let r = p.vergelijk(&regels(&["achternaam"]), &regels(&["naam", "gezondheid"]), nu());
        assert!(r.nieuw_in_bron.is_empty());
        assert_eq!(r.ontbreekt_in_bron, vec!["gezondheid".to_string()]);
    }

    #[test]
    fn een_genegeerd_veld_telt_niet_mee() {
        let mut p = profiel();
        p.koppel("achternaam", "naam", nu()).unwrap();
        p.negeer("aangemaakt_op", "technisch veld zonder persoonsgegevens", nu()).unwrap();

        let r = p.vergelijk(&regels(&["achternaam", "aangemaakt_op"]), &regels(&["naam"]), nu());
        assert!(!r.heeft_afwijkingen());
        assert_eq!(r.genegeerd, vec!["aangemaakt_op".to_string()]);
    }

    /// Een genegeerd veld zonder reden is een weggeklikte melding.
    #[test]
    fn negeren_zonder_reden_wordt_geweigerd() {
        let mut p = profiel();
        let fout = p.negeer("aangemaakt_op", "  ", nu()).unwrap_err();
        assert!(fout.to_string().contains("weggeklikte melding"));
    }

    #[test]
    fn negeren_haalt_een_bestaande_koppeling_weg() {
        let mut p = profiel();
        p.koppel("achternaam", "naam", nu()).unwrap();
        p.negeer("achternaam", "wordt in dit systeem niet meer gevuld", nu()).unwrap();
        assert!(p.koppelingen.is_empty());

        // En koppelen kan dan niet meer zonder eerst de uitzondering weg te halen.
        assert!(p.koppel("achternaam", "naam", nu()).is_err());
    }

    #[test]
    fn een_bestaande_koppeling_wordt_bijgewerkt_en_niet_verdubbeld() {
        let mut p = profiel();
        p.koppel("achternaam", "naam", nu()).unwrap();
        p.koppel("achternaam", "naamsgegevens", nu()).unwrap();
        assert_eq!(p.koppelingen.len(), 1);
        assert_eq!(p.koppelingen[0].categorie, "naamsgegevens");
    }

    /// Een koppeling waarvan het bronveld niet meer bestaat, bewijst de
    /// categorie niet.
    #[test]
    fn een_koppeling_zonder_bronveld_bevestigt_niets() {
        let mut p = profiel();
        p.koppel("achternaam", "naam", nu()).unwrap();
        let r = p.vergelijk(&regels(&[]), &regels(&["naam"]), nu());
        assert_eq!(r.ontbreekt_in_bron, vec!["naam".to_string()]);
        assert!(r.bevestigd.is_empty());
    }

    #[test]
    fn lege_regels_en_witruimte_worden_overgeslagen() {
        let mut p = profiel();
        p.koppel("achternaam", "naam", nu()).unwrap();
        let r = p.vergelijk(&regels(&["  achternaam  ", "", "   "]), &regels(&["naam"]), nu());
        assert!(!r.heeft_afwijkingen());
        assert_eq!(r.bevestigd, vec!["achternaam".to_string()]);
    }

    #[test]
    fn zonder_vergelijking_is_het_profiel_niet_af() {
        let mut p = profiel();
        p.koppel("achternaam", "naam", nu()).unwrap();
        let velden: Vec<_> = p.ontbrekende_onderdelen().into_iter().map(|o| o.veld).collect();
        assert!(velden.contains(&"mapping.vergelijking".to_string()));

        let r = p.vergelijk(&regels(&["achternaam"]), &regels(&["naam"]), nu());
        p.leg_rapport_vast(r, nu());
        assert!(p.volledigheid().is_volledig());
    }

    /// Afwijkingen blokkeren niet: het register bijwerken is werk, geen fout.
    #[test]
    fn afwijkingen_signaleren_maar_blokkeren_niet() {
        let mut p = profiel();
        p.koppel("achternaam", "naam", nu()).unwrap();
        let r = p.vergelijk(&regels(&["achternaam", "bsn"]), &regels(&["naam"]), nu());
        p.leg_rapport_vast(r, nu());

        let ontbreekt = p.ontbrekende_onderdelen();
        assert_eq!(ontbreekt.len(), 1);
        assert!(!ontbreekt[0].blokkeert_vaststelling);
    }

    #[test]
    fn het_profiel_overleeft_serialisatie() {
        let mut p = profiel();
        p.koppel("achternaam", "naam", nu()).unwrap();
        p.negeer("aangemaakt_op", "technisch veld", nu()).unwrap();
        let r = p.vergelijk(&regels(&["achternaam", "bsn"]), &regels(&["naam"]), nu());
        p.leg_rapport_vast(r, nu());

        let json = serde_json::to_string(&p).unwrap();
        let terug: Mappingprofiel = serde_json::from_str(&json).unwrap();
        assert_eq!(p, terug);
    }
}
