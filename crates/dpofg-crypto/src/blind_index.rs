//! Blinde index: zoeken in versleutelde velden zonder ze te ontsleutelen.
//!
//! Het probleem: een e-mailadres in een verzoekdossier moet versleuteld staan,
//! maar de behandelaar moet er wel op kunnen zoeken. Het adres onversleuteld in
//! een kolom zetten om te kunnen zoeken maakt de versleuteling zinloos.
//!
//! De oplossing: naast de envelop wordt een **blinde index** opgeslagen — een
//! HMAC over de genormaliseerde waarde, met een sleutel die alleen in de kluis
//! zit. Zoeken gebeurt door dezelfde HMAC over de zoekterm te berekenen en op
//! gelijkheid te vergelijken.
//!
//! Wat dit wél biedt:
//!
//! * Zoeken op exacte waarde zonder ontsleutelen.
//! * Zonder de indexsleutel is de index niet om te keren.
//!
//! Wat dit **niet** biedt, en waarom dat expliciet moet worden vastgelegd:
//!
//! * **Geen bereikzoekopdrachten en geen deelstrings.** Alleen exacte
//!   gelijkheid na normalisatie.
//! * **Gelijke waarden leveren gelijke indexen.** Wie de tabel kan lezen, ziet
//!   dus *welke records dezelfde waarde delen*, ook zonder die waarde te
//!   kennen. Bij een veld met weinig mogelijke waarden — bijvoorbeeld
//!   "risiconiveau" met drie opties — is dat feitelijk een lek. Daarom geldt de
//!   regel: **een blinde index alleen op velden met hoge variatie**
//!   (e-mailadres, dossiernummer, telefoonnummer), nooit op velden met een
//!   kleine waardenverzameling.
//! * De index wordt **afgekapt**. Dat introduceert bewust botsingen, zodat een
//!   treffer geen zekerheid geeft maar een voorselectie: de kandidaten worden
//!   daarna ontsleuteld en exact vergeleken. Dit dempt frequentieanalyse.

use hmac::{Hmac, Mac};
use sha2::Sha256;

use crate::{aead::Gegevenssleutel, Resultaat};

type HmacSha256 = Hmac<Sha256>;

/// Aantal bytes dat van de HMAC wordt bewaard.
///
/// Acht bytes geeft 2^64 mogelijke waarden: ruim genoeg om de kandidatenlijst
/// kort te houden, kort genoeg om botsingen niet uit te sluiten.
pub const INDEX_LENGTE: usize = 8;

/// Normaliseert een waarde vóór indexering.
///
/// Zonder normalisatie levert `Jan@Example.NL ` een andere index op dan
/// `jan@example.nl`, waardoor het zoeken stilzwijgend niets vindt — precies het
/// soort fout dat de gebruiker niet kan zien en dus niet kan corrigeren.
pub fn normaliseer(waarde: &str) -> String {
    waarde.trim().to_lowercase().split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Berekent de blinde index voor een waarde binnen een veld.
///
/// Het veld gaat mee in de berekening, zodat dezelfde waarde in twee
/// verschillende velden verschillende indexen oplevert. Anders zou zichtbaar
/// worden dat het e-mailadres van een melder gelijk is aan dat van een
/// contactpersoon.
pub fn indexeer(sleutel: &Gegevenssleutel, veld: &str, waarde: &str) -> Resultaat<String> {
    let genormaliseerd = normaliseer(waarde);
    let mut mac = <HmacSha256 as Mac>::new_from_slice(sleutel.bytes())
        .expect("HMAC-SHA256 aanvaardt elke sleutellengte");
    // Lengteprefix zodat ("ab","c") en ("a","bc") niet samenvallen.
    mac.update(&(veld.len() as u32).to_be_bytes());
    mac.update(veld.as_bytes());
    mac.update(genormaliseerd.as_bytes());
    let volledig = mac.finalize().into_bytes();
    Ok(hex::encode(&volledig[..INDEX_LENGTE]))
}

/// Geeft aan of een veld geschikt is voor een blinde index.
///
/// Deze lijst is bewust een allowlist en geen blocklist: een nieuw veld krijgt
/// pas een index nadat iemand heeft vastgesteld dat de waardenverzameling groot
/// genoeg is. Dat is een bewuste rem — zie de waarschuwing bovenaan deze module.
pub fn veld_is_geschikt(veld: &str) -> bool {
    const GESCHIKT: &[&str] = &[
        "betrokkene.emailadres",
        "betrokkene.telefoonnummer",
        "betrokkene.dossiernummer",
        "verzoek.kenmerk",
        "incident.kenmerk",
        "leverancier.kvknummer",
        "leverancier.naam",
        "verwerking.kenmerk",
        "melder.emailadres",
        "medewerker.emailadres",
    ];
    GESCHIKT.contains(&veld)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aead::nieuwe_gegevenssleutel;

    #[test]
    fn zelfde_waarde_geeft_zelfde_index() {
        let s = nieuwe_gegevenssleutel();
        let a = indexeer(&s, "betrokkene.emailadres", "jan@example.nl").unwrap();
        let b = indexeer(&s, "betrokkene.emailadres", "jan@example.nl").unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn normalisatie_vangt_schrijfwijzen_af() {
        let s = nieuwe_gegevenssleutel();
        let a = indexeer(&s, "betrokkene.emailadres", "  Jan@Example.NL ").unwrap();
        let b = indexeer(&s, "betrokkene.emailadres", "jan@example.nl").unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn interne_spaties_worden_genormaliseerd() {
        assert_eq!(normaliseer("Van   der  Berg "), "van der berg");
    }

    #[test]
    fn ander_veld_geeft_andere_index() {
        let s = nieuwe_gegevenssleutel();
        let a = indexeer(&s, "melder.emailadres", "jan@example.nl").unwrap();
        let b = indexeer(&s, "betrokkene.emailadres", "jan@example.nl").unwrap();
        assert_ne!(a, b);
    }

    #[test]
    fn andere_sleutel_geeft_andere_index() {
        let a = indexeer(&nieuwe_gegevenssleutel(), "verzoek.kenmerk", "2026-0041").unwrap();
        let b = indexeer(&nieuwe_gegevenssleutel(), "verzoek.kenmerk", "2026-0041").unwrap();
        assert_ne!(a, b);
    }

    #[test]
    fn index_onthult_de_waarde_niet() {
        let s = nieuwe_gegevenssleutel();
        let index = indexeer(&s, "betrokkene.emailadres", "jan@example.nl").unwrap();
        assert!(!index.contains("jan"));
        assert!(!index.contains("example"));
        assert_eq!(index.len(), INDEX_LENGTE * 2);
    }

    #[test]
    fn veldnaamgrens_is_ondubbelzinnig() {
        let s = nieuwe_gegevenssleutel();
        let a = indexeer(&s, "ab", "c").unwrap();
        let b = indexeer(&s, "a", "bc").unwrap();
        assert_ne!(a, b);
    }

    #[test]
    fn alleen_toegestane_velden_krijgen_een_index() {
        assert!(veld_is_geschikt("betrokkene.emailadres"));
        // Lage variatie: een index zou verklappen welke records hetzelfde
        // risiconiveau delen.
        assert!(!veld_is_geschikt("incident.risiconiveau"));
        assert!(!veld_is_geschikt("verwerking.grondslag"));
    }
}
