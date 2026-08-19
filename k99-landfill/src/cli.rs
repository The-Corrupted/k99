use k99_landfill::error::LandFillError;
use k99_landfill::rule::{Abi, FsMode, OperationType, RuleSpec};
use std::ffi::OsString;
use std::path::PathBuf;

// Simple parse object. Its only job should be to produce rulespec's and add them to the
// vector. On parse_finish we should collect the remaining command and return this such that the
// result of parse_args is Result<(Vec<RuleSpec>, Vec<OsString>), LandFillError>
pub(crate) fn parse_args(abi: &mut Abi) -> Result<(Vec<RuleSpec>, Vec<OsString>), LandFillError> {
    use lexopt::prelude::*;

    let mut parser = lexopt::Parser::from_env();
    let mut rules: Vec<RuleSpec> = Vec::new();
    let mut tail: Vec<OsString> = Vec::new();
    while let Some(arg) = parser.next()? {
        match arg {
            Long("ro") => {
                let spec = RuleSpec::Fs {
                    path: PathBuf::from(parser.value()?),
                    op: OperationType::Add,
                    mode: FsMode::ReadOnly,
                };
                rules.push(spec);
            }
            Long("rw") => {
                let spec = RuleSpec::Fs {
                    path: PathBuf::from(parser.value()?),
                    op: OperationType::Add,
                    mode: FsMode::ReadWrite,
                };
                rules.push(spec);
            }
            Long("rwx") => {
                let spec = RuleSpec::Fs {
                    path: PathBuf::from(parser.value()?),
                    op: OperationType::Add,
                    mode: FsMode::ReadWriteExecute,
                };
                rules.push(spec);
            }
            Long("exec") => {
                let spec = RuleSpec::Fs {
                    path: PathBuf::from(parser.value()?),
                    op: OperationType::Add,
                    mode: FsMode::Execute,
                };
                rules.push(spec);
            }
            Value(val) => {
                tail.push(val);
                tail.extend(parser.raw_args()?);
                break;
            }
            _ => Err(arg.unexpected())?,
        }
    }

    Ok((rules, tail))
}
