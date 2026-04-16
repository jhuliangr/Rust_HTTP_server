use std::str::FromStr;

/// All nine HTTP/1.1 methods defined in RFC 7231 and RFC 5789.
///
/// We implement `FromStr` so that a raw string like "GET" can be converted
/// to `Method::Get` using `.parse()` -- Rust's standard trait for parsing strings.
///
/// Clippy recommends PascalCase for enum variants even for acronyms.
/// This follows the Rust API Guidelines (C-CASE).
#[derive(Debug)]
pub enum Method {
    Get,
    Post,
    Put,
    Delete,
    Patch,
    Head,
    Options,
    Connect,
    Trace,
}

impl Method {
    /// Returns true for methods that typically carry a request body.
    /// Used by the request parser to decide whether to look for headers + body.
    pub fn expects_body(&self) -> bool {
        matches!(self, Method::Post | Method::Put | Method::Patch)
    }
}

impl FromStr for Method {
    type Err = MethodError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        match raw {
            "GET" => Ok(Self::Get),
            "POST" => Ok(Self::Post),
            "PUT" => Ok(Self::Put),
            "DELETE" => Ok(Self::Delete),
            "PATCH" => Ok(Self::Patch),
            "HEAD" => Ok(Self::Head),
            "OPTIONS" => Ok(Self::Options),
            "CONNECT" => Ok(Self::Connect),
            "TRACE" => Ok(Self::Trace),
            _ => Err(MethodError),
        }
    }
}

pub struct MethodError;
