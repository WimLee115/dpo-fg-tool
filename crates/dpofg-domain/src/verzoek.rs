//! Verzoeken van betrokkenen.
//!
//! # Waarom dit dossier zoveel afdwingt
//!
//! Dit is het werkproces waarop het vaakst wordt geklaagd. De onderzoeksbasis
//! achter hoofdstuk 2.4 van het foutbestendigheidsontwerp noemt drie patronen
//! die telkens terugkomen: te laat antwoorden, een verlengingsbericht sturen
//! als de termijn al verstreken is, en de reikwijdte te eng nemen — het verzoek
//! beperken tot "wat het ene systeem kan exporteren".
//!
//! Alle drie zijn hier onmogelijk gemaakt in plaats van afgeraden:
//!
//! * De status *verlengd* is onbereikbaar zonder een geregistreerde verzending
//!   van het verlengingsbericht binnen de eerste maand én een van de twee
//!   wettelijke gronden. De termijnenmotor bewaakt het eerste, dit dossier het
//!   tweede.
//! * *Afgehandeld* is onbereikbaar zolang niet elke vindplaats een uitkomst
//!   heeft. Wat er niet is gezocht, is niet stilzwijgend leeg.
//! * De uitkomst per vindplaats is een gesloten opsomming. "Account gesloten"
//!   bestaat niet als antwoord op een wisverzoek, en is dus ook niet in te
//!   vullen.
//!
//! # De omstreden lezing
//!
//! Loopt de maand vanaf ontvangst van het verzoek, of pas vanaf het moment
//! waarop de identiteit is vastgesteld? Daarover wordt verschillend gedacht.
//! Paragraaf 10.2 van het projectplan bindt de tool aan één gedragslijn: bij
//! een omstreden interpretatie worden **beide lezingen aangeboden**, met
//! bronvermelding, en wordt de gekozen lezing met motivering in het dossier
//! vastgelegd. Zij wordt nooit hard in de motor gebakken. Vandaar
//! [`Termijnlezing`] als expliciete keuze en niet als verborgen aanname.

use chrono::{DateTime, Utc};
use dpofg_terms::LopendeTermijn;
use serde::{Deserialize, Serialize};

use crate::{
    basis::{Compartiment, Herkomst, Id, Motivering, Status},
    error::{DomeinFout, Resultaat},
    volledigheid::{Ontbrekend, Volledig},
};

/// Het recht dat de betrokkene inroept.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Verzoeksoort {
    Inzage,
    Rectificatie,
    Wissing,
    Beperking,
    Overdraagbaarheid,
    Bezwaar,
    GeautomatiseerdBesluit,
}

impl Verzoeksoort {
    pub fn omschrijving(&self) -> &'static str {
        match self {
            Self::Inzage => "inzage",
            Self::Rectificatie => "rectificatie",
            Self::Wissing => "wissing",
            Self::Beperking => "beperking van de verwerking",
            Self::Overdraagbaarheid => "overdraagbaarheid",
            Self::Bezwaar => "bezwaar",
            Self::GeautomatiseerdBesluit => "geautomatiseerde besluitvorming",
        }
    }

    pub fn grondslag(&self) -> &'static str {
        match self {
            Self::Inzage => "art. 15 AVG",
            Self::Rectificatie => "art. 16 AVG",
            Self::Wissing => "art. 17 AVG",
            Self::Beperking => "art. 18 AVG",
            Self::Overdraagbaarheid => "art. 20 AVG",
            Self::Bezwaar => "art. 21 AVG",
            Self::GeautomatiseerdBesluit => "art. 22 AVG",
        }
    }

    /// Of een gehonoreerd verzoek van deze soort de ontvangers moet bereiken.
    ///
    /// Artikel 19 noemt rectificatie, wissing en beperking met zoveel woorden.
    pub fn vraagt_kennisgeving_aan_ontvangers(&self) -> bool {
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
            Self::GeautomatiseerdBesluit,
        ]
    }
}

/// Langs welke weg het verzoek binnenkwam.
///
/// Wordt vastgelegd omdat een verzoek langs elk kanaal geldig is: het hoeft
/// niet op een formulier, niet schriftelijk en niet bij de juiste afdeling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Verzoekkanaal {
    Post,
    Email,
    Telefonisch,
    Balie,
    Portaal,
    SocialeMedia,
    Anders,
}

impl Verzoekkanaal {
    pub fn omschrijving(&self) -> &'static str {
        match self {
            Self::Post => "per post",
            Self::Email => "per e-mail",
            Self::Telefonisch => "telefonisch",
            Self::Balie => "aan de balie",
            Self::Portaal => "via het portaal",
            Self::SocialeMedia => "via sociale media",
            Self::Anders => "anders",
        }
    }
}

/// Vanaf welk moment de termijn loopt.
///
/// Een omstreden punt: de verordening laat toe de identiteit vast te stellen
/// wanneer daarover gerede twijfel bestaat, maar zegt niet met zoveel woorden
/// dat de termijn dan pas begint. Beide lezingen worden aangeboden; de keuze
/// wordt met motivering vastgelegd.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Termijnlezing {
    /// De maand loopt vanaf ontvangst van het verzoek. De ruimste lezing voor
    /// de betrokkene, en de veiligste voor de organisatie.
    VanafOntvangst,
    /// De maand loopt vanaf het moment waarop de identiteit is vastgesteld.
    /// Verdedigbaar bij gerede twijfel, maar omstreden.
    VanafIdentiteitsvaststelling,
}

