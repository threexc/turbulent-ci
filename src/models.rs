use crate::config::{Repository};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildResult {
    pub id: u64,
    pub repository_id: Uuid,
    pub repository_name: String,
    pub success: bool,
    pub output: String,
    pub timestamp: u64,
    pub commit_hash: String,
    pub duration_ms: u64,
    pub repo_path: String,
    pub project_type: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct GlobalState {
    pub repositories: HashMap<Uuid, RepositoryState>,
    pub recent_builds: Vec<BuildResult>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RepositoryState {
    pub repository: Repository,
    pub builds: Vec<BuildResult>,
    pub current_status: String,
    pub repo_info: RepoInfo,
}

#[derive(Debug, Clone, Serialize)]
pub struct RepoInfo {
    pub path: String,
    pub branch: String,
    pub last_commit: String,
    pub commands: Vec<String>,
    pub project_type: String,
}

impl GlobalState {
    pub fn new() -> Self {
        Self {
            repositories: HashMap::new(),
            recent_builds: Vec::new(),
        }
    }
    
    pub fn add_repository_state(&mut self, repository: Repository) {
        let repo_info = RepoInfo {
            path: repository.path.clone(),
            branch: "unknown".to_string(),
            last_commit: "unknown".to_string(),
            commands: repository.commands.clone(),
            project_type: format!("{:?}", repository.project_type),
        };
        
        let state = RepositoryState {
            repository: repository.clone(),
            builds: Vec::new(),
            current_status: "Starting...".to_string(),
            repo_info,
        };
        
        self.repositories.insert(repository.id, state);
    }
    
    pub fn add_build(&mut self, build: BuildResult) {
        // Add to repository-specific builds
        if let Some(repo_state) = self.repositories.get_mut(&build.repository_id) {
            repo_state.builds.insert(0, build.clone());
            
            // Keep only last 50 builds per repository
            if repo_state.builds.len() > 50 {
                repo_state.builds.truncate(50);
            }
        }
        
        // Add to global recent builds
        self.recent_builds.insert(0, build);
        
        // Keep only last 100 recent builds globally
        if self.recent_builds.len() > 100 {
            self.recent_builds.truncate(100);
        }
    }
    
    pub fn update_repository_status(&mut self, repo_id: &Uuid, status: String) {
        if let Some(repo_state) = self.repositories.get_mut(repo_id) {
            repo_state.current_status = status;
        }
    }
    
    pub fn update_repository_info(&mut self, repo_id: &Uuid, branch: String, commit: String) {
        if let Some(repo_state) = self.repositories.get_mut(repo_id) {
            repo_state.repo_info.branch = branch;
            repo_state.repo_info.last_commit = commit;
        }
    }
}

impl RepositoryState {
    #[allow(dead_code)]
    pub fn new(repository: Repository) -> Self {
        Self {
            repo_info: RepoInfo {
                path: repository.path.clone(),
                branch: "unknown".to_string(),
                last_commit: "unknown".to_string(),
                commands: repository.commands.clone(),
                project_type: format!("{:?}", repository.project_type),
            },
            repository,
            builds: Vec::new(),
            current_status: "Starting...".to_string(),
        }
    }
}
