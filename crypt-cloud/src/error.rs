use thiserror::Error;

use crypt_core::prelude::Error as core_error;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    CoreError(#[from] core_error),

    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error),

    #[error(transparent)]
    ToStrError(#[from] reqwest::header::ToStrError),

    #[error("Missing Header: {0}")]
    HeaderError(&'static str),

    #[error("Could not query Google Drive.")]
    GeneralQueryError(serde_json::value::Value),

    #[error("Could not query the directory.")]
    DirectoryQueryError,

    #[error("Folder not found.")]
    FolderNotFoundError,

    #[error("Could not query file ID.")]
    FileIdError,

    #[error("Failed to upload: {0}")]
    ResponseError(u16),

    #[error("Failed to upload.")]
    UploadError,

    #[error("Error acessing root 'crypt' directory.")]
    RootDirectoryError,

    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    /// Errors that should/will never happen.
    #[error(transparent)]
    Infallible(#[from] std::convert::Infallible),
}
