use nix::sys::wait::{waitpid, WaitStatus};
use nix::unistd::{execv, fork, ForkResult};
use std::env;
use std::ffi::CString;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        eprintln!("Usage: {} <command> <arg>", args[0]);
        process::exit(1);
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
            let path = CString::new(args[1].to_string()).unwrap();
            let command_arg = CString::new(args[2].to_string()).unwrap();
            execv(&path, &[path.clone(), command_arg])
                .unwrap_or_else(|why| panic!("execv failed: {}", why.to_string()));
        }
    };
}
