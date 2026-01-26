# Linux Package Build Scripts

This directory contains scripts for building Linux installer packages.

## Scripts

### `build-all.sh`
Builds all package formats in one command.

**Usage:**
```bash
./scripts/package/build-all.sh
```

**Output:**
- `target/debian/keyrx_<version>_amd64.deb` - Debian/Ubuntu package
- `target/tarball/keyrx-<version>-linux-x86_64.tar.gz` - Universal Linux tarball

### `build-deb.sh`
Builds Debian/Ubuntu .deb package with systemd integration.

**Usage:**
```bash
./scripts/package/build-deb.sh
```

**Features:**
- systemd service file (`/etc/systemd/system/keyrx.service`)
- Automatic dependency management
- Post-install setup (creates log directory, reloads systemd)
- Clean uninstallation with prerm/postrm scripts

**Installation:**
```bash
sudo dpkg -i keyrx_<version>_amd64.deb
sudo apt-get install -f  # Install dependencies if needed
sudo systemctl start keyrx
```

**Package Contents:**
```
/usr/bin/keyrx_compiler          - Config compiler binary
/usr/bin/keyrx_daemon            - Remapping daemon
/usr/share/keyrx/                - Web UI files
/etc/systemd/system/keyrx.service - systemd service
/usr/share/doc/keyrx/            - Documentation
/var/log/keyrx/                  - Log directory (created by postinst)
```

### `build-tarball.sh`
Builds universal tarball with install/uninstall scripts.

**Usage:**
```bash
./scripts/package/build-tarball.sh
```

**Features:**
- Portable install script (works without dpkg)
- Custom install directory support
- systemd integration (if available)
- Clean uninstallation script

**Installation:**
```bash
tar -xzf keyrx-<version>-linux-x86_64.tar.gz
cd keyrx-<version>-linux-x86_64
sudo ./install.sh

# Custom install directory:
sudo INSTALL_DIR=/opt/keyrx ./install.sh
```

**Tarball Contents:**
```
bin/
  keyrx_compiler               - Config compiler
  keyrx_daemon                 - Remapping daemon
share/
  ui/                          - Web UI files
  keyrx.service                - systemd service template
doc/
  README.md                    - Documentation
install.sh                     - Installation script
uninstall.sh                   - Uninstallation script
README.txt                     - Quick start guide
```

## Requirements

### Build Requirements
- Rust 1.70+ (`cargo`)
- Node.js 18+ (`npm`)
- wasm-pack (`cargo install wasm-pack`)
- Standard build tools (`gcc`, `make`)

### Debian Package Additional Requirements
- `dpkg-deb` (usually pre-installed on Debian/Ubuntu)

## Build Process

Both scripts follow this process:

1. **Parse version** from `Cargo.toml`
2. **Build Rust binaries** (`cargo build --release`)
3. **Build WASM module** (`wasm-pack build`)
4. **Build UI bundle** (`npm run build`)
5. **Package files** with appropriate structure
6. **Create installer scripts** (systemd service, install/uninstall)
7. **Generate package** (.deb or .tar.gz)

## Testing Packages

### Test Debian Package

```bash
# Build
./scripts/package/build-deb.sh

# Install in a container (recommended)
docker run -it --rm -v $(pwd):/work ubuntu:22.04
apt-get update && apt-get install -y systemd
dpkg -i /work/target/debian/keyrx_*.deb
systemctl start keyrx

# Or install locally (be careful!)
sudo dpkg -i target/debian/keyrx_*.deb
sudo systemctl start keyrx
curl http://localhost:7777
```

### Test Tarball

```bash
# Build
./scripts/package/build-tarball.sh

# Extract and test
cd /tmp
tar -xzf ~/repos/keyrx/target/tarball/keyrx-*.tar.gz
cd keyrx-*
cat README.txt
sudo ./install.sh

# Verify
which keyrx_daemon
systemctl status keyrx
curl http://localhost:7777

# Uninstall
sudo ./uninstall.sh
```

## Integration with GitHub Actions

These scripts are used by `.github/workflows/release.yml` to automatically build and publish packages when a version tag is pushed:

```bash
git tag -a v0.2.0 -m "Release v0.2.0"
git push origin v0.2.0
```

The workflow will:
1. Build packages on Ubuntu
2. Upload to GitHub Releases
3. Generate installation instructions

See `docs/RELEASE.md` for the complete release process.

## Troubleshooting

### "WASM not available" Error

```bash
# Install wasm-pack
cargo install wasm-pack

# Add wasm32 target
rustup target add wasm32-unknown-unknown

# Rebuild
cd keyrx_ui
npm run build:wasm
```

### "npm: command not found"

```bash
# Install Node.js 18+
curl -fsSL https://deb.nodesource.com/setup_18.x | sudo -E bash -
sudo apt-get install -y nodejs
```

### Build Fails with Permission Error

```bash
# Make scripts executable
chmod +x scripts/package/*.sh

# Check disk space
df -h

# Clean old builds
cargo clean
rm -rf target/debian target/tarball
```

### Package Installation Fails

**Debian:**
```bash
# Check dependencies
dpkg -I keyrx_*.deb

# Install missing dependencies
sudo apt-get install -f
```

**Tarball:**
```bash
# Check for conflicts
which keyrx_daemon  # Should be empty before install

# Install with verbose output
sudo bash -x ./install.sh
```

## Security Considerations

### File Permissions
- Binaries: `755` (executable by all, writable by owner)
- Config files: `644` (readable by all, writable by owner)
- Log directory: `755` (created by postinst/install.sh)

### systemd Security
The service file includes security hardening:
- `NoNewPrivileges=true` - Prevents privilege escalation
- `ProtectSystem=strict` - Read-only filesystem except /var/log/keyrx
- `ProtectHome=true` - Home directories inaccessible
- Runs as root (required for keyboard access via evdev)

### SUID/SGID
No SUID or SGID bits are set. The daemon must run as root for keyboard access.

## Future Enhancements

- [ ] AppImage support (self-contained executable)
- [ ] Snap package (Ubuntu Software store)
- [ ] Flatpak package (Flathub)
- [ ] RPM package (Fedora/RHEL)
- [ ] AUR package (Arch Linux)
- [ ] Docker image
- [ ] Homebrew formula (macOS)

## References

- [Debian Policy Manual](https://www.debian.org/doc/debian-policy/)
- [systemd Service Files](https://www.freedesktop.org/software/systemd/man/systemd.service.html)
- [Linux Filesystem Hierarchy](https://refspecs.linuxfoundation.org/FHS_3.0/fhs/index.html)
