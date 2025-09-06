#!/bin/bash

# Stack-1 Initialization: Automatic Linting Setup

echo "Initializing Stack-1: Automatic Linting"

# Detect project type and setup appropriate linters
detect_and_setup_linters() {
    local project_dir="${1:-$(pwd)}"
    
    echo "Detecting project type in: $project_dir"
    
    # JavaScript/TypeScript projects
    if [[ -f "$project_dir/package.json" ]]; then
        echo "  Detected Node.js project"
        setup_js_linting "$project_dir"
    fi
    
    # Python projects
    if [[ -f "$project_dir/pyproject.toml" ]] || [[ -f "$project_dir/requirements.txt" ]] || [[ -f "$project_dir/setup.py" ]]; then
        echo "  Detected Python project"
        setup_python_linting "$project_dir"
    fi
    
    # Rust projects
    if [[ -f "$project_dir/Cargo.toml" ]]; then
        echo "  Detected Rust project"
        setup_rust_linting "$project_dir"
    fi
    
    # Go projects
    if [[ -f "$project_dir/go.mod" ]]; then
        echo "  Detected Go project"
        setup_go_linting "$project_dir"
    fi
}

setup_js_linting() {
    local dir="$1"
    echo "    Setting up JavaScript/TypeScript linting"
    
    cd "$dir" || return
    
    # Check if eslint is already configured
    if [[ ! -f ".eslintrc.js" ]] && [[ ! -f ".eslintrc.json" ]] && [[ ! -f "eslint.config.js" ]]; then
        echo "    No ESLint config found. Would need to set up basic ESLint configuration."
    fi
    
    # Check if prettier is configured
    if [[ ! -f ".prettierrc" ]] && [[ ! -f "prettier.config.js" ]]; then
        echo "    No Prettier config found. Would need to set up basic Prettier configuration."
    fi
}

setup_python_linting() {
    local dir="$1"
    echo "    Setting up Python linting"
    
    cd "$dir" || return
    
    # Check for ruff configuration
    if [[ ! -f "ruff.toml" ]] && [[ ! -f "pyproject.toml" ]]; then
        echo "    No Ruff config found. Would need to set up basic Ruff configuration."
    fi
}

setup_rust_linting() {
    local dir="$1"
    echo "    Setting up Rust linting"
    echo "    Rust projects use clippy by default"
}

setup_go_linting() {
    local dir="$1"
    echo "    Setting up Go linting"
    echo "    Go projects can use golangci-lint"
}

# Main execution
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    # Find the target project directory
    TARGET_DIR="${1:-$(pwd)}"
    
    # If we're in the stacks directory, go up to find the actual project
    if [[ "$(basename "$(pwd)")" == "stack-1" ]] || [[ "$(basename "$(dirname "$(pwd)")")" == "stacks" ]]; then
        # Look for project directories
        for dir in /home/*/development/*/; do
            if [[ -f "$dir/package.json" ]] || [[ -f "$dir/pyproject.toml" ]] || [[ -f "$dir/Cargo.toml" ]]; then
                echo "Found potential project: $dir"
            fi
        done
        echo ""
        echo "Please specify project directory or navigate to project root and run:"
        echo "  ../stack-router.sh run stack-1"
    else
        detect_and_setup_linters "$TARGET_DIR"
    fi
    
    echo ""
    echo "Stack-1 (Linting) initialized and ready for Claude Code"
    echo "Attach with: tmux attach-session -t claude-stacks"
fi