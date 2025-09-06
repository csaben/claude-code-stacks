use std::path::{Path, PathBuf};
use std::process::Command;
use anyhow::{Result, Context, bail};
use serde::{Deserialize, Serialize};
use dirs;

use super::stack_manager::Stack;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubFile {
    pub name: String,
    pub path: String,
    #[serde(rename = "type")]
    pub file_type: String,
}

#[derive(Debug, Clone)]
pub struct StackRepository {
    pub owner: String,
    pub repo: String,
    pub branch: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StackMetadata {
    pub source_repo: String,
    pub source_owner: String,
    pub source_name: String,
    pub source_branch: String,
    pub stack_name: String,
    pub original_path: String,
}

impl Default for StackRepository {
    fn default() -> Self {
        Self {
            owner: "csaben".to_string(),
            repo: "claude-code-stacks".to_string(),
            branch: "main".to_string(),
        }
    }
}

pub struct RemoteStackManager {
    pub repository: StackRepository,
    #[allow(dead_code)]
    cache_dir: PathBuf,
    client: reqwest::Client,
}

impl RemoteStackManager {
    pub fn new() -> Result<Self> {
        let cache_dir = dirs::cache_dir()
            .context("Failed to get cache directory")?
            .join("claude-stacks");
        
        std::fs::create_dir_all(&cache_dir)
            .context("Failed to create cache directory")?;

        Ok(Self {
            repository: StackRepository::default(),
            cache_dir,
            client: reqwest::Client::new(),
        })
    }

    #[allow(dead_code)]
    pub fn with_repository(repository: StackRepository) -> Result<Self> {
        let mut manager = Self::new()?;
        manager.repository = repository;
        Ok(manager)
    }

    /// Discover available stacks from the GitHub repository
    pub async fn discover_remote_stacks(&self) -> Result<Vec<Stack>> {
        let url = format!(
            "https://api.github.com/repos/{}/{}/contents/stacks?ref={}",
            self.repository.owner, self.repository.repo, self.repository.branch
        );

        let response = self.client
            .get(&url)
            .header("User-Agent", "claude-stacks-cli")
            .send()
            .await
            .context("Failed to fetch stacks from GitHub API")?;

        if !response.status().is_success() {
            bail!("GitHub API request failed with status: {}", response.status());
        }

        let files: Vec<GitHubFile> = response
            .json()
            .await
            .context("Failed to parse GitHub API response")?;

        let mut stacks = Vec::new();

        for file in files {
            if file.file_type == "dir" {
                let stack_name = file.name.clone();
                let local_path = std::env::current_dir()?.join("stacks").join(&stack_name);
                
                // Create a stack object for the remote stack
                let mut stack = Stack::new(stack_name, local_path);
                
                // Always fetch description from remote CLAUDE.md (don't rely on local cache)
                if let Some(description) = self.fetch_stack_description(&file.name).await? {
                    stack.description = Some(description);
                }
                
                stacks.push(stack);
            }
        }

        if stacks.is_empty() {
            bail!("No stacks found in repository {}/{}", self.repository.owner, self.repository.repo);
        }

        stacks.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(stacks)
    }

    /// Fetch the description from a stack's CLAUDE.md file
    async fn fetch_stack_description(&self, stack_name: &str) -> Result<Option<String>> {
        let url = format!(
            "https://raw.githubusercontent.com/{}/{}/{}/stacks/{}/CLAUDE.md",
            self.repository.owner, self.repository.repo, self.repository.branch, stack_name
        );

        let response = self.client
            .get(&url)
            .header("User-Agent", "claude-stacks-cli")
            .send()
            .await;

        match response {
            Ok(resp) if resp.status().is_success() => {
                let content = resp.text().await?;
                
                // Extract description from CLAUDE.md
                for line in content.lines() {
                    if line.starts_with("# Description:") {
                        return Ok(Some(line.trim_start_matches("# Description:").trim().to_string()));
                    }
                }
                Ok(None)
            }
            _ => Ok(None), // Ignore errors for description fetching
        }
    }

    /// Add a stack as a git subtree
    pub async fn add_stack_subtree(&self, stack_name: &str) -> Result<PathBuf> {
        let stack_path = std::env::current_dir()?.join("stacks").join(stack_name);
        
        // Check if already exists
        if stack_path.exists() {
            println!("  📦 Stack already exists: {}", stack_name);
            return Ok(stack_path);
        }
        
        // For existing stacks like ts-lint-stack, use the specific repository
        let repo_url = if stack_name == "ts-lint-stack" {
            "git@github.com:csaben/ts-lint-stack.git".to_string()
        } else {
            // For other stacks, assume they're in separate repositories following the pattern
            format!("git@github.com:{}/{}.git", self.repository.owner, stack_name)
        };
        
        println!("  📥 Adding {} as subtree from {}", stack_name, repo_url);
        
        // Add as git subtree
        let subtree_output = Command::new("git")
            .args([
                "subtree", "add", 
                "--prefix", &format!("stacks/{}", stack_name),
                &repo_url,
                "main",
                "--squash"
            ])
            .output()
            .context("Failed to execute git subtree add")?;
            
        if !subtree_output.status.success() {
            let error = String::from_utf8_lossy(&subtree_output.stderr);
            bail!("Git subtree add failed: {}", error);
        }
        
        println!("  ✅ Successfully added {} as subtree", stack_name);
        Ok(stack_path)
    }

