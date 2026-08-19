//! De spiegel: aantonen dát een record bestond, zonder de inhoud.
//!
//! Het persoonlijke dossier van de functionaris staat in een eigen kluis
//! waarvan de organisatie de inhoud niet kan lezen. Dat lost één probleem op
//! en schept een ander: een dossier dat alleen de functionaris beheert, is
//! achteraf ook alleen door hem te dateren, en een tijdstip dat uitsluitend op
//! zijn eigen opgave berust, is in een geschil weinig waard.
//!
//! De spiegel lost dat op. Bij het vastleggen van een advies of een
//! onafhankelijkheidsincident gaat er een hash van dat record naar de kluis
//! van de organisatie. Die hash zegt niets over de inhoud, maar hij hangt daar
//! wel in een ketenlogboek dat niet ongemerkt te wijzigen is en dat extern
//! wordt verankerd. Later kan de functionaris zijn record naast die hash
//! leggen en aantonen dat het op dat moment al bestond en sindsdien niet is
//! veranderd.
//!
//! # Wat de spiegel niet doet
//!
//! Hij verhindert niet dat de organisatie de hash verwijdert. Wel is dat
//! zichtbaar: het logboek is een keten, en een regel eruit halen breekt hem.
//! Hij verhindert evenmin dat de functionaris een record achteraf aanpast —
//! dan komt de hash alleen niet meer overeen, en dan is er geen bewijs. Dat is
//! het eerlijke resultaat: de spiegel toont overeenstemming of het ontbreken
//! daarvan, en niets ertussenin.

use blake3::Hasher;
use serde::{Deserialize, Serialize};

/// De soort waaronder spiegelregels in de kluis van de organisatie staan.
pub const SPIEGELSOORT: &str = "spiegel";

/// Het domeinscheidingsvoorvoegsel onder de hash.
///
/// Zonder dit voorvoegsel zou een hash uit een andere context als spiegel
/// kunnen worden aangeboden.
const CONTEXT: &[u8] = b"dpofg-spiegel-v1";

/// Wat er in de kluis van de organisatie achterblijft.
///
/// Er staat met opzet geen kenmerk in en geen omschrijving: allebei zouden
/// verraden waarover het advies ging, en dat is precies wat hier niet hoort te
/// lekken. Wat er wel in staat is de soort, want dat een functionaris
/// überhaupt onafhankelijkheidsincidenten vastlegt, is op zichzelf al een
/// gegeven dat het bestuur aangaat.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Spiegelregel {
    /// De hash van het record in het persoonlijke dossier.
    pub hash: String,
    /// Waarvan dit de spiegel is: `advies` of `onafhankelijkheidsincident`.
    pub soort: String,
    /// Wanneer de spiegel is vastgelegd.
    ///
    /// Dit is het tijdstip dat telt: het staat in het ketenlogboek van de
    /// organisatie en is daarmee niet eenzijdig te verschuiven.
    pub vastgelegd_op: chrono::DateTime<chrono::Utc>,
}

/// Berekent de hash van een record uit het persoonlijke dossier.
///
/// De hash gaat over de geserialiseerde vorm van het record, met de soort en
/// een vaste context ervoor. De soort gaat mee zodat een advies en een
/// onafhankelijkheidsincident met toevallig dezelfde velden niet dezelfde hash
/// opleveren.
pub fn spiegelhash<T: Serialize>(soort: &str, record: &T) -> Result<String, serde_json::Error> {
    let json = serde_json::to_vec(record)?;
    let mut h = Hasher::new();
    h.update(CONTEXT);
    h.update(&(soort.len() as u64).to_be_bytes());
    h.update(soort.as_bytes());
    h.update(&(json.len() as u64).to_be_bytes());
    h.update(&json);
    Ok(h.finalize().to_hex().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Serialize;

    #[derive(Serialize)]
    struct Proef {
        a: u32,
        b: String,
    }

    #[test]
    fn dezelfde_inhoud_levert_dezelfde_hash() {
        let x = Proef { a: 1, b: "een advies".into() };
        let y = Proef { a: 1, b: "een advies".into() };
        assert_eq!(spiegelhash("advies", &x).unwrap(), spiegelhash("advies", &y).unwrap());
    }

    #[test]
    fn een_andere_inhoud_levert_een_andere_hash() {
        let x = Proef { a: 1, b: "een advies".into() };
        let y = Proef { a: 1, b: "een ander advies".into() };
        assert_ne!(spiegelhash("advies", &x).unwrap(), spiegelhash("advies", &y).unwrap());
    }

    /// De soort gaat mee in de hash: anders zouden twee records met dezelfde
    /// velden voor elkaar kunnen doorgaan.
    #[test]
    fn de_soort_telt_mee() {
        let x = Proef { a: 1, b: "hetzelfde".into() };
        assert_ne!(
            spiegelhash("advies", &x).unwrap(),
            spiegelhash("onafhankelijkheidsincident", &x).unwrap()
        );
    }

    /// De lengtes gaan mee, zodat twee verschillende opsplitsingen niet
    /// dezelfde bytes voeden.
    #[test]
    fn de_lengtes_scheiden_de_velden() {
        let x = Proef { a: 1, b: String::new() };
        assert_ne!(spiegelhash("ad", &x).unwrap(), spiegelhash("a", &x).unwrap());
    }

    #[test]
    fn de_hash_is_vierenzestig_hexadecimale_tekens() {
        let h = spiegelhash("advies", &Proef { a: 1, b: "x".into() }).unwrap();
        assert_eq!(h.len(), 64);
        assert!(h.chars().all(|c| c.is_ascii_hexdigit()));
    }
}
