//! Verzoeken om informatie op grond van de Wet open overheid.
//!
//! # Waarom dit een eigen dossier is en geen variant van het inzageverzoek
//!
//! Een informatieverzoek en een inzageverzoek lijken op elkaar en zijn het
//! niet. Ze verschillen op alles wat telt:
//!
//! | | Inzageverzoek | Woo-verzoek |
//! |---|---|---|
//! | Wie vraagt | de betrokkene, over zichzelf | iedereen, over een bestuurlijke aangelegenheid |
//! | Termijn | één maand, verlengbaar met twee | vier weken, eenmaal te verdagen met twee |
//! | Gronden om te weigeren | de uitzonderingen van de AVG | de gronden van artikel 5.1 Woo |
//! | Rechtsbescherming | klacht bij de toezichthouder, gang naar de rechter | bezwaar en beroep bij de bestuursrechter |
//! | Derden | niet aan de orde | belanghebbenden krijgen gelegenheid een zienswijze te geven |
//!
//! Wie de AVG-maandtermijn op een Woo-verzoek loslaat, is vier weken te laat.
//! Daarom draagt dit dossier zijn eigen klok, zijn eigen gronden en zijn eigen
//! taal, en verwijst het hooguit naar het andere spoor.
//!
//! # Eén bericht, twee dossiers
//!
//! Bevat één binnengekomen bericht zowel een informatieverzoek als een
//! inzageverzoek, dan ontstaan er twee dossiers met twee klokken en een
//! onderlinge verwijzing. Ze samenvoegen zou betekenen dat één van beide
//! termijnen wordt genegeerd.

use chrono::{DateTime, Utc};
use dpofg_terms::LopendeTermijn;
use serde::{Deserialize, Serialize};

use crate::{
    basis::{Compartiment, Herkomst, Id, Motivering, Status},
    error::{DomeinFout, Resultaat},
    volledigheid::{Ontbrekend, Volledig},
};

/// Een grond om informatie niet of niet geheel te verstrekken.
///
/// Gesloten opsomming naar artikel 5.1 van de Wet open overheid. De absolute
/// gronden van lid 1 laten geen afweging toe; bij de relatieve gronden van
/// lid 2 wordt het belang van openbaarheid afgewogen tegen het genoemde belang.
/// Dat onderscheid staat hier in het type zelf, want het bepaalt wat er moet
/// worden opgeschreven.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Weigeringsgrond {
    // --- absoluut, art. 5.1 lid 1 ---
    EenheidVanDeKroon,
    VeiligheidVanDeStaat,
    BedrijfsEnFabricagegegevens,
    Persoonsgegevens,
    // --- relatief, art. 5.1 lid 2 ---
    BetrekkingenMetAndereStaten,
    EconomischeBelangen,
    OpsporingEnVervolging,
    InspectieEnToezicht,
    EerbiedigingPersoonlijkeLevenssfeer,
    BeschermingMilieu,
    BeveiligingVanPersonenEnBedrijven,
    GoedFunctionerenVanDeStaat,
}

