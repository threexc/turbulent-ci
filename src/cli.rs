use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "turbulent-ci")]
#[command(about = "A turbulent CI system for multiple repositories")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Start the CI daemon
    Start {
        /// Web interface port
        #[arg(short, long, default_value = "3030")]
        port: Option<u16>,
        /// Configuration file path
        #[arg(short, long)]
        config_file: Option<String>,
    },
    /// Add a repository to monitor
    Add {
        /// Repository path
        path: String,
        /// Repository name (optional)
        #[arg(short, long)]
        name: Option<String>,
    },
    /// Remove a repository from monitoring
    Remove {
        /// Repository name
        name: String,
    },
    /// List all configured repositories
    List,
    /// Check daemon status
    Status,
}
