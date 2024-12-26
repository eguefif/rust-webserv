use crate::http_frame::Error;
use crate::http_frame::{HttpFrame, Result};
use crate::parsers::http::HttpPacket;
use bytes::{Buf, BytesMut};
use chrono::Utc;
use chrono::prelude::*;
use std::io::Cursor;
use tokio::io::AsyncReadExt;
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

    pub async fn read_frame(&mut self) -> Result<Option<HttpFrame>> {
        loop {
            if let Some(frame) = self.parse_frame()? {
                return Ok(Some(frame));
            }
            if let Ok(n) = self.stream.read_buf(&mut self.buf).await {
                if self.buf.is_empty() && n == 0 {
                    return Ok(None);
                } else {
                    return Err(Error::Other("Connection was closed by peer".to_string()));
                }
            } else {
                return Err(Error::Other("Error while reading socket".to_string()));
            }
        }
    }

    fn parse_frame(&mut self) -> Result<Option<HttpFrame>> {
        let mut buf = Cursor::new(&self.buf[..]);
        if let Ok(_) = HttpFrame::is_header_receive(&mut buf) {
            let len = buf.position();
            buf.set_position(0);
            let retval = HttpFrame::parse_header(&mut buf, len as usize)?;
            self.buf.advance(len as usize);
            return Ok(Some(retval));
        }
        return Ok(None);
    }
}

fn is_connection_closing(http_packet: &HttpPacket) -> bool {
    if let Some(connection) = http_packet.headers.get("connection") {
        return connection != "keep-alive";
    }
    true
}

fn create_response(body: String) -> String {
    let mut response = String::new();
    response.push_str("HTTP/1.1 200 OK\r\n");
    response.push_str(format!("Date: {}\r\n", get_time().as_str()).as_str());
    response.push_str(format!("Content-Length: {}\r\n", body.len()).as_str());
    response.push_str("Server: rust-webserv");
    response.push_str("\r\n\r\n");
    response.push_str(body.as_str());

    response
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
