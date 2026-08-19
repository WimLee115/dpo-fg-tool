//! Het databaseschema en de migraties.
//!
//! # Waarom migraties genummerd en onomkeerbaar vooruit zijn
//!
//! Een kluis die met een nieuwere uitgave is aangemaakt, wordt **geweigerd**
//! in plaats van geopend. De verleiding is groot om "wel openen, onbekende
//! kolommen negeren" te doen, maar dat is precies hoe gegevens stilzwijgend
//! verdwijnen: de oude uitgave schrijft het record terug zonder de velden die
//! zij niet kent.
//!
//! # Wat er onversleuteld in de tabellen staat
//!
//! Eerlijk benoemen wat het lekt, want dat is een ontwerpbeslissing en geen
//! detail. Onversleuteld zijn: de identificatie van een record, de soort, het
//! compartiment, de status en de tijdstippen van aanmaken en wijzigen. Wie het
//! bestand in handen krijgt zonder wachtwoord, ziet daarmee **hoeveel**
//! dossiers er van elke soort zijn en **wanneer** eraan is gewerkt — niet wat
//! erin staat.
//!
//! Dat is bewust: zonder die kolommen is geen enkele lijst te tonen zonder
//! eerst de hele kluis te ontsleutelen, en dat maakt de toepassing onbruikbaar
//! bij duizenden records. De inhoud, inclusief elk persoonsgegeven, staat
//! uitsluitend in de versleutelde kolom.

use rusqlite::Connection;

use crate::Resultaat;

/// De schemaversie die deze uitgave schrijft en leest.
pub const SCHEMAVERSIE: u32 = 2;

/// Legt het schema aan of werkt het bij.
///
/// Alle migraties draaien met de ophoging van `user_version` in **één**
/// transactie. Zonder dat zou een afbreking halverwege — stroomuitval, een
/// afgeschoten proces — een kluis achterlaten met een deel van de tabellen en
/// `user_version` op de oude waarde: bij de volgende poging draait de migratie
/// dan opnieuw en loopt hij stuk op een tabel die al bestaat. `user_version` is
/// in SQLite onderdeel van de transactie, dus dit werkt zoals het eruitziet.
pub fn migreer(conn: &Connection) -> Resultaat<u32> {
    let huidig: u32 = conn.pragma_query_value(None, "user_version", |r| r.get(0))?;

    if huidig > SCHEMAVERSIE {
        return Err(crate::StoreFout::KluisIsNieuwer {
            in_kluis: huidig,
            ondersteund: SCHEMAVERSIE,
        });
    }
    if huidig == SCHEMAVERSIE {
        return Ok(huidig);
    }

    conn.execute_batch("BEGIN IMMEDIATE")?;
    let uitkomst = (|| -> Resultaat<()> {
        if huidig < 1 {
            migratie_1(conn)?;
        }
        if huidig < 2 {
            migratie_2(conn)?;
        }
        conn.pragma_update(None, "user_version", SCHEMAVERSIE)?;
        Ok(())
    })();

    match uitkomst {
        Ok(()) => {
            conn.execute_batch("COMMIT")?;
            Ok(SCHEMAVERSIE)
        }
        Err(fout) => {
            // De terugdraaiing mag de oorspronkelijke fout niet verdringen: die
            // zegt wát er misging, en dat is wat de gebruiker moet lezen.
            let _ = conn.execute_batch("ROLLBACK");
            Err(fout)
        }
    }
}

