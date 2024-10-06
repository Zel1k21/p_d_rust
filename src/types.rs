use std::collections::HashMap;
use std::net::TcpListener;

pub struct Server {
    pub(crate) listener: TcpListener,
}

#[derive(Debug, PartialEq)]
pub enum Method {
    Get,
    Post,
    Put,
    Head,
    Delete,
    Options,
    Patch,
}

#[derive(Debug, PartialEq)]
pub enum HttpParseError {
    InvalidMethod,
    InvalidPath,
    InvalidHttpVersion,
    InvalidHeader,

    Other(String),
}

#[derive(Debug, PartialEq)]
pub enum HttpVersion {
    Http1_1,
    Http2_0,
}

#[derive(Debug, PartialEq)]
pub struct Request {
    pub method: Method,
    pub path: String,
    pub http_version: HttpVersion,
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
}

#[derive(Debug, PartialEq)]
pub enum ContentType {
    Html,
    Css,
    Jpeg,
    Png,
}
