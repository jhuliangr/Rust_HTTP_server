use crate::http::{Request, Response, StatusCode,Method };
use super::server::Handler;
use std::fs;

pub struct WebsiteHandler{
    public_path: String
}
impl WebsiteHandler{
    pub fn new(public_path:String) -> Self {
        Self{public_path}
    }
    fn read_file(&self, file_path: &str) -> Option<String>{
        // if file_path.starts_with('/'){
        //     file_path = &file_path[1..];
        // }
        let path = format!("{}/{}", self.public_path, file_path);
        match fs::canonicalize(path) {
            Ok(path) => {
                if path.starts_with(&self.public_path){
                    fs::read_to_string(path).ok()
                }else{
                    println!("Directory Traversal Attack Attempted: {}", file_path);
                    None
                }
            },  
            Err(_) => None
        }
    }
}

impl Handler for WebsiteHandler{
    fn handle_reuqest(&mut self, request: &Request) -> Response {
        // Response::new(StatusCode::Ok, Some("<h1>Test 1</h1>".to_string()))
        match request.method() {
            Method::GET => match request.path(){
                "/" => Response::new(StatusCode::Ok, self.read_file("index.html")), 
                path => match self.read_file(path) {
                    Some(contents) => {
                        println!("_________________________> encontro algo");
                        Response::new(StatusCode::Ok, Some(contents))
                    },
                    None =>{ 
                        println!("_________________________> No lo encontro {}", path);
                        Response::new(StatusCode::NotFound, None)
                    }
                },
            }
            _ => Response::new(StatusCode::NotFound, None)
        }
    }
}