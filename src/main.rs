#[cfg(test)]
mod tests;

mod my_lib;

use std::env;
use crate::my_lib::{parse_args, get_paths};

fn main() -> Result<(), String>{
    let (start_programs, exit_on) = parse_args(env::args().collect())?;

    println!("{:?}", start_programs);
    println!("{:?}", exit_on);

    let paths = get_paths(start_programs)?;
    Ok(())

}
