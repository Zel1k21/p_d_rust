use std::net::{TcpListener, TcpStream};
use std::thread;

use crate::database::init_database;
use crate::parse::parse;
use crate::router::route;
use crate::types::Server;
use r2d2::{Pool, PooledConnection};
use r2d2_sqlite::SqliteConnectionManager;

impl Server {
    pub fn new(on: &str, db_path: &str) -> Server {
        init_database(db_path);
        let manager = SqliteConnectionManager::file(db_path);
        let db_connection_pool = Pool::new(manager).unwrap();
        Server {
            listener: TcpListener::bind(on).unwrap(),
            db_connection_pool,
        }
    }

    fn handle_connection(
        mut stream: TcpStream,
        db_conn: PooledConnection<SqliteConnectionManager>,
    ) {
        thread::spawn(move || match parse(&mut stream) {
            Ok(request) => {
                route(&stream, &request, &db_conn);
            }
            Err(err) => println!("Error: {:?}", err),
        });
    }

    pub fn listen_once(&mut self) {
        match self.listener.accept() {
            Ok((stream, _)) => {
                Server::handle_connection(stream, self.db_connection_pool.get().unwrap())
            }
            Err(err) => println!("Error: {:?}", err),
        }
    }

    pub fn listen(&mut self) {
        for stream in self.listener.incoming() {
            match stream {
                Ok(stream) => {
                    Server::handle_connection(stream, self.db_connection_pool.get().unwrap())
                }
                Err(err) => println!("Error: {:?}", err),
            }
        }
    }
}
