use landlock::{
    ABI, Access, AccessFs, AccessNet, BitFlags, Compatible, NetPort, PathBeneath, PathFd,
    PathFdError, RestrictSelfAttr, RestrictionStatus, Ruleset, RulesetAttr, RulesetCreatedAttr,
    RulesetError, Scope,
};
use lexopt;
use nix::unistd::execvpe;
use serde::Deserialize;
use std::ffi::{CString, NulError, OsString};
use std::os::unix::ffi::{OsStrExt, OsStringExt};
use std::{collections::BTreeMap, path::PathBuf};
use thiserror::Error;

#[derive(Error, Debug)]
enum LandFillError {
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
    #[error("Invalid usage. A command must be present")]
    MissingCommandError,
    #[error("Invalid file or directory {0}")]
    InvalidFileOrDirectory(PathBuf),
    #[error("Invalid permission selected for file type {pb} {pem}")]
    InvalidPermission { pb: PathBuf, pem: String },
}

#[derive(Clone, Debug, Deserialize)]
enum FsTarget {
    File,
    Directory,
}

#[derive(Clone, Debug)]
struct FileRule {
    file: FsTarget,
    rules: BitFlags<AccessFs>,
}

#[derive(Clone, Debug)]
struct NetworkRule {
    port: u32,
    rules: BitFlags<AccessNet>,
}

#[derive(Clone, Debug)]
struct K99LandFillBuilder {
    fs_rules: BTreeMap<PathBuf, FileRule>,
    //    network_rules: BTreeMap<PathBuf, NetworkRule>,
    //    scope_rules: BitFlags<Scope>,
    abi: ABI,
}

impl K99LandFillBuilder {
    fn default() -> Self {
        Self {
            fs_rules: BTreeMap::new(),
            // network_rules: BTreeMap::new(),
            // scope_rules: BitFlags::empty(),
            abi: ABI::V9,
        }
    }

    fn fs_access<A: Into<BitFlags<AccessFs>>>(
        mut self,
        key: PathBuf,
        access: A,
    ) -> Result<Self, LandFillError> {
        if let Some(entry) = self.fs_rules.get_mut(&key) {
            entry.update_permissions(access);
        } else {
            let target = if key.is_file() {
                FsTarget::File
            } else if key.is_dir() {
                FsTarget::Directory
            } else {
                return Err(LandFillError::InvalidFileOrDirectory(key));
            };

            let mut rule = FileRule::new(target);
            rule.update_permissions(access);
            self.fs_rules.insert(key, rule);
        }
        Ok(self)
    }

    fn compile(self) -> Result<RestrictionStatus, LandFillError> {
        Ok(Ruleset::default()
            .set_compatibility(landlock::CompatLevel::HardRequirement)
            .handle_access(AccessFs::from_all(self.abi))?
            .create()?
            .add_rules(
                self.fs_rules
                    .iter()
                    .map::<Result<_, LandFillError>, _>(|(k, v)| {
                        Ok(PathBeneath::new(PathFd::new(k)?, v.rules))
                    }),
            )?
            .no_new_privs(true)
            .all_threads(true)?
            .restrict_self()?)
    }
}

impl FileRule {
    fn new(file: FsTarget) -> Self {
        Self {
            file,
            rules: BitFlags::empty(),
        }
    }

    fn update_permissions<A: Into<BitFlags<AccessFs>>>(&mut self, flags: A) {
        self.rules |= flags.into();
    }
}

fn parse_args() -> Result<(K99LandFillBuilder, Vec<OsString>), LandFillError> {
    use lexopt::prelude::*;

    let mut k99 = K99LandFillBuilder::default();
    let mut parser = lexopt::Parser::from_env();
    let mut tail = Vec::new();
    while let Some(arg) = parser.next()? {
        match arg {
            Long("ro") => {
                let path = PathBuf::from(parser.value()?);
                let mut permissions = AccessFs::from_read(k99.abi) & !AccessFs::Execute;
                if path.is_file() {
                    permissions &= !AccessFs::ReadDir;
                }

                k99 = k99.fs_access(path, permissions)?;
            }
            Long("rw") => {
                let path = PathBuf::from(parser.value()?);
                let mut permissions = (AccessFs::from_read(k99.abi)
                    | AccessFs::from_write(k99.abi))
                    & !AccessFs::Execute;

                if path.is_file() {
                    permissions &= !AccessFs::ReadDir
                }

                k99 = k99.fs_access(path, permissions)?;
            }
            Long("exec") => {
                let path = PathBuf::from(parser.value()?);
                let mut permissions = AccessFs::from_read(k99.abi) | AccessFs::Execute;
                if path.is_file() {
                    permissions &= !AccessFs::ReadDir;
                }
                k99 = k99.fs_access(path, permissions)?;
            }
            Value(val) => {
                tail.push(val);
                tail.extend(parser.raw_args()?);
                break;
            }
            _ => Err(arg.unexpected())?,
        }
    }

    Ok((k99, tail))
}

fn execute(argv: Vec<OsString>) -> Result<(), LandFillError> {
    if argv.is_empty() {
        return Err(LandFillError::MissingCommandError);
    }

    let command: Vec<CString> = argv
        .iter()
        .map(|e| CString::new(e.as_bytes()))
        .collect::<Result<Vec<_>, _>>()?;

    let vars: Vec<CString> = std::env::vars_os()
        .map(|(key, value)| {
            let key = key.into_vec();
            let value = value.into_vec();

            let mut entry = Vec::with_capacity(key.len() + 1 + value.len());
            entry.extend_from_slice(&key);
            entry.push(b'=');
            entry.extend_from_slice(&value);

            CString::new(entry)
        })
        .collect::<Result<Vec<_>, _>>()?;

    execvpe::<CString, CString>(&command[0], &command, &vars)?;

    Ok(())
}

fn main() -> Result<(), LandFillError> {
    let (k99, tail) = parse_args()?;

    println!("{:?}", k99);
    k99.compile()?;
    execute(tail)?;
    Ok(())
}
