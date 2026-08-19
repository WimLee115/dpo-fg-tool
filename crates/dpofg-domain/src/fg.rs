//! Het persoonlijke dossier van de functionaris: advies en onafhankelijkheid.
//!
//! # Waarom dit dossier apart staat
//!
//! Artikel 38 lid 3 AVG verbiedt de functionaris te ontslaan of te straffen
//! voor de uitvoering van zijn taken. Die bescherming is waardeloos wanneer
//! het bewijs ervan uitsluitend berust bij degene tegen wie zij is gericht.
//! Daarom staan deze records niet in de kluis van de organisatie maar in een
//! tweede kluisbestand met een eigen wachtwoordzin, dat de functionaris bij
//! het einde van zijn aanstelling meeneemt.
//!
//! In de kluis van de organisatie blijft alleen een hash achter. Daarmee is
//! later aan te tonen *dát* een advies op een bepaald moment bestond, zonder
//! de inhoud prijs te geven.
//!
//! # Wat hierover omstreden is
//!
//! Of deze constructie standhoudt tegenover eigendoms- en archiefaanspraken
//! van de organisatie, is niet vastgesteld. Het plan noemt dat als een eigen
//! risico en wil er een juridische notitie en een modelbepaling voor de
//! aanstellingsovereenkomst bij. Dat voorbehoud staat in de uitvoer van de
//! bedieningsschil en hoort daar te blijven staan: de keuze om dit dossier te
//! voeren is aan de functionaris, en die keuze hoort geïnformeerd te zijn.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    basis::{Compartiment, Herkomst, Id, Motivering, Status},
    error::{DomeinFout, Resultaat},
    volledigheid::{Ontbrekend, Volledig},
};

/// Of de functionaris naar behoren en tijdig is betrokken.
///
/// Artikel 38 lid 1 AVG. Dit is geen bijzaak: te laat betrokken worden is de
/// gebruikelijke manier waarop de rol wordt uitgehold, en het is alleen
/// zichtbaar te maken door het per advies vast te leggen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Tijdigheid {
    Ja,
    Deels,
    Nee,
}

impl Tijdigheid {
    pub fn omschrijving(&self) -> &'static str {
        match self {
            Self::Ja => "naar behoren en tijdig betrokken",
            Self::Deels => "gedeeltelijk betrokken",
            Self::Nee => "niet tijdig betrokken",
        }
    }

    pub fn vraagt_toelichting(&self) -> bool {
        !matches!(self, Self::Ja)
    }
}

/// Wat het bestuur met het advies heeft gedaan.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Reactiestatus {
    Overgenomen,
    Deels,
    Niet,
    GeenReactie,
}

impl Reactiestatus {
    pub fn omschrijving(&self) -> &'static str {
        match self {
            Self::Overgenomen => "overgenomen",
            Self::Deels => "gedeeltelijk overgenomen",
            Self::Niet => "niet overgenomen",
            Self::GeenReactie => "geen reactie ontvangen",
        }
    }

    /// Of deze uitkomst een motivering van het bestuur vraagt.
    ///
    /// Dit is het comply-or-explain-beginsel: een advies naast zich neerleggen
    /// mag, maar dan moet er staan waarom.
    pub fn vraagt_motivering(&self) -> bool {
        matches!(self, Self::Deels | Self::Niet)
    }
}

/// De reactie van het bestuur op een advies.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Bestuursreactie {
    pub status: Reactiestatus,
    pub beslisser: String,
    pub datum: DateTime<Utc>,
    /// Verplicht wanneer het advies niet of deels is overgenomen.
    pub motivering: Option<Motivering>,
}

/// Eén stap in de escalatie van een advies.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Escalatiestap {
    /// Naar wie is opgeschaald: leidinggevende, directie, raad van toezicht.
    pub niveau: String,
    pub datum: DateTime<Utc>,
    /// Wat het heeft opgeleverd. Leeg zolang er niets is teruggekomen.
    pub uitkomst: Option<String>,
}

