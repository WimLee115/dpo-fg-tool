//! Redactieregie: bepalen wát er weg moet, en bewijzen dát het weg is.
//!
//! # Wat deze module wel en niet doet
//!
//! Zij redigeert niet. Het bewerken van een tekstlaag, het herkennen van tekst
//! op een scan en het zwart maken van beeld is een zelfstandig product; dat in
//! eigen beheer bouwen zou de meest waarschijnlijke oorzaak van een datalek
//! *door* de tool naar binnen halen. Paragraaf 9.5.1 van het plan legt daarom
//! vast: de tool wijst aan wat er moet worden geredigeerd, levert dat uit aan
//! een aangewezen extern hulpmiddel, en controleert het teruggeleverde bestand.
//!
//! # De terugleescontrole, en waarom zij eerlijk moet zijn
//!
//! De klassieke fout is niet dat er niets gebeurt, maar dat er een zwart vlak
//! over tekst wordt gelegd die in de tekstlaag blijft staan. Wie het bestand
//! opent en de tekst selecteert, leest alles alsnog. Daarom controleert dit
//! dossier drie dingen: de tekstlaag, de metagegevens en annotaties, en het
//! beeld op de geredigeerde plaatsen.
//!
//! Van die drie kan dit programma er precies één zelf: zoeken of de letterlijke
//! waarden die weg moesten nog ergens in de bytes van het bestand staan. Dat
//! vindt de meest gemaakte fout, en het vindt hem **niet** wanneer de tekst in
//! een samengedrukte stroom zit. Die grens wordt niet weggepoetst: een controle
//! die niet is uitgevoerd heet [`Controleuitkomst::NietUitvoerbaar`], telt niet
//! mee als geslaagd, en houdt de verstrekking tegen tot een mens met een tweede
//! persoon vastlegt dat hij het buiten de tool om heeft nagekeken.
//!
//! Dat is invariant I28: **geen verstrekking zonder geslaagde
//! terugleescontrole.**

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    basis::{Compartiment, Herkomst, Id, Status},
    error::{DomeinFout, Resultaat},
    volledigheid::{Ontbrekend, Volledig},
};

/// Wat er in een stuk kan staan dat eruit moet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Redactiecategorie {
    Burgerservicenummer,
    Naam,
    Adres,
    Contactgegevens,
    Gezondheidsgegevens,
    StrafrechtelijkeGegevens,
    FinancieleGegevens,
    Handtekening,
    Beeldmateriaal,
    Overig,
}

impl Redactiecategorie {
    pub fn omschrijving(&self) -> &'static str {
        match self {
            Self::Burgerservicenummer => "burgerservicenummer",
            Self::Naam => "naam van een persoon",
            Self::Adres => "adresgegevens",
            Self::Contactgegevens => "contactgegevens",
            Self::Gezondheidsgegevens => "gezondheidsgegevens",
            Self::StrafrechtelijkeGegevens => "strafrechtelijke gegevens",
            Self::FinancieleGegevens => "financiële gegevens",
            Self::Handtekening => "handtekening",
            Self::Beeldmateriaal => "beeldmateriaal",
            Self::Overig => "overig",
        }
    }

    /// Of deze categorie in de bytes van een bestand te zoeken is.
    ///
    /// Een handtekening en een gezicht staan niet als tekst in het bestand; die
    /// vindt een tekstscan niet, en doen alsof van wel zou de controle
    /// waardeloos maken.
    pub fn is_tekstueel(&self) -> bool {
        !matches!(self, Self::Handtekening | Self::Beeldmateriaal)
    }

    pub fn alle() -> [Self; 10] {
        [
            Self::Burgerservicenummer,
            Self::Naam,
            Self::Adres,
            Self::Contactgegevens,
            Self::Gezondheidsgegevens,
            Self::StrafrechtelijkeGegevens,
            Self::FinancieleGegevens,
            Self::Handtekening,
            Self::Beeldmateriaal,
            Self::Overig,
        ]
    }
}

