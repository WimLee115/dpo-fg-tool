//! Opmaak van de uitvoer.
//!
//! Twee regels die overal gelden:
//!
//! 1. **Onvolledigheid is voortgang, geen verwijt.** "11 van de 14 onderdelen"
//!    en niet "3 fouten". Dat is geen kwestie van toon maar van gedrag: een
//!    verwijt nodigt uit het scherm te sluiten, een teller om verder te gaan.
//! 2. **Bij elk oordeel staat de grondslag.** Een melding zonder bepaling is
//!    een mening.

use comfy_table::{presets::UTF8_FULL, Cell, Color, ContentArrangement, Table};

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
    format!(
        "{}{} {} van de {}",
        "█".repeat(gevuld),
        "░".repeat(BREEDTE - gevuld),
        compleet,
        totaal
    )
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
