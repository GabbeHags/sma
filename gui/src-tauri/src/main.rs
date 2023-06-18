#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::path::{Path, PathBuf};

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
        .invoke_handler(tauri::generate_handler![
            load_config,
            save_config,
            create_shortcut
        ])
        .run(tauri::generate_context!())?;
    Ok(())
}

#[tauri::command]
fn load_config(config_path: PathBuf) -> Result<config::Config<Verified>, String> {
    config::Config::from_existing_config_file(config_path)
        .map_err(|err| err.to_string())?
        .verify()
        .map_err(|err| err.to_string())
}

#[tauri::command]
fn save_config(config: Config<UnVerified>, config_path: PathBuf) -> Result<(), String> {
    config
        .verify()
        .map_err(|err| err.to_string())?
        .create_file(config_path, true)
        .map_err(|err| err.to_string())
}

#[tauri::command]
fn create_shortcut(config: Config<UnVerified>, config_path: PathBuf) -> Result<(), String> {
    mslnk::ShellLink::new(r"D:\Programming\Rust\sma\gui\src-tauri\sma.exe")
        .unwrap()
        .create_lnk(r"D:\Programming\Rust\sma\sma_test.lnk")
        .unwrap();
    Ok(())
}

fn main() -> anyhow::Result<()> {
    gui()
}
