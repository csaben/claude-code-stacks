# Description: Stack management and workflow automation

# Stack: stackstack - Stack Management & Workflow Automation

This stack provides comprehensive management capabilities for the stacks ecosystem, including automated git workflows when stacks are added or modified.

## Capabilities
- Automated detection of stack additions requiring git commits
- Stack management commands and workflows
- Git workflow automation for .claude file changes
- Stack validation and health checking
- Documentation and guidance for stack operations

## Usage
This stack automatically detects when stacks have been added and guides through the proper git workflow to commit the changes.

## Commands
- `/stack-commit` - Commit stack additions and .claude file changes
- `/stack-status` - Show status of all stacks and pending changes
- `/stack-validate` - Validate stack configurations and setup
- `/stack-sync` - Sync stack changes and update documentation

## Triggers
- After `stacks` checkout command completes
- When .claude files are modified by stack operations
- Before pushing stack changes

## Agent Configuration
The Claude Code agent in this stack has access to:
- Git operations for staging and committing changes
- File system access for reading stack configurations
- Stack status checking and validation
- Automated workflow recommendations

## MCP Permissions
- Git repository access for staging/committing changes
- File system access for .claude configuration files
- Bash execution for stack commands
- Read access to stack directories and metadata

## Workflow Integration
This stack integrates with the stacks workflow to:
1. Detect when new stacks have been added via `stacks checkout`
2. Identify changes to .claude symlinks and settings
3. Provide git commit recommendations with proper messages
4. Ensure stack additions are properly tracked in git history