use getopts::Options;
use std::collections::VecDeque;
use std::env;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::io::{BufReader, BufWriter};
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut opts = Options::new();
    opts.optopt("n", "lines", "print this help menu", "NAME");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => {
            panic!(f.to_string())
        }
    };

    let nlines: usize = match matches.opt_str("n") {
        Some(n) => n.parse().unwrap(),
        None => {
            println!("Usage: {:?} [-n LINES] [FILE...]", &args[0]);
            process::exit(1);
        }
    };

    for i in 0..matches.free.len() {
        do_tail(&matches.free[i], nlines)
    }

    process::exit(0);
}

fn do_tail(path: &str, n: usize) {
    let f = match File::open(path) {
        Ok(file) => file,
        Err(why) => panic!("couln't open {}: {}", path, why.to_string()),
    };
    let mut buf_f = BufReader::new(f);
    let mut buf_str = String::new();

    let mut tails: VecDeque<String> = VecDeque::new();

    loop {
        let num_bytes = buf_f
            .read_line(&mut buf_str)
            .unwrap_or_else(|why| panic!("error while reading file: {}", why.to_string()));

        if num_bytes == 0 {
            break;
        }

        if tails.len() == n {
            tails.pop_front().unwrap();
        }
        tails.push_back(buf_str.clone());
        buf_str.clear();
    }

    let stdout = io::stdout();
    let mut buf_out = BufWriter::new(stdout.lock());

    let all_lines = tails
        .into_iter()
        .fold(String::new(), |before, line| before + &line);

    buf_out
        .write_all(all_lines.as_bytes())
        .unwrap_or_else(|why| panic!("error while writing input string: {}", why.to_string()));
}
