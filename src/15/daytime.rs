use anyhow::Result;
use std::env;
use std::error::Error;
use std::fmt;
use std::fmt::Display;
use std::io::prelude::*;
use std::io::BufReader;
use std::net::{TcpStream, ToSocketAddrs};
use std::process::exit;

#[derive(Debug)]
struct NameResolutionError<'a> {
    hostname: &'a str,
}

impl Display for NameResolutionError<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "NameResolutionError: {}", self.hostname)
    }
}

impl Error for NameResolutionError<'_> {}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    let host = if args.len() > 1 {
        &args[1]
    } else {
        "localhost"
    };

    let mut stream = open_connection(host, "13")?;

    let mut buf = String::new();
    stream.read_line(&mut buf)?;
    print!("{}", buf);

    Ok(())
}

fn open_connection(host: &str, service: &str) -> Result<BufReader<TcpStream>> {
    // getaddrinfo
    let hostname = host.to_string() + ":" + service;
    let addr = match hostname.to_socket_addrs()?.next() {
        Some(addr) => addr,
        None => {
            exit(1);
            /* let err = NameResolutionError {
                hostname: format!("something wrong with name resolution: {}", hostname),
            };
            */
            // return err.with_context();
        }
    };
    // connect tcp
    let stream = TcpStream::connect(addr)?;

    Ok(BufReader::new(stream))
}
