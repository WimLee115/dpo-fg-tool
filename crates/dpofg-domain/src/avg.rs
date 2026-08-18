//! Begrippen uit de Algemene verordening gegevensbescherming.
//!
//! Deze opsommingen zijn gesloten en volgen de tekst van de verordening. Dat is
//! een bewuste keuze uit het foutbestendigheidshoofdstuk, paragraaf 3.2: geen
//! vrij tekstveld waar een beheerde lijst kan. Een grondslag die als vrije
//! tekst wordt ingetikt, is niet doorzoekbaar, niet te tellen en niet te
//! controleren — en "gerechtvaardigd belang" laat zich op zeven manieren
//! spellen.

use serde::{Deserialize, Serialize};

/// De rol waarin de organisatie verwerkt.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Rol {
    /// Bepaalt doel en middelen: art. 4 onder 7 AVG.
    Verwerkingsverantwoordelijke,
    /// Verwerkt ten behoeve van een ander: art. 4 onder 8 AVG.
    Verwerker,
    /// Bepaalt doel en middelen samen met een ander: art. 26 AVG.
    GezamenlijkVerantwoordelijke,
}

impl Rol {
    /// Welk registerschema van toepassing is.
    pub fn registerschema(&self) -> &'static str {
        match self {
            Self::Verwerkingsverantwoordelijke | Self::GezamenlijkVerantwoordelijke => {
                "art. 30 lid 1 AVG"
            }
            Self::Verwerker => "art. 30 lid 2 AVG",
        }
    }

    pub fn omschrijving(&self) -> &'static str {
        match self {
            Self::Verwerkingsverantwoordelijke => "verwerkingsverantwoordelijke",
            Self::Verwerker => "verwerker",
            Self::GezamenlijkVerantwoordelijke => "gezamenlijk verwerkingsverantwoordelijke",
        }
    }
}

/// De grondslagen van artikel 6 lid 1 AVG.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Grondslag {
    /// a) toestemming van de betrokkene
    Toestemming,
    /// b) noodzakelijk voor de uitvoering van een overeenkomst
    Overeenkomst,
    /// c) noodzakelijk om te voldoen aan een wettelijke verplichting
    WettelijkeVerplichting,
    /// d) noodzakelijk ter bescherming van vitale belangen
    VitaalBelang,
    /// e) noodzakelijk voor een taak van algemeen belang of openbaar gezag
    AlgemeenBelang,
    /// f) noodzakelijk voor de behartiging van een gerechtvaardigd belang
    GerechtvaardigdBelang,
}

impl Grondslag {
    pub fn letter(&self) -> &'static str {
        match self {
            Self::Toestemming => "a",
            Self::Overeenkomst => "b",
            Self::WettelijkeVerplichting => "c",
            Self::VitaalBelang => "d",
            Self::AlgemeenBelang => "e",
            Self::GerechtvaardigdBelang => "f",
        }
    }

    pub fn grondslagverwijzing(&self) -> String {
        format!("art. 6 lid 1 onder {} AVG", self.letter())
    }

    pub fn omschrijving(&self) -> &'static str {
        match self {
            Self::Toestemming => "toestemming van de betrokkene",
            Self::Overeenkomst => "noodzakelijk voor de uitvoering van een overeenkomst",
            Self::WettelijkeVerplichting => {
                "noodzakelijk om te voldoen aan een wettelijke verplichting"
            }
            Self::VitaalBelang => "noodzakelijk ter bescherming van vitale belangen",
            Self::AlgemeenBelang => {
                "noodzakelijk voor een taak van algemeen belang of openbaar gezag"
            }
            Self::GerechtvaardigdBelang => {
                "noodzakelijk voor de behartiging van een gerechtvaardigd belang"
            }
        }
    }

    /// Of deze grondslag een belangenafweging vereist.
    ///
    /// Alleen bij het gerechtvaardigd belang. Zonder afweging is de verwerking
    /// onvolledig; de afweging ís de grondslag.
    pub fn vereist_belangenafweging(&self) -> bool {
        matches!(self, Self::GerechtvaardigdBelang)
    }

    /// Of deze grondslag een bewijs van toestemming vereist.
    ///
    /// De bewijslast ligt bij de verwerkingsverantwoordelijke: art. 7 lid 1 AVG.
    pub fn vereist_toestemmingsbewijs(&self) -> bool {
        matches!(self, Self::Toestemming)
    }

    /// Of deze grondslag een aanwijsbare wettelijke bepaling vereist.
    pub fn vereist_wettelijke_bepaling(&self) -> bool {
        matches!(self, Self::WettelijkeVerplichting | Self::AlgemeenBelang)
    }

    /// Of de betrokkene bij deze grondslag bezwaar kan maken (art. 21 AVG).
    pub fn kent_bezwaarrecht(&self) -> bool {
        matches!(self, Self::AlgemeenBelang | Self::GerechtvaardigdBelang)
    }

    /// Of de betrokkene recht heeft op overdraagbaarheid (art. 20 AVG).
    ///
    /// Alleen bij toestemming of overeenkomst, én bij geautomatiseerde
    /// verwerking. Dit tweede deel volgt uit de verwerking zelf.
    pub fn kent_overdraagbaarheid(&self) -> bool {
        matches!(self, Self::Toestemming | Self::Overeenkomst)
    }

    pub fn alle() -> [Self; 6] {
        [
            Self::Toestemming,
            Self::Overeenkomst,
            Self::WettelijkeVerplichting,
            Self::VitaalBelang,
            Self::AlgemeenBelang,
            Self::GerechtvaardigdBelang,
        ]
    }
}

