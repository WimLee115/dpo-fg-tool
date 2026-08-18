//! De hashketen: elke regel bindt zich aan alles wat ervoor kwam.
//!
//! # Waarom een keten en geen gewoon logbestand
//!
//! Een logbestand met tijdstempels bewijst niets: wie schrijfrechten heeft, kan
//! een regel wijzigen of verwijderen en niemand ziet het. Een keten lost twee
//! van de drie manipulaties op:
//!
//! | Manipulatie | Detectie |
//! |---|---|
//! | Regel wijzigen | de hash van die regel verandert, waardoor elke volgende schakel niet meer klopt |
//! | Regel verwijderen | het volgnummer ontbreekt én de keten breekt |
//! | Regels aan het eind afkappen | **niet detecteerbaar zonder anker** — zie hieronder |
//!
//! # De grens die eerlijk benoemd moet worden
//!
//! Wie de laatste tien regels weggooit, houdt een keten over die intern perfect
//! klopt. Alleen een **anker** dat buiten het bestand is vastgelegd verraadt
//! dat er ooit meer regels waren. Daarom kent dit logboek ankerpunten, en
//! daarom vermeldt elk uitgevoerd verificatierapport expliciet tot welk anker
//! de vaststelling reikt.
//!
//! Evenmin bewijst de keten *wanneer* een regel is geschreven: de tijdstempel
//! komt van de machine zelf. De keten bewijst de **volgorde**; een extern
//! tijdstempel bewijst het **moment**.

use blake3::Hasher;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{AuditFout, Gebeurtenis, Resultaat};

/// De hash waarmee de keten begint. Vast en openbaar.
pub const GENESIS: &str = "0000000000000000000000000000000000000000000000000000000000000000";

/// Afleidingscontext, zodat een hash uit dit logboek nooit gelijk kan zijn aan
/// een hash die elders in het product met dezelfde bytes wordt berekend.
const CONTEXT: &str = "dpo-fg-tool audit-keten v1";

/// Eén geketende regel: de gebeurtenis plus haar plaats in de keten.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ketenregel {
    /// Volgnummer, begint bij 1 en loopt zonder gaten op.
    pub volgnummer: u64,
    /// De vastgelegde handeling.
    pub gebeurtenis: Gebeurtenis,
    /// Hash van de voorgaande regel; bij de eerste regel [`GENESIS`].
    pub vorige_hash: String,
    /// Hash van deze regel.
    pub hash: String,
}

impl Ketenregel {
    /// Berekent de hash die bij deze regel hoort.
    ///
    /// De berekening omvat het volgnummer en de voorgaande hash. Daardoor kan
    /// een regel niet naar een andere plaats in de keten worden verplaatst.
    pub fn bereken_hash(
        volgnummer: u64,
        gebeurtenis: &Gebeurtenis,
        vorige_hash: &str,
    ) -> Resultaat<String> {
        // Canonieke serialisatie: serde_json sorteert de velden van een struct
        // in declaratievolgorde, wat stabiel is over versies zolang de volgorde
        // niet verandert. De formaatversie hieronder legt dat vast.
        let inhoud = serde_json::to_vec(gebeurtenis)
            .map_err(|e| AuditFout::Serialisatie(e.to_string()))?;

        let mut hasher = Hasher::new_derive_key(CONTEXT);
        hasher.update(&volgnummer.to_be_bytes());
        hasher.update(&(vorige_hash.len() as u32).to_be_bytes());
        hasher.update(vorige_hash.as_bytes());
        hasher.update(&(inhoud.len() as u64).to_be_bytes());
        hasher.update(&inhoud);
        Ok(hasher.finalize().to_hex().to_string())
    }

    /// Controleert of de opgeslagen hash klopt met de inhoud.
    pub fn hash_klopt(&self) -> Resultaat<bool> {
        let berekend =
            Self::bereken_hash(self.volgnummer, &self.gebeurtenis, &self.vorige_hash)?;
        Ok(berekend == self.hash)
    }
}

/// Het lopende einde van de keten.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ketenstand {
    /// Volgnummer van de laatst geschreven regel; 0 wanneer het logboek leeg is.
    pub volgnummer: u64,
    /// Hash van de laatst geschreven regel; [`GENESIS`] wanneer leeg.
    pub hash: String,
    /// Tijdstip van de laatst geschreven regel.
    pub tijdstip: Option<DateTime<Utc>>,
}

impl Ketenstand {
    pub fn leeg() -> Self {
        Self { volgnummer: 0, hash: GENESIS.to_string(), tijdstip: None }
    }

    pub fn is_leeg(&self) -> bool {
        self.volgnummer == 0
    }
}

impl Default for Ketenstand {
    fn default() -> Self {
        Self::leeg()
    }
}

