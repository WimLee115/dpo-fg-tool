//! Sleutelafleiding uit een wachtwoordzin met Argon2id.
//!
//! Keuze en onderbouwing:
//!
//! * **Argon2id** en niet PBKDF2 of scrypt. Argon2id is de winnaar van de
//!   Password Hashing Competition en combineert weerstand tegen
//!   zijkanaalaanvallen (de `i`-tak) met weerstand tegen tijd-geheugenruil
//!   op GPU's en ASIC's (de `d`-tak). Voor een kluis die op een gestolen
//!   laptop offline aangevallen kan worden, is geheugenhardheid de enige
//!   maatregel die de aanvalskosten echt opdrijft.
//! * **De parameters staan in de kluis**, niet in de programmacode. Anders is
//!   verzwaren bij nieuwe hardware onmogelijk zonder elke bestaande kluis
//!   onleesbaar te maken.
//! * **Het zout is 32 bytes** en per kluis uniek, zodat één vooraf berekende
//!   tabel niet tegen meerdere kluizen werkt.

use argon2::{Algorithm, Argon2, Params, Version};
use serde::{Deserialize, Serialize};

use crate::{CryptoFout, Geheim, Resultaat, Wachtwoordzin};

/// Lengte van het zout in bytes.
pub const ZOUT_LENGTE: usize = 32;

/// Lengte van de afgeleide hoofdsleutel in bytes.
pub const HOOFDSLEUTEL_LENGTE: usize = 32;

/// Het zout waarmee de hoofdsleutel uit de wachtwoordzin wordt afgeleid.
pub type Zout = [u8; ZOUT_LENGTE];

/// Parameters van de sleutelafleiding, zoals opgeslagen bij de kluis.
///
/// Deze waarden worden onversleuteld bewaard — ze zijn geen geheim, en zonder
/// die waarden is de kluis niet meer te openen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct KdfParameters {
    /// Geheugengebruik in kibibytes.
    pub geheugen_kib: u32,
    /// Aantal iteraties (tijdkosten).
    pub iteraties: u32,
    /// Mate van parallellisme (aantal banen).
    pub parallellisme: u32,
}

impl KdfParameters {
    /// Standaardprofiel voor dagelijks gebruik: 256 MiB, 3 iteraties, 4 banen.
    ///
    /// Op courante hardware kost dit grofweg een halve tot anderhalve seconde.
    /// Dat is merkbaar bij het openen van de kluis en dat is de bedoeling: het
    /// is precies de vertraging die een offline raadaanval onbetaalbaar maakt.
    pub const STANDAARD: Self = Self { geheugen_kib: 256 * 1024, iteraties: 3, parallellisme: 4 };

    /// Zwaar profiel voor werkplekken met voldoende geheugen: 1 GiB, 4 iteraties.
    pub const ZWAAR: Self = Self { geheugen_kib: 1024 * 1024, iteraties: 4, parallellisme: 4 };

    /// Licht profiel voor machines met weinig geheugen: 64 MiB, 3 iteraties.
    ///
    /// Alleen te kiezen wanneer het standaardprofiel aantoonbaar niet past.
    /// De keuze wordt in het auditlogboek vastgelegd.
    pub const LICHT: Self = Self { geheugen_kib: 64 * 1024, iteraties: 3, parallellisme: 2 };

    /// Uitsluitend voor geautomatiseerde tests. Biedt geen bescherming.
    #[doc(hidden)]
    pub const TEST_ONVEILIG: Self = Self { geheugen_kib: 8 * 1024, iteraties: 1, parallellisme: 1 };

    /// Controleert of de parameters binnen de door Argon2 toegestane grenzen
    /// vallen en niet onder de ondergrens van dit product duiken.
    pub fn controleer(&self) -> Resultaat<()> {
        if self.parallellisme == 0 {
            return Err(CryptoFout::Sleutelafleiding("parallellisme moet minstens 1 zijn".into()));
        }
        if self.iteraties == 0 {
            return Err(CryptoFout::Sleutelafleiding("iteraties moet minstens 1 zijn".into()));
        }
        // Argon2 vereist ten minste 8 KiB per baan.
        if self.geheugen_kib < 8 * self.parallellisme {
            return Err(CryptoFout::Sleutelafleiding(format!(
                "geheugen {} KiB is te laag voor {} banen",
                self.geheugen_kib, self.parallellisme
            )));
        }
        Ok(())
    }

