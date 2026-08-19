//! De kluis: het versleutelde bestand waarin alles staat.
//!
//! # Wat deze laag afdwingt
//!
//! 1. **Niets wordt opgeslagen zonder logboekregel.** Bewaren en loggen
//!    gebeuren in één transactie. Faalt het loggen, dan gaat het bewaren
//!    ook niet door. Er is geen functie die het logboek overslaat.
//! 2. **Niets wordt hard overschreven.** Elke wijziging schrijft de vorige
//!    versie weg, zodat ongedaan maken altijd kan en de wijziging te tonen is.
//! 3. **Een gesloten compartiment is onleesbaar**, niet afgeschermd. Wie de
//!    sleutel niet heeft, krijgt geen leeg resultaat maar een expliciete
//!    melding dat het compartiment dicht is.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use dpofg_audit::{keten_aan, Actor, Gebeurtenis, Handeling, Ketenregel, Ketenstand};
use dpofg_crypto::{
    aead::{self, Binding, Envelop},
    identiteit::{Installatiesleutel, SigningKey},
    kdf::KdfParameters,
    keys::{Compartimenthoofd, Compartimentsleutel, GeopendeKluis, Kluishoofd},
    Wachtwoordzin,
};
use rusqlite::OpenFlags;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{de::DeserializeOwned, Serialize};

use crate::{schema, Resultaat, StoreFout};

/// Vaste aanduiding waaraan een kluisbestand te herkennen is.
const KLUISMERK: i32 = 0x4450_4647; // "DPFG"

/// Generatie van de ondertekenidentiteit.
///
/// Vast op 1 zolang er geen rotatie is. De kolom bestaat wél al, en het
/// generatienummer gaat mee in de binding van de wikkeling, zodat rotatie later
/// geen schemawijziging vergt.
const GENERATIE: u32 = 1;

/// Een geopende kluis.
pub struct Kluis {
    conn: Connection,
    pad: PathBuf,
    sleutels: GeopendeKluis,
    /// De compartimenten die op dit moment ontgrendeld zijn.
    ontgrendeld: BTreeMap<String, Compartimentsleutel>,
    /// De hoofden van alle compartimenten in deze kluis.
    compartimenten: BTreeMap<String, Compartimenthoofd>,
    ketenstand: Ketenstand,
    /// De vaste ondertekenidentiteit van deze installatie.
    installatie: Installatiesleutel,
    installatie_aangemaakt_op: DateTime<Utc>,
}

impl std::fmt::Debug for Kluis {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Kluis")
            .field("pad", &self.pad)
            .field("compartimenten", &self.compartimenten.keys().collect::<Vec<_>>())
            .field("ontgrendeld", &self.ontgrendeld.keys().collect::<Vec<_>>())
            .field("logboekregels", &self.ketenstand.volgnummer)
            .field("installatiesleutel", &kort(self.installatie.publieke_sleutel()))
            .finish()
    }
}

/// De eerste zestien tekens van een sleutel of hash, voor weergave.
fn kort(waarde: &str) -> String {
    waarde.chars().take(16).collect()
}

/// De publieke gegevens van de ondertekenidentiteit, zonder wachtwoord te lezen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Installatiekop {
    pub publieke_sleutel: String,
    pub generatie: u32,
    pub aangemaakt_op: DateTime<Utc>,
}

/// Beknopte gegevens van een record, zonder de versleutelde inhoud.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Recordkop {
    pub id: String,
    pub soort: String,
    pub compartiment: String,
    pub status: String,
    pub kenmerk: Option<String>,
    pub aangemaakt_op: DateTime<Utc>,
    pub gewijzigd_op: DateTime<Utc>,
    pub versie: u32,
}

impl Kluis {
    /// Maakt een nieuwe kluis aan.
    pub fn aanmaken(
        pad: impl AsRef<Path>,
        wachtwoord: &Wachtwoordzin,
        params: KdfParameters,
        nu: DateTime<Utc>,
    ) -> Resultaat<Self> {
        let pad = pad.as_ref().to_path_buf();
        if pad.exists() {
            return Err(StoreFout::Bestand(std::io::Error::new(
                std::io::ErrorKind::AlreadyExists,
                format!("{} bestaat al", pad.display()),
            )));
        }

        let conn = Connection::open(&pad)?;
        schema::stel_verbinding_in(&conn)?;
        conn.pragma_update(None, "application_id", KLUISMERK)?;
        schema::migreer(&conn)?;

        let sleutels = GeopendeKluis::aanmaken(wachtwoord, params)?;
        conn.execute(
            "INSERT INTO kluishoofd (id, hoofd_json, aangemaakt_op, programmaversie)
             VALUES (1, ?1, ?2, ?3)",
            params![
                serde_json::to_string(sleutels.hoofd())?,
                nu.to_rfc3339(),
                env!("CARGO_PKG_VERSION")
            ],
        )?;

        let (envelop, installatie) = sleutels.installatie_aanmaken(GENERATIE)?;
        schrijf_installatie(&conn, GENERATIE, &installatie, &envelop, nu)?;

        let mut kluis = Self {
            conn,
            pad,
            sleutels,
            ontgrendeld: BTreeMap::new(),
            compartimenten: BTreeMap::new(),
            ketenstand: Ketenstand::leeg(),
            installatie,
            installatie_aangemaakt_op: nu,
        };

        // Het algemene compartiment bestaat altijd.
        kluis.compartiment_aanmaken(
            dpofg_crypto::keys::COMPARTIMENT_ALGEMEEN,
            "gegevens zonder bijzondere vertrouwelijkheid",
            nu,
        )?;

        kluis.log(
            Gebeurtenis::nieuw(
                Handeling::KluisAangemaakt,
                Actor::systeem(),
                nu,
                "kluis",
                "1",
                dpofg_crypto::keys::COMPARTIMENT_ALGEMEEN,
                format!(
                    "kluis aangemaakt met schemaversie {}; installatiesleutel {}",
                    schema::SCHEMAVERSIE,
                    kluis.installatie.publieke_sleutel()
                ),
            ),
            None,
        )?;

        Ok(kluis)
    }