/// De bijzondere categorieën van artikel 9 lid 1 AVG.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BijzondereCategorie {
    RasOfEtnischeAfkomst,
    PolitiekeOpvattingen,
    ReligieuzeOfLevensbeschouwelijkeOvertuigingen,
    Vakbondslidmaatschap,
    GenetischeGegevens,
    BiometrischeGegevensVoorIdentificatie,
    Gezondheidsgegevens,
    SeksueelGedragOfGerichtheid,
}

impl BijzondereCategorie {
    pub fn omschrijving(&self) -> &'static str {
        match self {
            Self::RasOfEtnischeAfkomst => "ras of etnische afkomst",
            Self::PolitiekeOpvattingen => "politieke opvattingen",
            Self::ReligieuzeOfLevensbeschouwelijkeOvertuigingen => {
                "religieuze of levensbeschouwelijke overtuigingen"
            }
            Self::Vakbondslidmaatschap => "lidmaatschap van een vakbond",
            Self::GenetischeGegevens => "genetische gegevens",
            Self::BiometrischeGegevensVoorIdentificatie => {
                "biometrische gegevens met het oog op unieke identificatie"
            }
            Self::Gezondheidsgegevens => "gegevens over gezondheid",
            Self::SeksueelGedragOfGerichtheid => {
                "gegevens over seksueel gedrag of seksuele gerichtheid"
            }
        }
    }

    pub fn alle() -> [Self; 8] {
        [
            Self::RasOfEtnischeAfkomst,
            Self::PolitiekeOpvattingen,
            Self::ReligieuzeOfLevensbeschouwelijkeOvertuigingen,
            Self::Vakbondslidmaatschap,
            Self::GenetischeGegevens,
            Self::BiometrischeGegevensVoorIdentificatie,
            Self::Gezondheidsgegevens,
            Self::SeksueelGedragOfGerichtheid,
        ]
    }
}

/// De uitzonderingen van artikel 9 lid 2 AVG.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UitzonderingArtikel9 {
    /// a) uitdrukkelijke toestemming
    UitdrukkelijkeToestemming,
    /// b) arbeidsrecht en sociale zekerheid
    ArbeidsrechtEnSocialeZekerheid,
    /// c) vitale belangen, betrokkene kan geen toestemming geven
    VitaleBelangen,
    /// d) gerechtvaardigde activiteiten van een stichting of vereniging
    StichtingOfVereniging,
    /// e) door de betrokkene kennelijk openbaar gemaakt
    KennelijkOpenbaarGemaakt,
    /// f) instelling, uitoefening of onderbouwing van een rechtsvordering
    Rechtsvordering,
    /// g) zwaarwegend algemeen belang
    ZwaarwegendAlgemeenBelang,
    /// h) preventieve of arbeidsgeneeskunde, medische diagnose, zorg
    Gezondheidszorg,
    /// i) algemeen belang op het gebied van de volksgezondheid
    Volksgezondheid,
    /// j) archivering, wetenschappelijk of historisch onderzoek, statistiek
    ArchiveringOnderzoekStatistiek,
}

