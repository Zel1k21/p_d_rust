use std::collections::HashMap;
use std::{io::Read, net::TcpStream};

use crate::types::{HttpParseError, HttpVersion, Method, Request};

pub fn parse(stream: &mut TcpStream) -> Result<Request, HttpParseError> {
    let mut buf = [0u8; 4096];

    match stream.read(&mut buf) {
        Err(err) => Err(HttpParseError::Other(format!("{}", err))),
        Ok(n) => Ok(internal_parse(
            String::from_utf8_lossy(buf.split_at(n).0).into_owned(),
        )?),
    }
}

pub fn internal_parse(req: String) -> Result<Request, HttpParseError> {
    let mut head_body = req.split("\r\n\r\n");
    let mut head_lines = head_body.next().unwrap_or_default().split("\r\n");
    let first_line = head_lines.next().unwrap_or_default();

    let (method, path, http_version) = parse_first_line(first_line)?;

    let mut headers = HashMap::new();
    for header in head_lines {
        let (key, value) = parse_header(header)?;
        headers.insert(key.to_string(), value.to_string());
    }

    let body = match head_body.next() {
        Some(str) => {
            if !str.is_empty() {
                Some(str.as_bytes().to_vec())
            } else {
                None
            }
        }

        None => None,
    };

    Ok(Request {
        method,
        path,
        http_version,
        headers,
        body,
    })
}

fn parse_first_line(first_line: &str) -> Result<(Method, String, HttpVersion), HttpParseError> {
    let mut strings = first_line.split(' ');

    let method = get_method(strings.next())?;
    let path = get_path(strings.next())?;
    let http_version = get_http_version(strings.next())?;

    Ok((method, path, http_version))
}

fn parse_header(header: &str) -> Result<(&str, &str), HttpParseError> {
    let mut key_value = header.split(":");
    let key = match key_value.next() {
        Some(str) => Ok(str.trim()),
        None => Err(HttpParseError::InvalidHeader),
    }?;
    let value = match key_value.next() {
        Some(str) => Ok(str.trim()),
        None => Err(HttpParseError::InvalidHeader),
    }?;
    Ok((key, value))
}

fn get_http_version(version: Option<&str>) -> Result<HttpVersion, HttpParseError> {
    match version {
        Some("HTTP/1.1") => Ok(HttpVersion::Http1_1),
        Some("HTTP/2.0") => Ok(HttpVersion::Http2_0),
        _ => Err(HttpParseError::InvalidHttpVersion),
    }
}

pub fn get_path(req: Option<&str>) -> Result<String, HttpParseError> {
    let string = req.ok_or(HttpParseError::InvalidPath)?.to_owned();

    if string.is_empty() {
        Err(HttpParseError::InvalidPath)
    } else {
        Ok(string)
    }
}

pub fn get_method(method: Option<&str>) -> Result<Method, HttpParseError> {
    Ok(match method {
        Some("GET") => Method::Get,
        Some("DELETE") => Method::Delete,
        Some("HEAD") => Method::Head,
        Some("OPTIONS") => Method::Options,
        Some("PATCH") => Method::Patch,
        Some("POST") => Method::Post,
        Some("PUT") => Method::Put,
        _ => return Err(HttpParseError::InvalidMethod),
    })
}
