use crate::http_handler::{HttpState, handle_packet};
use tokio::io::AsyncReadExt;
use tokio::net::{TcpListener, TcpStream};
use tokio::signal;
use tokio_util::sync::CancellationToken;

pub mod http_handler;
pub mod parsers;

#[tokio::main]
async fn main() {
    let token = CancellationToken::new();
    let cloned_token = token.clone();
    let w = tokio::spawn(async move { worker(cloned_token).await });
    loop {
        match signal::ctrl_c().await {
            Ok(_) => {
                token.cancel();
                eprintln!("Gracefully shutdown");
                break;
            }
            _ => {}
        }
    }
}

async fn worker(token: CancellationToken) {
    if let Ok(server) = TcpListener::bind("127.0.0.1:8888").await {
        eprintln!("Listening on 127.0.0.1:8888");
        let mut tasks = Vec::new();
        loop {
            if let Ok((mut socket, _)) = server.accept().await {
                eprintln!("New connection: {}", socket.peer_addr().unwrap());
                let cloned_token = token.clone();
                tasks.push(tokio::spawn(async move {
                    process(&mut socket, cloned_token).await
                }));
            } else {
                eprintln!("Impossible to accept socket on 127.0.0.:8888");
            }
        }
    } else {
        eprintln!("Impossible to listen on 127.0.0.1:8888");
    }
}

async fn process(socket: &mut TcpStream, token: CancellationToken) {
    let mut raw_packet: Vec<u8> = Vec::with_capacity(1024 * 50);
    loop {
        let mut buff = [0; 1024];
        match socket.read(&mut buff[..]).await {
            Ok(0) => {
                eprintln!("Socket closed by client");
                break;
            }
            Ok(n) => {
                raw_packet.extend_from_slice(&buff[..n]);
                match handle_packet(&raw_packet, socket).await {
                    HttpState::Receiving => {}
                    HttpState::Closed => {
                        eprintln!("Connection closed");
                        break;
                    }

                    HttpState::Handled => raw_packet.clear(),
                }
            }
            Err(_) => raw_packet.clear(),
        }
    }
    eprintln!("Closing socket");
}
