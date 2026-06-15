use crate::cli;
use crate::error::SshxError;
use crate::model::SSHHost;
use shell_words::quote;

fn cmd_init() -> Result<(), SshxError> {
    let config_path = crate::config::SshxConfig::config_path();
    if config_path.exists() {
        println!("Config already exists at {}", config_path.display());
        return Ok(());
    }
    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| SshxError::SshxConfigWriteFailed {
            path: parent.to_path_buf(),
            reason: e.to_string(),
        })?;
    }
    let content = crate::config::SshxConfig::generate_default_toml();
    std::fs::write(&config_path, content).map_err(|e| SshxError::SshxConfigWriteFailed {
        path: config_path.clone(),
        reason: e.to_string(),
    })?;
    println!("✓ Created default config at {}", config_path.display());
    Ok(())
}

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
            println!(
                "\n{}",
                console::style(format!("[{group_name}]")).bold().cyan()
            );
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
        println!(
            "  LocalForward {} {}:{}",
            lf.local_port, lf.remote_host, lf.remote_port
        );
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
        println!(
            "  {} sshx: description = \"{desc}\"",
            console::style("##").dim()
        );
    }
    if let Some(ref pw) = host.sshx.password {
        println!(
            "  {} sshx: password = {}",
            console::style("##").dim(),
            "*".repeat(pw.len())
        );
    }
    if let Some(ref req) = host.sshx.requires {
        println!("  {} sshx: requires = {req}", console::style("##").dim());
    }
    if host.sshx.background {
        println!("  {} sshx: background = true", console::style("##").dim());
    }
    if let Some(ref ac) = host.sshx.after_connect {
        println!(
            "  {} sshx: after_connect = \"{ac}\"",
            console::style("##").dim()
        );
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

fn cmd_edit(host_alias: Option<&str>, cli: &cli::Cli) -> Result<(), SshxError> {
    let alias = host_alias.ok_or_else(|| SshxError::HostNotFound {
        input: "(none)".to_string(),
    })?;
    let index = crate::index::ConfigIndex::load(cli.config.as_ref())?;
    let host = index
        .resolve_alias(alias)
        .ok_or_else(|| SshxError::HostNotFound {
            input: alias.to_string(),
        })?;

    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vi".to_string());
    let file = &host.source.file;
    let line = host.source.line_start;

    let cmd = format!("{editor} +{line} {}", quote(&file.display().to_string()));
    if cli.dry_run {
        println!("Would run: {cmd}");
        return Ok(());
    }

    let status = std::process::Command::new("sh")
        .arg("-c")
        .arg(&cmd)
        .status()
        .map_err(|e| SshxError::ConfigWriteFailed {
            path: file.clone(),
            reason: format!("failed to launch editor: {e}"),
        })?;

    if !status.success() {
        return Err(SshxError::ConfigWriteFailed {
            path: file.clone(),
            reason: "Editor exited with error".to_string(),
        });
    }
    Ok(())
}

fn cmd_remove(host_alias: Option<&str>, cli: &cli::Cli) -> Result<(), SshxError> {
    let alias = host_alias.ok_or_else(|| SshxError::HostNotFound {
        input: "(none)".to_string(),
    })?;
    let index = crate::index::ConfigIndex::load(cli.config.as_ref())?;
    let host = index
        .resolve_alias(alias)
        .ok_or_else(|| SshxError::HostNotFound {
            input: alias.to_string(),
        })?;

    let file = &host.source.file;
    let confirmed = dialoguer::Confirm::new()
        .with_prompt(format!("Remove host \"{}\"?", host.name))
        .default(false)
        .interact()
        .map_err(|e| SshxError::ConfigWriteFailed {
            path: file.clone(),
            reason: e.to_string(),
        })?;

    if !confirmed {
        println!("Cancelled");
        return Ok(());
    }

    if cli.dry_run {
        println!(
            "Would remove host \"{}\" from {}",
            host.name,
            file.display()
        );
        return Ok(());
    }

    let content = std::fs::read_to_string(file).map_err(|e| SshxError::ConfigFileUnreadable {
        path: file.clone(),
        reason: e.to_string(),
    })?;

    let lines: Vec<&str> = content.lines().collect();
    let mut new_lines = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        let line_num = i + 1;
        if line_num < host.source.line_start || line_num > host.source.line_end {
            new_lines.push(*line);
        }
    }

    let new_content = if new_lines.is_empty() {
        String::new()
    } else {
        new_lines.join("\n")
    };
    std::fs::write(file, new_content).map_err(|e| SshxError::ConfigWriteFailed {
        path: file.clone(),
        reason: e.to_string(),
    })?;

    println!("✓ Removed host \"{}\"", host.name);
    Ok(())
}