impl Weigeringsgrond {
    pub fn omschrijving(&self) -> &'static str {
        match self {
            Self::EenheidVanDeKroon => "de eenheid van de Kroon",
            Self::VeiligheidVanDeStaat => "de veiligheid van de Staat",
            Self::BedrijfsEnFabricagegegevens => {
                "vertrouwelijk meegedeelde bedrijfs- en fabricagegegevens"
            }
            Self::Persoonsgegevens => "bijzondere en strafrechtelijke persoonsgegevens",
            Self::BetrekkingenMetAndereStaten => "de betrekkingen met andere staten",
            Self::EconomischeBelangen => "economische of financiële belangen",
            Self::OpsporingEnVervolging => "opsporing en vervolging van strafbare feiten",
            Self::InspectieEnToezicht => "inspectie, controle en toezicht",
            Self::EerbiedigingPersoonlijkeLevenssfeer => "de persoonlijke levenssfeer",
            Self::BeschermingMilieu => "de bescherming van het milieu",
            Self::BeveiligingVanPersonenEnBedrijven => "de beveiliging van personen en bedrijven",
            Self::GoedFunctionerenVanDeStaat => {
                "het goed functioneren van de Staat en andere publieke lichamen"
            }
        }
    }

    /// Of deze grond een afweging tegen het belang van openbaarheid vergt.
    ///
    /// Bij een absolute grond is die afweging er niet; bij een relatieve grond
    /// is zij de kern van het besluit, en dan is een besluit zonder afweging
    /// geen besluit.
    pub fn is_relatief(&self) -> bool {
        !matches!(
            self,
            Self::EenheidVanDeKroon
                | Self::VeiligheidVanDeStaat
                | Self::BedrijfsEnFabricagegegevens
                | Self::Persoonsgegevens
        )
    }

    pub fn grondslag(&self) -> &'static str {
        if self.is_relatief() {
            "art. 5.1 lid 2 Wet open overheid"
        } else {
            "art. 5.1 lid 1 Wet open overheid"
        }
    }

    pub fn alle() -> [Self; 12] {
        [
            Self::EenheidVanDeKroon,
            Self::VeiligheidVanDeStaat,
            Self::BedrijfsEnFabricagegegevens,
            Self::Persoonsgegevens,
            Self::BetrekkingenMetAndereStaten,
            Self::EconomischeBelangen,
            Self::OpsporingEnVervolging,
            Self::InspectieEnToezicht,
            Self::EerbiedigingPersoonlijkeLevenssfeer,
            Self::BeschermingMilieu,
            Self::BeveiligingVanPersonenEnBedrijven,
            Self::GoedFunctionerenVanDeStaat,
        ]
    }
}

/// Een ingeroepen weigeringsgrond, met wat eronder ligt.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IngeroepenGrond {
    pub grond: Weigeringsgrond,
    /// Op welk document of onderdeel de grond ziet.
    pub betreft: String,
    /// De afweging tegen het belang van openbaarheid. Verplicht bij een
    /// relatieve grond; bij een absolute grond is er niets af te wegen.
    pub afweging: Option<Motivering>,
}

impl IngeroepenGrond {
    /// Of deze grond draagt wat zij moet dragen.
    pub fn is_onderbouwd(&self) -> bool {
        !self.grond.is_relatief() || self.afweging.is_some()
    }
}

/// De gelegenheid tot een zienswijze voor een belanghebbende derde.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Zienswijze {
    pub belanghebbende: String,
    pub gevraagd_op: Option<DateTime<Utc>>,
    pub ontvangen_op: Option<DateTime<Utc>>,
    /// Wat de belanghebbende vindt; leeg wanneer hij niet heeft gereageerd.
    pub standpunt: Option<String>,
}

impl Zienswijze {
    /// Of deze belanghebbende de gelegenheid heeft gekregen.
    ///
    /// De wet vraagt dat de gelegenheid wordt geboden, niet dat er wordt
    /// gereageerd: wie zwijgt, heeft zijn kans gehad.
    pub fn is_afgedaan(&self) -> bool {
        self.gevraagd_op.is_some()
    }
}

/// De uitkomst van een Woo-verzoek.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Woouitkomst {
    Openbaar,
    GedeeltelijkOpenbaar,
    Geweigerd,
    /// Het bestuursorgaan heeft de gevraagde informatie niet.
    NietAanwezig,
}

impl Woouitkomst {
    pub fn omschrijving(&self) -> &'static str {
        match self {
            Self::Openbaar => "openbaar gemaakt",
            Self::GedeeltelijkOpenbaar => "gedeeltelijk openbaar gemaakt",
            Self::Geweigerd => "geweigerd",
            Self::NietAanwezig => "de informatie is niet aanwezig",
        }
    }

    /// Of hierbij ten minste één weigeringsgrond hoort.
    pub fn vraagt_grond(&self) -> bool {
        matches!(self, Self::Geweigerd | Self::GedeeltelijkOpenbaar)
    }
}

