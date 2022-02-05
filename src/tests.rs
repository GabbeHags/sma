use std::path::Path;
use crate::my_lib::*;

struct Setup {
    test_program_path: &'static str,
    test_program_path_args: &'static str,
    test_program_path_args_2: &'static str,
    test_program_path_wrong_extension: &'static str,
    test_program_path_wrong_path: &'static str,
    test_program_path_no_extension: &'static str,
}

impl Setup {
    fn new() -> Self {
        Self {
            test_program_path: r"test_program\test.exe",
            test_program_path_args: r"test_program\test.exe 2",
            test_program_path_args_2: r"test_program\test.exe 2 3",
            test_program_path_wrong_extension: r"test_program\test.ex",
            test_program_path_wrong_path: r"test_program\tes.exe",
            test_program_path_no_extension: r"test_program\test",
        }
    }
}

#[test]
fn program_test() {
    let setup = Setup::new();

    let args = vec!["self", "--start", setup.test_program_path]
        .iter()
        .map(|arg| arg.to_string())
        .collect();
    let (start, _exit) = match parse_args(args) {
        Ok(r) => r,
        Err(e) => {
            panic!("{e}")
        }
    };
    println!("{start:?}");
}

#[test]
fn parse_args_valid_input() {
    let setup = Setup::new();

    let args = vec![
        "self",
        "--start",
        setup.test_program_path,
        "--exit",
        "test.exe",
    ]
    .iter()
    .map(|arg| arg.to_string())
    .collect();
    let (start, exit) = match parse_args(args) {
        Ok(r) => r,
        Err(e) => {
            panic!("{e}")
        }
    };
    assert_eq!(start.len(), 1);
    assert_eq!(exit.len(), 1);
}
#[test]
fn parse_args_check_misspelled_flag_start() {
    let setup = Setup::new();

    let args = vec!["self", "--star", setup.test_program_path, "--exit", "2"]
        .iter()
        .map(|arg| arg.to_string())
        .collect();
    match parse_args(args) {
        Ok(_) => {
            panic!("Should not get here")
        }
        Err(e) => {
            assert_eq!(e, "`--star` is not a valid argument.");
        }
    }
}
#[test]
fn parse_args_with_args() {
    let setup = Setup::new();

    let args = vec!["self", "--start", setup.test_program_path_args, "--exit", "test.exe"]
        .iter()
        .map(|arg| arg.to_string())
        .collect();
    match parse_args(args) {
        Ok((ps, e)) => {
            let p = &ps[0];
            let arg = p.args.as_ref().unwrap().get(0).unwrap().as_str();
            assert_eq!(p.path.as_path(), Path::new(setup.test_program_path));
            assert_eq!(arg, "2");
        }
        Err(e) => {
            panic!("Should not get here");
        }
    }
}
#[test]
fn parse_args_with_args_2() {
    let setup = Setup::new();

    let args = vec!["self", "--start", setup.test_program_path_args_2, "--exit", "test.exe"]
        .iter()
        .map(|arg| arg.to_string())
        .collect();
    match parse_args(args) {
        Ok((ps, _)) => {
            let p = &ps[0];
            let arg_1 = p.args.as_ref().unwrap().get(0).unwrap().as_str();
            let arg_2 = p.args.as_ref().unwrap().get(1).unwrap().as_str();
            assert_eq!(p.path.as_path(), Path::new(setup.test_program_path));
            assert_eq!(arg_1, "2");
            assert_eq!(arg_2, "3");
        }
        Err(e) => {
            panic!("Should not get here");
        }
    }
}
#[test]
fn parse_args_argument_did_not_start_with() {
    let args = vec!["self", "start"]
        .iter()
        .map(|arg| arg.to_string())
        .collect();
    match parse_args(args) {
        Ok(_) => {
            panic!("Should not get here")
        }
        Err(e) => {
            assert_eq!(e, "Argument `start` did not start with `--`");
        }
    }
}
#[test]
fn parse_args_no_args_given() {
    let args = vec!["self"].iter().map(|arg| arg.to_string()).collect();
    match parse_args(args) {
        Ok(_) => {
            panic!("Should not get here")
        }
        Err(e) => {
            assert_eq!(e, "No arguments were given, this is probably a mistake");
        }
    }
}
#[test]
fn parse_args_start_given_empty() {
    let args = vec!["self", "--start"]
        .iter()
        .map(|arg| arg.to_string())
        .collect();
    match parse_args(args) {
        Ok(_) => {
            panic!("Should not get here")
        }
        Err(e) => {
            assert_eq!(e, "--start was given but no argument to it were given");
        }
    }
}
#[test]
fn parse_args_exit_but_no_start() {
    let args = vec!["self", "--exit"]
        .iter()
        .map(|arg| arg.to_string())
        .collect();
    match parse_args(args) {
        Ok(_) => {
            panic!("Should not get here")
        }
        Err(e) => {
            assert_eq!(
                e,
                "--exit was given but not --start, --exit uses the info given in --start"
            );
        }
    }
}
#[test]
fn parse_args_exit_given_empty() {
    let setup = Setup::new();

    let args = vec!["self", "--start", setup.test_program_path, "--exit"]
        .iter()
        .map(|arg| arg.to_string())
        .collect();
    match parse_args(args) {
        Ok(_) => {
            panic!("Should not get here")
        }
        Err(e) => {
            assert_eq!(e, "--exit was given but no argument to it were given");
        }
    }
}
#[test]
fn parse_args_check_misspelled_flag_exit() {
    let setup = Setup::new();
    let args = vec!["self", "--start", setup.test_program_path, "--exi", "2"]
        .iter()
        .map(|arg| arg.to_string())
        .collect();
    match parse_args(args) {
        Ok(_) => {
            panic!("Should not get here")
        }
        Err(e) => {
            assert_eq!(e, "`--exi` is not a valid argument.")
        }
    }
}
#[test]
fn parse_args_more_than_one_arg_on_exit() {
    let setup = Setup::new();
    let args = vec![
        "self",
        "--start",
        setup.test_program_path,
        "--exit",
        "2",
        "3",
    ]
    .iter()
    .map(|arg| arg.to_string())
    .collect();
    match parse_args(args) {
        Ok(_) => {
            panic!("Should not get here")
        }
        Err(e) => {
            assert_eq!(e, "--exit only accepts 1 argument but, 2 were given.")
        }
    }
}
// #[test]
// fn match_exit_with_start_not_correct_file_name() {
//     let setup = Setup::new();
//     let v: Vec<String> = vec![
//         setup.test_program_path,
//         "..",
//     ]
//     .iter()
//     .map(|arg| arg.to_string())
//     .collect();
//
//     match match_exit_with_start("2", &v) {
//         Ok(_) => {
//             panic!("Should not get here")
//         }
//         Err(e) => {
//             assert_eq!(e, "`..` does not have a correct file name.")
//         }
//     }
// }
// #[test]
// fn match_exit_with_start_no_match() {
//     let v: Vec<String> = vec!["a/b/c/d.exe"].iter().map(|arg| arg.to_string()).collect();
//     match match_exit_with_start("aaa.exe", &v) {
//         Ok(_) => {
//             panic!("Should not get here")
//         }
//         Err(e) => {
//             assert_eq!(e, "`aaa.exe` does not match with any argument in --start")
//         }
//     }
// }
//
// #[test]
// fn match_exit_with_start_match_with_multiple() {
//     let v: Vec<String> = vec!["a/b/c/d.exe", "a/b/c/d.exe"].iter().map(|arg| arg.to_string()).collect();
//     match match_exit_with_start("d.exe", &v) {
//         Ok(_) => {
//             panic!("Should not get here")
//         }
//         Err(e) => {
//             assert_eq!(e, "The --start arguments got two or more matching with the --exit")
//         }
//     }
// }

