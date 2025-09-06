# Linting Natural Language Interface

This command enables natural language interaction with the linting stack.

## Natural Language Patterns

Users can express linting requests in various ways:
- "Fix the linting issues"
- "Check code style"  
- "Apply code formatting"
- "Run the linter on this project"
- "Make the code follow our standards"

## Implementation

When users make linting-related requests, the system:

1. **Detects Intent**: Recognizes linting-related keywords and patterns
2. **Analyzes Project**: Determines project type (JS/TS, Python, Rust, etc.)
3. **Executes Workflow**: Runs appropriate linters headlessly
4. **Reports Results**: Provides natural language feedback

## Headless Execution

```bash
# Auto-detect project and run appropriate linter
claude -p "Analyze project structure and run appropriate linting tools" --mode auto-accept

# Fix auto-fixable issues  
claude -p "Auto-fix all safe linting violations in this project" --mode auto-accept

# Generate linting report
claude -p "Create comprehensive linting report" --mode plan
```

## Integration Points

- Connects to project's existing linting configuration
- Respects .eslintrc, ruff.toml, clippy settings
- Auto-installs missing linting dependencies
- Integrates with git pre-commit hooks