/// Een verzoek om informatie.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Wooverzoek {
    pub id: Id,
    pub kenmerk: String,
    pub onderwerp: String,
    pub status: Status,
    pub compartiment: Compartiment,
    pub herkomst: Herkomst,

    pub ontvangen_op: DateTime<Utc>,
    pub verzoeker: String,

    /// De beslistermijn van vier weken.
    pub termijn: Option<LopendeTermijn>,
    pub termijn_pakket: Option<String>,
    /// De verdaging, zoals aan de verzoeker medegedeeld.
    pub verdaging_medegedeeld_op: Option<DateTime<Utc>>,
    pub verdaging_motivering: Option<Motivering>,

    pub zienswijzen: Vec<Zienswijze>,
    pub gronden: Vec<IngeroepenGrond>,

    pub besluit_op: Option<DateTime<Utc>>,
    pub uitkomst: Option<Woouitkomst>,

    /// Het inzageverzoek dat uit hetzelfde bericht voortkwam.
    ///
    /// Twee dossiers, twee klokken, één onderlinge verwijzing. Samenvoegen zou
    /// betekenen dat één van beide termijnen wordt genegeerd.
    pub gerelateerd_verzoek_id: Option<Id>,
}

impl Wooverzoek {
    pub fn nieuw(
        kenmerk: impl Into<String>,
        onderwerp: impl Into<String>,
        verzoeker: impl Into<String>,
        ontvangen_op: DateTime<Utc>,
        door: impl Into<String>,
        op: DateTime<Utc>,
    ) -> Self {
        Self {
            id: Id::nieuw(),
            kenmerk: kenmerk.into(),
            onderwerp: onderwerp.into(),
            status: Status::Concept,
            compartiment: Compartiment::nieuw(Compartiment::VERTROUWELIJK),
            herkomst: Herkomst::nieuw(door, op),
            ontvangen_op,
            verzoeker: verzoeker.into(),
            termijn: None,
            termijn_pakket: None,
            verdaging_medegedeeld_op: None,
            verdaging_motivering: None,
            zienswijzen: Vec::new(),
            gronden: Vec::new(),
            besluit_op: None,
            uitkomst: None,
            gerelateerd_verzoek_id: None,
        }
    }

    /// Belanghebbenden die nog geen gelegenheid hebben gekregen.
    pub fn openstaande_zienswijzen(&self) -> Vec<&Zienswijze> {
        self.zienswijzen.iter().filter(|z| !z.is_afgedaan()).collect()
    }

    /// Ingeroepen gronden waarbij de afweging ontbreekt.
    pub fn ononderbouwde_gronden(&self) -> Vec<&IngeroepenGrond> {
        self.gronden.iter().filter(|g| !g.is_onderbouwd()).collect()
    }

    /// Start de beslistermijn met een reeds berekende klok.
    pub fn start_termijn(&mut self, klok: LopendeTermijn, op: DateTime<Utc>) -> Resultaat<()> {
        if self.termijn.is_some() {
            return Err(DomeinFout::OngeldigeStatusovergang {
                van: "termijn gestart".into(),
                naar: "termijn gestart".into(),
                reden: "de beslistermijn van dit verzoek loopt al".into(),
            });
        }
        if klok.anker != self.ontvangen_op {
            return Err(DomeinFout::OngeldigeWaarde {
                veld: "woo.termijn".into(),
                reden: "de beslistermijn loopt vanaf ontvangst van het verzoek; de klok is op een \
                        ander moment verankerd"
                    .into(),
            });
        }
        self.termijn = Some(klok);
        self.herkomst.wijzig("beslistermijn gestart", op);
        Ok(())
    }

