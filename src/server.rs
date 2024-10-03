use std::net::{TcpListener, TcpStream};
use std::thread;

use crate::parse::parse;
use crate::response::send_file;
use crate::types::{ContentType, Server};

impl Server {
    pub fn new(on: &str) -> Server {
        Server {
            listener: TcpListener::bind(on).unwrap(),
        }
    }

    fn handle_connection(mut stream: TcpStream) {
        thread::spawn(move || match parse(&mut stream) {
            Ok(_) => {
                send_file(&stream, "./static/html/index.html", &ContentType::Html);
                // send_file(&stream, "./static/images/eevee.png", &ContentType::Png);
            }
            Err(err) => println!("Error: {:?}", err),
        });
    }

    pub fn listen_once(&mut self) {
        match self.listener.accept() {
            Ok((stream, _)) => Server::handle_connection(stream),
            Err(err) => println!("Error: {:?}", err),
        }
    }

    pub fn listen(&mut self) {
        for stream in self.listener.incoming() {
            match stream {
                Ok(stream) => Server::handle_connection(stream),
                Err(err) => println!("Error: {:?}", err),
            }
        }
    }
}
