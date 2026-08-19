//! De correctieplicht: wat er met een bevinding gebeurt nadat zij is gezien.
//!
//! # Waarom dit record bestaat
//!
//! De controleronde rekent elke keer opnieuw. Een bevinding die vandaag
//! aanslaat, slaat morgen weer aan, en er is tot nu toe geen plaats waar het
//! besluit erover blijft staan. Daarmee zijn er maar twee uitkomsten: iemand
//! lost het op, of iedereen leert de melding weg te kijken. De tweede is de
//! gebruikelijke.
//!
//! Een correctie is dat ontbrekende besluit. Zij zegt wie er iets aan doet,
//! wanneer het klaar is en wat er gebeurt — of, wanneer de regel dat toestaat,
//! dat er tot een bepaalde datum bewust van wordt afgeweken en waarom.
//!
//! # Waarom een afwijking altijd een einddatum heeft
//!
//! Een afwijking zonder einddatum wordt de nieuwe norm. Dat staat al als
//! waarschuwing in de regelmotor en het is hier afgedwongen: `uiterlijk` is
//! geen `Option`. Wie langer wil afwijken, legt dat opnieuw vast, en die
//! herhaling is zichtbaar.
//!
//! # Waarom de tool niet bepaalt of afwijken mag
//!
//! Of van een regel mag worden afgeweken, staat in de regelcatalogus en niet
//! in dit record. De constructor krijgt het antwoord mee van de aanroeper; het
//! domein rekent niet met de catalogus, en de catalogus kent dit record niet.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    basis::{Compartiment, Herkomst, Id, Motivering, Status},
    error::{DomeinFout, Resultaat},
    volledigheid::{Ontbrekend, Volledig},
};

/// Waar de correctie op slaat.
///
/// Drie velden samen wijzen één bevinding aan, en die aanwijzing overleeft een
/// nieuwe controleronde: de ronde rekent opnieuw, deze sleutel niet.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Bevindingsleutel {
    pub regelcode: String,
    pub record_soort: String,
    pub record_kenmerk: String,
}

impl Bevindingsleutel {
    pub fn nieuw(
        regelcode: impl Into<String>,
        record_soort: impl Into<String>,
        record_kenmerk: impl Into<String>,
    ) -> Self {
        Self {
            regelcode: regelcode.into(),
            record_soort: record_soort.into(),
            record_kenmerk: record_kenmerk.into(),
        }
    }

    pub fn aanduiding(&self) -> String {
        format!("{} op {} {}", self.regelcode, self.record_soort, self.record_kenmerk)
    }
}

/// Wat er met de bevinding gaat gebeuren.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Correctiesoort {
    /// De tekortkoming wordt weggenomen, uiterlijk op de afgesproken datum.
    Herstel,
    /// Er wordt bewust van afgeweken, tot de afgesproken datum.
    Afwijking,
}

impl Correctiesoort {
    pub fn omschrijving(&self) -> &'static str {
        match self {
            Self::Herstel => "herstel",
            Self::Afwijking => "gemotiveerde afwijking",
        }
    }
}

/// Hoe een correctie is afgerond.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Afronding {
    pub op: DateTime<Utc>,
    pub door: String,
    pub motivering: Motivering,
}

/// Het besluit over één bevinding.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Correctie {
    pub id: Id,
    pub kenmerk: String,
    pub status: Status,
    pub compartiment: Compartiment,
    pub herkomst: Herkomst,

    pub bevinding: Bevindingsleutel,
    /// Wat de bevinding zei toen dit besluit werd genomen.
    ///
    /// Wordt bewaard omdat een regel kan worden aangescherpt: dan gaat de
    /// correctie over iets anders dan er nu staat, en dat hoort te zien te
    /// zijn.
    pub bevindingstekst: String,
    pub soort: Correctiesoort,
    pub eigenaar_rol: String,
    pub eigenaar_persoon: String,
    /// Wanneer het klaar is, of tot wanneer wordt afgeweken.
    ///
    /// Geen `Option`: een correctie zonder datum is een voornemen, en een
    /// afwijking zonder einddatum wordt de nieuwe norm.
    pub uiterlijk: DateTime<Utc>,
    pub aanpak: Motivering,
    pub afronding: Option<Afronding>,
}

