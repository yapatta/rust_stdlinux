use nix::sys::wait::{waitpid, WaitStatus};
use nix::unistd::{execvp, fork, ForkResult};
use std::ffi::CString;
use std::io;
use std::io::prelude::*;
use std::io::{BufReader, BufWriter};
use std::process::exit;
use whoami;

fn main() {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut buf_in = BufReader::new(stdin.lock());
    let mut input_string = String::new();
    let mut buf_out = BufWriter::new(stdout.lock());

    loop {
        buf_out
            .write(format!("{}@{}$ ", whoami::username(), whoami::hostname()).as_bytes())
            .unwrap_or_else(|why| panic!("aborted: {}", why.to_string()));
        buf_out
            .flush()
            .unwrap_or_else(|why| panic!("aborted: {}", why.to_string()));

        buf_in
            .read_line(&mut input_string)
            .unwrap_or_else(|why| panic!("aborted: {}", why.to_string()));

        input_string.remove(input_string.len() - 1);

        let args: Vec<&str> = input_string.split_whitespace().collect();

        if args.len() == 0 {
            continue;
        }

        match unsafe { fork() }.unwrap_or_else(|why| panic!("fork failed: {}", why.to_string())) {
            ForkResult::Parent { child, .. } => {
                match waitpid(child, None)
                    .unwrap_or_else(|why| panic!("waitpid failed: {}", why.to_string()))
                {
                    WaitStatus::Exited(pid, status) => {
                        println!("child (PID={}) finished: exit, status={}", pid, status);
                    }
                    WaitStatus::Signaled(pid, status, _) => {
                        println!("child (PID={}) finished: signal, sig={}", pid, status);
                    }
                    _ => println!("abnoraml exit"),
                };
            }
            ForkResult::Child => {
                let command = CString::new(args[0].to_string()).unwrap();
                let command_args: Vec<CString> =
                    args.iter().map(|arg| CString::new(*arg).unwrap()).collect();

                match execvp(&command, &command_args) {
                    Ok(_) => exit(0),
                    Err(_) => {
                        eprintln!("{}: command not found", args[0]);
                        exit(-1);
                    }
                };
            }
        };

        input_string.clear();
    }
}
