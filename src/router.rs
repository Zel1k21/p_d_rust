use crate::types::Request;
use std::collections::HashMap;
use std::fs;
use std::net::TcpStream;

use crate::database::add_user;
use crate::response::{ext_to_content_type_enum, send_file, send_response};
use crate::types::{ContentType, DatabaseError, Method, Response, ResponseCode};
use rusqlite::Connection;

fn handle_not_found(stream: &TcpStream) {
    let response = Response {
        response_code: ResponseCode::NotFound,
        headers: HashMap::new(),
        body: Some("Not found".as_bytes().to_vec()),
    };
    send_response(stream, response);
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

fn handle_register(stream: &TcpStream, request: &Request, db_conn: &Connection) {
    if request.method == Method::Post {
        if let Some(data) = request.parse_form() {
            match (|| -> Result<String, DatabaseError> {
                add_user(
                    data.get("username").ok_or(DatabaseError::Default)?,
                    data.get("password").ok_or(DatabaseError::Default)?,
                    db_conn,
                )
            })() {
                Err(_) => {
                    println!("username or password not found!");
                }
                Ok(pass_hash) => {
                    println!("registered successfully, passwprd hash is {:?}", pass_hash);
                }
            }
        }
    }
    send_file(stream, "./static/html/register.html", &ContentType::Html);
}

fn handle_success(stream: &TcpStream) {
    send_file(stream, "./static/html/success.html", &ContentType::Html);
}

fn handle_profile(stream: &TcpStream, request: &Request) {
    if request.method == Method::Post {
        let form_data_opt = request.parse_multipart_form();
        // temporary for testing
        if let Some(form_data) = form_data_opt {
            fs::write(
                "./image.png",
                &form_data.get("profilePic").unwrap().field_value,
            )
            .ok();
        }
        // TODO: filter file type, save to disk with unique name and add to DB
    }
    send_file(stream, "./static/html/profile.html", &ContentType::Html);
}

pub fn route(stream: &TcpStream, request: &Request, db_conn: &Connection) {
    match request.path.as_str() {
        path if path.to_string().starts_with("/static/")
            && !path.to_string().starts_with("/static/html/") =>
        {
            handle_static(stream, path)
        }
        "/" => handle_index(stream),
        "/register" => handle_register(stream, request, db_conn),
        "/success" => handle_success(stream),
        "/profile" => handle_profile(stream, request),
        _ => handle_not_found(stream),
    }
}
