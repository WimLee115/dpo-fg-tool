//! De ondertekenidentiteit van één installatie.
//!
//! # Waarom dit bestaat
//!
//! Een handtekening met een sleutelpaar dat ter plekke wordt aangemaakt en
//! daarna wordt weggegooid, toont precies twee dingen aan: dat er is
//! ondertekend, en dat de inhoud sindsdien niet is gewijzigd. Wat zij *niet*
//! toont, is van wie het stuk komt — en dat is nu juist de vraag die een
//! toezichthouder stelt.
//!
//! Deze module levert daarom een sleutelpaar dat bij de kluis hoort en blijft
//! bestaan: elk anker en elk dossiermanifest van diezelfde kluis draagt
//! dezelfde publieke sleutel. De ontvanger vergelijkt die met de sleutel die de
//! organisatie langs een ánder kanaal heeft gepubliceerd. Zonder die
//! vergelijking is de herkomst nog steeds niet vastgesteld; de vergelijking is
//! een handeling van de ontvanger en volgt niet uit het bestand zelf.
//!
//! # Wat "installatie" hier betekent
//!
//! Feitelijk: *dit kluisbestand*. Twee kluizen op één machine krijgen twee
//! identiteiten, en een kopie van een kluisbestand draagt dezelfde privésleutel
//! als het origineel. Dat is eerlijk benoemd en niet weggepoetst: wie het
//! kluisbestand én de wachtwoordzin heeft, kan ondertekenen namens de
//! organisatie.
//!
//! # Wat er niet in zit
//!
//! Geen rotatie, geen intrekking, geen sleutelboek en geen hardwaretoken. De
//! sleutel is zo sterk als het kluisbestand plus de wachtwoordzin.

use std::fmt;

pub use ed25519_dalek::SigningKey;

/// Lengte van het ondertekenzaad in bytes.
pub const ZAAD_LENGTE: usize = 32;

/// De vaste ondertekenidentiteit van deze installatie.
///
/// De privésleutel verlaat dit type niet in geserialiseerde vorm: er is geen
/// `Serialize`, geen `Clone` en geen accessor die het zaad teruggeeft. Wat
/// eruit komt is de publieke helft, en een leen op de ondertekensleutel voor
/// wie daadwerkelijk moet tekenen.
pub struct Installatiesleutel {
    sleutel: SigningKey,
    publiek: String,
}

impl Installatiesleutel {
    /// Bouwt de identiteit uit het ruwe zaad.
    pub fn uit_zaad(zaad: &[u8; ZAAD_LENGTE]) -> Self {
        let sleutel = SigningKey::from_bytes(zaad);
        let publiek = hex::encode(sleutel.verifying_key().to_bytes());
        Self { sleutel, publiek }
    }

    /// De publieke sleutel: 32 bytes, hexadecimaal in kleine letters.
    ///
    /// Exact de vorm die in `anker.sleutel` en `manifest.ondertekenaar` staat,
    /// zodat de gepubliceerde waarde letterlijk te vergelijken is met wat er in
    /// een uitgeleverd bestand staat.
    pub fn publieke_sleutel(&self) -> &str {
        &self.publiek
    }

    /// De sleutel waarmee wordt ondertekend.
    pub fn ondertekensleutel(&self) -> &SigningKey {
        &self.sleutel
    }
}

impl fmt::Debug for Installatiesleutel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Installatiesleutel")
            .field("publiek", &self.publiek)
            .field("prive", &"<verborgen>")
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Compileerwacht. Deze functie doet niets; wat telt is dat zij niet
    /// compileert zodra `SigningKey` de eigenschap verliest die het
    /// sleutelmateriaal bij het opruimen overschrijft. Dat gebeurt zodra iemand
    /// `default-features = false` op `ed25519-dalek` zet, en dan is het beter
    /// dat de bouw stukloopt dan dat het zaad stilletjes in het geheugen
    /// blijft staan.
    fn eist_zeroize<T: zeroize::ZeroizeOnDrop>() {}

    #[test]
    fn de_ondertekensleutel_wordt_genuld() {
        eist_zeroize::<SigningKey>();
    }

    #[test]
    fn dezelfde_zaad_levert_dezelfde_publieke_sleutel() {
        let zaad = [7u8; ZAAD_LENGTE];
        assert_eq!(
            Installatiesleutel::uit_zaad(&zaad).publieke_sleutel(),
            Installatiesleutel::uit_zaad(&zaad).publieke_sleutel()
        );
    }

    #[test]
    fn de_publieke_sleutel_is_vierenzestig_kleine_hextekens() {
        let s = Installatiesleutel::uit_zaad(&[3u8; ZAAD_LENGTE]);
        let pk = s.publieke_sleutel();
        assert_eq!(pk.len(), 64);
        assert!(pk.chars().all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()));
    }

    #[test]
    fn de_debugweergave_toont_geen_sleutelmateriaal() {
        let zaad = [9u8; ZAAD_LENGTE];
        let weergave = format!("{:?}", Installatiesleutel::uit_zaad(&zaad));
        assert!(weergave.contains("<verborgen>"));
        assert!(!weergave.contains(&hex::encode(zaad)));
        // Ook niet als losse bytes.
        assert!(!weergave.contains("9, 9, 9"));
    }

    #[test]
    fn de_handtekening_is_met_de_publieke_sleutel_te_controleren() {
        use ed25519_dalek::{Signer, Verifier};

        let s = Installatiesleutel::uit_zaad(&[1u8; ZAAD_LENGTE]);
        let handtekening = s.ondertekensleutel().sign(b"proefbericht");

        let bytes = hex::decode(s.publieke_sleutel()).unwrap();
        let array: [u8; 32] = bytes.try_into().unwrap();
        let vk = ed25519_dalek::VerifyingKey::from_bytes(&array).unwrap();
        assert!(vk.verify(b"proefbericht", &handtekening).is_ok());
    }
}
