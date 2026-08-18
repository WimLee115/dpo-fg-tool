//! Versleuteling van gegevens met XChaCha20-Poly1305.
//!
//! Keuze en onderbouwing:
//!
//! * **XChaCha20-Poly1305** en niet AES-GCM. De 192-bits nonce maakt het veilig
//!   om nonces willekeurig te trekken: de kans op hergebruik is
//!   verwaarloosbaar, ook na miljarden records. Bij AES-GCM met zijn 96-bits
//!   nonce moet een teller worden bijgehouden, en een teller die na een
//!   herstelde back-up terugspringt is catastrofaal — precies het scenario dat
//!   in een lokaal draaiend product gaat gebeuren.
//! * **ChaCha20 is niet afhankelijk van hardwareversnelling.** Op machines
//!   zonder AES-NI is AES in software zowel traag als gevoelig voor
//!   cachetiming; ChaCha20 is overal constante tijd.
//! * **Bijbehorende gegevens (AAD) zijn verplicht.** Elke envelop is gebonden
//!   aan zijn context: veldnaam, tabel en record-identificatie. Daardoor kan
//!   een versleuteld veld niet naar een ander record worden verplaatst zonder
//!   dat de authenticatie faalt. Zonder die binding zou een aanvaller met
//!   schrijfrechten op het bestand de bewaartermijn van record A naar record B
//!   kunnen kopiëren, zonder ooit iets te ontsleutelen.

use chacha20poly1305::{
    aead::{Aead, KeyInit, Payload},
    XChaCha20Poly1305, XNonce,
};
use rand::RngCore;
use serde::{Deserialize, Serialize};

use crate::{CryptoFout, Geheim, Resultaat};

/// Lengte van een gegevenssleutel in bytes.
pub const SLEUTEL_LENGTE: usize = 32;
/// Lengte van de nonce in bytes.
pub const NONCE_LENGTE: usize = 24;
/// Lengte van de authenticatietag in bytes.
pub const TAG_LENGTE: usize = 16;

/// Huidige versie van het envelopformaat.
pub const FORMAATVERSIE: u8 = 1;

/// Een symmetrische sleutel waarmee gegevens worden versleuteld.
pub type Gegevenssleutel = Geheim<SLEUTEL_LENGTE>;

/// De context waaraan een envelop is gebonden.
///
/// Deze waarden worden niet meeversleuteld maar wél meegeauthenticeerd: wie ze
/// wijzigt, maakt de envelop onleesbaar.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Binding {
    /// Logische plaats van de gegevens, bijvoorbeeld `verwerking.bewaartermijn`.
    pub veld: String,
    /// Identificatie van het record waartoe de gegevens behoren.
    pub record: String,
    /// Compartiment waarin het record valt.
    pub compartiment: String,
}

impl Binding {
    pub fn nieuw(
        veld: impl Into<String>,
        record: impl Into<String>,
        compartiment: impl Into<String>,
    ) -> Self {
        Self { veld: veld.into(), record: record.into(), compartiment: compartiment.into() }
    }

    /// Serialiseert de binding ondubbelzinnig.
    ///
    /// De lengte van elk onderdeel gaat mee, zodat de bindingen
    /// (`ab`, `c`) en (`a`, `bc`) niet dezelfde bytes opleveren.
    fn naar_bytes(&self) -> Vec<u8> {
        let mut uit =
            Vec::with_capacity(32 + self.veld.len() + self.record.len() + self.compartiment.len());
        uit.push(FORMAATVERSIE);
        for deel in [&self.veld, &self.record, &self.compartiment] {
            uit.extend_from_slice(&(deel.len() as u32).to_be_bytes());
            uit.extend_from_slice(deel.as_bytes());
        }
        uit
    }
}

/// Een versleuteld pakket: nonce, cijfertekst en authenticatietag.
///
/// Wordt als geheel opgeslagen. De inhoud is zonder de juiste sleutel niet te
/// onderscheiden van willekeur.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Envelop {
    /// Versie van het formaat, zodat later een ander schema kan worden ingevoerd.
    pub versie: u8,
    /// De willekeurige nonce.
    #[serde(with = "hex_bytes")]
    pub nonce: Vec<u8>,
    /// Cijfertekst met daarachter de authenticatietag.
    #[serde(with = "hex_bytes")]
    pub inhoud: Vec<u8>,
}

impl Envelop {
    /// Totale omvang in bytes zoals opgeslagen.
    pub fn omvang(&self) -> usize {
        1 + self.nonce.len() + self.inhoud.len()
    }

