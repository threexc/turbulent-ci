use crate::project_detector::ProjectDetector;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Duration;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Config {
    pub web_port: u16,
    pub config_file: String,
    #[allow(dead_code)]
    pub poll_interval: Duration,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ProjectType {
    Rust,
    Python,
    Node,
    Generic,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repository {
    pub id: Uuid,
    pub name: String,
    pub path: String,
    pub project_type: ProjectType,
    pub commands: Vec<String>,
    pub enabled: bool,
}

impl Config {
    pub fn new(port: u16, config_file: Option<String>) -> Self {
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("turbulent-ci");
        
        std::fs::create_dir_all(&config_dir).ok();
        
        let config_file = config_file.unwrap_or_else(|| {
            config_dir.join("repositories.json").to_string_lossy().to_string()
        });
        
        Self {
            web_port: port,
            config_file,
            poll_interval: Duration::from_secs(30),
        }
    }
    
    pub fn default() -> Self {
        Self::new(3030, None)
    }
}

impl Repository {
    pub fn new(path: String, name: Option<String>) -> Result<Self, Box<dyn std::error::Error>> {
        let detector = ProjectDetector::new();
        let project_type = detector.detect_project_type(&path);
        
        // Validate path exists
        if !std::path::Path::new(&path).exists() {
            return Err(format!("Path does not exist: {}", path).into());
        }
        
        let repo_name = name.unwrap_or_else(|| {
            std::path::Path::new(&path)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string()
        });
        
        let commands = Self::get_default_commands(&project_type);
        
        Ok(Self {
            id: Uuid::new_v4(),
            name: repo_name,
            path,
            project_type,
            commands,
            enabled: true,
        })
    }
    
    fn get_default_commands(project_type: &ProjectType) -> Vec<String> {
        match project_type {
            ProjectType::Rust => vec![
                "cargo check".to_string(),
                "cargo test".to_string(),
                "cargo clippy -- -D warnings".to_string(),
            ],
            ProjectType::Python => vec![
                "python -m py_compile $(find . -name '*.py' | head -10)".to_string(),
                "python -m pytest".to_string(),
                "python -m flake8 --max-line-length=88".to_string(),
            ],
            ProjectType::Node => vec![
                "npm ci".to_string(),
                "npm test".to_string(),
                "npm run lint".to_string(),
            ],
            ProjectType::Generic => vec![
                "echo 'Generic project - no default commands'".to_string(),
            ],
        }
    }
}
