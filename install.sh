#!/bin/bash

# Claude Code Stacks CLI Installer
# One-liner installation: curl -sSL https://raw.githubusercontent.com/csaben/claude-code-stacks/main/install.sh | bash

set -e

# Configuration
REPO="csaben/claude-code-stacks"
BINARY_NAME="stacks"
INSTALL_DIR="$HOME/.local/bin"
GITHUB_API="https://api.github.com/repos/$REPO"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

print_header() {
    echo -e "${BLUE}"
    cat << 'EOF'
   _____ _                 _        _____ _             _        
  / ____| |               | |      / ____| |           | |       
 | |    | | __ _ _   _  __| | ___ | (___ | |_ __ _  ___| | _____ 
 | |    | |/ _` | | | |/ _` |/ _ \ \___ \| __/ _` |/ __| |/ / __|
 | |____| | (_| | |_| | (_| |  __/ ____) | || (_| | (__|   <\__ \
  \_____|_|\__,_|\__,_|\__,_|\___||_____/ \__\__,_|\___|_|\_\___/
                                                                 
EOF
    echo -e "${NC}"
    echo -e "${GREEN}Claude Code Stacks - Global Workflow Automation${NC}"
    echo ""
}


# Detect platform and architecture
detect_platform() {
    case "$(uname -s)" in
        Linux*)     PLATFORM=linux;;
        Darwin*)    PLATFORM=macos;;
        CYGWIN*)    PLATFORM=windows;;
        MINGW*)     PLATFORM=windows;;
        *)          PLATFORM=unknown;;
    esac

    case "$(uname -m)" in
        x86_64*)    ARCH=x86_64;;
        arm64*)     ARCH=aarch64;;
        aarch64*)   ARCH=aarch64;;
        *)          ARCH=unknown;;
    esac

    print_status "Detected platform: $PLATFORM-$ARCH"
}

# Check dependencies
check_dependencies() {
    print_status "Checking dependencies..."
    
    local missing_deps=()
    
    # Check for required commands
    for cmd in curl git tmux fzf; do
        if ! command -v "$cmd" >/dev/null 2>&1; then
            missing_deps+=("$cmd")
        fi
    done
    
    if [ ${#missing_deps[@]} -ne 0 ]; then
        print_error "Missing required dependencies: ${missing_deps[*]}"
        print_status "Please install the missing dependencies and try again:"
        
        # Provide platform-specific installation instructions
        case "$PLATFORM" in
            linux)
                print_status "Ubuntu/Debian: sudo apt install curl git tmux fzf"
                print_status "Fedora/RHEL: sudo dnf install curl git tmux fzf"
                print_status "Arch: sudo pacman -S curl git tmux fzf"
                ;;
            macos)
                print_status "macOS: brew install curl git tmux fzf"
                ;;
        esac
        
        exit 1
    fi
    
    # Check for claude CLI
    if ! command -v claude >/dev/null 2>&1; then
        print_warning "Claude CLI not found. Please install it from: https://docs.anthropic.com/en/docs/claude-code/quickstart"
        print_warning "Stacks will still work but MCP functionality will be limited."
    fi
    
    print_success "All dependencies satisfied"
}

# Get the latest release from GitHub
get_latest_release() {
    print_status "Fetching latest release information..."
    
    local release_url="$GITHUB_API/releases/latest"
    local release_info
    
    if ! release_info=$(curl -s "$release_url"); then
        print_error "Failed to fetch release information from GitHub"
        exit 1
    fi
    
    # Extract tag name
    LATEST_VERSION=$(echo "$release_info" | grep '"tag_name":' | sed -E 's/.*"tag_name": *"([^"]+)".*/\1/')
    
    if [ -z "$LATEST_VERSION" ]; then
        print_error "Could not determine latest version"
        exit 1
    fi
    
    print_status "Latest version: $LATEST_VERSION"
}

# Download and install binary
download_and_install() {
    print_status "Downloading stacks binary..."
    
    # Construct download URL
    local binary_filename="${BINARY_NAME}-${PLATFORM}-${ARCH}"
    if [ "$PLATFORM" = "windows" ]; then
        binary_filename="${binary_filename}.exe"
    fi
    
    local download_url="https://github.com/$REPO/releases/download/$LATEST_VERSION/$binary_filename"
    
    # Create install directory if it doesn't exist
    mkdir -p "$INSTALL_DIR"
    
    local target_path="$INSTALL_DIR/$BINARY_NAME"
    if [ "$PLATFORM" = "windows" ]; then
        target_path="${target_path}.exe"
    fi
    
    # Download the binary
    if ! curl -L -o "$target_path" "$download_url"; then
        print_error "Failed to download binary from $download_url"
        
        # Fallback: try to build from source if cargo is available
        if command -v cargo >/dev/null 2>&1; then
            print_status "Attempting to build from source..."
            build_from_source
            return
        fi
        
        exit 1
    fi
    
    # Make executable
    chmod +x "$target_path"
    
    print_success "Installed stacks to $target_path"
}

# Build from source as fallback
build_from_source() {
    print_status "Building stacks from source..."
    
    local temp_dir=$(mktemp -d)
    cd "$temp_dir"
    
    # Clone repository
    print_status "Cloning repository..."
    if ! git clone "https://github.com/$REPO.git" .; then
        print_error "Failed to clone repository"
        exit 1
    fi
    
    # Build with cargo
    print_status "Building with cargo..."
    if ! cargo build --release; then
        print_error "Failed to build from source"
        exit 1
    fi
    
    # Install binary
    cp "target/release/$BINARY_NAME" "$INSTALL_DIR/"
    chmod +x "$INSTALL_DIR/$BINARY_NAME"
    
    # Cleanup
    cd - >/dev/null
    rm -rf "$temp_dir"
    
    print_success "Built and installed stacks from source"
}

# Update PATH if necessary
update_path() {
    if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
        print_warning "$INSTALL_DIR is not in your PATH"
        
        # Detect shell and update appropriate rc file
        local shell_rc=""
        case "$SHELL" in
            */bash) shell_rc="$HOME/.bashrc";;
            */zsh) shell_rc="$HOME/.zshrc";;
            */fish) shell_rc="$HOME/.config/fish/config.fish";;
            *) shell_rc="$HOME/.profile";;
        esac
        
        print_status "Adding $INSTALL_DIR to PATH in $shell_rc"
        echo "export PATH=\"\$PATH:$INSTALL_DIR\"" >> "$shell_rc"
        
        print_warning "Please restart your shell or run: export PATH=\"\$PATH:$INSTALL_DIR\""
    fi
}

# Verify installation
verify_installation() {
    print_status "Verifying installation..."
    
    if [ -x "$INSTALL_DIR/$BINARY_NAME" ]; then
        local version_output
        if version_output=$("$INSTALL_DIR/$BINARY_NAME" --version 2>/dev/null); then
            print_success "Installation verified: $version_output"
        else
            print_warning "Binary installed but version check failed"
        fi
    else
        print_error "Installation failed - binary not found or not executable"
        exit 1
    fi
}

# Main installation function
main() {
    print_header
    print_status "Starting Claude Code Stacks installation..."
    
    detect_platform
    
    if [ "$PLATFORM" = "unknown" ] || [ "$ARCH" = "unknown" ]; then
        print_error "Unsupported platform: $PLATFORM-$ARCH"
        print_status "Supported platforms: linux-x86_64, linux-aarch64, macos-x86_64, macos-aarch64"
        exit 1
    fi
    
    check_dependencies
    get_latest_release
    download_and_install
    update_path
    verify_installation
    
    print_success "Installation complete!"
    print_status ""
    print_status "Quick start:"
    print_status "  stacks          # Discover and checkout stacks"
    print_status "  stacks worktree # Create git worktree with tmux"
    print_status "  stacks sync     # Sync MCP servers from docker-compose"
    print_status ""
    print_status "For more information, visit: https://github.com/$REPO"
}

# Run main function
main "$@"