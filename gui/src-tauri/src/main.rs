#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::path::PathBuf;

use config::{Config, UnVerified, Verified};

#[cfg(debug_assertions)]
use tauri::Manager;

fn gui() -> anyhow::Result<()> {
    tauri::Builder::default()
        .setup(
            |#[cfg_attr(not(debug_assertions), allow(unused_variables))] app| {
                #[cfg(debug_assertions)] // only include this code on debug builds
                {
                    let window = app.get_window("main").unwrap();
                    window.open_devtools();
                    window.close_devtools();
                }
                Ok(())
            },
        )
        .invoke_handler(tauri::generate_handler![load_config, save_config])
        .run(tauri::generate_context!())?;
    Ok(())
}

fn main() -> anyhow::Result<()> {
    gui()
}

#[tauri::command]
fn load_config(config_path: PathBuf) -> Result<config::Config<Verified>, String> {
    // TODO: fix so that if the start vec is empty we should still send it to the front end
    config::Config::from_existing_config_file(config_path).map_err(|err| err.to_string())
}

#[tauri::command]
fn save_config(config: Config<UnVerified>, config_path: PathBuf) -> Result<(), String> {
    let verified_config = config.verify().map_err(|err| err.to_string())?;

    config::Config::create_file_from_config(verified_config, config_path, true)
        .map_err(|err| err.to_string())
}
