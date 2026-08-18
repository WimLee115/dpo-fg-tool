//! Het wachtwoord ophalen.
//!
//! # Waarom het nooit een argument is
//!
//! Een wachtwoord op de opdrachtregel belandt in de procesLijst, in de
//! geschiedenis van de schil en in elk logbestand dat opdrachten meeschrijft.
//! Daarom is er geen vlag `--wachtwoord`, en die komt er ook niet.
//!
//! Er zijn twee routes: interactief vragen, of een omgevingsvariabele voor
//! geautomatiseerd gebruik. De tweede route waarschuwt over zichzelf, want ook
//! een omgevingsvariabele is niet zonder risico — hij is leesbaar voor elk
//! proces van dezelfde gebruiker en belandt in een procesdump.

use anyhow::{bail, Result};
use dpofg_crypto::{kdf::beoordeel_wachtwoord, Wachtwoordzin};

/// Naam van de omgevingsvariabele voor geautomatiseerd gebruik.
pub const OMGEVINGSVARIABELE: &str = "DPOFG_WACHTWOORD";

/// Haalt het wachtwoord op.
pub fn vraag(prompt: &str) -> Result<Wachtwoordzin> {
    if let Ok(uit_omgeving) = std::env::var(OMGEVINGSVARIABELE) {
        eprintln!(
            "\x1b[33m▸\x1b[0m Het wachtwoord komt uit {OMGEVINGSVARIABELE}. \
             Dat is bruikbaar voor geautomatiseerd gebruik, maar leesbaar voor elk proces van \
             dezelfde gebruiker. Gebruik het niet op een werkplek."
        );
        return Ok(Wachtwoordzin::nieuw(uit_omgeving));
    }
    let tekst = rpassword::prompt_password(format!("{prompt}: "))?;
    if tekst.is_empty() {
        bail!("geen wachtwoord ingevoerd");
    }
    Ok(Wachtwoordzin::nieuw(tekst))
}

/// Vraagt een nieuw wachtwoord, twee keer, en beoordeelt de sterkte.
pub fn vraag_nieuw() -> Result<Wachtwoordzin> {
    if let Ok(uit_omgeving) = std::env::var(OMGEVINGSVARIABELE) {
        return Ok(Wachtwoordzin::nieuw(uit_omgeving));
    }

    println!(
        "Kies een wachtwoordzin. Lengte telt zwaarder dan leestekens: vier of meer\n\
         willekeurige woorden zijn sterker en beter te onthouden dan 'Welkom2026!'."
    );
    println!(
        "\n\x1b[33m▸\x1b[0m Er is geen herstelmogelijkheid. Wie deze zin kwijtraakt,\n\
         \x1b[33m \x1b[0m raakt de kluis kwijt — er bestaat geen achterdeur en die komt er niet."
    );

    let eerste = rpassword::prompt_password("\nWachtwoordzin: ")?;
    let tweede = rpassword::prompt_password("Nogmaals ter controle: ")?;
    if eerste != tweede {
        bail!("de twee invoeren komen niet overeen");
    }

    let zin = Wachtwoordzin::nieuw(eerste);
    let sterkte = beoordeel_wachtwoord(&zin);
    use dpofg_crypto::kdf::Wachtwoordsterkte::*;
    match sterkte {
        Onbruikbaar => bail!(
            "deze wachtwoordzin is {}; kies er een van minstens twaalf tekens",
            sterkte.toelichting()
        ),
        Zwak => println!("\x1b[33m▸\x1b[0m Deze zin is {}.", sterkte.toelichting()),
        Redelijk => println!("\x1b[32m✓\x1b[0m {}.", sterkte.toelichting()),
        Sterk => println!("\x1b[32m✓\x1b[0m {}.", sterkte.toelichting()),
    }
    Ok(zin)
}
