#!/usr/bin/env bash
# Build Debian package for keyrx
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

# Parse version from Cargo.toml
VERSION=$(grep '^version' "$PROJECT_ROOT/Cargo.toml" | head -1 | sed 's/version = "\(.*\)"/\1/')
ARCH="amd64"
PACKAGE_NAME="keyrx_${VERSION}_${ARCH}"
BUILD_DIR="$PROJECT_ROOT/target/debian"
DEB_DIR="$BUILD_DIR/$PACKAGE_NAME"

echo "Building Debian package: $PACKAGE_NAME"

# Clean and create build directory
rm -rf "$DEB_DIR"
mkdir -p "$DEB_DIR"/{DEBIAN,usr/bin,usr/share/keyrx,etc/systemd/system,usr/share/doc/keyrx}

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
cp "$PROJECT_ROOT/target/release/keyrx_compiler" "$DEB_DIR/usr/bin/"
cp "$PROJECT_ROOT/target/release/keyrx_daemon" "$DEB_DIR/usr/bin/"
chmod +x "$DEB_DIR/usr/bin/keyrx_compiler" "$DEB_DIR/usr/bin/keyrx_daemon"

# Copy UI bundle
cp -r "$PROJECT_ROOT/keyrx_ui/dist"/* "$DEB_DIR/usr/share/keyrx/"

# Create systemd service file
cat > "$DEB_DIR/etc/systemd/system/keyrx.service" <<'EOF'
[Unit]
Description=KeyRx Keyboard Remapping Daemon
After=network.target

[Service]
Type=simple
ExecStart=/usr/bin/keyrx_daemon
Restart=on-failure
RestartSec=5s
# Run as root for keyboard access
User=root
# Security hardening
NoNewPrivileges=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/var/log/keyrx

[Install]
WantedBy=multi-user.target
EOF

# Copy documentation
cat > "$DEB_DIR/usr/share/doc/keyrx/README" <<EOF
KeyRx - Advanced Keyboard Remapping
====================================

Version: $VERSION

KeyRx provides advanced keyboard remapping with layers, tap-hold,
and conditional mappings for Linux.

Usage:
  keyrx_compiler <config.rhai> <output.krx>
  keyrx_daemon [--config <config.krx>]

Web UI available at: http://localhost:7777

For more information, visit:
https://github.com/RyosukeMondo/keyrx

License: AGPL-3.0-or-later
EOF

cp "$PROJECT_ROOT/README.md" "$DEB_DIR/usr/share/doc/keyrx/" 2>/dev/null || true

# Copy desktop files (optional, include if available)
if [ -d "$PROJECT_ROOT/keyrx_tray" ]; then
    echo "Including desktop integration files..."

    # Copy .desktop files
    cp "$PROJECT_ROOT/keyrx_tray/keyrx.desktop" "$DEB_DIR/usr/share/applications/" 2>/dev/null || true
    cp "$PROJECT_ROOT/keyrx_tray/keyrx-config.desktop" "$DEB_DIR/usr/share/applications/" 2>/dev/null || true

    # Copy tray script (optional)
    if [ -f "$PROJECT_ROOT/keyrx_tray/keyrx-tray.py" ]; then
        cp "$PROJECT_ROOT/keyrx_tray/keyrx-tray.py" "$DEB_DIR/usr/local/bin/keyrx-tray"
        chmod +x "$DEB_DIR/usr/local/bin/keyrx-tray"
        echo "  ✓ System tray included (requires python3-gi, gir1.2-appindicator3-0.1)"
    fi
fi

# Create DEBIAN/control
cat > "$DEB_DIR/DEBIAN/control" <<EOF
Package: keyrx
Version: $VERSION
Section: utils
Priority: optional
Architecture: $ARCH
Maintainer: RyosukeMondo <ryosukemondo@users.noreply.github.com>
Description: Advanced keyboard remapping with layers and tap-hold
 KeyRx is a keyboard remapping tool that supports:
 - Multiple layers with conditional activation
 - Tap-hold dual-function keys
 - Fast DFA-based state machine
 - Web-based configuration UI
 - Systemd service integration
Homepage: https://github.com/RyosukeMondo/keyrx
Depends: libc6 (>= 2.31), systemd
Recommends: bash
Suggests: python3-gi, gir1.2-appindicator3-0.1, python3-requests
EOF

# Create postinst script
cat > "$DEB_DIR/DEBIAN/postinst" <<'EOF'
#!/bin/bash
set -e

echo "Setting up KeyRx..."

# Create log directory
mkdir -p /var/log/keyrx
chmod 755 /var/log/keyrx

# Reload systemd
systemctl daemon-reload

echo "KeyRx installed successfully!"
echo ""
echo "To start the daemon:"
echo "  sudo systemctl start keyrx"
echo ""
echo "To enable at boot:"
echo "  sudo systemctl enable keyrx"
echo ""
echo "Web UI will be available at http://localhost:9867"
echo ""
if command -v keyrx-tray &> /dev/null; then
    echo "System tray is available. To install for your user:"
    echo "  mkdir -p ~/.config/autostart"
    echo "  cp /usr/share/applications/keyrx-tray.desktop ~/.config/autostart/"
    echo "  keyrx-tray &"
    echo ""
    echo "Dependencies for system tray:"
    echo "  sudo apt-get install python3-gi gir1.2-appindicator3-0.1 python3-requests"
fi

exit 0
EOF
chmod +x "$DEB_DIR/DEBIAN/postinst"

# Create prerm script
cat > "$DEB_DIR/DEBIAN/prerm" <<'EOF'
#!/bin/bash
set -e

# Stop service if running
if systemctl is-active --quiet keyrx; then
    systemctl stop keyrx
fi

exit 0
EOF
chmod +x "$DEB_DIR/DEBIAN/prerm"

# Create postrm script
cat > "$DEB_DIR/DEBIAN/postrm" <<'EOF'
#!/bin/bash
set -e

# Reload systemd after removal
systemctl daemon-reload

exit 0
EOF
chmod +x "$DEB_DIR/DEBIAN/postrm"

# Calculate installed size (in KB)
INSTALLED_SIZE=$(du -sk "$DEB_DIR" | cut -f1)
echo "Installed-Size: $INSTALLED_SIZE" >> "$DEB_DIR/DEBIAN/control"

# Build the .deb package
echo "Creating .deb package..."
dpkg-deb --build --root-owner-group "$DEB_DIR"

# Move to target directory
mv "$DEB_DIR.deb" "$BUILD_DIR/keyrx_${VERSION}_${ARCH}.deb"

echo "✓ Debian package created: $BUILD_DIR/keyrx_${VERSION}_${ARCH}.deb"
echo ""
echo "Install with:"
echo "  sudo dpkg -i $BUILD_DIR/keyrx_${VERSION}_${ARCH}.deb"
echo "  sudo apt-get install -f  # Install dependencies if needed"
