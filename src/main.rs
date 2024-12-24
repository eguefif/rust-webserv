use std::collections::HashMap;
use tokio::net::{TcpListener, TcpStream};

struct HttpPacket {
    request_line: HashMap<String, String>,
    headers: HashMap<String, String>,
    body: Vec<u8>,
}

impl HttpPacket {
    pub fn new(packet: &str) -> HttpPacket {
        let (request_line, headers) = parse_header(packet);
        HttpPacket {
            request_line,
            headers,
            body: vec![],
        }
    }
}

fn parse_header(packet: &str) -> (HashMap<String, String>, HashMap<String, String>) {
    let mut headers: HashMap<String, String> = HashMap::new();
    let mut lines = packet.lines();
    let request_line = get_request_line(lines.next().expect("Error while parsing request line"));
    lines.for_each(|line| {
        if line.len() != 0 {
            let mut splits = line.split(":");
            let key = splits
                .next()
                .expect("Error while parsing headers")
                .trim()
                .to_lowercase();
            let content = splits
                .next()
                .expect("Error while parsing headers")
                .trim()
                .to_lowercase();
            headers.insert(key, content);
        }
    });
    (request_line, headers)
}

fn get_request_line(request_line_str: &str) -> HashMap<String, String> {
    let mut splits = request_line_str.split(" ");
    let mut request_line = HashMap::new();
    let method = String::from(
        splits
            .next()
            .expect("Error while parsing request line method"),
    );
    let path = String::from(
        splits
            .next()
            .expect("Error while parsing request line path"),
    );
    let version = String::from(
        splits
            .next()
            .expect("Error while parsing request line protocol"),
    );
    request_line.insert(String::from("method"), method);
    request_line.insert(String::from("path"), path);
    request_line.insert(String::from("version"), version);

    request_line
}

#[tokio::main]
async fn main() {
    let server = TcpListener::bind("127.0.0.1:8888").await.unwrap();
    loop {
        let (socket, _) = server.accept().await.unwrap();
        tokio::spawn(async move { process(socket).await });
    }
}

async fn process(socket: TcpStream) {
    let mut raw_packet: Vec<u8> = vec![];
    let mut nbytes = 0;
    loop {
        socket.readable().await.unwrap();
        let mut buff = [0; 10];
        match socket.try_read(&mut buff[..]) {
            Ok(n) => nbytes += n,
            Err(_) => break,
        }
        raw_packet.extend_from_slice(&buff);
    }
    let packet =
        String::from_utf8(raw_packet).expect("Error while converting bytes into string (reading)");
    let http_packet = HttpPacket::new(&packet);
    eprintln!("Http request line : {:?}", http_packet.request_line);
    eprintln!("Http headers : {:?}", http_packet.headers);
}
