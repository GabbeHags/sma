use std::process;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut sleep_time = 1;
    if args.len() != 1 {
        sleep_time = if let Ok(t) = args[1].parse() {
            t
        } else {
            sleep_time
        }
    }
    let id = process::id();
    println!("[id: {id}] Sleeping for {sleep_time} sec");
    for t in 1..=sleep_time {
        println!("[id: {id}] Sleept for {t} sec");
        std::thread::sleep(std::time::Duration::new(1, 0));
    }
    println!("[id: {id}] Woken up after {sleep_time} sec");
}
