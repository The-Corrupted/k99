pub mod cli;
pub mod execute;

use k99_landfill::error::LandFillError;
use k99_landfill::file::FileObject;
use k99_landfill::hierarchy::Hierarchy;
use k99_landfill::rule::OperationType;
use k99_landfill::rule::{Abi, PermissionOp, RuleSpec::Fs, normalize_rule};
use landlock::{
    Access, AccessFs, BitFlags, Compatible, PathBeneath, PathFd, RestrictSelfAttr, Ruleset,
    RulesetAttr, RulesetCreatedAttr,
};
use std::path::PathBuf;

use crate::cli::parse_args;
use crate::execute::execute;

fn main() -> Result<(), LandFillError> {
    let mut hierarchy = Hierarchy::new();
    let mut abi = Abi::default();

    let parsed = parse_args(&mut abi)?;

    for spec in parsed.0 {
        match spec {
            Fs { path, op, mode } => {
                let fo = FileObject::from_path(path)?;
                let normalized = normalize_rule(&fo, &mode, abi.get());
                let pem_op = match op {
                    OperationType::Add => PermissionOp::Add(normalized),
                    OperationType::Remove => PermissionOp::Remove(normalized),
                };
                hierarchy.insert_or_update(fo, pem_op);
            }
            _ => {}
        }
    }

    let compiled_rules = hierarchy.compile_rules();
    let access_all = AccessFs::from_all(abi.get());
    Ruleset::default()
        .handle_access(access_all)?
        .set_compatibility(landlock::CompatLevel::HardRequirement)
        .create()?
        .add_rules(
            compiled_rules
                .iter()
                .map::<Result<_, LandFillError>, _>(|p| {
                    Ok(PathBeneath::new(PathFd::new(p.path.clone())?, p.rules))
                }),
        )?
        .no_new_privs(true)
        .all_threads(true)?
        .restrict_self()?;

    println!("{:?}", compiled_rules);

    execute(parsed.1)?;

    Ok(())
}