/// Wanneer een versie van dit record is gespiegeld, en onder welke hash.
///
/// Zonder deze lijst kan een record na een aanvulling niet meer worden
/// aangetoond én is niet te zien dát het ooit is gespiegeld. Die twee gevallen
/// zijn dan niet uit elkaar te houden, en dat is bij bewijs het slechtst
/// denkbare antwoord.
///
/// De hash van een eerdere versie bewijst niet wat er in die versie stond; hij
/// bewijst dat er op dat moment iets was en dat het sindsdien is veranderd.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Spiegeling {
    pub hash: String,
    pub op: DateTime<Utc>,
}

/// Een uitgebracht advies met de reactie erop.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Advies {
    pub id: Id,
    pub kenmerk: String,
    pub status: Status,
    pub compartiment: Compartiment,
    pub herkomst: Herkomst,

    /// Waarover advies is gevraagd of gegeven.
    pub onderwerp: String,
    pub vraagsteller: String,
    pub afdeling: Option<String>,
    pub adviestekst: String,
    pub uitgebracht_aan: String,
    pub uitgebracht_op: DateTime<Utc>,
    pub tijdig_betrokken: Tijdigheid,
    /// Verplicht wanneer de betrokkenheid niet tijdig was.
    pub tijdigheidstoelichting: Option<Motivering>,

    pub bestuursreactie: Option<Bestuursreactie>,
    pub escalatie: Vec<Escalatiestap>,
    /// De momenten waarop een versie van dit record is gespiegeld.
    pub spiegelingen: Vec<Spiegeling>,
}

impl Advies {
    #[allow(clippy::too_many_arguments)]
    pub fn nieuw(
        kenmerk: impl Into<String>,
        onderwerp: impl Into<String>,
        vraagsteller: impl Into<String>,
        adviestekst: impl Into<String>,
        uitgebracht_aan: impl Into<String>,
        uitgebracht_op: DateTime<Utc>,
        tijdig_betrokken: Tijdigheid,
        tijdigheidstoelichting: Option<Motivering>,
        door: impl Into<String>,
        op: DateTime<Utc>,
    ) -> Resultaat<Self> {
        let onderwerp = onderwerp.into();
        let adviestekst = adviestekst.into();
        let uitgebracht_aan = uitgebracht_aan.into();
        if onderwerp.trim().is_empty() || adviestekst.trim().is_empty() {
            return Err(DomeinFout::OngeldigeWaarde {
                veld: "advies.adviestekst".into(),
                reden: "noem het onderwerp en wat er is geadviseerd; een advies dat niet is \
                        opgeschreven, is later niet te onderscheiden van een advies dat niet is \
                        gegeven"
                    .into(),
            });
        }
        if uitgebracht_aan.trim().is_empty() {
            return Err(DomeinFout::OngeldigeWaarde {
                veld: "advies.uitgebracht_aan".into(),
                reden: "noem aan wie het advies is uitgebracht; zonder geadresseerde valt niet \
                        vast te stellen wie het naast zich neer heeft gelegd"
                    .into(),
            });
        }
        if uitgebracht_op > op {
            return Err(DomeinFout::OnmogelijkTijdstip {
                veld: "advies.uitgebracht_op".into(),
                reden: "het advies zou in de toekomst zijn uitgebracht".into(),
            });
        }
        if tijdig_betrokken.vraagt_toelichting() && tijdigheidstoelichting.is_none() {
            return Err(DomeinFout::OngeldigeWaarde {
                veld: "advies.tijdigheidstoelichting".into(),
                reden: "leg vast waaruit blijkt dat u niet tijdig bent betrokken; te laat \
                        betrokken worden is de gebruikelijke manier waarop de rol wordt \
                        uitgehold, en dat is alleen aantoonbaar met de omstandigheden erbij \
                        (art. 38 lid 1 AVG)"
                    .into(),
            });
        }
        Ok(Self {
            id: Id::nieuw(),
            kenmerk: kenmerk.into(),
            status: Status::Concept,
            compartiment: Compartiment::nieuw(Compartiment::FG_PERSOONLIJK),
            herkomst: Herkomst::nieuw(door, op),
            onderwerp: onderwerp.trim().into(),
            vraagsteller: vraagsteller.into(),
            afdeling: None,
            adviestekst: adviestekst.trim().into(),
            uitgebracht_aan: uitgebracht_aan.trim().into(),
            uitgebracht_op,
            tijdig_betrokken,
            tijdigheidstoelichting,
            bestuursreactie: None,
            escalatie: Vec::new(),
            spiegelingen: Vec::new(),
        })
    }

