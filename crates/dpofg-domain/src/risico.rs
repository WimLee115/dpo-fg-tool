//! De risicobeoordeling als zelfstandig artefact.
//!
//! # Waarom dit een eigen dossier is
//!
//! De zorgplicht vraagt passende maatregelen, en of maatregelen passend zijn,
//! is alleen te beoordelen tegen de risico's die zijn onderkend. Zonder het
//! beoordelingsartefact — met methode, reikwijdte, uitvoerder, datum en de
//! aanvaarding van wat er overblijft — is de zorgplicht niet aantoonbaar; er
//! ligt dan alleen een lijst maatregelen zonder de vraag waarop zij het
//! antwoord zijn.
//!
//! Het zorgplichtdossier verwijst hiernaar en draagt de beoordeling niet zelf.
//! Twee plaatsen waar dezelfde beoordeling staat, lopen uit elkaar.
//!
//! # Waarom het niveau geen veld is
//!
//! Waarschijnlijkheid en impact worden ingeschat; de klasse volgt daaruit. Er
//! is geen route om een klasse rechtstreeks te zetten, want dat is de plaats
//! waar een beoordeling in een wens verandert: eerst het antwoord kiezen, dan
//! de inschattingen erbij zoeken.
//!
//! Er komt bewust geen getal uit. Een risicoscore van 12 op 25 suggereert een
//! nauwkeurigheid die twee schattingen op een vijfpuntsschaal niet hebben, en
//! zo een getal gaat in een bestuursstuk een eigen leven leiden.
//!
//! # Waarom het restrisico niet vanzelf lager wordt
//!
//! Een restrisico dat lager is ingeschat dan het brutorisico, zonder dat er
//! één maatregel bij staat, is geen beoordeling maar een aanname. Dat wordt
//! geweigerd. En wat er overblijft, aanvaardt de organisatie zelf: de tool
//! dwingt af dát het gebeurt, met naam en functie erbij, en laat de afweging
//! aan wie haar mag maken.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    basis::{Compartiment, Herkomst, Id, Motivering, Status},
    error::{DomeinFout, Resultaat},
    volledigheid::{Ontbrekend, Volledig},
};

/// Een inschatting op een vijfpuntsschaal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Inschatting {
    ZeerLaag,
    Laag,
    Gemiddeld,
    Hoog,
    ZeerHoog,
}

impl Inschatting {
    pub fn omschrijving(&self) -> &'static str {
        match self {
            Self::ZeerLaag => "zeer laag",
            Self::Laag => "laag",
            Self::Gemiddeld => "gemiddeld",
            Self::Hoog => "hoog",
            Self::ZeerHoog => "zeer hoog",
        }
    }

    pub fn alle() -> [Self; 5] {
        [Self::ZeerLaag, Self::Laag, Self::Gemiddeld, Self::Hoog, Self::ZeerHoog]
    }
}

/// De grofste indeling die uit twee inschattingen volgt.
///
/// Deze klasse dient precies één doel: bepalen wie een restrisico mag
/// aanvaarden. Zij is geen score, geen kleur en geen maat voor hoe erg iets
/// is; wie het risico wil kennen, leest de twee inschattingen en de
/// omschrijving.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Risicoklasse {
    Laag,
    Gemiddeld,
    Hoog,
}

impl Risicoklasse {
    /// Leidt de klasse af uit waarschijnlijkheid en impact.
    ///
    /// Bewust grof en zonder rekensom. Een vermenigvuldiging van twee
    /// ordinale schalen levert een getal op dat rekenkundig niets betekent
    /// maar wel als nauwkeurig wordt gelezen.
    pub fn bepaal(waarschijnlijkheid: Inschatting, impact: Inschatting) -> Self {
        use Inschatting::*;
        match (waarschijnlijkheid, impact) {
            (ZeerHoog, _) | (_, ZeerHoog) => Self::Hoog,
            (Hoog, Hoog) => Self::Hoog,
            (ZeerLaag | Laag, ZeerLaag | Laag) => Self::Laag,
            _ => Self::Gemiddeld,
        }
    }

    pub fn omschrijving(&self) -> &'static str {
        match self {
            Self::Laag => "laag",
            Self::Gemiddeld => "gemiddeld",
            Self::Hoog => "hoog",
        }
    }

    pub fn is_hoog(&self) -> bool {
        matches!(self, Self::Hoog)
    }
}

