mod request;

use std::{
    fs,
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
};
use request::Request;

fn main() {
    println!("were starting dude were starting");

    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        println!("were up dude were up");
        handle_connection(stream);
    }
}

fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&stream);
    let request_line = buf_reader.lines().next().unwrap().unwrap();

    let req = Request::new(&request_line);

    match req.path.as_str() {
        "/" => serve_file(stream, "index.html"),
        _ => serve_file(stream, "404.html"),
    }
}

fn serve_file(mut stream: TcpStream, file_path: &str) {
    let status_line = "HTTP/1.1 200 OK";
    let contents = fs::read_to_string(file_path).unwrap();
    let length = contents.len();

    let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");
    stream.write_all(response.as_bytes()).unwrap();
}