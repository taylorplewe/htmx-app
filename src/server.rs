use std::{
    fs,
    io::{
        prelude::*,
        BufReader,
    },
    net::{
        TcpListener,
        TcpStream,
    }
};
use crate::{
    city::City,
    request::Request,
    response::{Response, SendResponse}
};

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

pub struct Server {
    pub cities: Vec<City>,
}

impl Server {
    pub fn new() -> Self {
        Self { cities: vec![] }
    }
    pub fn serve(&self) {
        let listening_url = "127.0.0.1:7878";
        let listener = TcpListener::bind(listening_url).unwrap();
        println!("Listening on {listening_url}");

        for stream in listener.incoming() {
            let stream = stream.unwrap();
            self.handle_connection(stream);
        }
    }

    fn handle_connection(&self, mut stream: TcpStream) {
        let buf_reader = BufReader::new(&stream);

        let request_line = buf_reader.lines().next().unwrap().unwrap();

        println!("request: {request_line}");

        let req = Request::new(&request_line);

        match req.path.as_str() {
            "/" => self.serve_file(stream, "index.html"),
            "/enter" => self.serve_html(stream, String::from(ENTER_HTML)),
            _ => {
                let path = &req.path[1..];
                self.serve_file(
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

    fn serve_file(&self, stream: TcpStream, file_path: &str) {
        let mime = mime_guess::from_path(file_path).first();
        if mime.is_none() {
            eprintln!("could not get the MIME type for the file at {file_path}");
            return;
        }
        let mime = mime.unwrap();

        if let Ok(file_text) = fs::read_to_string(file_path) {
            self.serve_bytes(stream, mime, file_text.as_bytes());
        } else if let Ok(file_bytes) = fs::read(file_path) {
            self.serve_bytes(stream, mime, &file_bytes);
        } else {
            eprintln!("{file_path} does not exist!");
        }
    }

    fn serve_html(&self, mut stream: TcpStream, html: String) {
        stream.send_res(Response {
            status_code: 200,
            headers: vec![
                (String::from("Content-Length"), format!("{}", html.len())),
                (String::from("Content-Type"), String::from("text/html"))
            ],
            body: html.as_bytes().to_vec(),
        });
    }

    fn serve_bytes(
        &self,
        mut stream: TcpStream,
        mime: mime_guess::mime::Mime,
        bytes: &[u8]
    ) {
        stream.send_res(Response {
            status_code: 200,
            headers: vec![
                (String::from("Content-Length"), format!("{}", bytes.len())),
                (String::from("Content-Type"), format!("{mime}")),
            ],
            body: Vec::from(bytes)
        });
    }
}