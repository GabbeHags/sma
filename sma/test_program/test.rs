use std::process;

/// First arg is the COMMAND, the rest is operands to the command.
///
/// COMMANDS:
///     WRITE <file> <file content>
///     SLEEP <time in seconds>
///         
fn main() {
    // COMMANDS
    const WRITE: &str = "WRITE";
    const SLEEP: &str = "SLEEP";

    let args: Vec<String> = std::env::args().collect();
    if args.len() <= 1 {
        panic!("To few arguments used");
    }

    match args[1].as_str() {
        WRITE => command_write(&args[2], &args[3]),
        SLEEP => command_sleep(args[2].parse().unwrap()),
        _ => panic!("Unknown COMMAND"),
    }
}

fn command_write(file_path: &str, file_content: &str) {
    todo!()
}

fn command_sleep(seconds: usize) {
    // let id = process::id();
    // println!("[id: {id}] Sleeping for {seconds} sec");
    for t in 1..=seconds {
        // println!("[id: {id}] Sleept for {t} sec");
        std::thread::sleep(std::time::Duration::new(1, 0));
    }
    // println!("[id: {id}] Woken up after {seconds} sec");
}
