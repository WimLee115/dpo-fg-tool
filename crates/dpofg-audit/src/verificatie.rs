//! Verificatie van het logboek.
//!
//! Het rapport verzamelt **alle** bevindingen en stopt niet bij de eerste. Wie
//! een logboek moet beoordelen, wil weten hoeveel er mis is en waar, niet
//! alleen dat er iets mis is. Bovendien vermeldt het rapport altijd tot welk
//! anker de vaststelling reikt — zonder die zin is een groen vinkje misleidend.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{Anker, AuditFout, Ketenregel, Resultaat, GENESIS};

/// Uitkomst van een verificatie.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Verificatierapport {
    /// Aantal gecontroleerde regels.
    pub regels: u64,
    /// Volgnummer van de eerste regel.
    pub eerste_volgnummer: Option<u64>,
    /// Volgnummer van de laatste regel.
    pub laatste_volgnummer: Option<u64>,
    /// Hash van de laatste regel.
    pub laatste_hash: Option<String>,
    /// Tijdstip van de eerste en laatste regel.
    pub periode: Option<(DateTime<Utc>, DateTime<Utc>)>,
    /// Alle aangetroffen bevindingen.
    pub bevindingen: Vec<Bevinding>,
    /// Het anker waartegen is gecontroleerd, als dat is meegegeven.
    pub ankerstatus: Ankerstatus,
}

/// Eén aangetroffen probleem, met de plaats waar het zit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Bevinding {
    pub volgnummer: u64,
    pub omschrijving: String,
    pub soort: Bevindingsoort,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Bevindingsoort {
    /// De keten is verbroken.
    Ketenbreuk,
    /// Een regel ontbreekt.
    OntbrekendeRegel,
    /// Een volgnummer komt dubbel voor.
    DubbeleRegel,
    /// De inhoud van een regel is gewijzigd.
    InhoudGewijzigd,
    /// Het tijdstip loopt terug.
    TijdLooptTerug,
}

/// Hoe de keten zich verhoudt tot het meegegeven anker.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Ankerstatus {
    /// Er is geen anker meegegeven; afkappen aan het eind is niet uit te sluiten.
    GeenAnker,
    /// De keten komt overeen met het anker en is er sindsdien op doorgegaan.
    Bevestigd { volgnummer: u64, regels_sinds_anker: u64 },
    /// De keten is korter dan het anker: er zijn regels verdwenen.
    KetenIsIngekort { anker_volgnummer: u64, keten_volgnummer: u64 },
    /// De hash op de ankerpositie wijkt af: er is vóór het anker iets gewijzigd.
    HashWijktAf { volgnummer: u64, in_anker: String, in_keten: String },
    /// De handtekening onder het anker deugt niet.
    AnkerOngeldig(String),
}

impl Verificatierapport {
    /// Geeft aan of er geen enkele bevinding is.
    pub fn is_ongeschonden(&self) -> bool {
        self.bevindingen.is_empty()
            && !matches!(
                self.ankerstatus,
                Ankerstatus::KetenIsIngekort { .. }
                    | Ankerstatus::HashWijktAf { .. }
                    | Ankerstatus::AnkerOngeldig(_)
            )
    }

    /// De zin die onder elk uitgevoerd verificatierapport hoort te staan.
    ///
    /// Deze tekst is bewust vast: hij mag niet per rapport worden afgezwakt.
    pub fn reikwijdte(&self) -> String {
        match &self.ankerstatus {
            Ankerstatus::GeenAnker => format!(
                "De keten van {} regels is intern samenhangend. Er is geen anker meegegeven; \
                 daarmee is niet vast te stellen of er aan het einde regels zijn verwijderd.",
                self.regels
            ),
            Ankerstatus::Bevestigd { volgnummer, regels_sinds_anker } => format!(
                "De keten is bevestigd tot en met regel {volgnummer} door een geldig anker. \
                 De {regels_sinds_anker} regels daarna rusten uitsluitend op de keten zelf; \
                 verwijdering daarvan is niet uit te sluiten.",
            ),
            Ankerstatus::KetenIsIngekort { anker_volgnummer, keten_volgnummer } => format!(
                "Het anker verklaart regel {anker_volgnummer}, maar de keten eindigt bij \
                 {keten_volgnummer}. Er zijn regels verwijderd."
            ),
            Ankerstatus::HashWijktAf { volgnummer, .. } => format!(
                "Op de ankerpositie {volgnummer} wijkt de hash af van wat het anker verklaart. \
                 De inhoud is na het ankeren gewijzigd."
            ),
            Ankerstatus::AnkerOngeldig(reden) => {
                format!("Het anker is niet bruikbaar: {reden}. De keten is niet extern bevestigd.")
            }
        }
    }
}