/// Eén ding dat uit de stukken moet verdwijnen.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TeRedigeren {
    pub categorie: Redactiecategorie,
    /// De letterlijke waarde die weg moet, voor zover die als tekst bestaat.
    ///
    /// Dit is waarop de tekstcontrole zoekt. Bij een handtekening of een foto
    /// blijft dit leeg; daar valt niets op te zoeken.
    pub waarde: Option<String>,
    /// Waar het om gaat, in gewone taal.
    pub omschrijving: String,
}

impl TeRedigeren {
    /// Of hierop machinaal te controleren valt.
    pub fn is_controleerbaar(&self) -> bool {
        self.categorie.is_tekstueel() && self.waarde.as_ref().is_some_and(|w| !w.trim().is_empty())
    }
}

/// Welke controle er is gedaan.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Controlesoort {
    /// Staat de weggehaalde tekst nog in het bestand?
    Tekstlaag,
    /// Staan er nog gegevens in de metagegevens of in annotaties?
    Metagegevens,
    /// Is op de geredigeerde plaatsen werkelijk beeld verwijderd?
    Beeldvergelijking,
    /// Een mens heeft het nagekeken, buiten de tool om.
    Handmatig,
}

impl Controlesoort {
    pub fn omschrijving(&self) -> &'static str {
        match self {
            Self::Tekstlaag => "tekstlaag",
            Self::Metagegevens => "metagegevens en annotaties",
            Self::Beeldvergelijking => "beeldvergelijking",
            Self::Handmatig => "handmatige controle",
        }
    }

    /// Of dit programma deze controle zelf kan uitvoeren.
    pub fn is_machinaal(&self) -> bool {
        matches!(self, Self::Tekstlaag)
    }

    pub fn alle() -> [Self; 4] {
        [Self::Tekstlaag, Self::Metagegevens, Self::Beeldvergelijking, Self::Handmatig]
    }
}

/// De uitkomst van één controle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Controleuitkomst {
    Geslaagd,
    Gefaald,
    /// De controle is niet uitgevoerd, of kon niet worden uitgevoerd.
    ///
    /// Uitdrukkelijk geen derde smaak tussen goed en fout: hij telt als niet
    /// geslaagd. Een controle die niet is gedaan, heeft niets aangetoond.
    NietUitvoerbaar,
}

impl Controleuitkomst {
    pub fn omschrijving(&self) -> &'static str {
        match self {
            Self::Geslaagd => "geslaagd",
            Self::Gefaald => "gefaald",
            Self::NietUitvoerbaar => "niet uitgevoerd",
        }
    }
}

/// Eén uitgevoerde terugleescontrole.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Terugleescontrole {
    pub soort: Controlesoort,
    pub uitkomst: Controleuitkomst,
    pub uitgevoerd_op: DateTime<Utc>,
    pub door: String,
    /// Wat er is aangetroffen. Leeg bij een geslaagde controle.
    pub bevindingen: Vec<String>,
    /// Bij een handmatige controle: wie het heeft bevestigd.
    pub tweede_persoon: Option<String>,
    pub toelichting: Option<String>,
}

/// Eén stuk dat de deur uit zou gaan.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Redactiestuk {
    pub naam: String,
    /// Hash van het stuk zoals het de tool verliet.
    pub hash_origineel: String,
    /// Hash van het stuk zoals het terugkwam.
    pub hash_geredigeerd: Option<String>,
    pub controles: Vec<Terugleescontrole>,
}

impl Redactiestuk {
    /// De uitkomst van één soort controle op dit stuk.
    pub fn uitkomst_van(&self, soort: Controlesoort) -> Controleuitkomst {
        self.controles
            .iter()
            .filter(|c| c.soort == soort)
            .map(|c| c.uitkomst)
            // De laatste telt: een herstelde redactie mag een eerdere afkeuring
            // opheffen, maar een latere afkeuring haalt een eerdere goedkeuring
            // ook weer weg.
            .next_back()
            .unwrap_or(Controleuitkomst::NietUitvoerbaar)
    }

