use crate::http_frame::HttpFrame;
use crate::http_frame::RequestHead;
use bytes::{Buf, BytesMut};
use chrono::Utc;
use chrono::prelude::*;
use std::fs;
use std::io::Cursor;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

pub struct HttpConnection {
    stream: TcpStream,
    buf: BytesMut,
}
impl HttpConnection {
    pub fn new(stream: TcpStream) -> HttpConnection {
        return HttpConnection {
            stream,
            buf: BytesMut::with_capacity(1024 * 4),
        };
    }

    pub async fn handle(&mut self) {
        loop {
            match self.get_header().await {
                Ok(header_result) => {
                    if let Some(header) = header_result {
                        eprintln!(
                            "Header: {} {} {}\n{:?}\n",
                            header.method, header.uri, header.version, header.headers
                        );
                        match self.create_response_body(header.uri) {
                            Ok(body) => self.send_response(body, 200).await,
                            Err(error) => self.send_error(error).await,
                        }
                    } else {
                        eprintln!("Ending connection");
                        break;
                    }
                }
                Err(e) => {
                    eprintln!("Error while handling request: {:?}", e);
                    break;
                }
            }
        }
    }

    async fn get_header(&mut self) -> Result<Option<RequestHead>, String> {
        loop {
            if let Some(header) = self.try_parse_header()? {
                return Ok(Some(header));
            }

            match self.stream.read_buf(&mut self.buf).await {
                Ok(n) => {
                    if n == 0 {
                        if self.buf.is_empty() {
                            return Ok(None);
                        } else {
                            return Err("Connection was closed by peer".to_string());
                        }
                    }
                }
                Err(e) => {
                    return Err(format!("Error while reading buffer: {}", e));
                }
            }
        }
    }

    fn try_parse_header(&mut self) -> Result<Option<RequestHead>, String> {
        let mut buf = Cursor::new(&self.buf[..]);
        if let Some(_) = HttpFrame::is_header_receive(&mut buf) {
            let len = buf.position();
            buf.set_position(0);
            let retval = Some(HttpFrame::parse_header(&mut buf, len as usize)?);
            self.buf.advance(len as usize);
            return Ok(retval);
        } else {
            Ok(None)
        }
    }

    fn create_response_body(&mut self, uri: String) -> Result<Vec<u8>, u32> {
        let path;
        if uri == "/" {
            path = format!("./html/index.html");
        } else {
            path = format!("./html/{}", uri);
        }
        if let Ok(retval) = fs::read(path) {
            return Ok(retval);
        } else {
            return Err(404);
        }
    }

    async fn send_response(&mut self, body: Vec<u8>, status_code: u32) {
        let response = self.create_response(body.len(), status_code, "Keep-Alive".to_string());
        eprintln!("Sending: {}\n", response);
        self.stream.write_all(response.as_bytes()).await.unwrap();
        if !body.is_empty() {
            self.stream.write_all(&body).await.unwrap();
        }
    }

    async fn send_error(&mut self, error: u32) {
        let uri = match error {
            400 => String::from("400.html"),
            404 => String::from("404.html"),
            415 => String::from("415.html"),
            500 => String::from("500.html"),
            _ => String::from("500.html"),
        };
        let body = self.create_response_body(uri).unwrap();
        let header = self.create_response(body.len(), error, "Keep-Alive".to_string());
        eprintln!("Sending: {}\n", header);
        self.stream.write_all(header.as_bytes()).await.unwrap();
        if !body.is_empty() {
            self.stream.write_all(&body).await.unwrap();
        }
    }
    fn create_response(&mut self, len: usize, code: u32, connection: String) -> String {
        let mut response = String::new();
        response.push_str(format!("HTTP/1.1 {}\r\n", self.get_status_code(code)).as_str());
        response.push_str(format!("Date: {}\r\n", get_time().as_str()).as_str());
        response.push_str(format!("Content-Length: {}\r\n", len).as_str());
        response.push_str(format!("Connection: {}\r\n", connection).as_str());
        response.push_str("Server: rust-webserv");
        response.push_str("\r\n\r\n");

        response
    }

    fn get_status_code(&self, code: u32) -> String {
        match code {
            200 => "200 OK".to_string(),
            400 => "200 Bad Request".to_string(),
            404 => "200 Not Found".to_string(),
            415 => "200 Unsupported Media".to_string(),
            500 => "200 Internal Error".to_string(),
            _ => "500 Internal Error".to_string(),
        }
    }

    pub async fn send_close(&mut self) {
        let header = self.create_response(0, 200, "close".to_string());
        self.stream.write_all(header.as_bytes()).await.unwrap();
    }
}
fn get_time() -> String {
    let date = Utc::now();
    format!(
        "{}, {} {} {} {}:{}:{} GMT",
        date.weekday(),
        date.day(),
        get_month(date.month()),
        date.year(),
        date.hour(),
        date.minute(),
        date.second()
    )
}

fn get_month(month: u32) -> String {
    match month {
        1 => String::from("Jan"),
        2 => String::from("Feb"),
        3 => String::from("Mar4"),
        4 => String::from("Apr"),
        5 => String::from("May"),
        6 => String::from("Jun"),
        7 => String::from("Jul"),
        8 => String::from("Aug"),
        9 => String::from("Sep"),
        10 => String::from("Oct"),
        11 => String::from("Nov"),
        12 => String::from("Dec"),
        _ => String::from("Error month"),
    }
}
