#[cfg(test)]
mod tests;

mod my_lib;

use crate::my_lib::{get_paths, parse_args, start_program};
use std::env;
use std::process::Child;


fn main() -> Result<(), String> {
    // let (start_programs, exit_on) = parse_args(env::args().collect())?;
    let (start_programs, exit_on) = parse_args(
        vec!["sma", "--start", "test_program/hello_world.exe", "test_program/test.exe"]
            .iter()
            .map(|a| a.to_string())
            .collect(),
    )?;

    println!("Start programs: {:?}", start_programs);
    println!("Exit on:        {:?}", exit_on);

    let paths = get_paths(&start_programs)?;

    let mut started_programs = Vec::<Child>::with_capacity(paths.len());

    for path in paths {
        started_programs.push(start_program(&path, None)?);
    }

    Ok(())
}
