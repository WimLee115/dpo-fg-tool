//! Ankers: het enige middel tegen afkappen van de keten.
//!
//! Een anker is een korte, ondertekende verklaring met de strekking: *op dit
//! tijdstip stond deze keten op volgnummer N met hash H*. Het anker is klein
//! genoeg om buiten het systeem te bewaren — in de notulen van een
//! bestuursvergadering, in een e-mail aan de accountant, bij een notaris of in
//! een openbaar tijdstempelregister.
//!
//! # Wat een anker wel en niet bewijst
//!
//! | Vraag | Antwoord |
//! |---|---|
//! | Zijn er regels ná het anker weggegooid? | Ja, dat is zichtbaar: de keten is korter dan het anker aangeeft |
//! | Is er vóór het anker iets gewijzigd? | Ja, de hash klopt dan niet meer |
//! | Wanneer is regel N geschreven? | Alleen dat het vóór de ankerdatum was — en die datum is zo betrouwbaar als de plaats waar het anker is bewaard |
//! | Zijn er regels ná het láátste anker weggegooid? | **Nee.** Die ruimte blijft altijd bestaan; alleen vaker ankeren maakt hem kleiner |
//!
//! Dat laatste is de reden dat de ankerfrequentie een instelling is en dat het
//! verificatierapport altijd vermeldt hoe lang geleden het laatste anker is
//! geplaatst.

use chrono::{DateTime, Utc};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};

use crate::{AuditFout, Ketenstand, Resultaat};

/// Vaste tekst die meegetekend wordt, zodat een handtekening uit dit product
/// niet als handtekening van iets anders kan worden aangeboden.
const ANKERCONTEXT: &[u8] = b"dpo-fg-tool ketenanker v1";

/// Een ondertekende momentopname van de keten.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Anker {
    /// Identificatie van de kluis waarop dit anker betrekking heeft.
    pub kluis_id: String,
    /// Volgnummer van de laatste regel op het moment van ankeren.
    pub volgnummer: u64,
    /// Hash van die laatste regel.
    pub hash: String,
    /// Tijdstip waarop het anker is gemaakt, volgens de machine zelf.
    pub tijdstip: DateTime<Utc>,
    /// De publieke sleutel waarmee de handtekening te controleren is.
    pub sleutel: String,
    /// De handtekening, hexadecimaal.
    pub handtekening: String,
    /// Vrije aanduiding van de plaats waar dit anker buiten het systeem is
    /// vastgelegd, bijvoorbeeld "notulen directieoverleg 2026-08-18".
    pub bewaarplaats: Option<String>,
}

impl Anker {
    /// De bytes die daadwerkelijk worden ondertekend.
    fn te_ondertekenen(
        kluis_id: &str,
        volgnummer: u64,
        hash: &str,
        tijdstip: DateTime<Utc>,
    ) -> Vec<u8> {
        let mut uit = Vec::with_capacity(128);
        uit.extend_from_slice(ANKERCONTEXT);
        uit.extend_from_slice(&(kluis_id.len() as u32).to_be_bytes());
        uit.extend_from_slice(kluis_id.as_bytes());
        uit.extend_from_slice(&volgnummer.to_be_bytes());
        uit.extend_from_slice(&(hash.len() as u32).to_be_bytes());
        uit.extend_from_slice(hash.as_bytes());
        uit.extend_from_slice(&tijdstip.timestamp().to_be_bytes());
        uit.extend_from_slice(&tijdstip.timestamp_subsec_nanos().to_be_bytes());
        uit
    }

    /// Plaatst een anker op de huidige ketenstand.
    pub fn plaats(
        sleutel: &SigningKey,
        kluis_id: impl Into<String>,
        stand: &Ketenstand,
        tijdstip: DateTime<Utc>,
    ) -> Resultaat<Self> {
        if stand.is_leeg() {
            return Err(AuditFout::LeegLogboek);
        }
        let kluis_id = kluis_id.into();
        let boodschap =
            Self::te_ondertekenen(&kluis_id, stand.volgnummer, &stand.hash, tijdstip);
        let handtekening: Signature = sleutel.sign(&boodschap);
        Ok(Self {
            kluis_id,
            volgnummer: stand.volgnummer,
            hash: stand.hash.clone(),
            tijdstip,
            sleutel: hex::encode(sleutel.verifying_key().to_bytes()),
            handtekening: hex::encode(handtekening.to_bytes()),
            bewaarplaats: None,
        })
    }

    pub fn met_bewaarplaats(mut self, plaats: impl Into<String>) -> Self {
        self.bewaarplaats = Some(plaats.into());
        self
    }

