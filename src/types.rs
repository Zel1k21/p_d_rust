use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::net::TcpListener;

pub struct Server {
    pub(crate) listener: TcpListener,
    pub(crate) db_connection_pool: Pool<SqliteConnectionManager>,
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
    RequestTooBig,

    Other(String),
}

#[derive(Debug, PartialEq)]
pub enum DatabaseError {
    UniqueConstraintError,
    NoUserError,
    NoTypeError,
    NoMediaError,
    StingLengthError,

    Default,
}

impl fmt::Display for DatabaseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SuperError is here!")
    }
}

impl Error for DatabaseError {}

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
    pub body: Option<Vec<u8>>,
}

#[derive(Debug, PartialEq)]
pub struct Response {
    pub response_code: ResponseCode,
    pub headers: HashMap<String, String>,
    pub body: Option<Vec<u8>>,
}

#[derive(Debug, PartialEq)]
pub struct MultipartFormEntry {
    pub headers: HashMap<String, String>,
    pub field_value: Vec<u8>,
}

#[derive(Debug, PartialEq)]
pub enum ContentType {
    Html,
    Css,
    Jpeg,
    Png,
}

#[derive(Debug, PartialEq)]
pub enum ResponseCode {
    OK,
    SeeOther,
    NotFound,
}