    /// Of er een geldige handmatige bevestiging ligt.
    ///
    /// Vereist een tweede persoon: wie zijn eigen redactie goedkeurt, heeft
    /// niets gecontroleerd wat hij niet al dacht.
    pub fn heeft_handmatige_bevestiging(&self) -> bool {
        self.controles.iter().any(|c| {
            c.soort == Controlesoort::Handmatig
                && c.uitkomst == Controleuitkomst::Geslaagd
                && c.tweede_persoon.as_ref().is_some_and(|p| !p.trim().is_empty())
        })
    }

    /// Wat er aan dit stuk nog ontbreekt voordat het de deur uit mag.
    pub fn beletsels(&self) -> Vec<String> {
        let mut uit = Vec::new();

        let Some(geredigeerd) = &self.hash_geredigeerd else {
            uit.push(format!("'{}' is nog niet teruggeleverd", self.naam));
            return uit;
        };
        if geredigeerd == &self.hash_origineel {
            uit.push(format!(
                "'{}' is byte voor byte gelijk aan het origineel; er is niets geredigeerd",
                self.naam
            ));
        }

        for soort in [
            Controlesoort::Tekstlaag,
            Controlesoort::Metagegevens,
            Controlesoort::Beeldvergelijking,
        ] {
            match self.uitkomst_van(soort) {
                Controleuitkomst::Geslaagd => {}
                Controleuitkomst::Gefaald => uit.push(format!(
                    "'{}': de controle op de {} is gefaald",
                    self.naam,
                    soort.omschrijving()
                )),
                Controleuitkomst::NietUitvoerbaar if self.heeft_handmatige_bevestiging() => {}
                Controleuitkomst::NietUitvoerbaar => uit.push(format!(
                    "'{}': de controle op de {} is niet uitgevoerd. Voer haar uit, of laat een \
                     tweede persoon vastleggen dat hij het buiten de tool om heeft nagekeken",
                    self.naam,
                    soort.omschrijving()
                )),
            }
        }
        uit
    }

    pub fn mag_verstrekt_worden(&self) -> bool {
        self.beletsels().is_empty()
    }
}

/// De redactieregie bij één dossier.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Redactieopdracht {
    pub id: Id,
    pub kenmerk: String,
    pub omschrijving: String,
    pub status: Status,
    pub compartiment: Compartiment,
    pub herkomst: Herkomst,

    /// Het dossier waarvoor wordt geredigeerd: een verzoek of een Woo-verzoek.
    pub dossier_soort: String,
    pub dossier_kenmerk: String,

    /// Wat er weg moet.
    pub profiel: Vec<TeRedigeren>,
    pub stukken: Vec<Redactiestuk>,

    /// Aan welk extern hulpmiddel is uitgeleverd, en wanneer.
    pub hulpmiddel: Option<String>,
    pub uitgeleverd_op: Option<DateTime<Utc>>,
    pub teruggeleverd_op: Option<DateTime<Utc>>,

    /// Wanneer de stukken daadwerkelijk zijn verstrekt.
    pub verstrekt_op: Option<DateTime<Utc>>,
    pub verstrekt_aan: Option<String>,
}

impl Redactieopdracht {
    pub fn nieuw(
        kenmerk: impl Into<String>,
        omschrijving: impl Into<String>,
        dossier_soort: impl Into<String>,
        dossier_kenmerk: impl Into<String>,
        door: impl Into<String>,
        op: DateTime<Utc>,
    ) -> Self {
        Self {
            id: Id::nieuw(),
            kenmerk: kenmerk.into(),
            omschrijving: omschrijving.into(),
            status: Status::Concept,
            compartiment: Compartiment::nieuw(Compartiment::VERTROUWELIJK),
            herkomst: Herkomst::nieuw(door, op),
            dossier_soort: dossier_soort.into(),
            dossier_kenmerk: dossier_kenmerk.into(),
            profiel: Vec::new(),
            stukken: Vec::new(),
            hulpmiddel: None,
            uitgeleverd_op: None,
            teruggeleverd_op: None,
            verstrekt_op: None,
            verstrekt_aan: None,
        }
    }