    /// Legt de reactie van het bestuur vast.
    ///
    /// Weigert een afwijzing zonder motivering. Dat is het hele punt van
    /// comply-or-explain: een advies naast zich neerleggen mag, maar dan moet
    /// er staan waarom, en die reden is later het bewijs.
    pub fn leg_reactie_vast(
        &mut self,
        status: Reactiestatus,
        beslisser: impl Into<String>,
        datum: DateTime<Utc>,
        motivering: Option<Motivering>,
        op: DateTime<Utc>,
    ) -> Resultaat<()> {
        let beslisser = beslisser.into();
        if beslisser.trim().is_empty() {
            return Err(DomeinFout::OngeldigeWaarde {
                veld: "advies.bestuursreactie.beslisser".into(),
                reden: "noem wie het besluit heeft genomen".into(),
            });
        }
        if datum > op {
            return Err(DomeinFout::OnmogelijkTijdstip {
                veld: "advies.bestuursreactie.datum".into(),
                reden: "de reactie zou in de toekomst zijn gegeven".into(),
            });
        }
        if datum < self.uitgebracht_op {
            return Err(DomeinFout::OnmogelijkTijdstip {
                veld: "advies.bestuursreactie.datum".into(),
                reden: "de reactie ligt vóór het advies waarop zij ziet".into(),
            });
        }
        if status.vraagt_motivering() && motivering.is_none() {
            return Err(DomeinFout::OngeldigeWaarde {
                veld: "advies.bestuursreactie.motivering".into(),
                reden: "een advies dat niet of deels wordt overgenomen, vraagt een reden van \
                        degene die dat besluit; zonder die reden is er geen verantwoording, \
                        alleen een uitkomst"
                    .into(),
            });
        }
        if status == Reactiestatus::GeenReactie && motivering.is_some() {
            return Err(DomeinFout::OngeldigeWaarde {
                veld: "advies.bestuursreactie".into(),
                reden: "er is geen reactie ontvangen; een motivering erbij zou suggereren dat \
                        het bestuur zich heeft uitgesproken"
                    .into(),
            });
        }
        self.bestuursreactie =
            Some(Bestuursreactie { status, beslisser: beslisser.trim().into(), datum, motivering });
        self.herkomst.wijzig("bestuursreactie vastgelegd", op);
        Ok(())
    }

    /// Legt een escalatiestap vast.
    pub fn escaleer(
        &mut self,
        niveau: impl Into<String>,
        datum: DateTime<Utc>,
        uitkomst: Option<String>,
        op: DateTime<Utc>,
    ) -> Resultaat<()> {
        let niveau = niveau.into();
        if niveau.trim().is_empty() {
            return Err(DomeinFout::OngeldigeWaarde {
                veld: "advies.escalatie.niveau".into(),
                reden: "noem naar wie is opgeschaald".into(),
            });
        }
        if datum > op {
            return Err(DomeinFout::OnmogelijkTijdstip {
                veld: "advies.escalatie.datum".into(),
                reden: "de escalatie zou in de toekomst hebben plaatsgevonden".into(),
            });
        }
        self.escalatie.push(Escalatiestap {
            niveau: niveau.trim().into(),
            datum,
            uitkomst: uitkomst.filter(|u| !u.trim().is_empty()),
        });
        self.escalatie.sort_by_key(|e| e.datum);
        self.herkomst.wijzig("escalatiestap vastgelegd", op);
        Ok(())
    }

    /// Het record zoals het wordt gehasht voor de spiegel.
    ///
    /// Zonder de spiegelingen zelf. Die zijn boekhouding over het spiegelen en
    /// geen inhoud; zouden zij meetellen, dan verandert het record door het
    /// spiegelen en komt het meteen daarna niet meer overeen met de hash die
    /// zojuist is vastgelegd. Dat is precies één keer misgegaan en daarom
    /// staat er een test op.
    pub fn spiegelbaar(&self) -> Self {
        Self { spiegelingen: Vec::new(), ..self.clone() }
    }

