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

        println!("method: {}", req.method);

        match req.method.as_str() {
            "GET" => match req.path.as_str() {
                "/" => self.serve_file(stream, "index.html"),
                "/enter" => self.serve_files(stream, &["src/html/main-card.html", "src/html/new-city-dialog.html"]),
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
            "POST" => match req.path.as_str() {
                "/cities/add" => {
                    println!("cities add body");
                    println!("{}", req.body);
                },
                _ => unreachable!()
            }
            _ => unreachable!()
        }

    }

    fn serve_files(&self, mut stream: TcpStream, file_paths: &[&str]) {
        let full_text = file_paths
            .iter()
            .map(|p| fs::read_to_string(p).unwrap_or("".to_string()))
            .collect::<Vec<String>>()
            .join("");
        self.serve_bytes(stream, mime_guess::mime::TEXT_HTML, full_text.as_bytes());
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