    /// Controleert de handtekening onder dit anker.
    ///
    /// Slaagt deze controle, dan staat vast dat de houder van de bijbehorende
    /// privésleutel deze ketenstand heeft verklaard. Of die verklaring waar is,
    /// blijkt pas uit vergelijking met de werkelijke keten — zie
    /// [`crate::verificatie`].
    pub fn controleer_handtekening(&self) -> Resultaat<()> {
        let sleutelbytes = hex::decode(&self.sleutel)
            .map_err(|e| AuditFout::OngeldigZegel(format!("sleutel is geen hex: {e}")))?;
        let sleutelarray: [u8; 32] = sleutelbytes
            .try_into()
            .map_err(|_| AuditFout::OngeldigZegel("sleutel heeft niet de juiste lengte".into()))?;
        let vk = VerifyingKey::from_bytes(&sleutelarray)
            .map_err(|e| AuditFout::OngeldigZegel(format!("ongeldige publieke sleutel: {e}")))?;

        let hbytes = hex::decode(&self.handtekening)
            .map_err(|e| AuditFout::OngeldigZegel(format!("handtekening is geen hex: {e}")))?;
        let harray: [u8; 64] = hbytes.try_into().map_err(|_| {
            AuditFout::OngeldigZegel("handtekening heeft niet de juiste lengte".into())
        })?;
        let sig = Signature::from_bytes(&harray);

        let boodschap =
            Self::te_ondertekenen(&self.kluis_id, self.volgnummer, &self.hash, self.tijdstip);
        vk.verify(&boodschap, &sig)
            .map_err(|_| AuditFout::OngeldigZegel("handtekening klopt niet".into()))
    }
}

/// Maakt een nieuw ondertekensleutelpaar aan.
pub fn nieuw_sleutelpaar() -> SigningKey {
    SigningKey::generate(&mut rand::rngs::OsRng)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn stand() -> Ketenstand {
        Ketenstand {
            volgnummer: 42,
            hash: "a".repeat(64),
            tijdstip: Some(Utc.with_ymd_and_hms(2026, 8, 18, 9, 0, 0).unwrap()),
        }
    }

    fn nu() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 8, 18, 12, 0, 0).unwrap()
    }

    #[test]
    fn geldig_anker_wordt_aanvaard() {
        let sk = nieuw_sleutelpaar();
        let anker = Anker::plaats(&sk, "kluis-1", &stand(), nu()).unwrap();
        assert!(anker.controleer_handtekening().is_ok());
        assert_eq!(anker.volgnummer, 42);
    }

    #[test]
    fn gewijzigd_volgnummer_verbreekt_de_handtekening() {
        let sk = nieuw_sleutelpaar();
        let mut anker = Anker::plaats(&sk, "kluis-1", &stand(), nu()).unwrap();
        anker.volgnummer = 41;
        assert!(anker.controleer_handtekening().is_err());
    }

    #[test]
    fn gewijzigde_hash_verbreekt_de_handtekening() {
        let sk = nieuw_sleutelpaar();
        let mut anker = Anker::plaats(&sk, "kluis-1", &stand(), nu()).unwrap();
        anker.hash = "b".repeat(64);
        assert!(anker.controleer_handtekening().is_err());
    }

    #[test]
    fn gewijzigd_tijdstip_verbreekt_de_handtekening() {
        let sk = nieuw_sleutelpaar();
        let mut anker = Anker::plaats(&sk, "kluis-1", &stand(), nu()).unwrap();
        anker.tijdstip = Utc.with_ymd_and_hms(2026, 8, 17, 12, 0, 0).unwrap();
        assert!(anker.controleer_handtekening().is_err());
    }

    #[test]
    fn gewijzigde_kluis_id_verbreekt_de_handtekening() {
        let sk = nieuw_sleutelpaar();
        let mut anker = Anker::plaats(&sk, "kluis-1", &stand(), nu()).unwrap();
        anker.kluis_id = "kluis-2".into();
        assert!(anker.controleer_handtekening().is_err());
    }

    #[test]
    fn andere_sleutel_wordt_verworpen() {
        let sk = nieuw_sleutelpaar();
        let mut anker = Anker::plaats(&sk, "kluis-1", &stand(), nu()).unwrap();
        anker.sleutel = hex::encode(nieuw_sleutelpaar().verifying_key().to_bytes());
        assert!(anker.controleer_handtekening().is_err());
    }

    #[test]
    fn leeg_logboek_kan_niet_worden_geankerd() {
        let sk = nieuw_sleutelpaar();
        assert_eq!(
            Anker::plaats(&sk, "kluis-1", &Ketenstand::leeg(), nu()).unwrap_err(),
            AuditFout::LeegLogboek
        );
    }

    #[test]
    fn anker_overleeft_serialisatie() {
        let sk = nieuw_sleutelpaar();
        let anker = Anker::plaats(&sk, "kluis-1", &stand(), nu())
            .unwrap()
            .met_bewaarplaats("notulen directieoverleg 2026-08-18");
        let json = serde_json::to_string(&anker).unwrap();
        let terug: Anker = serde_json::from_str(&json).unwrap();
        assert_eq!(anker, terug);
        assert!(terug.controleer_handtekening().is_ok());
    }
}
