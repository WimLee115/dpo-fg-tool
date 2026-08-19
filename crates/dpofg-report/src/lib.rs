//! Dossiers samenstellen en ondertekenen.
//!
//! Een dossier is wat er de deur uitgaat: naar een toezichthouder, naar een
//! auditor, naar de directie. Het bestaat uit een **manifest** — wat zit erin,
//! met welke hashes, op welke ketenstand, met welke versie van de juridische
//! inhoud — en de stukken zelf.
//!
//! # Wat het manifest wél en niet aantoont
//!
//! Het manifest maakt controleerbaar dat de bundel na samenstelling niet is
//! gewijzigd, en waaraan de inhoud is ontleend. Het toont niet aan wanneer de
//! onderliggende records zijn vastgelegd — daarvoor is de ketenstand met haar
//! anker nodig, en die reikwijdte gaat als vaste tekst mee. Dat voorbehoud
//! staat in elk dossier, en niet als kleine letters: een bundel die meer
//! suggereert dan zij waarmaakt, is bij een inspectie erger dan geen bundel.

#![forbid(unsafe_code)]
#![warn(missing_debug_implementations, rust_2018_idioms)]

pub mod prognose;
pub mod werkbak;

use chrono::{DateTime, NaiveDate, Utc};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};

const DOSSIERCONTEXT: &[u8] = b"dpo-fg-tool dossiermanifest v1";

/// De vaste tekst die onder elk dossier staat.
///
/// Uit bijlage A van het plan. Deze tekst mag niet per dossier worden
/// afgezwakt; hij staat daarom in de programmacode en niet in een sjabloon.
pub const VOORBEHOUD: &str = "\
De integriteit van dit dossier is te controleren met de gepubliceerde \
formaatspecificatie en de bijgevoegde ankerbestanden. De ketenverificatie toont \
aan dat de inhoud na vastlegging niet ongemerkt is gewijzigd. Zij toont uit \
zichzelf niet aan op welk moment een record is vastgelegd; die vaststelling \
berust op de bijgevoegde externe tijdstempels. Ontbreken die, dan berust het \
tijdstip uitsluitend op de opgave van de organisatie zelf, en is dat in dit \
dossier als zodanig aangemerkt.";

/// Eén stuk in het dossier.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Dossierstuk {
    /// Bestandsnaam binnen de bundel.
    pub naam: String,
    /// Soort record.
    pub soort: String,
    /// Identificatie van het bronrecord.
    pub bron_id: String,
    /// De versie van het bronrecord op het moment van samenstellen.
    pub bron_versie: u32,
    /// Hash van de inhoud zoals opgenomen.
    pub hash: String,
    pub omvang: u64,
    /// Of het stuk is bewerkt vóór opname, bijvoorbeeld door onleesbaar maken.
    pub bewerkt: bool,
    /// Waarop de bewerking berust.
    pub bewerking_grondslag: Option<String>,
}

/// Het manifest van een dossier.
///
/// `deny_unknown_fields` op dit type en op [`Dossierstuk`] en [`Weglating`]:
/// een manifest met een veld dat deze uitgave niet kent, wordt geweigerd in
/// plaats van uitgekleed ingelezen. Zonder die regel zou de handtekening
/// worden gecontroleerd over een object waaruit stilzwijgend iets is
/// weggevallen, en dan is een geslaagde controle een vals groen vinkje.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub formaatversie: u8,
    /// Waarvoor het dossier is samengesteld.
    pub aanleiding: String,
    /// Voor wie het bestemd is.
    pub bestemd_voor: String,
    pub samengesteld_op: DateTime<Utc>,
    pub samengesteld_door: String,
    /// De periode waarover het dossier gaat.
    pub periode_van: Option<NaiveDate>,
    pub periode_tot: Option<NaiveDate>,

    /// De stand van het ketenlogboek op het moment van samenstellen.
    pub keten_volgnummer: u64,
    pub keten_hash: String,
    /// Het laatst geplaatste anker, als tekst.
    pub anker_omschrijving: Option<String>,
    /// De reikwijdte van de verificatie, letterlijk uit het rapport.
    pub reikwijdte: String,

    /// Welke versie van de juridische inhoud is gebruikt.
    pub kennispakket_code: String,
    pub kennispakket_versie: String,
    pub kennispakket_consolidatiedatum: NaiveDate,

    /// De programmaversie die het dossier heeft samengesteld.
    pub programmaversie: String,

    pub stukken: Vec<Dossierstuk>,

    /// Onderdelen die bewust buiten het dossier zijn gelaten.
    ///
    /// Verzwijgen wat er ontbreekt is de snelste manier om het vertrouwen in
    /// een dossier te verliezen. Wat er niet in zit en waarom, hoort erin.
    pub weggelaten: Vec<Weglating>,

    /// Het vaste voorbehoud.
    pub voorbehoud: String,
}

