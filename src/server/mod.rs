use std::io::{BufRead, BufReader, Error, Read, Write};
use std::net::{TcpListener, TcpStream};

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
        // https://doc.rust-lang.org/std/net/struct.TcpListener.html
        let listener = TcpListener::bind(&self.addr)?;

        for stream in listener.incoming() {
            let stream = stream?;
            println!("Connection established");
            self.handle_client(stream);
        }
        Ok(())
    }

    fn handle_client(&self, mut stream: TcpStream) -> Result<(), Error> {
        println!("running fn handle_client()");
        let mut buffer = [0; 512];
        loop {
            let nbytes = stream.read(&mut buffer)?;
            if nbytes == 0 {
                println!("closing connection");
                return Ok(());
            }
            stream.write_all(&buffer[..nbytes])?;
        }
    }
}
