use crate::cli;
use crate::error::SshxError;

pub fn config(_action: cli::ConfigAction, _cli: &cli::Cli) -> Result<(), SshxError> {
    println!("config command — not yet implemented");
    Ok(())
}