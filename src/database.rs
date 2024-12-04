extern crate rusqlite;
use std::{error::Error, time::SystemTime};

use crate::types::{DatabaseError, User};
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
) -> Result<(), Box<dyn Error>> {
    validate_str_len(user_name, 4, 50)?;
    validate_str_len(password, 8, 50)?;
    validate_str_len(nickname, 4, 50)?;
    validate_str_len(description, 0, 500)?;
    let stmt = database.prepare("SELECT * from user WHERE name = ?1");

    stmt.unwrap().exists([user_name]).map(|ok| {
        if !ok {
            Ok(())
        } else {
            Err(Box::new(DatabaseError::UniqueConstraintError))
        }
    })??;

    let mut hasher = Sha256::new();
    hasher.update(password);
    let check_hash = format!("{:x}", hasher.finalize());

    database.execute(
        "INSERT INTO user (name, pass_hash, nickname, description) values (?1, ?2, ?3, ?4)",
        [user_name, check_hash.as_str(), nickname, description],
    )?;

    Ok(())
}

pub fn delete_user(
    user_id: usize,
    password: &str,
    database: &Connection,
) -> Result<(), Box<dyn Error>> {
    let mut hasher = Sha256::new();
    hasher.update(password);
    let pass_hash = format!("{:x}", hasher.finalize());

    let stmt = database.prepare("SELECT * from user WHERE id = ?1 AND pass_hash = ?2");

    stmt.unwrap()
        .exists([user_id.to_string(), pass_hash.to_string()])
        .map(|ok| {
            if ok {
                Ok(())
            } else {
                Err(Box::new(DatabaseError::NoUserError))
            }
        })??;

    database.execute("DELETE FROM user WHERE id = (?1)", [user_id])?;

    database.execute("DELETE FROM auth_token WHERE user_id = (?1)", [user_id])?;

    Ok(())
}

pub fn get_user(auth_token: &str, database: &Connection) -> Option<usize> {
    match database.query_row_and_then(
        "SELECT user_id FROM auth_token WHERE token = (?1)",
        [auth_token],
        |row| row.get(0),
    ) {
        Ok(user_id) => Some(user_id),
        Err(_) => None,
    }
}

pub fn get_user_info(username: &str, database: &Connection) -> Option<User> {
    match database.query_row_and_then(
        "SELECT name, nickname, description FROM user WHERE name = (?1)",
        [username],
        |row| {
            Ok::<User, rusqlite::Error>(User {
                username: row.get(0).expect("Should get username"),
                nickname: row.get(1).expect("Should get nickname"),
                description: row.get(2).expect("Should get description"),
            })
        },
    ) {
        Ok(user) => Some(user),
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

    match database.query_row_and_then(
        "SELECT id FROM user WHERE name = ?1 AND pass_hash = ?2",
        [username, &pass_hash],
        |row| row.get::<usize, usize>(0),
    ) {
        Ok(user_id) => match database.execute(
            "INSERT INTO auth_token (user_id, token) VALUES (?1, ?2)",
            [user_id.to_string(), token.to_owned()],
        ) {
            Ok(_) => Ok(token),
            Err(e) => {
                println!("{:?}", e);
                Err(DatabaseError::Default)
            }
        },
        Err(e) => {
            println!("{:?}", e);
            Err(DatabaseError::NoUserError)
        }
    }
    .map_err(|e| e.into())
}

pub fn update_user_info(
    user_id: usize,
    nickname: &str,
    description: &str,
    database: &Connection,
) -> Result<(), Box<dyn Error>> {
    let stmt = database.prepare("SELECT * from user WHERE id = ?1");

    validate_str_len(nickname, 4, 50)?;
    validate_str_len(description, 0, 500)?;

    stmt.unwrap().exists([user_id]).map(|ok| {
        if ok {
            Ok(())
        } else {
            Err(Box::new(DatabaseError::UniqueConstraintError))
        }
    })??;

    database.execute(
        "UPDATE user SET (nickname, description) VALUES (?1, ?2) WHERE id = ?3",
        [
            nickname.to_string(),
            description.to_string(),
            user_id.to_string(),
        ],
    )?;

    Ok(())
}

pub fn add_media(
    user_id: usize,
    media_type: &str,
    f_path: &str,
    title: &str,
    description: &str,
    database: &Connection,
) -> Result<(), Box<dyn Error>> {
    validate_str_len(title, 0, 50)?;
    validate_str_len(description, 0, 200)?;

    let user_existence_stmt = database.prepare("SELECT * from user WHERE id = ?1");

    user_existence_stmt
        .unwrap()
        .exists([user_id])
        .map_err(|_| Box::new(DatabaseError::NoUserError))?;

    let type_existence_stmt = database.prepare("SELECT * from mime_type WHERE name = ?1");
    type_existence_stmt
        .unwrap()
        .exists([media_type])
        .map_err(|_| Box::new(DatabaseError::NoTypeError))?;

    database.execute(
        "INSERT INTO media (user_id, f_path, title, description) VALUES (?1, ?2, ?3, ?4)",
        [
            user_id.to_string(),
            f_path.to_string(),
            title.to_string(),
            description.to_string(),
        ],
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

pub fn update_media_info(
    media_id: usize,
    user_id: usize,
    title: &str,
    description: &str,
    database: &Connection,
) -> Result<(), Box<dyn Error>> {
    validate_str_len(title, 0, 50)?;
    validate_str_len(description, 0, 200)?;

    let stmt = database.prepare("SELECT * from media WHERE id = ?1 and user_id = ?2");

    stmt.unwrap().exists([media_id, user_id]).map(|ok| {
        if ok {
            Ok(())
        } else {
            Err(Box::new(DatabaseError::UniqueConstraintError))
        }
    })??;

    database.execute(
        "UPDATE user SET (title, description) VALUES (?1, ?2) WHERE id = ?3",
        [
            title.to_string(),
            description.to_string(),
            media_id.to_string(),
        ],
    )?;

    Ok(())
}

pub fn delete_media(
    media_id: usize,
    user_id: usize,
    database: &Connection,
) -> Result<(), Box<dyn Error>> {
    let stmt = database.prepare("SELECT * from user WHERE id = ?1 AND user_id = ?2");

    stmt.unwrap().exists([media_id, user_id]).map(|ok| {
        if ok {
            Ok(())
        } else {
            Err(Box::new(DatabaseError::NoMediaError))
        }
    })??;

    database.execute("DELETE FROM media WHERE id = (?1)", [media_id])?;

    Ok(())
}

pub fn validate_str_len(string: &str, min: usize, max: usize) -> Result<(), DatabaseError> {
    if string.len() < min || string.len() > max {
        Err(DatabaseError::StingLengthError)
    } else {
        Ok(())
    }
}