impl Termijnlezing {
    pub fn omschrijving(&self) -> &'static str {
        match self {
            Self::VanafOntvangst => "vanaf ontvangst van het verzoek",
            Self::VanafIdentiteitsvaststelling => "vanaf vaststelling van de identiteit",
        }
    }

    pub fn bron(&self) -> &'static str {
        match self {
            Self::VanafOntvangst => {
                "art. 12 lid 3 AVG: 'onverwijld en in ieder geval binnen een maand na ontvangst \
                 van het verzoek'"
            }
            Self::VanafIdentiteitsvaststelling => {
                "art. 12 lid 6 AVG, gelezen met lid 3: bij gerede twijfel over de identiteit mag \
                 aanvullende informatie worden gevraagd; dat de termijn dan opnieuw begint, staat \
                 er niet met zoveel woorden en is omstreden"
            }
        }
    }

    pub fn alle() -> [Self; 2] {
        [Self::VanafOntvangst, Self::VanafIdentiteitsvaststelling]
    }
}

/// De gekozen lezing, met de motivering die erbij hoort.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Lezingkeuze {
    pub lezing: Termijnlezing,
    pub motivering: Motivering,
    pub gekozen_op: DateTime<Utc>,
}

/// De twee gronden waarop de termijn mag worden verlengd.
///
/// Een gesloten opsomming, want de verordening kent er precies twee. Een derde
/// grond bestaat niet en is dus ook niet in te typen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Verlengingsgrond {
    Complexiteit,
    AantalVerzoeken,
}

impl Verlengingsgrond {
    pub fn omschrijving(&self) -> &'static str {
        match self {
            Self::Complexiteit => "de complexiteit van het verzoek",
            Self::AantalVerzoeken => "het aantal verzoeken",
        }
    }
}

/// De verlenging, zoals aan de betrokkene medegedeeld.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Verlenging {
    pub grond: Verlengingsgrond,
    pub medegedeeld_op: DateTime<Utc>,
    pub motivering: Motivering,
}

/// Wat er met de gegevens op één vindplaats is gebeurd.
///
/// Gesloten opsomming, en dat is het punt. "Account gesloten" is geen antwoord
/// op een wisverzoek en staat er daarom niet in; wie dat wil vastleggen, moet
/// zeggen wat er werkelijk met de gegevens is gebeurd.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Vindplaatsuitkomst {
    /// De gegevens zijn verstrekt aan de betrokkene.
    Verstrekt,
    /// De gegevens zijn gecorrigeerd.
    Gerectificeerd,
    /// De gegevens zijn vernietigd en niet meer herleidbaar.
    Verwijderd,
    /// De verwerking is beperkt; de gegevens staan er nog.
    Beperkt,
    /// Onomkeerbaar geanonimiseerd, met afgeronde toets.
    Geanonimiseerd,
    /// Gepseudonimiseerd: nog steeds persoonsgegevens.
    Gepseudonimiseerd,
    /// Er zijn hier geen gegevens van deze betrokkene aangetroffen.
    NietAangetroffen,
    /// Het verzoek is voor deze vindplaats geweigerd.
    Geweigerd,
}

impl Vindplaatsuitkomst {
    pub fn omschrijving(&self) -> &'static str {
        match self {
            Self::Verstrekt => "verstrekt",
            Self::Gerectificeerd => "gerectificeerd",
            Self::Verwijderd => "verwijderd",
            Self::Beperkt => "verwerking beperkt",
            Self::Geanonimiseerd => "geanonimiseerd",
            Self::Gepseudonimiseerd => "gepseudonimiseerd — nog persoonsgegevens",
            Self::NietAangetroffen => "niets aangetroffen",
            Self::Geweigerd => "geweigerd",
        }
    }
}

/// De toets die "geanonimiseerd" moet dragen.
///
/// Anonimiseren is een sterke bewering: als zij niet klopt, is er niets
/// gewist en staan de gegevens er nog. Zonder afgeronde toets valt de uitkomst
/// terug op *gepseudonimiseerd*.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Anonimiseringstoets {
    /// Is de betrokkene nog uit de gegevens te lichten?
    pub singling_out_uitgesloten: bool,
    /// Zijn de gegevens nog aan andere gegevens te koppelen?
    pub koppelbaarheid_uitgesloten: bool,
    /// Zijn er nog eigenschappen uit af te leiden?
    pub afleidbaarheid_uitgesloten: bool,
    pub motivering: Motivering,
    /// De tweede persoon die de toets bevestigt.
    pub bevestigd_door: String,
}

impl Anonimiseringstoets {
    pub fn is_geslaagd(&self) -> bool {
        self.singling_out_uitgesloten
            && self.koppelbaarheid_uitgesloten
            && self.afleidbaarheid_uitgesloten
    }
}

/// Eén plaats waar gegevens van de betrokkene kunnen staan.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Vindplaats {
    /// De registerregel waaruit deze vindplaats volgt.
    pub verwerking_id: Id,
    pub kenmerk: String,
    pub omschrijving: String,
    pub uitkomst: Option<Vindplaatsuitkomst>,
    pub toelichting: Option<String>,
    pub anonimiseringstoets: Option<Anonimiseringstoets>,
    pub afgehandeld_op: Option<DateTime<Utc>>,
}

impl Vindplaats {
    pub fn is_afgehandeld(&self) -> bool {
        self.uitkomst.is_some()
    }
}

/// De kennisgeving aan één ontvanger, op grond van artikel 19.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ontvangerkennisgeving {
    pub ontvanger: String,
    pub verzonden_op: Option<DateTime<Utc>>,
    pub wijze: Option<String>,
    /// Of de kennisgeving onmogelijk blijkt of onevenredig veel moeite kost.
    pub onmogelijk_of_onevenredig: bool,
    pub motivering: Option<Motivering>,
}

