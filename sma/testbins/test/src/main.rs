use std::borrow::BorrowMut;
/// First arg is the COMMAND, the rest is operands to the command.
///
/// COMMANDS:
///     DEBUG                       // Turns on debugging prints
///     WRITE <file> <file content> // Writes a file to <file> with the content <file content>
///     SLEEP <time in seconds>     // Sleeps for <time in seconds>
///     SPAWN <COMMANDs> stop       // Spawns a new process of it self and gives it commands
///                                    until stop is found
use std::cell::{Cell, RefCell};
use std::path::PathBuf;
use std::process::{self, Command};
use std::str::FromStr;

trait Runnable {
    fn parse_args<'a>(&self, all_args: &'a [String]) -> &'a [String] {
        &all_args[1..1 + self.num_args(all_args)]
    }
    fn run(&self, args: &[String]);
    fn num_args(&self, args: &[String]) -> usize;
    fn can_run(&self, args: &[String]) -> bool {
        args.len() >= self.num_args(args)
    }
}

struct Debug;
struct Write;
struct Sleep;
struct Spawn(Cell<Option<usize>>);

thread_local!(static DEBUG: RefCell<bool> = const{RefCell::new(false)});

macro_rules! log {
    ($($arg:expr),*) => {
        DEBUG.with(|cell| {
            if *cell.borrow() {
                print!("[pid: {}] ", process::id());
                println!($($arg),*)
            }
        })
    };
}

impl Runnable for Debug {
    fn run(&self, _args: &[String]) {
        DEBUG.with(|cell| *cell.borrow_mut() = true);
    }

    fn num_args(&self, _: &[String]) -> usize {
        0
    }
}

impl Runnable for Write {
    fn run(&self, args: &[String]) {
        let file_path = PathBuf::from_str(args[0].as_str()).unwrap();
        let file_content = args[1].clone();

        log!(
            "Writes to {} with content {file_content}",
            file_path.display()
        );
        std::fs::write(file_path.as_path(), file_content).unwrap();
    }

    fn num_args(&self, _: &[String]) -> usize {
        2
    }
}

impl Runnable for Sleep {
    fn run(&self, args: &[String]) {
        let seconds = args[0].parse().unwrap();
        log!("Sleeping for {seconds} sec");
        for _t in 1..=seconds {
            log!("Slept for {_t} sec");
            std::thread::sleep(std::time::Duration::new(1, 0));
        }
        log!("Woken up after {seconds} sec");
    }

    fn num_args(&self, _: &[String]) -> usize {
        1
    }
}
impl Runnable for Spawn {
    fn run(&self, args: &[String]) {
        log!("Spawning with commands: {:?}", args);
        Command::new(std::env::current_exe().unwrap())
            .args(&args[..args.len() - 1])
            .spawn()
            .unwrap();
    }

    fn num_args(&self, args: &[String]) -> usize {
        if let Some(num) = self.0.get() {
            num
        } else {
            let num = args
                .iter()
                .enumerate()
                .find(|(_, arg)| arg.as_str() == "stop")
                .expect("Could not find a `stop` command after a SPAWN")
                .0;
            self.0.set(Some(num));
            num
        }
    }
}

impl<S: AsRef<str>> From<S> for Box<dyn Runnable> {
    fn from(value: S) -> Self {
        const WRITE: &str = "WRITE";
        const SLEEP: &str = "SLEEP";
        const SPAWN: &str = "SPAWN";
        const DEBUG: &str = "DEBUG";

        match value.as_ref() {
            WRITE => Box::new(Write),
            SLEEP => Box::new(Sleep),
            SPAWN => Box::new(Spawn(Cell::new(None))),
            DEBUG => Box::new(Debug),
            unknown_command => panic!("Unknown COMMAND: {unknown_command}"),
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() <= 1 {
        panic!("To few arguments used");
    }
    let mut args_ref = &args[1..];
    let mut runnable: Box<dyn Runnable>;

    while !args_ref.is_empty() {
        runnable = (&args_ref[0]).into();
        if !runnable.can_run(args_ref) {
            break;
        }

        runnable.run(runnable.parse_args(args_ref));
        args_ref = &args_ref[1 + runnable.num_args(args_ref)..];
    }
}
