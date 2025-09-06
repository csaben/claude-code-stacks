use std::path::PathBuf;
use anyhow::{Result, Context};

pub struct ClaudeMdUpdater {
    claude_md_path: PathBuf,
}

impl ClaudeMdUpdater {
    pub fn new() -> Self {
        Self {
            claude_md_path: PathBuf::from("CLAUDE.md"),
        }
    }

    /// Add an import statement for a stack to CLAUDE.md
    pub async fn add_stack_import(&self, stack_name: &str) -> Result<()> {
        let import_line = format!("@stacks/{}/CLAUDE.md", stack_name);
        
        if self.claude_md_path.exists() {
            let content = tokio::fs::read_to_string(&self.claude_md_path)
                .await
                .with_context(|| format!("Failed to read {}", self.claude_md_path.display()))?;
            
            // Check if the import already exists
            if content.contains(&import_line) {
                return Ok(()); // Already imported
            }

            // Find the best place to insert the import
            let updated_content = self.insert_stack_import(&content, &import_line);
            
            tokio::fs::write(&self.claude_md_path, updated_content)
                .await
                .with_context(|| format!("Failed to write to {}", self.claude_md_path.display()))?;
        } else {
            // Create new CLAUDE.md with the import
            let content = format!("# Project Instructions\n\nSee {} for additional instructions.\n", import_line);
            tokio::fs::write(&self.claude_md_path, content)
                .await
                .with_context(|| format!("Failed to create {}", self.claude_md_path.display()))?;
        }

        println!("  📝 Added import to CLAUDE.md: {}", import_line);
        Ok(())
    }

    /// Insert the stack import in an appropriate location
    fn insert_stack_import(&self, content: &str, import_line: &str) -> String {
        let lines: Vec<&str> = content.lines().collect();
        let mut result_lines = Vec::new();
        let mut import_inserted = false;

        // Look for existing stack imports section or create one
        for (i, line) in lines.iter().enumerate() {
            result_lines.push(line.to_string());
            
            // If we find existing stack imports, insert after them
            if line.starts_with("@stacks/") && !import_inserted {
                // Find the end of the stack imports block
                let mut j = i + 1;
                while j < lines.len() && (lines[j].starts_with("@stacks/") || lines[j].trim().is_empty()) {
                    result_lines.push(lines[j].to_string());
                    j += 1;
                }
                
                // Insert our import
                result_lines.push(import_line.to_string());
                import_inserted = true;
                
                // Skip the lines we already added
                for k in (i + 1)..j {
                    if k < lines.len() {
                        // Already added above
                    }
                }
                continue;
            }
            
            // If we haven't found imports yet and we're at the end of the header section,
            // insert the import
            if !import_inserted && 
               (line.trim().is_empty() && 
                i > 0 && 
                !lines[i-1].trim().is_empty() && 
                !lines[i-1].starts_with("#")) {
                result_lines.push("".to_string()); // Empty line before imports
                result_lines.push(format!("See {} for additional stack instructions.", import_line));
                result_lines.push("".to_string()); // Empty line after imports
                import_inserted = true;
            }
        }

        // If we still haven't inserted it, add it at the end
        if !import_inserted {
            if !result_lines.is_empty() && !result_lines.last().unwrap().is_empty() {
                result_lines.push("".to_string());
            }
            result_lines.push("".to_string());
            result_lines.push(format!("See {} for additional stack instructions.", import_line));
        }

        result_lines.join("\n")
    }

    /// Remove a stack import from CLAUDE.md
    pub async fn remove_stack_import(&self, stack_name: &str) -> Result<()> {
        if !self.claude_md_path.exists() {
            return Ok(()); // Nothing to remove
        }

        let import_line = format!("@stacks/{}/CLAUDE.md", stack_name);
        let content = tokio::fs::read_to_string(&self.claude_md_path)
            .await
            .with_context(|| format!("Failed to read {}", self.claude_md_path.display()))?;

        let lines: Vec<&str> = content.lines().collect();
        let filtered_lines: Vec<String> = lines
            .iter()
            .filter(|line| !line.contains(&import_line))
            .map(|line| line.to_string())
            .collect();

        let updated_content = filtered_lines.join("\n");
        
        tokio::fs::write(&self.claude_md_path, updated_content)
            .await
            .with_context(|| format!("Failed to write to {}", self.claude_md_path.display()))?;

        println!("  📝 Removed import from CLAUDE.md: {}", import_line);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_stack_import() {
        let updater = ClaudeMdUpdater::new();
        
        let content = "# My Project\n\nThis is my project.\n\n## Features\n\n- Feature 1\n";
        let import_line = "@stacks/linting/CLAUDE.md";
        
        let result = updater.insert_stack_import(content, import_line);
        
        assert!(result.contains(import_line));
    }
    
    #[test]
    fn test_insert_with_existing_imports() {
        let updater = ClaudeMdUpdater::new();
        
        let content = "# My Project\n\n@stacks/testing/CLAUDE.md\n\n## Features\n";
        let import_line = "@stacks/linting/CLAUDE.md";
        
        let result = updater.insert_stack_import(content, import_line);
        
        assert!(result.contains("@stacks/testing/CLAUDE.md"));
        assert!(result.contains("@stacks/linting/CLAUDE.md"));
    }
}