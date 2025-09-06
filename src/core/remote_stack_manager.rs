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
                
                // Check if we have this stack cached
                if stack.claude_dir.exists() {
                    if stack.is_valid() {
                        stack.load_description().await?;
                        stacks.push(stack);
                        continue;
                    }
                }

                // Fetch description from remote CLAUDE.md
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

    /// Download and cache a stack from the remote repository
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

        println!("  ⬇️ Downloading stack: {}", stack_name);
        
        // Use git sparse-checkout for efficient downloading
        self.git_sparse_checkout(stack_name).await
            .with_context(|| format!("Failed to download stack: {}", stack_name))?;

        Ok(stack_path)
    }

    /// Use git sparse-checkout to download only the specific stack
    async fn git_sparse_checkout(&self, stack_name: &str) -> Result<()> {
        let repo_url = format!("https://github.com/{}/{}.git", self.repository.owner, self.repository.repo);
        let stack_path = std::env::current_dir()?.join("stacks").join(stack_name);
        let temp_repo_path = std::env::current_dir()?.join("stacks").join(format!("{}-temp", stack_name));

        // Clean up any existing temp directory
        if temp_repo_path.exists() {
            std::fs::remove_dir_all(&temp_repo_path)?;
        }

        // Clone with sparse checkout
        let clone_output = Command::new("git")
            .args(&[
                "clone",
                "--filter=blob:none",
                "--sparse",
                &repo_url,
                temp_repo_path.to_str().unwrap(),
            ])
            .output()
            .context("Failed to execute git clone")?;

        if !clone_output.status.success() {
            bail!("Git clone failed: {}", String::from_utf8_lossy(&clone_output.stderr));
        }

        // Set sparse checkout to only include the specific stack
        let sparse_output = Command::new("git")
            .current_dir(&temp_repo_path)
            .args(&["sparse-checkout", "set", &format!("stacks/{}", stack_name)])
            .output()
            .context("Failed to set sparse checkout")?;

        if !sparse_output.status.success() {
            bail!("Git sparse-checkout failed: {}", String::from_utf8_lossy(&sparse_output.stderr));
        }

        // Move the stack to its final location
        let source_stack = temp_repo_path.join("stacks").join(stack_name);
        if source_stack.exists() {
            std::fs::rename(&source_stack, &stack_path)
                .context("Failed to move downloaded stack")?;
        } else {
            bail!("Stack {} not found in repository", stack_name);
        }

        // Cleanup temp repository
        std::fs::remove_dir_all(&temp_repo_path)
            .context("Failed to cleanup temporary repository")?;

        Ok(())
    }

    /// Get the cache directory path
    pub fn cache_dir(&self) -> &Path {
        &self.cache_dir
    }

    /// Clear the entire stack cache
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