    /// Voegt iets toe dat weg moet.
    pub fn voeg_toe_aan_profiel(
        &mut self,
        categorie: Redactiecategorie,
        waarde: Option<String>,
        omschrijving: impl Into<String>,
        op: DateTime<Utc>,
    ) -> Resultaat<()> {
        let omschrijving = omschrijving.into();
        if omschrijving.trim().is_empty() {
            return Err(DomeinFout::OngeldigeWaarde {
                veld: "redactie.profiel".into(),
                reden: "schrijf op waar het om gaat; een categorie zonder omschrijving zegt de \
                        controleur niets"
                    .into(),
            });
        }
        if categorie.is_tekstueel() && waarde.as_ref().is_none_or(|w| w.trim().is_empty()) {
            return Err(DomeinFout::OngeldigeWaarde {
                veld: "redactie.profiel".into(),
                reden: format!(
                    "geef de letterlijke waarde die weg moet; zonder die waarde valt er op '{}' \
                     niets te controleren en zou de tool bewaking suggereren die er niet is",
                    categorie.omschrijving()
                ),
            });
        }
        self.profiel.push(TeRedigeren { categorie, waarde, omschrijving });
        self.herkomst.wijzig("profiel aangevuld", op);
        Ok(())
    }

    /// Neemt een stuk op in de opdracht.
    pub fn voeg_stuk_toe(
        &mut self,
        naam: impl Into<String>,
        hash_origineel: impl Into<String>,
        op: DateTime<Utc>,
    ) -> Resultaat<()> {
        let naam = naam.into();
        if self.stukken.iter().any(|s| s.naam == naam) {
            return Err(DomeinFout::OngeldigeWaarde {
                veld: "redactie.stukken".into(),
                reden: format!("'{naam}' staat al in de opdracht"),
            });
        }
        self.stukken.push(Redactiestuk {
            naam,
            hash_origineel: hash_origineel.into(),
            hash_geredigeerd: None,
            controles: Vec::new(),
        });
        self.herkomst.wijzig("stuk toegevoegd", op);
        Ok(())
    }

    /// Legt vast dat de stukken naar het externe hulpmiddel zijn gegaan.
    pub fn lever_uit(
        &mut self,
        hulpmiddel: impl Into<String>,
        moment: DateTime<Utc>,
        op: DateTime<Utc>,
    ) -> Resultaat<()> {
        if moment > op {
            return Err(DomeinFout::OnmogelijkTijdstip {
                veld: "redactie.uitgeleverd_op".into(),
                reden: "de uitlevering zou in de toekomst liggen".into(),
            });
        }
        if self.stukken.is_empty() {
            return Err(DomeinFout::OngeldigeWaarde {
                veld: "redactie.stukken".into(),
                reden: "er zijn geen stukken opgenomen om uit te leveren".into(),
            });
        }
        if self.profiel.is_empty() {
            return Err(DomeinFout::OngeldigeWaarde {
                veld: "redactie.profiel".into(),
                reden: "leg eerst vast wát er weg moet; zonder profiel is er niets aan te wijzen \
                        en straks niets te controleren"
                    .into(),
            });
        }
        self.hulpmiddel = Some(hulpmiddel.into());
        self.uitgeleverd_op = Some(moment);
        self.herkomst.wijzig("uitgeleverd aan het externe hulpmiddel", op);
        Ok(())
    }

