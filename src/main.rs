use clap::{CommandFactory, Parser};
use std::{
    error::Error,
    fmt,
    io::{self, BufRead, Write},
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
    process::ExitCode,
    str::FromStr,
};

#[derive(Parser, Debug)]
#[command(version, about)]
#[command(long_about = r#"
Extract a field from an address string.

Use '-' as the positional argument to read one line from stdin.
Supported inputs:
- IPv4
- IPv4:port
- IPv6
- [IPv6]:port
"#)]
struct Cli {
    #[arg(
        value_name = "ADDRESS",
        help = "Address string, or '-' to read from stdin"
    )]
    address: String,

    #[arg(long, help = "Extract the IP field (default)", conflicts_with = "port")]
    ip: bool,

    #[arg(
        short = 'p',
        long,
        help = "Extract the port field",
        conflicts_with = "ip"
    )]
    port: bool,
}

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

#[derive(Debug)]
struct ParsedAddress {
    ip: String,
    port: Option<String>,
}

fn read_address_from_stdin() -> Result<String, AppError> {
    let line = io::stdin()
        .lock()
        .lines()
        .next()
        .ok_or_else(|| {
            AppError::ReadStdin(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "no input provided",
            ))
        })?
        .map_err(AppError::ReadStdin)?;

    let line = line.trim().to_owned();
    if line.is_empty() {
        return Err(AppError::InvalidInput("empty input".to_owned()));
    }

    Ok(line)
}

fn resolve_address(raw: &str) -> Result<String, AppError> {
    if raw == "-" {
        read_address_from_stdin()
    } else {
        let value = raw.trim().to_owned();
        if value.is_empty() {
            Err(AppError::InvalidInput("empty input".to_owned()))
        } else {
            Ok(value)
        }
    }
}

fn parse_address(address: &str) -> Result<ParsedAddress, AppError> {
    if let Some(rest) = address.strip_prefix('[') {
        let (ip_part, tail) = rest
            .split_once(']')
            .ok_or_else(|| AppError::InvalidInput(address.to_owned()))?;

        let ip = Ipv6Addr::from_str(ip_part)
            .map(|ip| ip.to_string())
            .map_err(|_| AppError::InvalidInput(address.to_owned()))?;

        let port = if tail.is_empty() {
            None
        } else if let Some(port) = tail.strip_prefix(':') {
            if !port.is_empty() && port.chars().all(|c| c.is_ascii_digit()) {
                Some(port.to_owned())
            } else {
                return Err(AppError::InvalidInput(address.to_owned()));
            }
        } else {
            return Err(AppError::InvalidInput(address.to_owned()));
        };

        return Ok(ParsedAddress { ip, port });
    }

    if let Ok(ip) = IpAddr::from_str(address) {
        return Ok(ParsedAddress {
            ip: ip.to_string(),
            port: None,
        });
    }

    if let Some((host, port)) = address.rsplit_once(':')
        && !host.is_empty()
        && !port.is_empty()
        && port.chars().all(|c| c.is_ascii_digit())
    {
        let ip = Ipv4Addr::from_str(host)
            .map(|ip| ip.to_string())
            .map_err(|_| AppError::InvalidInput(address.to_owned()))?;

        return Ok(ParsedAddress {
            ip,
            port: Some(port.to_owned()),
        });
    }

    Err(AppError::InvalidInput(address.to_owned()))
}

fn run(cli: Cli) -> Result<(), AppError> {
    let address = resolve_address(&cli.address)?;
    let parsed = parse_address(&address)?;

    let output = if cli.port {
        parsed
            .port
            .ok_or_else(|| AppError::InvalidInput(address.clone()))?
    } else {
        parsed.ip
    };

    println!("{output}");

    Ok(())
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    match run(cli) {
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

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    fn assert_parsed(address: &str, expected_ip: &str, expected_port: Option<&str>) {
        let parsed = parse_address(address).expect("address should parse");
        assert_eq!(parsed.ip, expected_ip);
        assert_eq!(parsed.port.as_deref(), expected_port);
    }

    #[test]
    fn parses_ipv4_without_port() {
        assert_parsed("1.2.3.4", "1.2.3.4", None);
    }

    #[test]
    fn parses_ipv4_with_port() {
        assert_parsed("1.2.3.4:80", "1.2.3.4", Some("80"));
    }

    #[test]
    fn parses_ipv6_without_port() {
        assert_parsed("2001:db8::1", "2001:db8::1", None);
    }

    #[test]
    fn parses_bracketed_ipv6_with_port() {
        assert_parsed("[2001:db8::1]:443", "2001:db8::1", Some("443"));
    }

    #[test]
    fn rejects_invalid_ipv4_port() {
        assert!(parse_address("1.2.3.4:abc").is_err());
    }

    #[test]
    fn rejects_unbracketed_ipv6_with_port_like_suffix() {
        const IP: &str = "2001:db8:0:0:0:0:0:1";
        assert!(parse_address(&format!("{IP}:443")).is_err());
        assert!(parse_address(&format!("[{IP}]:443")).is_ok());
    }

    #[test]
    fn rejects_invalid_bracketed_ipv6() {
        assert!(parse_address("[2001:db8::zz]:443").is_err());
    }

    #[test]
    fn cli_defaults_to_ip() {
        let cli = Cli::try_parse_from(["extract-field-from-address", "1.2.3.4"])
            .expect("cli should parse");
        assert!(!cli.ip);
        assert!(!cli.port);
        assert_eq!(cli.address, "1.2.3.4");
    }

    #[test]
    fn cli_parses_ip_flag() {
        let cli = Cli::try_parse_from(["extract-field-from-address", "1.2.3.4", "--ip"])
            .expect("cli should parse");
        assert!(cli.ip);
        assert!(!cli.port);
    }

    #[test]
    fn cli_parses_port_flag() {
        let cli = Cli::try_parse_from(["extract-field-from-address", "1.2.3.4:80", "-p"])
            .expect("cli should parse");
        assert!(!cli.ip);
        assert!(cli.port);
    }

    #[test]
    fn cli_rejects_conflicting_flags() {
        let err =
            Cli::try_parse_from(["extract-field-from-address", "1.2.3.4:80", "--ip", "--port"])
                .expect_err("cli should reject conflicting flags");

        assert!(err.to_string().contains("cannot be used with"));
    }
}
