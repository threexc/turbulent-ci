use crate::config::ProjectType;
use std::fs;
use std::path::Path;

pub struct ProjectDetector;

impl ProjectDetector {
    pub fn new() -> Self {
        Self
    }
    
    pub fn detect_project_type(&self, path: &str) -> ProjectType {
        let project_path = Path::new(path);
        
        // Check for Rust project
        if project_path.join("Cargo.toml").exists() {
            return ProjectType::Rust;
        }
        
        // Check for Python project
        if self.has_python_indicators(project_path) {
            return ProjectType::Python;
        }
        
        // Check for Node.js project
        if project_path.join("package.json").exists() {
            return ProjectType::Node;
        }
        
        ProjectType::Generic
    }
    
    fn has_python_indicators(&self, path: &Path) -> bool {
        // Check for common Python project files
        let python_files = [
            "requirements.txt",
            "setup.py",
            "pyproject.toml",
            "Pipfile",
            "poetry.lock",
            "pytest.ini",
            "tox.ini",
        ];
        
        for file in &python_files {
            if path.join(file).exists() {
                return true;
            }
        }
        
        // Check for Python source files
        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.flatten() {
                if let Some(ext) = entry.path().extension() {
                    if ext == "py" {
                        return true;
                    }
                }
            }
        }
        
        false
    }
}
