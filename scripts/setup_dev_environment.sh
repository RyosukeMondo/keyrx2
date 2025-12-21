#!/bin/bash
# Development Environment Setup for AI-Coding-Agent Autonomous Development
# This script installs all tools needed for autonomous AI development

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Get script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Log functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $*"
}

log_success() {
    echo -e "${GREEN}[✓]${NC} $*"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $*"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $*"
}

# Check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Check if running as root
check_not_root() {
    if [ "$EUID" -eq 0 ]; then
        log_error "Do not run this script as root. It will prompt for sudo when needed."
        exit 1
    fi
}

# Check prerequisites
check_prerequisites() {
    log_info "Checking prerequisites..."

    # Check Rust
    if ! command_exists rustc; then
        log_error "Rust not found. Install from: https://rustup.rs"
        exit 1
    fi

    local rust_version=$(rustc --version | grep -oP '\d+\.\d+\.\d+' | head -1)
    log_success "Rust found: $rust_version"

    # Check Cargo
    if ! command_exists cargo; then
        log_error "Cargo not found. Install Rust from: https://rustup.rs"
        exit 1
    fi
    log_success "Cargo found: $(cargo --version)"

    # Check Git
    if ! command_exists git; then
        log_error "Git not found. Install with: sudo apt-get install git"
        exit 1
    fi
    log_success "Git found: $(git --version)"

    # Check if in git repository
    if ! git -C "$PROJECT_ROOT" rev-parse --git-dir >/dev/null 2>&1; then
        log_error "Not in a git repository. Please initialize git first."
        exit 1
    fi
    log_success "Git repository detected"
}