impl Correctie {
    /// Legt een correctie vast.
    ///
    /// `afwijking_toegestaan` komt van de aanroeper en niet uit dit record:
    /// of van een regel mag worden afgeweken, staat in de regelcatalogus.
    #[allow(clippy::too_many_arguments)]
    pub fn nieuw(
        kenmerk: impl Into<String>,
        bevinding: Bevindingsleutel,
        bevindingstekst: impl Into<String>,
        soort: Correctiesoort,
        afwijking_toegestaan: bool,
        eigenaar_rol: impl Into<String>,
        eigenaar_persoon: impl Into<String>,
        uiterlijk: DateTime<Utc>,
        aanpak: Motivering,
        door: impl Into<String>,
        op: DateTime<Utc>,
    ) -> Resultaat<Self> {
        let eigenaar_rol = eigenaar_rol.into();
        let eigenaar_persoon = eigenaar_persoon.into();
        if eigenaar_rol.trim().is_empty() || eigenaar_persoon.trim().is_empty() {
            return Err(DomeinFout::OngeldigeWaarde {
                veld: "correctie.eigenaar".into(),
                reden: "noem zowel de rol als de bezetting; een correctie zonder eigenaar is \
                        een voornemen dat vanzelf verdwijnt"
                    .into(),
            });
        }
        if uiterlijk <= op {
            return Err(DomeinFout::OnmogelijkTijdstip {
                veld: "correctie.uiterlijk".into(),
                reden: "de einddatum ligt niet in de toekomst; een correctie die vandaag al te \
                        laat is, is geen afspraak maar een constatering"
                    .into(),
            });
        }
        if soort == Correctiesoort::Afwijking && !afwijking_toegestaan {
            return Err(DomeinFout::OngeldigeWaarde {
                veld: "correctie.soort".into(),
                reden: format!(
                    "van regel {} mag niet gemotiveerd worden afgeweken; deze bevinding is \
                     alleen weg te nemen door de tekortkoming op te lossen",
                    bevinding.regelcode
                ),
            });
        }
        Ok(Self {
            id: Id::nieuw(),
            kenmerk: kenmerk.into(),
            status: Status::Concept,
            compartiment: Compartiment::algemeen(),
            herkomst: Herkomst::nieuw(door, op),
            bevinding,
            bevindingstekst: bevindingstekst.into(),
            soort,
            eigenaar_rol: eigenaar_rol.trim().into(),
            eigenaar_persoon: eigenaar_persoon.trim().into(),
            uiterlijk,
            aanpak,
            afronding: None,
        })
    }

    pub fn is_afgerond(&self) -> bool {
        self.afronding.is_some()
    }

    /// Of de afgesproken datum is verstreken zonder afronding.
    pub fn is_te_laat(&self, nu: DateTime<Utc>) -> bool {
        !self.is_afgerond() && self.uiterlijk <= nu
    }

    /// Of deze correctie op dit moment loopt.
    pub fn loopt(&self, nu: DateTime<Utc>) -> bool {
        !self.is_afgerond() && self.uiterlijk > nu
    }

    /// Of deze correctie de bevinding op dit moment onderdrukt.
    ///
    /// Alleen een lopende afwijking doet dat. Een herstelafspraak onderdrukt
    /// niets: zolang de tekortkoming er is, hoort zij in beeld te blijven, ook
    /// als er iemand aan werkt.
    pub fn onderdrukt(&self, nu: DateTime<Utc>) -> bool {
        self.soort == Correctiesoort::Afwijking && self.loopt(nu)
    }

    pub fn dagen_tot_uiterlijk(&self, nu: DateTime<Utc>) -> i64 {
        (self.uiterlijk - nu).num_days()
    }