impl Ontvangerkennisgeving {
    /// Of deze kennisgeving een uitkomst heeft.
    ///
    /// Invariant I18: het verzoek is niet af te sluiten zolang er één openstaat
    /// zonder verzenddatum en zonder motivering van onmogelijkheid.
    pub fn is_afgedaan(&self) -> bool {
        self.verzonden_op.is_some() || (self.onmogelijk_of_onevenredig && self.motivering.is_some())
    }
}

/// De uitkomst van het verzoek als geheel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Verzoekuitkomst {
    Voldaan,
    DeelsVoldaan,
    Geweigerd,
}

impl Verzoekuitkomst {
    pub fn omschrijving(&self) -> &'static str {
        match self {
            Self::Voldaan => "voldaan",
            Self::DeelsVoldaan => "deels voldaan",
            Self::Geweigerd => "geweigerd",
        }
    }

    /// Of hierbij het bericht van artikel 12 lid 4 hoort.
    pub fn vraagt_bericht_lid4(&self) -> bool {
        matches!(self, Self::Geweigerd | Self::DeelsVoldaan)
    }
}

/// Het bericht van artikel 12 lid 4.
///
/// De verordening schrijft niet alleen voor dát er bericht gaat, maar ook wat
/// erin moet staan: de redenen, én de mogelijkheid een klacht in te dienen bij
/// de toezichthouder en een beroep in te stellen bij de rechter. Die twee zijn
/// aparte velden omdat zij anders stilzwijgend wegvallen.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BerichtLid4 {
    pub verzonden_op: DateTime<Utc>,
    pub noemt_klachtrecht: bool,
    pub noemt_rechtsmiddel: bool,
    pub redenen: Motivering,
}

impl BerichtLid4 {
    pub fn is_volledig(&self) -> bool {
        self.noemt_klachtrecht && self.noemt_rechtsmiddel
    }
}

/// Een verzoek van een betrokkene.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Betrokkenenverzoek {
    pub id: Id,
    pub kenmerk: String,
    pub omschrijving: String,
    pub status: Status,
    pub compartiment: Compartiment,
    pub herkomst: Herkomst,

    // --- intake ---
    pub soort: Verzoeksoort,
    pub kanaal: Verzoekkanaal,
    pub ontvangen_op: DateTime<Utc>,
    pub identiteit_geverifieerd_op: Option<DateTime<Utc>>,
    pub lezing: Option<Lezingkeuze>,

    // --- de klok ---
    pub termijn: Option<LopendeTermijn>,
    pub termijn_pakket: Option<String>,
    pub verlenging: Option<Verlenging>,

    // --- het werk ---
    pub vindplaatsen: Vec<Vindplaats>,
    pub kennisgevingen: Vec<Ontvangerkennisgeving>,
    /// Of aan de betrokkene is medegedeeld wélke ontvangers zijn geïnformeerd.
    pub ontvangers_medegedeeld_op: Option<DateTime<Utc>>,

    // --- afronding ---
    pub uitkomst: Option<Verzoekuitkomst>,
    pub bericht_lid4: Option<BerichtLid4>,
    pub afgehandeld_op: Option<DateTime<Utc>>,

    pub behandelaar: String,
}

impl Betrokkenenverzoek {
    #[allow(clippy::too_many_arguments)]
    pub fn nieuw(
        kenmerk: impl Into<String>,
        omschrijving: impl Into<String>,
        soort: Verzoeksoort,
        kanaal: Verzoekkanaal,
        ontvangen_op: DateTime<Utc>,
        behandelaar: impl Into<String>,
        door: impl Into<String>,
        op: DateTime<Utc>,
    ) -> Self {
        Self {
            id: Id::nieuw(),
            kenmerk: kenmerk.into(),
            omschrijving: omschrijving.into(),
            status: Status::Concept,
            // Een verzoekdossier draagt de identiteit van de betrokkene en wat
            // er over hem is vastgelegd. Dat hoort niet in het algemene
            // compartiment.
            compartiment: Compartiment::nieuw(Compartiment::VERTROUWELIJK),
            herkomst: Herkomst::nieuw(door, op),
            soort,
            kanaal,
            ontvangen_op,
            identiteit_geverifieerd_op: None,
            lezing: None,
            termijn: None,
            termijn_pakket: None,
            verlenging: None,
            vindplaatsen: Vec::new(),
            kennisgevingen: Vec::new(),
            ontvangers_medegedeeld_op: None,
            uitkomst: None,
            bericht_lid4: None,
            afgehandeld_op: None,
            behandelaar: behandelaar.into(),
        }
    }

    /// Het moment waarop de klok volgens de gekozen lezing begint.
    ///
    /// `None` zolang er geen lezing is gekozen, of zolang de gekozen lezing een
    /// moment vergt dat er nog niet is.
    pub fn anker(&self) -> Option<DateTime<Utc>> {
        match self.lezing.as_ref()?.lezing {
            Termijnlezing::VanafOntvangst => Some(self.ontvangen_op),
            Termijnlezing::VanafIdentiteitsvaststelling => self.identiteit_geverifieerd_op,
        }
    }

    /// Vindplaatsen die nog geen uitkomst hebben.
    pub fn openstaande_vindplaatsen(&self) -> Vec<&Vindplaats> {
        self.vindplaatsen.iter().filter(|v| !v.is_afgehandeld()).collect()
    }

    /// Kennisgevingen die nog geen uitkomst hebben (invariant I18).
    pub fn openstaande_kennisgevingen(&self) -> Vec<&Ontvangerkennisgeving> {
        self.kennisgevingen.iter().filter(|k| !k.is_afgedaan()).collect()
    }

