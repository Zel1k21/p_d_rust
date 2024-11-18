use std::collections::HashMap;
use std::{io::Read, net::TcpStream};

use bstr::ByteSlice;

use crate::types::{HttpParseError, HttpVersion, Method, MultipartFormEntry, Request};

pub fn parse(stream: &mut TcpStream) -> Result<Request, HttpParseError> {
    let mut data = Vec::<u8>::new();
    let mut buf = [0u8; 2_usize.pow(14)];

    match stream.read(&mut buf) {
        Err(err) => Err(HttpParseError::Other(format!("{}", err))),
        Ok(n) => {
            data.extend(buf.split_at(n).0);
            let head_body = split_vec_u8_once(&data, "\r\n\r\n".as_bytes()).unwrap_or((&data, &[]));

            if let Ok(content_length) = parse_headers(&String::from_utf8_lossy(head_body.0))
                .get("Content-Length")
                .unwrap_or(&"_".to_owned())
                .trim()
                .parse::<usize>()
            {
                if content_length > 2_usize.pow(26) {
                    return Err(HttpParseError::RequestTooBig);
                }
                if content_length > head_body.1.len() {
                    let rest_len: usize = content_length - head_body.1.len();
                    let mut received_len: usize = 0;
                    while received_len < rest_len {
                        match stream.read(&mut buf) {
                            Err(err) => Err(HttpParseError::Other(format!("{}", err))),
                            Ok(n) => {
                                data.extend(buf.split_at(n).0);
                                received_len += n;
                                Ok(())
                            }
                        }?
                    }
                }
            }
            Ok(())
        }
    }?;

    internal_parse(&data)
}

pub fn internal_parse(req: &[u8]) -> Result<Request, HttpParseError> {
    let head_body = split_vec_u8_once(req, "\r\n\r\n".as_bytes()).unwrap_or((req, &[]));
    let head = String::from_utf8_lossy(head_body.0).into_owned();
    let first_line_headers = head.split_once("\r\n").unwrap_or((&head, ""));
    let (method, path, http_version) = parse_first_line(first_line_headers.0)?;
    let headers = parse_headers(first_line_headers.1);
    let body = if !head_body.1.is_empty() {
        Some(head_body.1.to_vec())
    } else {
        None
    };

    Ok(Request {
        method,
        path,
        http_version,
        headers,
        body,
    })
}

fn parse_headers(string: &str) -> HashMap<String, String> {
    string
        .split("\r\n")
        .filter_map(|header| parse_header(header).ok())
        .map(|pair| (pair.0.to_string(), pair.1.to_string()))
        .collect()
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

pub fn parse_semicolon_list(string: &str) -> HashMap<String, Option<String>> {
    string
        .split(";")
        .map(|str| str.trim())
        .map(|str| {
            str.split_once("=")
                .map(|pair| (pair.0.to_string(), Some(remove_quotes(pair.1).to_string())))
                .unwrap_or((str.to_string(), None))
        })
        .collect()
}

fn remove_quotes(string: &str) -> &str {
    let len = string.len();
    if len >= 2 && &string[0..1] == "\"" && &string[len - 1..len] == "\"" {
        return &string[1..len - 1];
    }
    string
}

impl Request {
    pub fn parse_form(&self) -> Option<HashMap<String, String>> {
        self.body.as_ref().map(|bytes| {
            String::from_utf8_lossy(bytes)
                .split("&")
                .filter_map(|str| str.split_once("="))
                .map(|pair| (pair.0.to_string(), pair.1.to_string()))
                .collect()
        })
    }

    pub fn parse_multipart_form(&self) -> Option<HashMap<String, MultipartFormEntry>> {
        let content_type_value = parse_semicolon_list(self.headers.get("Content-Type")?);
        let boundary = format!("--{}", &content_type_value.get("boundary")?.clone()?).into_bytes();
        self.body.as_ref().map(|bytes| {
            bytes
                .split_str(&boundary)
                .filter_map(|byte_str| {
                    let headers_value = split_vec_u8_once(byte_str, "\r\n\r\n".as_bytes())?;
                    let mut headers = parse_headers(
                        String::from_utf8_lossy(headers_value.0)
                            .into_owned()
                            .as_str(),
                    );
                    let content_disposition =
                        parse_semicolon_list(headers.get("Content-Disposition")?);

                    let name = content_disposition.get("name")?.clone()?;
                    headers.remove("name");
                    Some((
                        name,
                        MultipartFormEntry {
                            headers,
                            field_value: headers_value.1.to_vec(),
                        },
                    ))
                })
                .collect()
        })
    }
}

fn split_vec_u8_once<'a>(vec: &'a [u8], splitter: &'a [u8]) -> Option<(&'a [u8], &'a [u8])> {
    let first = vec.split_str(splitter).next()?;
    let rest = vec.split_at_checked(first.len() + splitter.len())?.1;
    Some((first, rest))
}
