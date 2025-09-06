use std::collections::HashMap;
use std::process::Command;
use anyhow::{Result, Context};
use serde_json::Value;

pub struct McpValidator;

#[derive(Debug, Clone)]
pub struct McpServer {
    pub name: String,
    pub transport: String,
    pub command: Option<String>,
    pub url: Option<String>,
    #[allow(dead_code)]
    pub env: Option<HashMap<String, String>>,
}

impl McpValidator {
    pub fn new() -> Self {
        Self
    }

    /// Check which MCP servers are referenced in settings but not installed
    pub async fn validate_mcp_servers(&self) -> Result<Vec<McpServer>> {
        let settings = self.load_merged_settings().await?;
        let required_servers = self.extract_mcp_servers_from_settings(&settings)?;
        let installed_servers = self.get_installed_mcp_servers().await?;
        
        let missing_servers: Vec<McpServer> = required_servers
            .into_iter()
            .filter(|server| !installed_servers.contains(&server.name))
            .collect();

        Ok(missing_servers)
    }

    /// Load and merge all settings files to get the complete MCP configuration
    async fn load_merged_settings(&self) -> Result<Value> {
        let mut merged = serde_json::Value::Object(serde_json::Map::new());
        
        // Load .claude/.local-settings.json if it exists
        let local_settings_path = ".claude/.local-settings.json";
        if std::path::Path::new(local_settings_path).exists() {
            let content = tokio::fs::read_to_string(local_settings_path).await?;
            let settings: Value = serde_json::from_str(&content)?;
            self.merge_json(&mut merged, settings);
        }

        Ok(merged)
    }

    /// Extract MCP server requirements from settings
    fn extract_mcp_servers_from_settings(&self, settings: &Value) -> Result<Vec<McpServer>> {
        let mut servers = Vec::new();
        
        // Look for MCP configurations in various possible locations
        if let Some(mcp_config) = settings.get("mcp") {
            if let Some(Value::Object(servers_map)) = mcp_config.get("servers") {
                for (name, config) in servers_map {
                    let server = self.parse_mcp_server_config(name, config)?;
                    servers.push(server);
                }
            }
        }

        // Also check for permissions that might reference MCP servers
        if let Some(permissions) = settings.get("permissions") {
            if let Some(Value::Array(allow_array)) = permissions.get("allow") {
                for permission in allow_array {
                    if let Value::String(perm_str) = permission {
                        if let Some(server) = self.extract_mcp_from_permission(perm_str) {
                            servers.push(server);
                        }
                    }
                }
            }
        }

        Ok(servers)
    }

    /// Parse MCP server configuration from JSON
    fn parse_mcp_server_config(&self, name: &str, config: &Value) -> Result<McpServer> {
        let transport = config.get("transport")
            .and_then(|t| t.as_str())
            .unwrap_or("stdio")
            .to_string();

        let command = config.get("command")
            .and_then(|c| c.as_str())
            .map(|s| s.to_string());

        let url = config.get("url")
            .and_then(|u| u.as_str())
            .map(|s| s.to_string());

        let env = config.get("env")
            .and_then(|e| e.as_object())
            .map(|obj| {
                obj.iter()
                    .map(|(k, v)| (k.clone(), v.as_str().unwrap_or("").to_string()))
                    .collect()
            });

        Ok(McpServer {
            name: name.to_string(),
            transport,
            command,
            url,
            env,
        })
    }

    /// Try to extract MCP server info from permission strings
    fn extract_mcp_from_permission(&self, permission: &str) -> Option<McpServer> {
        // This is a heuristic approach - look for common patterns
        if permission.contains("postgresql://") {
            return Some(McpServer {
                name: "postgres".to_string(),
                transport: "stdio".to_string(),
                command: Some("npx -y @modelcontextprotocol/server-postgres".to_string()),
                url: None,
                env: None,
            });
        }
        
        if permission.contains("redis://") {
            return Some(McpServer {
                name: "redis".to_string(),
                transport: "stdio".to_string(),
                command: Some("docker run -i --rm mcp/redis".to_string()),
                url: None,
                env: None,
            });
        }

        None
    }

    /// Get list of currently installed MCP servers
    async fn get_installed_mcp_servers(&self) -> Result<Vec<String>> {
        let output = Command::new("claude")
            .args(["mcp", "list"])
            .output()
            .context("Failed to run 'claude mcp list'")?;

        if !output.status.success() {
            // If claude mcp list fails, assume no servers are installed
            return Ok(Vec::new());
        }

        let output_str = String::from_utf8_lossy(&output.stdout);
        let servers: Vec<String> = output_str
            .lines()
            .filter_map(|line| {
                // Parse the output format of `claude mcp list`
                // This is a simplified parser - may need adjustment based on actual format
                if line.trim().is_empty() || line.starts_with("No servers") {
                    None
                } else {
                    // Extract server name (assuming first word is the name)
                    line.split_whitespace().next().map(|s| s.to_string())
                }
            })
            .collect();

        Ok(servers)
    }

    /// Generate installation commands for missing MCP servers
    pub fn generate_installation_commands(&self, missing_servers: &[McpServer]) -> Vec<String> {
        missing_servers
            .iter()
            .map(|server| self.generate_install_command(server))
            .collect()
    }

    /// Generate a claude mcp add command for a server
    fn generate_install_command(&self, server: &McpServer) -> String {
        match server.transport.as_str() {
            "http" => {
                if let Some(url) = &server.url {
                    format!("claude mcp add --transport http {} {}", server.name, url)
                } else {
                    format!("# Unable to generate command for {} - missing URL", server.name)
                }
            }
            "stdio" => {
                if let Some(command) = &server.command {
                    format!("claude mcp add {} -- {}", server.name, command)
                } else {
                    self.generate_common_server_command(&server.name)
                }
            }
            _ => format!("# Unknown transport type for server: {}", server.name),
        }
    }

    /// Generate common server installation commands
    fn generate_common_server_command(&self, server_name: &str) -> String {
        match server_name {
            "postgres" => "claude mcp add postgres -- npx -y @modelcontextprotocol/server-postgres postgresql://localhost/your_database".to_string(),
            "redis" => "claude mcp add redis -- docker run -i --rm mcp/redis redis://host.docker.internal:6379".to_string(),
            "github" => "# GitHub MCP requires authentication - see: https://github.com/github/github-mcp-server".to_string(),
            "sentry" => "claude mcp add --transport http sentry https://mcp.sentry.dev/mcp".to_string(),
            "jam" => "claude mcp add --transport http jam https://mcp.jam.dev/mcp".to_string(),
            _ => format!("# Unknown server type: {} - manual configuration required", server_name),
        }
    }

    /// Merge two JSON values
    fn merge_json(&self, target: &mut Value, source: Value) {
        Self::merge_json_static(target, source)
    }

    /// Static helper for merging JSON values
    fn merge_json_static(target: &mut Value, source: Value) {
        match (target, source) {
            (Value::Object(target_map), Value::Object(source_map)) => {
                for (key, value) in source_map {
                    match target_map.get_mut(&key) {
                        Some(target_value) => Self::merge_json_static(target_value, value),
                        None => {
                            target_map.insert(key, value);
                        }
                    }
                }
            }
            (Value::Array(target_array), Value::Array(source_array)) => {
                for item in source_array {
                    if !target_array.contains(&item) {
                        target_array.push(item);
                    }
                }
            }
            (target_val, source_val) => *target_val = source_val,
        }
    }
}