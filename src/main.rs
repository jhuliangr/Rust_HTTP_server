#![allow(dead_code)]
use server::Server;
use website_handler::WebsiteHandler;
use std::env;
// use std::sync::mpsc;
use std::thread;

mod server;
mod http;
mod website_handler;


fn main() {
    let default_path = format!("{}/public", env!("CARGO_MANIFEST_DIR"));
    let public_path = env::var("PUBLIC_PATH").unwrap_or(default_path);
    // println!("-------------> {}", public_path);
    let server: Server = Server::new("127.0.0.1".to_owned(), 8080);
    let pid = thread::spawn(||server.run(WebsiteHandler::new(public_path)));
    
    pid.join().unwrap();
    // server.run(WebsiteHandler::new(public_path));
}

//add threads with threads and sync library
//add async work