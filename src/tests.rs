use std::process::{Command, Stdio};
use crate::{get_paths, parse_args};

fn program_path() -> String{
    r"test_program\hello_world.exe".to_string()
}

#[test]
fn program_test() {
    let args = vec!["self", "--start", program_path().as_str()].iter().map(|arg| arg.to_string()).collect();
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
    let args = vec!["self", "--start", program_path().as_str(), "2", "3", "--exit",  "1"].iter().map(|arg| arg.to_string()).collect();
    let (start, exit) = match parse_args(args) {
        Ok(r) => {r}
        Err(e) => {panic!("{e}")}
    };
    assert_eq!(start.len(), 3);
    assert_eq!(exit.len(), 1);
}
#[test]
fn parse_args_test2() {
    let args = vec!["self", "--star", program_path().as_str(), "2", "3", "--exit",  "1"].iter().map(|arg| arg.to_string()).collect();
    assert!(parse_args(args).is_err());
    let args = vec!["self", "--start", program_path().as_str(), "2", "3", "--exi",  "1"].iter().map(|arg| arg.to_string()).collect();
    assert!(parse_args(args).is_err());
}

#[test]
fn get_paths_test1() {
    let args = vec!["self", "--start", program_path().as_str()].iter().map(|arg| arg.to_string()).collect();
    let (start, _exit) = parse_args(args).unwrap();
    let paths = match get_paths(start) {
        Ok(r) => {r}
        Err(e) => {panic!("{e}")}
    };
    assert_eq!(paths.len(), 1);
}
#[test]
fn get_paths_test2() {
    let args = vec!["self", "--start", r"test_program\hello_worl.exe"].iter().map(|arg| arg.to_string()).collect();
    let (start, _exit) = parse_args(args).unwrap();
    assert!(get_paths(start).is_err());
    let args = vec!["self", "--start", r"test_program\hello_world.ex"].iter().map(|arg| arg.to_string()).collect();
    let (start, _exit) = parse_args(args).unwrap();
    assert!(get_paths(start).is_err());
}