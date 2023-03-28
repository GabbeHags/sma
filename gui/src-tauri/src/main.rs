#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

fn gui() -> anyhow::Result<()> {
    tauri::Builder::default().run(tauri::generate_context!())?;
    Ok(())
}

fn main() -> anyhow::Result<()> {
    if std::env::args_os().len() == 1 {
        gui()?;
    } else {
        run::run()?;
    }

    Ok(())
}
