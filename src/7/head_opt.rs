use getopts;
use getopts::Options;
use std::env;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::io::{BufReader, BufWriter};
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut opts = Options::new();
    opts.optflag("h", "help", "print this help menu");
    opts.optopt("n", "lines", "number of lines", "NAME");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => panic!(f.to_string()),
    };

    if matches.opt_present("h") {
        println!("Usage: {:?} [-n LINES] [FILE...]", &args[0]);
        process::exit(0);
    }

    // --line tmp でtmpがない場合既に19行目でpanicしている？
    let nlines: i32 = match matches.opt_str("n") {
        Some(n) => n.parse().unwrap(),
        None => {
            println!("Usage: {:?} [-n LINES] [FILE...]", &args[0]);
            process::exit(1);
        }
    };

    if matches.free.is_empty() {
        let mut buf_file = BufReader::new(io::stdin());
        do_head(&mut buf_file, nlines);
    } else {
        for i in 0..matches.free.len() {
            let f = match File::open(&matches.free[i]) {
                Ok(file) => file,
                Err(why) => panic!("couln't open {}: {}", &args[i], why.to_string()),
            };
            let mut buf_file = BufReader::new(f);

            do_head(&mut buf_file, nlines);
        }
    }

    process::exit(0);
}

fn do_head(f: &mut dyn BufRead, mut nlines: i32) {
    let mut buf_out = BufWriter::new(io::stdout());
    for byte in f.bytes() {
        let c = byte.unwrap();
        buf_out.write(&[c]).unwrap();
        if c == b'\n' {
            nlines -= 1;
            if nlines == 0 {
                return;
            }
        }
    }
}
