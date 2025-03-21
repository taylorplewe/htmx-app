mod request;
mod city;
mod response;
mod server;

use mime_guess;

use std::{
    fs,
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
};

use request::Request;
use response::{Response, SendResponse};
use city::City;
use crate::server::Server;

fn main() {
    let server = Server::new();
    server.serve();
}