    /// Legt het teruggeleverde stuk vast met zijn nieuwe hash.
    pub fn neem_terug(
        &mut self,
        naam: &str,
        hash_geredigeerd: impl Into<String>,
        moment: DateTime<Utc>,
        op: DateTime<Utc>,
    ) -> Resultaat<()> {
        if moment > op {
            return Err(DomeinFout::OnmogelijkTijdstip {
                veld: "redactie.teruggeleverd_op".into(),
                reden: "de teruglevering zou in de toekomst liggen".into(),
            });
        }
        let stuk = self.stukken.iter_mut().find(|s| s.naam == naam).ok_or_else(|| {
            DomeinFout::OntbrekendeVerwijzing {
                veld: "redactie.stukken".into(),
                naar: format!("een stuk '{naam}'"),
            }
        })?;
        stuk.hash_geredigeerd = Some(hash_geredigeerd.into());
        // De controles van een vorige ronde zeggen niets over dit bestand.
        stuk.controles.clear();
        self.teruggeleverd_op = Some(moment);
        self.herkomst.wijzig(format!("stuk {naam} teruggenomen"), op);
        Ok(())
    }

    /// Legt een uitgevoerde controle vast.
    pub fn leg_controle_vast(&mut self, naam: &str, controle: Terugleescontrole) -> Resultaat<()> {
        if controle.soort == Controlesoort::Handmatig
            && controle.uitkomst == Controleuitkomst::Geslaagd
            && controle.tweede_persoon.as_ref().is_none_or(|p| p.trim().is_empty())
        {
            return Err(DomeinFout::TweedePersoonVereist {
                handeling: "handmatige goedkeuring van een redactie".into(),
                reden: "wie zijn eigen redactie goedkeurt, controleert niets wat hij niet al \
                        dacht. Noem de tweede persoon die het heeft nagekeken"
                    .into(),
            });
        }
        let op = controle.uitgevoerd_op;
        let stuk = self.stukken.iter_mut().find(|s| s.naam == naam).ok_or_else(|| {
            DomeinFout::OntbrekendeVerwijzing {
                veld: "redactie.stukken".into(),
                naar: format!("een stuk '{naam}'"),
            }
        })?;
        if stuk.hash_geredigeerd.is_none() {
            return Err(DomeinFout::OngeldigeStatusovergang {
                van: "nog niet teruggeleverd".into(),
                naar: "gecontroleerd".into(),
                reden: format!("'{naam}' is nog niet teruggekomen; er valt niets te controleren"),
            });
        }
        stuk.controles.push(controle);
        self.herkomst.wijzig(format!("controle op {naam} vastgelegd"), op);
        Ok(())
    }

    /// Alles wat verstrekking in de weg staat (invariant I28).
    pub fn beletsels(&self) -> Vec<String> {
        let mut uit = Vec::new();
        if self.stukken.is_empty() {
            uit.push("er zijn geen stukken opgenomen".into());
        }
        for s in &self.stukken {
            uit.extend(s.beletsels());
        }
        uit
    }

    pub fn mag_verstrekken(&self) -> bool {
        self.beletsels().is_empty()
    }

    /// Legt vast dat de stukken zijn verstrekt.
    ///
    /// Invariant I28: dit is de plaats waar de tool verstrekking tegenhoudt
    /// zolang de terugleescontrole niet is geslaagd.
    pub fn verstrek(
        &mut self,
        aan: impl Into<String>,
        moment: DateTime<Utc>,
        op: DateTime<Utc>,
    ) -> Resultaat<()> {
        if moment > op {
            return Err(DomeinFout::OnmogelijkTijdstip {
                veld: "redactie.verstrekt_op".into(),
                reden: "de verstrekking zou in de toekomst liggen".into(),
            });
        }
        let belet = self.beletsels();
        if !belet.is_empty() {
            return Err(DomeinFout::NietVolledig {
                soort: "redactieopdracht".into(),
                ontbreekt: belet,
            });
        }
        self.verstrekt_aan = Some(aan.into());
        self.verstrekt_op = Some(moment);
        self.status = Status::Vastgesteld;
        self.herkomst.stel_vast(self.herkomst.aangemaakt_door.clone(), op);
        Ok(())
    }

