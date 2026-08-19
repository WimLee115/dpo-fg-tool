// Voorkomt een tweede consolevenster op Windows bij een uitgave.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // Twee vlaggen, en verder niets. De schil is geen opdrachtregelprogramma;
    // daarvoor is er `dpofg`. `--stand` bestaat omdat een verkeerd gebouwde
    // binary er precies hetzelfde uitziet als een goede, totdat hij bij het
    // starten een leeg venster met "Connection refused" toont. Een installatie
    // moet dat kunnen nagaan zonder een venster te openen.
    match std::env::args().nth(1).as_deref() {
        Some("--versie" | "--version" | "-V") => {
            println!("dpofg-schil {}", env!("CARGO_PKG_VERSION"));
        }
        Some("--stand") => {
            println!("dpofg-schil {}", env!("CARGO_PKG_VERSION"));
            println!("bouw          {}", dpofg_schil_lib::bouwsoort());
            println!("scherm        {}", dpofg_schil_lib::schermbron());
            if dpofg_schil_lib::is_ontwikkelbouw() {
                std::process::exit(1);
            }
        }
        _ => dpofg_schil_lib::draai(),
    }
}
