use concurrent_network_server::server::sync;


fn main() {
    let server = sync::Server::new("127.0.0.1:7878");
    if let Err(e) = server.run() {
        println!("Error while trying to run server");
        std::process::exit(1);
    }
}
