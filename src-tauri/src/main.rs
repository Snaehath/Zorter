// Release builds use the Windows GUI subsystem so no console window appears.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    zorter_lib::run();
}
