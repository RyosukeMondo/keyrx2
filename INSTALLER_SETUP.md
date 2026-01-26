# Linux Installer Setup - Complete Guide

This document summarizes the Linux installer system for KeyRx.

## What Was Created

### 1. Package Build Scripts

**Location:** `scripts/package/`

- **`build-deb.sh`** - Creates Debian/Ubuntu .deb package
  - Full systemd integration
  - Automatic dependency management
  - Post-install setup scripts

- **`build-tarball.sh`** - Creates universal tarball
  - Works on any Linux distribution
  - Includes install/uninstall scripts
  - Custom install directory support

- **`build-all.sh`** - Builds both packages at once

### 2. GitHub Actions Integration

**Updated:** `.github/workflows/release.yml`

Changes made:
- Fixed `keyrx_ui_v2` → `keyrx_ui` references
- Added .deb package building
- Added tarball building
- Enhanced release notes generation
- Automatic upload to GitHub Releases

### 3. Documentation

- **`docs/RELEASE.md`** - Complete release process guide
- **`scripts/package/README.md`** - Package build documentation
- **`INSTALLER_SETUP.md`** (this file) - Setup summary

### 4. Makefile Targets

Added convenience targets to `Makefile`:
- `make package` - Build all packages
- `make package-deb` - Build .deb only
- `make package-tar` - Build tarball only
- `make release` - Interactive release helper

## Quick Start

### Build Packages Locally

```bash
# Build all packages
make package

# Or individually
make package-deb    # Debian package only
make package-tar    # Tarball only

# Or using scripts directly
./scripts/package/build-all.sh
```

### Create a GitHub Release

```bash
# 1. Update version in Cargo.toml
#    version = "0.2.0"

# 2. Update CHANGELOG.md (create if needed)

# 3. Commit and tag
git add Cargo.toml CHANGELOG.md
git commit -m "chore: bump version to 0.2.0"
git tag -a v0.2.0 -m "Release v0.2.0"

# 4. Push (this triggers automatic build and release)
git push
git push origin v0.2.0

# 5. Monitor progress
# https://github.com/RyosukeMondo/keyrx/actions
```

## Package Details

### Debian Package (`.deb`)

**File:** `target/debian/keyrx_<version>_amd64.deb`

**Installation:**
```bash
sudo dpkg -i keyrx_<version>_amd64.deb
sudo apt-get install -f
sudo systemctl start keyrx
```

**What it installs:**
```
/usr/bin/keyrx_compiler          # Config compiler
/usr/bin/keyrx_daemon            # Daemon
/usr/share/keyrx/                # Web UI
/etc/systemd/system/keyrx.service
/usr/share/doc/keyrx/
/var/log/keyrx/                  # Created by postinst
```

**Features:**
- Automatic systemd integration
- Dependency management via dpkg
- Clean uninstall with package manager
- Security hardening (NoNewPrivileges, ProtectSystem)

### Tarball (`.tar.gz`)

**File:** `target/tarball/keyrx-<version>-linux-x86_64.tar.gz`

**Installation:**
```bash
tar -xzf keyrx-<version>-linux-x86_64.tar.gz
cd keyrx-<version>-linux-x86_64
sudo ./install.sh

# Custom location:
sudo INSTALL_DIR=/opt/keyrx ./install.sh
```

**What it contains:**
```
bin/keyrx_compiler
bin/keyrx_daemon
share/ui/                        # Web UI files
share/keyrx.service              # systemd template
doc/README.md
install.sh
uninstall.sh
README.txt
```

**Features:**
- Works on any Linux distro
- No package manager required
- Custom install directory
- Includes uninstall script

## GitHub Release Workflow

When you push a tag (e.g., `v0.2.0`):

1. **Build Stage** (runs on Ubuntu and Windows):
   - Build Rust binaries
   - Build WASM module
   - Build UI bundle
   - Create .deb package (Linux only)
   - Create tarball (Linux only)
   - Create standalone binaries (both platforms)

2. **Release Stage**:
   - Download all artifacts
   - Generate release notes with installation instructions
   - Create GitHub release
   - Upload all packages

**Timeline:** ~5-10 minutes

