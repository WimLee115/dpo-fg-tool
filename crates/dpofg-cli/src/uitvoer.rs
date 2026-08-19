//! Opmaak van de uitvoer.
//!
//! Twee regels die overal gelden:
//!
//! 1. **Onvolledigheid is voortgang, geen verwijt.** "11 van de 14 onderdelen"
//!    en niet "3 fouten". Dat is geen kwestie van toon maar van gedrag: een
//!    verwijt nodigt uit het scherm te sluiten, een teller om verder te gaan.
//! 2. **Bij elk oordeel staat de grondslag.** Een melding zonder bepaling is
//!    een mening.

use chrono::{DateTime, NaiveDate, Utc};
use comfy_table::{presets::UTF8_FULL, Cell, Color, ContentArrangement, Table};

/// De tijdzone waarin dit product zijn tijdstippen toont.
///
/// Dezelfde zone waarin de termijnen worden gerekend. Een deadline die om
/// 11:20 Nederlandse tijd verstrijkt en als "09:20 UTC" op het scherm komt,
/// wordt om tien uur 's ochtends gelezen als verstreken. De vermelding "UTC"
/// staat er dan wel bij, maar wie onder tijdsdruk een meldtermijn naslaat,
/// leest cijfers en geen zone-aanduiding.
const ZONE: chrono_tz::Tz = chrono_tz::Europe::Amsterdam;

/// Iets wat een kalenderdag aanwijst.
///
/// Twee soorten, en het verschil doet ertoe. Een `DateTime<Utc>` is een
/// moment: die moet worden omgerekend, want 22:50 UTC op 19 augustus is in
/// Nederland al 20 augustus. Een `NaiveDate` is al een dag zoals hij is
/// vastgelegd — daar valt niets om te rekenen, en het zou verkeerd zijn het
/// toch te doen.
pub trait Kalenderdag {
    fn als_datum(&self) -> String;
}

impl Kalenderdag for DateTime<Utc> {
    fn als_datum(&self) -> String {
        self.with_timezone(&ZONE).format("%d-%m-%Y").to_string()
    }
}

impl Kalenderdag for NaiveDate {
    fn als_datum(&self) -> String {
        self.format("%d-%m-%Y").to_string()
    }
}

/// Een datum in Nederlandse notatie.
pub fn datum(d: impl Kalenderdag) -> String {
    d.als_datum()
}

/// Datum en tijd in Nederlandse tijd, met de zone erbij.
///
/// De zone-aanduiding blijft staan omdat een auditspoor over de zomertijdgrens
/// heen anders niet te lezen is: twee records met "02:30" zijn zonder CEST of
/// CET niet te ordenen.
pub fn tijdstip(t: DateTime<Utc>) -> String {
    t.with_timezone(&ZONE).format("%d-%m-%Y %H:%M %Z").to_string()
}

/// Dag en tijd zonder jaartal, voor een kolom waar het jaar al vaststaat.
pub fn dag_en_tijd(t: DateTime<Utc>) -> String {
    t.with_timezone(&ZONE).format("%d-%m %H:%M").to_string()
}

/// Een tabel met de vaste opmaak van dit product.
pub fn tabel(koppen: &[&str]) -> Table {
    let mut t = Table::new();
    t.load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(koppen.iter().map(|k| Cell::new(k).fg(Color::Cyan)));
    t
}

/// Een kop boven een blok uitvoer.
pub fn kop(tekst: &str) {
    println!("\n\x1b[1m{tekst}\x1b[0m");
    println!("{}", "─".repeat(tekst.chars().count().min(78)));
}

/// Een regel die aandacht vraagt zonder te schreeuwen.
pub fn let_op(tekst: &str) {
    println!("\x1b[33m▸\x1b[0m {tekst}");
}

/// Een regel die een blokkade meldt.
pub fn blokkade(tekst: &str) {
    println!("\x1b[31m■\x1b[0m {tekst}");
}

/// Een regel die iets bevestigt.
pub fn gelukt(tekst: &str) {
    println!("\x1b[32m✓\x1b[0m {tekst}");
}

/// Een terzijde: toelichting die niet in de weg zit.
pub fn terzijde(tekst: &str) {
    println!("\x1b[2m  {tekst}\x1b[0m");
}

/// Een voortgangsbalk voor de volledigheidsteller.
///
/// Bewust een balk en geen percentage alleen: een balk laat zien dat er
/// vooruitgang mogelijk is, een percentage leest als een cijfer.
pub fn voortgang(compleet: usize, totaal: usize) -> String {
    const BREEDTE: usize = 16;
    // Klemmen op de breedte: een teller die door een fout elders boven het
    // totaal uitkomt, mag de weergave niet laten crashen. Een balk die
    // volloopt is een zichtbaar signaal; een paniek is dat niet.
    let gevuld = match totaal {
        0 => BREEDTE,
        t => (compleet.min(t) * BREEDTE) / t,
    };
    format!("{}{} {} van de {}", "█".repeat(gevuld), "░".repeat(BREEDTE - gevuld), compleet, totaal)
}

