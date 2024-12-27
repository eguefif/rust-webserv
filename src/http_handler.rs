use crate::http_frame::{Error, RequestHead};
use crate::http_frame::{HttpFrame, Result};
use crate::parsers::http::HttpPacket;
use bytes::{Buf, Bytes, BytesMut};
use chrono::Utc;
use chrono::prelude::*;
use std::io::Cursor;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

#[derive(Debug)]
enum HttpState {
    Header,
    Body,
    Closed,
}

pub struct HttpConnection {
    stream: TcpStream,
    buf: BytesMut,
    state: HttpState,
}

impl HttpConnection {
    pub fn new(stream: TcpStream) -> HttpConnection {
        return HttpConnection {
            stream,
            buf: BytesMut::with_capacity(1024 * 4),
            state: HttpState::Header,
        };
    }

    pub async fn handle(&mut self) {
        loop {
            if let Ok(header) = self.get_header().await {
                if let Some(header) = header {
                    eprintln!(
                        "Header: {} {} {}\n{:?}",
                        header.method, header.uri, header.version, header.headers
                    );
                    self.send_response().await;
                }
            } else {
                eprintln!("Error while handling request");
            }
        }
    }

    async fn get_header(&mut self) -> Result<Option<RequestHead>> {
        loop {
            if let Some(header) = self.parse_header() {
                return Ok(Some(header));
            }

            if let Ok(n) = self.stream.read_buf(&mut self.buf).await {
                if self.buf.is_empty() && n == 0 {
                    return Ok(None);
                } else {
                    eprintln!("Error connection closed by peer");
                    return Err(Error::Other("Connection was closed by peer".to_string()));
                }
            } else {
                eprintln!("Error while reading buffer");
                return Err(Error::Other("Error while reading buffer".to_string()));
            }
        }
    }

    fn parse_header(&mut self) -> Option<RequestHead> {
        let mut buf = Cursor::new(&self.buf[..]);
        if let Ok(_) = HttpFrame::is_header_receive(&mut buf) {
            let len = buf.position();
            buf.set_position(0);
            let retval = Some(HttpFrame::parse_header(&mut buf, len as usize).unwrap());
            println!("Request: {:?}", self.buf);
            self.buf.advance(len as usize);
            self.state = HttpState::Body;
            return retval;
        } else {
            None
        }
    }

    async fn send_response(&mut self) {
        let response = self.create_response(String::from("Hello, World"));
        self.stream.write_all(response.as_bytes()).await.unwrap();
    }
    fn create_response(&mut self, body: String) -> String {
        let mut response = String::new();
        response.push_str("HTTP/1.1 200 OK\r\n");
        response.push_str(format!("Date: {}\r\n", get_time().as_str()).as_str());
        response.push_str(format!("Content-Length: {}\r\n", body.len()).as_str());
        response.push_str("Server: rust-webserv");
        response.push_str("\r\n\r\n");
        response.push_str(body.as_str());

        response
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
        3 => String::from("Mar"),
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
