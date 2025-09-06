# Description: Database setup with MCP configuration awareness

# Stack-7: Database Setup and MCP Configuration

This stack automatically sets up databases and configures MCP tools with correct connection URLs.

## Capabilities
- Docker Compose database parsing
- Automatic database URL extraction
- MCP configuration updates
- Database migration management
- Multi-database environment support

## Usage
This stack reads docker-compose files and automatically configures Claude agents with database access.

## Commands
- `./setup-databases.sh` - Initialize all databases
- `./configure-mcps.sh` - Update MCP configurations
- `./migrate-schemas.sh` - Run database migrations
- `./test-connections.sh` - Verify database connectivity

## Database Support
- PostgreSQL with pgvector extension
- MongoDB for document storage
- Redis for caching and sessions
- SQLite for development/testing

## MCP Integration
- Automatic URL generation from compose files
- Environment-specific configuration
- Credential management and security
- Connection pooling optimization

## Agent Configuration
The Claude Code agent in this stack has access to:
- Docker Compose file parsing
- Database connection tools
- MCP configuration management
- Schema migration utilities

## MCP Permissions
- Docker and Docker Compose access
- File system access for config files
- Database connection permissions
- Network access for database operations