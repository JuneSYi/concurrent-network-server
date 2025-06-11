use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt, Result, Error};
use tokio::task::spawn_blocking;
use std::io::ErrorKind;
use std::net::SocketAddr;
use std::str::from_utf8;
use crate::protocol;
use crate::worker::prime;

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
                    for (idx, &byte) in buffer[..bytes_read].iter().enumerate() {
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
                                if byte == protocol::PRIME_CMD {
                                    println!("Client {}: Saw PRIME_CMD (‘{}’), now collecting digits…", self.peer_addr, protocol::PRIME_CMD as char);
                                    let mut digits: Vec<u8> = Vec::new();
                                    let mut end_seen = false;
                                    let mut j = idx + 1;
                                    while j < bytes_read {
                                        let b = buffer[j];
                                        if b == protocol::END_MARKER {
                                            end_seen = true;
                                            break;
                                        } else if b.is_ascii_digit() {
                                            digits.push(b);
                                        } else {
                                            return Err(Error::new(ErrorKind::InvalidData, "non-digit in PRIME_CMD number"));
                                        }
                                        j += 1;
                                    }

                                    // Read the full number, continuing within fn read_ascii_number_prefill() if needed in case buffer only contains partial
                                    let n = self.read_ascii_number_prefill(digits, end_seen).await?;
                                    let result = spawn_blocking(move || prime::is_prime(n)).await?;
                                    let reply: &'static [u8] = if result { b"prime\n" } else { b"composite\n" };
                                    self.stream.write_all(reply).await?;
                                    println!("Client {}: Finished prime check for {}", self.peer_addr, n);
                                    self.state = ProcessingState::WaitForMsg;
                                    break;
                                }
                                if protocol::is_end(byte) {
                                    println!("Client {}: Received END_MARKER ('{}'). Transitioning to WaitForMsg", self.peer_addr, protocol::END_MARKER as char);
                                    self.state = ProcessingState::WaitForMsg;
                                } else {
                                    let response_byte = protocol::transform(byte);
                                    println!("Client {}: Transforming byte {} ('{}') to {} ('{}'). Sending response", self.peer_addr, byte, byte as char, response_byte, response_byte as char);
                                    self.stream.write_all(&[response_byte]).await?;
                                    continue;
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

    async fn read_ascii_number_prefill(&mut self, mut num_bytes_vec: Vec<u8>, end_seen: bool) -> Result<u64> {
        if !end_seen {
            loop {
                let byte = self.stream.read_u8().await?;
                if byte == protocol::END_MARKER {
                    break;
                } else if byte.is_ascii_digit() {
                    num_bytes_vec.push(byte);
                    if num_bytes_vec.len() > 20 {
                        return Err(Error::new(ErrorKind::InvalidInput, "number of digits exceed u64 limit (PRIME_CMD number)"));
                    }
                } else {
                    return Err(Error::new(ErrorKind::InvalidData, "non-digit received while expecting number for PRIME_CMD"));
                }
            }
        }

        if num_bytes_vec.is_empty() {
            return Err(Error::new(ErrorKind::InvalidInput, "no digits received for PRIME_CMD"));
        }

        let num_str = from_utf8(&num_bytes_vec).map_err(|e| Error::new(ErrorKind::InvalidData, e))?;
        let number = num_str.parse::<u64>().map_err(|e| Error::new(ErrorKind::InvalidInput, e))?;
        Ok(number)
    }
}
