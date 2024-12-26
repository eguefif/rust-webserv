use crate::parsers::http::HttpPacket;
use chrono::Utc;
use chrono::prelude::*;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

pub enum HttpState {
    Receiving,
    Handled,
    Closed,
}

pub async fn handle_packet(raw_packet: &Vec<u8>, socket: &mut TcpStream) -> HttpState {
    if raw_packet.len() < 4 || !is_header_receive(raw_packet) {
        eprintln!("raw: {:?}", raw_packet);
        eprintln!("");
        return HttpState::Receiving;
    }
    let packet = String::from_utf8(raw_packet.clone())
        .expect("Error while converting bytes into string (reading)");
    let http_packet = HttpPacket::new(&packet);
    eprintln!("Http request line : {:?}", http_packet.request_line);
    eprintln!("Http headers : {:?}", http_packet.headers);
    if is_connection_closing(&http_packet) {
        return HttpState::Closed;
    }
    let response = create_response(String::from("Hello, World!"));
    socket
        .write_all(response.as_bytes())
        .await
        .expect("Error while writing in socket");
    eprintln!("Sending");
    return HttpState::Handled;
}

fn is_connection_closing(http_packet: &HttpPacket) -> bool {
    if let Some(connection) = http_packet.headers.get("connection") {
        return connection != "keep-alive";
    }
    true
}

fn is_header_receive(raw_packet: &Vec<u8>) -> bool {
    let idx = raw_packet.len() - 4;
    raw_packet[idx] == 13
        && raw_packet[idx + 1] == 10
        && raw_packet[idx + 2] == 13
        && raw_packet[idx + 3] == 10
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
