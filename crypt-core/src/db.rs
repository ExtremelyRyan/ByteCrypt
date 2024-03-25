use crate::{
    common::{get_config_folder, write_contents_to_file},
    config::get_config,
    encryption::{KEY_SIZE, NONCE_SIZE},
    filecrypt::FileCrypt,
    prelude::*,
    token::{CloudService, UserToken},
};
use csv::{StringRecord, WriterBuilder};
use lazy_static::lazy_static;
use logfather::*;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{params, Connection};
use std::{
    fs,
    path::{Path, PathBuf},
    str::FromStr,
};

//Connection pool maintains a single connection to db for life of program
//TODO: increase pool size from 1 to allow for multithreading
lazy_static! {
    static ref KEEPER: Pool<SqliteConnectionManager> = {
        info!("Initializing database");
        let path;
        {//Ensure to only borrow config and release asap
            let config = get_config();
            path = config.database_path.to_string();
        }
        let manager = SqliteConnectionManager::file(path);
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
            drive_id TEXT NOT NULL,
            full_path TEXT NOT NULL,
            key_seed BLOB NOT NULL,
            nonce_seed BLOB NOT NULL,
            hash BLOB NOT NULL
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS user_token (
            service TEXT PRIMARY KEY,
            key_seed BLOB NOT NULL,
            nonce_seed BLOB NOT NULL,
            expiration INTEGER NOT NULL
        )",
        [],
    )?;

    return Ok(());
}

/// Export data from the keeper and write it to a CSV file.
///
/// # Arguments
///
/// * `alt_path`: An optional alternative path where the CSV file should be saved.
///
/// # Returns
///
/// Returns a `Result` or `Error` indicating success or failure.
pub fn export_keeper(alt_path: Option<&str>) -> Result<()> {
    // https://rust-lang-nursery.github.io/rust-cookbook/encoding/csv.html

    // Query keeper crypts
    let db_crypts = query_keeper_crypt()?;

    // Create CSV writer
    let mut wtr = WriterBuilder::new().has_headers(false).from_writer(vec![]);

    // Serialize crypts to CSV
    for crypt in db_crypts {
        wtr.serialize(crypt)?;
    }

    // Get CSV data as bytes
    let data = wtr.into_inner().map_err(|e| e.into_error())?;

    // get crypt dir "C:\\users\\USER\\crypt_config"
    let path: PathBuf = match alt_path {
        Some(p) => PathBuf::from_str(p)?,
        None => {
            let mut p = get_config_folder();
            p.push("crypt_export.csv");
            p
        }
    };

    info!("writing export to {}", &path.display());

    if let Some(ap) = alt_path {
        write_contents_to_file(ap, data)?;
    } else {
        write_contents_to_file(path, data)?;
    }
    return Ok(());
}

/// Imports csv into database. <b>WARNING</b>, overrides may occur!
pub fn import_keeper(path: &String) -> Result<()> {
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_path(path)?;

    for result in rdr.records() {
        let record: StringRecord = match result {
            Ok(it) => it,
            Err(err) => {
                error!("Failed to convert csv to StringRecord!: {}", err);
                continue;
            } // TODO: Fix with more elegant handling.
        };
        let fc: FileCrypt = match record.deserialize(None) {
            Ok(it) => it,
            Err(err) => {
                error!("Failed to convert StringRecord to FileCrypt!: {}", err);
                FileCrypt::default()
            } // TODO: Fix with more elegant handling.
        };
        _ = insert_crypt(&fc);
    }

    return Ok(());
}

///Grabs the connection
///
/// # Example:
///```ignore
/// let conn = get_keeper()?;
/// conn.execute("SELECT * FROM *");
///```
pub fn get_keeper() -> Result<r2d2::PooledConnection<r2d2_sqlite::SqliteConnectionManager>> {
    //Returns the static connection
    let keeper = KEEPER.get()?;
    return Ok(keeper);
}

///Insert a crypt into the database
///
/// # Example:
///```ignore
/// let fc = FileCrypt::new({...});
/// let _ = insert_crypt(&fc);
///```
pub fn insert_crypt(crypt: &FileCrypt) -> Result<()> {
    //Get the connection
    let conn = get_keeper()?;

    //Create insert command and execute -- should handle uuid conflicts
    conn.execute(
        "INSERT INTO crypt (
            uuid,
            filename,
            extension,
            drive_id,
            full_path,
            key_seed,
            nonce_seed,
            hash
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
        ON CONFLICT(uuid) DO UPDATE SET
            filename = excluded.filename,
            extension = excluded.extension,
            drive_id = excluded.drive_id,
            full_path = excluded.full_path,
            key_seed = excluded.key_seed,
            nonce_seed = excluded.nonce_seed,
            hash = excluded.hash",
        params![
            &crypt.uuid,
            &crypt.filename,
            &crypt.ext,
            &crypt.drive_id,
            &crypt.full_path.to_str().unwrap_or_default(),
            &crypt.key.as_ref(),
            &crypt.nonce.as_ref(),
            &crypt.hash.as_ref(),
        ],
    )?;

    return Ok(());
}

