#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

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
        .run(tauri::generate_context!())?;
    Ok(())
}

fn main() -> anyhow::Result<()> {
    gui()
}