    /// Opent een bestaande kluis.
    pub fn openen(
        pad: impl AsRef<Path>,
        wachtwoord: &Wachtwoordzin,
        nu: DateTime<Utc>,
    ) -> Resultaat<Self> {
        let pad = pad.as_ref().to_path_buf();
        let conn = Connection::open(&pad)?;
        schema::stel_verbinding_in(&conn)?;

        let merk: i32 = conn.pragma_query_value(None, "application_id", |r| r.get(0))?;
        if merk != KLUISMERK {
            return Err(StoreFout::GeenKluisbestand(pad.display().to_string()));
        }
        schema::migreer(&conn)?;

        let hoofd_json: String = conn
            .query_row("SELECT hoofd_json FROM kluishoofd WHERE id = 1", [], |r| r.get(0))
            .optional()?
            .ok_or_else(|| StoreFout::GeenKluisbestand(pad.display().to_string()))?;
        let hoofd: Kluishoofd = serde_json::from_str(&hoofd_json)?;

        let sleutels = GeopendeKluis::openen(hoofd, wachtwoord)?;

        let mut compartimenten = BTreeMap::new();
        {
            let mut stmt = conn.prepare("SELECT naam, hoofd_json FROM compartiment")?;
            let rijen =
                stmt.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))?;
            for rij in rijen {
                let (naam, json) = rij?;
                compartimenten.insert(naam, serde_json::from_str::<Compartimenthoofd>(&json)?);
            }
        }

        let ketenstand = lees_ketenstand(&conn)?;

        // De ondertekenidentiteit. Een kluis van schemaversie 1 heeft er nog
        // geen; die wordt hier alsnog aangemaakt. Dat kan niet in
        // `schema::migreer`, want die draait vóór het wachtwoord is verwerkt en
        // heeft dus geen sleutelmateriaal om iets mee te wikkelen.
        let bestaand = lees_installatie(&conn)?;
        let (installatie, aangemaakt_op, was_nieuw) = match bestaand {
            Some((generatie, publiek, envelop, aangemaakt_op)) => {
                let sleutel = sleutels.installatie_openen(&envelop, generatie)?;
                if sleutel.publieke_sleutel() != publiek {
                    return Err(StoreFout::InstallatiesleutelWijktAf {
                        in_kluis: publiek,
                        uit_zaad: sleutel.publieke_sleutel().to_string(),
                    });
                }
                (sleutel, aangemaakt_op, false)
            }
            None => {
                let (envelop, sleutel) = sleutels.installatie_aanmaken(GENERATIE)?;
                schrijf_installatie(&conn, GENERATIE, &sleutel, &envelop, nu)?;
                (sleutel, nu, true)
            }
        };

        let mut kluis = Self {
            conn,
            pad,
            sleutels,
            ontgrendeld: BTreeMap::new(),
            compartimenten,
            ketenstand,
            installatie,
            installatie_aangemaakt_op: aangemaakt_op,
        };

        kluis.log(
            Gebeurtenis::nieuw(
                Handeling::KluisGeopend,
                Actor::systeem(),
                nu,
                "kluis",
                "1",
                dpofg_crypto::keys::COMPARTIMENT_ALGEMEEN,
                "kluis geopend",
            ),
            None,
        )?;

        if was_nieuw {
            // In de keten zelf vastleggen vanaf welk volgnummer de
            // installatiesleutel geldt. Wat daarvóór is uitgeleverd, draagt een
            // wegwerpsleutel en kan dus nooit met deze sleutel overeenkomen.
            let publiek = kluis.installatie.publieke_sleutel().to_string();
            kluis.log(
                Gebeurtenis::nieuw(
                    Handeling::InstallatiesleutelAangemaakt,
                    Actor::systeem(),
                    nu,
                    "kluis",
                    "1",
                    dpofg_crypto::keys::COMPARTIMENT_ALGEMEEN,
                    format!(
                        "ondertekenidentiteit {publiek} aangemaakt; ankers en dossiers van vóór deze regel dragen een wegwerpsleutel"
                    ),
                ),
                None,
            )?;
        }

        Ok(kluis)
    }

    pub fn pad(&self) -> &Path {
        &self.pad
    }

    /// De publieke installatiesleutel: 64 hexadecimale tekens.
    ///
    /// Dit is de waarde die in elk anker en elk dossiermanifest van deze kluis
    /// terechtkomt, en die de organisatie langs een ander kanaal publiceert.
    pub fn installatiesleutel(&self) -> &str {
        self.installatie.publieke_sleutel()
    }

    /// Wanneer de ondertekenidentiteit is aangemaakt.
    pub fn installatiesleutel_aangemaakt_op(&self) -> DateTime<Utc> {
        self.installatie_aangemaakt_op
    }

    /// Tekent iets met de installatiesleutel.
    ///
    /// De enige uitgang naar de privésleutel. Er is bewust geen accessor die
    /// de sleutel of het zaad teruggeeft: code die het sleutelmateriaal ergens
    /// anders heen brengt, moet er dan opzettelijk uitzien. Een harde garantie
    /// is dat niet — `SigningKey` is `Clone` — maar wel een leesbare.
    pub fn onderteken_met<T>(&self, f: impl FnOnce(&SigningKey) -> T) -> T {
        f(self.installatie.ondertekensleutel())
    }

    /// Leest de publieke installatiesleutel zonder de kluis te openen.
    ///
    /// Geen wachtwoord en geen sleutelafleiding. Bedoeld voor het publiceren
    /// van de sleutel: een wachtwoordzin intypen om publiek materiaal voor te
    /// lezen, is een gewoonte die je niet wilt aanleren.
    ///
    /// # Wat er wel en niet wordt gecontroleerd
    ///
    /// De kolom `publieke_sleutel` staat in klare tekst en is dus met een
    /// gewone databasebewerking te wijzigen door iemand die het bestand kan
    /// beschrijven maar het wachtwoord niet kent. Zou deze functie die waarde
    /// ongetoetst teruggeven, dan zou juist die tegenstander de organisatie een
    /// vreemde sleutel kunnen laten publiceren. De waarde wordt daarom
    /// vergeleken met de sleutel die in het **ketenlogboek** staat: die is
    /// opgenomen in de hashketen en dus niet te wijzigen zonder de keten te
    /// breken.
    ///
    /// Wat hier níet gebeurt, is het narekenen van de hele keten — dat is
    /// `dpofg logboek verifieer`, en dat vereist de kluis. Iemand die zowel de
    /// kolom als de logregel aanpast, komt hierlangs; hij breekt daarmee wel de
    /// keten, en dat is precies wat de verificatie aantoont.
    ///
    /// Het bestand wordt niet gewijzigd, maar SQLite kan er in de normale modus
    /// een WAL-index (`-wal`, `-shm`) naast aanleggen. Lukt dat niet — een
    /// alleen-lezen medium of een teruggezette reservekopie op een
    /// niet-schrijfbare map — dan wordt het bestand als onveranderlijk geopend
    /// en blijft de map onaangeroerd.
    ///
    /// Levert `Ok(None)` bij een kluis van schemaversie 1, die nog geen
    /// identiteit draagt.
    pub fn installatiesleutel_lezen(pad: impl AsRef<Path>) -> Resultaat<Option<Installatiekop>> {
        let pad = pad.as_ref();
        let conn = open_alleen_lezen(pad)?;

        let merk: i32 = conn.pragma_query_value(None, "application_id", |r| r.get(0))?;
        if merk != KLUISMERK {
            return Err(StoreFout::GeenKluisbestand(pad.display().to_string()));
        }

        let heeft_tabel: bool = conn
            .query_row(
                "SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = 'installatie'",
                [],
                |r| r.get::<_, i32>(0),
            )
            .optional()?
            .is_some();
        if !heeft_tabel {
            return Ok(None);
        }

        let rij = conn
            .query_row(
                "SELECT generatie, publieke_sleutel, aangemaakt_op FROM installatie WHERE id = 1",
                [],
                |r| {
                    Ok((r.get::<_, i64>(0)? as u32, r.get::<_, String>(1)?, r.get::<_, String>(2)?))
                },
            )
            .optional()?;

        let Some((generatie, publieke_sleutel, aangemaakt)) = rij else {
            return Ok(None);
        };

        if let Some(in_logboek) = sleutel_uit_logboek(&conn)? {
            if !in_logboek.eq_ignore_ascii_case(&publieke_sleutel) {
                return Err(StoreFout::InstallatiesleutelWijktAfVanLogboek {
                    in_kluis: publieke_sleutel,
                    in_logboek,
                });
            }
        }

        Ok(Some(Installatiekop {
            publieke_sleutel,
            generatie,
            aangemaakt_op: parse_tijd(&aangemaakt),
        }))
    }

    /// Of de afleidingsparameters onder de huidige norm liggen.
    ///
    /// Wordt bij het openen getoond met het aanbod om te verzwaren. Stil laten
    /// staan zou betekenen dat een kluis uit 2026 in 2032 nog met de
    /// parameters van 2026 beschermd is.
    pub fn parameters_verouderd(&self) -> bool {
        self.sleutels.parameters_verouderd()
    }

    /// De namen van alle compartimenten in deze kluis.
    pub fn compartimenten(&self) -> Vec<&str> {
        self.compartimenten.keys().map(|s| s.as_str()).collect()
    }

    /// De namen van de op dit moment ontgrendelde compartimenten.
    pub fn ontgrendelde_compartimenten(&self) -> Vec<&str> {
        self.ontgrendeld.keys().map(|s| s.as_str()).collect()
    }

    /// Maakt een compartiment aan en ontgrendelt het meteen.
    pub fn compartiment_aanmaken(
        &mut self,
        naam: &str,
        omschrijving: &str,
        nu: DateTime<Utc>,
    ) -> Resultaat<()> {
        let (hoofd, sleutel) = self.sleutels.compartiment_aanmaken(naam)?;
        self.conn.execute(
            "INSERT INTO compartiment (naam, hoofd_json, omschrijving, aangemaakt_op)
             VALUES (?1, ?2, ?3, ?4)",
            params![naam, serde_json::to_string(&hoofd)?, omschrijving, nu.to_rfc3339()],
        )?;
        self.compartimenten.insert(naam.to_string(), hoofd);
        self.ontgrendeld.insert(naam.to_string(), sleutel);
        Ok(())
    }

    /// Ontgrendelt een compartiment.
    ///
    /// In deze uitgave ontsluit de kluissleutel alle compartimenten. De
    /// scheiding is dus cryptografisch aanwezig — de gegevens zijn per
    /// compartiment met een eigen sleutel versleuteld — maar de sleutels
    /// hangen aan dezelfde kluissleutel. Een compartiment met een eigen
    /// wachtwoord, zoals het persoonlijke dossier van de functionaris, komt in
    /// een latere fase; de structuur ligt er.
    pub fn compartiment_ontgrendelen(&mut self, naam: &str) -> Resultaat<()> {
        let hoofd = self
            .compartimenten
            .get(naam)
            .ok_or_else(|| StoreFout::OnbekendCompartiment(naam.to_string()))?;
        let sleutel = self.sleutels.compartiment_openen(hoofd)?;
        self.ontgrendeld.insert(naam.to_string(), sleutel);
        Ok(())
    }

    /// Vergrendelt een compartiment: de sleutel verdwijnt uit het geheugen.
    pub fn compartiment_vergrendelen(&mut self, naam: &str) {
        self.ontgrendeld.remove(naam);
    }

    fn sleutel_van(&self, compartiment: &str) -> Resultaat<&Compartimentsleutel> {
        self.ontgrendeld
            .get(compartiment)
            .ok_or_else(|| StoreFout::CompartimentGesloten(compartiment.to_string()))
    }

    /// Bewaart een record.
    ///
    /// Bewaren en loggen gebeuren in één transactie: er bestaat geen toestand
    /// waarin een wijziging is opgeslagen zonder logboekregel.
    #[allow(clippy::too_many_arguments)]
    pub fn bewaar<T: Serialize>(
        &mut self,
        soort: &str,
        id: &str,
        compartiment: &str,
        status: &str,
        kenmerk: Option<&str>,
        record: &T,
        actor: &Actor,
        handeling: Handeling,
        omschrijving: &str,
        nu: DateTime<Utc>,
    ) -> Resultaat<u32> {
        let sleutel = self.sleutel_van(compartiment)?.clone();
        let klaartekst = serde_json::to_vec(record)?;
        let binding = Binding::nieuw(format!("{soort}.inhoud"), id, compartiment);
        let envelop = aead::versleutel(&sleutel, &binding, &klaartekst)?;
        let versleuteld = envelop.naar_bytes();

        let tx = self.conn.transaction()?;

        // Bestaat het record al? Dan de vorige versie wegschrijven.
        let bestaand: Option<(u32, Vec<u8>, String)> = tx
            .query_row(
                "SELECT versie, inhoud, aangemaakt_op FROM record WHERE id = ?1",
                params![id],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .optional()?;

        let (nieuwe_versie, aangemaakt_op) = match bestaand {
            Some((versie, oude_inhoud, aangemaakt)) => {
                tx.execute(
                    "INSERT INTO recordversie (id, versie, compartiment, gewijzigd_op, gewijzigd_door, inhoud)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                    params![id, versie, compartiment, nu.to_rfc3339(), actor.id, oude_inhoud],
                )?;
                (versie + 1, aangemaakt)
            }
            None => (1, nu.to_rfc3339()),
        };

        tx.execute(
            "INSERT INTO record (id, soort, compartiment, status, kenmerk, aangemaakt_op, gewijzigd_op, versie, inhoud)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
             ON CONFLICT(id) DO UPDATE SET
                status = excluded.status,
                kenmerk = excluded.kenmerk,
                gewijzigd_op = excluded.gewijzigd_op,
                versie = excluded.versie,
                inhoud = excluded.inhoud",
            params![
                id,
                soort,
                compartiment,
                status,
                kenmerk,
                aangemaakt_op,
                nu.to_rfc3339(),
                nieuwe_versie,
                versleuteld
            ],
        )?;

        // In dezelfde transactie de logboekregel.
        let gebeurtenis =
            Gebeurtenis::nieuw(handeling, actor.clone(), nu, soort, id, compartiment, omschrijving);
        let (regel, nieuwe_stand) = keten_aan(&self.ketenstand, gebeurtenis)?;
        schrijf_logboekregel(&tx, &regel)?;

        tx.commit()?;
        self.ketenstand = nieuwe_stand;
        Ok(nieuwe_versie)
    }

    /// Leest een record.
    pub fn laad<T: DeserializeOwned>(&self, soort: &str, id: &str) -> Resultaat<T> {
        let (compartiment, versleuteld): (String, Vec<u8>) = self
            .conn
            .query_row(
                "SELECT compartiment, inhoud FROM record WHERE id = ?1 AND soort = ?2",
                params![id, soort],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .optional()?
            .ok_or_else(|| StoreFout::NietGevonden {
                soort: soort.to_string(),
                id: id.to_string(),
            })?;

        let sleutel = self.sleutel_van(&compartiment)?;
        let envelop = aead::Envelop::uit_bytes(&versleuteld)?;
        let binding = Binding::nieuw(format!("{soort}.inhoud"), id, &compartiment);
        let klaartekst = aead::ontsleutel(sleutel, &binding, &envelop)?;
        Ok(serde_json::from_slice(&klaartekst)?)
    }

    /// Leest een eerdere versie van een record.
    pub fn laad_versie<T: DeserializeOwned>(
        &self,
        soort: &str,
        id: &str,
        versie: u32,
    ) -> Resultaat<T> {
        let (compartiment, versleuteld): (String, Vec<u8>) = self
            .conn
            .query_row(
                "SELECT compartiment, inhoud FROM recordversie WHERE id = ?1 AND versie = ?2",
                params![id, versie],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .optional()?
            .ok_or_else(|| StoreFout::NietGevonden {
                soort: format!("{soort} versie {versie}"),
                id: id.to_string(),
            })?;

        let sleutel = self.sleutel_van(&compartiment)?;
        let envelop = aead::Envelop::uit_bytes(&versleuteld)?;
        let binding = Binding::nieuw(format!("{soort}.inhoud"), id, &compartiment);
        let klaartekst = aead::ontsleutel(sleutel, &binding, &envelop)?;
        Ok(serde_json::from_slice(&klaartekst)?)
    }

    /// Alle versies van een record, van oud naar nieuw.
    pub fn versies(&self, id: &str) -> Resultaat<Vec<u32>> {
        let mut stmt =
            self.conn.prepare("SELECT versie FROM recordversie WHERE id = ?1 ORDER BY versie")?;
        let rijen = stmt.query_map(params![id], |r| r.get::<_, u32>(0))?;
        Ok(rijen.collect::<std::result::Result<Vec<_>, _>>()?)
    }

    /// De koppen van alle records van een soort.
    ///
    /// Werkt ook wanneer het compartiment gesloten is: de kop bevat geen
    /// inhoud. Dat is bewust — anders zou een gesloten compartiment betekenen
    /// dat je niet eens ziet dát er dossiers zijn.
    pub fn lijst(&self, soort: &str) -> Resultaat<Vec<Recordkop>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, soort, compartiment, status, kenmerk, aangemaakt_op, gewijzigd_op, versie
             FROM record WHERE soort = ?1 ORDER BY gewijzigd_op DESC",
        )?;
        let rijen = stmt.query_map(params![soort], |r| {
            Ok(Recordkop {
                id: r.get(0)?,
                soort: r.get(1)?,
                compartiment: r.get(2)?,
                status: r.get(3)?,
                kenmerk: r.get(4)?,
                aangemaakt_op: parse_tijd(&r.get::<_, String>(5)?),
                gewijzigd_op: parse_tijd(&r.get::<_, String>(6)?),
                versie: r.get(7)?,
            })
        })?;
        Ok(rijen.collect::<std::result::Result<Vec<_>, _>>()?)
    }

    /// Legt een blinde index vast voor een veld van een record.
    pub fn indexeer(
        &mut self,
        record_id: &str,
        compartiment: &str,
        veld: &str,
        waarde: &str,
    ) -> Resultaat<()> {
        if !dpofg_crypto::blind_index::veld_is_geschikt(veld) {
            return Err(StoreFout::Crypto(dpofg_crypto::CryptoFout::OngeldigFormaat(format!(
                "veld '{veld}' is niet aangemerkt als geschikt voor een blinde index; \
                 een index op een veld met weinig mogelijke waarden verraadt welke records \
                 dezelfde waarde delen"
            ))));
        }
        let sleutel = self.sleutel_van(compartiment)?;
        let index = dpofg_crypto::blind_index::indexeer(sleutel, veld, waarde)?;
        self.conn.execute(
            "INSERT OR IGNORE INTO blinde_index (record_id, veld, waarde) VALUES (?1, ?2, ?3)",
            params![record_id, veld, index],
        )?;
        Ok(())
    }

    /// Zoekt records op een exacte waarde in een geïndexeerd veld.
    ///
    /// De uitkomst is een **voorselectie**: de index is afgekapt, dus er kunnen
    /// records tussen zitten die de waarde niet hebben. De aanroeper ontsleutelt
    /// de kandidaten en vergelijkt exact.
    pub fn zoek_op_index(
        &self,
        compartiment: &str,
        veld: &str,
        waarde: &str,
    ) -> Resultaat<Vec<String>> {
        let sleutel = self.sleutel_van(compartiment)?;
        let index = dpofg_crypto::blind_index::indexeer(sleutel, veld, waarde)?;
        let mut stmt = self
            .conn
            .prepare("SELECT record_id FROM blinde_index WHERE veld = ?1 AND waarde = ?2")?;
        let rijen = stmt.query_map(params![veld, index], |r| r.get::<_, String>(0))?;
        Ok(rijen.collect::<std::result::Result<Vec<_>, _>>()?)
    }

    /// Voegt een regel toe aan het logboek.
    pub fn log(&mut self, gebeurtenis: Gebeurtenis, motivering: Option<String>) -> Resultaat<u64> {
        let gebeurtenis = match motivering {
            Some(m) => gebeurtenis.met_motivering(m),
            None => gebeurtenis,
        };
        let (regel, nieuwe_stand) = keten_aan(&self.ketenstand, gebeurtenis)?;
        schrijf_logboekregel(&self.conn, &regel)?;
        self.ketenstand = nieuwe_stand;
        Ok(self.ketenstand.volgnummer)
    }

    /// De huidige stand van de keten.
    pub fn ketenstand(&self) -> &Ketenstand {
        &self.ketenstand
    }

    /// Leest alle logboekregels.
    pub fn logboek(&self) -> Resultaat<Vec<Ketenregel>> {
        let mut stmt = self.conn.prepare("SELECT regel_json FROM logboek ORDER BY volgnummer")?;
        let rijen = stmt.query_map([], |r| r.get::<_, String>(0))?;
        let mut uit = Vec::new();
        for rij in rijen {
            uit.push(serde_json::from_str(&rij?)?);
        }
        Ok(uit)
    }

    /// Leest de logboekregels die bij één onderwerp horen.
    pub fn logboek_van(&self, soort: &str, id: &str) -> Resultaat<Vec<Ketenregel>> {
        let mut stmt = self.conn.prepare(
            "SELECT regel_json FROM logboek
             WHERE onderwerp_soort = ?1 AND onderwerp_id = ?2 ORDER BY volgnummer",
        )?;
        let rijen = stmt.query_map(params![soort, id], |r| r.get::<_, String>(0))?;
        let mut uit = Vec::new();
        for rij in rijen {
            uit.push(serde_json::from_str(&rij?)?);
        }
        Ok(uit)
    }

    /// Bewaart een anker op de huidige ketenstand.
    pub fn anker_bewaren(&mut self, anker: &dpofg_audit::Anker) -> Resultaat<()> {
        self.conn.execute(
            "INSERT INTO anker (volgnummer, tijdstip, hash, anker_json, bewaarplaats)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                anker.volgnummer,
                anker.tijdstip.to_rfc3339(),
                anker.hash,
                serde_json::to_string(anker)?,
                anker.bewaarplaats
            ],
        )?;
        Ok(())
    }

    /// Het laatst geplaatste anker.
    pub fn laatste_anker(&self) -> Resultaat<Option<dpofg_audit::Anker>> {
        let json: Option<String> = self
            .conn
            .query_row("SELECT anker_json FROM anker ORDER BY volgnummer DESC LIMIT 1", [], |r| {
                r.get(0)
            })
            .optional()?;
        Ok(match json {
            Some(j) => Some(serde_json::from_str(&j)?),
            None => None,
        })
    }

    /// Controleert het logboek, met het laatste anker als ijkpunt.
    pub fn verifieer_logboek(&self) -> Resultaat<dpofg_audit::Verificatierapport> {
        let regels = self.logboek()?;
        let anker = self.laatste_anker()?;
        Ok(dpofg_audit::verifieer(&regels, anker.as_ref())?)
    }

    /// Voegt een bijlage toe, inhoudsgeadresseerd.
    pub fn bijlage_toevoegen(
        &mut self,
        record_id: &str,
        compartiment: &str,
        bestandsnaam: &str,
        inhoud: &[u8],
        actor: &Actor,
        nu: DateTime<Utc>,
    ) -> Resultaat<String> {
        let sleutel = self.sleutel_van(compartiment)?.clone();
        let hash = blake3::hash(inhoud).to_hex().to_string();
        let binding = Binding::nieuw("bijlage.inhoud", &hash, compartiment);
        let envelop = aead::versleutel(&sleutel, &binding, inhoud)?;

        let tx = self.conn.transaction()?;
        tx.execute(
            "INSERT OR IGNORE INTO bijlage (hash, compartiment, omvang, toegevoegd_op, inhoud)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![hash, compartiment, inhoud.len() as i64, nu.to_rfc3339(), envelop.naar_bytes()],
        )?;
        tx.execute(
            "INSERT OR IGNORE INTO bijlagekoppeling
             (record_id, hash, bestandsnaam, toegevoegd_op, toegevoegd_door)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![record_id, hash, bestandsnaam, nu.to_rfc3339(), actor.id],
        )?;

        let gebeurtenis = Gebeurtenis::nieuw(
            Handeling::BijlageToegevoegd,
            actor.clone(),
            nu,
            "bijlage",
            &hash,
            compartiment,
            format!("bijlage '{bestandsnaam}' toegevoegd aan {record_id}"),
        );
        let (regel, nieuwe_stand) = keten_aan(&self.ketenstand, gebeurtenis)?;
        schrijf_logboekregel(&tx, &regel)?;
        tx.commit()?;
        self.ketenstand = nieuwe_stand;
        Ok(hash)
    }

    /// Leest een bijlage en controleert of de inhoud bij de hash past.
    pub fn bijlage_lezen(&self, hash: &str) -> Resultaat<Vec<u8>> {
        let (compartiment, versleuteld): (String, Vec<u8>) = self
            .conn
            .query_row(
                "SELECT compartiment, inhoud FROM bijlage WHERE hash = ?1",
                params![hash],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .optional()?
            .ok_or_else(|| StoreFout::NietGevonden {
                soort: "bijlage".into(),
                id: hash.to_string(),
            })?;

        let sleutel = self.sleutel_van(&compartiment)?;
        let envelop = aead::Envelop::uit_bytes(&versleuteld)?;
        let binding = Binding::nieuw("bijlage.inhoud", hash, &compartiment);
        let inhoud = aead::ontsleutel(sleutel, &binding, &envelop)?;

        let werkelijk = blake3::hash(&inhoud).to_hex().to_string();
        if werkelijk != hash {
            return Err(StoreFout::BlobBeschadigd { hash: hash.to_string() });
        }
        Ok(inhoud)
    }

    /// Wijzigt het wachtwoord. Er wordt niets herversleuteld.
    pub fn wachtwoord_wijzigen(
        &mut self,
        nieuw: &Wachtwoordzin,
        params: KdfParameters,
        actor: &Actor,
        nu: DateTime<Utc>,
    ) -> Resultaat<()> {
        self.sleutels.wachtwoord_wijzigen(nieuw, params)?;
        self.conn.execute(
            "UPDATE kluishoofd SET hoofd_json = ?1 WHERE id = 1",
            params![serde_json::to_string(self.sleutels.hoofd())?],
        )?;
        self.log(
            Gebeurtenis::nieuw(
                Handeling::WachtwoordGewijzigd,
                actor.clone(),
                nu,
                "kluis",
                "1",
                dpofg_crypto::keys::COMPARTIMENT_ALGEMEEN,
                "wachtwoord gewijzigd; geen gegevens herversleuteld",
            ),
            None,
        )?;
        Ok(())
    }

    /// Sluit de kluis.
    pub fn sluiten(mut self, actor: &Actor, nu: DateTime<Utc>) -> Resultaat<()> {
        self.log(
            Gebeurtenis::nieuw(
                Handeling::KluisGesloten,
                actor.clone(),
                nu,
                "kluis",
                "1",
                dpofg_crypto::keys::COMPARTIMENT_ALGEMEEN,
                "kluis gesloten",
            ),
            None,
        )?;
        self.ontgrendeld.clear();
        Ok(())
    }
}

