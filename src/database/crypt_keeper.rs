use crate::util::encryption::{FileCrypt, KEY_SIZE, NONCE_SIZE};
use anyhow::{anyhow, Result};
use lazy_static::lazy_static;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{params, Connection, Error as rusqliteError};
use std::{
    fs,
    path::{Path, PathBuf},
};

//Connection pool maintains a single connection to db for life of program
lazy_static! {
    static ref KEEPER: Pool<SqliteConnectionManager> = {
        let manager = SqliteConnectionManager::file("src/database/crypt_keeper.db");
        let pool = Pool::new(manager).expect("Failed to generate pool");

        init_keeper(&pool.get().unwrap()).expect("Failed to initialize keeper");

        pool
    };
}

///Generates a connection to the database.
///Creates the database if one does not exist.
fn init_keeper(conn: &Connection) -> anyhow::Result<()> {
    //Table for tracking the FileCrypt
    conn.execute(
        "CREATE TABLE IF NOT EXISTS crypt (
            uuid TEXT PRIMARY KEY,
            filename TEXT NOT NULL,
            extension TEXT NOT NULL,
            full_path TEXT NOT NULL,
            key_seed BLOB NOT NULL,
            nonce_seed BLOB NOT NULL,
            hash BLOB NOT NULL
        )",
        [],
    )?;

    return Ok(());
}

///Grabs the connection
fn get_keeper() -> anyhow::Result<r2d2::PooledConnection<r2d2_sqlite::SqliteConnectionManager>> {
    //Returns the static connection
    return KEEPER.get().map_err(|e| e.into());
}

///Insert a crypt into the database
pub fn insert_crypt(crypt: &FileCrypt) -> anyhow::Result<()> {
    //Get the connection
    let conn = get_keeper().map_err(|_| anyhow!("Failed to get keeper"))?;

    //Create insert command and execute -- should handle uuid conflicts
    conn.execute(
        "INSERT INTO crypt (
            uuid,
            filename,
            extension,
            full_path,
            key_seed,
            nonce_seed,
            hash
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
        ON CONFLICT(uuid) DO UPDATE SET
            filename = excluded.filename,
            extension = excluded.extension,
            full_path = excluded.full_path,
            key_seed = excluded.key_seed,
            nonce_seed = excluded.nonce_seed,
            hash = excluded.hash",

        params![
            &crypt.uuid,
            &crypt.filename,
            &crypt.ext,
            &crypt.full_path.to_str().unwrap(),
            &crypt.key.to_owned().as_ref(),
            &crypt.nonce.as_ref(),
            &crypt.hash.to_owned().as_ref(),
        ],
    )
    .map_err(|e| anyhow!("Failed to insert crypt {} into keeper", e))?;

    return Ok(());
}

///Queries the database for the crypt
pub fn query_crypt(uuid: String) -> Result<FileCrypt> {
    //Get the connection
    let conn = get_keeper()?;

    //Get the results of the query
    conn.query_row(
        "SELECT *
        FROM crypt
        WHERE uuid = ?1",
        params![uuid],
        |row| {
            let get: String = row.get(3)?;
            Ok(FileCrypt {
                uuid: row.get(0)?,
                filename: row.get(1)?,
                ext: row.get(2)?,
                full_path: PathBuf::from(get),
                key: row.get(4)?,
                nonce: row.get(5)?,
                hash: row.get(6)?,
            })
        },
    )
    .map_err(|e| match e {
        //Handle the errors
        rusqliteError::QueryReturnedNoRows => {
            anyhow!("No crypt with that uuid exists")
        }
        _ => anyhow!("Keeper query failed {}", e),
    })
}

///Queries the database for all crypts
pub fn query_keeper() -> anyhow::Result<Vec<FileCrypt>> {
    //Get the connection
    let conn = get_keeper().map_err(|_| anyhow!("Failed to get keeper"))?;

    //Create the query and execute
    let mut query = conn.prepare(
        "
        SELECT *
        FROM crypt",
    )?;

    //Get the results of the query
    let query_result = query.query_map([], |row| {
        let get: String = row.get(3)?;
        let key: [u8; KEY_SIZE] = row.get(4)?;
        let nonce: [u8; NONCE_SIZE] = row.get(5)?;
        let hash: [u8; KEY_SIZE] = row.get(6)?;

        Ok(FileCrypt {
            uuid: row.get(0)?,
            filename: row.get(1)?,
            ext: row.get(2)?,
            full_path: PathBuf::from(get),
            key,
            nonce,
            hash,
        })
    })?;

    //Convert the results into a vector
    let mut crypts: Vec<FileCrypt> = Vec::new();
    for crypt in query_result {
        crypts.push(crypt.unwrap());
    }

    return Ok(crypts);
}

///Deletes the crypt
pub fn delete_crypt(uuid: String) -> anyhow::Result<()> {
    //Get the connection
    let conn = get_keeper().map_err(|_| anyhow!("Failed to get keeper"))?;

    conn.execute(
        "
            DELETE FROM crypt WHERE uuid = ?
        ",
        params![uuid],
    )?;

    return Ok(());
}

///Delete the database
pub fn delete_keeper() -> anyhow::Result<()> {
    if Path::new("src/database/crypt_keeper.db").exists() {
        fs::remove_file("src/database/crypt_keeper.db")?;
    }
    return Ok(());
}
