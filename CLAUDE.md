# Description: Automatic linting across the project
# Setup Prompt: "Set up comprehensive linting for this project. This includes: 1) Create a git worktree for stack-1 from https://github.com/csaben/claude-code-stacks.git in the stacks/stack-1 directory, 2) Create symlinks from stack-1/.claude/agents/ to .claude/agents/ for agent discovery, 3) Install and configure appropriate linters (ESLint/Biome for JS/TS, Ruff for Python, Clippy for Rust), 4) Set up pre-commit hooks, 5) Run initial linting and fix any issues found"

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