/// Een bron die bij de beoordeling is geraadpleegd.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Bron {
    pub aanduiding: String,
    pub soort: String,
    pub geraadpleegd_op: DateTime<Utc>,
}

/// De aanvaarding van wat er na de maatregelen overblijft.
///
/// Draagt naam én functie. Een handtekening zonder functie is later niet te
/// herleiden tot de vraag of degene die tekende dat ook mocht.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Restrisicoaanvaarding {
    pub door: String,
    pub functie: String,
    /// Of degene die aanvaardt tot het bestuur behoort.
    pub is_bestuurder: bool,
    pub op: DateTime<Utc>,
    pub motivering: Motivering,
}

/// Eén onderkend risico met zijn maatregelen en wat er overblijft.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Onderkendrisico {
    pub code: String,
    pub omschrijving: String,
    /// Waardoor het risico zich kan verwezenlijken.
    pub oorzaak: String,
    /// Wat er dan gebeurt.
    pub gevolg: String,
    pub waarschijnlijkheid: Inschatting,
    pub impact: Inschatting,
    /// De maatregelen die het risico verkleinen, als aanduiding.
    ///
    /// Waar een zorgplichtdossier bestaat, is dit de maatregelcode uit dat
    /// kader; de koppeling wordt niet afgedwongen, want een risico kan ook
    /// worden verkleind door iets wat buiten de controlset valt.
    pub maatregelen: Vec<String>,
    pub restwaarschijnlijkheid: Inschatting,
    pub restimpact: Inschatting,
    pub aanvaarding: Option<Restrisicoaanvaarding>,
}

impl Onderkendrisico {
    /// De klasse vóór maatregelen.
    pub fn brutoklasse(&self) -> Risicoklasse {
        Risicoklasse::bepaal(self.waarschijnlijkheid, self.impact)
    }

    /// De klasse die overblijft na de maatregelen.
    pub fn restklasse(&self) -> Risicoklasse {
        Risicoklasse::bepaal(self.restwaarschijnlijkheid, self.restimpact)
    }

    /// Of de maatregelen het risico volgens de inschatting verkleinen.
    pub fn is_verkleind(&self) -> bool {
        self.restwaarschijnlijkheid < self.waarschijnlijkheid || self.restimpact < self.impact
    }

    /// Of dit restrisico aanvaarding door het bestuur vraagt.
    pub fn vraagt_bestuur(&self) -> bool {
        self.restklasse().is_hoog()
    }
}

/// Een risicobeoordeling.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Risicobeoordeling {
    pub id: Id,
    pub kenmerk: String,
    /// Waarover de beoordeling gaat.
    pub reikwijdte: String,
    pub status: Status,
    pub compartiment: Compartiment,
    pub herkomst: Herkomst,

    pub methode: String,
    /// Waar de methode vandaan komt: een norm, een handreiking, eigen werk.
    pub methode_bron: String,
    pub uitgevoerd_door: String,
    pub uitgevoerd_op: DateTime<Utc>,
    /// Tot wanneer de beoordeling geldt. Geen `Option`: een beoordeling
    /// zonder houdbaarheid blijft eeuwig als actueel gelden.
    pub geldig_tot: DateTime<Utc>,

    pub bronnen: Vec<Bron>,
    pub risicos: Vec<Onderkendrisico>,
}

