use crate::http_connection::HttpConnection;
use tokio::net::{TcpListener, TcpStream};
use tokio::select;
use tokio::signal;
use tokio::task::JoinHandle;
use tokio::time::{Duration, sleep};
use tokio_util::sync::CancellationToken;

pub mod http_connection;
pub mod http_frame;
pub mod multipart_parser;

#[tokio::main]
async fn main() {
    let cancellation_token = CancellationToken::new();
    let token = cancellation_token.clone();
    let server_task = tokio::spawn(async move { worker(token).await });
    tokio::spawn(async move {
        signal::ctrl_c().await.unwrap();
        cancellation_token.cancel();
    });

    while !server_task.is_finished() {
        sleep(Duration::from_millis(100)).await;
    }
}

async fn worker(cancellation_token: CancellationToken) {
    let mut tasks: Vec<JoinHandle<()>> = vec![];
    if let Ok(server) = TcpListener::bind("127.0.0.1:8888").await {
        eprintln!("Listening on 127.0.0.1:8888\n");
        loop {
            select! {
                task = listen_task(&server, cancellation_token.clone()) => {
                    match task{
                        Some(value) => tasks.push(value),
                        None => {}
                    }
                }
                _ = cancellation_token.cancelled() => {
                        eprintln!("Closing server gracefully");
                        break
                }
            }
        }
    } else {
        eprintln!("Impossible to listen on 127.0.0.1:8888");
    }
    for task in tasks.iter() {
        while !task.is_finished() {}
    }
}

async fn listen_task(
    server: &TcpListener,
    cancellation_token: CancellationToken,
) -> Option<JoinHandle<()>> {
    if let Ok((socket, _)) = server.accept().await {
        eprintln!("New connection: {}\n", socket.peer_addr().unwrap());
        return Some(tokio::spawn(async move {
            process(socket, cancellation_token).await
        }));
    } else {
        eprintln!("Impossible to accept socket on 127.0.0.:8888");
        return None;
    }
}

async fn process(socket: TcpStream, cancellation_token: CancellationToken) {
    let peer = socket.peer_addr();
    let mut connection = HttpConnection::new(socket);
    select! {
        _ = connection.handle()=> {}
        _ = cancellation_token.cancelled() => {
                sleep(Duration::from_millis(500)).await;
                connection.send_close().await;
                eprintln!("Terminating connection with peer: {:?}", peer);
            }
    }
}
