use crate::error::LandFillError;
use std::os::unix::fs::FileTypeExt;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileObject {
    RegularFile(PathBuf),
    Directory(PathBuf),
    BlockDevice(PathBuf),
    CharDevice(PathBuf),
    Fifo(PathBuf),
    Socket(PathBuf),
}

impl FileObject {
    /// Canonicalizes the path and determines the type of file. This returns a new File
    /// with a canonical path or an error if the path doesn't exist or couldn't be normalized
    pub fn from_path(path: PathBuf) -> Result<Self, LandFillError> {
        let canonical = path.canonicalize()?;

        // This probably shouldn't fail unless there's some sort of race between us canonicalizing the path
        // and the path somehow disappearing
        let metadata = canonical.metadata()?;

        let file_type = metadata.file_type();

        if file_type.is_socket() {
            Ok(Self::Socket(canonical))
        } else if file_type.is_fifo() {
            Ok(Self::Fifo(canonical))
        } else if file_type.is_char_device() {
            Ok(Self::CharDevice(canonical))
        } else if file_type.is_block_device() {
            Ok(Self::BlockDevice(canonical))
        } else if file_type.is_dir() {
            Ok(Self::Directory(canonical))
        } else if file_type.is_file() {
            Ok(Self::RegularFile(canonical))
        } else {
            // Something is fucked up here. We couldn't discover the file type. Return an error
            Err(LandFillError::InvalidFileOrDirectory(canonical))
        }
    }

    pub fn path(&self) -> &Path {
        match self {
            Self::RegularFile(path)
            | Self::Directory(path)
            | Self::BlockDevice(path)
            | Self::CharDevice(path)
            | Self::Fifo(path)
            | Self::Socket(path) => path,
        }
    }
}
