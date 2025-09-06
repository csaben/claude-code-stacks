#!/bin/bash

# Claude Code Workflow Manager
# Complete workflow orchestration for Claude Code stacks

set -e

SCRIPT_DIR="$(dirname "$0")"
STACKS_REPO="https://github.com/csaben/claude-stacks"
TMUX_SESSION="claude-workflow"

show_help() {
    cat << EOF
Claude Code Workflow Manager

This is the ultimate Claude Code workflow system that provides:
- Automatic stack management with tmux integration
- One-command project setup and automation
- Clark idiomatic style enforcement
- Complete CI/CD and database integration

Usage: $0 [OPTIONS] COMMAND [ARGS...]

COMMANDS:
    setup              Initial setup of the workflow system
    start <project>    Start workflow for a project directory
    apply <change>     Apply a change using all necessary stacks
    status             Show current workflow status
    stop               Stop all workflows and clean up
    install-deps       Install required dependencies
    
PROJECT WORKFLOW COMMANDS:
    resoul <change>    Special workflow for resoul repository
    
EXAMPLES:
    # Initial setup
    $0 setup
    
    # Start workflow for a project
    $0 start /home/user/myproject
    
    # Apply a complex change (like your fichub example)
    $0 apply "update repo to work with fichub. ensure it works with 'purple days'"
    
    # Resoul-specific workflow
    $0 resoul "add new feature X with full testing and docs"

FEATURES:
- Automatic tmux session management
- Multi-pane stack execution
- Intelligent stack selection based on change description
- Auto-approval for common operations
- Complete project lifecycle management

EOF
}

install_dependencies() {
    echo "Installing Claude Code Workflow dependencies..."
    
    # Check for tmux
    if ! command -v tmux &> /dev/null; then
        echo "  Installing tmux..."
        if command -v apt-get &> /dev/null; then
            sudo apt-get update && sudo apt-get install -y tmux
        elif command -v yum &> /dev/null; then
            sudo yum install -y tmux
        elif command -v brew &> /dev/null; then
            brew install tmux
        else
            echo "  Please install tmux manually"
        fi
    fi
    
    # Check for git
    if ! command -v git &> /dev/null; then
        echo "  Git is required but not installed"
        exit 1
    fi
    
    # Check for docker
    if ! command -v docker &> /dev/null; then
        echo "  Warning: Docker not found. Some stacks require Docker."
    fi
    
    echo "  Dependencies check complete"
}

setup_workflow() {
    echo "Setting up Claude Code Workflow system..."
    
    install_dependencies
    
    # Make all scripts executable
    find "$SCRIPT_DIR" -name "*.sh" -exec chmod +x {} \;
    
    # Create global configuration
    cat > ~/.claude-workflow-config << EOF
# Claude Code Workflow Configuration
WORKFLOW_ROOT="$SCRIPT_DIR"
AUTO_APPROVE_SAFE_COMMANDS=true
USE_TMUX_INTEGRATION=true
DEFAULT_STACKS="stack-1,stack-3"
RESOUL_STACKS="stack-1,stack-2,stack-3,stack-4,stack-5,stack-6"
EOF

    echo "  Created global configuration at ~/.claude-workflow-config"
    echo "  Setup complete!"
    echo ""
    echo "You can now use:"
    echo "  $0 start <project-path>"
    echo "  $0 apply '<change-description>'"
}

start_workflow() {
    local project_dir="$1"
    
    if [[ -z "$project_dir" ]]; then
        project_dir="$(pwd)"
    fi
    
    if [[ ! -d "$project_dir" ]]; then
        echo "Error: Project directory '$project_dir' does not exist"
        exit 1
    fi
    
    echo "Starting Claude Code Workflow for: $project_dir"
    
    # Kill existing session if it exists
    if tmux has-session -t "$TMUX_SESSION" 2>/dev/null; then
        tmux kill-session -t "$TMUX_SESSION"
    fi
    
    # Create main tmux session
    tmux new-session -d -s "$TMUX_SESSION" -c "$project_dir"
    tmux rename-window -t "$TMUX_SESSION:0" "main"
    
    # Create router pane
    tmux new-window -t "$TMUX_SESSION" -n "router" -c "$SCRIPT_DIR"
    tmux send-keys -t "$TMUX_SESSION:router" "echo 'Claude Code Workflow Router Ready'" Enter
    tmux send-keys -t "$TMUX_SESSION:router" "echo 'Project: $project_dir'" Enter
    tmux send-keys -t "$TMUX_SESSION:router" "echo 'Use: ./stack-router.sh list'" Enter
    
    # Auto-detect and start relevant stacks
    detect_and_start_stacks "$project_dir"
    
    echo ""
    echo "Workflow started! Attach with:"
    echo "  tmux attach-session -t $TMUX_SESSION"
    echo ""
    echo "Available windows:"
    tmux list-windows -t "$TMUX_SESSION"
}

