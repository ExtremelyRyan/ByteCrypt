use crate::util::{
    config, config::Config,
    encryption::{FileCrypt, KEY_SIZE, NONCE_SIZE}, self,
};
use anyhow::{anyhow, Result, Ok};
use lazy_static::lazy_static;
use log::info;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{params, Connection, Error as rusqliteError};
use std::{
    fs,
    path::{Path, PathBuf},
};
use csv::*; 

//Connection pool maintains a single connection to db for life of program
lazy_static! {
    static ref KEEPER: Pool<SqliteConnectionManager> = {
        let config = config::load_config().unwrap();
        let manager = SqliteConnectionManager::file(config.get_database_path());
        let pool = Pool::new(manager).expect("Failed to generate pool");

        init_keeper(&pool.get().unwrap()).expect("Failed to initialize keeper");

        pool
    };
}

///Generates a connection to the database.
///Creates the database if one does not exist.
fn init_keeper(conn: &Connection) -> Result<()> {
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

    Ok(())
}

///Exports ALL content within the `crypt_keeper` database to a csv for easy sharing. 
/// Exports `crypt_export.csv` to crypt folder
pub fn export_keeper(config: Config) -> Result<()> {
    
    // https://rust-lang-nursery.github.io/rust-cookbook/encoding/csv.html
    let db_crypts = query_keeper().unwrap();
    let mut wtr = WriterBuilder::new().has_headers(false).from_writer(vec![]);
    for crypt in db_crypts { 
        wtr.serialize(crypt)?;
    }
    let data = String::from_utf8(wtr.into_inner()?)?;

    // get crypt dir "C:\\users\\USER\\crypt"
    let mut path = util::common::get_backup_folder();
    path.push("crypt_export.csv"); 

    info!("writing export to {}",&path);
    util::common::write_contents_to_file(path.to_str().unwrap(), data.into_bytes()) 
}

/// Imports csv into database. <b>WARNING</b>, overrides may occur!
pub fn import_keeper(config: Config) -> Result<()> { 

    // temp solution until we get this on config.
    let mut path = util::common::get_backup_folder();
    path.push("crypt_export.csv"); 

    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_path(path)?;
    
    for result in rdr.records() {
        let record: StringRecord = match result {
            Ok(it) => it,
            Err(err) => (), // TODO: Fix with more elegant handling.
        }; 
        insert_crypt(match record.deserialize(None) {
            Ok(it) => it,
            Err(err) => (), // TODO: Fix with more elegant handling.
        });
    }

    Ok(())
}

///Grabs the connection
pub fn get_keeper() -> Result<r2d2::PooledConnection<r2d2_sqlite::SqliteConnectionManager>> {
    //Returns the static connection
    KEEPER.get().map_err(|e| e.into())
}

///Insert a crypt into the database
pub fn insert_crypt(crypt: &FileCrypt) -> Result<()> {
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
            &crypt.key.as_ref(),
            &crypt.nonce.as_ref(),
            &crypt.hash.as_ref(),
        ],
    )
    .map_err(|e| anyhow!("Failed to insert crypt {} into keeper", e))?;

    Ok(())
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

///Queries the database if a file's metadata matches existing entry in crypt keeper
pub fn query_keeper_for_existing_file(full_path: PathBuf) -> Result<FileCrypt> {
    //Get the connection
    let conn = get_keeper()?;

    //Get the results of the query
    conn.query_row(
        "SELECT *
        FROM crypt
        WHERE full_path = ?1",
        params![full_path.to_str().unwrap().to_string()],
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
pub fn query_keeper() -> Result<Vec<FileCrypt>> {
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

    Ok(crypts)
}
 

///Deletes the crypt
pub fn delete_crypt(uuid: String) -> Result<()> {
    //Get the connection
    let conn = get_keeper().map_err(|_| anyhow!("Failed to get keeper"))?;

    conn.execute(
        "
            DELETE FROM crypt WHERE uuid = ?
        ",
        params![uuid],
    )?;

    Ok(())
}

///Delete the database
pub fn delete_keeper() -> Result<()> {
    // TODO remove hardcoded pathways for this
    if Path::new("src/database/crypt_keeper.db").exists() {
        fs::remove_file("src/database/crypt_keeper.db")?;
    }
    Ok(())
}
