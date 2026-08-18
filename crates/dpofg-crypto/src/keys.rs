//! De sleutelhiërarchie van de kluis.
//!
//! ```text
//!   wachtwoordzin ─Argon2id(zout,params)─► hoofdsleutel
//!                                              │ ontsluit (wikkeling)
//!                                              ▼
//!                                        kluissleutel
//!                                              │ ontsluit (wikkeling)
//!                    ┌─────────────────────────┼─────────────────────────┐
//!                    ▼                         ▼                         ▼
//!          compartimentsleutel      compartimentsleutel      compartimentsleutel
//!               "algemeen"             "vertrouwelijk"         "fg-persoonlijk"
//!                    │                         │                         │
//!                    ▼                         ▼                         ▼
//!               envelop per veld          envelop per veld          envelop per veld
//! ```
//!
//! Waarom drie lagen en niet één:
//!
//! * **Wachtwoord wijzigen zonder herversleutelen.** Het wachtwoord ontsluit
//!   alleen de kluissleutel. Bij een wachtwoordwijziging wordt uitsluitend die
//!   ene wikkeling vervangen; geen enkele byte gegevens hoeft eraan te komen.
//!   Zonder deze laag zou een wachtwoordwijziging de hele kluis moeten
//!   herschrijven — een operatie die halverwege kan afbreken en die mensen
//!   daarom gaan vermijden.
//! * **Compartimenten zijn cryptografisch gescheiden, niet met een
//!   programmaregel.** Wie de compartimentsleutel niet heeft, ziet
//!   onleesbare bytes — ook als hij het bestand rechtstreeks opent en ook als
//!   er een fout in de toegangscontrole zit. Dit is ontwerpprincipe P5 uit het
//!   plan.
//! * **Rotatie per compartiment.** Een compartimentsleutel kan afzonderlijk
//!   worden vervangen zonder de andere compartimenten te raken.
//!
//! Elke wikkeling is gebonden aan zijn plaats in de hiërarchie: een gewikkelde
//! compartimentsleutel kan niet als kluissleutel worden aangeboden, en een
//! wikkeling van compartiment A niet als die van compartiment B.

use rand::RngCore;
use serde::{Deserialize, Serialize};

use crate::{
    aead::{self, Binding, Gegevenssleutel, Envelop, SLEUTEL_LENGTE},
    kdf::{self, Hoofdsleutel, KdfParameters, Zout, ZOUT_LENGTE},
    CryptoFout, Geheim, Resultaat, Wachtwoordzin,
};

/// De sleutel die alle compartimentsleutels ontsluit.
pub type Kluissleutel = Geheim<SLEUTEL_LENGTE>;
/// De sleutel waarmee de gegevens van één compartiment worden versleuteld.
pub type Compartimentsleutel = Gegevenssleutel;

/// Naam van het compartiment dat standaard bestaat.
pub const COMPARTIMENT_ALGEMEEN: &str = "algemeen";

/// Onversleuteld deel van de kluis: alles wat nodig is om hem te openen.
///
/// Dit blok wordt in leesbare vorm bewaard. Het bevat geen sleutelmateriaal —
/// alleen het zout, de afleidingsparameters en de gewikkelde kluissleutel.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Kluishoofd {
    /// Versie van de kluisindeling.
    pub versie: u8,
    /// Zout voor de sleutelafleiding, hexadecimaal.
    pub zout: String,
    /// Parameters waarmee de hoofdsleutel is afgeleid.
    pub kdf: KdfParameters,
    /// De met de hoofdsleutel gewikkelde kluissleutel.
    pub kluissleutel: Envelop,
    /// Generatie van de kluissleutel; loopt op bij elke rotatie.
    pub generatie: u32,
}

/// Huidige versie van de kluisindeling.
pub const KLUISVERSIE: u8 = 1;

impl Kluishoofd {
    fn zout_bytes(&self) -> Resultaat<Zout> {
        let ruw = hex::decode(&self.zout)
            .map_err(|e| CryptoFout::OngeldigFormaat(format!("zout is geen hex: {e}")))?;
        if ruw.len() != ZOUT_LENGTE {
            return Err(CryptoFout::OngeldigeLengte {
                veld: "zout",
                verwacht: ZOUT_LENGTE,
                gekregen: ruw.len(),
            });
        }
        let mut zout = [0u8; ZOUT_LENGTE];
        zout.copy_from_slice(&ruw);
        Ok(zout)
    }

    fn binding(&self) -> Binding {
        Binding::nieuw("kluis.kluissleutel", format!("generatie-{}", self.generatie), "kluishoofd")
    }
}

