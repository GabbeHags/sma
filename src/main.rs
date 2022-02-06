#[cfg(test)]
mod tests;

mod my_lib;

use crate::my_lib::{parse_args, Program};
use std::env;

fn main() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();

    // if the help arg was given.
    if args.contains(&"--h".to_string()) {

        println!(
            concat!(
            "Help:\n",
            "--start, Starts the given applications.\n",
            "--exit,  Exits all the started applications if the argument given to --exit is exited.\n"
            )
        );
        return Ok(())
    }
    let mut start_programs = parse_args(args)?;

    let mut index = None;

    for (i, p) in start_programs.iter_mut().enumerate() {
        p.start()?;
        if p.exit_on_this {
            index = Some(i)
        }
    }

    let mut exit_on_this: Option<Program> = None;

    if let Some(i) = index {
        exit_on_this = Some(start_programs.swap_remove(i));
    }

    if let Some(mut exit) = exit_on_this {
        let proc = exit.proc.as_mut().unwrap();
        if let Err(e) =  proc.wait() {
            return Err(format!("Failed to wait on {:?} because: {e}", exit.path))
        }

        for p in start_programs.iter_mut() {
            if p.proc.as_mut().unwrap().kill().is_err() {
                return Err(format!("Failed to exit: {:?}", p.path))
            }
        }
    }
    Ok(())
}