    /// Compacte binaire weergave: versie ‖ nonce ‖ inhoud.
    pub fn naar_bytes(&self) -> Vec<u8> {
        let mut uit = Vec::with_capacity(self.omvang());
        uit.push(self.versie);
        uit.extend_from_slice(&self.nonce);
        uit.extend_from_slice(&self.inhoud);
        uit
    }

    /// Leest de compacte binaire weergave terug.
    pub fn uit_bytes(bytes: &[u8]) -> Resultaat<Self> {
        if bytes.len() < 1 + NONCE_LENGTE + TAG_LENGTE {
            return Err(CryptoFout::OngeldigFormaat(format!(
                "envelop is {} bytes, minimaal {} vereist",
                bytes.len(),
                1 + NONCE_LENGTE + TAG_LENGTE
            )));
        }
        let versie = bytes[0];
        if versie != FORMAATVERSIE {
            return Err(CryptoFout::OnbekendeVersie(versie));
        }
        Ok(Self {
            versie,
            nonce: bytes[1..1 + NONCE_LENGTE].to_vec(),
            inhoud: bytes[1 + NONCE_LENGTE..].to_vec(),
        })
    }
}

/// Versleutelt `klaartekst` met `sleutel`, gebonden aan `binding`.
pub fn versleutel(
    sleutel: &Gegevenssleutel,
    binding: &Binding,
    klaartekst: &[u8],
) -> Resultaat<Envelop> {
    let cipher = XChaCha20Poly1305::new(sleutel.bytes().into());

    let mut nonce_bytes = [0u8; NONCE_LENGTE];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = XNonce::from_slice(&nonce_bytes);

    let aad = binding.naar_bytes();
    let inhoud = cipher
        .encrypt(nonce, Payload { msg: klaartekst, aad: &aad })
        .map_err(|_| CryptoFout::Versleuteling)?;

    Ok(Envelop { versie: FORMAATVERSIE, nonce: nonce_bytes.to_vec(), inhoud })
}

/// Ontsleutelt een envelop. Faalt zodra sleutel, binding of inhoud niet kloppen.
pub fn ontsleutel(
    sleutel: &Gegevenssleutel,
    binding: &Binding,
    envelop: &Envelop,
) -> Resultaat<Vec<u8>> {
    if envelop.versie != FORMAATVERSIE {
        return Err(CryptoFout::OnbekendeVersie(envelop.versie));
    }
    if envelop.nonce.len() != NONCE_LENGTE {
        return Err(CryptoFout::OngeldigeLengte {
            veld: "nonce",
            verwacht: NONCE_LENGTE,
            gekregen: envelop.nonce.len(),
        });
    }
    let cipher = XChaCha20Poly1305::new(sleutel.bytes().into());
    let nonce = XNonce::from_slice(&envelop.nonce);
    let aad = binding.naar_bytes();
    cipher
        .decrypt(nonce, Payload { msg: &envelop.inhoud, aad: &aad })
        .map_err(|_| CryptoFout::Ontsleuteling)
}

/// Trekt een nieuwe, willekeurige gegevenssleutel.
pub fn nieuwe_gegevenssleutel() -> Gegevenssleutel {
    let mut sleutel = Gegevenssleutel::nul();
    rand::thread_rng().fill_bytes(sleutel.bytes_mut());
    sleutel
}

