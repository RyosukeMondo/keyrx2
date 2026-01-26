#!/usr/bin/env bash
# Build tarball with install script for keyrx
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

# Parse version from Cargo.toml
VERSION=$(grep '^version' "$PROJECT_ROOT/Cargo.toml" | head -1 | sed 's/version = "\(.*\)"/\1/')
ARCH="x86_64"
PACKAGE_NAME="keyrx-${VERSION}-linux-${ARCH}"
BUILD_DIR="$PROJECT_ROOT/target/tarball"
TAR_DIR="$BUILD_DIR/$PACKAGE_NAME"

echo "Building tarball: $PACKAGE_NAME"

# Clean and create build directory
rm -rf "$TAR_DIR"
mkdir -p "$TAR_DIR"/{bin,share,doc}

# Build binaries in release mode
echo "Building binaries..."
cd "$PROJECT_ROOT"
cargo build --release --bin keyrx_compiler --bin keyrx_daemon

# Build WASM and UI
echo "Building UI..."
cd "$PROJECT_ROOT/keyrx_ui"
npm ci
npm run build:wasm
npm run build

# Copy binaries
echo "Packaging files..."
cp "$PROJECT_ROOT/target/release/keyrx_compiler" "$TAR_DIR/bin/"
cp "$PROJECT_ROOT/target/release/keyrx_daemon" "$TAR_DIR/bin/"
chmod +x "$TAR_DIR/bin/"*

# Copy UI bundle
cp -r "$PROJECT_ROOT/keyrx_ui/dist" "$TAR_DIR/share/ui"

# Copy documentation
cp "$PROJECT_ROOT/README.md" "$TAR_DIR/doc/" 2>/dev/null || true

# Copy desktop files (optional)
if [ -d "$PROJECT_ROOT/keyrx_tray" ]; then
    echo "Including desktop integration files..."
    mkdir -p "$TAR_DIR/share/desktop"

    cp "$PROJECT_ROOT/keyrx_tray/keyrx.desktop" "$TAR_DIR/share/desktop/" 2>/dev/null || true
    cp "$PROJECT_ROOT/keyrx_tray/keyrx-config.desktop" "$TAR_DIR/share/desktop/" 2>/dev/null || true

    # Copy tray script
    if [ -f "$PROJECT_ROOT/keyrx_tray/keyrx-tray.py" ]; then
        cp "$PROJECT_ROOT/keyrx_tray/keyrx-tray.py" "$TAR_DIR/bin/keyrx-tray"
        chmod +x "$TAR_DIR/bin/keyrx-tray"
    fi
fi

# Create systemd service template
cat > "$TAR_DIR/share/keyrx.service" <<'EOF'
[Unit]
Description=KeyRx Keyboard Remapping Daemon
After=network.target

[Service]
Type=simple
ExecStart=/usr/local/bin/keyrx_daemon
Restart=on-failure
RestartSec=5s
User=root
NoNewPrivileges=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/var/log/keyrx

[Install]
WantedBy=multi-user.target
EOF

# Create install script
cat > "$TAR_DIR/install.sh" <<'INSTALL_EOF'
#!/usr/bin/env bash
# KeyRx installation script
set -euo pipefail

INSTALL_DIR="${INSTALL_DIR:-/usr/local}"
SYSTEMD_DIR="/etc/systemd/system"

# Check for root
if [[ $EUID -ne 0 ]]; then
   echo "This script must be run as root (use sudo)"
   exit 1
fi

echo "Installing KeyRx to $INSTALL_DIR..."

# Install binaries
install -m 755 bin/keyrx_compiler "$INSTALL_DIR/bin/"
install -m 755 bin/keyrx_daemon "$INSTALL_DIR/bin/"

# Install system tray (if available)
if [ -f bin/keyrx-tray ]; then
    install -m 755 bin/keyrx-tray "$INSTALL_DIR/bin/"
fi

