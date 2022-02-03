

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut sleep_time = 1;
    if args.len() != 1 {
        sleep_time = if let Ok(t) = args[1].parse() {
            t
        }
        else {
            sleep_time
        }
    }

    println!("Sleeping for {sleep_time} sec");
    std::thread::sleep(std::time::Duration::new(sleep_time, 0));
    println!("Sleeping for {sleep_time} sec");

}