    /// Hoeveel dagen het bestuur erover heeft gedaan, of tot nu toe doet.
    pub fn dagen_tot_reactie(&self, nu: DateTime<Utc>) -> i64 {
        let eind = self.bestuursreactie.as_ref().map(|r| r.datum).unwrap_or(nu);
        (eind - self.uitgebracht_op).num_days()
    }

    pub fn stel_vast(&mut self, door: impl Into<String>, op: DateTime<Utc>) -> Resultaat<()> {
        let rapport = self.volledigheid();
        if !rapport.mag_vaststellen() {
            return Err(DomeinFout::NietVolledig {
                soort: "advies".into(),
                ontbreekt: rapport.blokkades().iter().map(|o| o.veld.clone()).collect(),
            });
        }
        self.status = Status::Vastgesteld;
        self.herkomst.stel_vast(door, op);
        Ok(())
    }
}

impl Volledig for Advies {
    fn soortnaam(&self) -> &'static str {
        "advies"
    }

    fn aantal_verplichte_onderdelen(&self) -> usize {
        1
    }

    fn ontbrekende_onderdelen(&self) -> Vec<Ontbrekend> {
        // Onderwerp, adviestekst, geadresseerde en tijdigheid worden door de
        // constructor afgedwongen. Wat overblijft is de reactie, en die komt
        // van een ander: zij blokkeert daarom niet.
        if self.bestuursreactie.is_none() {
            vec![Ontbrekend::signalerend(
                "advies.bestuursreactie",
                "leg vast wat het bestuur met dit advies heeft gedaan, ook wanneer dat niets \
                 is; juist het uitblijven van een reactie is een feit dat later telt",
                "art. 38 lid 3 AVG",
            )]
        } else {
            Vec::new()
        }
    }
}

/// Wat een spiegeling over een record zegt op dit moment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Spiegelstand {
    /// Er is nooit gespiegeld; het tijdstip berust uitsluitend op de eigen
    /// opgave van de functionaris.
    NooitGespiegeld,
    /// De huidige inhoud komt overeen met de laatste spiegeling.
    Sluitend { op: DateTime<Utc> },
    /// Er is gespiegeld, maar de inhoud is sindsdien gewijzigd.
    ///
    /// Dat is geen fout: een advies krijgt een reactie, een gebeurtenis krijgt
    /// opvolging. Het betekent alleen dat er opnieuw moet worden gespiegeld,
    /// en dat de vorige versie niet meer is te tonen.
    Gewijzigd { laatste: DateTime<Utc>, aantal: usize },
}

/// Bepaalt wat de spiegelingen over een record zeggen.
pub fn spiegelstand(spiegelingen: &[Spiegeling], huidige_hash: &str) -> Spiegelstand {
    let Some(laatste) = spiegelingen.last() else {
        return Spiegelstand::NooitGespiegeld;
    };
    if spiegelingen.iter().any(|s| s.hash == huidige_hash) {
        Spiegelstand::Sluitend { op: laatste.op }
    } else {
        Spiegelstand::Gewijzigd { laatste: laatste.op, aantal: spiegelingen.len() }
    }
}

/// Wat de onafhankelijkheid van de functionaris heeft geraakt.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Aantastingsoort {
    /// Er is een instructie gegeven over de uitvoering van de taken.
    InstructieGegeven,
    /// Toegang tot gegevens of ruimten is geweigerd.
    ToegangGeweigerd,
    /// Gevraagde tijd of middelen zijn geweigerd.
    CapaciteitGeweigerd,
    /// Er is een belangenconflict ontstaan.
    Belangenconflict,
    /// Er is met een sanctie gedreigd.
    SanctieGedreigd,
    /// De beoordeling van de functionaris is gekoppeld aan de inhoud van zijn
    /// advies.
    BeoordelingGekoppeld,
}

