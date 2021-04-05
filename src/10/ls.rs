use std::env;
use std::fs::read_dir;
use std::io;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("{:?}: no arguments", &args[0]);
        process::exit(1);
    }

    for i in 1..args.len() {
        if let Err(why) = do_ls(&args[i]) {
            eprintln!("{:?}: some errors", why.to_string());
        }
    }
}

fn do_ls(path: &str) -> io::Result<()> {
    for entry in read_dir(path)? {
        let entry = entry?;
        let filename = entry.file_name().into_string().unwrap();
        // use Display trait because Debug shows text including double quotation,
        println!("{}", filename);
    }

    Ok(())
}
