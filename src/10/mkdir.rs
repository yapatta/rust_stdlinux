use getopts::Options;
use std::env;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut opts = Options::new();
    opts.optflag("h", "help", "print this help menu");
    opts.optflag("p", "", "create directory recursively");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => {
            panic!(f.to_string())
        }
    };

    if matches.opt_present("h") {
        println!("Usage: {:?} [-p] [FILE...]", &args[0]);
        process::exit(0);
    }

    let opt_p = matches.opt_present("p");

    for i in 0..matches.free.len() {
        if opt_p {
            if let Err(why) = fs::create_dir_all(&matches.free[i]) {
                eprintln!("{:?}: {:?}", &matches.free[i], why.to_string());
                continue;
            }
        } else {
            if let Err(why) = fs::create_dir(&matches.free[i]) {
                eprintln!("{:?}: {:?}", &matches.free[i], why.to_string());
                continue;
            }
        }

        let mut perms = fs::metadata(&matches.free[i]).unwrap().permissions();
        perms.set_mode(0o777);
        if let Err(why) = fs::set_permissions(&args[i], perms) {
            eprintln!("{:?}: {:?}", &matches.free[i], why.to_string());
            continue;
        }
    }

    process::exit(0);
}