/// Iets dat bewust niet in het dossier zit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Weglating {
    pub omschrijving: String,
    pub reden: String,
    pub aantal: usize,
}

impl Manifest {
    #[allow(clippy::too_many_arguments)]
    pub fn nieuw(
        aanleiding: impl Into<String>,
        bestemd_voor: impl Into<String>,
        samengesteld_door: impl Into<String>,
        nu: DateTime<Utc>,
        keten_volgnummer: u64,
        keten_hash: impl Into<String>,
        reikwijdte: impl Into<String>,
        kennispakket_code: impl Into<String>,
        kennispakket_versie: impl Into<String>,
        kennispakket_consolidatiedatum: NaiveDate,
    ) -> Self {
        Self {
            formaatversie: 1,
            aanleiding: aanleiding.into(),
            bestemd_voor: bestemd_voor.into(),
            samengesteld_op: nu,
            samengesteld_door: samengesteld_door.into(),
            periode_van: None,
            periode_tot: None,
            keten_volgnummer,
            keten_hash: keten_hash.into(),
            anker_omschrijving: None,
            reikwijdte: reikwijdte.into(),
            kennispakket_code: kennispakket_code.into(),
            kennispakket_versie: kennispakket_versie.into(),
            kennispakket_consolidatiedatum,
            programmaversie: env!("CARGO_PKG_VERSION").into(),
            stukken: Vec::new(),
            weggelaten: Vec::new(),
            voorbehoud: VOORBEHOUD.into(),
        }
    }

    /// Voegt een stuk toe en berekent de hash.
    pub fn voeg_toe(
        &mut self,
        naam: impl Into<String>,
        soort: impl Into<String>,
        bron_id: impl Into<String>,
        bron_versie: u32,
        inhoud: &[u8],
    ) {
        self.stukken.push(Dossierstuk {
            naam: naam.into(),
            soort: soort.into(),
            bron_id: bron_id.into(),
            bron_versie,
            hash: blake3::hash(inhoud).to_hex().to_string(),
            omvang: inhoud.len() as u64,
            bewerkt: false,
            bewerking_grondslag: None,
        });
    }

    /// Legt vast dat er iets is weggelaten.
    pub fn laat_weg(
        &mut self,
        omschrijving: impl Into<String>,
        reden: impl Into<String>,
        aantal: usize,
    ) {
        self.weggelaten.push(Weglating {
            omschrijving: omschrijving.into(),
            reden: reden.into(),
            aantal,
        });
    }

    fn te_ondertekenen(&self) -> Result<Vec<u8>, serde_json::Error> {
        let json = serde_json::to_vec(self)?;
        let mut uit = Vec::with_capacity(DOSSIERCONTEXT.len() + 8 + json.len());
        uit.extend_from_slice(DOSSIERCONTEXT);
        uit.extend_from_slice(&(json.len() as u64).to_be_bytes());
        uit.extend_from_slice(&json);
        Ok(uit)
    }

    /// Totale omvang van de opgenomen stukken.
    pub fn totale_omvang(&self) -> u64 {
        self.stukken.iter().map(|s| s.omvang).sum()
    }
}

/// Een ondertekend dossiermanifest.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OndertekendManifest {
    pub manifest: Manifest,
    pub ondertekenaar: String,
    pub handtekening: String,
}

impl OndertekendManifest {
    pub fn onderteken(manifest: Manifest, sleutel: &SigningKey) -> Result<Self, serde_json::Error> {
        let boodschap = manifest.te_ondertekenen()?;
        let handtekening: Signature = sleutel.sign(&boodschap);
        Ok(Self {
            manifest,
            ondertekenaar: hex::encode(sleutel.verifying_key().to_bytes()),
            handtekening: hex::encode(handtekening.to_bytes()),
        })
    }

