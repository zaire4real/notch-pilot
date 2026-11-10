// Prevents additional console window on Windows in release, has no effect on macOS.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    notch_pilot_lib::run();
}
