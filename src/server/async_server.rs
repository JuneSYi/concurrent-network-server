use tokio::net::TcpListener;
use tokio::io::Result;
use tokio::task;
use crate::handler::async_handler::AsyncConnectionHandler;

pub struct AsyncServer {
    addr: String
}

impl AsyncServer {
    pub fn new(addr: &str) -> Self {
        Self { addr: addr.to_string() }
    }

    pub async fn run(&self) -> Result<()> {
        let listener = TcpListener::bind(&self.addr).await?;
        println!("async server listening on {}", self.addr);
        loop {
            let (socket, peer) = listener.accept().await?;
            println!("new peer: {peer}");
            task::spawn(async move {
                let mut h = AsyncConnectionHandler::new(socket, peer);
                if let Err(e) = h.run().await {
                    eprintln!("client {peer} error: {e}");
                }
            });
        }
    }
}