use rusqlite::{Connection, Result as SQResult, Error};
use anyhow::{Ok, Result};
use crate::util::encryption::FileCrypt;


// pub struct FileCrypt {
//     pub uuid: String,
//     pub filename: String,
//     pub ext: String,
//     pub full_path: String,
//     pub key: [u8; KEY_SIZE],
//     pub nonce: [u8; NONCE_SIZE],
// }


fn enable_keeper() -> anyhow::Result<Connection> {
    let conn = Connection::open("crypt_keeper.db")?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS crypt (
            uuid TEXT PRIMARY KEY,
            filename TEXT NOT NULL,
            extension TEXT NOT NULL,
            full_path TEXT NOT NULL,
            key_seed BLOB NOT NULL,
            nonce_seed BLOB NOT NULL,
        )",
        [],
    )?;

    return Ok(conn);
}

fn insert(crypt: FileCrypt) -> anyhow::Result<()> {
    let conn = enable_keeper()?;

    conn.execute(
        "INSERT INTO crypt (
            uuid,
            filename,
            extension,
            full_path,
            key_seed,
            nonce_seed,
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