    /// De waarden waarop een tekstcontrole kan zoeken.
    pub fn te_zoeken_waarden(&self) -> Vec<(&Redactiecategorie, &str)> {
        self.profiel
            .iter()
            .filter(|p| p.is_controleerbaar())
            .filter_map(|p| p.waarde.as_deref().map(|w| (&p.categorie, w)))
            .collect()
    }
}

impl Volledig for Redactieopdracht {
    fn soortnaam(&self) -> &'static str {
        "redactieopdracht"
    }

    fn aantal_verplichte_onderdelen(&self) -> usize {
        // Het profiel, ten minste één stuk, de uitlevering, en per stuk de drie
        // controles.
        3 + self.stukken.len() * 3
    }

    fn ontbrekende_onderdelen(&self) -> Vec<Ontbrekend> {
        let mut uit = Vec::new();
        if self.profiel.is_empty() {
            uit.push(Ontbrekend::blokkerend(
                "redactie.profiel",
                "leg vast wát er uit de stukken moet verdwijnen",
                "art. 15 lid 4 AVG; de rechten van anderen",
            ));
        }
        if self.stukken.is_empty() {
            uit.push(Ontbrekend::blokkerend(
                "redactie.stukken",
                "neem de stukken op die de deur uit zouden gaan",
                "interne norm",
            ));
        }
        if self.uitgeleverd_op.is_none() {
            uit.push(Ontbrekend::blokkerend(
                "redactie.uitlevering",
                "lever de stukken uit aan het aangewezen redactiehulpmiddel",
                "interne norm; de tool redigeert niet zelf",
            ));
        }
        for beletsel in self.beletsels() {
            uit.push(Ontbrekend::blokkerend(
                "redactie.terugleescontrole",
                beletsel,
                "invariant I28: geen verstrekking zonder geslaagde terugleescontrole",
            ));
        }
        uit
    }
}

