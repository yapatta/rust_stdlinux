use std::env;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::io::{BufReader, BufWriter};
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("{:?}: file name not given", args[0]);
        process::exit(1);
    }

    let nlines: i32 = args[1].parse().unwrap();

    if args.len() == 2 {
        let mut buf_file = BufReader::new(io::stdin());
        do_head(&mut buf_file, nlines);
    } else {
        for i in 2..args.len() {
            let f = match File::open(&args[i]) {
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
