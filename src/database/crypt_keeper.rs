use crate::util::encryption::{FileCrypt, KEY_SIZE, NONCE_SIZE};
use rusqlite::Connection;


///Generates a connection to the database.
///Creates the database if one does not exist.
fn get_keeper() -> anyhow::Result<Connection> {
    //Creates/Opens database, change path if desired
    let conn = Connection::open("src/database/crypt_keeper.db")?;
    //Table for tracking the FileCrypt
    conn.execute(
        "CREATE TABLE IF NOT EXISTS crypt (
            uuid TEXT PRIMARY KEY,
            filename TEXT NOT NULL,
            extension TEXT NOT NULL,
            full_path TEXT NOT NULL,
            key_seed BLOB NOT NULL,
            nonce_seed BLOB NOT NULL
        )",
        [],
    )?;

    return Ok(conn);
}

///Insert a crypt into the database
pub fn insert(crypt: &FileCrypt) -> anyhow::Result<()> {
    //Get the connection
    let conn = get_keeper()?;

    //Create insert command and execute
    conn.execute(
        "INSERT INTO crypt (
            uuid,
            filename,
            extension,
            full_path,
            key_seed,
            nonce_seed
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)
        ON CONFLICT(uuid) DO UPDATE SET
            uuid = excluded.uuid,
            filename = excluded.filename,
            full_path = excluded.full_path,
            key_seed = excdlued.key_seed
            nonce_seed = excluded.nonce_seed",
        (
            &crypt.uuid,
            &crypt.filename,
            &crypt.ext,
            &crypt.full_path,
            &crypt.key.to_owned().as_ref(),
            &crypt.nonce.as_ref(),
        )
    )?;

    return Ok(());
}

///Queries the database for the crypt
pub fn query_crypt(uuid: String) -> anyhow::Result<Vec<FileCrypt>> {
    //Get the connection
    let conn = get_keeper()?;

    //Create the query and execute
    let mut query = conn.prepare("
        SELECT 
            uuid, 
            filename, 
            extension, 
            full_path, 
            key_seed, 
            nonce_seed
        FROM crypt WHERE uuid = ?1"
    )?;

    //Get the results of the query
    let query_result = query.query_map([uuid], |row| {
        let key: [u8; KEY_SIZE] = row.get(4)?;
        let nonce: [u8; NONCE_SIZE] = row.get(5)?;
        Ok(FileCrypt {
            uuid: row.get(0)?,
            filename: row.get(1)?,
            ext: row.get(2)?,
            full_path: row.get(3)?,
            key,
            nonce,
        })
    })?;

    //Convert the results into a vector
    //--find a method to return a single without vec
    let mut crypts: Vec<FileCrypt> = Vec::new();
    for crypt in query_result {
        crypts.push(crypt.unwrap());
    }
    
    return Ok(crypts);
}

///Queries the database for all crypts
pub fn query_keeper() -> anyhow::Result<Vec<FileCrypt>> {
    //Get the connection
    let conn = get_keeper()?;

    //Create the query and execute
    let mut query = conn.prepare("
        SELECT 
            uuid, 
            filename, 
            extension, 
            full_path, 
            key_seed, 
            nonce_seed
        FROM crypt"
    )?;

    //Get the results of the query
    let query_result = query.query_map([], |row| {
        let key: [u8; KEY_SIZE] = row.get(4)?;
        let nonce: [u8; NONCE_SIZE] = row.get(5)?;
        Ok(FileCrypt {
            uuid: row.get(0)?,
            filename: row.get(1)?,
            ext: row.get(2)?,
            full_path: row.get(3)?,
            key,
            nonce,
        })
    })?;

    //Convert the results into a vector
    let mut crypts: Vec<FileCrypt> = Vec::new();
    for crypt in query_result {
        crypts.push(crypt.unwrap());
    }
    
    return Ok(crypts);
}
