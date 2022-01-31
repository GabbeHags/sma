use crate::{get_paths, parse_args};

#[test]
fn program_test() {
    let args = vec!["self", "--start", r"test_program\hello_world.exe"].iter().map(|arg| arg.to_string()).collect();
    let (start, _exit) = match parse_args(args) {
        Ok(r) => {r}
        Err(e) => {panic!("{e}")}
    };
    let paths = match get_paths(start) {
        Ok(r) => {r}
        Err(e) => {panic!("{e}")}
    };
    println!("{paths:?}");
}

#[test]
fn parse_args_test1() {
    let args = vec!["self", "--start", r"C:\Program Files (x86)\Google\Chrome\Application\chrome.exe", "2", "3", "--exit",  "1"].iter().map(|arg| arg.to_string()).collect();
    let (start, exit) = match parse_args(args) {
        Ok(r) => {r}
        Err(e) => {panic!("{e}")}
    };
    assert_eq!(start.len(), 3);
    assert_eq!(exit.len(), 1);
}
#[test]
fn parse_args_test2() {
    let args = vec!["self", "--star", r"C:\Program Files (x86)\Google\Chrome\Application\chrome.exe", "2", "3", "--exit",  "1"].iter().map(|arg| arg.to_string()).collect();
    assert!(parse_args(args).is_err());
    let args = vec!["self", "--start", r"C:\Program Files (x86)\Google\Chrome\Application\chrome.exe", "2", "3", "--exi",  "1"].iter().map(|arg| arg.to_string()).collect();
    assert!(parse_args(args).is_err());
}

#[test]
fn get_paths_test1() {
    let args = vec!["self", "--start", r"C:\Program Files (x86)\Google\Chrome\Application\chrome.exe"].iter().map(|arg| arg.to_string()).collect();
    let (start, _exit) = parse_args(args).unwrap();
    let paths = match get_paths(start) {
        Ok(r) => {r}
        Err(e) => {panic!("{e}")}
    };
    assert_eq!(paths.len(), 1);
}
#[test]
fn get_paths_test2() {
    let args = vec!["self", "--start", r"C:\Program Files (x86)\Google\Chrome\Application\chrom.exe"].iter().map(|arg| arg.to_string()).collect();
    let (start, _exit) = parse_args(args).unwrap();
    assert!(get_paths(start).is_err());
    let args = vec!["self", "--start", r"C:\Program Files (x86)\Google\Chrome\Application\chrome.xe"].iter().map(|arg| arg.to_string()).collect();
    let (start, _exit) = parse_args(args).unwrap();
    assert!(get_paths(start).is_err());
}