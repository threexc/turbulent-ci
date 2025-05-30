use crate::config::{Config, Repository};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct RepositoryManager {
    repositories: HashMap<Uuid, Repository>,
}

impl RepositoryManager {
    pub fn new() -> Self {
        Self {
            repositories: HashMap::new(),
        }
    }
    
    pub fn load(config: &Config) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(&config.config_file)?;
        let manager: RepositoryManager = serde_json::from_str(&content)?;
        Ok(manager)
    }
    
    pub fn save(&self, config: &Config) -> Result<(), Box<dyn std::error::Error>> {
        let content = serde_json::to_string_pretty(self)?;
        fs::write(&config.config_file, content)?;
        Ok(())
    }
    
    pub fn add_repository(&mut self, path: String, name: Option<String>) -> Result<Repository, Box<dyn std::error::Error>> {
        // Check if repository with same path already exists
        for repo in self.repositories.values() {
            if repo.path == path {
                return Err(format!("Repository with path '{}' already exists", path).into());
            }
        }
        
        let repo = Repository::new(path, name)?;
        let repo_clone = repo.clone();
        self.repositories.insert(repo.id, repo);
        
        Ok(repo_clone)
    }
    
    pub fn remove_repository(&mut self, name: &str) -> bool {
        let repo_id = self.repositories
            .iter()
            .find(|(_, repo)| repo.name == name)
            .map(|(id, _)| *id);
        
        if let Some(id) = repo_id {
            self.repositories.remove(&id);
            true
        } else {
            false
        }
    }
    
    pub fn get_repositories(&self) -> Vec<Repository> {
        self.repositories.values().cloned().collect()
    }
}
