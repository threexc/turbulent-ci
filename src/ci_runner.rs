use crate::config::Repository;
use crate::models::{BuildResult, GlobalState};
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub type SharedGlobalState = Arc<Mutex<GlobalState>>;

pub struct CiRunner {
    repository: Repository,
    last_commit: Option<String>,
    global_state: SharedGlobalState,
    build_counter: u64,
}

impl CiRunner {
    pub fn new(repository: Repository, global_state: SharedGlobalState) -> Self {
        // Initialize repository state
        {
            let mut state = global_state.lock().unwrap();
            state.add_repository_state(repository.clone());
        }
        
        Self {
            repository,
            last_commit: None,
            global_state,
            build_counter: 0,
        }
    }

    fn get_latest_commit(&self) -> Result<String, Box<dyn std::error::Error>> {
        let output = Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(&self.repository.path)
            .output()?;

        if !output.status.success() {
            return Err("Failed to get git commit".into());
        }

        Ok(String::from_utf8(output.stdout)?.trim().to_string())
    }

    fn get_current_branch(&self) -> Result<String, Box<dyn std::error::Error>> {
        let output = Command::new("git")
            .args(["branch", "--show-current"])
            .current_dir(&self.repository.path)
            .output()?;

        if !output.status.success() {
            return Err("Failed to get git branch".into());
        }

        Ok(String::from_utf8(output.stdout)?.trim().to_string())
    }

    fn run_commands(&self, commit_hash: &str) -> BuildResult {
        let start_time = SystemTime::now();
        let mut all_output = String::new();
        let mut success = true;

        println!("[{}] 🔨 Starting {} build for commit {}...", 
                 self.repository.name,
                 format!("{:?}", self.repository.project_type).to_lowercase(),
                 &commit_hash[..8]);

        // Update status
        {
            let mut state = self.global_state.lock().unwrap();
            state.update_repository_status(&self.repository.id, "Building...".to_string());
        }

        for cmd in &self.repository.commands {
            println!("[{}] Running: {}", self.repository.name, cmd);
            
            let result = self.execute_command(cmd);
            
            match result {
                Ok((stdout, stderr, cmd_success)) => {
                    all_output.push_str(&format!("=== {} ===\n", cmd));
                    all_output.push_str(&stdout);
                    if !stderr.is_empty() {
                        all_output.push_str("STDERR:\n");
                        all_output.push_str(&stderr);
                    }
                    all_output.push('\n');

                    if !cmd_success {
                        success = false;
                        println!("[{}] ❌ Command failed: {}", self.repository.name, cmd);
                        break;
                    } else {
                        println!("[{}] ✅ Command succeeded: {}", self.repository.name, cmd);
                    }
                }
                Err(e) => {
                    success = false;
                    all_output.push_str(&format!("Failed to execute {}: {}\n", cmd, e));
                    println!("[{}] ❌ Failed to execute: {}", self.repository.name, cmd);
                    break;
                }
            }
        }

        let duration = start_time.elapsed().unwrap_or(Duration::from_secs(0));
        
        BuildResult {
            id: self.build_counter,
            repository_id: self.repository.id,
            repository_name: self.repository.name.clone(),
            success,
            output: all_output,
            timestamp: start_time.duration_since(UNIX_EPOCH).unwrap().as_secs(),
            commit_hash: commit_hash.to_string(),
            duration_ms: duration.as_millis() as u64,
            repo_path: self.repository.path.clone(),
            project_type: format!("{:?}", self.repository.project_type),
        }
    }
    
    fn execute_command(&self, cmd: &str) -> Result<(String, String, bool), Box<dyn std::error::Error>> {
        let output = if cfg!(target_os = "windows") {
            Command::new("cmd")
                .args(["/C", cmd])
                .current_dir(&self.repository.path)
                .output()?
        } else {
            Command::new("sh")
                .args(["-c", cmd])
                .current_dir(&self.repository.path)
                .output()?
        };
        
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let success = output.status.success();
        
        Ok((stdout, stderr, success))
    }

    fn check_and_build(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let current_commit = self.get_latest_commit()?;
        
        if let Some(ref last) = self.last_commit {
            if last == &current_commit {
                return Ok(()); // No changes
            }
        }

        println!("[{}] 📝 New commit detected: {}", self.repository.name, &current_commit[..8]);
        
        self.build_counter += 1;
        let result = self.run_commands(&current_commit);
        
        if result.success {
            println!("[{}] 🎉 Build successful!", self.repository.name);
        } else {
            println!("[{}] 💥 Build failed!", self.repository.name);
        }

        // Update state
        {
            let mut state = self.global_state.lock().unwrap();
            state.add_build(result.clone());
            
            let status = if result.success {
                "Passing".to_string()
            } else {
                "Failed".to_string()
            };
            state.update_repository_status(&self.repository.id, status);
            
            if let Ok(branch) = self.get_current_branch() {
                state.update_repository_info(&self.repository.id, branch, current_commit.clone());
            }
        }

        self.last_commit = Some(current_commit);
        Ok(())
    }

    pub fn run(&mut self) {
        println!("[{}] 🌪️  Turbulent CI Runner started", self.repository.name);
        println!("[{}] 📁 Monitoring: {}", self.repository.name, self.repository.path);
        println!("[{}] 🔧 Project type: {:?}", self.repository.name, self.repository.project_type);
        
        // Initialize status
        {
            let mut state = self.global_state.lock().unwrap();
            state.update_repository_status(&self.repository.id, "Idle".to_string());
        }
        
        loop {
            match self.check_and_build() {
                Ok(_) => {
                    let mut state = self.global_state.lock().unwrap();
                    if let Some(repo_state) = state.repositories.get(&self.repository.id) {
                        if repo_state.current_status == "Building..." {
                            state.update_repository_status(&self.repository.id, "Idle".to_string());
                        }
                    }
                },
                Err(e) => {
                    println!("[{}] Error: {}", self.repository.name, e);
                    let mut state = self.global_state.lock().unwrap();
                    state.update_repository_status(&self.repository.id, format!("Error: {}", e));
                }
            }
            
            thread::sleep(Duration::from_secs(30));
        }
    }
}
