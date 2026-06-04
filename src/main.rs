use clap::{CommandFactory, Parser};
use once_cell::sync::Lazy;
use regex::Regex;
use std::{
    error::Error,
    fmt,
    io::{self, BufRead, Write},
    process::ExitCode,
};
use tap::Pipe;

static BRACKETED_IPV6: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^\[(?P<ip>[^\]]+)\](?::\d+)?$").unwrap());

static IPV4_WITH_OPTIONAL_PORT: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^(?P<ip>(?:\d{1,3}\.){3}\d{1,3})(?::\d+)?$").unwrap());

static PLAIN_IPV6: Lazy<Regex> = Lazy::new(|| Regex::new(r"^(?P<ip>[0-9A-Fa-f:]+)$").unwrap());

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
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ReadStdin(e) => write!(f, "failed to read stdin: {}", e),
        }
    }
}

impl Error for AppError {}

fn extract_ip_from_address(address: &str) -> String {
    address.trim().pipe(|address| {
        BRACKETED_IPV6
            .captures(address)
            .and_then(|caps| caps.name("ip"))
            .map(|m| m.as_str().to_owned())
            .or_else(|| {
                IPV4_WITH_OPTIONAL_PORT
                    .captures(address)
                    .and_then(|caps| caps.name("ip"))
                    .map(|m| m.as_str().to_owned())
            })
            .or_else(|| {
                PLAIN_IPV6
                    .captures(address)
                    .and_then(|caps| caps.name("ip"))
                    .map(|m| m.as_str().to_owned())
            })
            .unwrap_or_else(|| address.to_owned())
    })
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
        .pipe(|result| println!("{result}"));
    Ok(())
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