///Inserts a token into the database
///
/// # Example:
///```ignore
/// let ut = UserToken::new({...});
/// let _ = insert_token(&ut);
///```
pub fn insert_token(user_token: &UserToken) -> Result<()> {
    //Get the connection
    let conn = get_keeper()?;

    //Create insert command and execute -- should handle uuid conflicts
    conn.execute(
        "INSERT INTO user_token (
            service,
            key_seed,
            nonce_seed,
            expiration
        ) VALUES (?1, ?2, ?3, ?4)
        ON CONFLICT(service) DO UPDATE SET
            key_seed = excluded.key_seed,
            nonce_seed = excluded.nonce_seed,
            expiration = excluded.expiration",
        params![
            &user_token.service.to_string(),
            &user_token.key_seed.as_ref(),
            &user_token.nonce_seed.as_ref(),
            &user_token.expiration,
        ],
    )?;

    return Ok(());
}

///Queries the database for the crypt
///
/// # Example:
///```ignore
/// let uuid = generate_uuid();
/// let fc = query_crypt(uuid);
///```
pub fn query_crypt(uuid: String) -> Result<FileCrypt> {
    //Get the connection
    let conn = get_keeper()?;

    //Get the results of the query
    let filecrypt = conn.query_row(
        "SELECT *
        FROM crypt
        WHERE uuid = ?1",
        params![uuid],
        |row| {
            let path: String = row.get(4)?;
            Ok(FileCrypt {
                uuid: row.get(0)?,
                filename: row.get(1)?,
                ext: row.get(2)?,
                drive_id: row.get(3)?,
                full_path: PathBuf::from(path),
                key: row.get(5)?,
                nonce: row.get(6)?,
                hash: row.get(7)?,
            })
        },
    )?;

    return Ok(filecrypt);
}

///Queries the database for the token
///
/// # Example:
///```ignore
/// let cs = CloudService::Google;
/// let user_token = query_token(&cs);
///```
pub fn query_token(service: CloudService) -> Result<UserToken> {
    //Get the connection
    let conn = get_keeper()?;

    //Get the results of the query
    let token = conn.query_row(
        "SELECT *
        FROM user_token
        WHERE service = ?1",
        params![service.to_string()],
        |row| {
            let service: String = row.get(0)?;
            let expiration: u64 = row.get(3)?;
            Ok(UserToken {
                service: CloudService::from_str(&service)
                    .map_err(|e| rusqlite::Error::UserFunctionError(Box::new(e)))?,
                key_seed: row.get(1)?,
                nonce_seed: row.get(2)?,
                expiration,
                access_token: String::new(),
            })
        },
    )?;

    return Ok(token);
}

///Queries the database if a file's metadata matches existing entry in crypt keeper
///
/// # Example:
///```ignore
/// let path = PathBuf::from("path/to/file.txt");
/// let fc = insert_crypt(path);
///```
pub fn query_keeper_for_existing_file(full_path: PathBuf) -> Result<FileCrypt> {
    //Get the connection
    let conn = get_keeper()?;

    //Get the results of the query
    let filecrypt = conn.query_row(
        "SELECT *
        FROM crypt
        WHERE full_path = ?1",
        params![full_path.to_str().unwrap_or_default().to_string()],
        |row| {
            let path: String = row.get(4)?;
            Ok(FileCrypt {
                uuid: row.get(0)?,
                filename: row.get(1)?,
                ext: row.get(2)?,
                drive_id: row.get(3)?,
                full_path: PathBuf::from(path),
                key: row.get(5)?,
                nonce: row.get(6)?,
                hash: row.get(7)?,
            })
        },
    )?;

    return Ok(filecrypt);
}

///Queries the database if a file's metadata matches existing entry in crypt keeper
///
/// # Example:
///```ignore
/// let path = PathBuf::from("path/to/file.txt");
/// let fc = insert_crypt(path);
///```
pub fn query_keeper_by_file_name<T: AsRef<Path>>(file_name: &T) -> Result<FileCrypt> {
    let file_name = file_name.as_ref();
    //Get the connection
    let conn = get_keeper()?;

    //Get the results of the query
    let filecrypt = conn.query_row(
        "SELECT *
        FROM crypt
        WHERE filename = ?1",
        params![file_name.display().to_string()],
        |row| {
            let get: String = row.get(4)?;
            Ok(FileCrypt {
                uuid: row.get(0)?,
                filename: row.get(1)?,
                ext: row.get(2)?,
                drive_id: row.get(3)?,
                full_path: PathBuf::from(get),
                key: row.get(5)?,
                nonce: row.get(6)?,
                hash: row.get(7)?,
            })
        },
    )?;

    return Ok(filecrypt);
}

