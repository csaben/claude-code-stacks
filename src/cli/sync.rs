use std::path::Path;
use std::collections::HashMap;
use anyhow::{Result, Context};
use serde_yaml::Value as YamlValue;
use dialoguer::Confirm;

#[derive(Debug, Clone)]
pub struct DockerService {
    pub name: String,
    pub image: String,
    pub ports: Vec<String>,
    pub environment: HashMap<String, String>,
    pub service_type: ServiceType,
}

#[derive(Debug, Clone)]
pub enum ServiceType {
    Postgres,
    Redis,
    MongoDB,
    MySQL,
    Unknown(String),
}

pub async fn run() -> Result<()> {
    println!("🔍 Discovering services in docker-compose files...");
    
    let compose_files = find_docker_compose_files()?;
    if compose_files.is_empty() {
        println!("No docker-compose files found. Nothing to sync.");
        return Ok(());
    }

    println!("📁 Found {} docker-compose file(s):", compose_files.len());
    for file in &compose_files {
        println!("  • {}", file.display());
    }

    let mut all_services = Vec::new();
    
    // Parse all compose files
    for compose_file in &compose_files {
        let services = parse_docker_compose(compose_file).await
            .with_context(|| format!("Failed to parse {}", compose_file.display()))?;
        all_services.extend(services);
    }

    if all_services.is_empty() {
        println!("No MCP-compatible services found in docker-compose files.");
        return Ok(());
    }

    // Show discovered services
    println!("\n🎯 MCP-compatible services discovered:");
    for service in &all_services {
        println!("  • {} ({:?})", service.name, service.service_type);
    }

    // Generate MCP commands
    let mcp_commands = generate_mcp_commands(&all_services);
    
    println!("\n📋 Generated MCP server commands:");
    for (service, command) in all_services.iter().zip(mcp_commands.iter()) {
        println!("  • {}: {}", service.name, command);
    }

    let should_apply = Confirm::new()
        .with_prompt("Apply these MCP server configurations?")
        .default(true)
        .interact()?;

    if !should_apply {
        println!("Sync cancelled. No changes made.");
        return Ok(());
    }

    // Execute MCP server additions
    execute_mcp_commands(&mcp_commands).await?;

    println!("\n🎉 MCP sync completed successfully!");
    println!("💡 All discovered services have been configured as MCP servers.");

    Ok(())
}

fn find_docker_compose_files() -> Result<Vec<std::path::PathBuf>> {
    let compose_filenames = [
        "docker-compose.yml",
        "docker-compose.yaml", 
        "compose.yml",
        "compose.yaml",
        "docker-compose.override.yml",
        "docker-compose.dev.yml",
    ];

    let mut found_files = Vec::new();
    
    for filename in &compose_filenames {
        let path = std::path::Path::new(filename);
        if path.exists() {
            found_files.push(path.to_path_buf());
        }
    }

    Ok(found_files)
}

async fn parse_docker_compose(compose_file: &Path) -> Result<Vec<DockerService>> {
    let content = tokio::fs::read_to_string(compose_file)
        .await
        .with_context(|| format!("Failed to read {}", compose_file.display()))?;

    let yaml: YamlValue = serde_yaml::from_str(&content)
        .with_context(|| format!("Failed to parse YAML in {}", compose_file.display()))?;

    let mut services = Vec::new();
    
    if let Some(services_section) = yaml.get("services") {
        if let YamlValue::Mapping(services_map) = services_section {
            for (service_name, service_config) in services_map {
                if let Some(name) = service_name.as_str() {
                    if let Some(service) = parse_service_config(name, service_config) {
                        services.push(service);
                    }
                }
            }
        }
    }

    Ok(services)
}

fn parse_service_config(name: &str, config: &YamlValue) -> Option<DockerService> {
    let image = config.get("image")
        .and_then(|i| i.as_str())
        .unwrap_or("")
        .to_string();

    // Determine service type from image name
    let service_type = determine_service_type(&image, name);
    
    // Skip if not an MCP-compatible service
    if matches!(service_type, ServiceType::Unknown(_)) {
        return None;
    }

    // Extract ports
    let ports = extract_ports(config);
    
    // Extract environment variables
    let environment = extract_environment(config);

    Some(DockerService {
        name: name.to_string(),
        image,
        ports,
        environment,
        service_type,
    })
}

fn determine_service_type(image: &str, service_name: &str) -> ServiceType {
    let image_lower = image.to_lowercase();
    let name_lower = service_name.to_lowercase();
    
    if image_lower.contains("postgres") || name_lower.contains("postgres") {
        ServiceType::Postgres
    } else if image_lower.contains("redis") || name_lower.contains("redis") {
        ServiceType::Redis
    } else if image_lower.contains("mongo") || name_lower.contains("mongo") {
        ServiceType::MongoDB
    } else if image_lower.contains("mysql") || name_lower.contains("mysql") {
        ServiceType::MySQL
    } else {
        ServiceType::Unknown(image.to_string())
    }
}

fn extract_ports(config: &YamlValue) -> Vec<String> {
    let mut ports = Vec::new();
    
    if let Some(ports_section) = config.get("ports") {
        if let YamlValue::Sequence(ports_array) = ports_section {
            for port in ports_array {
                if let Some(port_str) = port.as_str() {
                    ports.push(port_str.to_string());
                }
            }
        }
    }
    
    ports
}

