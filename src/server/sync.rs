use std::io::Error;
use std::net::TcpListener;
use std::thread;
use std::sync::{mpsc, Arc, Mutex};
use crate::handler::sync::ConnectionHandler;

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
                        let handler = ConnectionHandler::new(stream);
                        if let Err(e) = handler.handle() {
                            eprintln!("Error handling connecion: {}: {e}", peer_addr);
                        }
                        
                        // To run with TcpStream set to non-blocking mod
                        // if let Err(e) = handler.handle_non_blocking_raw() {
                        //     eprintln!("Error handling connecion (in non-blocking mode): {}: {e}", peer_addr);
                        // } else {
                        //     println!("Peer done (non-blocking mode)");
                        // }
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