/// Searches the Crypt for FileCrypts whose `drive_id` IS NOT NULL AND IS NOT "", and returns those results in a vector.
pub fn query_keeper_for_files_with_drive_id() -> Result<Vec<FileCrypt>> {
    //Get the connection
    let conn = get_keeper()?;

    //Create the query and execute
    let mut query = conn.prepare(
        r#"
        SELECT *
        FROM crypt
        WHERE drive_id IS NOT NULL AND drive_id != "" "#,
    )?;

    //Get the results of the query
    let query_result = query.query_map([], |row| {
        let path: String = row.get(4)?;
        let key: [u8; KEY_SIZE] = row.get(5)?;
        let nonce: [u8; NONCE_SIZE] = row.get(6)?;
        let hash: [u8; KEY_SIZE] = row.get(7)?;

        Ok(FileCrypt {
            uuid: row.get(0)?,
            filename: row.get(1)?,
            ext: row.get(2)?,
            drive_id: row.get(3)?,
            full_path: PathBuf::from(path),
            key,
            nonce,
            hash,
        })
    })?;

    //Convert the results into a vector
    let mut crypts = vec![];
    for crypt in query_result.into_iter() {
        crypts.push(crypt?);
    }

    return Ok(crypts);
}

///Queries the database for all crypts
///
/// # Example:
///```ignore
/// let fc = FileCrypt::new({...});
/// let _ = insert_crypt(&fc);
///```
pub fn query_keeper_crypt() -> Result<Vec<FileCrypt>> {
    //Get the connection
    let conn = get_keeper()?;

    //Create the query and execute
    let mut query = conn.prepare(
        "
        SELECT *
        FROM crypt",
    )?;

    //Get the results of the query
    let query_result = query.query_map([], |row| {
        let get: String = row.get(4)?;
        let key: [u8; KEY_SIZE] = row.get(5)?;
        let nonce: [u8; NONCE_SIZE] = row.get(6)?;
        let hash: [u8; KEY_SIZE] = row.get(7)?;

        Ok(FileCrypt {
            uuid: row.get(0)?,
            filename: row.get(1)?,
            ext: row.get(2)?,
            drive_id: row.get(3)?,
            full_path: PathBuf::from(get),
            key,
            nonce,
            hash,
        })
    })?;

    //Convert the results into a vector
    let mut crypts = vec![];
    for crypt in query_result.into_iter() {
        crypts.push(crypt?);
    }

    return Ok(crypts);
}

///Queries the database for all tokens
// /
// / # Example:
// /```ignore
// / let fc = FileCrypt::new({...});
// / let _ = insert_crypt(&fc);
// /```
pub fn query_keeper_token() -> Result<Vec<UserToken>> {
    //Get the connection
    let conn = get_keeper()?;

    //Create the query and execute
    let mut query = conn.prepare(
        "
        SELECT *
        FROM user_token",
    )?;

    //Get the results of the query
    let query_result = query.query_map([], |row| {
        let service: String = row.get(0)?;
        let key: [u8; KEY_SIZE] = row.get(1)?;
        let nonce: [u8; NONCE_SIZE] = row.get(2)?;

        Ok(UserToken {
            service: CloudService::from_str(&service)
                .map_err(|e| rusqlite::Error::UserFunctionError(Box::new(e)))?,
            key_seed: key,
            nonce_seed: nonce,
            expiration: row.get(3)?,
            access_token: String::new(),
        })
    })?;

    //Convert the results into a vector
    let mut tokens = vec![];
    for token in query_result.into_iter() {
        tokens.push(token?);
    }

    return Ok(tokens);
}

///Deletes the crypt
///
///
pub fn delete_crypt(uuid: String) -> Result<()> {
    //Get the connection
    let conn = get_keeper()?;

    conn.execute(
        "
            DELETE FROM crypt WHERE uuid = ?
        ",
        params![uuid],
    )?;

    Ok(())
}

///Delete the database
// /
// / # Example:
// /```ignore
// / let fc = FileCrypt::new({...});
// / let _ = insert_crypt(&fc);
// /```
pub fn delete_keeper() -> Result<()> {
    let path;
    {
        let config = get_config();
        path = config.database_path.to_string();
    }
    if Path::new(&path).exists() {
        fs::remove_file(path)?;
    }
    Ok(())
}
