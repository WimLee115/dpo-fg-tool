//! Het kennispakket: alle juridische inhoud, buiten de programmacode.
//!
//! # Waarom dit los van de binary staat
//!
//! Ontwerpprincipe P1 uit het plan: wetteksten, artikelnummers, termijnen,
//! drempels, feestdagenkalenders, autoriteiten, meldkanalen en **alle datums**
//! zitten in ondertekende pakketten met versie en consolidatiedatum.
//!
//! De reden is praktisch. Wetgeving verandert sneller dan een
//! softwarerelease. Een termijn die in de binary staat, betekent dat een
//! organisatie een nieuwe uitgave van het programma nodig heeft om een
//! wettelijke termijn juist te berekenen — en tot die tijd rekent zij fout
//! zonder het te weten. Een pakket kan binnen een dag worden uitgebracht en
//! geïnstalleerd, zonder de toepassing zelf aan te raken.
//!
//! # Wat dit betekent voor de verantwoording
//!
//! Elk pakket draagt een **consolidatiedatum**: tot welke datum de inhoud is
//! bijgewerkt. Die datum staat in elke export en elk auditdossier. Een
//! toezichthouder kan daarmee zien op welke stand van het recht een berekening
//! berust — en de organisatie kan aantonen dat zij niet met verouderde
//! inhoud werkte.

use chrono::{DateTime, NaiveDate, Utc};
use dpofg_terms::{Feestdagenkalender, Termijnsoort};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::{ContentFout, Resultaat};

/// Vaste tekst die meegetekend wordt, zodat een handtekening uit dit product
/// niet als handtekening van iets anders kan worden aangeboden.
const PAKKETCONTEXT: &[u8] = b"dpo-fg-tool kennispakket v1";

/// Een rechtsfeit: een datum die uit de wet volgt.
///
/// Datums staan nooit los in het model; er wordt naar een code verwezen. Zo
/// hoeft bij uitstel van een inwerkingtreding maar één waarde te wijzigen in
/// plaats van tientallen verspreide datums.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rechtsfeit {
    pub code: String,
    pub omschrijving: String,
    pub datum: NaiveDate,
    pub bron: String,
}

/// Een doorgifte-instrument met zijn geldigheid.
///
/// De meest volatiele juridische inhoud die er is: adequaatheidsbesluiten
/// worden vernieuwd, betwist, opgeschort en ingetrokken. Een statuswijziging
/// hier zet alle betrokken doorgiften op herbeoordelen.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Doorgifteinstrument {
    pub code: String,
    pub land_of_gebied: String,
    pub besluit_ref: String,
    pub status: Instrumentstatus,
    pub vastgesteld_op: NaiveDate,
    pub geldig_tot: Option<NaiveDate>,
    pub geverifieerd_op: NaiveDate,
    pub toelichting: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Instrumentstatus {
    Geldig,
    OnderToetsing,
    Ingetrokken,
    Vernieuwd,
}

impl Instrumentstatus {
    /// Of doorgiften op dit instrument opnieuw moeten worden beoordeeld.
    pub fn vereist_herbeoordeling(&self) -> bool {
        matches!(self, Self::OnderToetsing | Self::Ingetrokken)
    }
}

/// De inhoud van een kennispakket.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Pakketinhoud {
    /// Vaste code, bijvoorbeeld `nl-basis`.
    pub code: String,
    /// Leesbare naam.
    pub naam: String,
    /// Oplopende versie. Bepaalt of installeren een terugrol zou zijn.
    pub versie: u32,
    /// Leesbare versieaanduiding.
    pub versienaam: String,
    /// Tot welke datum de inhoud is bijgewerkt.
    pub consolidatiedatum: NaiveDate,
    /// Het rechtsgebied.
    pub jurisdictie: String,
    /// De laagste programmaversie waarmee dit pakket veilig werkt.
    ///
    /// Zonder automatische bijwerking is dit het enige middel om een
    /// organisatie te bereiken die een uitgave met een bekend gebrek draait:
    /// het pakket weigert zichzelf en zegt waarom.
    pub minimaal_aanbevolen_programmaversie: String,

    pub termijnen: Vec<Termijnsoort>,
    pub feestdagen: Vec<Feestdagenkalender>,
    pub rechtsfeiten: Vec<Rechtsfeit>,
    pub doorgifteinstrumenten: Vec<Doorgifteinstrument>,
    /// Vrije aanvullende inhoud: sjablonen, beslisbomen, standaardteksten.
    pub aanvullend: BTreeMap<String, serde_json::Value>,
}

