use std::io::{Error, ErrorKind, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::sync::{mpsc, Arc, Mutex};
use std::time::Duration;

#[derive(Debug)]
enum ProcessingState {
    WaitForMsg,
    InMsg,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    // fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Box<dyn FnOnce() + Send + 'static>>>>) -> Self {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Self {
        let thread = thread::spawn(move || loop {
            let message = receiver.lock().unwrap().recv(); // lock, then receive

            match message {
                Ok(job) => {
                    println!("Worker {id} got a job; executing.");
                    job();
                }
                Err(_) => {
                    // Sender offline or channel is broken
                    println!("Worker {id} disconnecting; sender offline");
                    break;
                }
            }
        });

        Worker {
            id,
            thread: Some(thread),
        }
    }
}

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}

impl ThreadPool {
    pub fn new(size: usize) -> Self {
        assert!(size > 0);
        let (tx, rx) = mpsc::channel();
        let rx = Arc::new(Mutex::new(rx));
        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&rx)));
        }

        ThreadPool { workers, sender: Some(tx) }
    }
    
    pub fn execute<F>(&self, f: F)
    where F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        if let Some(tx) = &self.sender {
            if tx.send(job).is_err() {
                eprintln!("Error: failed to send job to worker. channel may be offline");
            }
        }
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take());
        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.id);
            if let Some(thread) = worker.thread.take() {
                if thread.join().is_err() {
                    eprintln!("Error joining thread for worker {}", worker.id);
                }
            }
        }
        println!("all workers shut down");
    }
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

        let poolsize = 4;
        let pool = ThreadPool::new(poolsize);
        println!("thread pool with {} workers created", poolsize);

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let peer_addr = stream.peer_addr().map_or_else(
                        |_| "unknown".to_string(),
                        |f| f.to_string()
                    );
                    println!("New connection accepted from: {:?}", peer_addr);

                    pool.execute(move || {
                        println!("Job for {} dispatched to thread pool.", peer_addr);
                        // if let Err(e) = handle_client_in_thread(stream) {
                        //     eprintln!("Error handling connecion: {}: {e}", peer_addr);
                        // }
                        
                        // To run with TcpStream set to non-blocking mod
                        if let Err(e) = handle_connection_non_blocking_raw(stream) {
                            eprintln!("Error handling connecion (in non-blocking mode): {}: {e}", peer_addr);
                        } else {
                            println!("Peer done (non-blocking mode)");
                        }
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

fn handle_connection_non_blocking_raw(mut stream: TcpStream) -> Result<(), Error> {
        println!("Accepted connection. Setting to non-blocking");
        stream.set_nonblocking(true)?;

        let mut buffer = [0u8; 1024];

        loop {
            match stream.read(&mut buffer) {
                Ok(0) => {
                    println!("client disconnected gracefully");
                    return Ok(());
                },
                Ok(bytes_read) => {
                    println!("recv return {bytes_read} bytes");
                    // for a real app, buffer processed here
                },
                Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                    println!("Operation would block. Sleeping for 200ms...");
                    thread::sleep(Duration::from_millis(200));
                    continue;
                },
                Err(e) => {
                    eprintln!("Error reading from stream: {e}");
                    return Err(e);
                }
            }
        }
    }