impl Risicobeoordeling {
    #[allow(clippy::too_many_arguments)]
    pub fn nieuw(
        kenmerk: impl Into<String>,
        reikwijdte: impl Into<String>,
        methode: impl Into<String>,
        methode_bron: impl Into<String>,
        uitgevoerd_door: impl Into<String>,
        uitgevoerd_op: DateTime<Utc>,
        geldig_tot: DateTime<Utc>,
        door: impl Into<String>,
        op: DateTime<Utc>,
    ) -> Resultaat<Self> {
        let reikwijdte = reikwijdte.into();
        let methode = methode.into();
        if reikwijdte.trim().is_empty() {
            return Err(DomeinFout::OngeldigeWaarde {
                veld: "risico.reikwijdte".into(),
                reden: "noem waarover deze beoordeling gaat; een beoordeling zonder reikwijdte \
                        laat de vraag open wat er níet is bekeken, en dat is bij een uitvraag \
                        de eerste vraag"
                    .into(),
            });
        }
        if methode.trim().is_empty() {
            return Err(DomeinFout::OngeldigeWaarde {
                veld: "risico.methode".into(),
                reden: "noem de gebruikte methode; zonder methode is een beoordeling niet te \
                        herhalen en niet te toetsen"
                    .into(),
            });
        }
        if uitgevoerd_op > op {
            return Err(DomeinFout::OnmogelijkTijdstip {
                veld: "risico.uitgevoerd_op".into(),
                reden: "de beoordeling zou in de toekomst zijn uitgevoerd".into(),
            });
        }
        if geldig_tot <= uitgevoerd_op {
            return Err(DomeinFout::OnmogelijkTijdstip {
                veld: "risico.geldig_tot".into(),
                reden: "de beoordeling zou verlopen voordat zij is uitgevoerd".into(),
            });
        }
        Ok(Self {
            id: Id::nieuw(),
            kenmerk: kenmerk.into(),
            reikwijdte: reikwijdte.trim().into(),
            status: Status::Concept,
            compartiment: Compartiment::algemeen(),
            herkomst: Herkomst::nieuw(door, op),
            methode: methode.trim().into(),
            methode_bron: methode_bron.into(),
            uitgevoerd_door: uitgevoerd_door.into(),
            uitgevoerd_op,
            geldig_tot,
            bronnen: Vec::new(),
            risicos: Vec::new(),
        })
    }

    pub fn is_verlopen(&self, nu: DateTime<Utc>) -> bool {
        self.geldig_tot <= nu
    }

    pub fn dagen_tot_verval(&self, nu: DateTime<Utc>) -> i64 {
        (self.geldig_tot - nu).num_days()
    }

    pub fn risico(&self, code: &str) -> Option<&Onderkendrisico> {
        self.risicos.iter().find(|r| r.code == code)
    }

    fn risico_mut(&mut self, code: &str) -> Resultaat<&mut Onderkendrisico> {
        self.risicos.iter_mut().find(|r| r.code == code).ok_or_else(|| {
            DomeinFout::OntbrekendeVerwijzing {
                veld: "risico.onderkend".into(),
                naar: format!("risico met code '{code}' in deze beoordeling"),
            }
        })
    }

    /// Voegt een bron toe die bij de beoordeling is geraadpleegd.
    pub fn raadpleeg_bron(
        &mut self,
        aanduiding: impl Into<String>,
        soort: impl Into<String>,
        geraadpleegd_op: DateTime<Utc>,
        op: DateTime<Utc>,
    ) -> Resultaat<()> {
        let aanduiding = aanduiding.into();
        if aanduiding.trim().is_empty() {
            return Err(DomeinFout::OngeldigeWaarde {
                veld: "risico.bron".into(),
                reden: "noem de bron zo dat een ander hem kan terugvinden".into(),
            });
        }
        if geraadpleegd_op > op {
            return Err(DomeinFout::OnmogelijkTijdstip {
                veld: "risico.bron.geraadpleegd_op".into(),
                reden: "de bron zou in de toekomst zijn geraadpleegd".into(),
            });
        }
        if self.bronnen.iter().any(|b| b.aanduiding == aanduiding.trim()) {
            return Err(DomeinFout::OngeldigeWaarde {
                veld: "risico.bron".into(),
                reden: format!("'{}' staat al in de lijst", aanduiding.trim()),
            });
        }
        self.bronnen.push(Bron {
            aanduiding: aanduiding.trim().into(),
            soort: soort.into(),
            geraadpleegd_op,
        });
        self.herkomst.wijzig("bron geraadpleegd", op);
        Ok(())
    }

