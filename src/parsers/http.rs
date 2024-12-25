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
    lines.for_each(|line| {
        if line.len() != 0 {
            let mut splits = line.split(":");
            eprintln!("Line: {}", line);
            let key = splits
                .next()
                .expect("Error while parsing headers")
                .trim()
                .to_lowercase();
            let content = splits
                .next()
                .expect("Error while parsing headers")
                .trim()
                .to_lowercase();
            headers.insert(key, content);
        }
    });
    (request_line, headers)
}

fn get_request_line(request_line_str: &str) -> HashMap<String, String> {
    let mut splits = request_line_str.split(" ");
    let mut request_line = HashMap::new();
    let method = String::from(
        splits
            .next()
            .expect("Error while parsing request line method"),
    );
    let path = String::from(
        splits
            .next()
            .expect("Error while parsing request line path"),
    );
    let version = String::from(
        splits
            .next()
            .expect("Error while parsing request line protocol"),
    );
    request_line.insert(String::from("method"), method);
    request_line.insert(String::from("path"), path);
    request_line.insert(String::from("version"), version);

    request_line
}