/// Controleert een reeks ketenregels, eventueel tegen een anker.
///
/// De regels worden verwacht in oplopende volgorde zoals ze zijn opgeslagen.
pub fn verifieer(regels: &[Ketenregel], anker: Option<&Anker>) -> Resultaat<Verificatierapport> {
    let mut bevindingen = Vec::new();

    if regels.is_empty() {
        return Ok(Verificatierapport {
            regels: 0,
            eerste_volgnummer: None,
            laatste_volgnummer: None,
            laatste_hash: None,
            periode: None,
            bevindingen,
            ankerstatus: match anker {
                None => Ankerstatus::GeenAnker,
                Some(a) => Ankerstatus::KetenIsIngekort {
                    anker_volgnummer: a.volgnummer,
                    keten_volgnummer: 0,
                },
            },
        });
    }

    let mut vorige_hash = GENESIS.to_string();
    let mut vorig_volgnummer: Option<u64> = None;
    let mut vorig_tijdstip: Option<DateTime<Utc>> = None;

    for regel in regels {
        // 1. Volgnummers moeten zonder gaten oplopen.
        if let Some(vorig) = vorig_volgnummer {
            if regel.volgnummer == vorig {
                bevindingen.push(Bevinding {
                    volgnummer: regel.volgnummer,
                    omschrijving: AuditFout::DubbelVolgnummer(regel.volgnummer).to_string(),
                    soort: Bevindingsoort::DubbeleRegel,
                });
            } else if regel.volgnummer != vorig + 1 {
                bevindingen.push(Bevinding {
                    volgnummer: vorig + 1,
                    omschrijving: AuditFout::OntbrekendVolgnummer {
                        verwacht: vorig + 1,
                        gevonden: regel.volgnummer,
                    }
                    .to_string(),
                    soort: Bevindingsoort::OntbrekendeRegel,
                });
            }
        } else if regel.volgnummer != 1 {
            bevindingen.push(Bevinding {
                volgnummer: 1,
                omschrijving: AuditFout::OntbrekendVolgnummer {
                    verwacht: 1,
                    gevonden: regel.volgnummer,
                }
                .to_string(),
                soort: Bevindingsoort::OntbrekendeRegel,
            });
        }

        // 2. De schakel naar de voorgaande regel moet kloppen.
        if regel.vorige_hash != vorige_hash {
            bevindingen.push(Bevinding {
                volgnummer: regel.volgnummer,
                omschrijving: AuditFout::Ketenbreuk {
                    volgnummer: regel.volgnummer,
                    verwacht: vorige_hash.clone(),
                    gevonden: regel.vorige_hash.clone(),
                }
                .to_string(),
                soort: Bevindingsoort::Ketenbreuk,
            });
        }

        // 3. De inhoud moet overeenkomen met de vastgelegde hash.
        if !regel.hash_klopt()? {
            bevindingen.push(Bevinding {
                volgnummer: regel.volgnummer,
                omschrijving: AuditFout::InhoudGewijzigd { volgnummer: regel.volgnummer }
                    .to_string(),
                soort: Bevindingsoort::InhoudGewijzigd,
            });
        }

        // 4. De tijd hoort niet terug te lopen.
        if let Some(vorig) = vorig_tijdstip {
            if regel.gebeurtenis.tijdstip < vorig {
                bevindingen.push(Bevinding {
                    volgnummer: regel.volgnummer,
                    omschrijving: AuditFout::TijdLooptTerug {
                        volgnummer: regel.volgnummer,
                        vorige: vorig.to_rfc3339(),
                        deze: regel.gebeurtenis.tijdstip.to_rfc3339(),
                    }
                    .to_string(),
                    soort: Bevindingsoort::TijdLooptTerug,
                });
            }
        }

        vorige_hash = regel.hash.clone();
        vorig_volgnummer = Some(regel.volgnummer);
        vorig_tijdstip = Some(regel.gebeurtenis.tijdstip);
    }

    let eerste = regels.first().expect("niet leeg");
    let laatste = regels.last().expect("niet leeg");

    let ankerstatus = match anker {
        None => Ankerstatus::GeenAnker,
        Some(a) => beoordeel_anker(a, regels, laatste),
    };

    Ok(Verificatierapport {
        regels: regels.len() as u64,
        eerste_volgnummer: Some(eerste.volgnummer),
        laatste_volgnummer: Some(laatste.volgnummer),
        laatste_hash: Some(laatste.hash.clone()),
        periode: Some((eerste.gebeurtenis.tijdstip, laatste.gebeurtenis.tijdstip)),
        bevindingen,
        ankerstatus,
    })
}