    /// Legt de gekozen lezing van de termijn vast.
    pub fn kies_lezing(
        &mut self,
        lezing: Termijnlezing,
        motivering: Motivering,
        op: DateTime<Utc>,
    ) -> Resultaat<()> {
        if self.termijn.is_some() {
            return Err(DomeinFout::OngeldigeStatusovergang {
                van: "termijn gestart".into(),
                naar: "lezing gewijzigd".into(),
                reden: "de klok loopt al; een andere lezing zou de deadline met terugwerkende \
                        kracht verschuiven"
                    .into(),
            });
        }
        if lezing == Termijnlezing::VanafIdentiteitsvaststelling
            && self.identiteit_geverifieerd_op.is_none()
        {
            return Err(DomeinFout::OngeldigeWaarde {
                veld: "verzoek.lezing".into(),
                reden: "deze lezing rekent vanaf de vaststelling van de identiteit; leg dat \
                        moment eerst vast"
                    .into(),
            });
        }
        self.lezing = Some(Lezingkeuze { lezing, motivering, gekozen_op: op });
        self.herkomst.wijzig("lezing van de termijn gekozen", op);
        Ok(())
    }

    /// Legt vast wanneer de identiteit is vastgesteld.
    pub fn stel_identiteit_vast(
        &mut self,
        moment: DateTime<Utc>,
        op: DateTime<Utc>,
    ) -> Resultaat<()> {
        if moment > op {
            return Err(DomeinFout::OnmogelijkTijdstip {
                veld: "verzoek.identiteit_geverifieerd_op".into(),
                reden: "de vaststelling zou in de toekomst liggen; controleer het tijdstip".into(),
            });
        }
        if moment < self.ontvangen_op {
            return Err(DomeinFout::OnmogelijkTijdstip {
                veld: "verzoek.identiteit_geverifieerd_op".into(),
                reden: "de identiteit zou zijn vastgesteld vóórdat het verzoek binnenkwam; \
                        controleer welk van de twee tijdstippen verwisseld is"
                    .into(),
            });
        }
        self.identiteit_geverifieerd_op = Some(moment);
        self.herkomst.wijzig("identiteit vastgesteld", op);
        Ok(())
    }

    /// Start de termijn met een reeds berekende klok.
    pub fn start_termijn(&mut self, klok: LopendeTermijn, op: DateTime<Utc>) -> Resultaat<()> {
        if self.termijn.is_some() {
            return Err(DomeinFout::OngeldigeStatusovergang {
                van: "termijn gestart".into(),
                naar: "termijn gestart".into(),
                reden: "de klok van dit verzoek loopt al".into(),
            });
        }
        let Some(anker) = self.anker() else {
            return Err(DomeinFout::OngeldigeWaarde {
                veld: "verzoek.termijn".into(),
                reden: "kies eerst vanaf welk moment de termijn loopt; die keuze is omstreden en \
                        wordt daarom met motivering vastgelegd"
                    .into(),
            });
        };
        if klok.anker != anker {
            return Err(DomeinFout::OngeldigeWaarde {
                veld: "verzoek.termijn".into(),
                reden: "de klok is op een ander moment verankerd dan de gekozen lezing aanwijst"
                    .into(),
            });
        }
        self.termijn = Some(klok);
        self.herkomst.wijzig("termijn gestart", op);
        Ok(())
    }

    /// Legt de verlenging vast.
    ///
    /// De klok bewaakt dat het bericht binnen de oorspronkelijke termijn is
    /// verzonden; dit dossier bewaakt dat er een van de twee wettelijke gronden
    /// onder ligt. Zonder allebei is de status *verlengd* onbereikbaar.
    pub fn leg_verlenging_vast(
        &mut self,
        grond: Verlengingsgrond,
        medegedeeld_op: DateTime<Utc>,
        motivering: Motivering,
        op: DateTime<Utc>,
    ) -> Resultaat<()> {
        if self.termijn.is_none() {
            return Err(DomeinFout::OntbrekendeVerwijzing {
                veld: "verzoek.termijn".into(),
                naar: "een lopende termijn".into(),
            });
        }
        if medegedeeld_op > op {
            return Err(DomeinFout::OnmogelijkTijdstip {
                veld: "verzoek.verlenging".into(),
                reden: "de mededeling zou in de toekomst zijn verzonden; controleer het tijdstip"
                    .into(),
            });
        }
        self.verlenging = Some(Verlenging { grond, medegedeeld_op, motivering });
        self.herkomst.wijzig("verlenging vastgelegd", op);
        Ok(())
    }

    /// Voegt een vindplaats toe die uit het register volgt.
    pub fn voeg_vindplaats_toe(
        &mut self,
        verwerking_id: Id,
        kenmerk: impl Into<String>,
        omschrijving: impl Into<String>,
        op: DateTime<Utc>,
    ) -> Resultaat<()> {
        let kenmerk = kenmerk.into();
        if self.vindplaatsen.iter().any(|v| v.verwerking_id == verwerking_id) {
            return Err(DomeinFout::OngeldigeWaarde {
                veld: "verzoek.vindplaatsen".into(),
                reden: format!("'{kenmerk}' staat al in de lijst"),
            });
        }
        self.vindplaatsen.push(Vindplaats {
            verwerking_id,
            kenmerk,
            omschrijving: omschrijving.into(),
            uitkomst: None,
            toelichting: None,
            anonimiseringstoets: None,
            afgehandeld_op: None,
        });
        self.herkomst.wijzig("vindplaats toegevoegd", op);
        Ok(())
    }