    /// Legt de verdaging vast.
    ///
    /// De klok bewaakt dat de mededeling binnen de oorspronkelijke termijn
    /// viel; dit dossier eist de schriftelijke motivering die de wet verlangt.
    pub fn leg_verdaging_vast(
        &mut self,
        medegedeeld_op: DateTime<Utc>,
        motivering: Motivering,
        op: DateTime<Utc>,
    ) -> Resultaat<()> {
        if self.termijn.is_none() {
            return Err(DomeinFout::OntbrekendeVerwijzing {
                veld: "woo.termijn".into(),
                naar: "een lopende beslistermijn".into(),
            });
        }
        if medegedeeld_op > op {
            return Err(DomeinFout::OnmogelijkTijdstip {
                veld: "woo.verdaging".into(),
                reden: "de mededeling zou in de toekomst zijn verzonden".into(),
            });
        }
        self.verdaging_medegedeeld_op = Some(medegedeeld_op);
        self.verdaging_motivering = Some(motivering);
        self.herkomst.wijzig("verdaging vastgelegd", op);
        Ok(())
    }

    /// Voegt een belanghebbende toe die een zienswijze mag geven.
    pub fn voeg_belanghebbende_toe(
        &mut self,
        naam: impl Into<String>,
        op: DateTime<Utc>,
    ) -> Resultaat<()> {
        let naam = naam.into();
        if self.zienswijzen.iter().any(|z| z.belanghebbende == naam) {
            return Err(DomeinFout::OngeldigeWaarde {
                veld: "woo.zienswijzen".into(),
                reden: format!("'{naam}' staat al in de lijst"),
            });
        }
        self.zienswijzen.push(Zienswijze {
            belanghebbende: naam,
            gevraagd_op: None,
            ontvangen_op: None,
            standpunt: None,
        });
        self.herkomst.wijzig("belanghebbende toegevoegd", op);
        Ok(())
    }

    /// Legt vast dat een belanghebbende de gelegenheid heeft gekregen, en wat
    /// hij eventueel heeft ingebracht.
    pub fn leg_zienswijze_vast(
        &mut self,
        belanghebbende: &str,
        gevraagd_op: DateTime<Utc>,
        ontvangen_op: Option<DateTime<Utc>>,
        standpunt: Option<String>,
        op: DateTime<Utc>,
    ) -> Resultaat<()> {
        if gevraagd_op > op {
            return Err(DomeinFout::OnmogelijkTijdstip {
                veld: "woo.zienswijzen".into(),
                reden: "de gelegenheid zou in de toekomst zijn geboden".into(),
            });
        }
        let z =
            self.zienswijzen.iter_mut().find(|z| z.belanghebbende == belanghebbende).ok_or_else(
                || DomeinFout::OntbrekendeVerwijzing {
                    veld: "woo.zienswijzen".into(),
                    naar: format!("een belanghebbende '{belanghebbende}'"),
                },
            )?;
        z.gevraagd_op = Some(gevraagd_op);
        z.ontvangen_op = ontvangen_op;
        z.standpunt = standpunt;
        self.herkomst.wijzig(format!("zienswijze van {belanghebbende} vastgelegd"), op);
        Ok(())
    }

    /// Roept een weigeringsgrond in.
    pub fn roep_grond_in(
        &mut self,
        grond: Weigeringsgrond,
        betreft: impl Into<String>,
        afweging: Option<Motivering>,
        op: DateTime<Utc>,
    ) -> Resultaat<()> {
        if grond.is_relatief() && afweging.is_none() {
            return Err(DomeinFout::OngeldigeWaarde {
                veld: "woo.gronden".into(),
                reden: format!(
                    "'{}' is een relatieve grond: het belang van openbaarheid moet worden \
                     afgewogen tegen het genoemde belang. Zonder die afweging is er geen besluit \
                     maar een verwijzing naar een wetsartikel",
                    grond.omschrijving()
                ),
            });
        }
        self.gronden.push(IngeroepenGrond { grond, betreft: betreft.into(), afweging });
        self.herkomst.wijzig("weigeringsgrond ingeroepen", op);
        Ok(())
    }

