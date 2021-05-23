use anyhow::Result;
use nix::sys::signal::{sigaction, SaFlags, SigAction, SigHandler, SigSet, Signal};
use std::env;
use std::fmt;
use std::io;
use std::io::prelude::*;
use std::io::{BufReader, BufWriter};
use std::process::exit;
use std::str::FromStr;

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
    next: Option<Box<HTTPHeaderField>>,
}

impl HTTPHeaderField {
    pub fn new() -> HTTPHeaderField {
        HTTPHeaderField {
            name: String::new(),
            value: String::new(),
            next: None,
        }
    }
}

struct HTTPRequest {
    protocol_minor_version: i32,
    method: String,
    path: String,
    header: Box<HTTPHeaderField>,
    body: String,
    length: i64,
}

impl HTTPRequest {
    pub fn new() -> HTTPRequest {
        HTTPRequest {
            protocol_minor_version: 0,
            method: String::new(),
            path: String::new(),
            header: Box::new(HTTPHeaderField::new()),
            body: String::new(),
            length: 0,
        }
    }
}

fn service<R>(
    buf_in: BufReader<std::io::StdinLock>,
    buf_out: BufWriter<std::io::StdoutLock>,
    path: &str,
) -> Result<()> {
    let mut req = HTTPRequest::new();
    read_request(buf_in, &mut req)?;
    respond_to(req, buf_out, path)?;
    Ok(())
}

fn read_request(buf_in: BufReader<std::io::StdinLock>) -> Result<(HTTPRequest)> {
    let mut req = HTTPRequest::new();
    read_request_line(buf_in, &mut req)?;

    Ok(req)
}

fn read_request_line(buf_in: BufReader<std::io::StdinLock>, req: &mut HTTPRequest) -> Result<()> {
    let mut line = String::new();
    let _ = buf_in.read_line(&mut line)?;
    line.remove(line.len() - 1);

    let args: Vec<&str> = line.split_whitespace().collect();

    let method = args[0].to_uppercase();
    let path = args[1].to_string();
    let protocol = args[2].to_string();

    if !protocol.starts_with("HTTP/1.") {
        Err(CustomError::ParseError(protocol))?;
    }
    let protocol_minor_version: i32 = FromStr::from_str(&protocol[protocol.len() - 1..])?;

    req.protocol_minor_version = protocol_minor_version;
    req.method = method;
    req.path = path;

    Ok(())
}

#[derive(Debug)]
pub enum CustomError {
    ParseError(String),
}

impl std::error::Error for CustomError {}
impl fmt::Display for CustomError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CustomError::ParseError(s) => write!(f, "Parse Error: {}", s),
        }
    }
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