    /// Legt de uitkomst voor één vindplaats vast.
    ///
    /// De uitkomst *geanonimiseerd* vergt een afgeronde toets; zonder die toets
    /// valt zij terug op *gepseudonimiseerd*, en dat zijn nog steeds
    /// persoonsgegevens. De tool doet dat zichtbaar en niet stilzwijgend.
    pub fn stel_vindplaats_vast(
        &mut self,
        kenmerk: &str,
        uitkomst: Vindplaatsuitkomst,
        toelichting: Option<String>,
        toets: Option<Anonimiseringstoets>,
        op: DateTime<Utc>,
    ) -> Resultaat<Vindplaatsuitkomst> {
        let plaats =
            self.vindplaatsen.iter_mut().find(|v| v.kenmerk == kenmerk).ok_or_else(|| {
                DomeinFout::OntbrekendeVerwijzing {
                    veld: "verzoek.vindplaatsen".into(),
                    naar: format!("een vindplaats met kenmerk '{kenmerk}'"),
                }
            })?;

        let werkelijk = match (uitkomst, &toets) {
            (Vindplaatsuitkomst::Geanonimiseerd, Some(t)) if t.is_geslaagd() => {
                Vindplaatsuitkomst::Geanonimiseerd
            }
            // Terugval, en geen weigering: de handeling ís verricht, alleen niet
            // met het gevolg dat de invuller eraan verbond.
            (Vindplaatsuitkomst::Geanonimiseerd, _) => Vindplaatsuitkomst::Gepseudonimiseerd,
            (andere, _) => andere,
        };

        plaats.uitkomst = Some(werkelijk);
        plaats.toelichting = toelichting;
        plaats.anonimiseringstoets = toets;
        plaats.afgehandeld_op = Some(op);
        self.herkomst.wijzig(format!("vindplaats {kenmerk} afgehandeld"), op);
        Ok(werkelijk)
    }

    /// Voegt een ontvanger toe die op grond van artikel 19 bericht moet krijgen.
    pub fn voeg_kennisgeving_toe(
        &mut self,
        ontvanger: impl Into<String>,
        op: DateTime<Utc>,
    ) -> Resultaat<()> {
        let ontvanger = ontvanger.into();
        if self.kennisgevingen.iter().any(|k| k.ontvanger == ontvanger) {
            return Err(DomeinFout::OngeldigeWaarde {
                veld: "verzoek.kennisgevingen".into(),
                reden: format!("'{ontvanger}' staat al in de lijst"),
            });
        }
        self.kennisgevingen.push(Ontvangerkennisgeving {
            ontvanger,
            verzonden_op: None,
            wijze: None,
            onmogelijk_of_onevenredig: false,
            motivering: None,
        });
        self.herkomst.wijzig("ontvanger toegevoegd", op);
        Ok(())
    }

    /// Legt vast dat een ontvanger is bericht, of waarom dat niet kan.
    pub fn leg_kennisgeving_vast(
        &mut self,
        ontvanger: &str,
        verzonden_op: Option<DateTime<Utc>>,
        wijze: Option<String>,
        onmogelijk: bool,
        motivering: Option<Motivering>,
        op: DateTime<Utc>,
    ) -> Resultaat<()> {
        if onmogelijk && motivering.is_none() {
            return Err(DomeinFout::OngeldigeWaarde {
                veld: "verzoek.kennisgevingen".into(),
                reden: "schrijf op waarom de kennisgeving onmogelijk is of onevenredig veel \
                        moeite kost; zonder die reden is het geen uitzondering maar een omissie"
                    .into(),
            });
        }
        if let Some(moment) = verzonden_op {
            if moment > op {
                return Err(DomeinFout::OnmogelijkTijdstip {
                    veld: "verzoek.kennisgevingen".into(),
                    reden: "de kennisgeving zou in de toekomst zijn verzonden".into(),
                });
            }
        }
        let k =
            self.kennisgevingen.iter_mut().find(|k| k.ontvanger == ontvanger).ok_or_else(|| {
                DomeinFout::OntbrekendeVerwijzing {
                    veld: "verzoek.kennisgevingen".into(),
                    naar: format!("een ontvanger '{ontvanger}'"),
                }
            })?;
        k.verzonden_op = verzonden_op;
        k.wijze = wijze;
        k.onmogelijk_of_onevenredig = onmogelijk;
        k.motivering = motivering;
        self.herkomst.wijzig(format!("kennisgeving aan {ontvanger} vastgelegd"), op);
        Ok(())
    }

