//! Controleregels die continu over de hele gegevensverzameling draaien.
//!
//! # Het verschil met de volledigheidscontrole
//!
//! De volledigheidscontrole in [`dpofg_domain`] kijkt naar **één record**: mist
//! deze verwerking een bewaartermijn? De regels hier kijken naar de
//! **samenhang**: is er een verwerking met een verwerker maar zonder
//! overeenkomst, staat er een register al veertien maanden onaangeraakt, loopt
//! er een verzoek waarvan de termijn over drie dagen verstrijkt.
//!
//! Dat verschil is wezenlijk. De eerste soort fout ziet de gebruiker terwijl
//! hij aan het record werkt. De tweede soort ziet niemand, want er is geen
//! moment waarop iemand ernaar kijkt — tot de toezichthouder belt.
//!
//! # De drie niveaus, en waarom er niet meer zijn
//!
//! | Niveau | Betekenis | Wanneer |
//! |---|---|---|
//! | blokkerend | de handeling gaat niet door | een objectief bepaalbaar feit dat tot een overtreding leidt |
//! | signalerend | verschijnt in de werkvoorraad | iets dat aandacht vraagt maar een oordeel vergt |
//! | rapporterend | verschijnt alleen in de periodieke rapportage | een patroon dat pas over tijd betekenis krijgt |
//!
//! De verleiding is om alles blokkerend te maken. Dat is precies hoe een
//! product onbruikbaar wordt: wie bij elke stap wordt tegengehouden, leert de
//! melding wegklikken, en dan werkt ook de melding niet meer die er wél toe
//! doet. Daarom geldt het **waarschuwingsbudget** uit [`budget`]: meer dan vijf
//! onderbrekende meldingen per gebruiker per week is een defect in het
//! ontwerp, geen probleem van de gebruiker.

#![forbid(unsafe_code)]
#![warn(missing_debug_implementations, rust_2018_idioms)]

pub mod budget;
pub mod motor;
pub mod regels;

pub use budget::{Budgetstand, Waarschuwingsbudget};
pub use motor::{Bevinding, Niveau, Ontvangerrol, Regel, Regelmotor, Regelrapport};

/// Versie van deze crate.
pub const VERSIE: &str = env!("CARGO_PKG_VERSION");
