use crate::http::{ParseError, Request, Response, StatusCode};
use std::convert::TryFrom;
use std::io::Read;
use std::net::TcpListener;

// Maximum size of an HTTP request we'll accept (1 KB).
// Real servers like Nginx default to 8 KB; we keep it small since this
// server is designed for learning, not production traffic.
const MAX_REQUEST_SIZE: usize = 1024;

/// Trait that decouples the TCP server from specific routing logic.
///
/// This is the Strategy pattern: `Server` handles networking (accept, read, write),
/// while the `Handler` implementation decides *what* to do with each request.
/// You could swap `WebsiteHandler` for an `ApiHandler` or a `ProxyHandler`
/// without changing a single line in `Server`.
pub trait Handler {
    fn handle_request(&mut self, request: &Request) -> Response;

    /// Default implementation for malformed requests.
    /// Handlers can override this to customize error pages.
    fn handle_bad_request(&mut self, error: &ParseError) -> Response {
        eprintln!("[ERROR] Failed to parse request: {}", error);
        Response::new(StatusCode::BadRequest, None)
    }
}

pub struct Server {
    host: String,
    port: u16,
}

impl Server {
    pub fn new(host: String, port: u16) -> Self {
        Server { host, port }
    }

    /// Starts the TCP listener and enters the main accept loop.
    ///
    /// This method consumes `self` (takes ownership) because a server instance
    /// represents a single run lifecycle -- once started, its config shouldn't change.
    pub fn run(self, mut handler: impl Handler) {
        let address = format!("{}:{}", self.host, self.port);

        // `bind` performs both socket creation and binding in one call.
        // In C you'd need socket() + bind() + listen() separately.
        let listener = TcpListener::bind(&address).unwrap();
        println!("Listening on http://{}", address);

        // Each iteration of this loop handles one TCP connection.
        // Note: this is a single-threaded accept loop, meaning requests
        // are processed sequentially. See main.rs for the threading model.
        loop {
            match listener.accept() {
                Ok((mut tcp_stream, client_address)) => {
                    println!("[CONN] {} connected", client_address);

                    // Read raw bytes from the TCP stream into a fixed-size buffer.
                    // The buffer lives on the stack -- no heap allocation per request.
                    let mut request_buffer = [0u8; MAX_REQUEST_SIZE];

                    match tcp_stream.read(&mut request_buffer) {
                        Ok(bytes_read) => {
                            println!(
                                "[RECV] {} bytes from {}",
                                bytes_read, client_address
                            );

                            // Try to parse raw bytes into a structured Request.
                            // On success, delegate to the handler; on failure, return 400.
                            let response = match Request::try_from(&request_buffer[..]) {
                                Ok(request) => handler.handle_request(&request),
                                Err(parse_error) => handler.handle_bad_request(&parse_error),
                            };

                            if let Err(send_error) = response.send(&mut tcp_stream) {
                                eprintln!(
                                    "[ERROR] Failed to send response to {}: {}",
                                    client_address, send_error
                                );
                            }
                        }
                        Err(read_error) => {
                            eprintln!(
                                "[ERROR] Failed to read from {}: {}",
                                client_address, read_error
                            );
                        }
                    }
                }
                Err(accept_error) => {
                    eprintln!("[ERROR] Failed to accept connection: {}", accept_error);
                }
            }
        }
    }
}
