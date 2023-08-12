#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::path::PathBuf;

use config::{Config, UnVerified};

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
fn load_config(config_path: PathBuf) -> Result<config::Config<UnVerified>, String> {
    config::Config::from_existing_config_file(config_path).map_err(|err| err.to_string())
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
fn create_shortcut(
    // config: Config<UnVerified>,
    config_path: PathBuf,
    shortcut_path: PathBuf,
) -> Result<(), String> {
    // TODO: fix what happens if the config_path is empty. This should probably save the current config in ./config/sma_config_1.json -> ./config/sma_config_2.json, -> ./config/sma_config_n.json

    // TODO: if shortcut_path is empty we should not do anything and just return error.

    if !config_path.is_file() {
        return Err(format!(
            "The given config file path `{}` does not exist",
            config_path.display()
        ));
    }

    // let config = config.verify().map_err(|e| e.to_string())?;

    let sma_path = std::env::current_exe()
        .map_err(|e| e.to_string())?
        .parent()
        .unwrap()
        .to_path_buf()
        .join("sma.exe");

    if sma_path.try_exists().is_ok_and(|x| x) {
        let mut link = mslnk::ShellLink::new(sma_path).unwrap();
        link.set_arguments(Some(format!("config {}", config_path.display())));
        link.create_lnk(shortcut_path).unwrap();
    } else {
        return Err(format!(
            "There was an error while trying to find `sma.exe` at `{}`",
            sma_path.display()
        ));
    }
    Ok(())
}

#[tauri::command]
fn set_window_title(title: String) -> Result<(), String> {
    todo!()
}

// TODO: add bindings to do a test run of the application from the gui.

fn main() -> anyhow::Result<()> {
    gui()
}
