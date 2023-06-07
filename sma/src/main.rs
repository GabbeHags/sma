use std::process::Command;

fn main() -> anyhow::Result<()> {
    let gui_path = std::env::current_exe()?
        .parent()
        .unwrap()
        .to_path_buf()
        .join("sma-gui.exe");

    if gui_path.exists() && std::env::args().len() == 1 {
        _ = Command::new(&gui_path).spawn()?;
    } else {
        sma::run()?;
    }

    Ok(())
}
