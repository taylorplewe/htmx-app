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
    pub fn new(mut stream: &TcpStream) -> Option<Self> {
        let mut buf: [u8; u16::MAX as usize] = [0; u16::MAX as usize];
        let _num_bytes_peeked = stream.peek(&mut buf).expect("Could not peek the request buffer");
        let req_text = String::from_utf8(buf.to_vec()).expect("Could not convert utf8 block to String");

        let mut lines = req_text.lines();
        let req_info = lines.next().unwrap_or("");
        if req_info.is_empty() { return None; }
        let mut req_info_split = req_info.split(' ');
        let method = req_info_split.next().unwrap_or("").to_string();
        let path = req_info_split.next().unwrap_or("").to_string();
        if method.is_empty() || path.is_empty() { return None; }

        let headers: HashMap<String, String> = lines
            .filter_map(|line| {
                let mut key_val_str = line.splitn(2, ": ");
                Some((key_val_str.next()?.trim().to_string(), key_val_str.next()?.trim().to_string()))
            })
            .collect();

        let body = req_text.split("\r\n\r\n").nth(1).unwrap_or("").to_string();

        let params: HashMap<String, String> = body
            .split('&')
            .filter_map(|param| {
                let mut key_value_str = param.splitn(2, '=');
                Some((key_value_str.next()?.trim().to_string(), key_value_str.next()?.trim().to_string()))
            })
            .collect();

        Some(Self {
            method,
            path,
            headers,
            body,
            params,
        })
    }
}