use std::collections::HashMap;
use std::io::Write;
use std::net::TcpStream;

pub struct Response {
    pub status_code: u16,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

impl Response {
    pub fn to_bytes(&self) -> Vec<u8> {
        let headers_text = self.headers
            .iter()
            .map(|header| format!("{}: {}", header.0, header.1))
            .collect::<Vec<String>>()
            .join("\r\n");
        let metadata_text = format!("{}\r\n{}\r\n\r\n", self.get_status_text(), headers_text);
        let mut full_response_bytes = Vec::with_capacity(metadata_text.len() + headers_text.len());
        full_response_bytes.extend_from_slice(&metadata_text.as_bytes());
        full_response_bytes.extend_from_slice(&self.body);
        full_response_bytes
    }
    fn get_status_text(&self) -> String {
        const H: &str = "HTTP/1.1";
        match self.status_code {
            200 => format!("{H} 200 OK"),
            _ => format!("{H} 404 NOT FOUND"),
        }
    }
}

pub trait SendResponse {
    fn send_res(&mut self, res: Response);
}

impl SendResponse for TcpStream {
    fn send_res(&mut self, res: Response) {
        self.write_all(&res.to_bytes()).unwrap()
    }
}