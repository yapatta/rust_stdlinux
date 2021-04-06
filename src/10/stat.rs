use std::env;
use std::fs;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("{:?}: wrong arguments", &args[0]);
        process::exit(1);
    }

    let st = match fs::metadata(&args[1]) {
        Ok(st) => st,
        Err(why) => {
            eprintln!("{:?}: {:?}", &args[1], why.to_string());
            process::exit(1);
        }
    };
    println!("{:?}", st);

    process::exit(0);
}

g
