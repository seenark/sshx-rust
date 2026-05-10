use std::net::TcpStream;
use std::time::{Duration, Instant};

use crate::cli;
use crate::config::SshxConfig;
use crate::error::SshxError;
use crate::index::ConfigIndex;
use crate::model::SSHHost;
use crate::ssh_command::SSHCommand;

pub fn ensure_tunnel(
    jump_host: &SSHHost,
    index: &ConfigIndex,
    cli: &cli::Cli,
) -> Result<(), SshxError> {
    for port in &jump_host.local_forwards {
        if TcpStream::connect(format!("127.0.0.1:{}", port.local_port)).is_ok() {
            if cli.verbose {
                eprintln!(
                    "Tunnel port {} already in use — assuming tunnel is active",
                    port.local_port
                );
            }
            return Ok(());
        }
    }

    let sshx_config = SshxConfig::load().unwrap_or_default();

    for port in &jump_host.local_forwards {
        if TcpStream::connect(format!("127.0.0.1:{}", port.local_port)).is_ok() {
            return Err(SshxError::TunnelPortBusy {
                port: port.local_port,
            });
        }
    }

    let mut tunnel_cmd = SSHCommand::from_host(jump_host);
    tunnel_cmd.background = true;

    if cli.dry_run {
        eprintln!("Would start tunnel: {}", tunnel_cmd.build());
        return Ok(());
    }

    let parts = tunnel_cmd.build_parts();
    let mut child = std::process::Command::new(&parts[0])
        .args(&parts[1..])
        .spawn()
        .map_err(|e| SshxError::TunnelSpawnFailed {
            jump_host: jump_host.name.clone(),
            reason: e.to_string(),
        })?;

    if cli.verbose {
        eprintln!("Tunnel PID: {:?}", child.id());
    }

    let timeout = Duration::from_secs(sshx_config.tunnel.connect_timeout_s);
    let interval = Duration::from_millis(sshx_config.tunnel.check_interval_ms);
    let start = Instant::now();

    loop {
        let all_ready = jump_host
            .local_forwards
            .iter()
            .all(|lf| TcpStream::connect(format!("127.0.0.1:{}", lf.local_port)).is_ok());

        if all_ready {
            if cli.verbose {
                eprintln!("Tunnel ready");
            }
            return Ok(());
        }

        match child.try_wait() {
            Ok(Some(status)) => {
                return Err(SshxError::TunnelDiedEarly {
                    jump_host: jump_host.name.clone(),
                    exit_code: status.code(),
                });
            }
            Ok(None) => {}
            Err(_) => {
                return Err(SshxError::TunnelDiedEarly {
                    jump_host: jump_host.name.clone(),
                    exit_code: None,
                });
            }
        }

        if start.elapsed() >= timeout {
            let _ = child.kill();
            return Err(SshxError::TunnelTimeout {
                jump_host: jump_host.name.clone(),
                timeout_s: sshx_config.tunnel.connect_timeout_s,
            });
        }

        std::thread::sleep(interval);
    }
}