/// Migratie 1: de eerste indeling.
fn migratie_1(conn: &Connection) -> Resultaat<()> {
    conn.execute_batch(
        r#"
        -- ------------------------------------------------------------------
        -- Kluishoofd: alles wat nodig is om de kluis te openen.
        -- Bevat geen sleutelmateriaal, alleen het zout, de
        -- afleidingsparameters en de gewikkelde kluissleutel.
        -- ------------------------------------------------------------------
        CREATE TABLE kluishoofd (
            id            INTEGER PRIMARY KEY CHECK (id = 1),
            hoofd_json    TEXT    NOT NULL,
            aangemaakt_op TEXT    NOT NULL,
            programmaversie TEXT  NOT NULL
        ) STRICT;

        -- ------------------------------------------------------------------
        -- Compartimenten: per compartiment de met de kluissleutel gewikkelde
        -- compartimentsleutel.
        -- ------------------------------------------------------------------
        CREATE TABLE compartiment (
            naam          TEXT PRIMARY KEY,
            hoofd_json    TEXT NOT NULL,
            omschrijving  TEXT NOT NULL,
            aangemaakt_op TEXT NOT NULL
        ) STRICT;

        -- ------------------------------------------------------------------
        -- Records. De inhoud staat versleuteld in `inhoud`; de kolommen
        -- daarnaast zijn wat er nodig is om te kunnen sorteren en filteren
        -- zonder de hele kluis te ontsleutelen.
        -- ------------------------------------------------------------------
        CREATE TABLE record (
            id            TEXT PRIMARY KEY,
            soort         TEXT NOT NULL,
            compartiment  TEXT NOT NULL REFERENCES compartiment(naam),
            status        TEXT NOT NULL,
            kenmerk       TEXT,
            aangemaakt_op TEXT NOT NULL,
            gewijzigd_op  TEXT NOT NULL,
            versie        INTEGER NOT NULL DEFAULT 1,
            inhoud        BLOB NOT NULL
        ) STRICT;

        CREATE INDEX idx_record_soort  ON record(soort, status);
        CREATE INDEX idx_record_comp   ON record(compartiment);
        CREATE INDEX idx_record_gewijzigd ON record(gewijzigd_op);

        -- ------------------------------------------------------------------
        -- Versiegeschiedenis. Niets wordt hard overschreven: elke vorige
        -- versie blijft staan, zodat ongedaan maken altijd kan en de
        -- diff-weergave uit het foutbestendigheidsontwerp mogelijk is.
        -- ------------------------------------------------------------------
        CREATE TABLE recordversie (
            id            TEXT NOT NULL,
            versie        INTEGER NOT NULL,
            compartiment  TEXT NOT NULL,
            gewijzigd_op  TEXT NOT NULL,
            gewijzigd_door TEXT NOT NULL,
            inhoud        BLOB NOT NULL,
            PRIMARY KEY (id, versie)
        ) STRICT;

        -- ------------------------------------------------------------------
        -- Blinde index voor zoeken in versleutelde velden.
        -- ------------------------------------------------------------------
        CREATE TABLE blinde_index (
            record_id     TEXT NOT NULL REFERENCES record(id) ON DELETE CASCADE,
            veld          TEXT NOT NULL,
            waarde        TEXT NOT NULL,
            PRIMARY KEY (record_id, veld, waarde)
        ) STRICT;

        CREATE INDEX idx_blinde_index_zoek ON blinde_index(veld, waarde);

        -- ------------------------------------------------------------------
        -- Het ketenlogboek. Append-only: er is geen enkele plaats in de code
        -- die hieruit verwijdert of bijwerkt, en de trigger hieronder maakt
        -- dat ook op databaseniveau onmogelijk.
        -- ------------------------------------------------------------------
        CREATE TABLE logboek (
            volgnummer    INTEGER PRIMARY KEY,
            tijdstip      TEXT NOT NULL,
            handeling     TEXT NOT NULL,
            actor_id      TEXT NOT NULL,
            onderwerp_soort TEXT NOT NULL,
            onderwerp_id  TEXT NOT NULL,
            compartiment  TEXT NOT NULL,
            vorige_hash   TEXT NOT NULL,
            hash          TEXT NOT NULL,
            regel_json    TEXT NOT NULL
        ) STRICT;

        CREATE INDEX idx_logboek_onderwerp ON logboek(onderwerp_soort, onderwerp_id);
        CREATE INDEX idx_logboek_tijd      ON logboek(tijdstip);

        CREATE TRIGGER logboek_geen_wijziging
        BEFORE UPDATE ON logboek
        BEGIN
            SELECT RAISE(ABORT, 'het logboek is append-only; regels kunnen niet worden gewijzigd');
        END;

        CREATE TRIGGER logboek_geen_verwijdering
        BEFORE DELETE ON logboek
        BEGIN
            SELECT RAISE(ABORT, 'het logboek is append-only; regels kunnen niet worden verwijderd');
        END;

        -- ------------------------------------------------------------------
        -- Ankers op het logboek.
        -- ------------------------------------------------------------------
        CREATE TABLE anker (
            volgnummer    INTEGER PRIMARY KEY,
            tijdstip      TEXT NOT NULL,
            hash          TEXT NOT NULL,
            anker_json    TEXT NOT NULL,
            bewaarplaats  TEXT
        ) STRICT;

        -- ------------------------------------------------------------------
        -- Bijlagen, inhoudsgeadresseerd opgeslagen. Dezelfde bijlage die
        -- tweemaal wordt toegevoegd, bestaat eenmaal; de hash is de sleutel.
        -- Dat lost hoofdlettergevoeligheid, padlengte en verboden tekens in
        -- bestandsnamen in een keer op.
        -- ------------------------------------------------------------------
        CREATE TABLE bijlage (
            hash          TEXT PRIMARY KEY,
            compartiment  TEXT NOT NULL REFERENCES compartiment(naam),
            omvang        INTEGER NOT NULL,
            toegevoegd_op TEXT NOT NULL,
            inhoud        BLOB NOT NULL
        ) STRICT;

        CREATE TABLE bijlagekoppeling (
            record_id     TEXT NOT NULL REFERENCES record(id) ON DELETE CASCADE,
            hash          TEXT NOT NULL REFERENCES bijlage(hash),
            bestandsnaam  TEXT NOT NULL,
            toegevoegd_op TEXT NOT NULL,
            toegevoegd_door TEXT NOT NULL,
            PRIMARY KEY (record_id, hash)
        ) STRICT;

        -- ------------------------------------------------------------------
        -- Kennispakketten: welke versie van de juridische inhoud actief is.
        -- ------------------------------------------------------------------
        CREATE TABLE kennispakket (
            code          TEXT PRIMARY KEY,
            versie        TEXT NOT NULL,
            consolidatiedatum TEXT NOT NULL,
            uitgever_sleutel TEXT NOT NULL,
            geinstalleerd_op TEXT NOT NULL,
            inhoud_json   TEXT NOT NULL
        ) STRICT;
        "#,
    )?;
    Ok(())
}

