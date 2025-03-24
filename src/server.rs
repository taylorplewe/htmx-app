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
        Self { cities: HashMap::new(), current_city_id: 1 }
    }
    pub fn serve(&mut self) {
        let listening_url = "127.0.0.1:7878";
        let listener = TcpListener::bind(listening_url).unwrap();
        println!("Listening on {listening_url}");

        for stream in listener.incoming() {
            let stream = stream.unwrap();
            self.handle_connection(stream);
        }
    }

    fn handle_connection(&mut self, stream: TcpStream) {
        let req = if let Some(req) = Request::from_stream(&stream) { req } else { return; };

        match req.method.as_str() {
            "GET" => match req.path.as_str() {
                "/" => self.serve_file(stream, "index.html"),
                "/enter" => self.serve_files_combined(stream, &["src/html/main-card.html", "src/html/new-city-dialog.html"]),
                "/cities/select/get" => self.send_cities_select_html(stream),
                _ => {
                    let path = &req.path[1..];
                    self.serve_file(
                        stream,
                        if let Ok(_exists) = fs::exists(path) {
                            path
                        } else {
                            "404.html"
                        }
                    );
                }
            }
            "POST" => match req.path.as_str() {
                "/cities/add" => self.add_city(stream, req),
                "/cities/remove" => self.remove_city(stream, req),
                _ => unreachable!()
            }
            _ => unreachable!()
        }

    }

    fn serve_files_combined(&self, stream: TcpStream, file_paths: &[&str]) {
        let full_text = file_paths
            .iter()
            .map(|p| fs::read_to_string(p).unwrap_or("".to_string()))
            .collect::<Vec<String>>()
            .join("");
        self.serve_bytes(stream, mime_guess::mime::TEXT_HTML, full_text.as_bytes());
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

    fn add_city(&mut self, stream: TcpStream, req: Request) {
        // verify the request contains all the necessary fields
        if !["name", "state", "country", "sister_city_id"].iter().all(|key| req.params.contains_key(&key.to_string())) {
            eprintln!("Request does not contain all the keys necessary for a city");
            return;
        }

        // if the sister city already has its own sister city, break that connection
        let mut sister_city_id: Option<u32>;
        let sister_city_id_str = req.params.get("sister_city_id").unwrap();
        sister_city_id = Some(u32::from_str(sister_city_id_str).expect("sister_city_id must be a number"));
        if sister_city_id == Some(0) {
            sister_city_id = None
        } else {
            if let Some(third_city) = self.cities.get(&sister_city_id.unwrap())
                .and_then(|sister_city| sister_city.sister_city_id)
                .and_then(|id| self.cities.get_mut(&id))
            {
                third_city.sister_city_id = None;
            }

            if let Some(sister_city) = self.cities.get_mut(&sister_city_id.unwrap()) {
                sister_city.sister_city_id = Some(self.current_city_id);
            }
        }

        self.cities.insert(self.current_city_id, City {
            id: self.current_city_id,
            name: req.params.get("name").unwrap().into(),
            state: req.params.get("state").unwrap().into(),
            country: req.params.get("country").unwrap().into(),
            sister_city_id,
        });
        self.current_city_id += 1;

        // response
        self.send_cities_list_html(stream);
    }

    fn remove_city(&mut self, stream: TcpStream, req: Request) {
        let city_id_str = req.params.get("id").expect("Must send a city's ID to remove");
        let city_id = u32::from_str(city_id_str).expect("City ID must be a number");

        // if that city had a sister city, break that connection
        let city = self.cities.get(&city_id).unwrap();
        if let Some(sister_city_id) = city.sister_city_id {
            let sister_city = self.cities.get_mut(&sister_city_id).unwrap();
            sister_city.sister_city_id = None;
        }

        self.cities.remove(&city_id);

        // response
        self.send_cities_list_html(stream);
    }

    fn send_cities_list_html(&mut self, mut stream: TcpStream) {
        let cities_list_html = self.cities.values()
            .map(|city| format!(
                r##"
                    <li>
                        <p><strong><em>{}</em></strong></h3>
                        <p>{}, {}</p>
                        <p>sister city: {}</p>
                        <a
                            class="remove-city"
                            hx-post="/cities/remove"
                            hx-swap="innerHTML"
                            hx-target="#cities-list"
                            hx-vals='{{ "id": {} }}'
                        >
                            Remove
                        </a>
                    </li>
                "##,
                city.name,
                city.state, city.country,
                if let Some(id) = city.sister_city_id {
                    let sister_city = self.cities.get(&id).expect("No city found with that ID");
                    format!("{}, {} {}", sister_city.name, sister_city.state, sister_city.country)
                } else {
                    String::from("N/A")
                },
                city.id,
            ))
            .collect::<Vec<String>>()
            .join("\n");

        stream.send_res(Response {
            status_code: 200,
            headers: vec![
                (String::from("Content-Length"), format!("{}", cities_list_html.len())),
                (String::from("Content-Type"), mime_guess::mime::TEXT_HTML.to_string()),
            ],
            body: cities_list_html.as_bytes().to_vec(),
        });
        stream.flush().unwrap();
        stream.shutdown(Shutdown::Write).unwrap();
    }

    fn send_cities_select_html(&mut self, stream: TcpStream) {
        let cities_select_html = self.cities.values()
            .map(|city| format!(r#"<option value={}>{}, {} {}</option>"#, city.id, city.name, city.state, city.country))
            .chain(std::iter::once(String::from("<option value='0'>N/A</option>")))
            .collect::<Vec<String>>()
            .join("\n");

        self.serve_bytes(stream, mime_guess::mime::TEXT_HTML, cities_select_html.as_bytes());
    }
}