/// De gewikkelde sleutel van één compartiment.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Compartimenthoofd {
    /// Naam van het compartiment; gaat mee in de binding.
    pub naam: String,
    /// De met de kluissleutel gewikkelde compartimentsleutel.
    pub sleutel: Envelop,
    /// Generatie van deze compartimentsleutel; loopt op bij elke rotatie.
    pub generatie: u32,
}

impl Compartimenthoofd {
    fn binding(&self) -> Binding {
        Binding::nieuw(
            "compartiment.sleutel",
            format!("{}#{}", self.naam, self.generatie),
            "kluishoofd",
        )
    }
}

/// Een geopende kluis: de kluissleutel staat in het geheugen.
///
/// Zodra deze waarde wordt opgeruimd, wordt het sleutelmateriaal overschreven.
pub struct GeopendeKluis {
    hoofd: Kluishoofd,
    kluissleutel: Kluissleutel,
}

impl std::fmt::Debug for GeopendeKluis {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GeopendeKluis")
            .field("generatie", &self.hoofd.generatie)
            .field("kluissleutel", &"<verborgen>")
            .finish()
    }
}

impl GeopendeKluis {
    /// Maakt een nieuwe kluis aan met een willekeurige kluissleutel.
    pub fn aanmaken(wachtwoord: &Wachtwoordzin, params: KdfParameters) -> Resultaat<Self> {
        params.controleer()?;

        let mut zout = [0u8; ZOUT_LENGTE];
        rand::thread_rng().fill_bytes(&mut zout);

        let mut kluissleutel = Kluissleutel::nul();
        rand::thread_rng().fill_bytes(kluissleutel.bytes_mut());

        let hoofdsleutel = kdf::leid_hoofdsleutel_af(wachtwoord, &zout, params)?;

        let voorlopig = Kluishoofd {
            versie: KLUISVERSIE,
            zout: hex::encode(zout),
            kdf: params,
            // Wordt hieronder vervangen; de binding heeft het hoofd al nodig.
            kluissleutel: Envelop { versie: 1, nonce: vec![], inhoud: vec![] },
            generatie: 1,
        };
        let gewikkeld =
            aead::versleutel(&hoofdsleutel, &voorlopig.binding(), kluissleutel.bytes())?;

        Ok(Self { hoofd: Kluishoofd { kluissleutel: gewikkeld, ..voorlopig }, kluissleutel })
    }

    /// Opent een bestaande kluis met de wachtwoordzin.
    pub fn openen(hoofd: Kluishoofd, wachtwoord: &Wachtwoordzin) -> Resultaat<Self> {
        if hoofd.versie != KLUISVERSIE {
            return Err(CryptoFout::OnbekendeVersie(hoofd.versie));
        }
        let zout = hoofd.zout_bytes()?;
        let hoofdsleutel = kdf::leid_hoofdsleutel_af(wachtwoord, &zout, hoofd.kdf)?;
        let ruw = aead::ontsleutel(&hoofdsleutel, &hoofd.binding(), &hoofd.kluissleutel)?;
        let kluissleutel = Kluissleutel::uit_slice(&ruw)?;
        Ok(Self { hoofd, kluissleutel })
    }

    pub fn hoofd(&self) -> &Kluishoofd {
        &self.hoofd
    }

    pub fn generatie(&self) -> u32 {
        self.hoofd.generatie
    }

    /// Geeft aan of de afleidingsparameters onder de huidige norm liggen.
    pub fn parameters_verouderd(&self) -> bool {
        !self.hoofd.kdf.voldoet_aan_ondergrens()
    }

    /// Maakt een nieuw compartiment met een willekeurige sleutel aan.
    pub fn compartiment_aanmaken(
        &self,
        naam: impl Into<String>,
    ) -> Resultaat<(Compartimenthoofd, Compartimentsleutel)> {
        let naam = naam.into();
        if naam.is_empty() {
            return Err(CryptoFout::OngeldigFormaat("compartimentnaam is leeg".into()));
        }
        let sleutel = aead::nieuwe_gegevenssleutel();
        let voorlopig = Compartimenthoofd {
            naam,
            sleutel: Envelop { versie: 1, nonce: vec![], inhoud: vec![] },
            generatie: 1,
        };
        let gewikkeld =
            aead::versleutel(&self.kluissleutel, &voorlopig.binding(), sleutel.bytes())?;
        Ok((Compartimenthoofd { sleutel: gewikkeld, ..voorlopig }, sleutel))
    }

    /// Ontsluit de sleutel van een compartiment.
    pub fn compartiment_openen(
        &self,
        hoofd: &Compartimenthoofd,
    ) -> Resultaat<Compartimentsleutel> {
        let ruw = aead::ontsleutel(&self.kluissleutel, &hoofd.binding(), &hoofd.sleutel)?;
        Compartimentsleutel::uit_slice(&ruw)
    }

