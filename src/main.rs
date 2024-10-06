use p_d_rust::types::Server;
use std::thread;

const ADDRESS: &str = "localhost:3000";

fn main() {
    let handle = thread::spawn(|| {
        Server::new(ADDRESS).listen();
    });

    handle.join().unwrap();
}
