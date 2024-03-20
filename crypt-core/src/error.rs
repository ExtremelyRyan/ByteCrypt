use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    // #################### Database Errors ####################
    #[error(transparent)]
    DatabaseError(#[from] DatabaseError),

    #[error(transparent)]
    DbPoolingError(#[from] r2d2::Error),

    #[error(transparent)]
    DbError(#[from] rusqlite::Error),

    #[error(transparent)]
    CsvError(#[from] csv::Error),

    // #################### Token Errors ####################
    #[error(transparent)]
    TokenError(#[from] TokenError),

    // #################### FileCrypt Errors ####################
    #[error(transparent)]
    FcError(#[from] FcError),

    // #################### General Errors ####################
    #[error(transparent)]
    IoError(#[from] std::io::Error),

    /// Errors that should/will never happen.
    #[error(transparent)]
    Infallible(#[from] std::convert::Infallible),
}

/// Error types for Cloud Tokens
#[derive(Debug, Error)]
pub enum TokenError {
    #[error("Invalid platform.")]
    InvalidPlatform,

    #[error("Path does not exist.")]
    PathDoesNotExist,

    #[error("Expired token.")]
    ExpiredToken,

    #[error(transparent)]
    DbError(#[from] rusqlite::Error),
}

#[derive(Debug, Error)]
pub enum DatabaseError {
    #[error(transparent)]
    DbError(#[from] rusqlite::Error),
}

/// Represents various errors that can occur during file decryption.
///
/// The `FcError` enum provides specific error variants for different failure scenarios
/// encountered during the decryption process.
///
/// # Variants
///
/// - `HashFail(String)`: Hash comparison between file and decrypted content failed.
/// - `InvalidFilePath`: The provided file path is invalid.
/// - `CryptQueryError`: Failed to query the cryptographic information.
/// - `DecompressionError`: Failed to decompress the decrypted content.
/// - `FileDeletionError(std::io::Error, String)`: Failed to delete the original file.
/// - `FileReadError`: An error occurred while reading the file.
/// - `FileError(String)`: An error occurred during file operations (read or write).
/// - `DecryptError(String)`: Failed to decrypt the file contents.
///
/// # Examples
///
/// ```rust ignore
/// use crypt_core::FcError;
///
/// fn handle_error(err: FcError) {
///     match err {
///         FcError::HashFail(message) => eprintln!("Hash failure: {}", message),
///         FcError::InvalidFilePath => eprintln!("Invalid file path."),
///         FcError::CryptQueryError => eprintln!("Cryptographic query failed."),
///         FcError::DecompressionError => eprintln!("Decompression failed."),
///         FcError::FileDeletionError(io_err, path) => eprintln!("Failed to delete file {}: {:?}", path, io_err),
///         FcError::FileReadError => eprintln!("Error reading file."),
///         FcError::FileError(message) => eprintln!("File operation error: {}", message),
///         FcError::DecryptError(message) => eprintln!("Decryption error: {}", message),
///     }
/// }
/// ```
///
#[derive(Debug, Error)]
pub enum FcError {
    #[error("Hash comparison failed. {0}")]
    HashFail(String),

    #[error("")]
    InvalidFilePath,

    #[error("")]
    CryptQueryError,

    #[error("file decompression failed. {0}")]
    DecompressionError(String),

    #[error("")]
    FileDeletionError(std::io::Error, String),

    #[error("")]
    FileReadError,

    #[error("Error loading file")]
    FileError(String),

    #[error("Decryption failed: {0}")]
    DecryptError(String),

    #[error("Other error occured. {0}")]
    GeneralError(String),
}

#[derive(Debug, Error)]
pub enum EncryptionError {
    #[error("ChaCha encryption error: {0}")]
    ChaChaError(#[from] chacha20poly1305::Error),
}