fn schrijf_logboekregel(conn: &Connection, regel: &Ketenregel) -> Resultaat<()> {
    conn.execute(
        "INSERT INTO logboek
         (volgnummer, tijdstip, handeling, actor_id, onderwerp_soort, onderwerp_id,
          compartiment, vorige_hash, hash, regel_json)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params![
            regel.volgnummer,
            regel.gebeurtenis.tijdstip.to_rfc3339(),
            serde_json::to_string(&regel.gebeurtenis.handeling)?.trim_matches('"'),
            regel.gebeurtenis.actor.id,
            regel.gebeurtenis.onderwerp_soort,
            regel.gebeurtenis.onderwerp_id,
            regel.gebeurtenis.compartiment,
            regel.vorige_hash,
            regel.hash,
            serde_json::to_string(regel)?
        ],
    )?;
    Ok(())
}

/// Opent een kluisbestand om er uitsluitend uit te lezen.
///
/// Eerst gewoon alleen-lezen: dan mag SQLite een WAL-index naast het bestand
/// aanleggen, wat op een normale werkmap het snelst en het minst bijzonder is.
/// Lukt dat niet omdat er niets geschreven mag worden, dan wordt het bestand
/// als onveranderlijk geopend. Dat is bewust de terugval en niet de standaard:
/// `immutable=1` is een belofte aan SQLite dat niemand het bestand wijzigt, en
/// die belofte is onwaar zodra er een tweede, schrijvend proces in dezelfde
/// kluis werkt.
fn open_alleen_lezen(pad: &Path) -> Resultaat<Connection> {
    let vlaggen = OpenFlags::SQLITE_OPEN_READ_ONLY;
    match Connection::open_with_flags(pad, vlaggen) {
        Ok(conn) => match conn.query_row("SELECT 1 FROM sqlite_master LIMIT 1", [], |_| Ok(())) {
            Ok(()) | Err(rusqlite::Error::QueryReturnedNoRows) => Ok(conn),
            Err(_) => open_onveranderlijk(pad),
        },
        Err(_) => open_onveranderlijk(pad),
    }
}

