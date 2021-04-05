use std::env;
use std::fs;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("{:?}: no arguments", &args[0]);
        process::exit(1);
    }

    for i in 1..args.len() {
        if let Err(why) = fs::remove_file(&args[i]) {
            eprintln!("{:?}: {:?}", &args[1], why.to_string());
        }
    }
    process::exit(0);
}
