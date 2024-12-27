#![allow(dead_code)]
use bytes::Bytes;
use std::collections::HashMap;
use std::io::{Cursor, Seek, SeekFrom};

#[derive(Debug)]
pub enum HttpFrame {
    RequestHead(RequestHead),
    ResponseHead(ResponseHead),
    BodyChunk(Bytes),
}

#[derive(Debug)]
enum HttpError {
    Incomplete,
    Other(String),
}

pub enum BodyType {
    Image,
    Form,
    None,
}

impl HttpFrame {
    pub fn is_header_receive(buf: &mut Cursor<&[u8]>) -> Option<()> {
        if buf.get_ref().len() < 4 {
            return None;
        }
        if let Ok(mut position) = buf.seek(SeekFrom::End(0)) {
            while position > 4 {
                let raw_data = buf.get_ref();
                if raw_data[position as usize - 1] == 10
                    && raw_data[position as usize - 2] == 13
                    && raw_data[position as usize - 3] == 10
                    && raw_data[position as usize - 4] == 13
                {
                    buf.set_position(position as u64);
                    return Some(());
                }
                position -= 1;
            }
        }

        None
    }

    pub fn parse_header(buff: &mut Cursor<&[u8]>, end: usize) -> Result<RequestHead, String> {
        let (method, uri, version) = HttpFrame::parse_request_line(buff, end)?;
        let headers = HttpFrame::get_headers(buff, end)?;
        Ok(RequestHead::new(method, uri, version, headers))
    }

    fn parse_request_line(
        buff: &mut Cursor<&[u8]>,
        end: usize,
    ) -> Result<(String, String, String), String> {
        let method;
        let uri;
        let version;
        if let Some(first_line) = HttpFrame::get_next_line(buff, end) {
            if let Ok(request_line) = String::from_utf8(first_line) {
                (method, uri, version) = HttpFrame::get_request_line(request_line.as_str());
            } else {
                return Err("Invalid request line".to_string());
            }
        } else {
            return Err("Missing request line".to_string());
        }
        Ok((method, uri, version))
    }

    fn get_headers(
        buff: &mut Cursor<&[u8]>,
        end: usize,
    ) -> Result<HashMap<String, String>, String> {
        let mut headers = HashMap::new();
        while let Some(raw_line) = HttpFrame::get_next_line(buff, end) {
            if raw_line.len() != 0 {
                if let Ok(line) = String::from_utf8(raw_line) {
                    let mut splits = line.split(":");
                    if let Some(key) = splits.next() {
                        if let Some(content) = splits.next() {
                            headers
                                .insert(key.trim().to_lowercase(), content.trim().to_lowercase());
                        } else {
                            eprintln!("Parsing headers: Empty value for {}", key);
                        }
                    } else {
                        continue;
                    }
                } else {
                    return Err("Invalid headers".to_string());
                }
            }
        }
        Ok(headers)
    }

    fn get_next_line(buff: &mut Cursor<&[u8]>, end: usize) -> Option<Vec<u8>> {
        let start = buff.position() as usize;
        for i in start..(end - 1) {
            if buff.get_ref()[i] == 13 && buff.get_ref()[i + 1] == 10 {
                buff.set_position((i + 2) as u64);
                return Some(buff.get_ref()[start..i].to_vec());
            }
        }
        return None;
    }

    fn get_request_line(request_line_str: &str) -> (String, String, String) {
        let mut splits = request_line_str.split(" ");
        let method;
        let uri;
        let version;
        if let Some(retval) = splits.next() {
            method = String::from(retval);
        } else {
            method = String::from("error");
        }
        if let Some(retval) = splits.next() {
            uri = String::from(retval);
        } else {
            uri = String::from("error");
        }
        if let Some(retval) = splits.next() {
            version = String::from(retval);
        } else {
            version = String::from("error");
        }
        (method, uri, version)
    }
}

#[derive(Debug)]
pub struct RequestHead {
    pub method: String,
    pub uri: String,
    pub version: String,
    pub headers: HashMap<String, String>,
}

impl RequestHead {
    pub fn new(
        method: String,
        uri: String,
        version: String,
        headers: HashMap<String, String>,
    ) -> RequestHead {
        return RequestHead {
            method,
            uri,
            version,
            headers,
        };
    }

    pub fn content_length(&mut self) -> Option<usize> {
        if let Some(value) = self.headers.get("content-length") {
            if let Ok(length) = value.parse::<usize>() {
                return Some(length);
            }
        }
        None
    }

    pub fn content_type(&mut self) -> BodyType {
        if let Some(ctype) = self.headers.get("content-type") {
                return BodyType::MultiPart;
        else {
            None => return BodyType::None,
        }
    }
}

#[derive(Debug)]
pub struct ResponseHead {
    status: u16,
    version: String,
    headers: HashMap<String, String>,
}

impl ResponseHead {
    pub fn new(status: u16, version: String, headers: HashMap<String, String>) -> ResponseHead {
        return ResponseHead {
            status,
            version,
            headers,
        };
    }
}
