# Extract Field From Address

Extract the IP or port from an address string.

## Features

- Default output is the IP field
- `--ip` explicitly extracts IP
- `-p` or `--port` extracts port
- Use `-` as the positional argument to read one line from stdin
- Supports:
  - `IPv4`
  - `IPv4:port`
  - `IPv6`
  - `[IPv6]:port`

## Examples

```bash
extract-field-from-address 1.2.3.4:80
# 1.2.3.4
```

```bash
extract-field-from-address 1.2.3.4:80 --port
# 80
```

```bash
extract-field-from-address [2001:1a::1]:443 --ip
# 2001:1a::1
```

```bash
echo '10.0.0.1:8080\n' | extract-field-from-address -
# 10.0.0.1
```

## Build & Install with Cargo

```bash
cargo install --path .
```