    /// Onderkent een risico.
    ///
    /// Het restrisico wordt bij het onderkennen gelijkgesteld aan het
    /// brutorisico. Lager wordt het alleen met `verklein`, en die vraagt een
    /// maatregel.
    #[allow(clippy::too_many_arguments)]
    pub fn onderken(
        &mut self,
        code: impl Into<String>,
        omschrijving: impl Into<String>,
        oorzaak: impl Into<String>,
        gevolg: impl Into<String>,
        waarschijnlijkheid: Inschatting,
        impact: Inschatting,
        op: DateTime<Utc>,
    ) -> Resultaat<()> {
        let code = code.into();
        let omschrijving = omschrijving.into();
        let oorzaak = oorzaak.into();
        let gevolg = gevolg.into();
        if code.trim().is_empty() {
            return Err(DomeinFout::OngeldigeWaarde {
                veld: "risico.code".into(),
                reden: "geef het risico een code, zodat maatregelen en besluiten ernaar kunnen \
                        verwijzen"
                    .into(),
            });
        }
        if self.risicos.iter().any(|r| r.code == code.trim()) {
            return Err(DomeinFout::OngeldigeWaarde {
                veld: "risico.code".into(),
                reden: format!(
                    "code '{}' komt al voor; twee risico's met dezelfde code zijn niet uit \
                     elkaar te houden",
                    code.trim()
                ),
            });
        }
        if oorzaak.trim().is_empty() || gevolg.trim().is_empty() {
            return Err(DomeinFout::OngeldigeWaarde {
                veld: "risico.oorzaak".into(),
                reden: "noem waardoor het risico zich kan verwezenlijken en wat er dan gebeurt; \
                        zonder die twee is er geen maatregel bij te bedenken en niets te meten"
                    .into(),
            });
        }
        self.risicos.push(Onderkendrisico {
            code: code.trim().into(),
            omschrijving,
            oorzaak: oorzaak.trim().into(),
            gevolg: gevolg.trim().into(),
            waarschijnlijkheid,
            impact,
            maatregelen: Vec::new(),
            restwaarschijnlijkheid: waarschijnlijkheid,
            restimpact: impact,
            aanvaarding: None,
        });
        self.herkomst.wijzig(format!("risico {} onderkend", code.trim()), op);
        Ok(())
    }

    /// Verkleint een risico met een of meer maatregelen.
    ///
    /// Weigert een verlaging zonder maatregel, en weigert een restrisico dat
    /// hoger uitkomt dan het brutorisico. Dat eerste is de stilste manier om
    /// een dossier op orde te krijgen: de inschatting bijstellen in plaats van
    /// het risico aanpakken.
    pub fn verklein(
        &mut self,
        code: &str,
        maatregelen: Vec<String>,
        restwaarschijnlijkheid: Inschatting,
        restimpact: Inschatting,
        op: DateTime<Utc>,
    ) -> Resultaat<()> {
        let maatregelen: Vec<String> = maatregelen
            .into_iter()
            .map(|m| m.trim().to_string())
            .filter(|m| !m.is_empty())
            .collect();
        let r = self.risico_mut(code)?;
        if restwaarschijnlijkheid > r.waarschijnlijkheid || restimpact > r.impact {
            return Err(DomeinFout::OngeldigeWaarde {
                veld: "risico.restrisico".into(),
                reden: "het restrisico komt hoger uit dan het risico zonder maatregelen; \
                        maatregelen maken een risico niet groter. Stel de inschatting vooraf \
                        bij als die te laag was"
                    .into(),
            });
        }
        let verlaagt = restwaarschijnlijkheid < r.waarschijnlijkheid || restimpact < r.impact;
        if verlaagt && maatregelen.is_empty() {
            return Err(DomeinFout::OngeldigeWaarde {
                veld: "risico.maatregelen".into(),
                reden: "een restrisico dat lager is dan het risico zonder maatregelen, vraagt \
                        ten minste één maatregel. Zonder maatregel is de verlaging geen \
                        beoordeling maar een aanname"
                    .into(),
            });
        }
        let was_aanvaard = r.aanvaarding.is_some();
        r.maatregelen = maatregelen;
        r.restwaarschijnlijkheid = restwaarschijnlijkheid;
        r.restimpact = restimpact;
        // Een aanvaarding gaat over een bepaald restrisico. Verandert dat, dan
        // is de aanvaarding niet meer waarover is besloten.
        r.aanvaarding = None;
        self.herkomst.wijzig(format!("risico {code} verkleind"), op);
        if was_aanvaard {
            self.herkomst.wijzig(
                format!("aanvaarding van {code} vervallen: het restrisico is gewijzigd"),
                op,
            );
        }
        Ok(())
    }

