use mime_guess;

use std::{
    fs,
    io::{
        prelude::*,
    },
    net::{
        TcpListener,
        TcpStream,
        Shutdown,
    },
    collections::HashMap,
    str::FromStr,
};
use crate::{
    city::City,
    request::Request,
    response::{Response, SendResponse}
};

pub struct Server {
    pub cities: HashMap<u32, City>,
    current_city_id: u32,
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
        let req = if let Some(req) = Request::from_stream(&stream) { req } else { return; };

        match req.method.as_str() {
            "GET" => match req.path.as_str() {
                "/" => self.serve_file(stream, "index.html"),
                "/enter" => self.serve_files_combined(stream, &["src/html/main-card.html", "src/html/new-city-dialog.html"]),
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

    fn serve_files_combined(&self, mut stream: TcpStream, file_paths: &[&str]) {
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

    fn add_city(&mut self, req: Request) {
        // verify the request contains all the necessary fields
        if !["name", "state", "country", "sister_city_id"].iter().all(|key| req.params.contains_key(key)) {
            eprintln!("Request does not contain all the keys necessary for a city");
            return;
        }

        // if the sister city already has its own sister city, break that connection
        let sister_city_id = u32::from_str(req.params.get("sister_city_id").unwrap()).expect("sister_city_id must be a number");
        let sister_city = self.cities.get(&sister_city_id).expect("no sister city found with that id");
        if let Some(third_city_id) = &sister_city.sister_city_id {
            self.cities.get(third_city_id).unwrap().sister_city_id = None;
        }

        self.cities.insert(self.current_city_id, City {
            id: self.current_city_id,
            name: req.params.get("name")
        });
        self.cities.get(req.params.get("sister_city_id"))
        self.current_city_id += 1;
    }
}