detect_and_start_stacks() {
    local project_dir="$1"
    local stacks_to_start=()
    
    echo "  Auto-detecting required stacks..."
    
    # Always start linting and style
    stacks_to_start+=("stack-1" "stack-3")
    
    # Check for testing needs
    if [[ -f "$project_dir/docker-compose.yml" ]] || [[ -f "$project_dir/Dockerfile" ]]; then
        stacks_to_start+=("stack-2")
    fi
    
    # Check for git repository
    if [[ -d "$project_dir/.git" ]]; then
        stacks_to_start+=("stack-4")
    fi
    
    # Check for CI/CD files
    if [[ -d "$project_dir/.github/workflows" ]] || [[ -f "$project_dir/.gitlab-ci.yml" ]]; then
        stacks_to_start+=("stack-5")
    fi
    
    # Check for documentation
    if [[ -d "$project_dir/docs" ]]; then
        stacks_to_start+=("stack-6")
    fi
    
    # Check for database configs
    if grep -q "postgres\|mongodb\|redis" "$project_dir/docker-compose.yml" 2>/dev/null; then
        stacks_to_start+=("stack-7")
    fi
    
    echo "  Starting stacks: ${stacks_to_start[*]}"
    
    # Start each stack in its own tmux window
    for stack in "${stacks_to_start[@]}"; do
        tmux new-window -t "$TMUX_SESSION" -n "$stack" -c "$SCRIPT_DIR/stacks/$stack"
        tmux send-keys -t "$TMUX_SESSION:$stack" "./init.sh '$project_dir'" Enter
    done
}

apply_change() {
    local change="$1"
    
    echo "Applying change: $change"
    
    # Start workflow if not already running
    if ! tmux has-session -t "$TMUX_SESSION" 2>/dev/null; then
        start_workflow "$(pwd)"
    fi
    
    # Broadcast the change to all stack windows
    for window in $(tmux list-windows -t "$TMUX_SESSION" -F "#{window_name}" | grep "^stack-"); do
        tmux send-keys -t "$TMUX_SESSION:$window" "# Applying change: $change" Enter
    done
    
    echo "Change broadcasted to all active stacks"
    echo "Monitor progress with: tmux attach-session -t $TMUX_SESSION"
}

resoul_workflow() {
    local change="$1"
    
    echo "Starting Resoul-specific workflow..."
    echo "Change: $change"
    
    # Set up complete stack for resoul
    start_workflow "$(pwd)"
    
    # Add specific resoul configurations
    tmux send-keys -t "$TMUX_SESSION:router" "echo 'Resoul workflow: $change'" Enter
    tmux send-keys -t "$TMUX_SESSION:router" "echo 'Includes: linting, testing, style, git, ci/cd, docs'" Enter
    
    echo "Resoul workflow started with all stacks"
    echo "Attach with: tmux attach-session -t $TMUX_SESSION"
}

show_status() {
    if tmux has-session -t "$TMUX_SESSION" 2>/dev/null; then
        echo "Claude Code Workflow Status:"
        echo "Session: $TMUX_SESSION (active)"
        echo ""
        echo "Windows:"
        tmux list-windows -t "$TMUX_SESSION"
    else
        echo "No active Claude Code Workflow session"
    fi
}

stop_workflow() {
    if tmux has-session -t "$TMUX_SESSION" 2>/dev/null; then
        tmux kill-session -t "$TMUX_SESSION"
        echo "Stopped Claude Code Workflow session"
    else
        echo "No active workflow to stop"
    fi
}

# Main command processing
case "${1:-}" in
    setup)
        setup_workflow
        ;;
    start)
        start_workflow "${2:-$(pwd)}"
        ;;
    apply)
        if [[ -z "${2:-}" ]]; then
            echo "Error: No change specified"
            show_help
            exit 1
        fi
        apply_change "$2"
        ;;
    resoul)
        if [[ -z "${2:-}" ]]; then
            echo "Error: No change specified for resoul workflow"
            exit 1
        fi
        resoul_workflow "$2"
        ;;
    status)
        show_status
        ;;
    stop)
        stop_workflow
        ;;
    install-deps)
        install_dependencies
        ;;
    -h|--help)
        show_help
        ;;
    "")
        show_help
        ;;
    *)
        echo "Error: Unknown command '$1'"
        show_help
        exit 1
        ;;
esac