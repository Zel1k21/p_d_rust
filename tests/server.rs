use gall_rs::types::Server;
use std::fs;
use std::thread;

#[cfg(test)]
mod test_server {
    use super::*;
    const ADDRESS: &str = "localhost:3000";

    #[test]

    fn run_server() {
        let db_path = "test.db";

        let handle = thread::spawn(|| {
            Server::new(ADDRESS, db_path).listen_once();
        });

        reqwest::blocking::get(format!("http://{}/ok", ADDRESS)).unwrap();

        handle.join().unwrap();

        fs::remove_file(db_path).expect(&format!("Should be able to remove {}", db_path));
    }
}
