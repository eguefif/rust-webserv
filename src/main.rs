use crate::parsers::http::HttpPacket;
use std::error::Error;
use tokio::io::AsyncWriteExt;
use tokio::net::{TcpListener, TcpStream};

pub mod parsers;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let server = TcpListener::bind("127.0.0.1:8888").await?;
    loop {
        let (mut socket, _) = server.accept().await?;
        tokio::spawn(async move { process(&mut socket).await });
    }
}

async fn process(socket: &mut TcpStream) {
    let mut raw_packet: Vec<u8> = vec![];
    let mut nbytes = 0;
    loop {
        if let Ok(_) = socket.readable().await {
            let mut buff = [0; 10];
            match socket.try_read(&mut buff[..]) {
                Ok(n) => nbytes += n,
                Err(_) => break,
            }
            raw_packet.extend_from_slice(&buff);
        } else {
            eprintln!("Socket not readable");
        }
    }
    let packet =
        String::from_utf8(raw_packet).expect("Error while converting bytes into string (reading)");
    let http_packet = HttpPacket::new(&packet);
    eprintln!("Receive: {} bytes", nbytes);
    eprintln!("Http request line : {:?}", http_packet.request_line);
    eprintln!("Http headers : {:?}", http_packet.headers);
    let response = create_response(http_packet, String::from("Hello, World"));
    eprintln!("sending: {:?}", response.as_bytes());
    socket
        .write_all(response.as_bytes())
        .await
        .expect("Error while writing in socket");
}

fn create_response(http_packet: HttpPacket, body: String) -> String {
    let mut response = String::new();
    response.push_str("HTTP/1.1 200 OK\r\n");
    response.push_str(format!("Date: {}\r\n", get_time().as_str()).as_str());
    response.push_str(format!("Content-Length: {}, \r\n", body.len()).as_str());
    response.push_str("Server: rust-webserv");
    response.push_str("\r\n\r\n");
    response.push_str(body.as_str());

    response
}

fn get_time() -> String {
    // TODO: set date properly
    String::from("Wed, 25 Dec 2024 15:10:17 GMT")
}