    /// Neemt het besluit.
    pub fn neem_besluit(
        &mut self,
        uitkomst: Woouitkomst,
        moment: DateTime<Utc>,
        op: DateTime<Utc>,
    ) -> Resultaat<()> {
        if moment > op {
            return Err(DomeinFout::OnmogelijkTijdstip {
                veld: "woo.besluit_op".into(),
                reden: "het besluit zou in de toekomst zijn genomen".into(),
            });
        }

        let mut belet = Vec::new();
        let open = self.openstaande_zienswijzen().len();
        if open > 0 {
            belet.push(format!(
                "{open} belanghebbende(n) hebben nog geen gelegenheid gekregen een zienswijze te \
                 geven (art. 4.4 lid 4 Wet open overheid)"
            ));
        }
        if uitkomst.vraagt_grond() && self.gronden.is_empty() {
            belet.push(
                "er is geen weigeringsgrond ingeroepen; een weigering zonder grond is geen \
                 besluit (art. 5.1 Wet open overheid)"
                    .into(),
            );
        }
        let zonder = self.ononderbouwde_gronden().len();
        if zonder > 0 {
            belet.push(format!(
                "{zonder} relatieve grond(en) zonder afweging tegen het belang van openbaarheid \
                 (art. 5.1 lid 2 Wet open overheid)"
            ));
        }

        if !belet.is_empty() {
            return Err(DomeinFout::NietVolledig { soort: "Woo-verzoek".into(), ontbreekt: belet });
        }

        if let Some(klok) = self.termijn.as_mut() {
            klok.rond_af(moment);
        }
        self.uitkomst = Some(uitkomst);
        self.besluit_op = Some(moment);
        self.status = Status::Vastgesteld;
        self.herkomst.stel_vast(self.herkomst.aangemaakt_door.clone(), op);
        Ok(())
    }
}

