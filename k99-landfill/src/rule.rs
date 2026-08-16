// Rule handlers. These are responsible for constructing rule sets and creating intermediary formats the rest of
// the application can use.

use crate::file::FileObject;
use landlock::{ABI, Access, AccessFs, BitFlags, make_bitflags};
use std::path::PathBuf;

// Requested capabilities

#[inline]
fn read_only() -> BitFlags<AccessFs> {
    make_bitflags!(AccessFs::{ReadFile | ReadDir})
}

#[inline]
fn write() -> BitFlags<AccessFs> {
    make_bitflags!(AccessFs::{WriteFile
        | Truncate
        | RemoveFile
        | RemoveDir
        | MakeChar
        | MakeDir
        | MakeReg
        | MakeSock
        | MakeFifo
        | MakeBlock
        | MakeSym
        | Refer
        | IoctlDev
        | ResolveUnix
    })
}

#[inline]
fn read_write() -> BitFlags<AccessFs> {
    read_only() | write()
}

#[inline]
fn execute() -> BitFlags<AccessFs> {
    read_only() | AccessFs::Execute
}

#[inline]
fn read_write_execute() -> BitFlags<AccessFs> {
    read_write() | AccessFs::Execute
}

// Valid object capabilities
#[inline]
fn regular_file_access() -> BitFlags<AccessFs> {
    make_bitflags!(AccessFs::{ReadFile
        | WriteFile
        | Truncate
        | Execute
    })
}

// Directory permissions can accept any and all permissions that would make sense for any
// file to have
#[inline]
fn directory_access() -> BitFlags<AccessFs> {
    read_write_execute()
}

#[inline]
fn device_access() -> BitFlags<AccessFs> {
    make_bitflags!(AccessFs::{ReadFile
        | WriteFile
        | IoctlDev
    })
}

#[inline]
fn fifo_access() -> BitFlags<AccessFs> {
    make_bitflags!(AccessFs::{
        ReadFile
        | WriteFile
    })
}

#[inline]
fn socket_access() -> BitFlags<AccessFs> {
    make_bitflags!(AccessFs::{
        ResolveUnix
    })
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum FsMode {
    ReadOnly,
    ReadWrite,
    Execute,
    ReadWriteExecute,
    Explicit(BitFlags<AccessFs>),
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum NetMode {
    BindTcp,
    ConnectTcp,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum OperationType {
    Add,
    Remove,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuleSpec {
    Fs {
        path: PathBuf,
        op: OperationType,
        mode: FsMode,
    },
    Net {
        port: u16,
        mode: NetMode,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PermissionOp {
    Add(BitFlags<AccessFs>),
    Remove(BitFlags<AccessFs>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Abi {
    abi: ABI,
}

impl Abi {
    pub fn default() -> Self {
        Self { abi: ABI::V9 }
    }

    pub fn set(&mut self, abi: ABI) {
        self.abi = abi;
    }

    pub fn get(&self) -> ABI {
        self.abi
    }

    pub fn is_access_supported(&self, access: BitFlags<AccessFs>) -> bool {
        let supported = AccessFs::from_all(self.abi) & access;
        supported.contains(access)
    }
}

impl FsMode {
    fn to_access(&self) -> BitFlags<AccessFs> {
        match self {
            Self::ReadOnly => read_only(),
            Self::ReadWrite => read_write(),
            Self::Execute => execute(),
            Self::ReadWriteExecute => read_write_execute(),
            Self::Explicit(access) => *access,
        }
    }
}

impl FileObject {
    pub fn valid_access(&self) -> BitFlags<AccessFs> {
        match self {
            Self::RegularFile(_) => regular_file_access(),
            Self::Directory(_) => directory_access(),
            Self::BlockDevice(_) | Self::CharDevice(_) => device_access(),
            Self::Fifo(_) => fifo_access(),
            Self::Socket(_) => socket_access(),
        }
    }
}

pub fn normalize_rule(file_kind: &FileObject, fs_mode: &FsMode, abi: ABI) -> BitFlags<AccessFs> {
    match fs_mode {
        FsMode::Explicit(e) => *e,
        _ => fs_mode.to_access() & file_kind.valid_access() & AccessFs::from_all(abi),
    }
}
