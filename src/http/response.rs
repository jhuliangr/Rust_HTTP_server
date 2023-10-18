// use std::fmt::{Display, Formatter, Result as FmtResult};
use super::StatusCode;
use std::net::TcpStream;
use std::io::{Result as IoResult, Write};

#[derive(Debug)]
pub struct Response {
    status_code: StatusCode,
    body:Option<String>,

}

impl Response {
    pub fn new(status_code: StatusCode, body: Option<String>) -> Self{
        Response{ status_code, body }
    }
    // the &mut impl Write is for expecting any datatype wich can write on the stream
    pub fn send( &self, stream: &mut impl Write) -> IoResult<()>{
        let body = match &self.body{
            Some(b) => b,
            None => ""
        };
        write!(stream, 
            "HTTP/1.1 {} {}\r\n\r\n{}", 
            self.status_code, 
            self.status_code.reason_phrase(), 
            body
        )
    }
}