/// Een tijdsduur in gewone taal.
///
/// Afronden naar hele uren maakt twintig minuten tot "0 uur", en dat is
/// misleidend precies waar precisie telt: bij de verificatieperiode en de
/// registratievertraging.
pub fn duur(d: chrono::Duration) -> String {
    let minuten = d.num_minutes().abs();
    match minuten {
        0 => "minder dan een minuut".to_string(),
        1 => "1 minuut".to_string(),
        2..=59 => format!("{minuten} minuten"),
        60 => "1 uur".to_string(),
        61..=1439 => {
            let uren = minuten / 60;
            let rest = minuten % 60;
            if rest == 0 {
                format!("{uren} uur")
            } else {
                format!("{uren} uur en {rest} minuten")
            }
        }
        _ => {
            let dagen = minuten / 1440;
            let uren = (minuten % 1440) / 60;
            if uren == 0 {
                format!("{dagen} dagen")
            } else {
                format!("{dagen} dagen en {uren} uur")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn duurweergave_rondt_niet_weg() {
        assert_eq!(duur(Duration::minutes(20)), "20 minuten");
        assert_eq!(duur(Duration::minutes(1)), "1 minuut");
        assert_eq!(duur(Duration::seconds(30)), "minder dan een minuut");
        assert_eq!(duur(Duration::hours(1)), "1 uur");
        assert_eq!(duur(Duration::minutes(90)), "1 uur en 30 minuten");
        assert_eq!(duur(Duration::hours(11)), "11 uur");
        assert_eq!(duur(Duration::hours(30)), "1 dagen en 6 uur");
        assert_eq!(duur(Duration::days(3)), "3 dagen");
    }

    #[test]
    fn voortgang_toont_de_verhouding() {
        assert!(voortgang(0, 8).starts_with("░"));
        assert!(voortgang(8, 8).starts_with("█"));
        assert!(voortgang(4, 8).contains("4 van de 8"));
        // Nul verplichte onderdelen mag niet delen door nul.
        assert!(voortgang(0, 0).contains("0 van de 0"));
        // En een teller die door een fout elders te hoog uitvalt, mag de
        // weergave niet laten crashen.
        assert!(voortgang(99, 8).contains("99 van de 8"));
    }
}

#[cfg(test)]
mod tijdtests {
    use super::*;
    use chrono::TimeZone;

    fn moment(s: &str) -> DateTime<Utc> {
        s.parse::<DateTime<Utc>>().expect("het proefmoment moet leesbaar zijn")
    }

    // Dit is het geval waar het om begonnen was. Een record dat om 22:50 UTC
    // wordt vastgelegd, staat in Nederland op de volgende dag. Wie de
    // opdrachtregel en de schil naast elkaar legt, moet daar dezelfde datum
    // zien staan.
    #[test]
    fn een_moment_laat_op_de_avond_valt_in_nederland_op_de_volgende_dag() {
        assert_eq!(datum(moment("2026-08-19T22:50:00Z")), "20-08-2026");
        assert_eq!(tijdstip(moment("2026-08-19T22:50:00Z")), "20-08-2026 00:50 CEST");
    }

    #[test]
    fn in_de_winter_staat_er_cet_en_in_de_zomer_cest() {
        assert_eq!(tijdstip(moment("2026-01-15T09:00:00Z")), "15-01-2026 10:00 CET");
        assert_eq!(tijdstip(moment("2026-07-15T09:00:00Z")), "15-07-2026 11:00 CEST");
    }

    // De meldtermijn uit het proefdossier: 72 uur na kennisname om 09:20 UTC.
    // Op het scherm hoort daar de Nederlandse kloktijd te staan, want dat is
    // ook de klok waarop de termijn is gerekend.
    #[test]
    fn een_meldtermijn_staat_op_de_nederlandse_klok() {
        assert_eq!(tijdstip(moment("2026-08-21T09:20:00Z")), "21-08-2026 11:20 CEST");
    }

    // Een vastgelegde kalenderdag is geen moment en wordt niet omgerekend.
    // Zou dat wel gebeuren, dan zou een consolidatiedatum van een kennispakket
    // per tijdzone kunnen verschuiven.
    #[test]
    fn een_kalenderdag_wordt_niet_omgerekend() {
        let dag = NaiveDate::from_ymd_opt(2026, 8, 19).expect("een geldige dag");
        assert_eq!(datum(dag), "19-08-2026");
    }

    #[test]
    fn dag_en_tijd_laat_het_jaar_weg_maar_rekent_wel_om() {
        assert_eq!(dag_en_tijd(moment("2026-08-19T22:50:00Z")), "20-08 00:50");
    }

    // Op de nacht dat de klok verspringt staan er twee keer 02:30. Zonder de
    // zone-aanduiding zijn die twee records niet te ordenen, en een auditspoor
    // dat niet te ordenen is, bewijst niets.
    #[test]
    fn de_uren_op_de_omschakelnacht_blijven_te_onderscheiden() {
        let voor = Utc.with_ymd_and_hms(2026, 10, 25, 0, 30, 0).single().expect("een geldig uur");
        let na = Utc.with_ymd_and_hms(2026, 10, 25, 1, 30, 0).single().expect("een geldig uur");
        assert_eq!(tijdstip(voor), "25-10-2026 02:30 CEST");
        assert_eq!(tijdstip(na), "25-10-2026 02:30 CET");
        assert_ne!(tijdstip(voor), tijdstip(na));
    }
}
