#[cfg(test)]
mod tests;

mod my_lib;

use crate::my_lib::{get_paths, parse_args};
use std::env;

fn main() -> Result<(), String> {
    // let (start_programs, exit_on) = parse_args(env::args().collect())?;
    let (start_programs, exit_on) = parse_args(vec!["sma", "--start", "test_program/hello_world.exe", "--exit"]
        .iter().map(|a| a.to_string()).collect())?;

    if start_programs.is_empty() {
        // Err("")
    }
    println!("Start programs: {:?}", start_programs);
    println!("Exit on:        {:?}", exit_on);

    let paths = get_paths(start_programs)?;
    Ok(())
}