/// Voegt een gebeurtenis toe aan de keten en levert de nieuwe regel plus stand.
///
/// # Terugtellende klok
///
/// Wanneer het tijdstip vóór dat van de voorgaande regel ligt, wordt de regel
/// **wel** geschreven maar faalt de latere verificatie met
/// [`AuditFout::TijdLooptTerug`]. Weigeren zou erger zijn: dan is een
/// verspringende systeemklok een reden om een handeling niet vast te leggen, en
/// het niet-vastleggen van handelingen is precies wat dit logboek moet
/// voorkomen. Vastleggen en zichtbaar maken is het juiste antwoord.
pub fn keten_aan(
    stand: &Ketenstand,
    gebeurtenis: Gebeurtenis,
) -> Resultaat<(Ketenregel, Ketenstand)> {
    let volgnummer = stand.volgnummer + 1;
    let tijdstip = gebeurtenis.tijdstip;
    let hash = Ketenregel::bereken_hash(volgnummer, &gebeurtenis, &stand.hash)?;
    let regel = Ketenregel {
        volgnummer,
        gebeurtenis,
        vorige_hash: stand.hash.clone(),
        hash: hash.clone(),
    };
    let nieuwe_stand = Ketenstand { volgnummer, hash, tijdstip: Some(tijdstip) };
    Ok((regel, nieuwe_stand))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Actor, Handeling};
    use chrono::TimeZone;

    fn gebeurtenis(n: u32) -> Gebeurtenis {
        Gebeurtenis::nieuw(
            Handeling::RecordGewijzigd,
            Actor::nieuw("u1", "A. de Vries", "fg"),
            Utc.with_ymd_and_hms(2026, 8, 18, 9, n, 0).unwrap(),
            "verwerking",
            format!("0412-{n}"),
            "algemeen",
            "bewaartermijn ingevuld",
        )
    }

    #[test]
    fn eerste_regel_verwijst_naar_genesis() {
        let (regel, stand) = keten_aan(&Ketenstand::leeg(), gebeurtenis(1)).unwrap();
        assert_eq!(regel.volgnummer, 1);
        assert_eq!(regel.vorige_hash, GENESIS);
        assert_eq!(stand.volgnummer, 1);
        assert_eq!(stand.hash, regel.hash);
    }

    #[test]
    fn regels_koppelen_aan_elkaar() {
        let (r1, s1) = keten_aan(&Ketenstand::leeg(), gebeurtenis(1)).unwrap();
        let (r2, s2) = keten_aan(&s1, gebeurtenis(2)).unwrap();
        assert_eq!(r2.vorige_hash, r1.hash);
        assert_eq!(r2.volgnummer, 2);
        assert_eq!(s2.hash, r2.hash);
    }

    #[test]
    fn hash_klopt_na_aanmaken() {
        let (regel, _) = keten_aan(&Ketenstand::leeg(), gebeurtenis(1)).unwrap();
        assert!(regel.hash_klopt().unwrap());
    }

    #[test]
    fn gewijzigde_inhoud_breekt_de_hash() {
        let (mut regel, _) = keten_aan(&Ketenstand::leeg(), gebeurtenis(1)).unwrap();
        regel.gebeurtenis.omschrijving = "iets anders".into();
        assert!(!regel.hash_klopt().unwrap());
    }

    #[test]
    fn gewijzigde_motivering_breekt_de_hash() {
        let (mut regel, _) = keten_aan(&Ketenstand::leeg(), gebeurtenis(1)).unwrap();
        regel.gebeurtenis.motivering = Some("achteraf toegevoegd".into());
        assert!(!regel.hash_klopt().unwrap());
    }

    #[test]
    fn gewijzigde_actor_breekt_de_hash() {
        let (mut regel, _) = keten_aan(&Ketenstand::leeg(), gebeurtenis(1)).unwrap();
        regel.gebeurtenis.actor.naam = "iemand anders".into();
        assert!(!regel.hash_klopt().unwrap());
    }

    #[test]
    fn regel_kan_niet_naar_een_andere_plaats() {
        let (r1, s1) = keten_aan(&Ketenstand::leeg(), gebeurtenis(1)).unwrap();
        let (r2, _) = keten_aan(&s1, gebeurtenis(2)).unwrap();
        // Zet regel 2 op plaats 1: het volgnummer zit in de hash, dus dit valt op.
        let vervalst = Ketenregel { volgnummer: 1, ..r2.clone() };
        assert!(!vervalst.hash_klopt().unwrap());
        assert_ne!(r1.hash, r2.hash);
    }

    #[test]
    fn zelfde_gebeurtenis_op_andere_plaats_geeft_andere_hash() {
        let (r1, s1) = keten_aan(&Ketenstand::leeg(), gebeurtenis(1)).unwrap();
        let (r2, _) = keten_aan(&s1, gebeurtenis(1)).unwrap();
        assert_ne!(r1.hash, r2.hash, "identieke inhoud mag geen identieke schakel geven");
    }

    #[test]
    fn lege_stand_is_herkenbaar() {
        assert!(Ketenstand::leeg().is_leeg());
        let (_, s) = keten_aan(&Ketenstand::leeg(), gebeurtenis(1)).unwrap();
        assert!(!s.is_leeg());
    }
}
