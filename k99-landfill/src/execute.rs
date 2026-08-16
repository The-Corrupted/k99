use k99_landfill::error::LandFillError;
use nix::unistd::execvpe;
use std::ffi::{CString, OsString};
use std::os::unix::ffi::{OsStrExt, OsStringExt};

pub(crate) fn execute(argv: Vec<OsString>) -> Result<(), LandFillError> {
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
