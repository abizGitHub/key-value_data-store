use kvds::app_server::socket_server::AppServer;

#[tokio::main]
async fn main() -> tokio::io::Result<()> {
    AppServer::new("7878").start().await
}
