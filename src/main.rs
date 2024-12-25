use crate::http_handler::handle_packet;
use tokio::net::{TcpListener, TcpStream};
use tokio::select;
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
            let cloned_token = token.clone();
            if let Ok((mut socket, _)) = server.accept().await {
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
    let mut raw_packet: Vec<u8> = vec![];
    loop {
        if let Ok(_) = socket.readable().await {
            let mut buff = [0; 10];
            match socket.try_read(&mut buff[..]) {
                Ok(n) => {
                    if n == 0 {
                        eprintln!("Socket closed by client");
                        break;
                    }
                    raw_packet.extend_from_slice(&buff);
                    if !handle_packet(&raw_packet, socket).await {
                        eprintln!("Connection closed");
                    }
                }
                Err(_) => raw_packet.clear(),
            }
        } else {
            eprintln!("Socket not readable");
        }
    }
    eprintln!("Closing socket");
}