# Install system dependencies (requires sudo)
install_system_dependencies() {
    log_info "Installing system dependencies (may require sudo password)..."

    local packages=(
        build-essential    # C/C++ compiler (needed for some Rust crates)
        pkg-config         # Package config tool
        libssl-dev         # OpenSSL development files
        libevdev-dev       # evdev development files (for Linux platform)
        libudev-dev        # udev development files
        bats               # Bash Automated Testing System
        jq                 # JSON processor (for parsing script output)
    )

    # Check which packages are missing
    local missing_packages=()
    for pkg in "${packages[@]}"; do
        if ! dpkg -l | grep -q "^ii  $pkg "; then
            missing_packages+=("$pkg")
        fi
    done

    if [ ${#missing_packages[@]} -eq 0 ]; then
        log_success "All system dependencies already installed"
        return 0
    fi

    log_info "Missing packages: ${missing_packages[*]}"
    log_info "Installing with apt-get (requires sudo)..."

    sudo apt-get update
    sudo apt-get install -y "${missing_packages[@]}"

    log_success "System dependencies installed"
}

# Install Rust toolchain components
install_rust_components() {
    log_info "Installing Rust toolchain components..."

    # Check for nightly toolchain (needed for benchmarks)
    if ! rustup toolchain list | grep -q "nightly"; then
        log_info "Installing nightly toolchain (for benchmarks)..."
        rustup toolchain install nightly
        log_success "Nightly toolchain installed"
    else
        log_success "Nightly toolchain already installed"
    fi

    # Install rustfmt and clippy
    local components=("rustfmt" "clippy")
    for component in "${components[@]}"; do
        if ! rustup component list | grep -q "^${component}.*installed"; then
            log_info "Installing $component..."
            rustup component add "$component"
            log_success "$component installed"
        else
            log_success "$component already installed"
        fi
    done
}

# Install Cargo development tools
install_cargo_tools() {
    log_info "Installing Cargo development tools..."

    # List of tools to install
    declare -A tools=(
        ["cargo-watch"]="Continuous build on file changes"
        ["cargo-tarpaulin"]="Code coverage analysis"
        ["cargo-fuzz"]="Fuzzing support"
        ["wasm-pack"]="WASM compilation"
    )

    for tool in "${!tools[@]}"; do
        if command_exists "$tool"; then
            log_success "$tool already installed (${tools[$tool]})"
        else
            log_info "Installing $tool (${tools[$tool]})..."
            log_warn "This may take several minutes..."

            # Install with cargo
            if cargo install "$tool" --quiet; then
                log_success "$tool installed"
            else
                log_error "Failed to install $tool"
                exit 1
            fi
        fi
    done
}

# Setup Git hooks
setup_git_hooks() {
    log_info "Setting up Git hooks..."

    if [ -x "$SCRIPT_DIR/setup_hooks.sh" ]; then
        if "$SCRIPT_DIR/setup_hooks.sh"; then
            log_success "Git hooks installed"
        else
            log_error "Failed to install Git hooks"
            exit 1
        fi
    else
        log_warn "setup_hooks.sh not found or not executable"
    fi
}

# Verify installation
verify_installation() {
    log_info "Verifying installation..."

    local all_good=true

    # Check Rust tools
    local rust_tools=("cargo" "rustc" "rustfmt" "clippy-driver" "cargo-watch" "cargo-tarpaulin" "cargo-fuzz" "wasm-pack")
    for tool in "${rust_tools[@]}"; do
        if command_exists "$tool"; then
            log_success "$tool: ✓"
        else
            log_error "$tool: ✗ (not found)"
            all_good=false
        fi
    done

    # Check system tools
    local sys_tools=("git" "jq" "bats")
    for tool in "${sys_tools[@]}"; do
        if command_exists "$tool"; then
            log_success "$tool: ✓"
        else
            log_error "$tool: ✗ (not found)"
            all_good=false
        fi
    done

    # Check git hooks
    if [ -f "$PROJECT_ROOT/.git/hooks/pre-commit" ] && [ -x "$PROJECT_ROOT/.git/hooks/pre-commit" ]; then
        log_success "Git pre-commit hook: ✓"
    else
        log_warn "Git pre-commit hook: ✗ (not installed or not executable)"
        all_good=false
    fi

    if [ "$all_good" = true ]; then
        log_success "All tools verified!"
        return 0
    else
        log_error "Some tools are missing. Please check errors above."
        return 1
    fi
}

# Create development tools summary
create_summary() {
    log_info "Creating development tools summary..."

    cat > "$PROJECT_ROOT/.dev-tools.md" << 'EOF'
# Development Tools Setup

## Installed Tools

### Rust Toolchain
- **rustc**: Rust compiler (stable)
- **rustc (nightly)**: Nightly toolchain for benchmarks
- **cargo**: Rust package manager
- **rustfmt**: Code formatter
- **clippy**: Linter

### Cargo Development Tools
- **cargo-watch**: Continuous build on file changes
  - Usage: `cargo watch -x build` or `scripts/build.sh --watch`
- **cargo-tarpaulin**: Code coverage analysis
  - Usage: `cargo tarpaulin --workspace` or `scripts/verify.sh`
- **cargo-fuzz**: Fuzzing support
  - Usage: `cd keyrx_core/fuzz && cargo fuzz run fuzz_target_1`
- **wasm-pack**: WASM compilation
  - Usage: `wasm-pack build keyrx_core --target web`

### System Tools
- **git**: Version control
- **jq**: JSON processor for parsing script output
- **bats**: Bash Automated Testing System

### Build Dependencies
- **build-essential**: C/C++ compiler
- **pkg-config**: Package config tool
- **libssl-dev**: OpenSSL development files
- **libevdev-dev**: evdev development files (Linux keyboard input)
- **libudev-dev**: udev development files

## Git Hooks
- **pre-commit**: Runs `scripts/verify.sh --quiet` before every commit
  - Checks: clippy, rustfmt, tests, coverage
  - Blocks commit if quality checks fail

## Quick Commands

```bash
# Build workspace
make build

# Run all quality checks
make verify

# Run tests
make test

# Launch daemon
make launch

# Clean build artifacts
make clean

# Reinstall tools (if needed)
make setup
```

## Troubleshooting

### cargo-tarpaulin fails on coverage
- Install libssl-dev: `sudo apt-get install libssl-dev`
- Rebuild: `cargo clean && cargo install cargo-tarpaulin --force`

### cargo-watch not found
- Reinstall: `cargo install cargo-watch --force`

### Pre-commit hook not running
- Reinstall hooks: `scripts/setup_hooks.sh`
- Verify: `ls -la .git/hooks/pre-commit`

### BATS tests fail
- Install BATS: `sudo apt-get install bats`
- Verify: `bats --version`

## AI Agent Notes

All tools are CLI-based and automation-friendly:
- Scripts output structured logs with status markers
- JSON mode available: `scripts/build.sh --json`
- Exit codes: 0 = success, 1 = failure, 2 = warning
- Pre-commit hooks enforce quality automatically
- All operations are deterministic and reproducible

See `.claude/CLAUDE.md` for complete AI agent development guide.
EOF

    log_success "Development tools summary created: .dev-tools.md"
}

# Test basic workflow
test_basic_workflow() {
    log_info "Testing basic development workflow..."

    cd "$PROJECT_ROOT"

    # Test build
    log_info "Testing: make build"
    if make build >/dev/null 2>&1; then
        log_success "Build: ✓"
    else
        log_error "Build: ✗ (failed)"
        return 1
    fi

    # Test that scripts are executable
    local scripts=("build.sh" "verify.sh" "test.sh" "launch.sh" "setup_hooks.sh")
    for script in "${scripts[@]}"; do
        if [ -x "$SCRIPT_DIR/$script" ]; then
            log_success "Script $script: ✓ (executable)"
        else
            log_error "Script $script: ✗ (not executable)"
            return 1
        fi
    done

    log_success "Basic workflow test passed"
}

# Main setup function
main() {
    echo ""
    echo "╔════════════════════════════════════════════════════════════════╗"
    echo "║  KeyRx Development Environment Setup                          ║"
    echo "║  Setting up tools for AI-Coding-Agent autonomous development  ║"
    echo "╚════════════════════════════════════════════════════════════════╝"
    echo ""

    check_not_root
    check_prerequisites

    echo ""
    log_info "This script will install:"
    log_info "  - System dependencies (requires sudo)"
    log_info "  - Rust toolchain components (nightly, rustfmt, clippy)"
    log_info "  - Cargo development tools (cargo-watch, cargo-tarpaulin, cargo-fuzz, wasm-pack)"
    log_info "  - Git pre-commit hooks"
    echo ""

    # Ask for confirmation
    read -p "Continue? (y/N) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        log_warn "Setup cancelled by user"
        exit 0
    fi

    echo ""
    install_system_dependencies
    echo ""
    install_rust_components
    echo ""
    install_cargo_tools
    echo ""
    setup_git_hooks
    echo ""
    verify_installation
    echo ""
    create_summary
    echo ""
    test_basic_workflow
    echo ""

    echo "╔════════════════════════════════════════════════════════════════╗"
    echo "║  ✓ Development Environment Setup Complete!                    ║"
    echo "╚════════════════════════════════════════════════════════════════╝"
    echo ""
    log_success "All tools installed and verified"
    log_success "Development tools summary: .dev-tools.md"
    log_success "AI agent guide: .claude/CLAUDE.md"
    echo ""
    log_info "Next steps:"
    log_info "  1. Review .dev-tools.md for tool usage"
    log_info "  2. Run 'make build' to build the workspace"
    log_info "  3. Run 'make verify' to check code quality"
    log_info "  4. Start development!"
    echo ""
}

# Run main function
main "$@"
