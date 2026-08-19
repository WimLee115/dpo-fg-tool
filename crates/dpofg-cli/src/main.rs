//! Bedieningsschil van `dpo-fg-tool`.
//!
//! Deze schil is bewust eerst gebouwd: hij dwingt af dat elke handeling ook
//! zonder grafische omgeving werkt, en daarmee dat de logica in de lagen
//! eronder zit en niet in een scherm. Alles wat hier kan, kan straks ook in de
//! grafische schil — en niets meer dan dat.

#![forbid(unsafe_code)]

mod opdrachten;
mod uitvoer;
mod wachtwoord;

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// Lokaal draaiend, versleuteld werkplatform voor de functionaris voor
/// gegevensbescherming en de security officer.
#[derive(Parser, Debug)]
#[command(
    name = "dpofg",
    version,
    about,
    long_about = "Beheert het verwerkingsregister, datalekken, verzoeken van betrokkenen en de \
                  bijbehorende termijnen in een versleuteld dossier op de eigen machine.\n\n\
                  Er is geen cloud, geen telemetrie en geen uitgaand netwerkverkeer.",
    after_help = "Het wachtwoord wordt nooit als argument aangenomen; het wordt gevraagd of uit \
                  DPOFG_WACHTWOORD gelezen."
)]
struct Opdrachtregel {
    /// Pad naar het kluisbestand.
    #[arg(long, short = 'k', global = true, env = "DPOFG_KLUIS")]
    kluis: Option<PathBuf>,

    /// Toon uitgebreide meldingen.
    #[arg(long, global = true)]
    uitgebreid: bool,

    #[command(subcommand)]
    opdracht: Opdracht,
}

#[derive(Subcommand, Debug)]
enum Opdracht {
    /// Beheer van het kluisbestand zelf.
    #[command(subcommand)]
    Kluis(opdrachten::kluis::Kluisopdracht),

    /// Het verwerkingsregister.
    #[command(subcommand)]
    Register(opdrachten::register::Registeropdracht),

    /// Datalekken en beveiligingsincidenten.
    #[command(subcommand)]
    Incident(opdrachten::incident::Incidentopdracht),

    /// Gegevensbeschermingseffectbeoordelingen.
    #[command(subcommand)]
    Dpia(opdrachten::dpia::Dpiaopdracht),

    /// Verzoeken van betrokkenen.
    #[command(subcommand)]
    Verzoek(opdrachten::verzoek::Verzoekopdracht),

    /// Verzoeken om informatie op grond van de Wet open overheid.
    #[command(subcommand)]
    Woo(opdrachten::woo::Wooopdracht),

    /// Redactieregie: aanwijzen wat weg moet en bewijzen dat het weg is.
    #[command(subcommand)]
    Redactie(opdrachten::redactie::Redactieopdrachtkeuze),

    /// Het spoor van de Wet politiegegevens.
    #[command(subcommand)]
    Wpg(opdrachten::wpg::Wpgopdracht),

    /// Veldmapping: het register naast de werkelijkheid leggen.
    #[command(subcommand)]
    Mapping(opdrachten::mapping::Mappingopdracht),

    /// Belangenafwegingen bij een gerechtvaardigd belang.
    #[command(subcommand)]
    Lia(opdrachten::lia::Liaopdracht),

    /// Doorgiften buiten de Europese Economische Ruimte.
    #[command(subcommand)]
    Doorgifte(opdrachten::doorgifte::Doorgifteopdracht),

    /// Leveranciers en verwerkersovereenkomsten.
    #[command(subcommand)]
    Leverancier(opdrachten::leverancier::Leveranciersopdracht),

    /// Welke eisen op een gekozen datum niet meer aantoonbaar zijn.
    Prognose(opdrachten::prognose::Prognoseargumenten),

    /// Risicobeoordelingen waarop de zorgplichtmaatregelen steunen.
    #[command(subcommand)]
    Risico(opdrachten::risico::Risicoopdracht),

    /// De zorgplichtcontrolset van artikel 21 lid 3 Cyberbeveiligingswet.
    #[command(subcommand)]
    Zorgplicht(opdrachten::zorgplicht::Zorgplichtopdracht),

    /// De controleregels over de hele verzameling draaien.
    Controle(opdrachten::controle::Controleopties),

    /// Het ketenlogboek.
    #[command(subcommand)]
    Logboek(opdrachten::logboek::Logboekopdracht),

    /// Termijnen berekenen.
    Termijn(opdrachten::termijn::Termijnopties),

    /// Het kennispakket met de juridische inhoud.
    #[command(subcommand)]
    Pakket(opdrachten::pakket::Pakketopdracht),

    /// Stel een dossier samen voor een toezichthouder of auditor.
    Dossier(opdrachten::dossier::Dossieropties),
}

fn main() {
    if let Err(fout) = draai() {
        eprintln!("\n\x1b[31m■\x1b[0m {fout}");
        // De keten van oorzaken erbij: een melding die alleen het symptoom
        // noemt, laat de gebruiker raden.
        let mut bron = fout.source();
        while let Some(b) = bron {
            eprintln!("  \x1b[2m← {b}\x1b[0m");
            bron = b.source();
        }
        std::process::exit(1);
    }
}

fn draai() -> Result<()> {
    let args = Opdrachtregel::parse();

    if args.uitgebreid {
        tracing_subscriber::fmt()
            .with_env_filter(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| "dpofg=debug".into()),
            )
            .init();
    }

    let nu = chrono::Utc::now();

    match args.opdracht {
        Opdracht::Kluis(o) => opdrachten::kluis::draai(o, args.kluis, nu),
        Opdracht::Register(o) => opdrachten::register::draai(o, args.kluis, nu),
        Opdracht::Incident(o) => opdrachten::incident::draai(o, args.kluis, nu),
        Opdracht::Dpia(o) => opdrachten::dpia::draai(o, args.kluis, nu),
        Opdracht::Verzoek(o) => opdrachten::verzoek::draai(o, args.kluis, nu),
        Opdracht::Woo(o) => opdrachten::woo::draai(o, args.kluis, nu),
        Opdracht::Redactie(o) => opdrachten::redactie::draai(o, args.kluis, nu),
        Opdracht::Wpg(o) => opdrachten::wpg::draai(o, args.kluis, nu),
        Opdracht::Mapping(o) => opdrachten::mapping::draai(o, args.kluis, nu),
        Opdracht::Lia(o) => opdrachten::lia::draai(o, args.kluis, nu),
        Opdracht::Doorgifte(o) => opdrachten::doorgifte::draai(o, args.kluis, nu),
        Opdracht::Leverancier(o) => opdrachten::leverancier::draai(o, args.kluis, nu),
        Opdracht::Prognose(o) => opdrachten::prognose::draai(o, args.kluis, nu),
        Opdracht::Risico(o) => opdrachten::risico::draai(o, args.kluis, nu),
        Opdracht::Zorgplicht(o) => opdrachten::zorgplicht::draai(o, args.kluis, nu),
        Opdracht::Controle(o) => opdrachten::controle::draai(o, args.kluis, nu),
        Opdracht::Logboek(o) => opdrachten::logboek::draai(o, args.kluis, nu),
        Opdracht::Termijn(o) => opdrachten::termijn::draai(o, nu),
        Opdracht::Pakket(o) => opdrachten::pakket::draai(o, args.kluis, nu),
        Opdracht::Dossier(o) => opdrachten::dossier::draai(o, args.kluis, nu),
    }
}
