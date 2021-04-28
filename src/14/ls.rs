use anyhow::Result;
use chrono::{DateTime, Local, TimeZone};
use nix::dir::Dir;
use nix::fcntl::OFlag;
use nix::sys::stat::{stat, Mode};
use nix::unistd::{Gid, Group, Uid, User};
use std::env;
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
            process::exit(1);
        }
    }

    process::exit(0);
}

fn do_ls(path: &str) -> Result<()> {
    let dir = Dir::open(path, OFlag::O_RDONLY, Mode::S_IROTH)?;
    for entry in dir.into_iter() {
        let entry = entry?;
        let filename = entry.file_name().to_str()?;
        let stat = stat(filename)?;

        if let Some(user) = User::from_uid(Uid::from_raw(stat.st_uid))? {
            if let Some(group) = Group::from_gid(Gid::from_raw(stat.st_gid))? {
                let dt: DateTime<Local> = Local.timestamp(stat.st_mtime, stat.st_mtime_nsec as u32);
                println!(
                    "{: <10} {: >6} {: >8} {}",
                    filename,
                    user.name,
                    group.name,
                    dt.format("%Y-%m-%d %H:%M:%S").to_string()
                );
            }
        }
    }

    Ok(())
}
