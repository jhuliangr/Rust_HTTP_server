use super::method::{Method, MethodError};
use super::QueryString;
use std::convert::TryFrom;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter, Result as FmtResult};
use std::str::{self, Utf8Error};

/// Represents a parsed HTTP request.
///
/// Uses Rust lifetimes to borrow directly from the raw TCP buffer (`'buf`),
/// achieving zero-copy parsing -- no heap allocations needed for path, headers, or body.
/// This is the same technique used by high-performance parsers like `httparse`.
#[derive(Debug)]
pub struct Request<'buf> {
    path: &'buf str,
    query_string: Option<QueryString<'buf>>,
    method: Method,
    headers: &'buf str,
    body: &'buf str,
}

impl<'buf> Request<'buf> {
    pub fn path(&self) -> &str {
        self.path
    }

    pub fn method(&self) -> &Method {
        &self.method
    }

    pub fn query_string(&self) -> Option<&QueryString<'_>> {
        self.query_string.as_ref()
    }

    pub fn headers(&self) -> &str {
        self.headers
    }

    pub fn body(&self) -> &str {
        self.body
    }
}

/// Parses raw bytes into a structured `Request`.
///
/// We implement `TryFrom` instead of a manual `parse()` method because it's the
/// idiomatic Rust trait for fallible conversions. This also lets us use the `?`
/// operator with our custom `ParseError` type thanks to the `From` implementations below.
///
/// An HTTP/1.1 request looks like:
/// ```text
/// GET /search?q=rust&lang=en HTTP/1.1\r\n
/// Host: example.com\r\n
/// Content-Type: application/json\r\n
/// \r\n
/// {"key": "value"}
/// ```
impl<'buf> TryFrom<&'buf [u8]> for Request<'buf> {
    type Error = ParseError;

    fn try_from(raw_bytes: &'buf [u8]) -> Result<Request<'buf>, Self::Error> {
        // Convert raw bytes to UTF-8. The `?` operator automatically converts
        // Utf8Error -> ParseError::Encoding via our From impl below.
        let raw_request = str::from_utf8(raw_bytes)?;

        // HTTP request line has 3 space-separated tokens: METHOD PATH PROTOCOL
        // Each call to `extract_next_token` returns (token, remaining_str).
        let (method_str, after_method) =
            extract_next_token(raw_request).ok_or(ParseError::Request)?;

        let (mut path, after_path) =
            extract_next_token(after_method).ok_or(ParseError::Request)?;

        let (protocol, remaining_after_request_line) =
            extract_next_token(after_path).ok_or(ParseError::Request)?;

        if protocol != "HTTP/1.1" {
            return Err(ParseError::Protocol);
        }

        // The `parse()` call triggers `FromStr` for Method, and `?` converts
        // MethodError -> ParseError::Method via our From impl.
        let method: Method = method_str.parse()?;

        // For requests with a body (POST, PUT, PATCH), split the remaining
        // content into headers and body sections.
        let (headers, body) = if method.expects_body() {
            split_headers_and_body(remaining_after_request_line).unwrap_or(("", ""))
        } else {
            ("", "")
        };

        // Extract query string if present: "/search?q=rust" -> path="/search", query="q=rust"
        // The query string borrows from the same buffer -- still zero-copy.
        let mut query_string = None;
        if let Some(query_separator_index) = path.find('?') {
            let raw_query = &path[query_separator_index + 1..];
            query_string = Some(QueryString::from(raw_query));
            path = &path[..query_separator_index];
        }

        Ok(Self {
            path,
            query_string,
            method,
            headers,
            body,
        })
    }
}

/// Extracts the next whitespace-delimited token from an HTTP request string.
///
/// Returns `(token, rest_of_string)`. Splits on space (' ') or carriage return ('\r')
/// because HTTP uses "\r\n" as line terminators (RFC 2616, Section 2.2).
///
/// Uses `char_indices()` instead of `chars().enumerate()` so that the index
/// is a byte offset -- required for safe string slicing in Rust.
fn extract_next_token(input: &str) -> Option<(&str, &str)> {
    for (byte_index, character) in input.char_indices() {
        if character == ' ' || character == '\r' {
            let token = &input[..byte_index];
            let remaining = &input[byte_index + 1..];
            return Some((token, remaining));
        }
    }
    None
}

/// Splits the raw request content after the request line into headers and body.
///
/// In HTTP/1.1, headers and body are separated by a blank line ("\r\n\r\n").
/// For simplicity, this parser locates the JSON body by finding the outermost
/// `{...}` braces -- a pragmatic approach for a learning project that handles
/// the most common POST payloads (JSON objects).
fn split_headers_and_body(raw_content: &str) -> Option<(&str, &str)> {
    let mut body_start_index = 0;
    let mut body_end_index = 0;

    for (byte_index, character) in raw_content.char_indices() {
        if character == '{' && body_start_index == 0 {
            body_start_index = byte_index;
        }
        if character == '}' {
            body_end_index = byte_index;
        }
    }

    if body_start_index == 0 {
        return None;
    }

    let headers = &raw_content[..body_start_index];
    let body = &raw_content[body_start_index..body_end_index + 1];
    Some((headers, body))
}

// ---------------------------------------------------------------------------
// Error types
// ---------------------------------------------------------------------------

/// Enumerates everything that can go wrong when parsing an HTTP request.
///
/// Each variant maps to a specific failure mode, making it easy for callers
/// to pattern-match and return the appropriate HTTP status code.
pub enum ParseError {
    Request,
    Encoding,
    Protocol,
    Method,
}

/// Converts a UTF-8 decoding error into our domain error.
/// This is what makes the `?` operator work on `str::from_utf8()`.
impl From<Utf8Error> for ParseError {
    fn from(_: Utf8Error) -> Self {
        ParseError::Encoding
    }
}

/// Converts a method parsing error into our domain error.
/// This is what makes the `?` operator work on `method_str.parse()`.
impl From<MethodError> for ParseError {
    fn from(_: MethodError) -> Self {
        ParseError::Method
    }
}

impl ParseError {
    fn message(&self) -> &str {
        match self {
            Self::Request => "Invalid Request",
            Self::Encoding => "Invalid Encoding",
            Self::Protocol => "Invalid Protocol",
            Self::Method => "Invalid Method",
        }
    }
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "{}", self.message())
    }
}

impl Debug for ParseError {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "{}", self.message())
    }
}

impl Error for ParseError {}
