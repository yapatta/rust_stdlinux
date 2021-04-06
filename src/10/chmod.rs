use std::env;
use std::fs;
use std::io;
use std::os::unix::fs::PermissionsExt;
use std::process;

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("{:?}: no mode given", &args[0]);
        process::exit(1);
    }

    let mode: u32 = u32::from_str_radix(&args[1], 8).unwrap();

    for i in 2..args.len() {
        let mut perms = fs::metadata(&args[i])?.permissions();
        perms.set_mode(mode);
        fs::set_permissions(&args[i], perms)?;
    }

    process::exit(0);
}
