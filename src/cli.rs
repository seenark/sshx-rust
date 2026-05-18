use std::path::PathBuf;

#[derive(Debug, clap::Parser)]
#[command(name = "sshx", version, about = "Enhanced SSH connection manager")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    #[arg(help = "Host alias or name to connect to")]
    pub host_alias: Option<String>,

    #[arg(long, global = true, help = "Override SSH config file path")]
    pub config: Option<PathBuf>,

    #[arg(
        long,
        global = true,
        help = "Print command instead of copying to clipboard"
    )]
    pub no_clipboard: bool,

    #[arg(long, global = true, help = "Show what would happen without executing")]
    pub dry_run: bool,

    #[arg(long, short, global = true, help = "Debug output")]
    pub verbose: bool,
}

#[derive(Debug, clap::Subcommand)]
pub enum Commands {
    Connect {
        #[arg(help = "Host alias or name")]
        host_alias: Option<String>,
    },
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
    Version,
}

#[derive(Debug, Clone, clap::Subcommand)]
pub enum ConfigAction {
    Add,
    Edit {
        #[arg(help = "Host alias or name to edit")]
        host_alias: String,
    },
    Remove {
        #[arg(help = "Host alias or name to remove")]
        host_alias: String,
    },
    List {
        #[arg(long, help = "Filter by group")]
        group: Option<String>,
    },
    Validate,
    Show {
        #[arg(help = "Host alias or name to show")]
        host_alias: String,
    },
    Init,
}