    /// Legt vast dat iemand het restrisico aanvaardt.
    ///
    /// Een hoog restrisico kan alleen door het bestuur worden aanvaard. Dat is
    /// geen tooloordeel maar een verdeling van bevoegdheid: wie de gevolgen
    /// draagt, neemt het besluit.
    pub fn aanvaard_restrisico(
        &mut self,
        code: &str,
        door: impl Into<String>,
        functie: impl Into<String>,
        is_bestuurder: bool,
        motivering: Motivering,
        op: DateTime<Utc>,
    ) -> Resultaat<()> {
        let door = door.into();
        let functie = functie.into();
        if door.trim().is_empty() || functie.trim().is_empty() {
            return Err(DomeinFout::OngeldigeWaarde {
                veld: "risico.aanvaarding".into(),
                reden: "noem naam én functie; een handtekening zonder functie is later niet te \
                        herleiden tot de vraag of degene die tekende dat ook mocht"
                    .into(),
            });
        }
        let r = self.risico_mut(code)?;
        if r.vraagt_bestuur() && !is_bestuurder {
            return Err(DomeinFout::OngeldigeWaarde {
                veld: "risico.aanvaarding".into(),
                reden: format!(
                    "het restrisico van {code} is hoog; dat aanvaardt het bestuur en niet \
                     iemand anders namens het bestuur"
                ),
            });
        }
        r.aanvaarding = Some(Restrisicoaanvaarding {
            door: door.trim().into(),
            functie: functie.trim().into(),
            is_bestuurder,
            op,
            motivering,
        });
        self.herkomst.wijzig(format!("restrisico van {code} aanvaard"), op);
        Ok(())
    }

    /// De risico's waarvan het restrisico nog niet is aanvaard.
    pub fn onaanvaard(&self) -> Vec<&Onderkendrisico> {
        self.risicos.iter().filter(|r| r.aanvaarding.is_none()).collect()
    }

    /// De risico's waarvoor geen enkele maatregel is genoemd.
    pub fn zonder_maatregel(&self) -> Vec<&Onderkendrisico> {
        self.risicos.iter().filter(|r| r.maatregelen.is_empty()).collect()
    }

    /// Hoeveel risico's er per restklasse zijn.
    pub fn restklassen(&self) -> Vec<(Risicoklasse, usize)> {
        let mut uit = Vec::new();
        for klasse in [Risicoklasse::Hoog, Risicoklasse::Gemiddeld, Risicoklasse::Laag] {
            let aantal = self.risicos.iter().filter(|r| r.restklasse() == klasse).count();
            if aantal > 0 {
                uit.push((klasse, aantal));
            }
        }
        uit
    }

    pub fn stel_vast(&mut self, door: impl Into<String>, op: DateTime<Utc>) -> Resultaat<()> {
        let rapport = self.volledigheid();
        if !rapport.mag_vaststellen() {
            return Err(DomeinFout::NietVolledig {
                soort: "risicobeoordeling".into(),
                ontbreekt: rapport.blokkades().iter().map(|o| o.veld.clone()).collect(),
            });
        }
        self.status = Status::Vastgesteld;
        self.herkomst.stel_vast(door, op);
        Ok(())
    }
}