    /// Controleert de handtekening.
    ///
    /// **Dit toont uitsluitend zelfconsistentie aan.** De sleutel waartegen
    /// wordt gecontroleerd, staat in het manifest zelf. Om vast te stellen dat
    /// het dossier van een bepaalde installatie komt, is
    /// [`OndertekendManifest::controleer_ondertekenaar`] nodig, met een sleutel
    /// die langs een ander kanaal is verkregen dan het dossier.
    pub fn controleer(&self) -> Result<(), String> {
        let sleutelbytes =
            hex::decode(&self.ondertekenaar).map_err(|e| format!("sleutel is geen hex: {e}"))?;
        let sleutelarray: [u8; 32] = sleutelbytes
            .try_into()
            .map_err(|_| "sleutel heeft niet de juiste lengte".to_string())?;
        let vk = VerifyingKey::from_bytes(&sleutelarray)
            .map_err(|e| format!("ongeldige sleutel: {e}"))?;

        let hbytes = hex::decode(&self.handtekening)
            .map_err(|e| format!("handtekening is geen hex: {e}"))?;
        let harray: [u8; 64] = hbytes
            .try_into()
            .map_err(|_| "handtekening heeft niet de juiste lengte".to_string())?;
        let sig = Signature::from_bytes(&harray);

        let boodschap =
            self.manifest.te_ondertekenen().map_err(|e| format!("serialisatie: {e}"))?;
        vk.verify(&boodschap, &sig)
            .map_err(|_| "de handtekening klopt niet met de inhoud".to_string())
    }

    /// Controleert dat dit manifest van een van de opgegeven installaties komt.
    ///
    /// Eerst de sleutel, dan pas de handtekening — anders meldt de controle een
    /// dossier van een vreemde installatie eerst als geldig.
    pub fn controleer_ondertekenaar(&self, vertrouwde: &[String]) -> Result<(), String> {
        let deze = self.ondertekenaar.to_ascii_lowercase();
        if !vertrouwde.iter().any(|v| v.trim().to_ascii_lowercase() == deze) {
            return Err(format!(
                "het manifest is ondertekend met sleutel {deze}; die staat niet in de lijst met sleutels waarmee u vergelijkt"
            ));
        }
        self.controleer()
    }

    /// Controleert dat het vaste voorbehoud niet is afgezwakt.
    ///
    /// Het voorbehoud wordt meegetekend, dus wie de tekst ná het ondertekenen
    /// aanpast, verbreekt de handtekening. Wie hem *vóór* het ondertekenen
    /// aanpast, komt door [`OndertekendManifest::controleer`] heen — en juist
    /// dat is het geval dat telt, want dat is het geval dat de aanleverende
    /// organisatie zelf in de hand heeft.
    pub fn controleer_voorbehoud(&self) -> Result<(), String> {
        if self.manifest.voorbehoud == VOORBEHOUD {
            Ok(())
        } else {
            Err("het voorbehoud in dit manifest wijkt af van de vaste tekst; \
                 de tekst is vóór het ondertekenen aangepast"
                .to_string())
        }
    }

    /// Controleert of de bijgeleverde stukken overeenkomen met het manifest.
    pub fn controleer_stukken(&self, stukken: &[(String, Vec<u8>)]) -> Vec<String> {
        let mut afwijkingen = Vec::new();
        for stuk in &self.manifest.stukken {
            match stukken.iter().find(|(naam, _)| naam == &stuk.naam) {
                None => afwijkingen
                    .push(format!("'{}' staat in het manifest maar ontbreekt", stuk.naam)),
                Some((_, inhoud)) => {
                    let hash = blake3::hash(inhoud).to_hex().to_string();
                    if hash != stuk.hash {
                        afwijkingen.push(format!(
                            "'{}' komt niet overeen met de hash in het manifest",
                            stuk.naam
                        ));
                    }
                }
            }
        }
        for (naam, _) in stukken {
            if !self.manifest.stukken.iter().any(|s| &s.naam == naam) {
                afwijkingen
                    .push(format!("'{naam}' zit in de bundel maar staat niet in het manifest"));
            }
        }
        afwijkingen
    }
}