    /// Download and cache a stack from the remote repository (deprecated - use add_stack_subtree)
    #[allow(dead_code)]
    pub async fn cache_stack(&self, stack_name: &str) -> Result<PathBuf> {
        // Check out to current working directory instead of cache
        let stack_path = std::env::current_dir()?.join("stacks").join(stack_name);
        
        // Check if already cached and valid
        if stack_path.exists() {
            let stack = Stack::new(stack_name.to_string(), stack_path.clone());
            if stack.is_valid() {
                println!("  📦 Using existing stack: {}", stack_name);
                return Ok(stack_path);
            }
            
            // Remove invalid stack
            std::fs::remove_dir_all(&stack_path)
                .context("Failed to remove invalid existing stack")?;
        }

        // Ensure stacks directory exists
        let stacks_dir = std::env::current_dir()?.join("stacks");
        std::fs::create_dir_all(&stacks_dir)
            .context("Failed to create stacks directory")?;

        println!("  ⬇️ Checking out stack: {}", stack_name);
        
        // Use sparse checkout to get only the specific stack
        self.git_clone_stack(stack_name).await
            .with_context(|| format!("Failed to checkout stack: {}", stack_name))?;

        Ok(stack_path)
    }

    /// Clone the repository and extract just the stack directory content
    async fn git_clone_stack(&self, stack_name: &str) -> Result<()> {
        let ssh_url = format!("git@github.com:{}/{}.git", self.repository.owner, self.repository.repo);
        let temp_path = std::env::current_dir()?.join(format!("temp-{}", stack_name));
        let final_stack_path = std::env::current_dir()?.join("stacks").join(stack_name);
        
        // Clean up temp path if it exists
        if temp_path.exists() {
            std::fs::remove_dir_all(&temp_path)?;
        }
        
        // Clone the full repository to a temporary location
        println!("  📦 Cloning repository...");
        let clone_output = Command::new("git")
            .args([
                "clone",
                &ssh_url,
                temp_path.to_str().unwrap(),
            ])
            .output()
            .context("Failed to execute git clone")?;

        if !clone_output.status.success() {
            bail!("Git clone failed: {}", String::from_utf8_lossy(&clone_output.stderr));
        }

        // Copy just the stack directory content to final location
        let source_stack_path = temp_path.join("stacks").join(stack_name);
        if !source_stack_path.exists() {
            bail!("Stack '{}' not found in repository", stack_name);
        }
        
        println!("  📁 Extracting stack content...");
        std::fs::create_dir_all(&final_stack_path)?;
        self.copy_dir_all(&source_stack_path, &final_stack_path)?;
        
        // Initialize git repository in the stack directory
        let git_init_output = Command::new("git")
            .current_dir(&final_stack_path)
            .args(["init"])
            .output()
            .context("Failed to initialize git repository")?;
            
        if !git_init_output.status.success() {
            bail!("Git init failed: {}", String::from_utf8_lossy(&git_init_output.stderr));
        }
        
        // Add the remote origin
        let remote_output = Command::new("git")
            .current_dir(&final_stack_path)
            .args(["remote", "add", "origin", &ssh_url])
            .output()
            .context("Failed to add remote origin")?;
            
        if !remote_output.status.success() {
            bail!("Failed to add remote: {}", String::from_utf8_lossy(&remote_output.stderr));
        }
        
        // Fetch from origin
        let fetch_output = Command::new("git")
            .current_dir(&final_stack_path)
            .args(["fetch", "origin"])
            .output()
            .context("Failed to fetch from origin")?;
            
        if !fetch_output.status.success() {
            bail!("Failed to fetch: {}", String::from_utf8_lossy(&fetch_output.stderr));
        }
        
        // Set up tracking branch
        let branch_output = Command::new("git")
            .current_dir(&final_stack_path)
            .args(["checkout", "-b", &self.repository.branch, &format!("origin/{}", self.repository.branch)])
            .output()
            .context("Failed to checkout branch")?;
            
        if !branch_output.status.success() {
            bail!("Failed to checkout branch: {}", String::from_utf8_lossy(&branch_output.stderr));
        }

        // Clean up temporary directory
        std::fs::remove_dir_all(&temp_path)?;

        // Create metadata file
        let metadata = StackMetadata {
            source_repo: ssh_url.clone(),
            source_owner: self.repository.owner.clone(),
            source_name: self.repository.repo.clone(),
            source_branch: self.repository.branch.clone(),
            stack_name: stack_name.to_string(),
            original_path: format!("stacks/{}", stack_name),
        };

        self.save_stack_metadata(&final_stack_path, &metadata)?;
        println!("  📋 Stack initialized as independent git repository");

        Ok(())
    }

