use clap::{CommandFactory, Parser};
use std::{
    error::Error,
    fmt,
    io::{self, BufRead, Write},
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
    process::ExitCode,
    str::FromStr,
};
use tap::Pipe;

#[derive(Parser, Debug)]
#[command(
    name = "extract-ip",
    version,
    about = "Extract IP from an 'ip:port' like address string."
)]
#[command(long_about = r#"
Extract IP from an 'ip:port' like address string from stdin.
Support IPv4 and IPv6 formats, with or without ports.
"#)]
struct Cli {}

#[derive(Debug)]
enum AppError {
    ReadStdin(io::Error),
    InvalidInput(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ReadStdin(e) => write!(f, "failed to read stdin: {}", e),
            Self::InvalidInput(s) => write!(f, "invalid address input: {}", s),
        }
    }
}

impl Error for AppError {}

fn parse_plain_ip(address: &str) -> Result<String, AppError> {
    IpAddr::from_str(address)
        .map(|ip| ip.to_string())
        .map_err(|_| AppError::InvalidInput(address.to_owned()))
}

fn parse_ipv4_with_optional_port(address: &str) -> Result<String, AppError> {
    address
        .rsplit_once(':')
        .filter(|(host, port)| {
            host.parse::<Ipv4Addr>().is_ok()
                && !port.is_empty()
                && port.chars().all(|c| c.is_ascii_digit())
        })
        .map(|(host, _)| host.to_owned())
        .ok_or_else(|| AppError::InvalidInput(address.to_owned()))
}

fn parse_bracketed_ipv6(address: &str) -> Result<String, AppError> {
    address
        .strip_prefix('[')
        .and_then(|rest| rest.split_once(']'))
        .map(|(ip_part, tail)| {
            Ipv6Addr::from_str(ip_part)
                .map(|ip| (ip.to_string(), tail))
                .map_err(|_| AppError::InvalidInput(address.to_owned()))
        })
        .ok_or_else(|| AppError::InvalidInput(address.to_owned()))?
        .and_then(|(ip, tail)| {
            if tail.is_empty()
                || (tail.starts_with(':') && tail[1..].chars().all(|c| c.is_ascii_digit()))
            {
                Ok(ip)
            } else {
                Err(AppError::InvalidInput(address.to_owned()))
            }
        })
}

fn extract_ip_from_address(address: &str) -> Result<String, AppError> {
    let address = address.trim();

    parse_bracketed_ipv6(address)
        .or_else(|_| parse_ipv4_with_optional_port(address))
        .or_else(|_| parse_plain_ip(address))
}

fn read_address() -> Result<String, AppError> {
    io::stdin()
        .lock()
        .lines()
        .next()
        .ok_or_else(|| {
            AppError::ReadStdin(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "no input provided",
            ))
        })?
        .map_err(AppError::ReadStdin)
        .map(|s| s.trim().to_owned())
}

fn run() -> Result<(), AppError> {
    read_address()?
        .pipe_borrow(extract_ip_from_address)
        .map(|result| println!("{result}"))
}

fn main() -> ExitCode {
    let _cli = Cli::parse();

    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!(
                "Error processing input - {} - {}",
                std::any::type_name::<AppError>(),
                e
            );
            eprintln!();

            let mut cmd = Cli::command();
            let _ = cmd.print_help();
            let _ = io::stderr().write_all(b"\n");

            ExitCode::from(1)
        }
    }
}
