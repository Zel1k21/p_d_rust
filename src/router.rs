use crate::types::Request;
use std::net::TcpStream;

use crate::response::{ext_to_content_type_enum, send_data, send_file};
use crate::types::ContentType;

fn handle_not_found(stream: &TcpStream) {
    send_data(stream, &ContentType::Html, b"URL not found");
}

fn handle_static(stream: &TcpStream, path: &str) {
    let file_ext = path.split(".").last().unwrap();
    match ext_to_content_type_enum(file_ext) {
        Ok(content_type) => send_file(stream, format!(".{}", path).as_str(), content_type),
        Err(_) => handle_not_found(stream),
    }
}

fn handle_index(stream: &TcpStream) {
    send_file(stream, "./static/html/index.html", &ContentType::Html);
}

pub fn route(stream: &TcpStream, request: &Request) {
    match request.path.as_str() {
        path if path.to_string().starts_with("/static/")
            && !path.to_string().starts_with("/static/html/") =>
        {
            handle_static(stream, path)
        }
        "/" => handle_index(stream),
        _ => handle_not_found(stream),
    }
}
