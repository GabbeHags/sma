use std::process::{Command, Stdio};
use crate::{get_paths, parse_args};

struct Setup {
    test_program_path: &'static str,
    test_program_path_wrong_extension: &'static str,
    test_program_path_wrong_path: &'static str,
}

impl Setup {
    fn new() -> Self {
        Self {
            test_program_path: r"test_program\hello_world.exe",
            test_program_path_wrong_extension: r"test_program\hello_world.ex",
            test_program_path_wrong_path: r"test_program\hello_worl.exe"
        }
    }
}

#[test]
fn program_test() {
    let setup = Setup::new();

    let args = vec!["self", "--start", setup.test_program_path].iter().map(|arg| arg.to_string()).collect();
    let (start, _exit) = match parse_args(args) {
        Ok(r) => {r}
        Err(e) => {panic!("{e}")}
    };
    let paths = match get_paths(start) {
        Ok(r) => {r}
        Err(e) => {panic!("{e}")}
    };
    println!("{paths:?}");
    // let p = paths[0].to_path_buf();
    // let program = Command::new(p)
    //     .stdout(Stdio::piped())
    //     .output()
    //     .expect("Failed to execute command");
    // assert_eq!(String::from_utf8_lossy(&program.stdout), "Hello, world!\n")
}

#[test]
fn parse_args_test1() {
    let setup = Setup::new();

    let args = vec!["self", "--start", setup.test_program_path, "2", "3", "--exit",  "1"].iter().map(|arg| arg.to_string()).collect();
    let (start, exit) = match parse_args(args) {
        Ok(r) => {r}
        Err(e) => {panic!("{e}")}
    };
    assert_eq!(start.len(), 3);
    assert_eq!(exit.len(), 1);
}
#[test]
fn parse_args_test2() {
    let setup = Setup::new();

    let args = vec!["self", "--star", setup.test_program_path, "2", "3", "--exit",  "1"].iter().map(|arg| arg.to_string()).collect();
    assert!(parse_args(args).is_err());
    let args = vec!["self", "--start", setup.test_program_path, "2", "3", "--exi",  "1"].iter().map(|arg| arg.to_string()).collect();
    assert!(parse_args(args).is_err());
}

#[test]
fn get_paths_test1() {
    let setup = Setup::new();

    let args = vec!["self", "--start", setup.test_program_path].iter().map(|arg| arg.to_string()).collect();
    let (start, _exit) = parse_args(args).unwrap();
    let paths = match get_paths(start) {
        Ok(r) => {r}
        Err(e) => {panic!("{e}")}
    };
    assert_eq!(paths.len(), 1);
}
#[test]
fn get_paths_test2() {
    let setup = Setup::new();

    let args = vec!["self", "--start", setup.test_program_path_wrong_path].iter().map(|arg| arg.to_string()).collect();
    let (start, _exit) = parse_args(args).unwrap();
    assert!(get_paths(start).is_err());
    let args = vec!["self", "--start", setup.test_program_path_wrong_extension].iter().map(|arg| arg.to_string()).collect();
    let (start, _exit) = parse_args(args).unwrap();
    assert!(get_paths(start).is_err());
}