    /// Geeft aan of dit profiel voldoet aan de ondergrens voor productiegebruik.
    ///
    /// Wordt gebruikt om bij het openen van een kluis te waarschuwen dat de
    /// parameters onder de huidige norm liggen en verzwaring aan te bieden.
    pub fn voldoet_aan_ondergrens(&self) -> bool {
        self.geheugen_kib >= Self::LICHT.geheugen_kib && self.iteraties >= 2
    }

    fn naar_argon2(&self) -> Resultaat<Params> {
        self.controleer()?;
        Params::new(
            self.geheugen_kib,
            self.iteraties,
            self.parallellisme,
            Some(HOOFDSLEUTEL_LENGTE),
        )
        .map_err(|e| CryptoFout::Sleutelafleiding(e.to_string()))
    }
}

impl Default for KdfParameters {
    fn default() -> Self {
        Self::STANDAARD
    }
}

/// De sleutel die rechtstreeks uit de wachtwoordzin volgt.
///
/// Deze sleutel versleutelt zelf geen gegevens. Hij ontsluit uitsluitend de
/// kluissleutel (zie [`crate::keys`]). Daardoor kan het wachtwoord worden
/// gewijzigd zonder dat één byte aan gegevens hoeft te worden herversleuteld.
pub type Hoofdsleutel = Geheim<HOOFDSLEUTEL_LENGTE>;

/// Leidt de hoofdsleutel af uit een wachtwoordzin en een zout.
///
/// Deze functie is bewust traag. Roep haar niet aan in een lus.
pub fn leid_hoofdsleutel_af(
    wachtwoord: &Wachtwoordzin,
    zout: &Zout,
    params: KdfParameters,
) -> Resultaat<Hoofdsleutel> {
    if wachtwoord.is_leeg() {
        return Err(CryptoFout::Sleutelafleiding("lege wachtwoordzin".into()));
    }
    let argon = Argon2::new(Algorithm::Argon2id, Version::V0x13, params.naar_argon2()?);
    let mut sleutel = Hoofdsleutel::nul();
    argon
        .hash_password_into(wachtwoord.bytes(), zout, sleutel.bytes_mut())
        .map_err(|e| CryptoFout::Sleutelafleiding(e.to_string()))?;
    Ok(sleutel)
}

/// Beoordeelt de sterkte van een wachtwoordzin.
///
/// Bewust géén regels over hoofdletters, cijfers en leestekens: die leiden
/// aantoonbaar tot voorspelbare wachtwoorden zoals `Welkom2026!`. Wat telt is
/// lengte en variatie. Deze uitkomst blokkeert niet, maar wordt getoond en bij
/// een zwakke keuze in het auditlogboek vastgelegd.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Wachtwoordsterkte {
    Onbruikbaar,
    Zwak,
    Redelijk,
    Sterk,
}

impl Wachtwoordsterkte {
    pub fn toelichting(&self) -> &'static str {
        match self {
            Self::Onbruikbaar => "te kort; gebruik minstens twaalf tekens",
            Self::Zwak => "kort of eenvormig; kies een zin van vier of meer woorden",
            Self::Redelijk => "bruikbaar; een langere zin is beter",
            Self::Sterk => "voldoende lang en gevarieerd",
        }
    }
}

