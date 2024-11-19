extern crate rusqlite;
use crate::types::DatabaseError;
use rusqlite::{Connection, Result};
use sha2::{Digest, Sha256};

pub fn init_database(path: &str) {
    let accounts = Connection::open(path).expect("Error opening db");

    accounts
        .execute(
            "
    create table if not exists user (
        id integer primary key,
        name text not null unique,
        pass_hash text not null
    )",
            [],
        )
        .ok();
}

pub fn check_authorization(username: &str, password: &str, database: &Connection) -> Result<bool> {
    let mut hasher = Sha256::new();
    hasher.update(password);

    let check_hash = String::from_utf8_lossy(&hasher.finalize()).into_owned();

    let mut stmt = database.prepare("SELECT * FROM user WHERE name = ?1 AND pass_hash = ?2")?;
    let user_exist = stmt.exists(rusqlite::params![username, check_hash])?;
    if !user_exist {
        Err(rusqlite::Error::InvalidQuery)?;
    }
    Ok(true)
}

pub fn add_user(
    user_name: &str,
    password: &str,
    database: &Connection,
) -> Result<String, DatabaseError> {
    let stmt = database.prepare("SELECT * from user WHERE name = ?1");

    if match stmt.unwrap().exists([user_name]) {
        Err(_) => Err(DatabaseError::Default),
        Ok(user_exists) => {
            if user_exists {
                Err(DatabaseError::Default)
            } else {
                Ok(())
            }
        }
    }
    .is_err()
    {
        return Err(DatabaseError::UniqueConstraintError);
    }
    let mut hasher = Sha256::new();
    hasher.update(password);
    let pass_hash = String::from_utf8_lossy(&hasher.finalize()).into_owned();

    if database
        .execute(
            "INSERT INTO user (name, pass_hash) values (?1, ?2)",
            [user_name, pass_hash.as_str()],
        )
        .is_err()
    {
        return Err(DatabaseError::UniqueConstraintError);
    }
    Ok(pass_hash)
}

pub fn delete_user(user_name: &str, database: &Connection) -> Result<()> {
    database.execute("DELETE FROM user WHERE name = (?1)", [user_name])?;

    Ok(())
}
