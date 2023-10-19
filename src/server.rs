use crate::http::{Response, Request, StatusCode, ParseError};
use std::convert::TryFrom;
// use std::convert::TryInto;
use std::net::TcpListener;
use std::io::Read;

pub trait Handler{
    fn handle_request(&mut self, request: &Request) -> Response;
    fn handle_bad_request(&mut self, err: &ParseError) -> Response{
        println!("Failed to parse the request: {}", err);
        Response::new(StatusCode::BadRequest, None)
    }
}

pub struct Server{
    host: String,
    port: u16
}
impl Server{
    pub fn new(host: String, port: u16) -> Self{
        Server {
            host, 
            port 
        }
    }
    pub fn run (self, mut handler: impl Handler){
        let connection_string = self.host.to_owned() + ":" + &self.port.to_string();
        let listener = TcpListener::bind(connection_string).unwrap();
        println!("Rust Http server on host: {} listening on port {}",self.host, self.port);    
        loop {
            match listener.accept(){
                Ok((mut stream, addr)) => {
                    println!("Connection established on {}", addr);
                    let mut buffer = [0; 1024];
                    match stream.read(&mut buffer){
                        Ok(_) =>{
                            println!("Received a request: {}", String::from_utf8_lossy(&buffer));
                            let response = match Request::try_from(&buffer[..]){
                                Ok(request) => handler.handle_request(&request),                                
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