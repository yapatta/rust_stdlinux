use std::env;
use std::fs::File;
use std::io::{self, Read, Write};
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        // eprintln!("{:?}: file name not given", args[0]);
        // process::exit(1);
        do_cat_stdin();
    } else {
        for i in 1..args.len() {
            do_cat(&args[i]);
        }
    }

    process::exit(0);
}

const BUFFER_SIZE: usize = 2048;

fn do_cat_stdin() {
    let mut buffer = [0; BUFFER_SIZE];
    let mut befn: usize = 0;

    loop {
        let n = match io::stdin().read(&mut buffer) {
            Ok(len) => len,
            Err(_) => panic!("couldn't read stdin"),
        };

        // if n < befn, need to erase [n, befn)
        for i in n..befn {
            buffer[i] = 0;
        }

        io::stdout().write(&buffer).unwrap();

        befn = n;
    }
}

// open file associated with filepath, and then export the contents into stdout
fn do_cat(path: &str) {
    let mut fd = match File::open(path) {
        Ok(file) => file,
        Err(why) => panic!("couln't open {}: {}", path, why.to_string()),
    };

    let mut buffer = [0; BUFFER_SIZE];
    let mut befn: usize = 0;

    loop {
        let n = match fd.read(&mut buffer) {
            Ok(len) => len,
            Err(why) => panic!("couldn't read file {}: {}", path, why.to_string()),
        };

        if n == 0 {
            break;
        }

        // if n < befn, need to erase [n, befn)
        for i in n..befn {
            buffer[i] = 0;
        }

        io::stdout().write(&buffer).unwrap();

        befn = n;
    }

    // file goes out of scope, and automatically file will be closed
}
