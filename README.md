# concurrent-network-server

A step-by-step learning project that re-implements [Eli Bendersky’s C server series](https://eli.thegreenplace.net/2017/concurrent-servers-part-1-introduction/) in idiomatic Rust, progressing from blocking single-threaded code to async/await.

## Server Overview

A client connection follows this protocol:
- Server sends a `*` byte handshake.
- Client frames each request between `^` and `$`.
- If payload starts with `P`, the server parses digits, offloads a prime check, and replies with `prime\n` or `composite\n`.
- Otherwise it adds `+1` to each byte and echoes back the transformed data.
- The server then waits for the next request, allowing multiple in one session.

## Running the Server

Build and start:
```bash
cargo run
```
By default it listens on `127.0.0.1:7878`.

## Testing with simple_test_client.py

In another terminal, run:
```bash
python3 simple_test_client.py
```
This script sends three requests in the following order:
1. **LongPrime**: large prime number that the server inefficiently (on purpose) calculates to simulate a long-running task in a non-blocking thread.
2. **Echo**: a short string `echo`.
3. **SmallPrime**: small prime number that is calculated quickly.
The output order demonstrates non-blocking behavior: echo and small-prime finish before the long-prime result.

## Project Structure

- `src/main.rs`: The main entry point for the server application.
- `src/lib.rs`: Defines the public modules of the library.
- `src/server/`: Contains different server implementations (e.g., `sync.rs` for synchronous, `async.rs` for asynchronous).
- `src/handler/`: Holds logic for handling client requests, separated by concurrency model (e.g., `sync.rs`, `async.rs`).
- `src/protocol/`: Defines the communication protocol and message parsing logic.
- `simple_test_client.py`: A Python client for testing the server.

## Learning Objectives
- Familiarize with TCP networking concepts
- Compare concurrency patterns: thread-per-connection ➜ thread-pool ➜ non-blocking event loop ➜ async/await
- Deepen understanding of ownership, borrowing, and error-handling
- Explore modular design and separation of concerns

## Implementation Roadmap

1. Sequential server
2. Stateful protocol (single thread)
3. Thread-per-client
4. Thread-pool (mpsc)
5. Non-blocking `TcpStream`
6. Refactor for Separation of concerns (Single Responsibility Principle)
7. Async handler module
8. Async server (Tokio)
9. Off-loading with `spawn_blocking`
