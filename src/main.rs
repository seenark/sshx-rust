mod cli;
mod clipboard;
mod commands;
mod config;
mod error;
mod index;
mod model;
mod parser;
mod selector;
mod ssh_command;
mod tunnel;

use clap::Parser;

fn main() {
    let cli = cli::Cli::parse();

    if cli.verbose {
        eprintln!("Verbose mode enabled");
    }

    let result = match cli.command {
        None => commands::connect(cli.host_alias.as_deref(), &cli),
        Some(cli::Commands::Connect { ref host_alias }) => {
            commands::connect(host_alias.as_deref().or(cli.host_alias.as_deref()), &cli)
        }
        Some(cli::Commands::Config { ref action }) => commands::config(action.clone(), &cli),
        Some(cli::Commands::Version) => {
            println!("sshx {}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
    };

    if let Err(e) = result {
        eprintln!("{}", e.display_full());
        std::process::exit(1);
    }
}