/// Migratie 2: de ondertekenidentiteit van de installatie.
///
/// Zonder eigen transactie: [`migreer`] zet er al één omheen.
fn migratie_2(conn: &Connection) -> Resultaat<()> {
    conn.execute_batch(
        r#"
        -- ------------------------------------------------------------------
        -- De vaste ondertekenidentiteit van deze installatie.
        --
        -- `publieke_sleutel` staat in klare tekst. Dat is geen omissie: de
        -- publieke helft ís publiek, en hij moet voor te lezen zijn zonder het
        -- wachtwoord — anders wordt het intypen van een wachtwoordzin voor het
        -- opvragen van publiek materiaal een gewoonte. Precedent in dit schema:
        -- `kennispakket.uitgever_sleutel`.
        --
        -- `zaad_envelop` bevat het ondertekenzaad, uitsluitend gewikkeld onder
        -- de kluissleutel. Daardoor overleeft de identiteit een
        -- wachtwoordwijziging: die vervangt alleen de wikkeling van de
        -- kluissleutel zelf.
        --
        -- Er staat één rij in, altijd. Twee identiteiten in één kluis zouden
        -- betekenen dat twee ankers van dezelfde kluis een andere ondertekenaar
        -- dragen, en dan is de vergelijking bij de ontvanger waardeloos.
        -- ------------------------------------------------------------------
        CREATE TABLE IF NOT EXISTS installatie (
            id               INTEGER PRIMARY KEY CHECK (id = 1),
            generatie        INTEGER NOT NULL,
            publieke_sleutel TEXT    NOT NULL,
            zaad_envelop     BLOB    NOT NULL,
            aangemaakt_op    TEXT    NOT NULL,
            programmaversie  TEXT    NOT NULL
        ) STRICT;
        "#,
    )?;
    Ok(())
}

/// De vaste instellingen waarmee elke verbinding wordt geopend.
pub fn stel_verbinding_in(conn: &Connection) -> Resultaat<()> {
    // Write-ahead logging: sneller en beter bestand tegen een onverwachte
    // afsluiting. Wel: een kluis bestaat dan uit drie bestanden, wat gevolgen
    // heeft voor back-ups. Dat staat in de handleiding.
    conn.pragma_update(None, "journal_mode", "WAL")?;
    // Volledige synchronisatie: bij een stroomstoring mag geen bevestigde
    // wijziging verloren gaan. Voor een dossier met wettelijke termijnen weegt
    // dat zwaarder dan schrijfsnelheid.
    conn.pragma_update(None, "synchronous", "FULL")?;
    conn.pragma_update(None, "foreign_keys", "ON")?;
    // Tijdelijke tabellen in het geheugen: geen onversleutelde resten op schijf.
    conn.pragma_update(None, "temp_store", "MEMORY")?;
    // Beperk hoe lang op een vergrendeling wordt gewacht voordat er een
    // duidelijke fout komt in plaats van een vastloper.
    conn.busy_timeout(std::time::Duration::from_secs(5))?;
    Ok(())
}
