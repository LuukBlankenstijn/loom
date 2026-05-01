#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    loom_station_registration_lib::run()
}
