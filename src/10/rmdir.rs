use getopts::Options;
use std::env;
use std::fs;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut opts = Options::new();
    opts.optflag("h", "help", "print this help menu");
    opts.optflag("r", "", "remove directory recursively");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => {
            panic!(f.to_string())
        }
    };

    if matches.opt_present("h") {
        println!("Usage: {:?} [-r] [FILE...]", &args[0]);
        process::exit(0);
    }

    let opt_p = matches.opt_present("r");

    for i in 0..matches.free.len() {
        if opt_p {
            if let Err(why) = fs::remove_dir_all(&matches.free[i]) {
                eprintln!("{:?}: {:?}", &matches.free[i], why.to_string());
                continue;
            }
        } else {
            if let Err(why) = fs::remove_dir(&matches.free[i]) {
                eprintln!("{:?}: {:?}", &matches.free[i], why.to_string());
                continue;
            }
        }
    }

    process::exit(0);
}
