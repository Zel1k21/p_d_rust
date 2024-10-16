use crate::types::Request;
use std::collections::HashMap;
use std::net::TcpStream;

use crate::response::{ext_to_content_type_enum, send_file, send_response};
use crate::types::{ContentType, Response, ResponseCode};

fn handle_not_found(stream: &TcpStream) {
    let mut response = Response {
        response_code: ResponseCode::NotFound,
        headers: HashMap::new(),
        body: Some("Not found".as_bytes().to_vec()),
    };
    send_response(stream, &mut response);
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

fn handle_register(stream: &TcpStream) {
    send_file(stream, "./static/html/register.html", &ContentType::Html);
}

fn handle_success(stream: &TcpStream) {
    send_file(stream, "./static/html/success.html", &ContentType::Html);
}

pub fn route(stream: &TcpStream, request: &Request) {
    match request.path.as_str() {
        path if path.to_string().starts_with("/static/")
            && !path.to_string().starts_with("/static/html/") =>
        {
            handle_static(stream, path)
        }
        "/" => handle_index(stream),
        "/register" => handle_register(stream),
        "/success" => handle_success(stream),
        _ => handle_not_found(stream),
    }
}
