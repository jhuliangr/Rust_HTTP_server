use crate::http::{Response, Request, StatusCode, ParseError};
use std::convert::TryFrom;
// use std::convert::TryInto;
use std::net::TcpListener;
use std::io::Read;

pub trait Handler{
    fn handle_reuqest(&mut self, request: &Request) -> Response;
    fn handle_bad_request(&mut self, err: &ParseError) -> Response{
        println!("Failed to parse the request: {}", err);
        Response::new(StatusCode::BadRequest, None)
    }
}

pub struct Server{
    addr: String,
}
impl Server{
    pub fn new(addr: String) -> Self{
        Server {
            addr 
        }
    }
    pub fn run (self, mut handler: impl Handler){
        let listener = TcpListener::bind(&self.addr).unwrap();
        println!("Rust Http server listening on port {}", self.addr);    
        loop {
            match listener.accept(){
                Ok((mut stream, addr)) => {
                    println!("Connection established on {}", addr);
                    let mut buffer = [0; 1024];
                    match stream.read(&mut buffer){
                        Ok(_) =>{
                            println!("Received a request: {}", String::from_utf8_lossy(&buffer));
                            let response = match Request::try_from(&buffer[..]){
                                Ok(request) => handler.handle_reuqest(&request),                                
                                Err(err) =>  handler.handle_bad_request(&err)

                            };
                            if let Err(err) = response.send(&mut stream){
                                println!("Failed to send response: {}", err)
                            }




                        },
                        Err(err) => println!("There was an error: {}", err)
                    } 
                },
                Err(err) => println!("Failed to establish connection {}", err),
            }
        }
    }
}


// 30