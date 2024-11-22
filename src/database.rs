extern crate rusqlite;
use std::{error::Error, time::SystemTime};

use crate::types::DatabaseError;
use rusqlite::{Connection, Result};
use sha2::{Digest, Sha256};

pub fn init_database(path: &str) {
    let database = Connection::open(path).expect("Error opening db");

    database
        .execute(
            "
    create table if not exists user (
        id integer primary key,
        name text not null unique,
        pass_hash varchar, 
        nickname varchar,
        description text
    )",
            [],
        )
        .ok();

    database
        .execute(
            "create table if not exists media (
    id integer primary key,
    user_id integer not null,
    type_id integer not null,
    f_path varchar,
    title varchar,
    description text
    )",
            [],
        )
        .ok();

    database
        .execute(
            "create table if not exists mime_type (
    id integer primary key, 
    name varchar)",
            [],
        )
        .ok();

    database
        .execute(
            "create table if not exists auth_token (
        id integer primary key,
        user_id integer not null,
        token varchar unique
    )",
            [],
        )
        .ok();

    database
        .execute(
            "INSERT INTO mime_type (name) VALUES
    (?1), (?2), (?3), (?4), (?5), (?6), (?7), (?8), (?9) ",
            [
                "image/apng",
                "image/avif",
                "image/bmp",
                "image/gif",
                "image/jpeg",
                "image/png",
                "image/svg+xml",
                "image/tiff",
                "image/webp",
            ],
        )
        .ok();
}

pub fn user_authorized(username: &str, password: &str, database: &Connection) -> bool {
    let mut hasher = Sha256::new();
    hasher.update(password);
    let check_hash = format!("{:x}", hasher.finalize());

    if let Ok(mut stmt) = database.prepare("SELECT * FROM user WHERE name = ?1 AND pass_hash = ?2")
    {
        stmt.exists(rusqlite::params![username, check_hash])
            .unwrap_or(false)
    } else {
        false
    }
}

pub fn add_user(
    user_name: &str,
    password: &str,
    nickname: &str,
    description: &str,
    database: &Connection,
) -> Result<String, Box<dyn Error>> {
    let stmt = database.prepare("SELECT * from user WHERE name = ?1");

    stmt.unwrap()
        .exists([user_name])
        .map_err(|_| Box::new(DatabaseError::UniqueConstraintError))?;

    let mut hasher = Sha256::new();
    hasher.update(password);
    let check_hash = format!("{:x}", hasher.finalize());

    database.execute(
        "INSERT INTO user (name, pass_hash, nickname, description) values (?1, ?2, ?3, ?4)",
        [user_name, check_hash.as_str(), nickname, description],
    )?;

    Ok(check_hash)
}

pub fn delete_user(user_name: &str, database: &Connection) -> Result<()> {
    database.execute("DELETE FROM user WHERE name = (?1)", [user_name])?;

    Ok(())
}

pub fn get_user(auth_token: &str, database: &Connection) -> Option<usize> {
    match database.execute(
        "SELECT user_id FROM auth_token WHERE name = (?1)",
        [auth_token],
    ) {
        Ok(user_id) => Some(user_id),
        Err(_) => None,
    }
}

pub fn do_login(
    username: &str,
    password: &str,
    database: &Connection,
) -> Result<String, Box<dyn Error>> {
    let mut hasher = Sha256::new();
    hasher.update(password);
    let pass_hash = format!("{:x}", hasher.finalize());

    let now = SystemTime::now().elapsed().unwrap().as_nanos();
    let token = format!("{:x}", Sha256::digest(now.to_string()));

    match database.execute(
        "SELECT user_id FROM user WHERE name = ?1 AND pass_hash = ?2",
        [username, &pass_hash],
    ) {
        Ok(user_id) => match database.execute(
            "INSERT INTO auth_token (token) 
            WHERE user_id = ?1 VALUES (?2)",
            [user_id.to_string(), token.to_owned()],
        ) {
            Ok(_) => Ok(token),
            Err(_) => Err(DatabaseError::Default),
        },
        Err(_) => Err(DatabaseError::NoUserError),
    }
    .map_err(|e| e.into())
}

pub fn add_media(
    user_id: usize,
    media_type: &str,
    f_path: &str,
    database: &Connection,
) -> Result<(), Box<dyn Error>> {
    let user_existence_stmt = database.prepare("SELECT * from user WHERE id = ?1");
    match user_existence_stmt.unwrap().exists([user_id]) {
        Err(_) => Err(DatabaseError::Default),
        Ok(user_exists) => {
            if user_exists {
                Ok(())
            } else {
                Err(DatabaseError::NoUserError)
            }
        }
    }
    .map_err(Box::new)?;

    let type_existence_stmt = database.prepare("SELECT * from mime_type WHERE name = ?1");
    match type_existence_stmt?.exists([media_type]) {
        Err(_) => Err(DatabaseError::Default),
        Ok(type_exists) => {
            if type_exists {
                Ok(())
            } else {
                Err(DatabaseError::NoTypeError)
            }
        }
    }
    .map_err(Box::new)?;

    database.execute(
        "INSERT INTO media (user_id, f_path) VALUES (?1, ?2)",
        [user_id.to_string(), f_path.to_string()],
    )?;
    database.execute(
        "INSERT INTO media (type_id) 
    SELECT u.*
    FROM mime_type mt
    JOIN media m ON mt.id = m.type_id
    JOIN user u ON m.user_id = u.id
    WHERE mt.name = (?1)",
        [media_type],
    )?;

    Ok(())
}
