use crate::types::Request;
use std::collections::HashMap;
use std::fs;
use std::net::TcpStream;

use crate::database::add_user;
use crate::response::{ext_to_content_type_enum, file_resp, send_response};
use crate::types::{ContentType, DatabaseError, Method, Response, ResponseCode};
use rusqlite::Connection;

fn handle_not_found() -> Response {
    Response {
        response_code: ResponseCode::NotFound,
        headers: HashMap::new(),
        body: Some("Not found".as_bytes().to_vec()),
    }
}

fn handle_static(path: &str) -> Response {
    let file_ext = path.split(".").last().unwrap();
    match ext_to_content_type_enum(file_ext) {
        Ok(content_type) => file_resp(format!(".{}", path).as_str(), content_type),
        Err(_) => handle_not_found(),
    }
}

fn handle_index() -> Response {
    file_resp("./static/html/index.html", &ContentType::Html)
}

fn handle_register(request: &Request, db_conn: &Connection) -> Response {
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
    file_resp("./static/html/register.html", &ContentType::Html)
}

fn handle_success() -> Response {
    file_resp("./static/html/success.html", &ContentType::Html)
}

fn handle_profile(request: &Request) -> Response {
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
    file_resp("./static/html/profile.html", &ContentType::Html)
}

pub fn route(stream: &TcpStream, request: &Request, db_conn: &Connection) {
    let response = match request.path.as_str() {
        path if path.to_string().starts_with("/static/")
            && !path.to_string().starts_with("/static/html/") =>
        {
            handle_static(path)
        }
        "/" => handle_index(),
        "/register" => handle_register(request, db_conn),
        "/success" => handle_success(),
        "/profile" => handle_profile(request),
        _ => handle_not_found(),
    };
    send_response(stream, response);
}