fn beoordeel_anker(anker: &Anker, regels: &[Ketenregel], laatste: &Ketenregel) -> Ankerstatus {
    if let Err(e) = anker.controleer_handtekening() {
        return Ankerstatus::AnkerOngeldig(e.to_string());
    }
    if laatste.volgnummer < anker.volgnummer {
        return Ankerstatus::KetenIsIngekort {
            anker_volgnummer: anker.volgnummer,
            keten_volgnummer: laatste.volgnummer,
        };
    }
    match regels.iter().find(|r| r.volgnummer == anker.volgnummer) {
        None => Ankerstatus::KetenIsIngekort {
            anker_volgnummer: anker.volgnummer,
            keten_volgnummer: laatste.volgnummer,
        },
        Some(op_ankerpositie) if op_ankerpositie.hash != anker.hash => Ankerstatus::HashWijktAf {
            volgnummer: anker.volgnummer,
            in_anker: anker.hash.clone(),
            in_keten: op_ankerpositie.hash.clone(),
        },
        Some(_) => Ankerstatus::Bevestigd {
            volgnummer: anker.volgnummer,
            regels_sinds_anker: laatste.volgnummer - anker.volgnummer,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{anker::nieuw_sleutelpaar, keten_aan, Actor, Gebeurtenis, Handeling, Ketenstand};
    use chrono::TimeZone;

    fn bouw(aantal: u32) -> (Vec<Ketenregel>, Ketenstand) {
        let mut stand = Ketenstand::leeg();
        let mut regels = Vec::new();
        for n in 1..=aantal {
            let g = Gebeurtenis::nieuw(
                Handeling::RecordGewijzigd,
                Actor::nieuw("u1", "A. de Vries", "fg"),
                Utc.with_ymd_and_hms(2026, 8, 18, 9, n, 0).unwrap(),
                "verwerking",
                format!("0412-{n}"),
                "algemeen",
                "veld ingevuld",
            );
            let (r, s) = keten_aan(&stand, g).unwrap();
            regels.push(r);
            stand = s;
        }
        (regels, stand)
    }

    #[test]
    fn ongeschonden_keten() {
        let (regels, _) = bouw(10);
        let rapport = verifieer(&regels, None).unwrap();
        assert!(rapport.is_ongeschonden());
        assert_eq!(rapport.regels, 10);
        assert_eq!(rapport.laatste_volgnummer, Some(10));
        assert!(rapport.reikwijdte().contains("geen anker"));
    }

    #[test]
    fn leeg_logboek_zonder_anker_is_geldig() {
        let rapport = verifieer(&[], None).unwrap();
        assert!(rapport.is_ongeschonden());
        assert_eq!(rapport.regels, 0);
    }

    #[test]
    fn verwijderde_regel_wordt_gevonden() {
        let (mut regels, _) = bouw(10);
        regels.remove(4); // volgnummer 5
        let rapport = verifieer(&regels, None).unwrap();
        assert!(!rapport.is_ongeschonden());
        assert!(rapport
            .bevindingen
            .iter()
            .any(|b| b.soort == Bevindingsoort::OntbrekendeRegel && b.volgnummer == 5));
        assert!(rapport.bevindingen.iter().any(|b| b.soort == Bevindingsoort::Ketenbreuk));
    }

    #[test]
    fn gewijzigde_inhoud_wordt_gevonden() {
        let (mut regels, _) = bouw(5);
        regels[2].gebeurtenis.omschrijving = "stiekem aangepast".into();
        let rapport = verifieer(&regels, None).unwrap();
        assert!(rapport
            .bevindingen
            .iter()
            .any(|b| b.soort == Bevindingsoort::InhoudGewijzigd && b.volgnummer == 3));
    }

    #[test]
    fn alle_bevindingen_worden_gemeld() {
        let (mut regels, _) = bouw(8);
        regels[1].gebeurtenis.omschrijving = "gewijzigd".into();
        regels[5].gebeurtenis.omschrijving = "ook gewijzigd".into();
        let rapport = verifieer(&regels, None).unwrap();
        let gewijzigd: Vec<_> = rapport
            .bevindingen
            .iter()
            .filter(|b| b.soort == Bevindingsoort::InhoudGewijzigd)
            .collect();
        assert_eq!(gewijzigd.len(), 2, "verificatie moet doortellen na de eerste fout");
    }

    #[test]
    fn teruglopende_tijd_wordt_gemeld() {
        let mut stand = Ketenstand::leeg();
        let mut regels = Vec::new();
        for (n, uur) in [(1u32, 10u32), (2, 11), (3, 9)] {
            let g = Gebeurtenis::nieuw(
                Handeling::RecordGewijzigd,
                Actor::nieuw("u1", "A", "fg"),
                Utc.with_ymd_and_hms(2026, 8, 18, uur, 0, 0).unwrap(),
                "verwerking",
                n.to_string(),
                "algemeen",
                "x",
            );
            let (r, s) = keten_aan(&stand, g).unwrap();
            regels.push(r);
            stand = s;
        }
        let rapport = verifieer(&regels, None).unwrap();
        assert!(rapport
            .bevindingen
            .iter()
            .any(|b| b.soort == Bevindingsoort::TijdLooptTerug && b.volgnummer == 3));
    }

    #[test]
    fn afkappen_wordt_zonder_anker_niet_gezien() {
        let (mut regels, _) = bouw(10);
        regels.truncate(6);
        let rapport = verifieer(&regels, None).unwrap();
        // Dit is de eerlijk benoemde grens: intern klopt alles.
        assert!(rapport.is_ongeschonden());
        assert!(rapport.reikwijdte().contains("niet vast te stellen"));
    }

    #[test]
    fn afkappen_wordt_met_anker_wel_gezien() {
        let (mut regels, stand) = bouw(10);
        let sk = nieuw_sleutelpaar();
        let anker = Anker::plaats(
            &sk,
            "kluis-1",
            &stand,
            Utc.with_ymd_and_hms(2026, 8, 18, 12, 0, 0).unwrap(),
        )
        .unwrap();

        regels.truncate(6);
        let rapport = verifieer(&regels, Some(&anker)).unwrap();
        assert!(!rapport.is_ongeschonden());
        assert_eq!(
            rapport.ankerstatus,
            Ankerstatus::KetenIsIngekort { anker_volgnummer: 10, keten_volgnummer: 6 }
        );
        assert!(rapport.reikwijdte().contains("verwijderd"));
    }

    #[test]
    fn anker_bevestigt_en_benoemt_de_staart() {
        let (regels_bij_anker, stand) = bouw(6);
        let sk = nieuw_sleutelpaar();
        let anker = Anker::plaats(
            &sk,
            "kluis-1",
            &stand,
            Utc.with_ymd_and_hms(2026, 8, 18, 12, 0, 0).unwrap(),
        )
        .unwrap();

        // Ga door op dezelfde keten.
        let mut regels = regels_bij_anker;
        let mut s = stand;
        for n in 7..=9u32 {
            let g = Gebeurtenis::nieuw(
                Handeling::RecordGewijzigd,
                Actor::nieuw("u1", "A", "fg"),
                Utc.with_ymd_and_hms(2026, 8, 18, 13, n, 0).unwrap(),
                "verwerking",
                n.to_string(),
                "algemeen",
                "x",
            );
            let (r, ns) = keten_aan(&s, g).unwrap();
            regels.push(r);
            s = ns;
        }

        let rapport = verifieer(&regels, Some(&anker)).unwrap();
        assert!(rapport.is_ongeschonden());
        assert_eq!(
            rapport.ankerstatus,
            Ankerstatus::Bevestigd { volgnummer: 6, regels_sinds_anker: 3 }
        );
        assert!(rapport.reikwijdte().contains("bevestigd tot en met regel 6"));
        assert!(rapport.reikwijdte().contains("niet uit te sluiten"));
    }

    #[test]
    fn wijziging_voor_het_anker_wordt_gezien() {
        let (mut regels, stand) = bouw(6);
        let sk = nieuw_sleutelpaar();
        let anker = Anker::plaats(
            &sk,
            "kluis-1",
            &stand,
            Utc.with_ymd_and_hms(2026, 8, 18, 12, 0, 0).unwrap(),
        )
        .unwrap();

        // Wijzig regel 6 en herbereken de hash zodat de keten intern lijkt te kloppen.
        regels[5].gebeurtenis.omschrijving = "stilletjes veranderd".into();
        regels[5].hash = Ketenregel::bereken_hash(
            regels[5].volgnummer,
            &regels[5].gebeurtenis,
            &regels[5].vorige_hash,
        )
        .unwrap();

        let rapport = verifieer(&regels, Some(&anker)).unwrap();
        assert!(rapport.bevindingen.is_empty(), "intern klopt de keten weer");
        assert!(!rapport.is_ongeschonden(), "maar het anker verraadt de wijziging");
        assert!(matches!(rapport.ankerstatus, Ankerstatus::HashWijktAf { volgnummer: 6, .. }));
    }

    #[test]
    fn vervalst_anker_wordt_verworpen() {
        let (regels, stand) = bouw(6);
        let sk = nieuw_sleutelpaar();
        let mut anker = Anker::plaats(
            &sk,
            "kluis-1",
            &stand,
            Utc.with_ymd_and_hms(2026, 8, 18, 12, 0, 0).unwrap(),
        )
        .unwrap();
        anker.hash = "f".repeat(64);
        let rapport = verifieer(&regels, Some(&anker)).unwrap();
        assert!(matches!(rapport.ankerstatus, Ankerstatus::AnkerOngeldig(_)));
        assert!(!rapport.is_ongeschonden());
    }
}