/// Beoordeelt een wachtwoordzin op lengte en tekenvariatie.
pub fn beoordeel_wachtwoord(wachtwoord: &Wachtwoordzin) -> Wachtwoordsterkte {
    let lengte = wachtwoord.lengte();
    if lengte < 12 {
        return Wachtwoordsterkte::Onbruikbaar;
    }
    // Aantal verschillende bytes als ruwe maat voor variatie.
    let mut gezien = [false; 256];
    for b in wachtwoord.bytes() {
        gezien[*b as usize] = true;
    }
    let variatie = gezien.iter().filter(|g| **g).count();
    // Spaties als aanwijzing dat het om een zin gaat.
    let woorden = wachtwoord.bytes().split(|b| *b == b' ').filter(|w| !w.is_empty()).count();

    match (lengte, variatie, woorden) {
        (l, v, w) if l >= 20 && (v >= 12 || w >= 4) => Wachtwoordsterkte::Sterk,
        (l, v, w) if l >= 16 && (v >= 8 || w >= 3) => Wachtwoordsterkte::Redelijk,
        (l, _, _) if l >= 12 => Wachtwoordsterkte::Zwak,
        _ => Wachtwoordsterkte::Onbruikbaar,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ZOUT: Zout = [42u8; ZOUT_LENGTE];

    #[test]
    fn zelfde_invoer_geeft_zelfde_sleutel() {
        let w = Wachtwoordzin::nieuw("een voldoende lange wachtwoordzin");
        let a = leid_hoofdsleutel_af(&w, &ZOUT, KdfParameters::TEST_ONVEILIG).unwrap();
        let b = leid_hoofdsleutel_af(&w, &ZOUT, KdfParameters::TEST_ONVEILIG).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn ander_zout_geeft_andere_sleutel() {
        let w = Wachtwoordzin::nieuw("een voldoende lange wachtwoordzin");
        let a = leid_hoofdsleutel_af(&w, &ZOUT, KdfParameters::TEST_ONVEILIG).unwrap();
        let b =
            leid_hoofdsleutel_af(&w, &[7u8; ZOUT_LENGTE], KdfParameters::TEST_ONVEILIG).unwrap();
        assert_ne!(a, b);
    }

    #[test]
    fn ander_wachtwoord_geeft_andere_sleutel() {
        let a = leid_hoofdsleutel_af(
            &Wachtwoordzin::nieuw("wachtwoordzin nummer een"),
            &ZOUT,
            KdfParameters::TEST_ONVEILIG,
        )
        .unwrap();
        let b = leid_hoofdsleutel_af(
            &Wachtwoordzin::nieuw("wachtwoordzin nummer twee"),
            &ZOUT,
            KdfParameters::TEST_ONVEILIG,
        )
        .unwrap();
        assert_ne!(a, b);
    }

    #[test]
    fn andere_parameters_geven_andere_sleutel() {
        let w = Wachtwoordzin::nieuw("een voldoende lange wachtwoordzin");
        let a = leid_hoofdsleutel_af(&w, &ZOUT, KdfParameters::TEST_ONVEILIG).unwrap();
        let zwaarder = KdfParameters { iteraties: 2, ..KdfParameters::TEST_ONVEILIG };
        let b = leid_hoofdsleutel_af(&w, &ZOUT, zwaarder).unwrap();
        assert_ne!(a, b, "iteraties horen deel uit te maken van de afleiding");
    }

    #[test]
    fn leeg_wachtwoord_wordt_geweigerd() {
        let w = Wachtwoordzin::nieuw("");
        assert!(leid_hoofdsleutel_af(&w, &ZOUT, KdfParameters::TEST_ONVEILIG).is_err());
    }

    #[test]
    fn parameters_worden_gecontroleerd() {
        assert!(KdfParameters::STANDAARD.controleer().is_ok());
        assert!(KdfParameters::ZWAAR.controleer().is_ok());
        assert!(KdfParameters::LICHT.controleer().is_ok());
        assert!(KdfParameters { geheugen_kib: 1024, iteraties: 0, parallellisme: 1 }
            .controleer()
            .is_err());
        assert!(KdfParameters { geheugen_kib: 1024, iteraties: 1, parallellisme: 0 }
            .controleer()
            .is_err());
        assert!(KdfParameters { geheugen_kib: 8, iteraties: 1, parallellisme: 4 }
            .controleer()
            .is_err());
    }

    #[test]
    fn ondergrens_wordt_bewaakt() {
        assert!(KdfParameters::STANDAARD.voldoet_aan_ondergrens());
        assert!(KdfParameters::LICHT.voldoet_aan_ondergrens());
        assert!(!KdfParameters::TEST_ONVEILIG.voldoet_aan_ondergrens());
    }

    #[test]
    fn wachtwoordbeoordeling() {
        use Wachtwoordsterkte::*;
        assert_eq!(beoordeel_wachtwoord(&Wachtwoordzin::nieuw("kort")), Onbruikbaar);
        assert_eq!(beoordeel_wachtwoord(&Wachtwoordzin::nieuw("aaaaaaaaaaaa")), Zwak);
        assert_eq!(beoordeel_wachtwoord(&Wachtwoordzin::nieuw("paard batterij niet")), Redelijk);
        assert_eq!(
            beoordeel_wachtwoord(&Wachtwoordzin::nieuw("paard batterij niet vastzetten")),
            Sterk
        );
    }

    #[test]
    fn parameters_zijn_serialiseerbaar() {
        let json = serde_json::to_string(&KdfParameters::STANDAARD).unwrap();
        let terug: KdfParameters = serde_json::from_str(&json).unwrap();
        assert_eq!(terug, KdfParameters::STANDAARD);
    }
}
