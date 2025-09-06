# Description: CI/CD workflows for git

# Stack-5: CI/CD Workflows

This stack manages continuous integration and deployment workflows for git repositories.

## Capabilities
- GitHub Actions workflow management
- GitLab CI/CD pipeline configuration
- Automated testing on push/PR
- Deployment automation
- Release management and versioning

## Usage
This stack automatically configures and manages CI/CD pipelines based on project structure.

## Commands
- `./setup-github-actions.sh` - Configure GitHub Actions
- `./setup-gitlab-ci.sh` - Configure GitLab CI
- `./create-release.sh` - Create and tag releases
- `./deploy.sh` - Execute deployment

## Workflow Templates
- Node.js/Bun projects with testing and deployment
- Python/uv projects with testing and packaging
- Docker-based deployments
- Multi-environment deployment strategies

## Agent Configuration
The Claude Code agent in this stack has access to:
- CI/CD configuration file management
- Git tagging and release operations
- Deployment script generation
- Environment variable management

## MCP Permissions
- File system access for CI/CD configs
- Git operations for releases and tags
- Network access for deployment operations