impl UitzonderingArtikel9 {
    pub fn letter(&self) -> &'static str {
        match self {
            Self::UitdrukkelijkeToestemming => "a",
            Self::ArbeidsrechtEnSocialeZekerheid => "b",
            Self::VitaleBelangen => "c",
            Self::StichtingOfVereniging => "d",
            Self::KennelijkOpenbaarGemaakt => "e",
            Self::Rechtsvordering => "f",
            Self::ZwaarwegendAlgemeenBelang => "g",
            Self::Gezondheidszorg => "h",
            Self::Volksgezondheid => "i",
            Self::ArchiveringOnderzoekStatistiek => "j",
        }
    }

    pub fn grondslagverwijzing(&self) -> String {
        format!("art. 9 lid 2 onder {} AVG", self.letter())
    }

    /// Of deze uitzondering daarnaast een nationale bepaling vereist.
    ///
    /// Verschillende uitzonderingen werken alleen wanneer het nationale recht
    /// erin voorziet. Wie zich op b, g, h, i of j beroept, moet dus ook een
    /// bepaling uit het nationale recht kunnen aanwijzen — in Nederland
    /// doorgaans de UAVG.
    pub fn vereist_nationale_bepaling(&self) -> bool {
        matches!(
            self,
            Self::ArbeidsrechtEnSocialeZekerheid
                | Self::ZwaarwegendAlgemeenBelang
                | Self::Gezondheidszorg
                | Self::Volksgezondheid
                | Self::ArchiveringOnderzoekStatistiek
        )
    }
}

/// De rechten van de betrokkene, artikelen 15 tot en met 22 AVG.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Betrokkenenrecht {
    /// art. 15: inzage
    Inzage,
    /// art. 16: rectificatie
    Rectificatie,
    /// art. 17: gegevenswissing
    Wissing,
    /// art. 18: beperking van de verwerking
    Beperking,
    /// art. 20: overdraagbaarheid
    Overdraagbaarheid,
    /// art. 21: bezwaar
    Bezwaar,
    /// art. 22: niet onderworpen worden aan uitsluitend geautomatiseerde besluitvorming
    GeautomatiseerdeBesluitvorming,
}

