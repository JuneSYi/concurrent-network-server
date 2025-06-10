use concurrent_network_server::server::{sync_server, async_server};
use tokio::io::Result;

#[tokio::main]
async fn main() -> Result<()> {
    async_server::AsyncServer::new("127.0.0.1:7878").run().await
    // let server = sync_server::Server::new("127.0.0.1:7878");
    // if let Err(e) = server.run() {
    //     println!("Error while trying to run server");
    //     std::process::exit(1);
    // }
}
