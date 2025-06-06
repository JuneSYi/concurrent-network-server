use concurrent_network_server::server::Server;


fn main() {
    let server = Server::new("127.0.0.1:7878");
    if let Err(e) = server.run() {
        println!("Error while trying to run server");
        std::process::exit(1);
    }
}