impl Betrokkenenrecht {
    pub fn artikel(&self) -> &'static str {
        match self {
            Self::Inzage => "art. 15 AVG",
            Self::Rectificatie => "art. 16 AVG",
            Self::Wissing => "art. 17 AVG",
            Self::Beperking => "art. 18 AVG",
            Self::Overdraagbaarheid => "art. 20 AVG",
            Self::Bezwaar => "art. 21 AVG",
            Self::GeautomatiseerdeBesluitvorming => "art. 22 AVG",
        }
    }

    pub fn omschrijving(&self) -> &'static str {
        match self {
            Self::Inzage => "inzage",
            Self::Rectificatie => "rectificatie",
            Self::Wissing => "gegevenswissing",
            Self::Beperking => "beperking van de verwerking",
            Self::Overdraagbaarheid => "overdraagbaarheid van gegevens",
            Self::Bezwaar => "bezwaar",
            Self::GeautomatiseerdeBesluitvorming => "geautomatiseerde besluitvorming",
        }
    }

    /// Of honorering van dit recht leidt tot een kennisgevingsplicht aan
    /// ontvangers (art. 19 AVG).
    ///
    /// Dit is randgeval T-32 uit het plan: bij vier ontvangers ontstaan vier
    /// verplichtingen, en het verzoek is pas af te sluiten als alle vier een
    /// uitkomst hebben.
    pub fn leidt_tot_kennisgeving_art19(&self) -> bool {
        matches!(self, Self::Rectificatie | Self::Wissing | Self::Beperking)
    }

    pub fn alle() -> [Self; 7] {
        [
            Self::Inzage,
            Self::Rectificatie,
            Self::Wissing,
            Self::Beperking,
            Self::Overdraagbaarheid,
            Self::Bezwaar,
            Self::GeautomatiseerdeBesluitvorming,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grondslagverwijzingen_kloppen() {
        assert_eq!(Grondslag::Toestemming.grondslagverwijzing(), "art. 6 lid 1 onder a AVG");
        assert_eq!(
            Grondslag::GerechtvaardigdBelang.grondslagverwijzing(),
            "art. 6 lid 1 onder f AVG"
        );
    }

    #[test]
    fn alleen_gerechtvaardigd_belang_vereist_een_afweging() {
        for g in Grondslag::alle() {
            assert_eq!(
                g.vereist_belangenafweging(),
                g == Grondslag::GerechtvaardigdBelang,
                "{g:?}"
            );
        }
    }

    #[test]
    fn alleen_toestemming_vereist_bewijs() {
        for g in Grondslag::alle() {
            assert_eq!(g.vereist_toestemmingsbewijs(), g == Grondslag::Toestemming, "{g:?}");
        }
    }

    #[test]
    fn bezwaarrecht_hoort_bij_de_juiste_grondslagen() {
        assert!(Grondslag::AlgemeenBelang.kent_bezwaarrecht());
        assert!(Grondslag::GerechtvaardigdBelang.kent_bezwaarrecht());
        assert!(!Grondslag::Toestemming.kent_bezwaarrecht());
        assert!(!Grondslag::WettelijkeVerplichting.kent_bezwaarrecht());
    }

    #[test]
    fn overdraagbaarheid_hoort_bij_de_juiste_grondslagen() {
        assert!(Grondslag::Toestemming.kent_overdraagbaarheid());
        assert!(Grondslag::Overeenkomst.kent_overdraagbaarheid());
        assert!(!Grondslag::AlgemeenBelang.kent_overdraagbaarheid());
    }

    #[test]
    fn registerschema_volgt_de_rol() {
        assert_eq!(Rol::Verwerkingsverantwoordelijke.registerschema(), "art. 30 lid 1 AVG");
        assert_eq!(Rol::GezamenlijkVerantwoordelijke.registerschema(), "art. 30 lid 1 AVG");
        assert_eq!(Rol::Verwerker.registerschema(), "art. 30 lid 2 AVG");
    }

    #[test]
    fn uitzonderingen_die_nationaal_recht_vereisen() {
        assert!(UitzonderingArtikel9::Gezondheidszorg.vereist_nationale_bepaling());
        assert!(UitzonderingArtikel9::ZwaarwegendAlgemeenBelang.vereist_nationale_bepaling());
        assert!(!UitzonderingArtikel9::UitdrukkelijkeToestemming.vereist_nationale_bepaling());
        assert!(!UitzonderingArtikel9::Rechtsvordering.vereist_nationale_bepaling());
    }

    #[test]
    fn kennisgevingsplicht_bij_de_juiste_rechten() {
        let met: Vec<_> = Betrokkenenrecht::alle()
            .into_iter()
            .filter(|r| r.leidt_tot_kennisgeving_art19())
            .collect();
        assert_eq!(
            met,
            vec![
                Betrokkenenrecht::Rectificatie,
                Betrokkenenrecht::Wissing,
                Betrokkenenrecht::Beperking
            ]
        );
    }

    #[test]
    fn alle_opsommingen_zijn_serialiseerbaar() {
        for g in Grondslag::alle() {
            let json = serde_json::to_string(&g).unwrap();
            let terug: Grondslag = serde_json::from_str(&json).unwrap();
            assert_eq!(g, terug);
        }
        for c in BijzondereCategorie::alle() {
            let json = serde_json::to_string(&c).unwrap();
            assert_eq!(serde_json::from_str::<BijzondereCategorie>(&json).unwrap(), c);
        }
    }
}
