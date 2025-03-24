mod request;
mod city;
mod response;
mod server;

use server::Server;

fn main() {
    let mut server = Server::new();
    server.serve();
}