    /// Handelt het verzoek af.
    ///
    /// Weigert zolang er iets openstaat dat de betrokkene raakt: een vindplaats
    /// zonder uitkomst, een ontvanger zonder bericht en zonder reden
    /// (invariant I18), of een weigering zonder het bericht van artikel 12
    /// lid 4 met de klacht- en beroepsmogelijkheid erin.
    pub fn handel_af(
        &mut self,
        uitkomst: Verzoekuitkomst,
        moment: DateTime<Utc>,
        op: DateTime<Utc>,
    ) -> Resultaat<()> {
        if moment > op {
            return Err(DomeinFout::OnmogelijkTijdstip {
                veld: "verzoek.afgehandeld_op".into(),
                reden: "de afhandeling zou in de toekomst liggen; controleer het tijdstip".into(),
            });
        }

        let mut belet = Vec::new();
        let open_vind = self.openstaande_vindplaatsen().len();
        if open_vind > 0 {
            belet.push(format!(
                "{open_vind} vindplaats(en) zonder uitkomst; wat niet is doorzocht, is niet \
                 stilzwijgend leeg (art. 15 lid 1 AVG)"
            ));
        }
        let open_kennis = self.openstaande_kennisgevingen().len();
        if open_kennis > 0 {
            belet.push(format!(
                "{open_kennis} ontvanger(s) zonder bericht en zonder reden waarom dat niet kan \
                 (art. 19 AVG)"
            ));
        }
        if uitkomst.vraagt_bericht_lid4() {
            match &self.bericht_lid4 {
                None => belet.push(
                    "het bericht van art. 12 lid 4 is nog niet verzonden; bij een geheel of \
                     gedeeltelijke weigering heeft de betrokkene recht op de redenen"
                        .into(),
                ),
                Some(b) if !b.is_volledig() => belet.push(
                    "het bericht van art. 12 lid 4 noemt niet zowel het klachtrecht bij de \
                     toezichthouder als de mogelijkheid van een beroep bij de rechter"
                        .into(),
                ),
                Some(_) => {}
            }
        }

        if !belet.is_empty() {
            return Err(DomeinFout::NietVolledig { soort: "verzoek".into(), ontbreekt: belet });
        }

        if let Some(klok) = self.termijn.as_mut() {
            klok.rond_af(moment);
        }
        self.uitkomst = Some(uitkomst);
        self.afgehandeld_op = Some(moment);
        self.status = Status::Vastgesteld;
        self.herkomst.stel_vast(self.behandelaar.clone(), op);
        Ok(())
    }

    /// Legt het bericht van artikel 12 lid 4 vast.
    pub fn leg_bericht_lid4_vast(
        &mut self,
        verzonden_op: DateTime<Utc>,
        noemt_klachtrecht: bool,
        noemt_rechtsmiddel: bool,
        redenen: Motivering,
        op: DateTime<Utc>,
    ) -> Resultaat<()> {
        if verzonden_op > op {
            return Err(DomeinFout::OnmogelijkTijdstip {
                veld: "verzoek.bericht_lid4".into(),
                reden: "het bericht zou in de toekomst zijn verzonden".into(),
            });
        }
        self.bericht_lid4 =
            Some(BerichtLid4 { verzonden_op, noemt_klachtrecht, noemt_rechtsmiddel, redenen });
        self.herkomst.wijzig("bericht art. 12 lid 4 vastgelegd", op);
        Ok(())
    }
}

