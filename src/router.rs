use crate::types::Request;
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::fs;
use std::net::TcpStream;
use tera::{Context, Tera};

use crate::database::{add_user, do_login, get_user};
use crate::response::{
    ext_to_content_type_enum, file_resp, redirect_resp, send_response, string_resp,
};
use crate::types::{ContentType, DatabaseError, Method, Response, ResponseCode};
use rusqlite::Connection;

lazy_static! {
    pub static ref TEMPLATES: Tera = {
        match Tera::new("templates/*.html") {
            Ok(t) => t,
            Err(e) => {
                println!("Parsing error(s): {}", e);
                ::std::process::exit(1);
            }
        }
    };
}

fn get_request_user_id(request: &Request, db_conn: &Connection) -> Option<usize> {
    request
        .read_cookie("auth_token")
        .map(|token| get_user(token, db_conn))
        .unwrap_or(None)
}

fn handle_login_form(request: &Request, db_conn: &Connection) -> Response {
    if let Some(data) = request.parse_form() {
        match do_login(
            data.get("username").expect("Should get username"),
            data.get("password").expect("Should get password"),
            db_conn,
        ) {
            Err(_) => redirect_resp(&request.path),
            Ok(token) => {
                let mut resp = redirect_resp("/");
                resp.write_cookie("auth_token", &token, 30 * 24 * 60 * 60);
                resp
            }
        }
    } else {
        redirect_resp(&request.path)
    }
}

fn handle_register_form(request: &Request, db_conn: &Connection) -> Response {
    if let Some(data) = request.parse_form() {
        match (|| {
            add_user(
                data.get("username").ok_or(DatabaseError::Default)?,
                data.get("password").ok_or(DatabaseError::Default)?,
                "",
                "",
                db_conn,
            )
        })() {
            Err(_) => redirect_resp(&request.path),
            Ok(_) => handle_login_form(request, db_conn),
        }
    } else {
        redirect_resp(&request.path)
    }
}

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
    let rendered = TEMPLATES
        .render("index.html", &Context::new())
        .expect("Should render");
    string_resp(&rendered)
}

fn handle_register(request: &Request, db_conn: &Connection) -> Response {
    if request.method == Method::Post {
        return handle_register_form(request, db_conn);
    }
    file_resp("./static/html/register.html", &ContentType::Html)
}

fn handle_success() -> Response {
    file_resp("./static/html/success.html", &ContentType::Html)
}

fn handle_profile(request: &Request, db_conn: &Connection) -> Response {
    if request.method == Method::Post {
        if get_request_user_id(request, db_conn).is_none() {
            return redirect_resp("/register");
        }
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
    let clean_path = "/".to_owned()
        + &request
            .path
            .split("/")
            .filter(|s| !s.is_empty())
            .map(|s| s.to_owned())
            .collect::<Vec<String>>()
            .join("/");

    if clean_path != request.path {
        send_response(stream, redirect_resp(&clean_path));
        return;
    }

    let response = match request.path.as_str() {
        path if path.starts_with("/static/") => handle_static(path),
        "/" => handle_index(),
        "/register" => handle_register(request, db_conn),
        "/success" => handle_success(),
        "/profile" => handle_profile(request, db_conn),
        _ => handle_not_found(),
    };
    send_response(stream, response);
}
