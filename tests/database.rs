#[cfg(test)]
mod test_db {
    use p_d_rust::database::{add_user, delete_user, init_database, user_authorized};
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
        let db_path = "test.db";
        if Path::new(db_path).exists() {
            Err("DB file exists")
        } else {
            Ok(())
        }?;
        init_database(db_path);
        let connection = Connection::open(db_path).expect("Error opening DB");
        add_user(
            "Billy Bones",
            "treasure map",
            "tokyo ghoul",
            "i am ghoul, let me die",
            &connection,
        )?;

        if user_authorized("Billy Bones", "treasure map", &connection) {
            Ok(())
        } else {
            Err("Could not authorize")
        }?;

        println!("Users before delete:");
        for user in get_users(&connection)? {
            println!("{:?}", user);
        }

        delete_user(1, "treasure map", &connection)?;

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
