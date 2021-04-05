use std::env;
use std::fs;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        eprintln!("{:?}: wrong arguments", &args[0]);
        process::exit(1);
    }

    if let Err(why) = fs::hard_link(&args[1], &args[2]) {
        eprintln!("{:?}: {:?}", &args[1], why.to_string());
    }

    process::exit(0);
}