impl Pakketinhoud {
    /// De bytes die worden ondertekend.
    ///
    /// Canoniek geserialiseerd: dezelfde inhoud levert altijd dezelfde bytes,
    /// anders is de handtekening niet reproduceerbaar.
    fn te_ondertekenen(&self) -> Resultaat<Vec<u8>> {
        let json =
            serde_json::to_vec(self).map_err(|e| ContentFout::OngeldigFormaat(e.to_string()))?;
        let mut uit = Vec::with_capacity(PAKKETCONTEXT.len() + 8 + json.len());
        uit.extend_from_slice(PAKKETCONTEXT);
        uit.extend_from_slice(&(json.len() as u64).to_be_bytes());
        uit.extend_from_slice(&json);
        Ok(uit)
    }

    /// Zoekt een termijnsoort op code.
    pub fn termijn(&self, code: &str) -> Resultaat<&Termijnsoort> {
        self.termijnen.iter().find(|t| t.code == code).ok_or_else(|| ContentFout::OnbekendeCode {
            soort: "termijn".into(),
            code: code.to_string(),
        })
    }

    /// Zoekt de feestdagenkalender voor een rechtsgebied.
    pub fn kalender(&self, jurisdictie: &str) -> Resultaat<&Feestdagenkalender> {
        self.feestdagen.iter().find(|k| k.jurisdictie == jurisdictie).ok_or_else(|| {
            ContentFout::OnbekendeCode {
                soort: "feestdagenkalender".into(),
                code: jurisdictie.to_string(),
            }
        })
    }

    /// Zoekt een rechtsfeit op code.
    pub fn rechtsfeit(&self, code: &str) -> Resultaat<&Rechtsfeit> {
        self.rechtsfeiten.iter().find(|r| r.code == code).ok_or_else(|| {
            ContentFout::OnbekendeCode { soort: "rechtsfeit".into(), code: code.to_string() }
        })
    }

    /// Zoekt een doorgifte-instrument op code.
    pub fn instrument(&self, code: &str) -> Resultaat<&Doorgifteinstrument> {
        self.doorgifteinstrumenten.iter().find(|d| d.code == code).ok_or_else(|| {
            ContentFout::OnbekendeCode {
                soort: "doorgifte-instrument".into(),
                code: code.to_string(),
            }
        })
    }

    /// Alle instrumenten die om herbeoordeling vragen.
    pub fn instrumenten_met_herbeoordeling(&self) -> Vec<&Doorgifteinstrument> {
        self.doorgifteinstrumenten.iter().filter(|d| d.status.vereist_herbeoordeling()).collect()
    }

    /// Hoeveel dagen geleden de inhoud is bijgewerkt.
    pub fn ouderdom_in_dagen(&self, nu: DateTime<Utc>) -> i64 {
        (nu.date_naive() - self.consolidatiedatum).num_days()
    }

    /// Waarschuwt wanneer de inhoud te oud wordt.
    ///
    /// Geen blokkade: doorwerken met een oud pakket is beter dan niet kunnen
    /// werken. Wel zichtbaar, en de consolidatiedatum staat in elke export.
    pub fn controleer_ouderdom(&self, nu: DateTime<Utc>, maximaal_dagen: i64) -> Resultaat<()> {
        let dagen = self.ouderdom_in_dagen(nu);
        if dagen > maximaal_dagen {
            return Err(ContentFout::Verouderd {
                consolidatiedatum: self.consolidatiedatum.to_string(),
                dagen,
            });
        }
        Ok(())
    }
}