fn cmd_add(cli: &cli::Cli) -> Result<(), SshxError> {
    let index = crate::index::ConfigIndex::load(cli.config.as_ref())?;

    let name: String = dialoguer::Input::new()
        .with_prompt("Host name")
        .interact_text()
        .map_err(|e| SshxError::ConfigWriteFailed {
            path: "stdin".into(),
            reason: e.to_string(),
        })?;

    if name.contains(' ') || name.contains('\t') {
        return Err(SshxError::InvalidHostName { name });
    }
    if index.find_host(&name).is_some() {
        return Err(SshxError::HostAlreadyExists { name });
    }

    let hostname: String = dialoguer::Input::new()
        .with_prompt("HostName (IP or domain)")
        .interact_text()
        .map_err(|e| SshxError::ConfigWriteFailed {
            path: "stdin".into(),
            reason: e.to_string(),
        })?;

    let port: String = dialoguer::Input::new()
        .with_prompt("Port")
        .default("22".to_string())
        .interact_text()
        .map_err(|e| SshxError::ConfigWriteFailed {
            path: "stdin".into(),
            reason: e.to_string(),
        })?;
    let port: u16 = port
        .parse()
        .map_err(|_| SshxError::InvalidPort { input: port })?;

    let user: String = dialoguer::Input::new()
        .with_prompt("User")
        .interact_text()
        .map_err(|e| SshxError::ConfigWriteFailed {
            path: "stdin".into(),
            reason: e.to_string(),
        })?;

    let group: String = dialoguer::Input::new()
        .with_prompt("Group (optional)")
        .allow_empty(true)
        .interact_text()
        .unwrap_or_default();

    let description: String = dialoguer::Input::new()
        .with_prompt("Description (optional)")
        .allow_empty(true)
        .interact_text()
        .unwrap_or_default();

    let alias: String = dialoguer::Input::new()
        .with_prompt("Alias (optional)")
        .allow_empty(true)
        .interact_text()
        .unwrap_or_default();

    let mut lines = vec![format!("Host {name}")];
    lines.push(format!("    HostName {hostname}"));
    if port != 22 {
        lines.push(format!("    Port {port}"));
    }
    if !user.is_empty() {
        lines.push(format!("    User {user}"));
    }
    if !group.is_empty() {
        lines.push(format!("    ## sshx: group = {group}"));
    }
    if !description.is_empty() {
        lines.push(format!("    ## sshx: description = \"{description}\""));
    }
    if !alias.is_empty() {
        lines.push(format!("    ## sshx: alias = {alias}"));
    }
    lines.push(String::new());

    let block = lines.join("\n");

    if cli.dry_run {
        println!("Would append to config:");
        println!("{block}");
        return Ok(());
    }

    let config_path: std::path::PathBuf = match cli.config.clone() {
        Some(path) => path,
        None => crate::config::SshxConfig::load()?.ssh_config_path(),
    };

    let existing =
        std::fs::read_to_string(&config_path).map_err(|e| SshxError::ConfigWriteFailed {
            path: config_path.clone(),
            reason: e.to_string(),
        })?;
    let new_content = if existing.ends_with('\n') || existing.is_empty() {
        format!("{existing}{block}")
    } else {
        format!("{existing}\n{block}")
    };

    std::fs::write(&config_path, new_content).map_err(|e| SshxError::ConfigWriteFailed {
        path: config_path.clone(),
        reason: e.to_string(),
    })?;

    println!("✓ Added host \"{name}\" to {}", config_path.display());
    Ok(())
}

pub fn config(action: cli::ConfigAction, cli: &cli::Cli) -> Result<(), SshxError> {
    match action {
        cli::ConfigAction::Validate => cmd_validate(cli),
        cli::ConfigAction::List { group } => cmd_list(cli, group.as_deref()),
        cli::ConfigAction::Show { host_alias } => cmd_show(Some(&host_alias), cli),
        cli::ConfigAction::Init => cmd_init(),
        cli::ConfigAction::Add => cmd_add(cli),
        cli::ConfigAction::Edit { host_alias } => cmd_edit(Some(&host_alias), cli),
        cli::ConfigAction::Remove { host_alias } => cmd_remove(Some(&host_alias), cli),
    }
}
