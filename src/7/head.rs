use std::env;
use std::io;
use std::io::prelude::*;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("{:?}: file name not given", args[0]);
        process::exit(1);
    }

    let nlines: i32 = args[1].parse().unwrap();
    do_head(io::stdin(), nlines);
    process::exit(0);
}

fn do_head(f: io::Stdin, mut nlines: i32) {
    let mut stdout = io::stdout();
    for byte in f.bytes() {
        let c = byte.unwrap();
        stdout.write(&[c]).unwrap();
        if c == b'\n' {
            nlines -= 1;
            if nlines == 0 {
                return;
            }
        }
    }
}
