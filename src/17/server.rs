use anyhow::Result;
use nix::sys::signal::{sigaction, SaFlags, SigAction, SigHandler, SigSet, Signal};
use std::env;
use std::io;
use std::io::prelude::*;
use std::io::{BufReader, BufWriter};
use std::process::exit;

fn install_signal_handlers() {
    trap_signal(Signal::SIGPIPE, signal_exit);
}

fn trap_signal(sig: Signal, handler: extern "C" fn(i32)) -> Result<()> {
    let act = SigAction::new(
        SigHandler::Handler(handler),
        SaFlags::SA_RESTART,
        SigSet::empty(),
    );

    unsafe { sigaction(sig, &act) }?;

    Ok(())
}

extern "C" fn signal_exit(signum: i32) {
    println!("exit by signal {}", signum);
}

struct HTTPHeaderField {
    name: String,
    value: String,
    next: Box<HTTPHeaderField>,
}

struct HTTPRequest {
    protocol_minor_version: i32,
    method: String,
    path: String,
    header: HTTPHeaderField,
    body: String,
    length: i64,
}

fn service<R>(
    buf_in: BufReader<R>,
    buf_out: BufWriter<std::io::StdoutLock>,
    path: &str,
) -> Result<()> {
    let req: HTTPRequest;
    Ok(())
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: {} <docroot>", &args[0]);
        exit(1);
    }

    install_signal_handlers();

    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut buf_in = BufReader::new(stdin.lock());
    let mut buf_out = BufWriter::new(stdout.lock());

    service(buf_in, buf_out, &args[1])?;

    Ok(())
}
