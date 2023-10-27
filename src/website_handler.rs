use crate::http::{Request, Response, StatusCode, Method };
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

impl Handler for WebsiteHandler {
    fn handle_request(&mut self, request: &Request) -> Response {
        // Response::new(StatusCode::Ok, Some("<h1>Test 1</h1>".to_string()))
        match request.method() {
            Method::GET => match request.path(){
                
                "/" => Response::new(StatusCode::Ok, self.read_file("index.html")), 

                path => match self.read_file(path) {
                    Some(contents) => {
                        Response::new(StatusCode::Ok, Some(contents))
                    },
                    None =>{ 
                        Response::new(StatusCode::NotFound, None)
                    }
                },
            }
            Method::POST =>match request.path(){
                "/" => {
                    let str = format!("It was sent a post to / with data: {} Thats ok", request.body().to_string());
                    Response::new(StatusCode::Ok, Some(str))
                },

                not_found => {
                    let str = format!("It was sent a post to {} with data: {} thats a not yet defined endpoint", not_found, request.body().to_string());
                    Response::new(StatusCode::Ok, Some(str))
                }
            }
            _ => Response::new(StatusCode::NotFound, None)
        }
    }
}