//! De getypeerde termijnsoorten.
//!
//! Het uitgangspunt uit het plan: **een maandtermijn wordt nooit in dagen
//! omgerekend.** Daarom is de eenheid onderdeel van het type en niet een getal
//! met een losse aanduiding ernaast. Wie een termijn van één maand als dertig
//! dagen doorrekent, komt bij een verzoek van 31 januari op 2 maart uit in
//! plaats van op 28 februari — twee dagen te laat, zonder dat iemand het ziet.

use serde::{Deserialize, Serialize};

/// De eenheid waarin een termijn is uitgedrukt.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Eenheid {
    /// Klokuren die doorlopen door nacht, weekend en feestdag heen.
    ///
    /// Grondslag: Verordening (EEG, Euratom) nr. 1182/71, art. 3 lid 1 en 2.
    /// Voorbeelden: de 72-uurstermijn van art. 33 AVG en de meldtermijnen uit
    /// de zorgplichtregelgeving.
    Klokuren,
    /// Aaneengesloten kalenderdagen.
    Kalenderdagen,
    /// Dagen waarop niet wordt gewerkt tellen niet mee.
    Werkdagen,
    /// Weken van zeven kalenderdagen.
    Weken,
    /// Kalendermaanden met maandeindeklem.
    Maanden,
    /// Kalenderjaren.
    Jaren,
}

impl Eenheid {
    /// Geeft aan of deze eenheid in klokuren rekent.
    ///
    /// Urentermijnen worden nooit verlengd wegens een weekend of feestdag.
    pub fn is_urentermijn(&self) -> bool {
        matches!(self, Self::Klokuren)
    }

    pub fn enkelvoud(&self) -> &'static str {
        match self {
            Self::Klokuren => "uur",
            Self::Kalenderdagen => "kalenderdag",
            Self::Werkdagen => "werkdag",
            Self::Weken => "week",
            Self::Maanden => "maand",
            Self::Jaren => "jaar",
        }
    }

    pub fn meervoud(&self) -> &'static str {
        match self {
            Self::Klokuren => "uur",
            Self::Kalenderdagen => "kalenderdagen",
            Self::Werkdagen => "werkdagen",
            Self::Weken => "weken",
            Self::Maanden => "maanden",
            Self::Jaren => "jaar",
        }
    }
}

/// Het rechtsstelsel waaruit de termijn voortvloeit.
///
/// Dit bepaalt welke verlengingsregel geldt en welke bepaling in de
/// verantwoording wordt genoemd.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Rechtsstelsel {
    /// Termijn uit Unierecht. Verordening (EEG, Euratom) nr. 1182/71.
    Unierecht,
    /// Termijn uit nationale wetgeving. Algemene termijnenwet.
    NationaalRecht,
    /// Termijn die de organisatie zichzelf stelt.
    ///
    /// Heeft geen wettelijke verlengingsregel; overschrijding is geen
    /// wetsovertreding maar wel een signaal.
    ZelfGesteld,
}

impl Rechtsstelsel {
    /// De bepaling die de verlengingsregel draagt, voor de verantwoording.
    pub fn verlengingsbepaling(&self) -> &'static str {
        match self {
            Self::Unierecht => "Verordening (EEG, Euratom) nr. 1182/71, art. 3 lid 4",
            Self::NationaalRecht => "Algemene termijnenwet, art. 1",
            Self::ZelfGesteld => "geen wettelijke verlengingsregel; intern vastgesteld",
        }
    }
}

/// Wanneer de termijn begint te lopen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Aanvang {
    /// De termijn loopt vanaf het ankermoment zelf.
    ///
    /// Zo werken de urentermijnen: 72 uur na kennisname betekent 72 uur na dat
    /// exacte tijdstip.
    VanafGebeurtenis,
    /// De termijn loopt vanaf de dag ná het ankermoment.
    ///
    /// Zo werkt onder meer de bezwaartermijn: die vangt aan met ingang van de
    /// dag na die waarop het besluit is bekendgemaakt (Awb art. 6:8).
    VanafDagNaGebeurtenis,
}