    /// Ensure we're in a git repository, initialize if needed
    #[allow(dead_code)]
    async fn ensure_git_repository(&self) -> Result<()> {
        let git_dir = std::env::current_dir()?.join(".git");
        
        if !git_dir.exists() {
            println!("  🎯 Initializing git repository...");
            let init_output = Command::new("git")
                .args(["init"])
                .output()
                .context("Failed to initialize git repository")?;
                
            if !init_output.status.success() {
                bail!("Git init failed: {}", String::from_utf8_lossy(&init_output.stderr));
            }
            
            // Set up initial commit if no commits exist
            let log_output = Command::new("git")
                .args(["log", "--oneline", "-1"])
                .output();
                
            if log_output.is_err() || !log_output.unwrap().status.success() {
                // Create initial commit
                println!("  📝 Creating initial commit...");
                
                // Create a README if it doesn't exist
                let readme_path = std::env::current_dir()?.join("README.md");
                if !readme_path.exists() {
                    std::fs::write(readme_path, "# Project with Claude Code Stacks\n\nThis project uses stacks for Claude Code workflows.\n")?;
                }
                
                let add_output = Command::new("git")
                    .args(["add", "."])
                    .output()
                    .context("Failed to add files")?;
                    
                if !add_output.status.success() {
                    bail!("Git add failed: {}", String::from_utf8_lossy(&add_output.stderr));
                }
                
                let commit_output = Command::new("git")
                    .args(["commit", "-m", "feat: initial commit with stacks setup"])
                    .output()
                    .context("Failed to create initial commit")?;
                    
                if !commit_output.status.success() {
                    bail!("Git commit failed: {}", String::from_utf8_lossy(&commit_output.stderr));
                }
            }
        }
        
        Ok(())
    }

    /// Copy a directory recursively
    fn copy_dir_all(&self, src: &std::path::Path, dst: &std::path::Path) -> Result<()> {
        Self::copy_dir_all_static(src, dst)
    }

    /// Static helper for copying directories recursively  
    fn copy_dir_all_static(src: &std::path::Path, dst: &std::path::Path) -> Result<()> {
        std::fs::create_dir_all(dst)?;
        for entry in std::fs::read_dir(src)? {
            let entry = entry?;
            let ty = entry.file_type()?;
            if ty.is_dir() {
                Self::copy_dir_all_static(&entry.path(), &dst.join(entry.file_name()))?;
            } else {
                std::fs::copy(entry.path(), dst.join(entry.file_name()))?;
            }
        }
        Ok(())
    }

    /// Save metadata about the stack's source repository
    fn save_stack_metadata(&self, stack_path: &Path, metadata: &StackMetadata) -> Result<()> {
        let metadata_file = stack_path.join(".stack-metadata.json");
        let metadata_json = serde_json::to_string_pretty(metadata)
            .context("Failed to serialize stack metadata")?;
        
        std::fs::write(metadata_file, metadata_json)
            .context("Failed to write stack metadata file")?;
        
        Ok(())
    }

    /// Get the cache directory path
    #[allow(dead_code)]
    pub fn cache_dir(&self) -> &Path {
        &self.cache_dir
    }

    /// Clear the entire stack cache
    #[allow(dead_code)]
    pub fn clear_cache(&self) -> Result<()> {
        if self.cache_dir.exists() {
            std::fs::remove_dir_all(&self.cache_dir)
                .context("Failed to clear stack cache")?;
            std::fs::create_dir_all(&self.cache_dir)
                .context("Failed to recreate cache directory")?;
        }
        Ok(())
    }

    /// Update cached stack (re-download)
    #[allow(dead_code)]
    pub async fn update_stack(&self, stack_name: &str) -> Result<PathBuf> {
        let stack_path = self.cache_dir.join(stack_name);
        
        // Remove existing cache
        if stack_path.exists() {
            std::fs::remove_dir_all(&stack_path)
                .context("Failed to remove existing stack cache")?;
        }
        
        // Re-download
        self.cache_stack(stack_name).await
    }
}

/// Fallback to local stacks directory for development/testing
#[allow(dead_code)]
pub async fn discover_local_stacks() -> Result<Vec<Stack>> {
    use super::stack_manager::discover_stacks;
    discover_stacks().await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_remote_stack_discovery() {
        let manager = RemoteStackManager::new().unwrap();
        
        // This test requires internet access and the actual repository
        // In a real scenario, you'd mock the HTTP client
        match manager.discover_remote_stacks().await {
            Ok(stacks) => {
                assert!(!stacks.is_empty());
                println!("Found {} stacks", stacks.len());
                for stack in stacks {
                    println!("  - {}: {:?}", stack.name, stack.description);
                }
            }
            Err(e) => {
                println!("Failed to discover remote stacks (expected in CI): {}", e);
            }
        }
    }
}