impl Aantastingsoort {
    pub fn omschrijving(&self) -> &'static str {
        match self {
            Self::InstructieGegeven => "instructie gegeven over de uitvoering van de taken",
            Self::ToegangGeweigerd => "toegang tot gegevens of ruimten geweigerd",
            Self::CapaciteitGeweigerd => "gevraagde tijd of middelen geweigerd",
            Self::Belangenconflict => "belangenconflict",
            Self::SanctieGedreigd => "met een sanctie gedreigd",
            Self::BeoordelingGekoppeld => "beoordeling gekoppeld aan de inhoud van het advies",
        }
    }

    /// De bepaling waaruit de bescherming volgt.
    pub fn grondslag(&self) -> &'static str {
        match self {
            Self::InstructieGegeven => "art. 38 lid 3, eerste volzin AVG",
            Self::ToegangGeweigerd | Self::CapaciteitGeweigerd => "art. 38 lid 2 AVG",
            Self::Belangenconflict => "art. 38 lid 6 AVG",
            Self::SanctieGedreigd | Self::BeoordelingGekoppeld => {
                "art. 38 lid 3, tweede volzin AVG"
            }
        }
    }

    pub fn alle() -> [Self; 6] {
        [
            Self::InstructieGegeven,
            Self::ToegangGeweigerd,
            Self::CapaciteitGeweigerd,
            Self::Belangenconflict,
            Self::SanctieGedreigd,
            Self::BeoordelingGekoppeld,
        ]
    }
}

/// Een gebeurtenis die de onafhankelijkheid van de functionaris raakt.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Onafhankelijkheidsincident {
    pub id: Id,
    pub kenmerk: String,
    pub status: Status,
    pub compartiment: Compartiment,
    pub herkomst: Herkomst,

    pub soort: Aantastingsoort,
    pub datum: DateTime<Utc>,
    pub omschrijving: String,
    /// Wie het betrof, wanneer dat niet de vastlegger zelf is.
    pub betrokken_functionaris: String,
    /// Wie het heeft gedaan of gezegd.
    pub van: String,
    pub opvolging: Option<String>,
    /// De momenten waarop een versie van dit record is gespiegeld.
    pub spiegelingen: Vec<Spiegeling>,
}

impl Onafhankelijkheidsincident {
    #[allow(clippy::too_many_arguments)]
    pub fn nieuw(
        kenmerk: impl Into<String>,
        soort: Aantastingsoort,
        datum: DateTime<Utc>,
        omschrijving: impl Into<String>,
        betrokken_functionaris: impl Into<String>,
        van: impl Into<String>,
        door: impl Into<String>,
        op: DateTime<Utc>,
    ) -> Resultaat<Self> {
        let omschrijving = omschrijving.into();
        let van = van.into();
        if omschrijving.trim().len() < 20 {
            return Err(DomeinFout::OngeldigeWaarde {
                veld: "onafhankelijkheid.omschrijving".into(),
                reden: "beschrijf wat er feitelijk is gebeurd, met de woorden die zijn gebruikt \
                        als u die kent; een aantekening van drie woorden is over twee jaar geen \
                        bewijs meer"
                    .into(),
            });
        }
        if van.trim().is_empty() {
            return Err(DomeinFout::OngeldigeWaarde {
                veld: "onafhankelijkheid.van".into(),
                reden: "noem van wie het kwam; zonder afzender is niet vast te stellen of het \
                        iemand betrof die daartoe in de positie was"
                    .into(),
            });
        }
        if datum > op {
            return Err(DomeinFout::OnmogelijkTijdstip {
                veld: "onafhankelijkheid.datum".into(),
                reden: "de gebeurtenis zou in de toekomst hebben plaatsgevonden".into(),
            });
        }
        Ok(Self {
            id: Id::nieuw(),
            kenmerk: kenmerk.into(),
            status: Status::Concept,
            compartiment: Compartiment::nieuw(Compartiment::FG_PERSOONLIJK),
            herkomst: Herkomst::nieuw(door, op),
            soort,
            datum,
            omschrijving: omschrijving.trim().into(),
            betrokken_functionaris: betrokken_functionaris.into(),
            van: van.trim().into(),
            opvolging: None,
            spiegelingen: Vec::new(),
        })
    }

