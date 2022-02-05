#[cfg(test)]
mod tests;

mod my_lib;

use crate::my_lib::{parse_args, Program};
use std::env;

fn main() -> Result<(), String> {
    let mut start_programs = parse_args(env::args().collect())?;

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
