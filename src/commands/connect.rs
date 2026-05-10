use crate::cli;
use crate::error::SshxError;

pub fn connect(_host_alias: Option<&str>, _cli: &cli::Cli) -> Result<(), SshxError> {
    println!("connect command — not yet implemented");
    Ok(())
}