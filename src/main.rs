use kvds::app_server::socket_server::AppServer;
use std::env;

#[tokio::main]
async fn main() -> tokio::io::Result<()> {
    let mut port = String::from("6379");
    let mut args = env::args().into_iter();
    let mut persist = false;
    while let Some(arg) = args.next() {
        if arg == "port" {
            port = args.next().expect("wrong port number!");
        }
        if arg == "PERSIST" {
            persist = args.next().expect("wrong boolean!") == "true";
        }
    }
    AppServer::new(port.as_str(), persist).start().await
}