    /// Verlengt de afgesproken datum.
    ///
    /// Vraagt een eigen motivering: uitstel is een besluit en geen
    /// administratieve handeling, en een reeks verlengingen is zelf het
    /// signaal dat de afspraak niet werkt.
    pub fn verleng(
        &mut self,
        nieuwe_datum: DateTime<Utc>,
        motivering: Motivering,
        op: DateTime<Utc>,
    ) -> Resultaat<()> {
        if self.is_afgerond() {
            return Err(DomeinFout::OngeldigeStatusovergang {
                van: "afgerond".into(),
                naar: "verlengd".into(),
                reden: "een afgeronde correctie verlengen zou de afronding stilzwijgend \
                        terugdraaien"
                    .into(),
            });
        }
        if nieuwe_datum <= self.uiterlijk {
            return Err(DomeinFout::OnmogelijkTijdstip {
                veld: "correctie.uiterlijk".into(),
                reden: "de nieuwe datum ligt niet ná de huidige; verlengen is iets anders dan \
                        vervroegen"
                    .into(),
            });
        }
        self.uiterlijk = nieuwe_datum;
        self.aanpak = motivering;
        self.herkomst.wijzig("termijn verlengd", op);
        Ok(())
    }

    /// Rondt de correctie af.
    pub fn rond_af(
        &mut self,
        door: impl Into<String>,
        motivering: Motivering,
        op: DateTime<Utc>,
    ) -> Resultaat<()> {
        if self.is_afgerond() {
            return Err(DomeinFout::OngeldigeStatusovergang {
                van: "afgerond".into(),
                naar: "afgerond".into(),
                reden: "deze correctie is al afgerond".into(),
            });
        }
        let door = door.into();
        self.afronding = Some(Afronding { op, door: door.clone(), motivering });
        self.status = Status::Vastgesteld;
        self.herkomst.stel_vast(door, op);
        Ok(())
    }
}

