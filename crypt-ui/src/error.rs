use thiserror::Error;

use crate::crypt_core::prelude::Error as core_error;


#[derive(Debug, Error)]
pub enum Error {

}


/// Generic Cloud Errors
#[derive(Debug, Error)]
pub enum CloudError {
    /// Error accessing Crypt "root" folder
    #[error("Error accessing Crypt 'root' folder")]
    RemoteCryptDirectoryAccessError,

    /// Runtime error
    #[error("Runtime error")]
    RuntimeError(#[from] std::io::Error),
}