fn extract_environment(config: &YamlValue) -> HashMap<String, String> {
    let mut env = HashMap::new();
    
    if let Some(env_section) = config.get("environment") {
        match env_section {
            YamlValue::Mapping(env_map) => {
                for (key, value) in env_map {
                    if let (Some(k), Some(v)) = (key.as_str(), value.as_str()) {
                        env.insert(k.to_string(), v.to_string());
                    }
                }
            }
            YamlValue::Sequence(env_array) => {
                for item in env_array {
                    if let Some(env_str) = item.as_str() {
                        if let Some((key, value)) = env_str.split_once('=') {
                            env.insert(key.to_string(), value.to_string());
                        }
                    }
                }
            }
            _ => {}
        }
    }
    
    env
}

fn generate_mcp_commands(services: &[DockerService]) -> Vec<String> {
    services
        .iter()
        .map(|service| generate_mcp_command_for_service(service))
        .collect()
}

fn generate_mcp_command_for_service(service: &DockerService) -> String {
    match service.service_type {
        ServiceType::Postgres => {
            let password = service.environment.get("POSTGRES_PASSWORD")
                .or_else(|| service.environment.get("POSTGRES_DB"))
                .map(|p| p.as_str())
                .unwrap_or("password");
            
            let database = service.environment.get("POSTGRES_DB")
                .map(|db| db.as_str())
                .unwrap_or("postgres");
            
            let user = service.environment.get("POSTGRES_USER")
                .map(|u| u.as_str())
                .unwrap_or("postgres");
            
            let port = extract_host_port(&service.ports).unwrap_or("5432");
            
            format!(
                "claude mcp add postgres -- npx -y @modelcontextprotocol/server-postgres postgresql://{}:{}@localhost:{}/{}",
                user, password, port, database
            )
        }
        
        ServiceType::Redis => {
            let port = extract_host_port(&service.ports).unwrap_or("6379");
            let password = service.environment.get("REDIS_PASSWORD");
            
            if let Some(pwd) = password {
                format!("claude mcp add redis -- docker run -i --rm mcp/redis redis://default:{}@host.docker.internal:{}", pwd, port)
            } else {
                format!("claude mcp add redis -- docker run -i --rm mcp/redis redis://host.docker.internal:{}", port)
            }
        }
        
        ServiceType::MongoDB => {
            let port = extract_host_port(&service.ports).unwrap_or("27017");
            let user = service.environment.get("MONGO_INITDB_ROOT_USERNAME").map_or("admin", |v| v);
            let password = service.environment.get("MONGO_INITDB_ROOT_PASSWORD").map_or("password", |v| v);
            let database = service.environment.get("MONGO_INITDB_DATABASE").map_or("admin", |v| v);
            
            format!(
                "# MongoDB MCP server not officially available, manual setup required\n# Connection: mongodb://{}:{}@localhost:{}/{}",
                user, password, port, database
            )
        }
        
        ServiceType::MySQL => {
            let port = extract_host_port(&service.ports).unwrap_or("3306");
            let user = service.environment.get("MYSQL_USER").map_or("root", |v| v);
            let password = service.environment.get("MYSQL_PASSWORD")
                .or_else(|| service.environment.get("MYSQL_ROOT_PASSWORD"))
                .map_or("password", |v| v);
            let database = service.environment.get("MYSQL_DATABASE").map_or("mysql", |v| v);
            
            format!(
                "# MySQL MCP server not officially available, manual setup required\n# Connection: mysql://{}:{}@localhost:{}/{}",
                user, password, port, database
            )
        }
        
        ServiceType::Unknown(_) => {
            format!("# Unknown service type: {}", service.name)
        }
    }
}

fn extract_host_port(ports: &[String]) -> Option<&str> {
    ports.first()
        .and_then(|port| {
            if port.contains(':') {
                port.split(':').next()
            } else {
                Some(port.as_str())
            }
        })
}

async fn execute_mcp_commands(commands: &[String]) -> Result<()> {
    println!("\n🚀 Executing MCP server configurations...");
    
    for (i, command) in commands.iter().enumerate() {
        if command.starts_with('#') {
            println!("  ℹ️ Skipping manual configuration: {}", command);
            continue;
        }

        println!("  {} Executing: {}", i + 1, command);
        
        // Parse the command to extract arguments
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.len() < 4 || parts[0] != "claude" || parts[1] != "mcp" || parts[2] != "add" {
            println!("    ⚠️ Invalid command format, skipping");
            continue;
        }

        let output = std::process::Command::new("claude")
            .args(&parts[1..])
            .output()
            .with_context(|| format!("Failed to execute command: {}", command))?;

        if output.status.success() {
            println!("    ✅ Success");
        } else {
            let error = String::from_utf8_lossy(&output.stderr);
            println!("    ❌ Failed: {}", error);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_determine_service_type() {
        assert!(matches!(
            determine_service_type("postgres:13", "db"), 
            ServiceType::Postgres
        ));
        
        assert!(matches!(
            determine_service_type("redis:alpine", "cache"), 
            ServiceType::Redis
        ));
        
        assert!(matches!(
            determine_service_type("mongo:4.4", "documents"), 
            ServiceType::MongoDB
        ));
    }

    #[test]
    fn test_extract_host_port() {
        assert_eq!(extract_host_port(&["5432:5432".to_string()]), Some("5432"));
        assert_eq!(extract_host_port(&["8080:80".to_string()]), Some("8080"));
        assert_eq!(extract_host_port(&["3000".to_string()]), Some("3000"));
    }
}