/// Doorzoekt ruwe bytes op waarden die weg hadden moeten zijn.
///
/// Dit is de enige controle die het programma zelf kan doen, en zij vindt
/// precies de meest gemaakte fout: een zwart vlak over tekst die in de
/// tekstlaag blijft staan. Wat zij **niet** vindt, is tekst in een
/// samengedrukte stroom; een bestandsformaat dat zijn inhoud comprimeert,
/// verbergt de tekst voor deze zoekactie.
///
/// Die grens hoort bij de uitkomst te worden gemeld en niet weggelaten: een
/// controle die zwijgt over wat zij niet heeft gezien, is erger dan geen
/// controle.
pub fn zoek_in_bytes(bytes: &[u8], waarden: &[(&Redactiecategorie, &str)]) -> Vec<String> {
    let mut uit = Vec::new();
    for (categorie, waarde) in waarden {
        let naald = waarde.as_bytes();
        if naald.is_empty() {
            continue;
        }
        if bytes.windows(naald.len()).any(|w| w == naald) {
            uit.push(format!(
                "'{waarde}' ({}) staat nog leesbaar in het bestand",
                categorie.omschrijving()
            ));
        }
    }
    uit
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn nu() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 8, 19, 9, 0, 0).unwrap()
    }

    fn opdracht() -> Redactieopdracht {
        Redactieopdracht::nieuw(
            "RED-2026-004",
            "inzagebundel",
            "verzoek",
            "VZ-2026-014",
            "u1",
            nu(),
        )
    }

    fn controle(soort: Controlesoort, uitkomst: Controleuitkomst) -> Terugleescontrole {
        Terugleescontrole {
            soort,
            uitkomst,
            uitgevoerd_op: nu(),
            door: "u1".into(),
            bevindingen: Vec::new(),
            tweede_persoon: None,
            toelichting: None,
        }
    }

    fn klaar_voor_controle() -> Redactieopdracht {
        let mut o = opdracht();
        o.voeg_toe_aan_profiel(
            Redactiecategorie::Burgerservicenummer,
            Some("123456782".into()),
            "het bsn van een collega",
            nu(),
        )
        .unwrap();
        o.voeg_stuk_toe("bijlage-1.pdf", "a".repeat(64), nu()).unwrap();
        o.lever_uit("het aangewezen redactiehulpmiddel", nu(), nu()).unwrap();
        o.neem_terug("bijlage-1.pdf", "b".repeat(64), nu(), nu()).unwrap();
        o
    }

    /// Een tekstuele categorie zonder waarde zou bewaking suggereren die er
    /// niet is: er valt dan niets op te zoeken.
    #[test]
    fn een_tekstuele_categorie_vergt_een_waarde() {
        let mut o = opdracht();
        let fout = o
            .voeg_toe_aan_profiel(Redactiecategorie::Naam, None, "de naam van de melder", nu())
            .unwrap_err();
        assert!(fout.to_string().contains("niets te controleren"));
    }

    /// Een handtekening staat niet als tekst in het bestand; daar een waarde
    /// voor eisen zou de gebruiker dwingen iets te verzinnen.
    #[test]
    fn beeld_vergt_geen_waarde() {
        let mut o = opdracht();
        o.voeg_toe_aan_profiel(
            Redactiecategorie::Handtekening,
            None,
            "de handtekening onder de brief",
            nu(),
        )
        .unwrap();
        assert!(o.te_zoeken_waarden().is_empty(), "hierop valt niets machinaal te zoeken");
    }

    #[test]
    fn uitleveren_zonder_profiel_wordt_geweigerd() {
        let mut o = opdracht();
        o.voeg_stuk_toe("bijlage-1.pdf", "a".repeat(64), nu()).unwrap();
        let fout = o.lever_uit("hulpmiddel", nu(), nu()).unwrap_err();
        assert!(
            fout.to_string().contains("niets te controleren")
                || fout.to_string().contains("zonder profiel")
        );
    }

    /// Invariant I28. Zonder controles gaat er niets de deur uit.
    #[test]
    fn verstrekken_zonder_controle_wordt_geweigerd() {
        let mut o = klaar_voor_controle();
        let fout = o.verstrek("de betrokkene", nu(), nu()).unwrap_err();
        let tekst = fout.to_string();
        assert!(tekst.contains("tekstlaag"), "kreeg: {tekst}");
        assert!(tekst.contains("metagegevens"), "kreeg: {tekst}");
        assert!(tekst.contains("beeldvergelijking"), "kreeg: {tekst}");
    }

    #[test]
    fn een_gefaalde_tekstcontrole_houdt_de_verstrekking_tegen() {
        let mut o = klaar_voor_controle();
        for soort in [Controlesoort::Metagegevens, Controlesoort::Beeldvergelijking] {
            o.leg_controle_vast("bijlage-1.pdf", controle(soort, Controleuitkomst::Geslaagd))
                .unwrap();
        }
        o.leg_controle_vast(
            "bijlage-1.pdf",
            controle(Controlesoort::Tekstlaag, Controleuitkomst::Gefaald),
        )
        .unwrap();

        assert!(!o.mag_verstrekken());
        assert!(o
            .verstrek("de betrokkene", nu(), nu())
            .unwrap_err()
            .to_string()
            .contains("gefaald"));
    }

    #[test]
    fn drie_geslaagde_controles_openen_de_verstrekking() {
        let mut o = klaar_voor_controle();
        for soort in [
            Controlesoort::Tekstlaag,
            Controlesoort::Metagegevens,
            Controlesoort::Beeldvergelijking,
        ] {
            o.leg_controle_vast("bijlage-1.pdf", controle(soort, Controleuitkomst::Geslaagd))
                .unwrap();
        }
        assert!(o.mag_verstrekken());
        o.verstrek("de betrokkene", nu(), nu()).unwrap();
        assert_eq!(o.status, Status::Vastgesteld);
    }

    /// Wat de tool niet kan, mag een mens overnemen — maar niet alleen.
    #[test]
    fn een_handmatige_bevestiging_vergt_een_tweede_persoon() {
        let mut o = klaar_voor_controle();
        let fout = o
            .leg_controle_vast(
                "bijlage-1.pdf",
                controle(Controlesoort::Handmatig, Controleuitkomst::Geslaagd),
            )
            .unwrap_err();
        assert!(matches!(fout, DomeinFout::TweedePersoonVereist { .. }));

        let mut met_tweede = controle(Controlesoort::Handmatig, Controleuitkomst::Geslaagd);
        met_tweede.tweede_persoon = Some("B. Jansen".into());
        o.leg_controle_vast("bijlage-1.pdf", met_tweede).unwrap();
        assert!(o.mag_verstrekken(), "de handmatige bevestiging dekt wat de tool niet kon");
    }

    /// Een teruggeleverd bestand dat gelijk is aan het origineel betekent dat
    /// er niets is gebeurd.
    #[test]
    fn een_onveranderd_bestand_valt_op() {
        let mut o = opdracht();
        o.voeg_toe_aan_profiel(
            Redactiecategorie::Burgerservicenummer,
            Some("123456782".into()),
            "het bsn",
            nu(),
        )
        .unwrap();
        o.voeg_stuk_toe("bijlage-1.pdf", "a".repeat(64), nu()).unwrap();
        o.lever_uit("hulpmiddel", nu(), nu()).unwrap();
        o.neem_terug("bijlage-1.pdf", "a".repeat(64), nu(), nu()).unwrap();

        assert!(o.beletsels().iter().any(|b| b.contains("byte voor byte gelijk")));
    }

    /// Een nieuwe teruglevering wist de controles van de vorige ronde: die
    /// zeiden iets over een ánder bestand.
    #[test]
    fn een_nieuwe_teruglevering_wist_de_oude_controles() {
        let mut o = klaar_voor_controle();
        o.leg_controle_vast(
            "bijlage-1.pdf",
            controle(Controlesoort::Tekstlaag, Controleuitkomst::Geslaagd),
        )
        .unwrap();
        o.neem_terug("bijlage-1.pdf", "c".repeat(64), nu(), nu()).unwrap();
        assert_eq!(
            o.stukken[0].uitkomst_van(Controlesoort::Tekstlaag),
            Controleuitkomst::NietUitvoerbaar
        );
    }

    #[test]
    fn controleren_kan_niet_voor_de_teruglevering() {
        let mut o = opdracht();
        o.voeg_stuk_toe("bijlage-1.pdf", "a".repeat(64), nu()).unwrap();
        let fout = o
            .leg_controle_vast(
                "bijlage-1.pdf",
                controle(Controlesoort::Tekstlaag, Controleuitkomst::Geslaagd),
            )
            .unwrap_err();
        assert!(fout.to_string().contains("nog niet teruggekomen"));
    }

    #[test]
    fn de_bytescan_vindt_wat_er_nog_staat() {
        let bsn = Redactiecategorie::Burgerservicenummer;
        let waarden = vec![(&bsn, "123456782")];
        let bevindingen = zoek_in_bytes(b"naam: J. Jansen, bsn: 123456782, einde", &waarden);
        assert_eq!(bevindingen.len(), 1);
        assert!(bevindingen[0].contains("nog leesbaar"));

        assert!(zoek_in_bytes(b"naam: J. Jansen, einde", &waarden).is_empty());
    }

    #[test]
    fn de_opdracht_overleeft_serialisatie() {
        let o = klaar_voor_controle();
        let json = serde_json::to_string(&o).unwrap();
        let terug: Redactieopdracht = serde_json::from_str(&json).unwrap();
        assert_eq!(o, terug);
    }
}
