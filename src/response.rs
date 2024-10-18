use std::collections::HashMap;
use std::fs;
use std::{io::Write, net::TcpStream};

use crate::types::{ContentType, Response, ResponseCode};

pub fn send_response(mut stream: &TcpStream, mut response: Response) {
    match &mut response.body {
        Some(bytes) => response
            .headers
            .insert("Content-Length".to_string(), bytes.len().to_string()),
        None => None,
    };
    write_head(stream, &mut response);
    match &mut response.body {
        Some(bytes) => stream.write_all(bytes).ok(),
        None => None,
    };
}

pub fn send_file(
    stream: &TcpStream,
    path: &str,
    content_type: &ContentType,
    response: Option<Response>,
) {
    let contents = fs::read(path).expect("Error reading file");
    let mut resp = response.unwrap_or(Response {
        response_code: ResponseCode::OK,
        headers: HashMap::new(),
        body: Some(contents),
    });
    resp.headers.insert(
        "Content-Type".to_string(),
        content_type_enum_to_str(content_type).to_string(),
    );
    send_response(stream, resp);
}

fn write_head(mut stream: &TcpStream, response: &mut Response) {
    let head = format!(
        "HTTP/1.1 {}\r\n",
        response_code_enum_to_str(&response.response_code)
    );
    stream.write_all(head.as_bytes()).ok();
    for header in &response.headers {
        write_header(stream, header);
    }
    stream.write_all("\r\n".as_bytes()).ok();
}

fn write_header(mut stream: &TcpStream, header: (&String, &String)) {
    let header_str = format!("{}: {}\r\n", header.0, header.1);
    stream.write_all(header_str.as_bytes()).ok();
}

pub fn ext_to_content_type_enum(file_ext: &str) -> Result<&ContentType, &'static str> {
    match file_ext {
        "html" => Ok(&ContentType::Html),
        "css" => Ok(&ContentType::Css),
        "jpeg" | "jpg" => Ok(&ContentType::Jpeg),
        "png" => Ok(&ContentType::Png),
        _ => Err("Unknown file type"),
    }
}

fn content_type_enum_to_str(content_type: &ContentType) -> &str {
    match content_type {
        ContentType::Html => "text/html",
        ContentType::Css => "text/css",
        ContentType::Jpeg => "image/jpeg",
        ContentType::Png => "image/png",
    }
}

fn response_code_enum_to_str(response_code: &ResponseCode) -> &str {
    match response_code {
        ResponseCode::OK => "200 OK",
        ResponseCode::NotFound => "404 Not Found",
    }
}
