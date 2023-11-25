// use chacha20poly1305::aead::Result;
use rusqlite::{Connection, Result, Error};
use crate::util::encryption::{FileCrypt, KEY_SIZE, NONCE_SIZE};


///Generates a connection to the database.
///Creates the database if one does not exist.
fn enable_keeper() -> Result<Connection> {
    let conn = Connection::open("crypt_keeper.db")?;

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
pub fn insert(crypt: &FileCrypt) -> Result<()> {
    let conn = enable_keeper()?;

    conn.execute(
        "INSERT INTO crypt (
            uuid,
            filename,
            extension,
            full_path,
            key_seed,
            nonce_seed
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
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
pub fn query(uuid: String) -> Result<FileCrypt> {
    let conn = enable_keeper()?;
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

    let mut query_result = query.query_map([uuid], |row| {
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

    return query_result.next().unwrap();
}
