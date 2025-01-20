use crate::error::{CliError, Result};

pub fn parse_key_val(s: &str) -> Result<(String, String)> {
    let pos = s.find('=').ok_or_else(|| {
        CliError::InvalidArgument(format!("Invalid KEY=value: no `=` found in `{}`", s))
    })?;
    Ok((s[..pos].to_string(), s[pos + 1..].to_string()))
}
