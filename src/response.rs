use std::io::Write;
use std::net::TcpStream;

pub struct Response {
    pub status_code: u16,
    pub status_msg: String,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

impl Response {
    pub fn new(status_code: u16, status_msg: String, headers: Vec<(String, String)>, body: Vec<u8>) -> Self {
        Self {
            status_code,
            status_msg,
            headers,
            body,
        }
    }
    pub fn to_bytes(&self) -> Vec<u8> {
        let headers_text = self.headers
            .iter()
            .map(|header| format!("{}: {}", header.0, header.1))
            .collect::<Vec<String>>()
            .join("\r\n");
        let metadata_text = format!("HTTP/1.1 {} {}\r\n{}\r\n\r\n", self.status_code, self.status_msg, headers_text);
        let mut full_response_bytes = Vec::with_capacity(metadata_text.len() + headers_text.len());
        full_response_bytes.extend_from_slice(&metadata_text.as_bytes());
        full_response_bytes.extend_from_slice(&self.body);
        full_response_bytes
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