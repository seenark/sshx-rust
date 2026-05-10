use crate::cli;
use crate::error::SshxError;
use crate::index::ConfigIndex;
use crate::ssh_command::SSHCommand;

pub fn connect(host_alias: Option<&str>, cli: &cli::Cli) -> Result<(), SshxError> {
    let index = ConfigIndex::load(cli.config.as_ref())?;

    let host_name = match host_alias {
        Some(input) => {
            match index.resolve_alias(input) {
                Some(host) => Some(host.name.clone()),
                None => {
                    if cli.verbose {
                        eprintln!("No exact match for \"{input}\", opening selector...");
                    }
                    crate::selector::select_host(&index, Some(input))
                }
            }
        }
        None => crate::selector::select_host(&index, None),
    };

    let host_name = host_name.ok_or_else(|| SshxError::HostNotFound {
        input: host_alias.unwrap_or("(selector cancelled)").to_string(),
    })?;

    let host = index.find_host(&host_name).unwrap();

    if let Some(ref requires) = host.sshx.requires {
        let jump_host = index.jump_host_for(host).ok_or_else(|| {
            SshxError::RequiresHostNotFound {
                host: host.name.clone(),
                requires: requires.clone(),
            }
        })?;
        crate::tunnel::ensure_tunnel(jump_host, &index, cli)?;
    }

    let ssh_cmd = SSHCommand::from_host(host);

    if cli.verbose || cli.dry_run {
        eprintln!("Command: {}", ssh_cmd.build());
        if cli.dry_run {
            return Ok(());
        }
    }

    if host.sshx.password.is_some() {
        which::which("sshpass").map_err(|_| SshxError::SshpassNotFound)?;
    }
    which::which("ssh").map_err(|_| SshxError::SshNotFound)?;

    let parts = ssh_cmd.build_parts();
    let status = std::process::Command::new(&parts[0])
        .args(&parts[1..])
        .status()
        .map_err(|e| SshxError::SshCommandFailed {
            exit_code: None,
        })?;

    if !status.success() {
        return Err(SshxError::SshCommandFailed {
            exit_code: status.code(),
        });
    }

    if let Some(ref cmd) = host.sshx.after_connect {
        if cli.verbose {
            eprintln!("Running after_connect: {cmd}");
        }
        let ac_status = std::process::Command::new("sh")
            .arg("-c")
            .arg(cmd)
            .status()
            .map_err(|_| SshxError::SshCommandFailed { exit_code: None })?;
        if !ac_status.success() {
            eprintln!("Warning: after_connect command failed");
        }
    }

    Ok(())
}