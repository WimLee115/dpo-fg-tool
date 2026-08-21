//! De vormen die over de brug gaan.
//!
//! Eigen typen en geen hergebruik van de domeintypen. Dat is met opzet: een
//! domeintype dat rechtstreeks naar de webview gaat, stuurt alles mee wat er
//! in zit, ook wat het scherm niet nodig heeft en niet hoort te zien. Wat hier
//! staat is wat er in beeld komt, en niets meer.

use chrono::{DateTime, Utc};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Kluisstand {
    pub pad: String,
    pub ontgrendeld: bool,
    pub kennispakket: String,
    pub consolidatiedatum: String,
    pub ketenreikwijdte: String,
    pub keten_in_orde: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct Buitenbeeld {
    pub wat: String,
    pub waar: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Ontbrekend {
    pub veld: String,
    pub omschrijving: String,
    pub grondslag: String,
    pub blokkeert_vaststelling: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct Volledigheid {
    pub soort: String,
    pub verplicht: usize,
    pub compleet: usize,
    pub ontbreekt: Vec<Ontbrekend>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Recordkop {
    pub id: String,
    pub soort: String,
    pub kenmerk: Option<String>,
    pub status: String,
    pub gewijzigd_op: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Veld {
    pub naam: String,
    pub waarde: String,
    /// Of `waarde` een tijdstip in ISO-vorm is.
    ///
    /// De schil zet die om naar de plaatselijke tijd, net als in de werkbak.
    /// Dat gebeurt daar en niet hier, zodat er één plaats is waar een tijdstip
    /// in tekst verandert en de dossiertabel niet een andere notatie krijgt
    /// dan de rest van het scherm.
    pub is_tijdstip: bool,
    pub herkomst: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Dossier {
    pub kop: Recordkop,
    pub volledigheid: Volledigheid,
    pub velden: Vec<Veld>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Bevinding {
    pub regelcode: String,
    pub niveau: String,
    pub ontvanger: String,
    pub record_soort: String,
    pub record_kenmerk: Option<String>,
    pub toelichting: String,
    pub grondslag: String,
    pub afwijking_tot: Option<DateTime<Utc>>,
}

/// De uitkomst van één controleronde.
///
/// Niet alleen de bevindingen: ook wat er *niet* is nagekeken. Een scherm dat
/// enkel een lijst toont, wordt gelezen als "dit is alles" — en juist een
/// termijn die niet te berekenen viel, is iets anders dan een termijn die in
/// orde is.
#[derive(Debug, Clone, Serialize)]
pub struct Controleronde {
    pub peilmoment: DateTime<Utc>,
    pub bevindingen: Vec<Bevinding>,
    /// Hoeveel dossiers er zijn nagekeken.
    pub beoordeeld: usize,
    /// Wat deze ronde niet heeft kunnen beoordelen, met de reden erbij.
    pub niet_beoordeeld: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Vervalpunt {
    pub eis: String,
    pub grondslag: String,
    pub oorzaak: String,
    pub record_soort: String,
    pub record_kenmerk: String,
    pub eigenaar: Option<String>,
    pub vervalt_op: DateTime<Utc>,
}

/// De werkbak, met het moment waarop hij is berekend.
///
/// Het peilmoment gaat mee omdat de band in Rust wordt bepaald en de tekst
/// "nog zoveel uur" in de schil. Zouden die twee elk hun eigen klok lezen, dan
/// kan een regel in de band "verloopt vandaag" staan terwijl de tekst ernaast
/// "te laat" zegt. Eén klok, één antwoord.
#[derive(Debug, Clone, Serialize)]
pub struct Werkvoorraad {
    pub peilmoment: DateTime<Utc>,
    pub regels: Vec<dpofg_report::werkbak::Werkbakregel>,
}