    /// Het record zoals het wordt gehasht voor de spiegel.
    ///
    /// Zonder de spiegelingen zelf. Die zijn boekhouding over het spiegelen en
    /// geen inhoud; zouden zij meetellen, dan verandert het record door het
    /// spiegelen en komt het meteen daarna niet meer overeen met de hash die
    /// zojuist is vastgelegd. Dat is precies één keer misgegaan en daarom
    /// staat er een test op.
    pub fn spiegelbaar(&self) -> Self {
        Self { spiegelingen: Vec::new(), ..self.clone() }
    }

    /// Legt vast wat er met de gebeurtenis is gedaan.
    pub fn leg_opvolging_vast(
        &mut self,
        opvolging: impl Into<String>,
        op: DateTime<Utc>,
    ) -> Resultaat<()> {
        let opvolging = opvolging.into();
        if opvolging.trim().is_empty() {
            return Err(DomeinFout::OngeldigeWaarde {
                veld: "onafhankelijkheid.opvolging".into(),
                reden: "beschrijf wat u ermee hebt gedaan, ook wanneer dat niets is en waarom"
                    .into(),
            });
        }
        self.opvolging = Some(opvolging.trim().into());
        self.herkomst.wijzig("opvolging vastgelegd", op);
        Ok(())
    }
}