# Install UI files
mkdir -p "$INSTALL_DIR/share/keyrx"
cp -r share/ui/* "$INSTALL_DIR/share/keyrx/"

# Install desktop files (if available)
if [ -d share/desktop ]; then
    mkdir -p /usr/share/applications
    cp share/desktop/*.desktop /usr/share/applications/ 2>/dev/null || true
fi

# Create log directory
mkdir -p /var/log/keyrx
chmod 755 /var/log/keyrx

# Install systemd service (if systemd is available)
if command -v systemctl &> /dev/null; then
    echo "Installing systemd service..."

    # Update ExecStart path in service file
    sed "s|/usr/local/bin|$INSTALL_DIR/bin|g" share/keyrx.service > "$SYSTEMD_DIR/keyrx.service"

    systemctl daemon-reload

    echo ""
    echo "To start KeyRx:"
    echo "  systemctl start keyrx"
    echo ""
    echo "To enable at boot:"
    echo "  systemctl enable keyrx"
else
    echo "Systemd not found. Manual service setup required."
fi

echo ""
echo "✓ KeyRx installed successfully!"
echo ""
echo "Binaries installed to: $INSTALL_DIR/bin/"
echo "Web UI files: $INSTALL_DIR/share/keyrx/"
echo "Web UI will be available at: http://localhost:9867"
echo ""
echo "Usage:"
echo "  keyrx_compiler <config.rhai> <output.krx>"
echo "  keyrx_daemon [--config <config.krx>]"
echo ""
if command -v keyrx-tray &> /dev/null; then
    echo "System tray installed. To auto-start:"
    echo "  mkdir -p ~/.config/autostart"
    echo "  cp /usr/share/applications/keyrx-tray.desktop ~/.config/autostart/"
    echo ""
    echo "Dependencies: python3-gi gir1.2-appindicator3-0.1 python3-requests"
fi
INSTALL_EOF
chmod +x "$TAR_DIR/install.sh"

# Create uninstall script
cat > "$TAR_DIR/uninstall.sh" <<'UNINSTALL_EOF'
#!/usr/bin/env bash
# KeyRx uninstallation script
set -euo pipefail

INSTALL_DIR="${INSTALL_DIR:-/usr/local}"

# Check for root
if [[ $EUID -ne 0 ]]; then
   echo "This script must be run as root (use sudo)"
   exit 1
fi

echo "Uninstalling KeyRx..."

# Stop and disable systemd service
if command -v systemctl &> /dev/null; then
    if systemctl is-active --quiet keyrx; then
        systemctl stop keyrx
    fi
    if systemctl is-enabled --quiet keyrx 2>/dev/null; then
        systemctl disable keyrx
    fi
    rm -f /etc/systemd/system/keyrx.service
    systemctl daemon-reload
fi

# Remove binaries
rm -f "$INSTALL_DIR/bin/keyrx_compiler"
rm -f "$INSTALL_DIR/bin/keyrx_daemon"

# Remove UI files
rm -rf "$INSTALL_DIR/share/keyrx"

# Remove log directory (optional, asks user)
if [[ -d /var/log/keyrx ]]; then
    read -p "Remove log directory /var/log/keyrx? [y/N] " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        rm -rf /var/log/keyrx
    fi
fi

echo "✓ KeyRx uninstalled successfully!"
UNINSTALL_EOF
chmod +x "$TAR_DIR/uninstall.sh"

# Create README
cat > "$TAR_DIR/README.txt" <<README_EOF
KeyRx $VERSION - Linux Installation Package
============================================

Installation:
  sudo ./install.sh

Uninstallation:
  sudo ./uninstall.sh

Custom install directory:
  sudo INSTALL_DIR=/opt/keyrx ./install.sh

After installation:
  - Start: sudo systemctl start keyrx
  - Enable at boot: sudo systemctl enable keyrx
  - Web UI: http://localhost:7777

For documentation, see doc/README.md or visit:
https://github.com/RyosukeMondo/keyrx
README_EOF

# Create the tarball
echo "Creating tarball..."
cd "$BUILD_DIR"
tar -czf "$PACKAGE_NAME.tar.gz" "$PACKAGE_NAME"

echo "✓ Tarball created: $BUILD_DIR/$PACKAGE_NAME.tar.gz"
echo ""
echo "Extract and install with:"
echo "  tar -xzf $PACKAGE_NAME.tar.gz"
echo "  cd $PACKAGE_NAME"
echo "  sudo ./install.sh"