/// Een ondertekend kennispakket.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Kennispakket {
    pub inhoud: Pakketinhoud,
    /// De publieke sleutel van de uitgever, hexadecimaal.
    pub uitgever: String,
    /// De handtekening, hexadecimaal.
    pub handtekening: String,
}

impl Kennispakket {
    /// Ondertekent inhoud tot een pakket.
    pub fn onderteken(inhoud: Pakketinhoud, sleutel: &SigningKey) -> Resultaat<Self> {
        let boodschap = inhoud.te_ondertekenen()?;
        let handtekening: Signature = sleutel.sign(&boodschap);
        Ok(Self {
            inhoud,
            uitgever: hex::encode(sleutel.verifying_key().to_bytes()),
            handtekening: hex::encode(handtekening.to_bytes()),
        })
    }

    /// Controleert de handtekening tegen een lijst van vertrouwde uitgevers.
    ///
    /// De lijst is verplicht: een handtekening die alleen "klopt met de
    /// bijgeleverde sleutel" bewijst niets, want die sleutel komt uit hetzelfde
    /// bestand. Alleen een vooraf vertrouwde sleutel zegt iets.
    pub fn controleer(&self, vertrouwde_uitgevers: &[String]) -> Resultaat<()> {
        if !vertrouwde_uitgevers.iter().any(|u| u == &self.uitgever) {
            return Err(ContentFout::OnbekendeUitgever { sleutel: self.uitgever.clone() });
        }

        let sleutelbytes = hex::decode(&self.uitgever)
            .map_err(|e| ContentFout::OngeldigeHandtekening(format!("sleutel is geen hex: {e}")))?;
        let sleutelarray: [u8; 32] = sleutelbytes.try_into().map_err(|_| {
            ContentFout::OngeldigeHandtekening("sleutel heeft niet de juiste lengte".into())
        })?;
        let vk = VerifyingKey::from_bytes(&sleutelarray)
            .map_err(|e| ContentFout::OngeldigeHandtekening(e.to_string()))?;

        let hbytes = hex::decode(&self.handtekening).map_err(|e| {
            ContentFout::OngeldigeHandtekening(format!("handtekening is geen hex: {e}"))
        })?;
        let harray: [u8; 64] = hbytes.try_into().map_err(|_| {
            ContentFout::OngeldigeHandtekening("handtekening heeft niet de juiste lengte".into())
        })?;
        let sig = Signature::from_bytes(&harray);

        let boodschap = self.inhoud.te_ondertekenen()?;
        vk.verify(&boodschap, &sig)
            .map_err(|_| ContentFout::OngeldigeHandtekening("de inhoud is gewijzigd".into()))
    }

    /// Controleert of installeren geen terugrol zou zijn.
    pub fn controleer_volgorde(&self, huidige: Option<&Pakketinhoud>) -> Resultaat<()> {
        if let Some(h) = huidige {
            if h.code == self.inhoud.code && self.inhoud.versie < h.versie {
                return Err(ContentFout::Terugrol {
                    huidig: format!("{} ({})", h.versienaam, h.versie),
                    aangeboden: format!("{} ({})", self.inhoud.versienaam, self.inhoud.versie),
                });
            }
        }
        Ok(())
    }

    /// Volledige controle voorafgaand aan installeren.
    pub fn controleer_voor_installatie(
        &self,
        vertrouwde_uitgevers: &[String],
        huidige: Option<&Pakketinhoud>,
    ) -> Resultaat<()> {
        self.controleer(vertrouwde_uitgevers)?;
        self.controleer_volgorde(huidige)?;
        Ok(())
    }
}

/// Maakt een nieuw uitgeverssleutelpaar aan.
pub fn nieuw_uitgeverspaar() -> SigningKey {
    SigningKey::generate(&mut rand::rngs::OsRng)
}
