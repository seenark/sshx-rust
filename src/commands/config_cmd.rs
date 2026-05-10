use crate::cli;
use crate::error::SshxError;

fn cmd_validate(cli: &cli::Cli) -> Result<(), SshxError> {
    let ssh_config = cli.config.as_ref();
    let _index = crate::index::ConfigIndex::load(ssh_config)?;
    println!("✓ All SSH configurations are valid");
    Ok(())
}

pub fn config(action: cli::ConfigAction, cli: &cli::Cli) -> Result<(), SshxError> {
    match action {
        cli::ConfigAction::Validate => cmd_validate(cli),
        _ => {
            println!("Command not yet implemented: {:?}", action);
            Ok(())
        }
    }
}