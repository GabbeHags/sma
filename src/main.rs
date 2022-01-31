#[cfg(test)]
mod tests;

mod my_lib;

use std::env;
use std::process::exit;
use crate::my_lib::{parse_args, get_paths};

fn main() {

    let (start_programs, exit_on) = match parse_args(env::args().collect()) {
        Ok(r) => r,
        Err(e) => {
            println!("{e}");
            exit(-1)
        }
    };
    println!("{:?}", start_programs);
    println!("{:?}", exit_on);

    let paths = match get_paths(start_programs) {
        Ok(r) => {r}
        Err(e) => {
            println!("{e}");
            exit(-1)
        }
    };

}
