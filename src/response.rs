use std::fs;
use std::{io::Write, net::TcpStream};

use crate::types::ContentType;

pub fn send_data(mut stream: &TcpStream, content_type: &ContentType, data: &[u8]) {
    write_head(stream, content_type, &data.len());
    stream.write_all(data).ok();
}

pub fn send_file(stream: &TcpStream, path: &str, content_type: &ContentType) {
    let contents = fs::read(path).expect("Error reading file");
    send_data(stream, content_type, &contents);
}

fn write_head(mut stream: &TcpStream, content_type: &ContentType, content_length: &usize) {
    let head = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: {}\r\nContent-Length: {}\r\n\r\n",
        content_type_enum_to_str(content_type),
        content_length
    )
    .into_bytes();

    stream.write_all(&head).ok();
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

pub fn content_type_enum_to_str(content_type: &ContentType) -> &str {
    match content_type {
        ContentType::Html => "text/html",
        ContentType::Css => "text/css",
        ContentType::Jpeg => "image/jpeg",
        ContentType::Png => "image/png",
    }
}
