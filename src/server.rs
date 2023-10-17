use crate::http::Request; 
use std::convert::TryFrom;
use std::convert::TryInto;
use std::net::TcpListener;
use std::io::Read;
pub struct Server{
    addr: String,
}
impl Server{
    pub fn new(addr: String) -> Self{
        Server {
            addr 
        }
    }
    pub fn run (self){
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
                            match Request::try_from(&buffer[..]){
                                Ok(request) => {
                                    // println!(request);
                                    dbg!(request);// no esta mostrando el response
                                },
                                Err(err) => println!("Failed to parse the request: {}", err)
                            }
                            // let res: &Result<Request, _> = &buffer[..].try_into();



                        },
                        Err(err) => println!("There was an error: {}", err)
                    } 
                },
                Err(err) => println!("Failed to establish connection {}", err),
            }
        }
    }
}