/// De verlengingsregel die is toegepast, voor de verantwoording in beeld.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToegepasteVerlenging {
    /// Geen verlenging nodig: de einddatum viel al op een werkdag.
    GeenNodig,
    /// Niet van toepassing: urentermijnen worden nooit verlengd.
    NietVanToepassingBijUren,
    /// De einddatum viel op een zaterdag, zondag of feestdag en is doorgeschoven.
    NaarEerstvolgendeWerkdag { van: String, naar: String },
}

/// De omschrijving van één wettelijke termijn.
///
/// Deze waarden komen uit het kennispakket, niet uit de programmacode.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Termijnsoort {
    /// Vaste code, bijvoorbeeld `AVG-33-MELDING`.
    pub code: String,
    /// Leesbare naam.
    pub naam: String,
    /// Hoeveel eenheden.
    pub duur: u32,
    /// In welke eenheid.
    pub eenheid: Eenheid,
    /// Uit welk rechtsstelsel.
    pub stelsel: Rechtsstelsel,
    /// Vanaf welk moment.
    pub aanvang: Aanvang,
    /// De wettelijke bepaling waarop de termijn zelf berust.
    pub grondslag: String,
    /// Of deze termijn opgeschort mag worden.
    ///
    /// Standaard onwaar: opschorten is de uitzondering en moet per termijn
    /// bewust worden toegestaan. Een 72-uurstermijn opschorten kan niet.
    pub opschortbaar: bool,
    /// Of deze termijn verlengd mag worden, en met hoeveel.
    pub verlenging: Option<Verlengingsrecht>,
}

/// Het recht om een termijn te verlengen, wanneer de wet dat toestaat.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Verlengingsrecht {
    /// Met hoeveel eenheden mag worden verlengd.
    pub duur: u32,
    /// In welke eenheid.
    pub eenheid: Eenheid,
    /// Hoe vaak mag worden verlengd.
    pub aantal_keer: u32,
    /// Binnen welke termijn moet het bericht van verlenging zijn verzonden.
    ///
    /// Bij het inzageverzoek moet de mededeling binnen de oorspronkelijke maand
    /// zijn gedaan; wie dat later doet, verliest het recht. Dat is randgeval
    /// T-12 uit het plan.
    pub bericht_binnen_oorspronkelijke_termijn: bool,
    /// De bepaling waarop het verlengingsrecht berust.
    pub grondslag: String,
}

impl Termijnsoort {
    /// Bouwt een urentermijn: loopt in kalendertijd, nooit verlengd, niet
    /// opschortbaar.
    pub fn uren(
        code: impl Into<String>,
        naam: impl Into<String>,
        uren: u32,
        grondslag: impl Into<String>,
    ) -> Self {
        Self {
            code: code.into(),
            naam: naam.into(),
            duur: uren,
            eenheid: Eenheid::Klokuren,
            stelsel: Rechtsstelsel::Unierecht,
            aanvang: Aanvang::VanafGebeurtenis,
            grondslag: grondslag.into(),
            opschortbaar: false,
            verlenging: None,
        }
    }

    /// Bouwt een kalendertermijn.
    pub fn kalender(
        code: impl Into<String>,
        naam: impl Into<String>,
        duur: u32,
        eenheid: Eenheid,
        stelsel: Rechtsstelsel,
        aanvang: Aanvang,
        grondslag: impl Into<String>,
    ) -> Self {
        Self {
            code: code.into(),
            naam: naam.into(),
            duur,
            eenheid,
            stelsel,
            aanvang,
            grondslag: grondslag.into(),
            opschortbaar: false,
            verlenging: None,
        }
    }

    pub fn opschortbaar(mut self) -> Self {
        self.opschortbaar = true;
        self
    }

    pub fn met_verlenging(mut self, recht: Verlengingsrecht) -> Self {
        self.verlenging = Some(recht);
        self
    }

    /// Leesbare weergave van de duur, bijvoorbeeld "72 uur" of "1 maand".
    pub fn duur_in_woorden(&self) -> String {
        if self.duur == 1 {
            format!("1 {}", self.eenheid.enkelvoud())
        } else {
            format!("{} {}", self.duur, self.eenheid.meervoud())
        }
    }
}
