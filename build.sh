#!/bin/bash

# Build script for the Stacks CLI
# Supports cross-compilation for multiple targets

set -e

# Configuration
BINARY_NAME="stacks"
BUILD_DIR="target"
DIST_DIR="dist"

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m'

print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Supported targets
TARGETS=(
    "x86_64-unknown-linux-gnu"
    "aarch64-unknown-linux-gnu"
    "x86_64-apple-darwin"
    "aarch64-apple-darwin"
    "x86_64-pc-windows-gnu"
)

# Check if cargo is installed
check_cargo() {
    if ! command -v cargo >/dev/null 2>&1; then
        print_error "Cargo is not installed. Please install Rust: https://rustup.rs/"
        exit 1
    fi
}

# Install required targets
install_targets() {
    print_status "Installing cross-compilation targets..."
    
    for target in "${TARGETS[@]}"; do
        print_status "Adding target: $target"
        rustup target add "$target" || true
    done
}

# Build for a specific target
build_target() {
    local target=$1
    local output_name=$2
    
    print_status "Building for $target..."
    
    if cargo build --release --target "$target"; then
        # Copy and rename binary
        local source_path="$BUILD_DIR/$target/release/$BINARY_NAME"
        local dest_path="$DIST_DIR/$output_name"
        
        # Windows binaries have .exe extension
        if [[ "$target" == *"windows"* ]]; then
            source_path="${source_path}.exe"
            dest_path="${dest_path}.exe"
        fi
        
        if [ -f "$source_path" ]; then
            cp "$source_path" "$dest_path"
            print_success "Built $output_name"
        else
            print_error "Binary not found at $source_path"
            return 1
        fi
    else
        print_error "Failed to build for $target"
        return 1
    fi
}

# Build all targets
build_all() {
    print_status "Starting cross-compilation build..."
    
    # Create dist directory
    mkdir -p "$DIST_DIR"
    
    # Build for each target
    build_target "x86_64-unknown-linux-gnu" "stacks-linux-x86_64"
    build_target "aarch64-unknown-linux-gnu" "stacks-linux-aarch64" 
    build_target "x86_64-apple-darwin" "stacks-macos-x86_64"
    build_target "aarch64-apple-darwin" "stacks-macos-aarch64"
    build_target "x86_64-pc-windows-gnu" "stacks-windows-x86_64"
    
    print_success "All builds completed!"
    print_status "Binaries are available in the $DIST_DIR/ directory"
}

# Build for current platform only
build_local() {
    print_status "Building for current platform..."
    
    if cargo build --release; then
        mkdir -p "$DIST_DIR"
        cp "$BUILD_DIR/release/$BINARY_NAME" "$DIST_DIR/"
        print_success "Local build completed!"
    else
        print_error "Local build failed"
        exit 1
    fi
}

# Clean build artifacts
clean() {
    print_status "Cleaning build artifacts..."
    cargo clean
    rm -rf "$DIST_DIR"
    print_success "Clean completed!"
}

# Run tests
test() {
    print_status "Running tests..."
    cargo test --all-features
    print_success "Tests completed!"
}

# Show help
show_help() {
    echo "Stacks CLI Build Script"
    echo ""
    echo "Usage: $0 [COMMAND]"
    echo ""
    echo "Commands:"
    echo "  all     Build for all supported platforms (default)"
    echo "  local   Build for current platform only"
    echo "  test    Run tests"
    echo "  clean   Clean build artifacts"
    echo "  help    Show this help message"
    echo ""
    echo "Supported platforms:"
    for target in "${TARGETS[@]}"; do
        echo "  - $target"
    done
}

# Main function
main() {
    check_cargo
    
    case "${1:-all}" in
        "all")
            install_targets
            build_all
            ;;
        "local")
            build_local
            ;;
        "test")
            test
            ;;
        "clean")
            clean
            ;;
        "help"|"-h"|"--help")
            show_help
            ;;
        *)
            print_error "Unknown command: $1"
            echo ""
            show_help
            exit 1
            ;;
    esac
}

main "$@"