**Artifacts uploaded:**
- `keyrx_<version>_amd64.deb` - Debian package
- `keyrx-<version>-linux-x86_64.tar.gz` - Linux tarball
- `keyrx_compiler-linux-x86_64` - Standalone compiler
- `keyrx_daemon-linux-x86_64` - Standalone daemon
- `keyrx-ui-bundle-<version>.tar.gz` - UI files only
- Windows binaries (.exe)

## Testing Before Release

### Test Package Building

```bash
# Test all packages
make package

# Verify outputs
ls -lh target/debian/*.deb
ls -lh target/tarball/*.tar.gz
```

### Test Installation

**Debian (in Docker):**
```bash
docker run -it --rm -v $(pwd):/work ubuntu:22.04
apt-get update && apt-get install -y systemd
dpkg -i /work/target/debian/keyrx_*.deb
systemctl status keyrx
```

**Tarball:**
```bash
cd /tmp
tar -xzf ~/repos/keyrx/target/tarball/keyrx-*.tar.gz
cd keyrx-*
sudo ./install.sh
systemctl status keyrx
curl http://localhost:7777
sudo ./uninstall.sh
```

### Test GitHub Actions Locally

Use [act](https://github.com/nektos/act) to test workflows:

```bash
# Install act
curl https://raw.githubusercontent.com/nektos/act/master/install.sh | sudo bash

# Test release workflow
act -j build-release --secret GITHUB_TOKEN=$GITHUB_TOKEN
```

## Maintenance

### Update Package Scripts

When changing package contents or structure:

1. Edit `scripts/package/build-deb.sh` or `build-tarball.sh`
2. Test locally: `make package`
3. Verify installation in clean environment
4. Commit changes
5. Next release will use updated scripts

### Update systemd Service

The service file is defined in `build-deb.sh` and `build-tarball.sh`.

To modify:
1. Edit the service file template in both scripts
2. Test changes locally
3. Verify with `systemctl status keyrx`

### Add New Files to Packages

**For .deb:**
```bash
# Edit scripts/package/build-deb.sh
cp new-file "$DEB_DIR/usr/share/keyrx/"
```

**For tarball:**
```bash
# Edit scripts/package/build-tarball.sh
cp new-file "$TAR_DIR/share/"
```

## Troubleshooting

### Build Failures

**"npm: command not found"**
```bash
curl -fsSL https://deb.nodesource.com/setup_18.x | sudo -E bash -
sudo apt-get install -y nodejs
```

**"wasm-pack not found"**
```bash
cargo install wasm-pack
rustup target add wasm32-unknown-unknown
```

**"Permission denied"**
```bash
chmod +x scripts/package/*.sh
```

### Installation Failures

**Debian: "dependency problems"**
```bash
sudo apt-get install -f  # Install missing deps
```

**Tarball: "systemctl: command not found"**
- Normal on non-systemd systems
- Daemon must be started manually

### Release Workflow Failures

1. Check Actions tab: https://github.com/RyosukeMondo/keyrx/actions
2. Common issues:
   - Build errors: Fix code and create new tag
   - Upload errors: Check GITHUB_TOKEN permissions
   - Path errors: Verify file paths in workflow

## Next Steps

### Recommended Enhancements

1. **AppImage** - Self-contained executable
2. **Snap** - Ubuntu Software store distribution
3. **Flatpak** - Flathub distribution
4. **RPM** - Fedora/RHEL support
5. **AUR** - Arch Linux User Repository

### Current Distribution Support

| Distribution | Package Type | Status |
|--------------|--------------|--------|
| Ubuntu/Debian | .deb | ✅ Ready |
| Fedora/RHEL | .rpm | ❌ Not yet |
| Arch Linux | AUR | ❌ Not yet |
| Any Linux | .tar.gz | ✅ Ready |
| Snap Store | .snap | ❌ Not yet |
| Flathub | .flatpak | ❌ Not yet |

## Resources

- **Release Guide:** `docs/RELEASE.md`
- **Package Scripts:** `scripts/package/README.md`
- **GitHub Actions:** `.github/workflows/release.yml`
- **Makefile Targets:** `make help`

## License

KeyRx is licensed under AGPL-3.0-or-later.

Package scripts and documentation are part of the KeyRx project and follow the same license.
