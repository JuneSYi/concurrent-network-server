use std::io::{Error, ErrorKind, Read, Write};
use std::net::TcpStream;
use crate::protocol;

#[derive(Debug)]
enum ProcessingState {
    WaitForMsg,
    InMsg,
}

pub struct ConnectionHandler {
    stream: TcpStream,
    state: ProcessingState,
}

impl ConnectionHandler {
    pub fn new(stream: TcpStream) -> Self {
        ConnectionHandler { stream, state: ProcessingState::WaitForMsg }
    }

    pub fn handle(mut self) -> Result<(), std::io::Error> {
        println!("client connected. sending initial '*' character");
        self.stream.write_all(&[protocol::HANDSHAKE])?;

        let mut buffer = [0u8; 1024];

        println!("entering protocol loop. initial state: WaitForMsg");
        loop {
            // read data from stream
            match stream.read(&mut buffer) {
                Ok(0) => {
                    // connection closed by client
                    println!("client disconnected");
                    return Ok(())
                }
                Ok(bytes_read) => {
                    // process the bytes received
                    for &byte in &buffer[..bytes_read] {
                        match self.state {
                            ProcessingState::WaitForMsg if protocol::is_start(byte) => {
                                println!("Received '^', transitioning to InMsg state.");
                                self.state = ProcessingState::InMsg;
                            }
                            ProcessingState::InMsg if protocol::is_end(byte) => {
                                println!("Received '$', transitioning back to WaitForMsg state.");
                                self.state = ProcessingState::WaitForMsg;
                            }
                            ProcessingState::InMsg => {
                                // increment byte by 1 and send back
                                let response_byte = protocol::transform(byte);
                                // println!("InMsg: Sending byte {response_byte} ('{}')", response_byte as char);
                                self.stream.write_all(&[response_byte])?;
                            }
                            _ => {}
                        }
                    }
                }
                Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                    // error typically for non-block sockets
                    eprintln!("this error occurred because the operation would need to block");
                    continue;
                }
                Err(e) => {
                    eprintln!("Error reading from stream: {e}");
                    return Err(e);
                }
            }
        }
    }
}