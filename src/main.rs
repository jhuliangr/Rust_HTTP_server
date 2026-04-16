#![allow(dead_code)]

use server::Server;
use std::env;
use std::thread;
use website_handler::WebsiteHandler;

mod http;
mod server;
mod website_handler;

fn main() {
    // Resolve the static files directory. `CARGO_MANIFEST_DIR` is set at compile time
    // by Cargo, so the default always points to the project's `public/` folder
    // regardless of where the binary is executed from.
    let default_public_path = format!("{}/public", env!("CARGO_MANIFEST_DIR"));
    let public_path = env::var("PUBLIC_PATH").unwrap_or(default_public_path);

    let server = Server::new("127.0.0.1".to_owned(), 8080);

    // Spawn the server on a dedicated thread. `move` transfers ownership of
    // `server` and `public_path` into the thread's closure. The main thread
    // then blocks on `join()`, waiting for the server to finish (which it won't,
    // since the accept loop runs forever -- Ctrl+C terminates the process).
    let server_thread = thread::spawn(move || {
        server.run(WebsiteHandler::new(public_path));
    });

    server_thread.join().unwrap();
}
