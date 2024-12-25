#![allow(dead_code)]

use std::collections::HashMap;

pub struct HttpPacket {
    pub request_line: HashMap<String, String>,
    pub headers: HashMap<String, String>,
    body: Vec<u8>,
}

impl HttpPacket {
    pub fn new(packet: &str) -> HttpPacket {
        let (request_line, headers) = parse_header(packet);
        HttpPacket {
            request_line,
            headers,
            body: vec![],
        }
    }
}

pub fn parse_header(packet: &str) -> (HashMap<String, String>, HashMap<String, String>) {
    let mut headers: HashMap<String, String> = HashMap::new();
    let mut lines = packet.lines();
    let request_line = get_request_line(lines.next().expect("Error while parsing request line"));
    for line in lines {
        if line.len() != 0 {
            let mut splits = line.split(":");
            if let Some(key) = splits.next() {
                if let Some(content) = splits.next() {
                    headers.insert(key.trim().to_lowercase(), content.trim().to_lowercase());
                } else {
                    eprintln!("Parsing headers: Empty value for {}", key);
                }
            } else {
                continue;
            }
        }
    }
    (request_line, headers)
}

fn get_request_line(request_line_str: &str) -> HashMap<String, String> {
    let mut splits = request_line_str.split(" ");
    let mut request_line = HashMap::new();
    if let Some(method) = splits.next() {
        request_line.insert(String::from("method"), String::from(method));
    } else {
        request_line.insert(String::from("method"), String::from("error"));
    }
    if let Some(path) = splits.next() {
        request_line.insert(String::from("path"), String::from(path));
    } else {
        request_line.insert(String::from("path"), String::from("error"));
    }
    if let Some(protocol) = splits.next() {
        request_line.insert(String::from("protocol"), String::from(protocol));
    } else {
        request_line.insert(String::from("protocol"), String::from("error"));
    }
    request_line
}
