use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

use crate::app_server::parser::parse_command;
use crate::services::command_handler::handle;
use crate::services::persistence_service;

pub struct AppServer {
    port: String,
}

impl AppServer {
    pub fn new(port: &str) -> Self {
        AppServer {
            port: port.to_string(),
        }
    }

    pub async fn start(&self) -> tokio::io::Result<()> {
        persistence_service::load_data().await;
        let url = format!("127.0.0.1:{}", self.port);
        let listener = TcpListener::bind(&url).await?;
        println!("Async server running on {url}");

        loop {
            match listener.accept().await {
                Ok((socket, addr)) => {
                    println!("New client: {}", addr);
                    tokio::spawn(async move {
                        handle_client(socket).await;
                    });
                }
                Err(e) => eprintln!("Failed to accept client: {}", e),
            }
        }
    }
}

async fn handle_client(mut socket: TcpStream) {
    let mut buf = [0; 1024];
    loop {
        match socket.read(&mut buf).await {
            Ok(0) => {
                println!("Client disconnected");
                return;
            }
            Ok(n) => {
                let received = String::from_utf8_lossy(&buf[..n]).into_owned();
                let cmd = parse_command(received.chars());
                let mut resp = match cmd {
                    Ok(req) => handle(req).await,
                    Err(e) => format!("-ERR unknown command: {e}"),
                };
                resp.push_str("\r\n");
                socket.write_all(resp.as_bytes()).await.unwrap();
            }
            Err(e) => {
                eprintln!("Failed to read from socket: {}", e);
                return;
            }
        }
    }
}
