use crate::http_frame::HttpFrame;
use crate::http_handler::HttpConnection;
use tokio::net::{TcpListener, TcpStream};

pub mod http_frame;
pub mod http_handler;
pub mod parsers;

#[tokio::main]
async fn main() {
    tokio::spawn(async move { worker().await });
}

async fn worker() {
    if let Ok(server) = TcpListener::bind("127.0.0.1:8888").await {
        eprintln!("Listening on 127.0.0.1:8888");
        let mut tasks = Vec::new();
        loop {
            if let Ok((socket, _)) = server.accept().await {
                eprintln!("New connection: {}", socket.peer_addr().unwrap());
                tasks.push(tokio::spawn(async move { process(socket).await }));
            } else {
                eprintln!("Impossible to accept socket on 127.0.0.:8888");
            }
        }
    } else {
        eprintln!("Impossible to listen on 127.0.0.1:8888");
    }
}

async fn process(socket: TcpStream) {
    let mut connection = HttpConnection::new(socket);
    while Some(frame) = connection.read_frame().await.unwrap() {}
}
