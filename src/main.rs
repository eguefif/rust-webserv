use crate::http_handler::handle_packet;
use std::error::Error;
use tokio::net::{TcpListener, TcpStream};

pub mod http_handler;
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
    loop {
        if let Ok(_) = socket.readable().await {
            let mut buff = [0; 10];
            match socket.try_read(&mut buff[..]) {
                Ok(n) => {
                    if n == 0 {
                        eprintln!("Socket closed by client");
                    }
                    raw_packet.extend_from_slice(&buff);
                    if !handle_packet(&raw_packet, socket).await {
                        eprintln!("Connection closed");
                    }
                }
                Err(_) => break,
            }
        } else {
            eprintln!("Socket not readable");
        }
    }
}
