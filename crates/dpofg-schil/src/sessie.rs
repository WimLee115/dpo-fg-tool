//! De open kluis, en niets anders.

use std::sync::Mutex;

use dpofg_store::Kluis;

/// De kluis die op dit moment openstaat.
///
/// Eén mutex om één optie. Er staat met opzet geen tweede veld in: alles wat
/// een scherm nodig heeft wordt bij elke aanroep opnieuw uit de kluis gelezen.
/// Een tweede plaats waar dezelfde gegevens staan, loopt uit de pas met de
/// eerste, en dan is de vraag welke van de twee gold.
#[derive(Default)]
pub struct Sessie {
    kluis: Mutex<Option<Kluis>>,
}

impl Sessie {
    pub fn nieuw() -> Self {
        Self::default()
    }

    /// Voert iets uit op de open kluis.
    ///
    /// Geeft een leesbare fout wanneer de kluis dicht is; dat is een gewone
    /// toestand en geen storing.
    pub fn met_kluis<T>(
        &self,
        werk: impl FnOnce(&mut Kluis) -> Result<T, String>,
    ) -> Result<T, String> {
        let mut slot = self.kluis.lock().map_err(|_| {
            "de sessie is in een onbruikbare toestand geraakt; sluit de schil en open hem \
             opnieuw"
                .to_string()
        })?;
        let kluis = slot.as_mut().ok_or_else(|| "de kluis is niet geopend".to_string())?;
        werk(kluis)
    }

    pub fn zet(&self, kluis: Kluis) -> Result<(), String> {
        let mut slot = self.kluis.lock().map_err(|_| "de sessie is onbruikbaar".to_string())?;
        *slot = Some(kluis);
        Ok(())
    }

    pub fn sluit(&self) -> Result<(), String> {
        let mut slot = self.kluis.lock().map_err(|_| "de sessie is onbruikbaar".to_string())?;
        *slot = None;
        Ok(())
    }

    pub fn is_open(&self) -> bool {
        self.kluis.lock().map(|s| s.is_some()).unwrap_or(false)
    }
}