// #[test]
// fn get_paths_valid_input() {
//     let setup = Setup::new();
//
//     let args = vec!["self", "--start", setup.test_program_path]
//         .iter()
//         .map(|arg| arg.to_string())
//         .collect();
//     let (start, _exit) = parse_args(args).unwrap();
//     let paths = match get_paths(&start) {
//         Ok(r) => r,
//         Err(e) => {
//             panic!("{e}")
//         }
//     };
//     assert_eq!(paths.len(), 1);
// }
// #[test]
// fn get_paths_not_a_file() {
//     let setup = Setup::new();
//
//     let args = vec!["self", "--start", setup.test_program_path_wrong_path]
//         .iter()
//         .map(|arg| arg.to_string())
//         .collect();
//     let (start, _) = parse_args(args).unwrap();
//     match get_paths(&start) {
//         Ok(_) => {
//             panic!("Should not get here")
//         }
//         Err(e) => {
//             assert_eq!(
//                 e,
//                 r"The given path does not point to a file: `test_program\tes.exe`"
//             );
//         }
//     }
//
//     let args = vec!["self", "--start", setup.test_program_path_wrong_extension]
//         .iter()
//         .map(|arg| arg.to_string())
//         .collect();
//     let (start, _exit) = parse_args(args).unwrap();
//     assert!(get_paths(&start).is_err());
// }
// #[test]
// fn get_paths_not_a_exe() {
//     let setup = Setup::new();
//
//     let args = vec!["self", "--start", setup.test_program_path_wrong_extension]
//         .iter()
//         .map(|arg| arg.to_string())
//         .collect();
//     let (start, _exit) = parse_args(args).unwrap();
//     match get_paths(&start) {
//         Ok(_) => {
//             panic!("Should not get here")
//         }
//         Err(e) => {
//             assert_eq!(e, "The given file is not a .exe: `\"test.ex\"`")
//         }
//     }
// }
// #[test]
// fn get_paths_no_extension() {
//     let setup = Setup::new();
//
//     let args = vec!["self", "--start", setup.test_program_path_no_extension]
//         .iter()
//         .map(|arg| arg.to_string())
//         .collect();
//     let (start, _exit) = parse_args(args).unwrap();
//     match get_paths(&start) {
//         Ok(_) => {
//             panic!("Should not get here")
//         }
//         Err(e) => {
//             assert_eq!(
//                 e,
//                 "Something is wrong with the extension of the given file: `\"test\"`"
//             )
//         }
//     }
// }
