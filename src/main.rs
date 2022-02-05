#[cfg(test)]
mod tests;

mod my_lib;

use crate::my_lib::{parse_args, start_program};
use std::env;
use std::process::Child;

fn main() -> Result<(), String> {
    let (start_programs, exit_on) = parse_args(env::args().collect())?;

    println!("Start programs: {:?}", start_programs);
    println!("Exit on:        {:?}", exit_on);

    // let paths = get_paths(&start_programs)?;

    let mut started_programs = Vec::<Child>::with_capacity(start_programs.len());

    for program in &start_programs {
        started_programs.push(start_program(program)?);
    }

    Ok(())
}
