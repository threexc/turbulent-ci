mod config;
mod models;
mod ci_runner;
mod web_server;
mod project_detector;
mod repository_manager;
mod cli;

use config::Config;
use models::GlobalState;
use ci_runner::CiRunner;
use web_server::WebServer;
use repository_manager::RepositoryManager;
use cli::{Cli, Commands};
use clap::Parser;
use std::sync::{Arc, Mutex};
use std::thread;
use std::process;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Start { port, config_file } => {
            start_daemon(port, config_file).await;
        }
        Commands::Add { path, name } => {
            add_repository(path, name).await;
        }
        Commands::Remove { name } => {
            remove_repository(name).await;
        }
        Commands::List => {
            list_repositories().await;
        }
        Commands::Status => {
            show_status().await;
        }
    }
}

async fn start_daemon(port: Option<u16>, config_file: Option<String>) {
    let config = Config::new(port.unwrap_or(3030), config_file);
    let repo_manager = RepositoryManager::load(&config).unwrap_or_else(|e| {
        println!("Warning: Failed to load repositories: {}", e);
        RepositoryManager::new()
    });
    
    println!("🌪️  Turbulent CI Multi-Repository Daemon");
    println!("📁 Config file: {}", config.config_file);
    println!("🌐 Web interface: http://localhost:{}", config.web_port);
    
    let global_state = Arc::new(Mutex::new(GlobalState::new()));
    let global_state_clone = Arc::clone(&global_state);
    
    // Start CI runners for each repository
    let repositories = repo_manager.get_repositories().clone();
    for repo in repositories {
        let repo_clone = repo.clone();
        let state_clone = Arc::clone(&global_state);
        
        thread::spawn(move || {
            let mut runner = CiRunner::new(repo_clone, state_clone);
            runner.run();
        });
    }
    
    // Start web server
    let web_server = WebServer::new(global_state_clone, config.web_port);
    web_server.start().await;
}

async fn add_repository(path: String, name: Option<String>) {
    let config = Config::default();
    let mut repo_manager = RepositoryManager::load(&config).unwrap_or_else(|_| RepositoryManager::new());
    
    match repo_manager.add_repository(path, name) {
        Ok(repo) => {
            if let Err(e) = repo_manager.save(&config) {
                eprintln!("Failed to save configuration: {}", e);
                process::exit(1);
            }
            println!("✅ Added repository: {} ({})", repo.name, repo.path);
            println!("💡 Restart the daemon to begin monitoring this repository");
        }
        Err(e) => {
            eprintln!("❌ Failed to add repository: {}", e);
            process::exit(1);
        }
    }
}

async fn remove_repository(name: String) {
    let config = Config::default();
    let mut repo_manager = RepositoryManager::load(&config).unwrap_or_else(|_| RepositoryManager::new());
    
    if repo_manager.remove_repository(&name) {
        if let Err(e) = repo_manager.save(&config) {
            eprintln!("Failed to save configuration: {}", e);
            process::exit(1);
        }
        println!("✅ Removed repository: {}", name);
        println!("💡 Restart the daemon to stop monitoring this repository");
    } else {
        eprintln!("❌ Repository '{}' not found", name);
        process::exit(1);
    }
}

async fn list_repositories() {
    let config = Config::default();
    let repo_manager = RepositoryManager::load(&config).unwrap_or_else(|_| RepositoryManager::new());
    
    let repositories = repo_manager.get_repositories();
    if repositories.is_empty() {
        println!("No repositories configured");
        return;
    }
    
    println!("📋 Configured repositories:");
    for repo in repositories {
        println!("  • {} - {} ({:?})", repo.name, repo.path, repo.project_type);
    }
}

async fn show_status() {
    match reqwest::get("http://localhost:3030/api/status").await {
        Ok(response) => {
            if response.status().is_success() {
                println!("✅ Turbulent CI daemon is running");
            } else {
                println!("❌ Daemon responded with error: {}", response.status());
            }
        }
        Err(_) => {
            println!("❌ Turbulent CI daemon is not running or not accessible");
        }
    }
}