impl Volledig for Onafhankelijkheidsincident {
    fn soortnaam(&self) -> &'static str {
        "onafhankelijkheidsincident"
    }

    fn aantal_verplichte_onderdelen(&self) -> usize {
        1
    }

    fn ontbrekende_onderdelen(&self) -> Vec<Ontbrekend> {
        if self.opvolging.is_none() {
            vec![Ontbrekend::signalerend(
                "onafhankelijkheid.opvolging",
                "leg vast wat u ermee hebt gedaan, ook wanneer dat niets is en waarom",
                "art. 38 lid 3 AVG",
            )]
        } else {
            Vec::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, TimeZone};

    fn nu() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 8, 19, 9, 0, 0).unwrap()
    }

    fn motivering(tekst: &str) -> Motivering {
        Motivering::nieuw(tekst, "fg", nu()).unwrap()
    }

    fn advies() -> Advies {
        Advies::nieuw(
            "ADV-2026-014",
            "de invoering van een aanwezigheidsregistratie",
            "de afdeling bedrijfsvoering",
            "de gekozen opzet verwerkt meer gegevens dan voor het doel nodig is",
            "de directie",
            nu() - Duration::days(30),
            Tijdigheid::Ja,
            None,
            "fg",
            nu(),
        )
        .unwrap()
    }

    /// Een advies dat niet is opgeschreven, is later niet te onderscheiden van
    /// een advies dat niet is gegeven.
    #[test]
    fn een_advies_zonder_tekst_of_geadresseerde_wordt_geweigerd() {
        let fout = Advies::nieuw(
            "ADV-001",
            "iets",
            "iemand",
            "   ",
            "de directie",
            nu(),
            Tijdigheid::Ja,
            None,
            "fg",
            nu(),
        )
        .unwrap_err();
        assert!(fout.to_string().contains("niet is gegeven"), "kreeg: {fout}");

        let fout = Advies::nieuw(
            "ADV-001",
            "iets",
            "iemand",
            "een advies",
            "  ",
            nu(),
            Tijdigheid::Ja,
            None,
            "fg",
            nu(),
        )
        .unwrap_err();
        assert!(fout.to_string().contains("wie het naast zich neer heeft gelegd"), "kreeg: {fout}");
    }

    /// Te laat betrokken worden is de gebruikelijke manier waarop de rol wordt
    /// uitgehold, en dat is alleen aantoonbaar met de omstandigheden erbij.
    #[test]
    fn niet_tijdig_betrokken_vraagt_een_toelichting() {
        let fout = Advies::nieuw(
            "ADV-001",
            "iets",
            "iemand",
            "een advies",
            "de directie",
            nu(),
            Tijdigheid::Nee,
            None,
            "fg",
            nu(),
        )
        .unwrap_err();
        assert!(fout.to_string().contains("art. 38 lid 1 AVG"), "kreeg: {fout}");

        assert!(Advies::nieuw(
            "ADV-001",
            "iets",
            "iemand",
            "een advies",
            "de directie",
            nu(),
            Tijdigheid::Deels,
            Some(motivering("het contract was al getekend toen de vraag kwam")),
            "fg",
            nu(),
        )
        .is_ok());
    }

    /// Comply-or-explain: een advies naast zich neerleggen mag, maar dan moet
    /// er staan waarom.
    #[test]
    fn een_afwijzing_zonder_reden_wordt_geweigerd() {
        let mut a = advies();
        let fout =
            a.leg_reactie_vast(Reactiestatus::Niet, "de directie", nu(), None, nu()).unwrap_err();
        assert!(fout.to_string().contains("geen verantwoording"), "kreeg: {fout}");

        a.leg_reactie_vast(
            Reactiestatus::Niet,
            "de directie",
            nu(),
            Some(motivering("de kosten van een andere opzet wegen niet op tegen het risico")),
            nu(),
        )
        .unwrap();
        assert_eq!(a.bestuursreactie.as_ref().unwrap().status, Reactiestatus::Niet);
    }

    /// Geen reactie is iets anders dan een gemotiveerde afwijzing.
    #[test]
    fn geen_reactie_draagt_geen_motivering() {
        let mut a = advies();
        let fout = a
            .leg_reactie_vast(
                Reactiestatus::GeenReactie,
                "de directie",
                nu(),
                Some(motivering("zij hebben niets van zich laten horen")),
                nu(),
            )
            .unwrap_err();
        assert!(fout.to_string().contains("zich heeft uitgesproken"), "kreeg: {fout}");

        a.leg_reactie_vast(Reactiestatus::GeenReactie, "de directie", nu(), None, nu()).unwrap();
        assert!(a.volledigheid().is_volledig());
    }

    #[test]
    fn een_reactie_van_voor_het_advies_wordt_geweigerd() {
        let mut a = advies();
        let fout = a
            .leg_reactie_vast(
                Reactiestatus::Overgenomen,
                "de directie",
                nu() - Duration::days(60),
                None,
                nu(),
            )
            .unwrap_err();
        assert!(fout.to_string().contains("vóór het advies"), "kreeg: {fout}");
    }

    /// Juist het uitblijven van een reactie is een feit dat later telt; het
    /// blokkeert niet, want het komt van een ander.
    #[test]
    fn een_uitblijvende_reactie_signaleert_maar_blokkeert_niet() {
        let mut a = advies();
        let r = a.volledigheid();
        assert!(!r.is_volledig());
        assert!(r.mag_vaststellen());
        assert_eq!(a.dagen_tot_reactie(nu()), 30);

        a.stel_vast("fg", nu()).unwrap();
        assert_eq!(a.status, Status::Vastgesteld);
    }

    #[test]
    fn escalatiestappen_staan_op_volgorde() {
        let mut a = advies();
        a.escaleer("de raad van toezicht", nu(), Some("nog geen uitkomst".into()), nu()).unwrap();
        a.escaleer("de directie", nu() - Duration::days(10), None, nu()).unwrap();
        assert_eq!(a.escalatie[0].niveau, "de directie");
        assert_eq!(a.escalatie[1].niveau, "de raad van toezicht");
        assert!(a.escalatie[0].uitkomst.is_none());
    }

    #[test]
    fn een_escalatie_in_de_toekomst_wordt_geweigerd() {
        let mut a = advies();
        assert!(a.escaleer("de directie", nu() + Duration::days(1), None, nu()).is_err());
    }

    /// Een aantekening van drie woorden is over twee jaar geen bewijs meer.
    #[test]
    fn een_te_korte_omschrijving_wordt_geweigerd() {
        let fout = Onafhankelijkheidsincident::nieuw(
            "ONA-001",
            Aantastingsoort::InstructieGegeven,
            nu(),
            "gedoe met de baas",
            "A. de Vries",
            "de directeur",
            "fg",
            nu(),
        )
        .unwrap_err();
        assert!(fout.to_string().contains("geen bewijs meer"), "kreeg: {fout}");
    }

    #[test]
    fn elke_soort_draagt_zijn_eigen_grondslag() {
        for soort in Aantastingsoort::alle() {
            assert!(soort.grondslag().contains("art. 38"), "{soort:?}");
        }
        assert_eq!(Aantastingsoort::Belangenconflict.grondslag(), "art. 38 lid 6 AVG");
    }

    #[test]
    fn een_incident_zonder_afzender_wordt_geweigerd() {
        let fout = Onafhankelijkheidsincident::nieuw(
            "ONA-001",
            Aantastingsoort::SanctieGedreigd,
            nu(),
            "er is gezegd dat mijn contract niet zou worden verlengd",
            "A. de Vries",
            "  ",
            "fg",
            nu(),
        )
        .unwrap_err();
        assert!(fout.to_string().contains("daartoe in de positie was"), "kreeg: {fout}");
    }

    #[test]
    fn de_records_staan_in_het_persoonlijke_compartiment() {
        assert_eq!(advies().compartiment.naam(), Compartiment::FG_PERSOONLIJK);
        let i = Onafhankelijkheidsincident::nieuw(
            "ONA-001",
            Aantastingsoort::ToegangGeweigerd,
            nu(),
            "de toegang tot het personeelssysteem is geweigerd door de beheerder",
            "A. de Vries",
            "de systeembeheerder",
            "fg",
            nu(),
        )
        .unwrap();
        assert_eq!(i.compartiment.naam(), Compartiment::FG_PERSOONLIJK);
    }

    /// Nooit gespiegeld en na het spiegelen gewijzigd zijn twee verschillende
    /// antwoorden. Die niet uit elkaar kunnen houden is bij bewijs het
    /// slechtst denkbare resultaat.
    #[test]
    fn de_spiegelstand_scheidt_nooit_van_gewijzigd() {
        assert_eq!(spiegelstand(&[], "abc"), Spiegelstand::NooitGespiegeld);

        let een = vec![Spiegeling { hash: "abc".into(), op: nu() }];
        assert_eq!(spiegelstand(&een, "abc"), Spiegelstand::Sluitend { op: nu() });
        assert_eq!(spiegelstand(&een, "def"), Spiegelstand::Gewijzigd { laatste: nu(), aantal: 1 });

        // Een oudere spiegeling die nog overeenkomt, telt ook.
        let twee = vec![
            Spiegeling { hash: "abc".into(), op: nu() - Duration::days(30) },
            Spiegeling { hash: "def".into(), op: nu() },
        ];
        assert_eq!(spiegelstand(&twee, "abc"), Spiegelstand::Sluitend { op: nu() });
    }

    /// Het spiegelen mag het record niet veranderen. Deed het dat wel, dan
    /// komt het meteen na het spiegelen al niet meer overeen met zijn eigen
    /// hash.
    #[test]
    fn een_spiegeling_verandert_de_gehashte_inhoud_niet() {
        let mut a = advies();
        let voor = serde_json::to_string(&a.spiegelbaar()).unwrap();
        a.spiegelingen.push(Spiegeling { hash: "abc".into(), op: nu() });
        assert_eq!(voor, serde_json::to_string(&a.spiegelbaar()).unwrap());

        // En wél veranderen wanneer de inhoud verandert.
        a.leg_reactie_vast(Reactiestatus::Overgenomen, "de directie", nu(), None, nu()).unwrap();
        assert_ne!(voor, serde_json::to_string(&a.spiegelbaar()).unwrap());
    }

    #[test]
    fn de_records_overleven_serialisatie() {
        let mut a = advies();
        a.escaleer("de directie", nu(), None, nu()).unwrap();
        a.leg_reactie_vast(Reactiestatus::Overgenomen, "de directie", nu(), None, nu()).unwrap();
        let terug: Advies = serde_json::from_str(&serde_json::to_string(&a).unwrap()).unwrap();
        assert_eq!(a, terug);
    }
}
