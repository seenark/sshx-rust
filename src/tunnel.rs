use crate::cli;
use crate::error::SshxError;
use crate::index::ConfigIndex;
use crate::model::SSHHost;

pub fn ensure_tunnel(_jump_host: &SSHHost, _index: &ConfigIndex, _cli: &cli::Cli) -> Result<(), SshxError> {
    Ok(())
}