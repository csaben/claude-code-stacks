#!/bin/bash

# Claude Code Stack Router
# Manages and orchestrates different workflow stacks for project automation

set -e

STACKS_DIR="$(dirname "$0")/stacks"
TMUX_SESSION="claude-stacks"

show_help() {
    cat << EOF
Claude Code Stack Router

Usage: $0 [OPTIONS] COMMAND [STACKS...]

COMMANDS:
    list                List available stacks
    run <stacks>       Run specified stacks in tmux panes
    apply <change>     Apply change using all necessary stacks
    status             Show current tmux session status
    kill               Kill the tmux session

STACKS:
    stack-1    Automatic linting across the project
    stack-2    Automatic testing of examples (nginx, docker, compose)
    stack-3    Clark idiomatic style formatting
    stack-4    Git commands with auto subtree services
    stack-5    CI/CD workflows for git
    stack-6    Design doc generation to Google Drive
    stack-7    Database setup with MCP configuration

OPTIONS:
    -h, --help     Show this help message
    -d, --dir      Specify project directory (default: current)

EXAMPLES:
    $0 list
    $0 run stack-1 stack-2
    $0 apply "update repo to work with fichub"
EOF
}

list_stacks() {
    echo "Available stacks:"
    for stack in "$STACKS_DIR"/stack-*; do
        if [[ -d "$stack" ]]; then
            stack_name=$(basename "$stack")
            description=$(grep -m1 "^# Description:" "$stack/CLAUDE.md" 2>/dev/null | cut -d: -f2- | xargs || echo "No description")
            echo "  $stack_name: $description"
        fi
    done
}

setup_tmux_session() {
    if tmux has-session -t "$TMUX_SESSION" 2>/dev/null; then
        echo "Tmux session '$TMUX_SESSION' already exists"
        return 0
    fi
    
    tmux new-session -d -s "$TMUX_SESSION" -c "$(pwd)"
    tmux rename-window -t "$TMUX_SESSION:0" "router"
    echo "Created tmux session: $TMUX_SESSION"
}

run_stack() {
    local stack_name="$1"
    local stack_path="$STACKS_DIR/$stack_name"
    
    if [[ ! -d "$stack_path" ]]; then
        echo "Error: Stack '$stack_name' not found"
        return 1
    fi
    
    setup_tmux_session
    
    # Create new window for this stack
    tmux new-window -t "$TMUX_SESSION" -n "$stack_name" -c "$stack_path"
    
    # Run the stack's initialization script
    if [[ -f "$stack_path/init.sh" ]]; then
        tmux send-keys -t "$TMUX_SESSION:$stack_name" "./init.sh" Enter
    else
        tmux send-keys -t "$TMUX_SESSION:$stack_name" "echo 'Stack $stack_name initialized. Ready for Claude Code.'" Enter
    fi
    
    echo "Started stack: $stack_name"
}

run_multiple_stacks() {
    local stacks=("$@")
    
    for stack in "${stacks[@]}"; do
        run_stack "$stack"
    done
    
    echo ""
    echo "All stacks started. To attach to tmux session:"
    echo "  tmux attach-session -t $TMUX_SESSION"
}

apply_change() {
    local change="$1"
    
    echo "Applying change: $change"
    echo "Auto-detecting required stacks..."
    
    # Auto-detect required stacks based on change description
    local required_stacks=()
    
    # Always include linting and styling
    required_stacks+=("stack-1" "stack-3")
    
    # Check if testing is needed
    if [[ "$change" =~ (test|testing|example) ]]; then
        required_stacks+=("stack-2")
    fi
    
    # Check if git operations are needed
    if [[ "$change" =~ (repo|git|push|commit) ]]; then
        required_stacks+=("stack-4" "stack-5")
    fi
    
    # Check if documentation is mentioned
    if [[ "$change" =~ (doc|documentation|design) ]]; then
        required_stacks+=("stack-6")
    fi
    
    # Check if database operations are needed
    if [[ "$change" =~ (database|db|docker.*compose) ]]; then
        required_stacks+=("stack-7")
    fi
    
    echo "Required stacks: ${required_stacks[*]}"
    run_multiple_stacks "${required_stacks[@]}"
}

kill_session() {
    if tmux has-session -t "$TMUX_SESSION" 2>/dev/null; then
        tmux kill-session -t "$TMUX_SESSION"
        echo "Killed tmux session: $TMUX_SESSION"
    else
        echo "No active tmux session found"
    fi
}

show_status() {
    if tmux has-session -t "$TMUX_SESSION" 2>/dev/null; then
        echo "Tmux session '$TMUX_SESSION' is running:"
        tmux list-windows -t "$TMUX_SESSION"
    else
        echo "No active tmux session"
    fi
}

# Main command processing
case "${1:-}" in
    list)
        list_stacks
        ;;
    run)
        shift
        if [[ $# -eq 0 ]]; then
            echo "Error: No stacks specified"
            show_help
            exit 1
        fi
        run_multiple_stacks "$@"
        ;;
    apply)
        if [[ -z "${2:-}" ]]; then
            echo "Error: No change specified"
            show_help
            exit 1
        fi
        apply_change "$2"
        ;;
    status)
        show_status
        ;;
    kill)
        kill_session
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