impl Volledig for Correctie {
    fn soortnaam(&self) -> &'static str {
        "correctie"
    }

    fn aantal_verplichte_onderdelen(&self) -> usize {
        1
    }

    fn ontbrekende_onderdelen(&self) -> Vec<Ontbrekend> {
        // Eigenaar, datum en aanpak worden door de constructor afgedwongen;
        // wat overblijft is de afronding.
        if self.is_afgerond() {
            Vec::new()
        } else {
            vec![Ontbrekend::blokkerend(
                "correctie.afronding",
                format!(
                    "rond deze correctie af of verleng de termijn; de afspraak loopt tot {}",
                    self.uiterlijk.format("%d-%m-%Y")
                ),
                "interne norm; de correctieplicht volgt uit de verantwoordingsplicht",
            )]
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
        Motivering::nieuw(tekst, "u1", nu()).unwrap()
    }

    fn sleutel() -> Bevindingsleutel {
        Bevindingsleutel::nieuw("ZRP-04", "zorgplicht", "ZRP-2026")
    }

    fn correctie(soort: Correctiesoort, toegestaan: bool) -> Resultaat<Correctie> {
        Correctie::nieuw(
            "COR-001",
            sleutel(),
            "3 maatregelen zijn ingericht zonder bewijs van de uitvoering",
            soort,
            toegestaan,
            "de security officer",
            "J. Jansen",
            nu() + Duration::days(60),
            motivering("de uitdraaien worden bij de volgende kwartaalcontrole aangeleverd"),
            "u1",
            nu(),
        )
    }

    #[test]
    fn een_correctie_zonder_eigenaar_wordt_geweigerd() {
        let fout = Correctie::nieuw(
            "COR-001",
            sleutel(),
            "iets",
            Correctiesoort::Herstel,
            false,
            "  ",
            "J. Jansen",
            nu() + Duration::days(60),
            motivering("wij pakken dit op"),
            "u1",
            nu(),
        )
        .unwrap_err();
        assert!(fout.to_string().contains("voornemen dat vanzelf verdwijnt"), "kreeg: {fout}");
    }

    /// Een correctie die vandaag al te laat is, is geen afspraak.
    #[test]
    fn een_datum_in_het_verleden_wordt_geweigerd() {
        let fout = Correctie::nieuw(
            "COR-001",
            sleutel(),
            "iets",
            Correctiesoort::Herstel,
            false,
            "de security officer",
            "J. Jansen",
            nu() - Duration::days(1),
            motivering("wij pakken dit op"),
            "u1",
            nu(),
        )
        .unwrap_err();
        assert!(fout.to_string().contains("geen afspraak maar een constatering"), "kreeg: {fout}");
    }

    /// Of afwijken mag, staat in de regelcatalogus en niet in dit record.
    #[test]
    fn afwijken_kan_alleen_waar_de_regel_dat_toestaat() {
        let fout = correctie(Correctiesoort::Afwijking, false).unwrap_err();
        assert!(
            fout.to_string().contains("mag niet gemotiveerd worden afgeweken"),
            "kreeg: {fout}"
        );
        assert!(correctie(Correctiesoort::Afwijking, true).is_ok());
    }

    /// Een herstelafspraak onderdrukt niets: zolang de tekortkoming er is,
    /// hoort zij in beeld te blijven, ook als er iemand aan werkt.
    #[test]
    fn alleen_een_lopende_afwijking_onderdrukt_de_bevinding() {
        let herstel = correctie(Correctiesoort::Herstel, false).unwrap();
        assert!(!herstel.onderdrukt(nu()));

        let afwijking = correctie(Correctiesoort::Afwijking, true).unwrap();
        assert!(afwijking.onderdrukt(nu()));
        assert!(!afwijking.onderdrukt(nu() + Duration::days(90)));
    }

    #[test]
    fn een_verstreken_termijn_is_te_bepalen() {
        let c = correctie(Correctiesoort::Herstel, false).unwrap();
        assert!(!c.is_te_laat(nu()));
        assert!(c.loopt(nu()));
        assert!(c.is_te_laat(nu() + Duration::days(61)));
        assert!(!c.loopt(nu() + Duration::days(61)));
    }

    #[test]
    fn verlengen_vraagt_een_datum_die_verder_ligt() {
        let mut c = correctie(Correctiesoort::Herstel, false).unwrap();
        assert!(c
            .verleng(nu() + Duration::days(30), motivering("het duurt korter"), nu())
            .is_err());
        c.verleng(
            nu() + Duration::days(120),
            motivering("de leverancier levert de uitdraai pas in het volgende kwartaal"),
            nu(),
        )
        .unwrap();
        assert_eq!(c.dagen_tot_uiterlijk(nu()), 120);
    }

    #[test]
    fn een_afgeronde_correctie_is_niet_te_verlengen_en_niet_opnieuw_af_te_ronden() {
        let mut c = correctie(Correctiesoort::Herstel, false).unwrap();
        c.rond_af("A. de Vries", motivering("de uitdraaien zijn aangeleverd"), nu()).unwrap();
        assert!(c.is_afgerond());
        assert_eq!(c.status, Status::Vastgesteld);

        assert!(c.verleng(nu() + Duration::days(200), motivering("toch nog even"), nu()).is_err());
        assert!(c.rond_af("A. de Vries", motivering("nogmaals afgerond"), nu()).is_err());
    }

    #[test]
    fn een_openstaande_correctie_is_niet_volledig() {
        let mut c = correctie(Correctiesoort::Herstel, false).unwrap();
        assert!(!c.volledigheid().is_volledig());
        c.rond_af("A. de Vries", motivering("de uitdraaien zijn aangeleverd"), nu()).unwrap();
        assert!(c.volledigheid().is_volledig());
    }

    #[test]
    fn de_correctie_overleeft_serialisatie() {
        let c = correctie(Correctiesoort::Afwijking, true).unwrap();
        let json = serde_json::to_string(&c).unwrap();
        let terug: Correctie = serde_json::from_str(&json).unwrap();
        assert_eq!(c, terug);
    }
}
