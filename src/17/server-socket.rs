use anyhow::Result;
use chrono::{DateTime, Utc};
use env_logger;
use getopts;
use getopts::Options;
use libc::_exit;
use nix::fcntl::{open, OFlag};
use nix::sys::signal::{sigaction, SaFlags, SigAction, SigHandler, SigSet, Signal};
use nix::sys::stat::Mode;
use nix::unistd::{chdir, dup2, fork, setsid, ForkResult};
use std::env;
use std::fmt;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::io::{BufReader, BufWriter};
use std::net::{TcpListener, TcpStream, ToSocketAddrs};
use std::process::exit;
use std::str::FromStr;

const MAX_REQUEST_BODY_LENGTH: i64 = 10_000;
const HTTP_MINOR_VERSION: u32 = 1;
const SERVER_NAME: &str = "RustHTTP";
const SERVER_VERSION: &str = "0.0.1";

#[derive(Debug)]
pub enum CustomError {
    ParseError(String),
    TooLongRequestBodyError,
    NoAddressError(String),
}

impl std::error::Error for CustomError {}
impl fmt::Display for CustomError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CustomError::ParseError(s) => write!(f, "parse error on request line: {}", s),
            CustomError::TooLongRequestBodyError => write!(f, "too long request body"),
            CustomError::NoAddressError(s) => write!(f, "no address: {} does not exist", s),
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
            Err(_) => {
                return FileInfo {
                    path: path,
                    size: 0,
                    ok: false,
                };
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

// TODO: implement like OOP
fn respond_to(
    req: &HTTPRequest,
    buf_out: &mut BufWriter<&TcpStream>,
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

// TODO: 404
fn not_found(req: &HTTPRequest, buf_out: &mut BufWriter<&TcpStream>) -> Result<()> {
    output_common_header_fields(req, buf_out, "404 Not Found")?;
    Ok(())
}

// TODO: 501
fn not_implemented(req: &HTTPRequest, buf_out: &mut BufWriter<&TcpStream>) -> Result<()> {
    output_common_header_fields(req, buf_out, "501 Not Implemented")?;
    Ok(())
}

// TODO: 405
fn method_not_allowed(req: &HTTPRequest, buf_out: &mut BufWriter<&TcpStream>) -> Result<()> {
    output_common_header_fields(req, buf_out, "405 Method Not Allowed")?;
    Ok(())
}

// TODO: implement like OOP
fn do_file_response(
    req: &HTTPRequest,
    buf_out: &mut BufWriter<&TcpStream>,
    docroot: String,
) -> Result<()> {
    let info = FileInfo::new(docroot, &req.path);

    if !info.ok {
        not_found(req, buf_out)?;
        return Ok(());
    }

    output_common_header_fields(req, buf_out, "200 OK")?;
    write!(buf_out, "Content-Length: {}\r\n", info.size)?;
    // TODO: implement guess_content_type fn
    write!(buf_out, "Content-Type: {}\r\n", "text/plain")?;
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
    _req: &HTTPRequest,
    buf_out: &mut BufWriter<&TcpStream>,
    status: &str,
) -> Result<()> {
    write!(buf_out, "HTTP/1.{} {}\r\n", HTTP_MINOR_VERSION, status)?;

    let utc: DateTime<Utc> = Utc::now();
    write!(
        buf_out,
        "Date: {}\r\n",
        utc.format("%a, %d %b %Y %H:%M:%S GMT")
    )?;

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
    buf_in: &mut BufReader<&TcpStream>,
    buf_out: &mut BufWriter<&TcpStream>,
    path: &str,
) -> Result<()> {
    let req = read_request(buf_in)?;
    respond_to(&req, buf_out, path.to_string())?;
    Ok(())
}

fn read_request(buf_in: &mut BufReader<&TcpStream>) -> Result<HTTPRequest> {
    let mut req = HTTPRequest::new();
    read_request_line(buf_in, &mut req)?;

    while let Some(mut h) = read_header_field(buf_in) {
        h.next = req.header;
        req.header = Some(Box::new(h));
    }

    req.length = if let Some(l) = content_length(&req.header) {
        l
    } else {
        0
        // Err(CustomError::ParseError("no content length".to_string()))?;
    };

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

fn read_header_field(buf_in: &mut BufReader<&TcpStream>) -> Option<HTTPHeaderField> {
    let mut line = String::new();
    if let Some(n) = buf_in.read_line(&mut line).ok() {
        if n == 0 {
            return None;
        }
        let kv: Vec<&str> = line.split_whitespace().collect();

        if kv.len() >= 2 {
            let mut h = HTTPHeaderField::new();
            h.name = kv[0].to_string();
            h.value = kv[1].to_string();

            return Some(h);
        }
    }
    return None;
}

fn read_request_line(buf_in: &mut BufReader<&TcpStream>, req: &mut HTTPRequest) -> Result<()> {
    let mut line = String::new();
    let _ = buf_in.read_line(&mut line)?;
    line.remove(line.len() - 1);

    let args: Vec<&str> = line.split_whitespace().collect();

    if args.len() < 3 {
        return Err(From::from(CustomError::ParseError(line)));
    }
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

fn become_daemon() -> Result<()> {
    chdir("/")?;

    let nio = open("/dev/null", OFlag::O_RDWR, Mode::S_IRWXU)?;
    dup2(nio, 0)?;
    dup2(nio, 1)?;
    dup2(nio, 2)?;

    match unsafe { fork() }? {
        ForkResult::Parent { child, .. } => {
            let mut f = File::create("/home/yuzi/myprogramming/rust/rust_stdlinux/aba.txt")?;
            write!(f, "child pid: {}", child)?;
            f.flush()?;
            unsafe { _exit(0) };
        }
        ForkResult::Child => {
            setsid()?;
        }
    }
    Ok(())
}

fn listen_socket(port: String) -> Result<TcpListener> {
    let hostname = "localhost".to_string() + ":" + &port;
    if let Some(addr) = hostname.to_socket_addrs()?.next() {
        let listener = TcpListener::bind(addr)?;

        return Ok(listener);
    }

    return Err(From::from(CustomError::NoAddressError(hostname)));
}

fn server_main(listner: TcpListener, docroot: String) -> Result<()> {
    loop {
        let (socket, _addr) = listner.accept()?;

        match unsafe { fork() }? {
            ForkResult::Parent { child, .. } => {}
            ForkResult::Child => {
                let mut ins = BufReader::new(&socket);
                let mut outs = BufWriter::new(&socket);

                service(&mut ins, &mut outs, &docroot)?;

                exit(0);
            }
        }
    }
}
fn setup_environment(docroot: String, user: String, group: String) -> Result<()> {
    Ok(())
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    let mut debug_mode = false;
    let mut do_chroot = false;
    let mut user: String = String::new();
    let mut group: String = String::new();
    let mut port: String = String::new();

    let mut opts = Options::new();
    opts.optflag("h", "help", "print this help menu");
    opts.optflag("d", "debug", "debug mode");
    opts.optflag("c", "chroot", "change root");
    opts.optopt("u", "user", "user name", "NAME");
    opts.optopt("g", "group", "group name", "GROUP");
    opts.optopt("p", "port", "port number", "PORT");

    let matches = opts.parse(&args[1..])?;

    if matches.opt_present("h") {
        println!(
            "Usage: {} [--port=n] [--chroot --user=u --group=g] <docroot>",
            &args[0]
        );
        return Ok(());
    }

    if matches.opt_present("d") {
        debug_mode = true;
    };

    if matches.opt_present("c") {
        do_chroot = true;
    }

    if let Some(u) = matches.opt_str("u") {
        user = u;
    }

    if let Some(g) = matches.opt_str("g") {
        group = g;
    }

    if let Some(p) = matches.opt_str("p") {
        port = p;
    }

    if matches.free.is_empty() {
        eprintln!(
            "Usage: {} [--port=n] [--chroot --user=u --group=g] <docroot>",
            &args[0]
        );

        exit(1);
    }

    install_signal_handlers()?;

    let mut docroot = matches.free[0].clone();

    if do_chroot {
        setup_environment(docroot, user, group)?;
        docroot = "".to_string();
    }

    let listener = listen_socket(port)?;

    if !debug_mode {
        env::set_var("RUST_LOG", "info");
        env_logger::init();
        become_daemon()?;
    }

    server_main(listener, docroot)?;

    Ok(())
}
