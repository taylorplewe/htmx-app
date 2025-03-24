use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read};
use std::net::TcpStream;

pub struct Request {
    pub method: String,
    pub path: String,
    pub headers: HashMap<String, String>,
    pub body: String,
    pub params: HashMap<String, String>,
}
impl Request {
    pub fn from_stream(mut stream: &TcpStream) -> Option<Self> {
        let mut buf: [u8; u16::MAX as usize] = [0; u16::MAX as usize];
        let num_bytes_peeked = stream.peek(&mut buf).expect("Could not peek the request buffer");
        let req_text = String::from_utf8(buf[..num_bytes_peeked].to_vec()).expect("Could not convert utf8 block to String");

        let mut req_text_lines = req_text.lines();
        let mut req_info_split = req_text_lines.next()?.split(' ');
        let method = req_info_split.next()?.to_string();
        let path = req_info_split.next()?.to_string();

        // "Content-Length: 5356\nContent-Type: image/png" -> { "Content-Length": "5356", "Content-Type": "image/png" }
        let headers: HashMap<String, String> = req_text_lines
            .filter_map(|line| {
                line
                    .split_once(": ")
                    .map(|(k, v)| (k.to_string(), v.to_string()))
            })
            .collect();

        let body = req_text.split("\r\n\r\n").nth(1).unwrap_or("").to_string();

        // "name=Denver&state=CO" -> { "name": "Denver", "state": "CO" }
        let params: HashMap<String, String> = body
            .split('&')
            .filter_map(|param| {
                param
                    .split_once('=')
                    .map(|(k, v)| (k.to_string(), v.to_string()))
            })
            .collect();

        Some(Self {
            method: method.to_string(),
            path: path.to_string(),
            headers,
            body,
            params,
        })
    }
}