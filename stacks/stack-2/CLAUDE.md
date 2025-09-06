# Description: Automatic testing of examples (nginx, docker compose, dockerfile)

# Stack-2: Automatic Testing

This stack provides comprehensive testing capabilities for infrastructure examples and application code.

## Capabilities
- Docker Compose validation and testing
- Dockerfile linting and security scanning
- Nginx configuration testing
- Integration testing of containerized services
- Example validation across project documentation

## Usage
This stack automatically discovers and validates infrastructure configurations and examples.

## Commands
- `./test-docker.sh` - Test all Docker configurations
- `./test-nginx.sh` - Validate Nginx configurations
- `./test-examples.sh` - Run all example validations
- `docker-compose config` - Validate compose files

## Testing Strategies
- Syntax validation for configuration files
- Container build and startup testing
- Service connectivity testing
- Security scanning with hadolint and other tools

## Agent Configuration
The Claude Code agent in this stack has access to:
- Docker and Docker Compose commands
- File reading for configuration validation
- Network testing tools
- Security scanning utilities

## MCP Permissions
- Docker daemon access
- File system access for configuration files
- Network access for connectivity testing
- Bash execution for testing scripts