use std::io::{BufRead, BufReader, Error, ErrorKind, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;

#[derive(Debug)]
enum ProcessingState {
    WaitForMsg,
    InMsg,
}

pub struct Server {
    addr: String,
}

impl Server {
    pub fn new(addr: &str) -> Self {
        Server {
            addr: addr.to_string(),
        }
    }

    pub fn run(&self) -> Result<(), Error> {
        let listener = TcpListener::bind(&self.addr)?;

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let peer_addr = stream.peer_addr()?;
                    println!("New connection accepted from: {:?}", peer_addr);

                    thread::spawn(move || {
                        println!("Thread spawned for: {peer_addr:?}");
                        if let Err(e) = handle_client_in_thread(stream) {
                            eprintln!("Error handling connecion: {e}. Peer done.");
                        }
                        println!("thread for client {peer_addr:?} finished"); // thread dropped because thread now stored in a variable
                    });
                }
                Err(e) => {
                    eprintln!("Failed to accept connection: {e}");
                }
            }
        }
        Ok(())
    }
}

fn handle_client_in_thread(mut stream: TcpStream) -> Result<(), Error> {
    println!("client connected. sending initial '*' character");
    stream.write_all(b"*")?;
    
    let mut state = ProcessingState::WaitForMsg;
    let mut buffer = [0u8; 1024];
    
    println!("entering protocol loop. initial state: WaitForMsg");
    loop {
        // read data from client stream
        match stream.read(&mut buffer) {
            Ok(0) => {
                // connection closed by client
                println!("client disconnected");
                return Ok(())
            }
            Ok(bytes_read) => {
                // process the bytes received
                for i in 0..bytes_read {
                    let byte = buffer[i];
                    // println!("State: {state:?}, Received byte: {byte} ('{}')", byte as char);

                    match state {
                        ProcessingState::WaitForMsg => {
                            if byte == b'^' {
                                println!("Received '^', transitioning to InMsg state.");
                                state = ProcessingState::InMsg;
                            }
                        }
                        ProcessingState::InMsg => {
                            if byte == b'$' {
                                println!("Received '$', transitioning back to WaitForMsg state.");
                                state = ProcessingState::WaitForMsg;
                            } else {
                                // increment byte by 1 and send back
                                let response_byte = byte.wrapping_add(1);
                                // println!("InMsg: Sending byte {response_byte} ('{}')", response_byte as char);

                                if stream.write_all(&[response_byte]).is_err() {
                                    eprintln!("Error sending response byte.");
                                    return Err(Error::new(ErrorKind::Other, "Failed to send response byte"));
                                }
                            }
                        }
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