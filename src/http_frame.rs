use bytes::Bytes;
use std::collections::HashMap;
use std::io::Cursor;

#[derive(Debug)]
pub enum HttpFrame {
    RequestHead(RequestHead),
    ResponseHead(ResponseHead),
    BodyChunk(Bytes),
}

#[derive(Debug)]
pub enum Error {
    Incomplete,
    Other(String),
}

pub type Result<T> = std::result::Result<T, Error>;

impl HttpFrame {
    pub fn is_header_receive(buf: &mut Cursor<&[u8]>) -> Result<()> {
        let mut position = buf.position() as usize;
        if position < 4 {
            return Err(Error::Incomplete);
        }
        let raw_data = buf.get_ref();
        while position > 4 {
            if raw_data[position] == 10
                && raw_data[position - 1] == 13
                && raw_data[position - 2] == 10
                && raw_data[position - 3] == 13
            {
                buf.set_position(position as u64);
                return Ok(());
            }
            position -= 1;
        }

        Err(Error::Incomplete)
    }

    pub fn parse_header(buff: &mut Cursor<&[u8]>, end: usize) -> Result<HttpFrame> {
        let method;
        let uri;
        let version;
        if let Some(request_line) = HttpFrame::get_next_line(buff, end) {
            (method, uri, version) = HttpFrame::get_request_line(request_line.as_str());
        } else {
            return Err(Error::Incomplete);
        }
        let mut headers = HashMap::new();
        while let Some(line) = HttpFrame::get_next_line(buff, end) {
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
        Ok(HttpFrame::RequestHead(RequestHead::new(
            method, uri, version, headers,
        )))
    }

    fn get_next_line(buff: &mut Cursor<&[u8]>, end: usize) -> Option<String> {
        let start = buff.position() as usize;
        for i in start..(end - 1) {
            if buff.get_ref()[i] == 13 && buff.get_ref()[i + 1] == 10 {
                buff.set_position((i + 2) as u64);
                let retval = buff.get_ref()[start..i].to_vec();
                return Some(String::from_utf8(retval));
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
    method: String,
    uri: String,
    version: String,
    headers: HashMap<String, String>,
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