/// Maakt een nieuw ondertekensleutelpaar aan.
///
/// In productie tekent de tool met de vaste installatiesleutel uit de kluis;
/// die overleeft een wachtwoordwissel en is via `dpofg kluis sleutel` te
/// publiceren. Deze functie bestaat voor tests en voorbeelden.
pub fn nieuw_sleutelpaar() -> SigningKey {
    SigningKey::generate(&mut rand::rngs::OsRng)
}

/// Versie van deze crate.
pub const VERSIE: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn manifest() -> Manifest {
        Manifest::nieuw(
            "uitvraag toezichthouder van 12 augustus 2026",
            "Autoriteit Persoonsgegevens",
            "A. de Vries",
            Utc.with_ymd_and_hms(2026, 8, 18, 9, 0, 0).unwrap(),
            412,
            "a".repeat(64),
            "De keten is bevestigd tot en met regel 400 door een geldig anker.",
            "nl-start",
            "0.1-start",
            NaiveDate::from_ymd_opt(2026, 8, 18).unwrap(),
        )
    }

    #[test]
    fn het_voorbehoud_staat_er_altijd_in() {
        let m = manifest();
        assert_eq!(m.voorbehoud, VOORBEHOUD);
        assert!(m.voorbehoud.contains("toont uit zichzelf niet aan op welk moment"));
    }

    #[test]
    fn ondertekenen_en_controleren() {
        let sleutel = nieuw_sleutelpaar();
        let o = OndertekendManifest::onderteken(manifest(), &sleutel).unwrap();
        assert!(o.controleer().is_ok());
    }

    #[test]
    fn een_gewijzigd_manifest_valt_door_de_mand() {
        let sleutel = nieuw_sleutelpaar();
        let mut o = OndertekendManifest::onderteken(manifest(), &sleutel).unwrap();
        o.manifest.bestemd_voor = "iemand anders".into();
        assert!(o.controleer().is_err());
    }

    #[test]
    fn een_afgezwakt_voorbehoud_valt_door_de_mand() {
        let sleutel = nieuw_sleutelpaar();
        let mut o = OndertekendManifest::onderteken(manifest(), &sleutel).unwrap();
        o.manifest.voorbehoud = "alles is aantoonbaar".into();
        assert!(o.controleer().is_err(), "het voorbehoud is meegetekend");
    }

    #[test]
    fn een_manifest_van_een_vreemde_sleutel_wordt_geweigerd() {
        let vreemde = nieuw_sleutelpaar();
        let eigen = nieuw_sleutelpaar();
        let o = OndertekendManifest::onderteken(manifest(), &vreemde).unwrap();

        assert!(o.controleer().is_ok());
        let publiek_eigen = hex::encode(eigen.verifying_key().to_bytes());
        let fout = o.controleer_ondertekenaar(&[publiek_eigen]).unwrap_err();
        assert!(fout.contains(&hex::encode(vreemde.verifying_key().to_bytes())));
    }

    #[test]
    fn een_manifest_van_de_eigen_sleutel_wordt_aanvaard() {
        let sleutel = nieuw_sleutelpaar();
        let o = OndertekendManifest::onderteken(manifest(), &sleutel).unwrap();
        let publiek = hex::encode(sleutel.verifying_key().to_bytes());

        assert!(o.controleer_ondertekenaar(std::slice::from_ref(&publiek)).is_ok());
        assert!(o.controleer_ondertekenaar(&[format!(" {} ", publiek.to_uppercase())]).is_ok());
        assert!(o.controleer_ondertekenaar(&[]).is_err());
    }

    #[test]
    fn een_vervangen_ondertekenaar_valt_door_de_pin() {
        // Dit is het gat dat de pin moet dichten: wie het manifest opnieuw
        // ondertekent met een eigen sleutel én het veld `ondertekenaar`
        // meeverandert, komt vandaag ongehinderd door `controleer()`.
        let vervalser = nieuw_sleutelpaar();
        let echte = nieuw_sleutelpaar();
        let echt = OndertekendManifest::onderteken(manifest(), &echte).unwrap();

        let mut vervalst =
            OndertekendManifest::onderteken(echt.manifest.clone(), &vervalser).unwrap();
        vervalst.manifest.bestemd_voor = "een andere ontvanger".into();
        let vervalst = OndertekendManifest::onderteken(vervalst.manifest, &vervalser).unwrap();

        assert!(vervalst.controleer().is_ok(), "zelfconsistent, dus geldig");
        let publiek_echt = hex::encode(echte.verifying_key().to_bytes());
        assert!(vervalst.controleer_ondertekenaar(&[publiek_echt]).is_err());
    }

    #[test]
    fn een_vooraf_afgezwakt_voorbehoud_wordt_geweigerd() {
        let sleutel = nieuw_sleutelpaar();
        let mut m = manifest();
        m.voorbehoud = "de integriteit van dit dossier staat vast".into();
        let o = OndertekendManifest::onderteken(m, &sleutel).unwrap();

        assert!(o.controleer().is_ok(), "de afgezwakte tekst is netjes meegetekend");
        assert!(o.controleer_voorbehoud().is_err());
    }

    #[test]
    fn het_vaste_voorbehoud_komt_door_de_controle() {
        let sleutel = nieuw_sleutelpaar();
        let o = OndertekendManifest::onderteken(manifest(), &sleutel).unwrap();
        assert!(o.controleer_voorbehoud().is_ok());
    }

    /// Met de hand vastgelegd, niet met de huidige code gegenereerd: deze test
    /// moet blijven falen wanneer iemand de veldnamen of de vorm van het
    /// manifest wijzigt, ook als de code dan nog met zichzelf overweg kan.
    #[test]
    fn een_manifest_van_formaatversie_1_blijft_leesbaar() {
        let json = format!(
            r#"{{
              "manifest": {{
                "formaatversie": 1,
                "aanleiding": "uitvraag van 12 augustus 2026",
                "bestemd_voor": "de toezichthouder",
                "samengesteld_op": "2026-08-18T09:00:00Z",
                "samengesteld_door": "A. de Vries",
                "periode_van": null,
                "periode_tot": null,
                "keten_volgnummer": 412,
                "keten_hash": "{hash}",
                "anker_omschrijving": null,
                "reikwijdte": "De keten is bevestigd tot en met regel 400.",
                "kennispakket_code": "nl-start",
                "kennispakket_versie": "0.1-start",
                "kennispakket_consolidatiedatum": "2026-08-18",
                "programmaversie": "0.1.0",
                "stukken": [
                  {{
                    "naam": "verwerking-0412-K.json",
                    "soort": "verwerking",
                    "bron_id": "0198f2c1",
                    "bron_versie": 3,
                    "hash": "{hash}",
                    "omvang": 2481,
                    "bewerkt": false,
                    "bewerking_grondslag": null
                  }}
                ],
                "weggelaten": [
                  {{
                    "omschrijving": "verwerkingen met de status concept",
                    "reden": "concepten zijn nog niet vastgesteld",
                    "aantal": 3
                  }}
                ],
                "voorbehoud": {voorbehoud}
              }},
              "ondertekenaar": "{hash}",
              "handtekening": "{handtekening}"
            }}"#,
            hash = "a".repeat(64),
            handtekening = "b".repeat(128),
            voorbehoud = serde_json::to_string(VOORBEHOUD).unwrap(),
        );

        let gelezen: OndertekendManifest = serde_json::from_str(&json).unwrap();
        assert_eq!(gelezen.manifest.formaatversie, 1);
        assert_eq!(gelezen.manifest.stukken.len(), 1);
        assert_eq!(gelezen.manifest.weggelaten[0].aantal, 3);
        assert!(gelezen.controleer_voorbehoud().is_ok());
    }

    #[test]
    fn een_onbekend_veld_in_het_manifest_wordt_geweigerd() {
        let sleutel = nieuw_sleutelpaar();
        let o = OndertekendManifest::onderteken(manifest(), &sleutel).unwrap();
        let mut waarde: serde_json::Value = serde_json::to_value(&o).unwrap();
        waarde["manifest"]["toegevoegd_veld"] = serde_json::json!("iets");
        assert!(serde_json::from_value::<OndertekendManifest>(waarde).is_err());
    }

    #[test]
    fn een_onbekend_veld_naast_het_manifest_wordt_geweigerd() {
        // Het buitenste niveau valt per constructie buiten de ondertekende
        // bytes. Juist daarom mag er niets stilzwijgend bij: een lezer die de
        // gepubliceerde specificatie heeft nagebouwd, weigert zo'n bestand, en
        // dan hoort de meegeleverde binary dat ook te doen.
        let sleutel = nieuw_sleutelpaar();
        let o = OndertekendManifest::onderteken(manifest(), &sleutel).unwrap();
        let mut waarde: serde_json::Value = serde_json::to_value(&o).unwrap();
        waarde["opmerking_van_de_organisatie"] = serde_json::json!("door de AP goedgekeurd");
        assert!(serde_json::from_value::<OndertekendManifest>(waarde).is_err());
    }

    #[test]
    fn een_onbekend_veld_in_een_stuk_wordt_geweigerd() {
        let sleutel = nieuw_sleutelpaar();
        let mut m = manifest();
        m.voeg_toe("register.json", "verwerking", "0412-K", 3, b"inhoud");
        let o = OndertekendManifest::onderteken(m, &sleutel).unwrap();
        let mut waarde: serde_json::Value = serde_json::to_value(&o).unwrap();
        waarde["manifest"]["stukken"][0]["extra"] = serde_json::json!(1);
        assert!(serde_json::from_value::<OndertekendManifest>(waarde).is_err());
    }

    #[test]
    fn stukken_worden_gehasht_en_gecontroleerd() {
        let sleutel = nieuw_sleutelpaar();
        let mut m = manifest();
        m.voeg_toe("register.json", "verwerking", "0412-K", 3, b"inhoud van het register");
        m.voeg_toe("incident.json", "incident", "2026-0041", 1, b"inhoud van het incident");
        assert_eq!(m.totale_omvang(), 23 + 23);

        let o = OndertekendManifest::onderteken(m, &sleutel).unwrap();
        let goed = vec![
            ("register.json".to_string(), b"inhoud van het register".to_vec()),
            ("incident.json".to_string(), b"inhoud van het incident".to_vec()),
        ];
        assert!(o.controleer_stukken(&goed).is_empty());

        let gewijzigd = vec![
            ("register.json".to_string(), b"stiekem aangepast!!!!!!".to_vec()),
            ("incident.json".to_string(), b"inhoud van het incident".to_vec()),
        ];
        let afwijkingen = o.controleer_stukken(&gewijzigd);
        assert_eq!(afwijkingen.len(), 1);
        assert!(afwijkingen[0].contains("register.json"));
    }

    #[test]
    fn een_ontbrekend_of_extra_stuk_valt_op() {
        let sleutel = nieuw_sleutelpaar();
        let mut m = manifest();
        m.voeg_toe("register.json", "verwerking", "0412-K", 3, b"inhoud");
        let o = OndertekendManifest::onderteken(m, &sleutel).unwrap();

        assert!(o.controleer_stukken(&[]).iter().any(|a| a.contains("ontbreekt")));

        let extra = vec![
            ("register.json".to_string(), b"inhoud".to_vec()),
            ("stiekem.json".to_string(), b"toegevoegd".to_vec()),
        ];
        assert!(o
            .controleer_stukken(&extra)
            .iter()
            .any(|a| a.contains("staat niet in het manifest")));
    }

    /// Verzwijgen wat er ontbreekt is de snelste manier om vertrouwen te verliezen.
    #[test]
    fn weglatingen_horen_in_het_manifest() {
        let mut m = manifest();
        m.laat_weg(
            "dossiers uit het persoonlijke compartiment van de functionaris",
            "die vallen buiten de uitvraag en berusten bij de functionaris zelf",
            3,
        );
        assert_eq!(m.weggelaten.len(), 1);
        assert_eq!(m.weggelaten[0].aantal, 3);
    }

    #[test]
    fn het_manifest_overleeft_serialisatie() {
        let sleutel = nieuw_sleutelpaar();
        let mut m = manifest();
        m.voeg_toe("register.json", "verwerking", "0412-K", 3, b"inhoud");
        let o = OndertekendManifest::onderteken(m, &sleutel).unwrap();

        let json = serde_json::to_string(&o).unwrap();
        let terug: OndertekendManifest = serde_json::from_str(&json).unwrap();
        assert_eq!(o, terug);
        assert!(terug.controleer().is_ok());
    }
}