impl Volledig for Risicobeoordeling {
    fn soortnaam(&self) -> &'static str {
        "risicobeoordeling"
    }

    fn aantal_verplichte_onderdelen(&self) -> usize {
        // Ten minste één risico, ten minste één bron, en per risico een
        // maatregel en een aanvaarding.
        2 + self.risicos.len() * 2
    }

    fn ontbrekende_onderdelen(&self) -> Vec<Ontbrekend> {
        let mut uit = Vec::new();

        if self.risicos.is_empty() {
            uit.push(Ontbrekend::blokkerend(
                "risico.risicos",
                "onderken ten minste één risico; een beoordeling zonder risico's is geen \
                 beoordeling maar een verklaring dat er niets aan de hand is",
                "art. 21 lid 1 Cyberbeveiligingswet",
            ));
        }
        if self.bronnen.is_empty() {
            uit.push(Ontbrekend::signalerend(
                "risico.bronnen",
                "leg vast wat er is geraadpleegd; een beoordeling die alleen op het eigen beeld \
                 berust, ziet wat de organisatie al wist",
                "interne norm; methodische onderbouwing",
            ));
        }

        for r in &self.risicos {
            if r.maatregelen.is_empty() {
                uit.push(Ontbrekend::signalerend(
                    format!("risico.{}.maatregelen", r.code),
                    format!(
                        "noem welke maatregelen {} verkleinen, of leg vast dat er geen zijn en \
                         het restrisico daarmee gelijk is aan het risico",
                        r.code
                    ),
                    "art. 21 lid 1 Cyberbeveiligingswet",
                ));
            }
            if r.aanvaarding.is_none() {
                let veld = format!("risico.{}.aanvaarding", r.code);
                let tekst = format!(
                    "laat vastleggen wie het restrisico van {} aanvaardt, met naam, functie en \
                     onderbouwing",
                    r.code
                );
                if r.vraagt_bestuur() {
                    uit.push(Ontbrekend::blokkerend(
                        veld,
                        format!("{tekst}; bij een hoog restrisico is dat het bestuur"),
                        "art. 24 lid 1 Cyberbeveiligingswet",
                    ));
                } else {
                    uit.push(Ontbrekend::signalerend(veld, tekst, "interne norm"));
                }
            }
        }

        uit
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

    fn beoordeling() -> Risicobeoordeling {
        Risicobeoordeling::nieuw(
            "RIS-2026",
            "de netwerk- en informatiesystemen van de hele organisatie",
            "scenarioanalyse met een vijfpuntsschaal",
            "eigen methodebeschrijving, versie 2",
            "de security officer",
            nu() - Duration::days(30),
            nu() + Duration::days(300),
            "u1",
            nu(),
        )
        .unwrap()
    }

    fn met_risico() -> Risicobeoordeling {
        let mut b = beoordeling();
        b.onderken(
            "R-01",
            "uitval van het rekencentrum",
            "een langdurige stroomstoring",
            "de dienstverlening ligt meer dan een dag stil",
            Inschatting::Gemiddeld,
            Inschatting::Hoog,
            nu(),
        )
        .unwrap();
        b
    }

    /// Zonder reikwijdte blijft de vraag open wat er níet is bekeken.
    #[test]
    fn een_beoordeling_zonder_reikwijdte_of_methode_wordt_geweigerd() {
        let fout = Risicobeoordeling::nieuw(
            "RIS-2026",
            "  ",
            "scenarioanalyse",
            "eigen",
            "de security officer",
            nu() - Duration::days(30),
            nu() + Duration::days(300),
            "u1",
            nu(),
        )
        .unwrap_err();
        assert!(fout.to_string().contains("wat er níet is bekeken"), "kreeg: {fout}");

        let fout = Risicobeoordeling::nieuw(
            "RIS-2026",
            "de hele organisatie",
            "  ",
            "eigen",
            "de security officer",
            nu() - Duration::days(30),
            nu() + Duration::days(300),
            "u1",
            nu(),
        )
        .unwrap_err();
        assert!(fout.to_string().contains("niet te herhalen"), "kreeg: {fout}");
    }

    #[test]
    fn een_beoordeling_die_verloopt_voor_zij_is_uitgevoerd_wordt_geweigerd() {
        let fout = Risicobeoordeling::nieuw(
            "RIS-2026",
            "de hele organisatie",
            "scenarioanalyse",
            "eigen",
            "de security officer",
            nu() - Duration::days(30),
            nu() - Duration::days(60),
            "u1",
            nu(),
        )
        .unwrap_err();
        assert!(fout.to_string().contains("verlopen voordat"), "kreeg: {fout}");
    }

    /// De klasse volgt uit de twee inschattingen en is nergens te zetten.
    #[test]
    fn de_klasse_volgt_uit_de_inschattingen() {
        use Inschatting::*;
        assert_eq!(Risicoklasse::bepaal(Laag, Laag), Risicoklasse::Laag);
        assert_eq!(Risicoklasse::bepaal(Gemiddeld, Hoog), Risicoklasse::Gemiddeld);
        assert_eq!(Risicoklasse::bepaal(Hoog, Hoog), Risicoklasse::Hoog);
        assert_eq!(Risicoklasse::bepaal(ZeerLaag, ZeerHoog), Risicoklasse::Hoog);
        assert_eq!(Risicoklasse::bepaal(ZeerHoog, ZeerLaag), Risicoklasse::Hoog);
    }

    /// Bij het onderkennen is het restrisico gelijk aan het risico. Lager
    /// wordt het alleen met een maatregel erbij.
    #[test]
    fn een_vers_onderkend_risico_is_nog_niet_verkleind() {
        let b = met_risico();
        let r = b.risico("R-01").unwrap();
        assert_eq!(r.brutoklasse(), r.restklasse());
        assert!(!r.is_verkleind());
        assert!(r.maatregelen.is_empty());
    }

    /// De stilste manier om een dossier op orde te krijgen: de inschatting
    /// bijstellen in plaats van het risico aanpakken.
    #[test]
    fn verlagen_zonder_maatregel_wordt_geweigerd() {
        let mut b = met_risico();
        let fout =
            b.verklein("R-01", vec![], Inschatting::Laag, Inschatting::Laag, nu()).unwrap_err();
        assert!(fout.to_string().contains("geen beoordeling maar een aanname"), "kreeg: {fout}");

        b.verklein("R-01", vec!["CBB-09".into()], Inschatting::Laag, Inschatting::Gemiddeld, nu())
            .unwrap();
        let r = b.risico("R-01").unwrap();
        assert!(r.is_verkleind());
        assert_eq!(r.restklasse(), Risicoklasse::Gemiddeld);
    }

    #[test]
    fn een_restrisico_hoger_dan_het_risico_wordt_geweigerd() {
        let mut b = met_risico();
        let fout = b
            .verklein("R-01", vec!["CBB-09".into()], Inschatting::ZeerHoog, Inschatting::Hoog, nu())
            .unwrap_err();
        assert!(fout.to_string().contains("maken een risico niet groter"), "kreeg: {fout}");
    }

    /// Gelijk houden mag zonder maatregel: dat is een eerlijk antwoord.
    #[test]
    fn gelijk_houden_mag_zonder_maatregel() {
        let mut b = met_risico();
        b.verklein("R-01", vec![], Inschatting::Gemiddeld, Inschatting::Hoog, nu()).unwrap();
        assert!(!b.risico("R-01").unwrap().is_verkleind());
    }

    /// Wie de gevolgen draagt, neemt het besluit.
    #[test]
    fn een_hoog_restrisico_aanvaardt_het_bestuur() {
        let mut b = beoordeling();
        b.onderken(
            "R-02",
            "gijzelsoftware",
            "een besmetting via een bijlage",
            "de gegevens zijn versleuteld en niet te herstellen",
            Inschatting::Hoog,
            Inschatting::ZeerHoog,
            nu(),
        )
        .unwrap();
        assert!(b.risico("R-02").unwrap().vraagt_bestuur());

        let fout = b
            .aanvaard_restrisico(
                "R-02",
                "J. Jansen",
                "de security officer",
                false,
                motivering("verdere maatregelen zijn niet haalbaar binnen het budget"),
                nu(),
            )
            .unwrap_err();
        assert!(fout.to_string().contains("dat aanvaardt het bestuur"), "kreeg: {fout}");

        b.aanvaard_restrisico(
            "R-02",
            "P. de Boer",
            "bestuurder",
            true,
            motivering("verdere maatregelen zijn niet haalbaar binnen het budget"),
            nu(),
        )
        .unwrap();
        assert!(b.risico("R-02").unwrap().aanvaarding.is_some());
    }

    #[test]
    fn een_aanvaarding_zonder_functie_wordt_geweigerd() {
        let mut b = met_risico();
        let fout = b
            .aanvaard_restrisico(
                "R-01",
                "J. Jansen",
                "  ",
                false,
                motivering("dit is aanvaardbaar gelet op de omvang"),
                nu(),
            )
            .unwrap_err();
        assert!(fout.to_string().contains("naam én functie"), "kreeg: {fout}");
    }

    /// Een aanvaarding gaat over een bepaald restrisico. Verandert dat, dan is
    /// zij niet meer waarover is besloten.
    #[test]
    fn een_gewijzigd_restrisico_laat_de_aanvaarding_vervallen() {
        let mut b = met_risico();
        b.aanvaard_restrisico(
            "R-01",
            "J. Jansen",
            "de security officer",
            false,
            motivering("dit is aanvaardbaar gelet op de omvang"),
            nu(),
        )
        .unwrap();
        assert!(b.risico("R-01").unwrap().aanvaarding.is_some());

        b.verklein("R-01", vec!["CBB-09".into()], Inschatting::Laag, Inschatting::Hoog, nu())
            .unwrap();
        assert!(b.risico("R-01").unwrap().aanvaarding.is_none());
    }

    #[test]
    fn een_risico_zonder_oorzaak_of_gevolg_wordt_geweigerd() {
        let mut b = beoordeling();
        let fout = b
            .onderken(
                "R-03",
                "iets vervelends",
                "  ",
                "schade",
                Inschatting::Laag,
                Inschatting::Laag,
                nu(),
            )
            .unwrap_err();
        assert!(fout.to_string().contains("geen maatregel bij te bedenken"), "kreeg: {fout}");
    }

    #[test]
    fn dezelfde_risicocode_komt_er_niet_twee_keer_in() {
        let mut b = met_risico();
        assert!(b
            .onderken(
                "R-01",
                "iets anders",
                "een oorzaak",
                "een gevolg",
                Inschatting::Laag,
                Inschatting::Laag,
                nu(),
            )
            .is_err());
    }

    #[test]
    fn een_bron_kan_niet_in_de_toekomst_zijn_geraadpleegd() {
        let mut b = beoordeling();
        assert!(b
            .raadpleeg_bron("dreigingsbeeld", "publicatie", nu() + Duration::days(1), nu())
            .is_err());
        b.raadpleeg_bron("dreigingsbeeld", "publicatie", nu() - Duration::days(10), nu()).unwrap();
        assert!(b
            .raadpleeg_bron("dreigingsbeeld", "publicatie", nu() - Duration::days(5), nu())
            .is_err());
    }

    /// Een beoordeling zonder risico's is een verklaring dat er niets aan de
    /// hand is.
    #[test]
    fn vaststellen_vergt_ten_minste_een_risico() {
        let mut b = beoordeling();
        let fout = b.stel_vast("A. de Vries", nu()).unwrap_err();
        assert!(fout.to_string().contains("risico.risicos"), "kreeg: {fout}");
    }

    #[test]
    fn een_hoog_restrisico_zonder_aanvaarding_blokkeert_het_vaststellen() {
        let mut b = beoordeling();
        b.onderken(
            "R-02",
            "gijzelsoftware",
            "een besmetting",
            "de gegevens zijn versleuteld",
            Inschatting::Hoog,
            Inschatting::ZeerHoog,
            nu(),
        )
        .unwrap();
        b.raadpleeg_bron("dreigingsbeeld", "publicatie", nu() - Duration::days(10), nu()).unwrap();
        let fout = b.stel_vast("A. de Vries", nu()).unwrap_err();
        assert!(fout.to_string().contains("risico.R-02.aanvaarding"), "kreeg: {fout}");

        b.aanvaard_restrisico(
            "R-02",
            "P. de Boer",
            "bestuurder",
            true,
            motivering("verdere maatregelen zijn niet haalbaar binnen het budget"),
            nu(),
        )
        .unwrap();
        b.stel_vast("A. de Vries", nu()).unwrap();
        assert_eq!(b.status, Status::Vastgesteld);
    }

    /// De teller mag nooit onder nul zakken.
    #[test]
    fn de_teller_dekt_alles_wat_kan_ontbreken() {
        let mut b = beoordeling();
        for i in 1..=4 {
            b.onderken(
                format!("R-0{i}"),
                "iets",
                "een oorzaak",
                "een gevolg",
                Inschatting::Hoog,
                Inschatting::Hoog,
                nu(),
            )
            .unwrap();
        }
        let r = b.volledigheid();
        assert!(
            r.ontbreekt.len() <= r.verplicht,
            "{} ontbrekend tegenover {} verplicht",
            r.ontbreekt.len(),
            r.verplicht
        );
    }

    #[test]
    fn de_beoordeling_overleeft_serialisatie() {
        let mut b = met_risico();
        b.raadpleeg_bron("dreigingsbeeld", "publicatie", nu() - Duration::days(10), nu()).unwrap();
        b.verklein("R-01", vec!["CBB-09".into()], Inschatting::Laag, Inschatting::Hoog, nu())
            .unwrap();
        let json = serde_json::to_string(&b).unwrap();
        let terug: Risicobeoordeling = serde_json::from_str(&json).unwrap();
        assert_eq!(b, terug);
    }
}
