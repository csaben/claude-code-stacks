# Description: Automatic linting across the project

# Stack-1: Automatic Linting

This stack provides comprehensive linting capabilities across multiple languages and frameworks in your project.

## Capabilities
- Automatic detection of project type and language
- Configuration of appropriate linters (eslint, ruff, clippy, etc.)
- Real-time linting feedback
- Auto-fix capabilities where possible
- Integration with pre-commit hooks

## Usage
This stack automatically detects your project structure and applies appropriate linting rules.

## Commands
- `npm run lint` - Run JavaScript/TypeScript linting
- `ruff check` - Run Python linting
- `cargo clippy` - Run Rust linting
- `./fix-all.sh` - Auto-fix all linting issues

## Agent Configuration
The Claude Code agent in this stack has access to:
- File reading and editing tools
- Bash execution for linting commands
- Pattern matching for code style enforcement

## MCP Permissions
- File system access for reading/writing linting configurations
- Bash execution permissions for running linters
- Git access for pre-commit hook setup