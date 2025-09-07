use std::path::{Path, PathBuf};
use anyhow::{Result, Context};
use serde_json::{json, Value};

/// Generates permission configurations for feature branch worktrees
/// that protect the main directory while allowing full access to feature directories
pub struct PermissionGenerator {
    main_directory: PathBuf,
    feature_directory: PathBuf,
}

impl PermissionGenerator {
    pub fn new(main_directory: PathBuf, feature_directory: PathBuf) -> Self {
        Self {
            main_directory,
            feature_directory,
        }
    }

    /// Generate the permission configuration JSON that protects main directory
    /// while allowing full access to feature directory
    pub fn generate_permission_config(&self) -> Result<Value> {
        let main_path = self.main_directory.canonicalize()
            .context("Failed to resolve main directory path")?;
        let feature_path = self.feature_directory.canonicalize()
            .context("Failed to resolve feature directory path")?;

        let main_path_str = main_path.to_str()
            .context("Main directory path contains invalid UTF-8")?;
        let feature_path_str = feature_path.to_str()
            .context("Feature directory path contains invalid UTF-8")?;

        let config = json!({
            "permissions": {
                "allow": [
                    format!("Read({}/*)", main_path_str),
                    format!("Read({}/*)", feature_path_str),
                    format!("Bash(cd:{})", main_path_str),
                    format!("Bash(cd:{})", feature_path_str),
                    "Bash(git:*)",
                    "Bash(stacks:cleanup)",
                    "Bash(touch:*)",
                    "Bash(mkdir:*)",
                    "Bash(echo:*)",
                    "Bash(cat:*)",
                    "Bash(vim:*)",
                    "Bash(nano:*)",
                    "Bash(cp:*)",
                    "Bash(mv:*)",
                    "Bash(rm:*)"
                ],
                "deny": [
                    format!("Write({}/*)", main_path_str),
                    format!("Edit({}/*)", main_path_str),
                    format!("MultiEdit({}/*)", main_path_str),
                    format!("DeleteFile({}/*)", main_path_str),
                    format!("Bash(rm:{}/*)", main_path_str),
                    format!("Bash(mv:{}/*)", main_path_str),
                    format!("Bash(cp:*/{}/*)", main_path_str)
                ]
            }
        });

        Ok(config)
    }

    /// Generate permission config and merge it into existing settings
    pub async fn apply_to_local_settings(&self, settings_path: &Path) -> Result<()> {
        let permission_config = self.generate_permission_config()?;

        // Read existing settings or create new ones
        let mut existing_settings = if settings_path.exists() {
            let content = tokio::fs::read_to_string(settings_path).await
                .with_context(|| format!("Failed to read settings from {}", settings_path.display()))?;
            
            serde_json::from_str(&content)
                .with_context(|| format!("Failed to parse JSON in {}", settings_path.display()))?
        } else {
            // Create directory if it doesn't exist
            if let Some(parent) = settings_path.parent() {
                tokio::fs::create_dir_all(parent).await
                    .with_context(|| format!("Failed to create directory {}", parent.display()))?;
            }
            json!({})
        };

        // Merge the permission config into existing settings
        deep_merge(&mut existing_settings, permission_config);

        // Write back to file
        let content = serde_json::to_string_pretty(&existing_settings)
            .context("Failed to serialize settings")?;
        
        tokio::fs::write(settings_path, content).await
            .with_context(|| format!("Failed to write settings to {}", settings_path.display()))?;

        Ok(())
    }
}

/// Deep merge two JSON values, with the second value taking precedence for permissions
fn deep_merge(target: &mut Value, source: Value) {
    match (target, source) {
        (Value::Object(target_map), Value::Object(source_map)) => {
            for (key, value) in source_map {
                match target_map.get_mut(&key) {
                    Some(target_value) => {
                        // For permissions, we want to merge arrays
                        if key == "permissions" {
                            deep_merge(target_value, value);
                        } else {
                            // For other keys, source takes precedence
                            *target_value = value;
                        }
                    }
                    None => {
                        // Insert new key-value pair
                        target_map.insert(key, value);
                    }
                }
            }
        }
        (Value::Array(target_array), Value::Array(source_array)) => {
            // For permission arrays, replace entirely to avoid duplicates
            *target_array = source_array;
        }
        (target_val, source_val) => {
            // For primitive values, source takes precedence
            *target_val = source_val;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_generate_permission_config() {
        let temp_main = TempDir::new().unwrap();
        let temp_feature = TempDir::new().unwrap();
        
        let generator = PermissionGenerator::new(
            temp_main.path().to_path_buf(),
            temp_feature.path().to_path_buf(),
        );
        
        let config = generator.generate_permission_config().unwrap();
        
        // Verify structure
        assert!(config["permissions"]["allow"].is_array());
        assert!(config["permissions"]["deny"].is_array());
        
        let allow_rules = config["permissions"]["allow"].as_array().unwrap();
        let deny_rules = config["permissions"]["deny"].as_array().unwrap();
        
        // Should have read access to both directories
        assert!(allow_rules.iter().any(|v| 
            v.as_str().unwrap().contains("Read") && 
            v.as_str().unwrap().contains(temp_main.path().to_str().unwrap())
        ));
        
        // Should deny writes to main directory
        assert!(deny_rules.iter().any(|v| 
            v.as_str().unwrap().contains("Write") && 
            v.as_str().unwrap().contains(temp_main.path().to_str().unwrap())
        ));
        
        // Should allow git operations
        assert!(allow_rules.iter().any(|v| v.as_str().unwrap() == "Bash(git:*)"));
    }
}