use my_lib::{gui::gui, run::run};

mod my_lib;

fn main() -> anyhow::Result<()> {
    if std::env::args_os().len() == 1 {
        gui()?;
    } else {
        run()?;
    }

    Ok(())
}
