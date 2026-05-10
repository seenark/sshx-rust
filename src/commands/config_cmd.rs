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

fn cmd_show(host_alias: Option<&str>, cli: &cli::Cli) -> Result<(), SshxError> {
    let alias = host_alias.ok_or_else(|| SshxError::HostNotFound {
        input: "(none)".to_string(),
    })?;
    let index = crate::index::ConfigIndex::load(cli.config.as_ref())?;
    let host = index
        .resolve_alias(alias)
        .ok_or_else(|| SshxError::HostNotFound {
            input: alias.to_string(),
        })?;

    println!("{}", console::style(format!("Host {}", host.name)).bold());
    println!("  HostName {}", host.hostname);
    if let Some(port) = host.port {
        println!("  Port {port}");
    }
    if let Some(ref user) = host.user {
        println!("  User {user}");
    }
    if let Some(ref idf) = host.identity_file {
        println!("  IdentityFile {}", idf.display());
    }
    for lf in &host.local_forwards {
        println!("  LocalForward {} {}:{}", lf.local_port, lf.remote_host, lf.remote_port);
    }
    if let Some(ref shc) = host.strict_host_checking {
        let val = match shc {
            crate::model::StrictHostChecking::Yes => "yes",
            crate::model::StrictHostChecking::No => "no",
            crate::model::StrictHostChecking::Ask => "ask",
            crate::model::StrictHostChecking::AcceptNew => "accept-new",
        };
        println!("  StrictHostKeyChecking {val}");
    }
    if let Some(ref ukhf) = host.user_known_hosts_file {
        println!("  UserKnownHostsFile {ukhf}");
    }
    for (key, value) in &host.extra_options {
        println!("  {key} {value}");
    }
    println!();
    if let Some(ref group) = host.sshx.group {
        println!("  {} sshx: group = {group}", console::style("##").dim());
    }
    if let Some(ref alias) = host.sshx.alias {
        println!("  {} sshx: alias = {alias}", console::style("##").dim());
    }
    if let Some(ref desc) = host.sshx.description {
        println!("  {} sshx: description = \"{desc}\"", console::style("##").dim());
    }
    if let Some(ref pw) = host.sshx.password {
        println!("  {} sshx: password = {}", console::style("##").dim(), "*".repeat(pw.len()));
    }
    if let Some(ref req) = host.sshx.requires {
        println!("  {} sshx: requires = {req}", console::style("##").dim());
    }
    if host.sshx.background {
        println!("  {} sshx: background = true", console::style("##").dim());
    }
    if let Some(ref ac) = host.sshx.after_connect {
        println!("  {} sshx: after_connect = \"{ac}\"", console::style("##").dim());
    }

    println!();
    println!(
        "  Source: {}:{}-{}",
        host.source.file.display(),
        host.source.line_start,
        host.source.line_end
    );
    Ok(())
}

pub fn config(action: cli::ConfigAction, cli: &cli::Cli) -> Result<(), SshxError> {
    match action {
        cli::ConfigAction::Validate => cmd_validate(cli),
        cli::ConfigAction::List { group } => cmd_list(cli, group.as_deref()),
        cli::ConfigAction::Show { host_alias } => cmd_show(Some(&host_alias), cli),
        _ => {
            println!("Command not yet implemented: {:?}", action);
            Ok(())
        }
    }
}