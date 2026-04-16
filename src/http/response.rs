use super::StatusCode;
use std::io::{Result as IoResult, Write};

#[derive(Debug)]
pub struct Response {
    status_code: StatusCode,
    body: Option<String>,
}

impl Response {
    pub fn new(status_code: StatusCode, body: Option<String>) -> Self {
        Response { status_code, body }
    }

    /// Writes the full HTTP response directly to any writable stream.
    ///
    /// The parameter `&mut impl Write` uses Rust's static dispatch (monomorphization):
    /// the compiler generates a specialized version of this method for each concrete
    /// type (TcpStream, Vec<u8>, etc.) -- zero overhead compared to a virtual call.
    /// This also means you can pass a `Vec<u8>` in tests to capture the output.
    pub fn send(&self, stream: &mut impl Write) -> IoResult<()> {
        let body_content = match &self.body {
            Some(content) => content,
            None => "",
        };

        // HTTP/1.1 requires a status line, headers, a blank line, then the body.
        // Content-Length tells the client exactly how many bytes to expect,
        // preventing it from hanging while waiting for more data.
        write!(
            stream,
            "HTTP/1.1 {} {}\r\nContent-Length: {}\r\n\r\n{}",
            self.status_code,
            self.status_code.reason_phrase(),
            body_content.len(),
            body_content
        )
    }
}
