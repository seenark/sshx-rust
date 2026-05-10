use crate::cli;
use crate::error::SshxError;
use crate::model::SSHHost;

fn cmd_validate(cli: &cli::Cli) -> Result<(), SshxError> {
    let ssh_config = cli.config.as_ref();
    let _index = crate::index::ConfigIndex::load(ssh_config)?;
    println!("✓ All SSH configurations are valid");
    Ok(())
}

fn cmd_list(cli: &cli::Cli, group_filter: Option<&str>) -> Result<(), SshxError> {
    let index = crate::index::ConfigIndex::load(cli.config.as_ref())?;

    if let Some(group) = group_filter {
        let hosts = index.hosts_in_group(group);
        if hosts.is_empty() {
            println!("No hosts in group \"{group}\"");
            return Ok(());
        }
        print_host_table(&hosts);
    } else if !index.groups.is_empty() {
        for (group_name, _) in &index.groups {
            let hosts = index.hosts_in_group(group_name);
            println!("\n{}", console::style(format!("[{group_name}]")).bold().cyan());
            print_host_table(&hosts);
        }
        let ungrouped: Vec<&SSHHost> = index
            .hosts
            .iter()
            .filter(|h| h.sshx.group.is_none())
            .collect();
        if !ungrouped.is_empty() {
            println!("\n{}", console::style("[ungrouped]").bold());
            print_host_table(&ungrouped);
        }
    } else {
        print_host_table(&index.hosts.iter().collect::<Vec<_>>());
    }
    Ok(())
}

fn print_host_table(hosts: &[&SSHHost]) {
    for host in hosts {
        let alias_str = host
            .sshx
            .alias
            .as_deref()
            .map(|a| format!(" ({a})"))
            .unwrap_or_default();
        let desc_str = host
            .sshx
            .description
            .as_deref()
            .map(|d| format!(" — {d}"))
            .unwrap_or_default();
        let port_str = host.port.map(|p| format!(":{p}")).unwrap_or_default();
        println!(
            "  {}  {}  {}{}",
            console::style(&host.name).green().bold(),
            console::style(alias_str).yellow(),
            console::style(format!("{}{}", host.hostname, port_str)).dim(),
            console::style(desc_str).dim(),
        );
    }
}

pub fn config(action: cli::ConfigAction, cli: &cli::Cli) -> Result<(), SshxError> {
    match action {
        cli::ConfigAction::Validate => cmd_validate(cli),
        cli::ConfigAction::List { group } => cmd_list(cli, group.as_deref()),
        _ => {
            println!("Command not yet implemented: {:?}", action);
            Ok(())
        }
    }
}