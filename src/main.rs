mod request;

use std::{
    fs,
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
};
use request::Request;

static HTMX_TEST: &str = r#"
    <article>
        <h2>htmx town</h2>
        <p><code>htmx</code> has just changed the content of the DOM</p>
        <ul>
            <li>Item one</li>
            <li>Item two</li>
            <li>Item three</li>
        </ul>
    </article>
"#;

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

    let req = Request::new(&request_line);

    match req.path.as_str() {
        "/" => serve_text(stream, fs::read_to_string("index.html").unwrap()),
        "/htmx-test" => serve_text(stream, String::from(HTMX_TEST)),
        _ => {
            let path = &req.path[1..];
            serve_text(
                stream,
                if let Ok(content) = fs::read_to_string(path) {
                    content
                } else {
                    fs::read_to_string("404.html").unwrap()
                }
            );
        }
    }
}

fn serve_text(mut stream: TcpStream, text: String) {
    let status_line = "HTTP/1.1 200 OK";
    let length = text.len();
    let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{text}");
    stream.write_all(response.as_bytes()).unwrap();
}