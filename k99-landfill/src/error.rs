use landlock::{PathFdError, RulesetError};
use std::ffi::NulError;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LandFillError {
    #[error(transparent)]
    ExecError(#[from] nix::Error),
    #[error(transparent)]
    NullError(#[from] NulError),
    #[error(transparent)]
    ParseError(#[from] lexopt::Error),
    #[error(transparent)]
    PathFdError(#[from] PathFdError),
    #[error(transparent)]
    RulesetError(#[from] RulesetError),
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error("Invalid usage. A command must be present")]
    MissingCommandError,
    #[error("Invalid file or directory {0}")]
    InvalidFileOrDirectory(PathBuf),
    #[error("Invalid permission selected for file type {pb} {pem}")]
    InvalidPermission { pb: PathBuf, pem: String },
}