fn open_onveranderlijk(pad: &Path) -> Resultaat<Connection> {
    let uri = format!("file:{}?immutable=1", pad.display().to_string().replace('?', "%3f"));
    Ok(Connection::open_with_flags(
        uri,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_URI,
    )?)
}

/// De installatiesleutel zoals die in het ketenlogboek is vastgelegd.
///
/// Beide handelingen die de identiteit vastleggen zetten de publieke sleutel
/// letterlijk in hun omschrijving: `kluis_aangemaakt` bij een nieuwe kluis,
/// `installatiesleutel_aangemaakt` bij een kluis die uit schemaversie 1 is
/// gemigreerd. Die omschrijving zit in de hashketen.
fn sleutel_uit_logboek(conn: &Connection) -> Resultaat<Option<String>> {
    let regel: Option<String> = conn
        .query_row(
            "SELECT regel_json FROM logboek
             WHERE handeling IN ('installatiesleutel_aangemaakt', 'kluis_aangemaakt')
             ORDER BY volgnummer DESC LIMIT 1",
            [],
            |r| r.get(0),
        )
        .optional()?;

    let Some(json) = regel else { return Ok(None) };
    let regel: Ketenregel = serde_json::from_str(&json)?;
    Ok(hex_uit(&regel.gebeurtenis.omschrijving))
}

