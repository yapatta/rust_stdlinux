use regex::Regex;
use std::env;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::io::{BufReader, BufWriter};
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("no pattern");
        process::exit(1);
    }

    let re = Regex::new(&args[1]).unwrap();

    if args.len() == 2 {
        let stdin = io::stdin();
        let mut buf_in = BufReader::new(stdin.lock());
        do_grep(&re, &mut buf_in);
    } else {
        for i in 2..args.len() {
            let f = match File::open(&args[i]) {
                Ok(file) => file,
                Err(why) => panic!("couln't open {}: {}", &args[i], why.to_string()),
            };

            let mut buf_file = BufReader::new(f);

            do_grep(&re, &mut buf_file);
        }
    }
}

fn do_grep(re: &Regex, f: &mut dyn BufRead) {
    let stdout = io::stdout();
    let mut buf_out = BufWriter::new(stdout.lock());
    loop {
        let mut buf_in_string = String::new();
        let buf_len = match f.read_line(&mut buf_in_string) {
            Ok(l) => l,
            Err(why) => panic!("error while reading file: {}", why.to_string()),
        };
        if buf_len == 0 {
            break;
        }

        if re.is_match(&buf_in_string) {
            buf_out.write(buf_in_string.as_bytes()).unwrap();
        }
    }
}
