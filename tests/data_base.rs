#[cfg(test)]
mod test_db{
    use rusqlite::{Connection, Result};
    use std::collections::HashMap;
    use p_d_rust::data_base::{add_user, delete_user};

    #[derive(Debug)]
    struct User {
        name: String,
        password: String,
    }

    #[test]
    fn test_db() -> Result<()>{
        let accounts = Connection::open("accounts.db").expect("Error opening db");
        accounts.execute("
        create table if not exists user (
            id integer primary key,
            name text not null unique,
            password text not null
        )", [])?;

        let users = HashMap::new();
        add_user(&String::from("Jack"), &String::from("1 iq"), users.clone(), &accounts)?;
        delete_user(&String::from("Jack"), users.clone(), &accounts)?;
        let mut stmt = accounts.prepare("SELECT * from user")?;
        let users = stmt.query_map([],|row| {
            Ok(User {
                name: row.get(1)?,
                password: row.get(2)?,
            })
        })?;

        for user in users{
            println!("Found user {:?}", user);
        }
        Ok(())
    }
}