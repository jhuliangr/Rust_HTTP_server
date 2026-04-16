# Rust HTTP Server

A lightweight HTTP/1.1 server built **from scratch** in Rust using only the standard library -- no external crates or frameworks. This project demonstrates low-level networking, manual HTTP parsing, and idiomatic Rust patterns.

## Why This Project?

Building an HTTP server without dependencies is one of the best ways to deeply understand:

- **TCP networking** -- binding sockets, accepting connections, reading raw byte streams
- **HTTP protocol internals** -- parsing request lines, headers, query strings, and bodies
- **Rust's ownership model** -- lifetimes, borrowing, and zero-copy parsing with `&'buf str`
- **Trait-based polymorphism** -- the `Handler` trait abstracts request handling, making the server extensible
- **Error handling** -- custom error types with `From` conversions, `Display`, and `Debug` implementations
- **Security** -- filesystem path canonicalization to prevent directory traversal attacks

## Architecture

```
src/
├── main.rs               # Entry point -- spawns the server on a dedicated thread
├── server.rs             # TCP listener, connection loop, and Handler trait definition
├── website_handler.rs    # Concrete Handler: routes requests and serves static files
└── http/
    ├── mod.rs            # Public re-exports for the HTTP module
    ├── method.rs         # HTTP Method enum with FromStr parsing
    ├── request.rs        # Request struct with zero-copy TryFrom<&[u8]> parser
    ├── response.rs       # Response builder that writes directly to the TCP stream
    ├── status_code.rs    # StatusCode enum (200, 400, 404, 500) with Display
    └── query_string.rs   # Query string parser using HashMap with single/multi-value support

public/
├── index.html            # Homepage served on GET /
├── post.html             # Template for POST responses (uses {-} placeholder)
└── css/styles.css        # Basic stylesheet
```

## Key Implementation Details

### Zero-Copy Request Parsing

The `Request` struct borrows directly from the raw byte buffer using Rust lifetimes (`'buf`), avoiding unnecessary allocations during parsing:

```rust
pub struct Request<'buf> {
    path: &'buf str,
    query_string: Option<QueryString<'buf>>,
    method: Method,
    headers: &'buf str,
    body: &'buf str,
}
```

Parsing is implemented via `TryFrom<&'buf [u8]>`, which is the idiomatic Rust conversion trait for fallible transformations.

### Extensible Handler Trait

The server is decoupled from specific routing logic through the `Handler` trait:

```rust
pub trait Handler {
    fn handle_request(&mut self, request: &Request) -> Response;
    fn handle_bad_request(&mut self, err: &ParseError) -> Response { /* default impl */ }
}
```

This allows swapping in different handlers without modifying the server core.

### Directory Traversal Protection

File serving uses `fs::canonicalize()` to resolve the real path and verifies it stays within the `public/` directory, preventing attacks like `GET /../../etc/passwd`:

```rust
fn read_file(&self, file_path: &str) -> Option<String> {
    let full_path = format!("{}/{}", self.public_path, file_path);
    match fs::canonicalize(full_path) {
        Ok(canonical_path) => {
            if canonical_path.starts_with(&self.public_path) {
                fs::read_to_string(canonical_path).ok()
            } else {
                eprintln!("[SECURITY] Directory traversal attempt blocked: {}", file_path);
                None
            }
        }
        Err(_) => None,
    }
}
```

### Query String Multi-Value Support

The query string parser handles both single and repeated keys, storing them as `Single` or `Multiple` variants:

```rust
pub enum Value<'buf> {
    Single(&'buf str),
    Multiple(Vec<&'buf str>),
}
```

`?color=red&color=blue` produces `Value::Multiple(vec!["red", "blue"])`.

## API Endpoints

| Method | Path        | Description                                          |
|--------|-------------|------------------------------------------------------|
| GET    | `/`         | Serves `index.html`                                  |
| GET    | `/<file>`   | Serves any file from the `public/` directory         |
| POST   | `/`         | Returns `post.html` with the request body injected   |
| POST   | `/<other>`  | Returns a plain-text echo of the path and body       |

## Getting Started

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (stable toolchain)

### Build & Run

```bash
cargo build
cargo run
```

The server starts on **http://127.0.0.1:8080**.

### Test It

```bash
# GET request
curl http://127.0.0.1:8080/

# POST request with body
curl -X POST http://127.0.0.1:8080/ -d '{"message": "hello"}'

# Static file
curl http://127.0.0.1:8080/css/styles.css
```

### Configuration

| Environment Variable | Default                          | Description                     |
|----------------------|----------------------------------|---------------------------------|
| `PUBLIC_PATH`        | `<project_root>/public`          | Directory for serving static files |

## Concepts Demonstrated

| Rust Concept                 | Where It's Used                                       |
|------------------------------|-------------------------------------------------------|
| Lifetimes (`'buf`)           | `Request`, `QueryString` -- zero-copy parsing         |
| `TryFrom` / `FromStr` traits| Request parsing, Method parsing                       |
| Custom error types           | `ParseError`, `MethodError` with `From` conversions   |
| Trait objects / generics     | `Handler` trait, `impl Write` for response sending    |
| Pattern matching             | Request routing, error handling, query string building |
| `HashMap` with enums         | Query string multi-value storage                      |
| `fs::canonicalize`           | Security: directory traversal prevention              |
| Threads (`std::thread`)      | Server runs on a spawned thread                       |

## Tech Stack

- **Language:** Rust (2021 edition)
- **Dependencies:** None -- 100% standard library
- **Protocol:** HTTP/1.1

## License

MIT