impl Volledig for Wooverzoek {
    fn soortnaam(&self) -> &'static str {
        "Woo-verzoek"
    }

    fn aantal_verplichte_onderdelen(&self) -> usize {
        // De klok en het besluit staan er altijd; elke belanghebbende en elke
        // ingeroepen grond komen erbij.
        2 + self.zienswijzen.len() + self.gronden.len()
    }

    fn ontbrekende_onderdelen(&self) -> Vec<Ontbrekend> {
        let mut uit = Vec::new();

        if self.termijn.is_none() {
            uit.push(Ontbrekend::blokkerend(
                "woo.termijn",
                "start de beslistermijn van vier weken; die is korter dan de maandtermijn van een \
                 inzageverzoek",
                "art. 4.4 lid 1 Wet open overheid",
            ));
        }
        for z in self.openstaande_zienswijzen() {
            uit.push(Ontbrekend::blokkerend(
                format!("woo.zienswijze.{}", z.belanghebbende),
                format!("bied '{}' gelegenheid een zienswijze te geven", z.belanghebbende),
                "art. 4.4 lid 4 Wet open overheid",
            ));
        }
        for g in self.ononderbouwde_gronden() {
            uit.push(Ontbrekend::blokkerend(
                format!("woo.grond.{}", g.betreft),
                format!("weeg het belang van openbaarheid af tegen {}", g.grond.omschrijving()),
                g.grond.grondslag(),
            ));
        }
        if self.uitkomst.is_none() {
            uit.push(Ontbrekend::blokkerend(
                "woo.besluit",
                "neem het besluit op het verzoek",
                "art. 4.4 lid 1 Wet open overheid",
            ));
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

    fn motivering(tekst: &str) -> Motivering {
        Motivering::nieuw(tekst, "u1", nu()).unwrap()
    }

    fn verzoek() -> Wooverzoek {
        Wooverzoek::nieuw(
            "WOO-2026-003",
            "correspondentie over de aanbesteding",
            "een journalist",
            nu(),
            "u1",
            nu(),
        )
    }

    #[test]
    fn een_relatieve_grond_zonder_afweging_wordt_geweigerd() {
        let mut v = verzoek();
        let fout = v
            .roep_grond_in(Weigeringsgrond::EconomischeBelangen, "bijlage 3", None, nu())
            .unwrap_err();
        assert!(fout.to_string().contains("afgewogen"), "kreeg: {fout}");
        assert!(fout.to_string().contains("verwijzing naar een wetsartikel"));
    }

    /// Bij een absolute grond valt er niets af te wegen; die eis stellen zou de
    /// gebruiker dwingen een afweging te verzinnen die de wet niet kent.
    #[test]
    fn een_absolute_grond_vergt_geen_afweging() {
        let mut v = verzoek();
        v.roep_grond_in(Weigeringsgrond::VeiligheidVanDeStaat, "bijlage 1", None, nu()).unwrap();
        assert!(v.ononderbouwde_gronden().is_empty());
    }

    #[test]
    fn elke_grond_draagt_het_juiste_lid() {
        for grond in Weigeringsgrond::alle() {
            let lid = if grond.is_relatief() { "lid 2" } else { "lid 1" };
            assert!(grond.grondslag().contains(lid), "{grond:?} verwijst naar het verkeerde lid");
        }
    }

    #[test]
    fn een_besluit_zonder_grond_wordt_geweigerd() {
        let mut v = verzoek();
        let fout = v.neem_besluit(Woouitkomst::Geweigerd, nu(), nu()).unwrap_err();
        assert!(fout.to_string().contains("weigering zonder grond"));
    }

    #[test]
    fn een_besluit_wacht_op_de_zienswijze_van_een_derde() {
        let mut v = verzoek();
        v.voeg_belanghebbende_toe("de aannemer", nu()).unwrap();
        let fout = v.neem_besluit(Woouitkomst::Openbaar, nu(), nu()).unwrap_err();
        assert!(fout.to_string().contains("art. 4.4 lid 4"));

        v.leg_zienswijze_vast("de aannemer", nu(), None, None, nu()).unwrap();
        assert!(v.neem_besluit(Woouitkomst::Openbaar, nu(), nu()).is_ok());
    }

    /// Wie zwijgt, heeft zijn kans gehad: de wet vraagt dat de gelegenheid
    /// wordt geboden, niet dat er wordt gereageerd.
    #[test]
    fn een_belanghebbende_die_zwijgt_houdt_het_besluit_niet_tegen() {
        let mut v = verzoek();
        v.voeg_belanghebbende_toe("de aannemer", nu()).unwrap();
        v.leg_zienswijze_vast("de aannemer", nu(), None, None, nu()).unwrap();
        assert!(v.openstaande_zienswijzen().is_empty());
    }

    #[test]
    fn een_verdaging_vergt_een_lopende_termijn() {
        let mut v = verzoek();
        let fout =
            v.leg_verdaging_vast(nu(), motivering("de omvang van het dossier"), nu()).unwrap_err();
        assert!(matches!(fout, DomeinFout::OntbrekendeVerwijzing { .. }));
    }

    #[test]
    fn het_verzoek_overleeft_serialisatie() {
        let mut v = verzoek();
        v.voeg_belanghebbende_toe("de aannemer", nu()).unwrap();
        v.roep_grond_in(
            Weigeringsgrond::EconomischeBelangen,
            "bijlage 3",
            Some(motivering("de onderhandelingspositie zou worden geschaad")),
            nu(),
        )
        .unwrap();

        let json = serde_json::to_string(&v).unwrap();
        let terug: Wooverzoek = serde_json::from_str(&json).unwrap();
        assert_eq!(v, terug);
    }
}
