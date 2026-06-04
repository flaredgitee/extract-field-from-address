# Extract IP From Address

A small Rust CLI utility to extract the IP portion
from address strings that may include ports.
Supports IPv4 and IPv6 (bracketed) formats.

## Quick Start

### Run from stdin

```shell
echo "192.168.1.1:8080" | cargo run --quiet
```

### Build & Install with Cargo

```shell
cargo install --path .
```

## Notes & Edge Cases

- Bracketed IPv6 with port (`[ipv6]:port`) is supported
  and returns the IPv6 without brackets.
- Plain IPv6 without brackets is supported only when there is no trailing port
  (for example `2001:db8::1`).
- IPv4 with numeric port (`x.x.x.x:port`) is supported.
- Invalid or unsupported inputs are returned unchanged.
- Input read failures are reported as errors.

## License

MIT or Apache-2.0
