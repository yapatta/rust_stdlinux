use nix::sys::signal;
use nix::sys::signal::{sigaction, SaFlags, SigAction, SigHandler, SigSet};
use nix::unistd::pause;
use std::process::exit;

extern "C" fn sigint_handler(signum: i32) {
    println!("sigint_handler({}) is spawned", signum);
    exit(1);
}

fn main() {
    let sa = SigAction::new(
        SigHandler::Handler(sigint_handler),
        SaFlags::SA_RESTART,
        SigSet::empty(),
    );

    unsafe { sigaction(signal::SIGINT, &sa) }.unwrap();

    loop {
        pause();
    }
}
