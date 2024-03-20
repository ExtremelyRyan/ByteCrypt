use thiserror::Error;

use crypt_cloud::crypt_core::prelude::Error as core_error;
use crypt_cloud::prelude::Error as cloud_error;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    CloudError(#[from] cloud_error),

    #[error(transparent)]
    CoreError(#[from] core_error),

    #[error(transparent)]
    DirectiveError(#[from] DirectiveError),

    #[error(transparent)]
    UploadError(#[from] UploadError),

    #[error(transparent)]
    IoError(#[from] std::io::Error),

    #[error(transparent)]
    StdError(#[from] Box<dyn std::error::Error>),
}

/// Generic Cloud Errors
#[derive(Debug, Error)]
pub enum DirectiveError {
    /// Error accessing Crypt "root" folder
    #[error("Error accessing Crypt 'root' folder")]
    RemoteCryptDirectoryAccessError,
}

#[derive(Debug, Error)]
pub enum UploadError {
    /// Generated when the user aborts the operation.
    #[error("User aborted the operation")]
    UserAbortedError,

    /// Generated if no crypt files exist within the directory provided.
    #[error("no files were found in the directory provided")]
    NoCryptFilesFound,
}

#[derive(Debug, Error)]
pub enum DownloadError {}