    /// Wijzigt de wachtwoordzin.
    ///
    /// Alleen de wikkeling van de kluissleutel wordt vervangen. De kluissleutel
    /// zelf, alle compartimentsleutels en alle versleutelde gegevens blijven
    /// ongewijzigd — er wordt niets herversleuteld.
    pub fn wachtwoord_wijzigen(
        &mut self,
        nieuw: &Wachtwoordzin,
        params: KdfParameters,
    ) -> Resultaat<()> {
        params.controleer()?;
        let mut zout = [0u8; ZOUT_LENGTE];
        rand::thread_rng().fill_bytes(&mut zout);
        let hoofdsleutel = kdf::leid_hoofdsleutel_af(nieuw, &zout, params)?;

        let voorlopig = Kluishoofd {
            versie: KLUISVERSIE,
            zout: hex::encode(zout),
            kdf: params,
            kluissleutel: Envelop { versie: 1, nonce: vec![], inhoud: vec![] },
            generatie: self.hoofd.generatie,
        };
        let gewikkeld =
            aead::versleutel(&hoofdsleutel, &voorlopig.binding(), self.kluissleutel.bytes())?;
        self.hoofd = Kluishoofd { kluissleutel: gewikkeld, ..voorlopig };
        Ok(())
    }

    /// Roteert de sleutel van één compartiment.
    ///
    /// Levert de nieuwe sleutel plus het nieuwe hoofd op. De aanroeper is
    /// verantwoordelijk voor het herversleutelen van de inhoud; zolang dat niet
    /// klaar is, moet ook de oude sleutel beschikbaar blijven.
    pub fn compartiment_roteren(
        &self,
        oud: &Compartimenthoofd,
    ) -> Resultaat<(Compartimenthoofd, Compartimentsleutel)> {
        let sleutel = aead::nieuwe_gegevenssleutel();
        let voorlopig = Compartimenthoofd {
            naam: oud.naam.clone(),
            sleutel: Envelop { versie: 1, nonce: vec![], inhoud: vec![] },
            generatie: oud.generatie.saturating_add(1),
        };
        let gewikkeld =
            aead::versleutel(&self.kluissleutel, &voorlopig.binding(), sleutel.bytes())?;
        Ok((Compartimenthoofd { sleutel: gewikkeld, ..voorlopig }, sleutel))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST: KdfParameters = KdfParameters::TEST_ONVEILIG;

    fn kluis() -> GeopendeKluis {
        GeopendeKluis::aanmaken(&Wachtwoordzin::nieuw("een lange wachtwoordzin"), TEST).unwrap()
    }

    #[test]
    fn openen_met_juist_wachtwoord() {
        let w = Wachtwoordzin::nieuw("een lange wachtwoordzin");
        let k = GeopendeKluis::aanmaken(&w, TEST).unwrap();
        let hoofd = k.hoofd().clone();
        let heropend = GeopendeKluis::openen(hoofd, &w).unwrap();
        assert_eq!(heropend.kluissleutel, k.kluissleutel);
    }

    #[test]
    fn openen_met_onjuist_wachtwoord_faalt() {
        let k = kluis();
        let fout =
            GeopendeKluis::openen(k.hoofd().clone(), &Wachtwoordzin::nieuw("verkeerd wachtwoord"))
                .unwrap_err();
        assert_eq!(fout, CryptoFout::Ontsleuteling);
    }

    #[test]
    fn compartiment_heen_en_terug() {
        let k = kluis();
        let (hoofd, sleutel) = k.compartiment_aanmaken(COMPARTIMENT_ALGEMEEN).unwrap();
        let opnieuw = k.compartiment_openen(&hoofd).unwrap();
        assert_eq!(sleutel, opnieuw);
    }

    #[test]
    fn compartimenten_hebben_verschillende_sleutels() {
        let k = kluis();
        let (_, a) = k.compartiment_aanmaken("algemeen").unwrap();
        let (_, b) = k.compartiment_aanmaken("vertrouwelijk").unwrap();
        assert_ne!(a, b);
    }

    #[test]
    fn compartimentwikkeling_kan_niet_worden_omgewisseld() {
        let k = kluis();
        let (mut a, _) = k.compartiment_aanmaken("algemeen").unwrap();
        let (b, _) = k.compartiment_aanmaken("vertrouwelijk").unwrap();
        // Doe de wikkeling van B voor als die van A.
        a.sleutel = b.sleutel;
        assert_eq!(k.compartiment_openen(&a).unwrap_err(), CryptoFout::Ontsleuteling);
    }

    #[test]
    fn wachtwoord_wijzigen_behoudt_alle_sleutels() {
        let mut k = kluis();
        let (hoofd, sleutel) = k.compartiment_aanmaken("algemeen").unwrap();
        let oude_kluissleutel = k.kluissleutel.clone();

        let nieuw = Wachtwoordzin::nieuw("een heel ander wachtwoord");
        k.wachtwoord_wijzigen(&nieuw, TEST).unwrap();

        // De kluissleutel is ongewijzigd: er hoeft niets herversleuteld te worden.
        assert_eq!(k.kluissleutel, oude_kluissleutel);
        assert_eq!(k.compartiment_openen(&hoofd).unwrap(), sleutel);

        // Met het nieuwe wachtwoord gaat de kluis open, met het oude niet.
        let heropend = GeopendeKluis::openen(k.hoofd().clone(), &nieuw).unwrap();
        assert_eq!(heropend.compartiment_openen(&hoofd).unwrap(), sleutel);
        assert!(GeopendeKluis::openen(
            k.hoofd().clone(),
            &Wachtwoordzin::nieuw("een lange wachtwoordzin")
        )
        .is_err());
    }

    #[test]
    fn wachtwoord_wijzigen_vernieuwt_het_zout() {
        let mut k = kluis();
        let oud_zout = k.hoofd().zout.clone();
        k.wachtwoord_wijzigen(&Wachtwoordzin::nieuw("een heel ander wachtwoord"), TEST).unwrap();
        assert_ne!(k.hoofd().zout, oud_zout);
    }

    #[test]
    fn rotatie_levert_een_nieuwe_sleutel_en_generatie() {
        let k = kluis();
        let (hoofd, oud) = k.compartiment_aanmaken("algemeen").unwrap();
        let (nieuw_hoofd, nieuw) = k.compartiment_roteren(&hoofd).unwrap();
        assert_ne!(oud, nieuw);
        assert_eq!(nieuw_hoofd.generatie, hoofd.generatie + 1);
        // De oude wikkeling blijft bruikbaar zolang de inhoud nog niet is omgezet.
        assert_eq!(k.compartiment_openen(&hoofd).unwrap(), oud);
        assert_eq!(k.compartiment_openen(&nieuw_hoofd).unwrap(), nieuw);
    }

    #[test]
    fn kluishoofd_bevat_geen_sleutelmateriaal() {
        let k = kluis();
        let json = serde_json::to_string(k.hoofd()).unwrap();
        let ruw = k.kluissleutel.bytes();
        assert!(!json.contains(&hex::encode(ruw)));
    }

    #[test]
    fn kluishoofd_overleeft_serialisatie() {
        let w = Wachtwoordzin::nieuw("een lange wachtwoordzin");
        let k = GeopendeKluis::aanmaken(&w, TEST).unwrap();
        let (comp, sleutel) = k.compartiment_aanmaken("algemeen").unwrap();

        let hoofd_json = serde_json::to_string(k.hoofd()).unwrap();
        let comp_json = serde_json::to_string(&comp).unwrap();

        let hoofd: Kluishoofd = serde_json::from_str(&hoofd_json).unwrap();
        let comp: Compartimenthoofd = serde_json::from_str(&comp_json).unwrap();

        let heropend = GeopendeKluis::openen(hoofd, &w).unwrap();
        assert_eq!(heropend.compartiment_openen(&comp).unwrap(), sleutel);
    }

    #[test]
    fn onbekende_kluisversie_wordt_geweigerd() {
        let k = kluis();
        let mut hoofd = k.hoofd().clone();
        hoofd.versie = 99;
        assert_eq!(
            GeopendeKluis::openen(hoofd, &Wachtwoordzin::nieuw("een lange wachtwoordzin"))
                .unwrap_err(),
            CryptoFout::OnbekendeVersie(99)
        );
    }

    #[test]
    fn gewijzigde_kdf_parameters_maken_de_kluis_onleesbaar() {
        // De parameters zitten in de afleiding; ermee knoeien geeft een andere
        // hoofdsleutel en dus een mislukte ontsleuteling — geen stille degradatie.
        let k = kluis();
        let mut hoofd = k.hoofd().clone();
        hoofd.kdf = KdfParameters { iteraties: 2, ..TEST };
        assert_eq!(
            GeopendeKluis::openen(hoofd, &Wachtwoordzin::nieuw("een lange wachtwoordzin"))
                .unwrap_err(),
            CryptoFout::Ontsleuteling
        );
    }

    #[test]
    fn twee_kluizen_krijgen_verschillend_zout() {
        let w = Wachtwoordzin::nieuw("een lange wachtwoordzin");
        let a = GeopendeKluis::aanmaken(&w, TEST).unwrap();
        let b = GeopendeKluis::aanmaken(&w, TEST).unwrap();
        assert_ne!(a.hoofd().zout, b.hoofd().zout);
        assert_ne!(a.kluissleutel, b.kluissleutel);
    }
}
