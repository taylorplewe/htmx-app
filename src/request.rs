use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read};
use std::net::TcpStream;

pub struct Request {
    pub method: String,
    pub path: String,
    pub headers: HashMap<String, String>,
    pub body: String,
}
impl Request {
    pub fn new(mut stream: &TcpStream) -> Option<Self> {
        let mut buf: [u8; u16::MAX as usize] = [0; u16::MAX as usize];
        let _num_bytes_peeked = stream.peek(&mut buf).unwrap();
        let req_text = String::from_utf8(buf.to_vec()).unwrap();

        // let mut req_text = String::new();
        // let _ = stream.read_to_string(&mut req_text).unwrap();
        let mut lines = req_text.lines();
        let req_info = lines.next().unwrap_or("");
        if req_info.is_empty() { return None; }
        let mut req_info_split = req_info.split(' ');
        let method = req_info_split.next().unwrap_or("");
        let path = req_info_split.next().unwrap_or("");
        if method.is_empty() || path.is_empty() { return None; }

        let mut headers: HashMap<String, String> = HashMap::new();
        while let Some(line) = lines.next() {
            if line.is_empty() { break; }
            let mut split_iter = line.split(": ");
            headers.insert(split_iter.next().unwrap().to_string(), split_iter.next().unwrap().to_string());
        }

        let body = req_text.split("\r\n\r\n").nth(1).unwrap_or("");

        Some(Self {
            method: String::from(method),
            path: String::from(path),
            headers,
            body: String::from(body),
        })
    }
}