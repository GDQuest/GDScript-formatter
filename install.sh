#!/usr/bin/env bash

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# GitHub repository information
REPO_OWNER="GDQuest"
REPO_NAME="godot-gdscript-formatter-tree-sitter"
BINARY_NAME="gdscript-formatter"

# Print colored messages
print_info() {
    echo -e "${BLUE}[INFO]${NC} $1" >&2
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1" >&2
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1" >&2
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1" >&2
}

# Detect operating system and architecture
detect_platform() {
    local os=""
    local arch=""

    # Detect OS
    case "$(uname -s)" in
        Linux*)
            os="linux"
            ;;
        Darwin*)
            os="macos"
            ;;
        CYGWIN*|MINGW*|MSYS*)
            os="windows"
            ;;
        *)
            print_error "Unsupported operating system: $(uname -s)"
            exit 1
            ;;
    esac

    # Detect architecture
    case "$(uname -m)" in
        x86_64|amd64)
            arch="x86_64"
            ;;
        aarch64|arm64)
            arch="aarch64"
            ;;
        *)
            print_error "Unsupported architecture: $(uname -m)"
            exit 1
            ;;
    esac

    echo "${os}-${arch}"
}

# Check if a command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Install dependencies
install_dependencies() {
    print_info "Checking for required dependencies..."

    local missing_deps=()

    # Check for curl or wget
    if ! command_exists curl && ! command_exists wget; then
        missing_deps+=("curl")
    fi

    # Check for unzip (needed for archive extraction)
    if ! command_exists unzip; then
        missing_deps+=("unzip")
    fi

    if [ ${#missing_deps[@]} -eq 0 ]; then
        print_success "All required dependencies are installed"
        return 0
    fi

    print_warning "Missing dependencies: ${missing_deps[*]}"
    print_info "Attempting to install missing dependencies..."

    # Detect package manager and install dependencies
    if command_exists apt-get; then
        print_info "Using apt-get to install dependencies..."
        sudo apt-get update
        sudo apt-get install -y "${missing_deps[@]}"
    elif command_exists yum; then
        print_info "Using yum to install dependencies..."
        sudo yum install -y "${missing_deps[@]}"
    elif command_exists dnf; then
        print_info "Using dnf to install dependencies..."
        sudo dnf install -y "${missing_deps[@]}"
    elif command_exists pacman; then
        print_info "Using pacman to install dependencies..."
        sudo pacman -S --noconfirm "${missing_deps[@]}"
    elif command_exists brew; then
        print_info "Using Homebrew to install dependencies..."
        brew install "${missing_deps[@]}"
    elif command_exists apk; then
        print_info "Using apk to install dependencies..."
        sudo apk add "${missing_deps[@]}"
    else
        print_error "Could not detect package manager. Please install manually: ${missing_deps[*]}"
        exit 1
    fi

    print_success "Dependencies installed successfully"
}

# Get the latest release version from GitHub
get_latest_version() {
    print_info "Fetching latest release version..."

    local api_url="https://api.github.com/repos/${REPO_OWNER}/${REPO_NAME}/releases/latest"
    local version=""

    if command_exists curl; then
        version=$(curl -sL "${api_url}" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')
    elif command_exists wget; then
        version=$(wget -qO- "${api_url}" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')
    else
        print_error "Neither curl nor wget is available"
        exit 1
    fi

    if [ -z "$version" ]; then
        print_error "Failed to fetch latest version"
        exit 1
    fi

    print_success "Latest version: ${version}"
    echo "$version"
}

# Download and extract the binary
download_and_install() {
    local platform="$1"
    local version="$2"

    print_info "Downloading GDScript Formatter for ${platform}..."

    # Parse platform
    local os_part="${platform%-*}"
    local arch_part="${platform#*-}"

    # Map platform to release asset naming convention based on release.yml
    local asset_name=""
    case "$os_part-$arch_part" in
        linux-x86_64)
            asset_name="gdscript-formatter-${version}-linux-x86_64"
            ;;
        linux-aarch64)
            asset_name="gdscript-formatter-${version}-linux-aarch64"
            ;;
        macos-x86_64)
            asset_name="gdscript-formatter-${version}-macos-x86_64"
            ;;
        macos-aarch64)
            asset_name="gdscript-formatter-${version}-macos-aarch64"
            ;;
        windows-x86_64)
            asset_name="gdscript-formatter-${version}-windows-x86_64.exe"
            ;;
        windows-aarch64)
            asset_name="gdscript-formatter-${version}-windows-aarch64.exe"
            ;;
        *)
            print_error "Unsupported platform: ${platform}"
            exit 1
            ;;
    esac

    local download_url="https://github.com/${REPO_OWNER}/${REPO_NAME}/releases/download/${version}/${asset_name}.zip"
    local temp_dir=$(mktemp -d)
    local download_file="${temp_dir}/${asset_name}.zip"

    print_info "Download URL: ${download_url}"

    # Download the file
    if command_exists curl; then
        if ! curl -L -f -o "${download_file}" "${download_url}"; then
            print_error "Failed to download ${asset_name}.zip"
            print_error "URL: ${download_url}"
            rm -rf "${temp_dir}"
            exit 1
        fi
    elif command_exists wget; then
        if ! wget -O "${download_file}" "${download_url}"; then
            print_error "Failed to download ${asset_name}.zip"
            print_error "URL: ${download_url}"
            rm -rf "${temp_dir}"
            exit 1
        fi
    fi

    print_success "Downloaded successfully"

    # Extract the archive
    print_info "Extracting archive..."
    cd "${temp_dir}"

    if ! unzip -q "${download_file}"; then
        print_error "Failed to extract archive"
        rm -rf "${temp_dir}"
        exit 1
    fi

    # Find the binary (it should be the asset_name file)
    local binary_path=""
    if [ -f "${asset_name}" ]; then
        binary_path="${temp_dir}/${asset_name}"
    else
        print_error "Binary not found in archive: ${asset_name}"
        print_info "Contents of archive:"
        ls -la "${temp_dir}"
        rm -rf "${temp_dir}"
        exit 1
    fi

    # Determine installation directory
    local install_dir=""
    if [ -w "/usr/local/bin" ]; then
        install_dir="/usr/local/bin"
    elif [ -w "$HOME/.local/bin" ]; then
        install_dir="$HOME/.local/bin"
    elif [ -w "$HOME/bin" ]; then
        install_dir="$HOME/bin"
    else
        # Create ~/.local/bin if it doesn't exist
        install_dir="$HOME/.local/bin"
        mkdir -p "$install_dir"
    fi

    # Determine final binary name (remove .exe suffix if on Unix)
    local final_binary_name="${BINARY_NAME}"
    if [[ "$os_part" == "windows" ]]; then
        final_binary_name="${BINARY_NAME}.exe"
    fi

    # Install the binary
    print_info "Installing to ${install_dir}/${final_binary_name}..."

    if [ -w "$install_dir" ]; then
        cp "${binary_path}" "${install_dir}/${final_binary_name}"
        chmod +x "${install_dir}/${final_binary_name}"
    else
        sudo cp "${binary_path}" "${install_dir}/${final_binary_name}"
        sudo chmod +x "${install_dir}/${final_binary_name}"
    fi

    # Clean up
    rm -rf "${temp_dir}"

    print_success "Installation complete!"

    # Check if install directory is in PATH
    if [[ ":$PATH:" != *":${install_dir}:"* ]]; then
        print_warning "${install_dir} is not in your PATH"
        print_info "Add the following line to your shell configuration file (~/.bashrc, ~/.zshrc, etc.):"
        echo -e "\n    export PATH=\"\$PATH:${install_dir}\"\n" >&2
        print_info "Then reload your shell configuration:"
        echo -e "    source ~/.bashrc  # or source ~/.zshrc\n" >&2
    fi

    echo "" >&2
    print_success "GDScript Formatter ${version} installed successfully!"
    echo "" >&2
    print_info "You can now use it by running:"
    echo -e "    ${BINARY_NAME} --help" >&2
    echo "" >&2
    print_info "To format a file:"
    echo -e "    ${BINARY_NAME} path/to/file.gd" >&2
    echo "" >&2
    print_info "To lint a file:"
    echo -e "    ${BINARY_NAME} lint path/to/file.gd" >&2
    echo "" >&2
    print_info "For more information, visit:"
    echo -e "    https://www.gdquest.com/library/gdscript_formatter/" >&2
    echo -e "    https://github.com/${REPO_OWNER}/${REPO_NAME}" >&2
    echo "" >&2
}

# Main installation flow
main() {
    echo "" >&2
    echo "=========================================" >&2
    echo "  GDScript Formatter Installation" >&2
    echo "=========================================" >&2
    echo "" >&2

    # Install dependencies
    install_dependencies

    # Detect platform
    print_info "Detecting platform..."
    PLATFORM=$(detect_platform)
    print_success "Platform detected: ${PLATFORM}"

    # Get latest version
    VERSION=$(get_latest_version)

    # Download and install
    download_and_install "$PLATFORM" "$VERSION"
}

# Run main function
main