/// Hexadecimale (de)serialisatie voor de bytevelden van een envelop.
mod hex_bytes {
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S: Serializer>(v: &[u8], s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&hex::encode(v))
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<u8>, D::Error> {
        let s = String::deserialize(d)?;
        hex::decode(&s).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn binding() -> Binding {
        Binding::nieuw("verwerking.omschrijving", "0412-K", "algemeen")
    }

    #[test]
    fn heen_en_terug() {
        let sleutel = nieuwe_gegevenssleutel();
        let tekst = "Verzuimregistratie met gezondheidsgegevens".as_bytes();
        let env = versleutel(&sleutel, &binding(), tekst).unwrap();
        let terug = ontsleutel(&sleutel, &binding(), &env).unwrap();
        assert_eq!(terug, tekst);
    }

    #[test]
    fn cijfertekst_bevat_de_klaartekst_niet() {
        let sleutel = nieuwe_gegevenssleutel();
        let tekst = b"BSN 123456782";
        let env = versleutel(&sleutel, &binding(), tekst).unwrap();
        assert!(!env.inhoud.windows(tekst.len()).any(|w| w == tekst));
    }

    #[test]
    fn verkeerde_sleutel_faalt() {
        let env = versleutel(&nieuwe_gegevenssleutel(), &binding(), b"geheim").unwrap();
        let fout = ontsleutel(&nieuwe_gegevenssleutel(), &binding(), &env).unwrap_err();
        assert_eq!(fout, CryptoFout::Ontsleuteling);
    }

    #[test]
    fn envelop_kan_niet_naar_ander_record() {
        let sleutel = nieuwe_gegevenssleutel();
        let env = versleutel(&sleutel, &binding(), b"bewaartermijn 7 jaar").unwrap();
        let ander = Binding::nieuw("verwerking.omschrijving", "0413-K", "algemeen");
        assert_eq!(ontsleutel(&sleutel, &ander, &env).unwrap_err(), CryptoFout::Ontsleuteling);
    }

    #[test]
    fn envelop_kan_niet_naar_ander_veld() {
        let sleutel = nieuwe_gegevenssleutel();
        let env = versleutel(&sleutel, &binding(), b"waarde").unwrap();
        let ander = Binding::nieuw("verwerking.grondslag", "0412-K", "algemeen");
        assert_eq!(ontsleutel(&sleutel, &ander, &env).unwrap_err(), CryptoFout::Ontsleuteling);
    }

    #[test]
    fn envelop_kan_niet_naar_ander_compartiment() {
        let sleutel = nieuwe_gegevenssleutel();
        let env = versleutel(&sleutel, &binding(), b"waarde").unwrap();
        let ander = Binding::nieuw("verwerking.omschrijving", "0412-K", "vertrouwelijk");
        assert_eq!(ontsleutel(&sleutel, &ander, &env).unwrap_err(), CryptoFout::Ontsleuteling);
    }

    #[test]
    fn gewijzigde_inhoud_faalt() {
        let sleutel = nieuwe_gegevenssleutel();
        let mut env = versleutel(&sleutel, &binding(), b"niet melden").unwrap();
        env.inhoud[0] ^= 0x01;
        assert_eq!(ontsleutel(&sleutel, &binding(), &env).unwrap_err(), CryptoFout::Ontsleuteling);
    }

    #[test]
    fn gewijzigde_nonce_faalt() {
        let sleutel = nieuwe_gegevenssleutel();
        let mut env = versleutel(&sleutel, &binding(), b"inhoud").unwrap();
        env.nonce[0] ^= 0x01;
        assert_eq!(ontsleutel(&sleutel, &binding(), &env).unwrap_err(), CryptoFout::Ontsleuteling);
    }

    #[test]
    fn nonces_worden_niet_hergebruikt() {
        let sleutel = nieuwe_gegevenssleutel();
        let mut gezien = std::collections::HashSet::new();
        for _ in 0..1000 {
            let env = versleutel(&sleutel, &binding(), b"zelfde inhoud").unwrap();
            assert!(gezien.insert(env.nonce.clone()), "nonce hergebruikt");
        }
    }

    #[test]
    fn zelfde_klaartekst_geeft_andere_cijfertekst() {
        let sleutel = nieuwe_gegevenssleutel();
        let a = versleutel(&sleutel, &binding(), b"identiek").unwrap();
        let b = versleutel(&sleutel, &binding(), b"identiek").unwrap();
        assert_ne!(a.inhoud, b.inhoud);
    }

    #[test]
    fn lege_klaartekst_werkt() {
        let sleutel = nieuwe_gegevenssleutel();
        let env = versleutel(&sleutel, &binding(), b"").unwrap();
        assert_eq!(ontsleutel(&sleutel, &binding(), &env).unwrap(), Vec::<u8>::new());
    }

    #[test]
    fn binaire_weergave_heen_en_terug() {
        let sleutel = nieuwe_gegevenssleutel();
        let env = versleutel(&sleutel, &binding(), b"inhoud van het veld").unwrap();
        let bytes = env.naar_bytes();
        let terug = Envelop::uit_bytes(&bytes).unwrap();
        assert_eq!(env, terug);
        assert_eq!(ontsleutel(&sleutel, &binding(), &terug).unwrap(), b"inhoud van het veld");
    }

    #[test]
    fn te_korte_envelop_wordt_geweigerd() {
        assert!(matches!(
            Envelop::uit_bytes(&[1u8; 10]).unwrap_err(),
            CryptoFout::OngeldigFormaat(_)
        ));
    }

    #[test]
    fn onbekende_versie_wordt_geweigerd() {
        let mut bytes = vec![99u8];
        bytes.extend_from_slice(&[0u8; NONCE_LENGTE + TAG_LENGTE]);
        assert_eq!(Envelop::uit_bytes(&bytes).unwrap_err(), CryptoFout::OnbekendeVersie(99));
    }

    #[test]
    fn binding_is_ondubbelzinnig() {
        let a = Binding::nieuw("ab", "c", "x").naar_bytes();
        let b = Binding::nieuw("a", "bc", "x").naar_bytes();
        assert_ne!(a, b, "lengteprefixen moeten samenvoegen voorkomen");
    }
}
