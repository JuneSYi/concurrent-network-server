use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt, Result};
use std::net::SocketAddr;
use crate::protocol;

#[derive(Debug)]
enum ProcessingState {
    InitialAck,
    WaitForMsg,
    InMsg,
}

pub struct AsyncConnectionHandler {
    stream: TcpStream,
    peer_addr: SocketAddr,
    state: ProcessingState,
}

impl AsyncConnectionHandler {
    pub fn new(stream: TcpStream, peer_addr: SocketAddr) -> Self {
        AsyncConnectionHandler { stream, peer_addr, state: ProcessingState::InitialAck }
    }

    pub async fn run(&mut self) -> Result<()> {
        println!("AsyncConnectionHandler started for client: {}", self.peer_addr);

        if let ProcessingState::InitialAck = self.state {
            println!("Client {}: State InitialAck. Sending HANDSHAKE ('{}')", self.peer_addr, protocol::HANDSHAKE as char);
            self.stream.write_all(&[protocol::HANDSHAKE]).await?;
            self.state = ProcessingState::WaitForMsg;
            println!("Client {}: HANDSHAKE sent. Transitioning to state WaitForMsg", self.peer_addr);
        }

        let mut buffer = [0u8; 1024];

        loop {
            match self.stream.read(&mut buffer).await {
                Ok(0) => {
                    println!("Client {}: disconnected gracefully", self.peer_addr);
                    return Ok(());
                }
                Ok(bytes_read) => {
                    for &byte in &buffer[..bytes_read] {
                        match self.state {
                            ProcessingState::WaitForMsg => {
                                if protocol::is_start(byte) {
                                    println!("Client {}: Received START_MARKER ('{}'). Transitioning to InMsg", self.peer_addr, protocol::START_MARKER as char);
                                    self.state = ProcessingState::InMsg;
                                } else {
                                    println!("Client {}: Received unexpected byte {} ('{}') in WaitForMsg. Ignoring", self.peer_addr, byte, byte as char);
                                }
                            }
                            ProcessingState::InMsg => {
                                if protocol::is_end(byte) {
                                    println!("Client {}: Received END_MARKER ('{}'). Transitioning to WaitForMsg", self.peer_addr, protocol::END_MARKER as char);
                                    self.state = ProcessingState::WaitForMsg;
                                } else {
                                    let response_byte = protocol::transform(byte);
                                    println!("Client {}: Transforming byte {} ('{}') to {} ('{}'). Sending response", self.peer_addr, byte, byte as char, response_byte, response_byte as char);
                                    self.stream.write_all(&[response_byte]).await?;
                                }
                            }
                            ProcessingState::InitialAck => unreachable!(),
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Client {}: Error reading from stream: {}", self.peer_addr, e);
                    return Err(e.into());
                }
            }
        }
    }
}