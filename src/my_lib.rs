use std::path::{Path, PathBuf};
use std::process::{Child, Command};

#[derive(Default, Debug)]
pub struct Program {
    pub path: PathBuf,
    pub args: Option<Vec<String>>,
    pub proc: Option<Child>,
    pub exit_on_this: bool
}

impl PartialEq for Program {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
    }
}

impl Program {
    fn add_arg(&mut self, arg: &str) {
        if let Some(args) = self.args.as_mut() {
            args.push(arg.to_string());
        }
        else {
            self.args = Some(vec![arg.to_string()]);
        }
    }

    fn add_args(&mut self, args: &[&str]) {
        for arg in args {
            self.add_arg(arg)
        }
    }

    fn add_proc(&mut self, proc: Child) {
        if self.proc.is_none() {
            self.proc = Some(proc)
        }
        else {
            panic!("You tried to assign a process to a already running process")
        }
    }

    pub fn start(&mut self) -> Result<(), String>{
        let mut cmd = Command::new(self.path.as_path());
        if let Some(args) = self.args.as_ref() {
            cmd.args(args);
        }
        match cmd.spawn() {
            Ok(child) => self.add_proc(child),
            Err(e) => return Err(format!(
                "Failed to start {:?}, given error was: {e}",
                self.path.file_name().unwrap()
            )),
        }
        Ok(())
    }
}

pub fn parse_args(args: Vec<String>) -> Result<Vec<Program>, String> {
    const ARG_START: &str = "--start";
    const ARG_EXIT_ON: &str = "--exit";
    let mut arg_start_given: bool = false;
    let mut arg_exit_on_given: bool = false;

    let mut start_these: Vec<Program> = Vec::new();
    let mut exit_on_this: Vec<String> = Vec::new();

    let mut current_vec: &str = "None";
    for (index, arg) in args.iter().enumerate() {
        if index != 0 {
            if arg.as_str().starts_with("--") {
                match arg.as_str() {
                    ARG_START => {
                        current_vec = ARG_START;
                        arg_start_given = true;
                    }
                    ARG_EXIT_ON => {
                        current_vec = ARG_EXIT_ON;
                        arg_exit_on_given = true;
                    }
                    _ => return Err(format!("`{arg}` is not a valid argument.")),
                }
            } else {
                match current_vec {
                    ARG_START => {
                        let mut program = Program::default();
                        if arg.contains(".exe ") {
                            let mut split = arg.split_inclusive(".exe ").collect::<Vec<&str>>();
                            let args_str = split.pop().unwrap();
                            let args_split = args_str.split_whitespace().collect::<Vec<&str>>();
                            program.add_args(&args_split);

                            let str_path = split.pop().unwrap().trim();
                            program.path = get_path(str_path)?;
                        }
                        else {
                            program.path = get_path(arg)?;
                        }
                        start_these.push(program)
                    }
                    ARG_EXIT_ON => exit_on_this.push(arg.to_string()),
                    _ => return Err(format!("Argument `{arg}` did not start with `--`")),
                }
            }
        }
    }
    if !arg_start_given && !arg_exit_on_given {
        return Err("No arguments were given, this is probably a mistake".to_string());
    }
    if arg_start_given && start_these.is_empty() {
        return Err("--start was given but no argument to it were given".to_string());
    }
    if arg_exit_on_given {
        if !arg_start_given {
            return Err(
                "--exit was given but not --start, --exit uses the info given in --start"
                    .to_string(),
            );
        }
        if exit_on_this.len() == 1 {
            match_exit_with_starts(&exit_on_this[0], &mut start_these)?
        } else if exit_on_this.len() > 1 {
            return Err(format!(
                "--exit only accepts 1 argument but, {} were given.",
                exit_on_this.len()
            ));
        } else if exit_on_this.is_empty() {
            return Err("--exit was given but no argument to it were given".to_string());
        }
    }

    Ok(start_these)
}
fn is_exit_match(exit: &str, start: &mut Program) -> Result<bool, String> {
    let path = start.path.as_path();
    let f_name = match path.file_name() {
        None => return Err(format!("`{start:?}` does not have a correct file name.")),
        Some(f) => match f.to_str() {
            None => return Err(format!("`{start:?}` contains invalid unicode character.")),
            Some(s) => s,
        },
    };
    start.exit_on_this = f_name == exit;
    Ok(f_name == exit)
}

fn match_exit_with_starts(exit: &str, starts: &mut [Program]) -> Result<(), String> {
    let mut is_match = false;
    for start in starts {
        let tmp_match = is_exit_match(exit, start)?;
        if is_match && tmp_match {
            return Err(
                "The --start arguments got two or more matching with the --exit".to_string(),
            );
        } else if tmp_match {
            is_match = tmp_match
        }
    }
    if !is_match {
        return Err(format!(
            "`{}` does not match with any argument in --start",
            exit
        ));
    }
    Ok(())
}

fn get_path(s: &str) -> Result<PathBuf, String> {
    let path = Path::new(&s);
    if !path.is_file() {
        return Err(format!(
            "The given path does not point to a file: `{}`",
            path.display()
        ));
    }
    match path.extension() {
        None => {
            return Err(format!(
                "Something is wrong with the extension of the given file: `{:?}`",
                path.file_name().unwrap()
            ))
        }
        Some(extension) => {
            if extension != "exe" {
                return Err(format!(
                    "The given file is not a .exe: `{:?}`",
                    path.file_name().unwrap()
                ));
            }
        }
    }
    Ok(path.to_path_buf())
}
