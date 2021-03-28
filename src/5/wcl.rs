use std::env;
use std::fs::File;
use std::io::Read;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("{:?}: file name not given", args[0]);
        process::exit(1);
    } else {
        for i in 1..args.len() {
            println!("{:?}", do_wcl(&args[i]));
        }
    }

    process::exit(0);
}

const BUFFER_SIZE: usize = 2048;

// get the number of lines
fn do_wcl(path: &str) -> i32 {
    let mut fd = match File::open(path) {
        Ok(file) => file,
        Err(why) => panic!("couln't open {}: {}", path, why.to_string()),
    };

    let mut buffer = [0; BUFFER_SIZE];
    let mut line_num: i32 = 0;

    loop {
        let n = match fd.read(&mut buffer) {
            Ok(len) => len,
            Err(why) => panic!("couldn't read file {}: {}", path, why.to_string()),
        };

        if n == 0 {
            return line_num;
        }

        for i in 0..n {
            if buffer[i] == b'\n' {
                line_num += 1;
            }
        }
    }
    // file goes out of scope, and automatically file will be closed
}
