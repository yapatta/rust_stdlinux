use anyhow::Result;
use chrono::{DateTime, Utc};
use nix::sys::signal::{sigaction, SaFlags, SigAction, SigHandler, SigSet, Signal};
use std::env;
use std::fmt;
use std::fs;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::io::{BufReader, BufWriter};
use std::process;
use std::process::exit;
use std::str::FromStr;

const MAX_REQUEST_BODY_LENGTH: i64 = 10_000;
const HTTP_MINOR_VERSION: u32 = 1;
const SERVER_NAME: &str = "Rust Http";
const SERVER_VERSION: &str = "0.0.1";

#[derive(Debug)]
pub enum CustomError {
    ParseError(String),
    TooLongRequestBodyError,
}

impl std::error::Error for CustomError {}
impl fmt::Display for CustomError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CustomError::ParseError(s) => write!(f, "Parse Error: {}", s),
            CustomError::TooLongRequestBodyError => write!(f, "Too Long RequestBody Error"),
        }
    }
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
    header: Option<Box<HTTPHeaderField>>,
    body: String,
    length: i64,
}

impl HTTPRequest {
    pub fn new() -> HTTPRequest {
        HTTPRequest {
            protocol_minor_version: 0,
            method: String::new(),
            path: String::new(),
            header: None,
            body: String::new(),
            length: 0,
        }
    }
}

struct FileInfo {
    path: String,
    size: u64,
    ok: bool,
}

impl FileInfo {
    pub fn new(docroot: String, urlpath: &str) -> FileInfo {
        let path = docroot + "/" + &urlpath;

        let st = match fs::metadata(&path) {
            Ok(st) => st,
            Err(why) => {
                eprintln!("{:?}: {:?}", &path, why.to_string());
                process::exit(1);
            }
        };

        if !st.is_file() {
            return FileInfo {
                path: path,
                size: 0,
                ok: false,
            };
        }

        FileInfo {
            path: path,
            size: st.len(),
            ok: true,
        }
    }
}

fn respond_to(
    req: &HTTPRequest,
    buf_out: &mut BufWriter<io::StdoutLock>,
    docroot: String,
) -> Result<()> {
    if req.method == "GET" {
        do_file_response(req, buf_out, docroot)?;
    } else if req.method == "HEAD" {
        do_file_response(req, buf_out, docroot)?;
    } else if req.method == "POST" {
        method_not_allowed(req, buf_out)?;
    } else {
        not_implemented(req, buf_out)?;
    }

    Ok(())
}

fn not_implemented(req: &HTTPRequest, buf_out: &mut BufWriter<io::StdoutLock>) -> Result<()> {
    Ok(())
}

fn method_not_allowed(req: &HTTPRequest, buf_out: &mut BufWriter<io::StdoutLock>) -> Result<()> {
    Ok(())
}

fn do_file_response(
    req: &HTTPRequest,
    buf_out: &mut BufWriter<io::StdoutLock>,
    docroot: String,
) -> Result<()> {
    let info = FileInfo::new(docroot, &req.path);

    if !info.ok {
        // not_found(req, buf_out)?;
        return Ok(());
    }

    output_common_header_fields(req, buf_out, "200 OK")?;
    write!(buf_out, "Content-Length: {}\r\n", info.size)?;
    // TODO: implement guess_content_type fn
    write!(buf_out, "Content-Type: {}\r\n", "text/html")?;
    write!(buf_out, "\r\n")?;

    if req.method != "HEAD" {
        let file = File::open(info.path)?;
        let mut file_buf = BufReader::new(file);
        let mut file_string = String::new();
        file_buf.read_to_string(&mut file_string)?;

        buf_out.write_all(file_string.as_bytes())?;
    }

    buf_out.flush()?;

    Ok(())
}

fn output_common_header_fields(
    req: &HTTPRequest,
    buf_out: &mut BufWriter<io::StdoutLock>,
    status: &str,
) -> Result<()> {
    write!(buf_out, "HTTP/1.{} {}\r\n", HTTP_MINOR_VERSION, status)?;

    let utc: DateTime<Utc> = Utc::now();
    write!(buf_out, "Date: {}", utc.format("%a, %d %b %Y %H:%M:%S GMT"))?;

    write!(buf_out, "Server: {}/{}\r\n", SERVER_NAME, SERVER_VERSION)?;
    write!(buf_out, "Connection: close\r\n")?;
    Ok(())
}

fn install_signal_handlers() -> Result<()> {
    trap_signal(Signal::SIGPIPE, signal_exit)?;

    Ok(())
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

fn service(
    buf_in: &mut BufReader<io::StdinLock>,
    buf_out: &mut BufWriter<io::StdoutLock>,
    path: &str,
) -> Result<()> {
    let req = read_request(buf_in)?;
    respond_to(&req, buf_out, path.to_string())?;
    Ok(())
}

fn read_request(buf_in: &mut BufReader<io::StdinLock>) -> Result<(HTTPRequest)> {
    let mut req = HTTPRequest::new();
    read_request_line(buf_in, &mut req)?;

    while let Some(mut h) = read_header_field(buf_in) {
        h.next = req.header;
        req.header = Some(Box::new(h));
    }

    if let Some(l) = content_length(&req.header) {
        req.length = l;
    } else {
        Err(CustomError::ParseError("no content length".to_string()))?;
    }

    if req.length != 0 {
        if req.length > MAX_REQUEST_BODY_LENGTH {
            Err(CustomError::TooLongRequestBodyError)?;
        }

        let mut body = String::with_capacity(req.length as usize);
        buf_in.read_to_string(&mut body)?;
        req.body = body;
    }

    Ok(req)
}

fn content_length(h: &Option<Box<HTTPHeaderField>>) -> Option<i64> {
    if let Some(kv) = h {
        if kv.name == "Content-Length" {
            return kv.value.parse::<i64>().ok();
        } else {
            return content_length(&kv.next);
        }
    }
    return None;
}

fn read_header_field(buf_in: &mut BufReader<io::StdinLock>) -> Option<HTTPHeaderField> {
    let mut line = String::new();
    if let Some(n) = buf_in.read_line(&mut line).ok() {
        if n == 0 {
            return None;
        }
        let kv: Vec<&str> = line.split_whitespace().collect();

        let mut h = HTTPHeaderField::new();
        h.name = kv[0].to_string();
        h.value = kv[1].to_string();

        return Some(h);
    }
    return None;
}

fn read_request_line(buf_in: &mut BufReader<io::StdinLock>, req: &mut HTTPRequest) -> Result<()> {
    let mut line = String::new();
    let _ = buf_in.read_line(&mut line)?;
    line.remove(line.len() - 1);

    let args: Vec<&str> = line.split_whitespace().collect();

    let method = args[0].to_uppercase();
    let path = args[1].to_string();
    let protocol = args[2].to_string();

    if !protocol.starts_with("HTTP/1.") {
        // 本当は Err(CustomError::ParseError(protocol))? と書きたかった...
        return Err(From::from(CustomError::ParseError(protocol)));
    }
    let protocol_minor_version: i32 = FromStr::from_str(&protocol[protocol.len() - 1..])?;

    req.protocol_minor_version = protocol_minor_version;
    req.method = method;
    req.path = path;

    Ok(())
}
fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: {} <docroot>", &args[0]);
        exit(1);
    }

    install_signal_handlers()?;

    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut buf_in = BufReader::new(stdin.lock());
    let mut buf_out = BufWriter::new(stdout.lock());

    service(&mut buf_in, &mut buf_out, &args[1])?;

    Ok(())
}
