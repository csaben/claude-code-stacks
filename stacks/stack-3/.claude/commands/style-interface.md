# Clark Style Natural Language Interface

This command enables natural language interaction with Clark's idiomatic style enforcement.

## Natural Language Patterns

Users can request style improvements in various ways:
- "Apply Clark's style guidelines"
- "Remove emojis from the documentation"
- "Make this README more concise"
- "Convert to uv/bun package management"
- "Clean up the documentation style"

## Clark Style Guidelines

The system enforces these specific style preferences:
1. **No emojis** in code, comments, or documentation
2. **Concise READMEs** - essential information only, under 200 lines
3. **Use uv** for Python package management (not pip/poetry)
4. **Use bun** for JavaScript/TypeScript (not npm/yarn)
5. **Minimal dependencies** - prefer standard library
6. **Clear, technical language** without marketing fluff

## Implementation

When users make style-related requests, the system:

1. **Scans Project**: Analyzes all documentation and code files
2. **Detects Violations**: Identifies style guideline violations  
3. **Auto-Fixes**: Applies safe automatic corrections
4. **Reports Changes**: Explains what was modified and why

## Headless Execution

```bash
# Apply all Clark style guidelines
claude -p "Apply Clark idiomatic style guidelines to entire project" --mode auto-accept

# Check for style violations only
claude -p "Generate Clark style compliance report" --mode plan

# Convert package managers
claude -p "Migrate project to use uv/bun package management" --mode auto-accept
```

## Example Transformations

### Before (Violations)
```markdown
# My Awesome Project! 🚀✨

Welcome to the most amazing project ever! 🎉 This will revolutionize everything! 

## Features 🌟
- Super cool feature A 😎
- Mind-blowing feature B 🤯
- Game-changing feature C 🔥

[... 300 lines of marketing speak ...]
```

### After (Clark Style)
```markdown
# My Awesome Project

Command-line tool for processing data files.

## Usage
```bash
myproject input.txt
```

## Requirements
- Python 3.8+
- uv for package management
```

## Integration Points

- Works with existing documentation structure
- Preserves essential technical information
- Integrates with package management workflows
- Coordinates with git stack for clean commits