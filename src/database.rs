extern crate rusqlite;
use rusqlite::{Connection, Result};
use std::collections::HashMap;
            
pub fn add_user(user_name: &String, password: &String, mut hash: HashMap<String, String>, data_base: &Connection) -> Result<()>{
    let stmt = data_base.prepare("SELECT * from user WHERE name = ?1");
    let user_exists = stmt?.exists([user_name])?;
    if user_exists {
        return Err(rusqlite::Error::InvalidQuery);
    }

    hash.insert(user_name.to_string(), password.to_string());
    data_base.execute(
        "INSERT INTO user (name, password) values (?1, ?2)",
        [&user_name.to_string(), &password.to_string()],
    )?;

    Ok(())
}

pub fn delete_user(user_name: &String, mut hash: HashMap<String, String>, data_base: &Connection) -> Result<()>{
    hash.remove(user_name);
    data_base.execute("DELETE FROM user WHERE name = (?1)", [&user_name.to_string()])?;
    Ok(())
}