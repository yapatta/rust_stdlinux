use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::io::{self};
use std::io::{BufReader, BufWriter};
use std::process;

// syscall version
fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        let mut buf_in = BufReader::new(io::stdin());
        do_cat(&mut buf_in);
    } else {
        for i in 1..args.len() {
            let f = match File::open(&args[i]) {
                Ok(file) => file,
                Err(why) => panic!("couln't open {}: {}", &args[i], why.to_string()),
            };
            let mut buf_in = BufReader::new(f);
            do_cat(&mut buf_in);
        }
    }

    process::exit(0);
}

const BUFFER_SIZE: usize = 2048;

// open file associated with filepath, and then export the contents into stdout
fn do_cat(buf_in: &mut dyn BufRead) {
    let mut buffer = [0; BUFFER_SIZE];
    let mut befn: usize = 0;
    let mut buf_out = BufWriter::new(io::stdout());

    loop {
        let n = match buf_in.read(&mut buffer) {
            Ok(len) => len,
            Err(why) => panic!("couldn't read file: {}", why.to_string()),
        };

        if n == 0 {
            break;
        }

        // if n < befn, need to erase [n, befn)
        for i in n..befn {
            buffer[i] = 0;
        }

        buf_out.write(&buffer).unwrap();

        befn = n;
    }

    // file goes out of scope, and automatically file will be closed
}
