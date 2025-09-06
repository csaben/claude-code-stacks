# Description: Clark idiomatic style formatting (no emojis, concise readmes, use uv, use bun)

# Stack-3: Clark Idiomatic Style Formatting

This stack enforces Clark's preferred coding and documentation style across the project.

## Style Guidelines
- **No emojis** in code, comments, or documentation
- **Concise READMEs** - essential information only
- **Use uv** for Python package management
- **Use bun** for JavaScript/TypeScript package management
- **Minimal dependencies** - prefer standard library when possible
- **Clear, direct documentation** without fluff

## Capabilities
- Automatic style enforcement across codebases
- README optimization and conciseness checks
- Package manager standardization (uv/bun)
- Code style consistency enforcement
- Documentation clarity improvements

## Usage
This stack automatically applies Clark's preferred styles and suggests improvements.

## Commands
- `./apply-clark-style.sh` - Apply all style guidelines
- `./check-style.sh` - Check for style violations
- `./optimize-readme.sh` - Optimize README files
- `./standardize-deps.sh` - Standardize package managers

## Style Rules
1. Remove all emojis from code and docs
2. Keep READMEs under 200 lines, focus on essentials
3. Convert npm/yarn projects to bun
4. Convert pip/poetry projects to uv
5. Use clear, technical language
6. Avoid marketing speak in documentation

## Agent Configuration
The Claude Code agent in this stack has access to:
- File editing for style corrections
- Package manager conversion tools
- Documentation optimization utilities
- Style violation detection

## MCP Permissions
- File system access for style corrections
- Package manager command execution
- Git operations for style commits