impl Volledig for Betrokkenenverzoek {
    fn soortnaam(&self) -> &'static str {
        "verzoek"
    }

    fn aantal_verplichte_onderdelen(&self) -> usize {
        // De intake: lezing van de termijn, de lopende klok, en ten minste één
        // vindplaats. Het kanaal en het ontvangstmoment staan er al bij het
        // aanmaken in en tellen dus niet als openstaand onderdeel.
        let mut verplicht = 3;
        // Elke vindplaats moet een uitkomst krijgen.
        verplicht += self.vindplaatsen.len();
        // Elke ontvanger moet bericht krijgen of een reden waarom niet.
        verplicht += self.kennisgevingen.len();
        // De uitkomst van het verzoek als geheel.
        verplicht += 1;
        // En bij een weigering het bericht van lid 4.
        if self.uitkomst.is_some_and(|u| u.vraagt_bericht_lid4()) {
            verplicht += 1;
        }
        verplicht
    }

    fn ontbrekende_onderdelen(&self) -> Vec<Ontbrekend> {
        let mut uit = Vec::new();

        if self.lezing.is_none() {
            uit.push(Ontbrekend::blokkerend(
                "verzoek.lezing",
                "kies vanaf welk moment de termijn loopt; beide lezingen worden getoond met hun \
                 bron, en de keuze wordt met motivering vastgelegd",
                "art. 12 lid 3 AVG; omstreden punt, zie het voorbehoud",
            ));
        }
        if self.termijn.is_none() {
            uit.push(Ontbrekend::blokkerend(
                "verzoek.termijn",
                "start de termijn, zodat de deadline zichtbaar is",
                "art. 12 lid 3 AVG",
            ));
        }
        if self.vindplaatsen.is_empty() {
            uit.push(Ontbrekend::blokkerend(
                "verzoek.vindplaatsen",
                "leid uit het register af waar gegevens van deze betrokkene kunnen staan; een \
                 verzoek beperken tot één systeem is de meest gemaakte fout",
                "art. 15 lid 1 AVG",
            ));
        }

        for v in self.openstaande_vindplaatsen() {
            uit.push(Ontbrekend::blokkerend(
                format!("verzoek.vindplaats.{}", v.kenmerk),
                format!("leg vast wat er op '{}' met de gegevens is gebeurd", v.kenmerk),
                "art. 15 lid 1 AVG",
            ));
        }
        for k in self.openstaande_kennisgevingen() {
            uit.push(Ontbrekend::blokkerend(
                format!("verzoek.kennisgeving.{}", k.ontvanger),
                format!(
                    "bericht ontvanger '{}', of leg vast waarom dat onmogelijk is of onevenredig \
                     veel moeite kost",
                    k.ontvanger
                ),
                "art. 19 AVG",
            ));
        }

        if self.uitkomst.is_none() {
            uit.push(Ontbrekend::blokkerend(
                "verzoek.uitkomst",
                "leg vast of aan het verzoek is voldaan, deels is voldaan of dat het is geweigerd",
                "art. 12 lid 3 AVG",
            ));
        }
        if self.uitkomst.is_some_and(|u| u.vraagt_bericht_lid4()) && self.bericht_lid4.is_none() {
            uit.push(Ontbrekend::blokkerend(
                "verzoek.bericht_lid4",
                "stuur het bericht met de redenen, het klachtrecht bij de toezichthouder en de \
                 mogelijkheid van beroep bij de rechter",
                "art. 12 lid 4 AVG",
            ));
        }

        // Signalerend: de betrokkene heeft recht op mededeling wélke ontvangers
        // zijn geïnformeerd, maar alleen wanneer hij daarom vraagt.
        if !self.kennisgevingen.is_empty() && self.ontvangers_medegedeeld_op.is_none() {
            uit.push(Ontbrekend::signalerend(
                "verzoek.ontvangers_medegedeeld",
                "deel de betrokkene desgevraagd mee welke ontvangers zijn geïnformeerd",
                "art. 19, tweede volzin, AVG",
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

    fn verzoek() -> Betrokkenenverzoek {
        Betrokkenenverzoek::nieuw(
            "VZ-2026-014",
            "inzageverzoek van een oud-medewerker",
            Verzoeksoort::Inzage,
            Verzoekkanaal::Email,
            nu(),
            "A. de Vries",
            "u1",
            nu(),
        )
    }

    #[test]
    fn een_leeg_verzoek_vraagt_om_lezing_termijn_en_vindplaatsen() {
        let v = verzoek();
        let velden: Vec<_> = v.ontbrekende_onderdelen().into_iter().map(|o| o.veld).collect();
        assert!(velden.contains(&"verzoek.lezing".to_string()));
        assert!(velden.contains(&"verzoek.termijn".to_string()));
        assert!(velden.contains(&"verzoek.vindplaatsen".to_string()));
    }

    /// De omstreden lezing wordt niet in de motor gebakken: zij is een keuze
    /// met een motivering.
    #[test]
    fn de_lezing_bepaalt_het_anker() {
        let mut v = verzoek();
        assert_eq!(v.anker(), None, "zonder keuze is er geen anker");

        v.kies_lezing(Termijnlezing::VanafOntvangst, motivering("ruimste lezing gekozen"), nu())
            .unwrap();
        assert_eq!(v.anker(), Some(nu()));
    }

    #[test]
    fn de_tweede_lezing_vergt_een_vastgestelde_identiteit() {
        let mut v = verzoek();
        let fout = v
            .kies_lezing(
                Termijnlezing::VanafIdentiteitsvaststelling,
                motivering("gerede twijfel over de identiteit"),
                nu(),
            )
            .unwrap_err();
        assert!(fout.to_string().contains("leg dat moment eerst vast"));

        let later = nu() + chrono::Duration::days(3);
        v.stel_identiteit_vast(later, later).unwrap();
        v.kies_lezing(
            Termijnlezing::VanafIdentiteitsvaststelling,
            motivering("gerede twijfel over de identiteit"),
            later,
        )
        .unwrap();
        assert_eq!(v.anker(), Some(later));
    }

    #[test]
    fn beide_lezingen_dragen_hun_eigen_bron() {
        for lezing in Termijnlezing::alle() {
            assert!(lezing.bron().contains("art. 12"), "{lezing:?} mist een bron");
        }
    }

    #[test]
    fn een_identiteit_voor_het_verzoek_wordt_geweigerd() {
        let mut v = verzoek();
        let fout = v.stel_identiteit_vast(nu() - chrono::Duration::days(1), nu()).unwrap_err();
        assert!(fout.to_string().contains("verwisseld"));
    }

    /// "Account gesloten" is geen antwoord op een wisverzoek. Dat is hier geen
    /// waarschuwing maar een onmogelijkheid: de opsomming kent die waarde niet.
    #[test]
    fn geanonimiseerd_valt_zonder_toets_terug_op_gepseudonimiseerd() {
        let mut v = verzoek();
        v.voeg_vindplaats_toe(Id::nieuw(), "0412-K", "Verzuimregistratie", nu()).unwrap();

        let uitkomst = v
            .stel_vindplaats_vast("0412-K", Vindplaatsuitkomst::Geanonimiseerd, None, None, nu())
            .unwrap();
        assert_eq!(uitkomst, Vindplaatsuitkomst::Gepseudonimiseerd);
        assert!(uitkomst.omschrijving().contains("nog persoonsgegevens"));
    }

    #[test]
    fn geanonimiseerd_blijft_staan_met_een_geslaagde_toets() {
        let mut v = verzoek();
        v.voeg_vindplaats_toe(Id::nieuw(), "0412-K", "Verzuimregistratie", nu()).unwrap();
        let toets = Anonimiseringstoets {
            singling_out_uitgesloten: true,
            koppelbaarheid_uitgesloten: true,
            afleidbaarheid_uitgesloten: true,
            motivering: motivering("aggregatie tot groepen van minimaal twintig"),
            bevestigd_door: "B. Jansen".into(),
        };
        let uitkomst = v
            .stel_vindplaats_vast(
                "0412-K",
                Vindplaatsuitkomst::Geanonimiseerd,
                None,
                Some(toets),
                nu(),
            )
            .unwrap();
        assert_eq!(uitkomst, Vindplaatsuitkomst::Geanonimiseerd);
    }

    #[test]
    fn een_toets_met_een_open_punt_valt_ook_terug() {
        let mut v = verzoek();
        v.voeg_vindplaats_toe(Id::nieuw(), "0412-K", "Verzuimregistratie", nu()).unwrap();
        let toets = Anonimiseringstoets {
            singling_out_uitgesloten: true,
            koppelbaarheid_uitgesloten: false,
            afleidbaarheid_uitgesloten: true,
            motivering: motivering("koppeling met het personeelssysteem blijft mogelijk"),
            bevestigd_door: "B. Jansen".into(),
        };
        let uitkomst = v
            .stel_vindplaats_vast(
                "0412-K",
                Vindplaatsuitkomst::Geanonimiseerd,
                None,
                Some(toets),
                nu(),
            )
            .unwrap();
        assert_eq!(uitkomst, Vindplaatsuitkomst::Gepseudonimiseerd);
    }

    /// Invariant I18: geen afsluiting zolang een ontvanger openstaat.
    #[test]
    fn afsluiten_wordt_geweigerd_zolang_een_ontvanger_openstaat() {
        let mut v = verzoek();
        v.voeg_vindplaats_toe(Id::nieuw(), "0412-K", "Verzuimregistratie", nu()).unwrap();
        v.stel_vindplaats_vast("0412-K", Vindplaatsuitkomst::Verstrekt, None, None, nu()).unwrap();
        v.voeg_kennisgeving_toe("het pensioenfonds", nu()).unwrap();

        let fout = v.handel_af(Verzoekuitkomst::Voldaan, nu(), nu()).unwrap_err();
        assert!(fout.to_string().contains("art. 19 AVG"), "kreeg: {fout}");
    }

    #[test]
    fn een_onmogelijke_kennisgeving_vergt_een_reden() {
        let mut v = verzoek();
        v.voeg_kennisgeving_toe("een opgeheven stichting", nu()).unwrap();
        let fout = v
            .leg_kennisgeving_vast("een opgeheven stichting", None, None, true, None, nu())
            .unwrap_err();
        assert!(fout.to_string().contains("onevenredig"));

        v.leg_kennisgeving_vast(
            "een opgeheven stichting",
            None,
            None,
            true,
            Some(motivering("de stichting is opgeheven en heeft geen rechtsopvolger")),
            nu(),
        )
        .unwrap();
        assert!(v.openstaande_kennisgevingen().is_empty());
    }

    #[test]
    fn afsluiten_wordt_geweigerd_zolang_een_vindplaats_openstaat() {
        let mut v = verzoek();
        v.voeg_vindplaats_toe(Id::nieuw(), "0412-K", "Verzuimregistratie", nu()).unwrap();
        let fout = v.handel_af(Verzoekuitkomst::Voldaan, nu(), nu()).unwrap_err();
        assert!(fout.to_string().contains("niet stilzwijgend leeg"), "kreeg: {fout}");
    }

    #[test]
    fn een_weigering_vergt_het_bericht_van_lid_vier() {
        let mut v = verzoek();
        v.voeg_vindplaats_toe(Id::nieuw(), "0412-K", "Verzuimregistratie", nu()).unwrap();
        v.stel_vindplaats_vast("0412-K", Vindplaatsuitkomst::Geweigerd, None, None, nu()).unwrap();

        let fout = v.handel_af(Verzoekuitkomst::Geweigerd, nu(), nu()).unwrap_err();
        assert!(fout.to_string().contains("art. 12 lid 4"));
    }

    #[test]
    fn een_bericht_zonder_klachtrecht_telt_niet() {
        let mut v = verzoek();
        v.voeg_vindplaats_toe(Id::nieuw(), "0412-K", "Verzuimregistratie", nu()).unwrap();
        v.stel_vindplaats_vast("0412-K", Vindplaatsuitkomst::Geweigerd, None, None, nu()).unwrap();
        v.leg_bericht_lid4_vast(nu(), false, true, motivering("kennelijk ongegrond"), nu())
            .unwrap();

        let fout = v.handel_af(Verzoekuitkomst::Geweigerd, nu(), nu()).unwrap_err();
        assert!(fout.to_string().contains("klachtrecht"));

        v.leg_bericht_lid4_vast(nu(), true, true, motivering("kennelijk ongegrond"), nu()).unwrap();
        assert!(v.handel_af(Verzoekuitkomst::Geweigerd, nu(), nu()).is_ok());
    }

    #[test]
    fn een_verlenging_vergt_een_lopende_termijn() {
        let mut v = verzoek();
        let fout = v
            .leg_verlenging_vast(
                Verlengingsgrond::Complexiteit,
                nu(),
                motivering("het verzoek raakt zeven systemen"),
                nu(),
            )
            .unwrap_err();
        assert!(matches!(fout, DomeinFout::OntbrekendeVerwijzing { .. }));
    }

    #[test]
    fn dezelfde_vindplaats_komt_er_niet_twee_keer_in() {
        let mut v = verzoek();
        let id = Id::nieuw();
        v.voeg_vindplaats_toe(id, "0412-K", "Verzuimregistratie", nu()).unwrap();
        assert!(v.voeg_vindplaats_toe(id, "0412-K", "Verzuimregistratie", nu()).is_err());
    }

    #[test]
    fn elke_soort_draagt_een_grondslag() {
        for soort in Verzoeksoort::alle() {
            assert!(soort.grondslag().starts_with("art."), "{soort:?} mist een grondslag");
        }
        assert!(Verzoeksoort::Rectificatie.vraagt_kennisgeving_aan_ontvangers());
        assert!(!Verzoeksoort::Inzage.vraagt_kennisgeving_aan_ontvangers());
    }

    #[test]
    fn het_verzoek_overleeft_serialisatie() {
        let mut v = verzoek();
        v.kies_lezing(Termijnlezing::VanafOntvangst, motivering("ruimste lezing"), nu()).unwrap();
        v.voeg_vindplaats_toe(Id::nieuw(), "0412-K", "Verzuimregistratie", nu()).unwrap();

        let json = serde_json::to_string(&v).unwrap();
        let terug: Betrokkenenverzoek = serde_json::from_str(&json).unwrap();
        assert_eq!(v, terug);
    }
}
