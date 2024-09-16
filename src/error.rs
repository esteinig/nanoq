use std::path::PathBuf;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    /// Represents all other cases of `niffler::Error`.
    #[error(transparent)]
    NifflerError(#[from] niffler::Error),
    /// Failed to create a file at the specified path
    #[error("failed to create file: {0}")]
    CreateFilePath(PathBuf),
}