use std::fmt::{Display, Formatter, Result as FmtResult};

/// HTTP status codes as a Rust enum with numeric discriminants.
///
/// The `#[repr(u16)]` isn't needed here because we cast with `as u16` explicitly,
/// but the enum discriminant syntax (`Ok = 200`) lets us write `StatusCode::Ok as u16`
/// to get the numeric code -- used when formatting the HTTP response line.
#[derive(Clone, Copy, Debug)]
pub enum StatusCode {
    Ok = 200,
    BadRequest = 400,
    NotFound = 404,
    InternalServerError = 500,
}

impl StatusCode {
    /// Returns the human-readable phrase for the status line (RFC 7231, Section 6).
    /// For example: "HTTP/1.1 404 Not Found" -- "Not Found" comes from here.
    pub fn reason_phrase(&self) -> &str {
        match self {
            Self::Ok => "Ok",
            Self::BadRequest => "Bad Request",
            Self::NotFound => "Not Found",
            Self::InternalServerError => "Internal Server Error",
        }
    }
}

impl Display for StatusCode {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "{}", *self as u16)
    }
}
