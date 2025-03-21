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
use std::io::Lines;
use std::net::Shutdown;
use crate::{
    city::City,
    request::Request,
    response::{Response, SendResponse}
};

static ENTER_HTML: &str = r#"
    <article>
        <h1>htmx town</h1>
        <p><code>htmx</code> has just changed the content of the DOM</p>
        <ul>
            <li>Item one</li>
            <li>Item two</li>
            <li>Item three</li>
        </ul>
        <button hx-post="/cities/add" hx-vals='{"name": "Laramie", "state": "WY"}'>Add city</button>
    </article>
"#;

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
        let req = Request::new(&stream);
        if req.is_none() { return; }
        let req = req.unwrap();

        println!("method: {}, path: {}", req.method, req.path);
        println!("headers:");
        req.headers.iter().for_each(|header| {
            println!("{} - {}", header.0, header.1);
        });

        println!("body: {}", req.body);

        match req.path.as_str() {
            "/" => self.serve_file(stream, "index.html"),
            "/enter" => self.serve_html(stream, String::from(ENTER_HTML)),
            "/cities/add" => self.add_city(stream),
            _ => {
                let path = &req.path[1..];
                self.serve_file(
                    stream,
                    if let Ok(exists) = fs::exists(path) {
                        path
                    } else {
                        "404.html"
                    }
                );
            }
        }
    }

    fn serve_file(&self, mut stream: TcpStream, file_path: &str) {
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
        self.serve_bytes(
            stream,
            mime_guess::mime::TEXT_HTML,
            html.as_bytes()
        );
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
        stream.flush().unwrap();
        stream.shutdown(Shutdown::Write).unwrap();
    }

    fn add_city(&self, stream: TcpStream) {
        println!("ADD CITY???? dude");
    }
}