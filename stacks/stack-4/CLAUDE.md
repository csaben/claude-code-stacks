# Description: Git commands with auto subtree services

# Stack-4: Git Operations with Auto Subtree Services

This stack provides automated git operations including subtree management for service repositories.

## Capabilities
- Automated git workflows (add, commit, push, pull)
- Subtree management for microservices
- Branch management and merging
- Automated commit message generation
- Pre-commit hook management

## Usage
This stack handles complex git operations automatically while providing manual override capabilities.

## Commands
- `./git-auto-commit.sh` - Intelligent auto-commit
- `./subtree-sync.sh` - Sync all subtrees
- `./branch-manager.sh` - Branch operations
- `./setup-hooks.sh` - Configure git hooks

## Subtree Management
- Automatic detection of service subdirectories
- Sync with remote service repositories
- Conflict resolution assistance
- Branch strategy enforcement

## Agent Configuration
The Claude Code agent in this stack has access to:
- Full git command suite
- File system for repository operations
- Branch and merge conflict resolution
- Automated commit message generation

## MCP Permissions
- Full git repository access
- File system read/write for git operations
- Network access for remote repository operations