/// Haalt de eerste reeks van 64 hexadecimale tekens uit een tekst.
fn hex_uit(tekst: &str) -> Option<String> {
    tekst
        .split(|c: char| !c.is_ascii_hexdigit())
        .find(|w| w.len() == 64)
        .map(|w| w.to_ascii_lowercase())
}

/// Schrijft de ondertekenidentiteit weg. Eén rij, altijd id 1.
fn schrijf_installatie(
    conn: &Connection,
    generatie: u32,
    sleutel: &Installatiesleutel,
    envelop: &Envelop,
    nu: DateTime<Utc>,
) -> Resultaat<()> {
    conn.execute(
        "INSERT INTO installatie
             (id, generatie, publieke_sleutel, zaad_envelop, aangemaakt_op, programmaversie)
         VALUES (1, ?1, ?2, ?3, ?4, ?5)",
        params![
            generatie,
            sleutel.publieke_sleutel(),
            envelop.naar_bytes(),
            nu.to_rfc3339(),
            env!("CARGO_PKG_VERSION"),
        ],
    )?;
    Ok(())
}

/// Leest de rij met de ondertekenidentiteit, als die er is.
#[allow(clippy::type_complexity)]
fn lees_installatie(conn: &Connection) -> Resultaat<Option<(u32, String, Envelop, DateTime<Utc>)>> {
    let rij: Option<(u32, String, Vec<u8>, String)> = conn
        .query_row(
            "SELECT generatie, publieke_sleutel, zaad_envelop, aangemaakt_op
             FROM installatie WHERE id = 1",
            [],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
        )
        .optional()?;

    match rij {
        None => Ok(None),
        Some((generatie, publiek, bytes, aangemaakt)) => {
            let envelop = Envelop::uit_bytes(&bytes)?;
            Ok(Some((generatie, publiek, envelop, parse_tijd(&aangemaakt))))
        }
    }
}

fn lees_ketenstand(conn: &Connection) -> Resultaat<Ketenstand> {
    let rij: Option<(u64, String, String)> = conn
        .query_row(
            "SELECT volgnummer, hash, tijdstip FROM logboek ORDER BY volgnummer DESC LIMIT 1",
            [],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .optional()?;
    Ok(match rij {
        None => Ketenstand::leeg(),
        Some((volgnummer, hash, tijdstip)) => {
            Ketenstand { volgnummer, hash, tijdstip: Some(parse_tijd(&tijdstip)) }
        }
    })
}

fn parse_tijd(s: &str) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(s)
        .map(|t| t.with_timezone(&Utc))
        .unwrap_or_else(|_| DateTime::UNIX_EPOCH)
}
