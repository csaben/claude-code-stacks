use std::path::PathBuf;
use std::fs;
use anyhow::{Result, Context};
use serde_json::{Value, Map};

use super::stack_manager::Stack;

pub struct SettingsMerger {
    local_settings_path: PathBuf,
}

impl SettingsMerger {
    pub fn new() -> Self {
        Self {
            local_settings_path: PathBuf::from(".claude/.local-settings.json"),
        }
    }

    /// Merge settings from a stack into the local settings file
    pub async fn merge_stack_settings(&self, stack: &Stack) -> Result<()> {
        let stack_settings_path = stack.claude_dir.join(".local-settings.json");
        
        if !stack_settings_path.exists() {
            // No settings to merge
            return Ok(());
        }

        // Read stack settings
        let stack_settings_content = tokio::fs::read_to_string(&stack_settings_path)
            .await
            .with_context(|| format!("Failed to read stack settings from {}", stack_settings_path.display()))?;
        
        let stack_settings: Value = serde_json::from_str(&stack_settings_content)
            .with_context(|| format!("Failed to parse JSON in {}", stack_settings_path.display()))?;

        // Read existing local settings or create empty object
        let mut local_settings = if self.local_settings_path.exists() {
            let local_content = tokio::fs::read_to_string(&self.local_settings_path)
                .await
                .with_context(|| format!("Failed to read local settings from {}", self.local_settings_path.display()))?;
            
            serde_json::from_str(&local_content)
                .with_context(|| format!("Failed to parse JSON in {}", self.local_settings_path.display()))?
        } else {
            // Ensure parent directory exists
            if let Some(parent) = self.local_settings_path.parent() {
                fs::create_dir_all(parent)
                    .with_context(|| format!("Failed to create directory {}", parent.display()))?;
            }
            Value::Object(Map::new())
        };

        // Merge stack settings into local settings
        deep_merge(&mut local_settings, stack_settings);

        // Write merged settings back
        let merged_content = serde_json::to_string_pretty(&local_settings)
            .context("Failed to serialize merged settings")?;
        
        tokio::fs::write(&self.local_settings_path, merged_content)
            .await
            .with_context(|| format!("Failed to write merged settings to {}", self.local_settings_path.display()))?;

        println!("  ⚙️ Merged settings from stack {}", stack.name);
        Ok(())
    }
}

/// Deep merge two JSON values, with the second value taking precedence
fn deep_merge(target: &mut Value, source: Value) {
    match (target, source) {
        (Value::Object(target_map), Value::Object(source_map)) => {
            for (key, value) in source_map {
                match target_map.get_mut(&key) {
                    Some(target_value) => {
                        // Recursively merge if both are objects
                        deep_merge(target_value, value);
                    }
                    None => {
                        // Insert new key-value pair
                        target_map.insert(key, value);
                    }
                }
            }
        }
        (Value::Array(target_array), Value::Array(source_array)) => {
            // For arrays, append unique items from source to target
            for source_item in source_array {
                if !target_array.contains(&source_item) {
                    target_array.push(source_item);
                }
            }
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
    use serde_json::json;

    #[test]
    fn test_deep_merge_objects() {
        let mut target = json!({
            "permissions": {
                "allow": ["npm run lint"]
            },
            "env": {
                "EXISTING": "value"
            }
        });

        let source = json!({
            "permissions": {
                "allow": ["ruff check"],
                "deny": ["rm -rf"]
            },
            "env": {
                "NEW": "value"
            }
        });

        deep_merge(&mut target, source);

        assert_eq!(target["permissions"]["allow"].as_array().unwrap().len(), 2);
        assert_eq!(target["permissions"]["deny"], json!(["rm -rf"]));
        assert_eq!(target["env"]["EXISTING"], json!("value"));
        assert_eq!(target["env"]["NEW"], json!("value"));
    }

    #[test]
    fn test_deep_merge_arrays() {
        let mut target = json!([1, 2, 3]);
        let source = json!([3, 4, 5]);
        
        deep_merge(&mut target, source);
        
        assert_eq!(target, json!([1, 2, 3, 4, 5]));
    }
}