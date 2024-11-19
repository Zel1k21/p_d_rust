#[cfg(test)]
mod test_db {
    use p_d_rust::{
        database::{add_user, check_authorization, delete_user, init_database},
        types::DatabaseError,
    };
    use rusqlite::{Connection, Result};
    use std::error::Error;
    use std::fs;
    use std::path::Path;

    #[derive(Debug)]
    #[allow(dead_code)]
    struct User {
        name: String,
        password: String,
    }

    fn get_users(connection: &Connection) -> Result<Vec<User>> {
        Ok(connection
            .prepare("SELECT * from user")?
            .query_map([], |row| {
                Ok(User {
                    name: row.get(1)?,
                    password: row.get(2)?,
                })
            })?
            .filter_map(|res| res.ok())
            .collect::<Vec<User>>())
    }

    #[test]
    fn test_db() -> Result<(), Box<dyn Error>> {
        let db_path = "database.db";
        if Path::new(db_path).exists() {
            Err("DB file exists")
        } else {
            Ok(())
        }?;
        init_database(db_path);
        let connection = Connection::open(db_path).expect("Error opening DB");
        let pass_hash = match add_user("Billy Bones", "treasure map", &connection) {
            Err(err) => Err(match err {
                DatabaseError::Default => "DefaultError",
                DatabaseError::UniqueConstraintError => "UniqueConstraintError",
            }),
            Ok(s) => Ok(s),
        }?;

        match check_authorization("Billy Bones", "treasure map", &connection) {
            Err(_) => Err("Could not authorize"),
            Ok(_) => Ok(()),
        }?;

        println!("Users before delete:");
        for user in get_users(&connection)? {
            println!("{:?}", user);
        }

        delete_user("Billy Bones", &connection)?;

        println!("\nUsers after delete:");
        for user in get_users(&connection)? {
            println!("{:?}", user);
        }

        match connection.close() {
            Err(_) => Err("Could not close DB connection"),
            Ok(_) => Ok(()),
        }?;

        fs::remove_file(db_path)?;

        Ok(())
    }
}
