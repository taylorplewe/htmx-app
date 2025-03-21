mod request;
mod city;
mod response;

use mime_guess;

use std::{
    fs,
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
};

use request::Request;
use response::Response;
use city::City;
use crate::response::SendResponse;

static ENTER_HTML: &str = r"
    <article>
        <h1>htmx town</h1>
        <p><code>htmx</code> has just changed the content of the DOM</p>
        <ul>
            <li>Item one</li>
            <li>Item two</li>
            <li>Item three</li>
        </ul>
    </article>
";

fn main() {
    let listening_url = "127.0.0.1:7878";
    let listener = TcpListener::bind(listening_url).unwrap();
    println!("Listening on {listening_url}");

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        handle_connection(stream);
    }
}

fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&stream);

    let request_line = buf_reader.lines().next().unwrap().unwrap();

    println!("request: {request_line}");

    let req = Request::new(&request_line);

    match req.path.as_str() {
        "/" => serve_file(stream, "index.html"),
        "/enter" => serve_text(stream, String::from(ENTER_HTML)),
        _ => {
            let path = &req.path[1..];
            serve_file(
                stream,
                if let Ok(content) = fs::exists(path) {
                    path
                } else {
                    "404.html"
                }
            );
        }
    }
}

fn serve_file(stream: TcpStream, file_path: &str) {
    if let Some(mime) = mime_guess::from_path(file_path).first() {
        if let Ok(file_text) = fs::read_to_string(file_path) {
            serve_bytes(stream, mime, file_text.as_bytes());
        } else if let Ok(exists) = fs::exists(file_path) {
            if let Ok(file_bytes) = fs::read(file_path) {
                serve_bytes(stream, mime, &file_bytes);
            }
        } else {
            eprintln!("{file_path} does not exist!");
        }
    } else {
        eprintln!("could not get the MIME type for the file at {file_path}");
    }
}

fn serve_text(mut stream: TcpStream, text: String) {
    let status_line = "HTTP/1.1 200 OK";
    let length = text.len();
    let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{text}");
    stream.write_all(response.as_bytes()).unwrap();
}

fn serve_bytes(mut stream: TcpStream, mime: mime_guess::mime::Mime, bytes: &[u8]) {
    stream.send_res(Response {
        status_code: 200,
        status_msg: String::from("OK"),
        headers: vec!(
            (String::from("Content-Length"), format!("{}", bytes.len())),
            (String::from("Content-Type"), format!("{}", mime)),
        ),
        body: Vec::from(bytes)
    });
}