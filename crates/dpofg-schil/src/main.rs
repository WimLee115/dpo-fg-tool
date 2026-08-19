// Voorkomt een tweede consolevenster op Windows bij een uitgave.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    dpofg_schil_lib::draai()
}
