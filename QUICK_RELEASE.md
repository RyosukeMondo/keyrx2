# Quick Release Reference Card

## Test Build Locally

```bash
# Validate setup
./scripts/package/validate.sh

# Build all packages
make package

# Or individually
make package-deb    # Debian package
make package-tar    # Tarball

# Check outputs
ls -lh target/debian/*.deb
ls -lh target/tarball/*.tar.gz
```

## Create GitHub Release

### Method 1: Quick (one-liner after version update)

```bash
VERSION=0.2.0 && \
git add Cargo.toml CHANGELOG.md && \
git commit -m "chore: bump version to $VERSION" && \
git tag -a "v$VERSION" -m "Release v$VERSION" && \
git push && git push origin "v$VERSION"
```

### Method 2: Step-by-step

```bash
# 1. Update version in Cargo.toml
sed -i 's/^version = .*/version = "0.2.0"/' Cargo.toml

# 2. Update CHANGELOG.md (manually)

# 3. Commit
git add Cargo.toml CHANGELOG.md
git commit -m "chore: bump version to 0.2.0"

# 4. Tag
git tag -a v0.2.0 -m "Release v0.2.0"

# 5. Push
git push
git push origin v0.2.0
```

## Monitor Release

```bash
# Open Actions in browser
gh workflow view release --web

# Or check status
gh run list --workflow=release.yml --limit 1

# View specific run
gh run view <run-id>
```

## Test Installation

### Debian Package

```bash
# In Docker (safe)
docker run -it --rm -v $(pwd):/work ubuntu:22.04
apt-get update
dpkg -i /work/target/debian/keyrx_*.deb
systemctl start keyrx
curl http://localhost:7777
```

### Tarball

```bash
cd /tmp
tar -xzf ~/repos/keyrx/target/tarball/keyrx-*.tar.gz
cd keyrx-*
sudo ./install.sh
systemctl status keyrx
curl http://localhost:7777
sudo ./uninstall.sh
```

## Rollback Release

```bash
# Delete release
gh release delete v0.2.0 --yes

# Delete tag
git tag -d v0.2.0
git push origin :refs/tags/v0.2.0
```

## User Installation (from GitHub)

### Ubuntu/Debian

```bash
VERSION=0.2.0
wget https://github.com/RyosukeMondo/keyrx/releases/download/v${VERSION}/keyrx_${VERSION}_amd64.deb
sudo dpkg -i keyrx_${VERSION}_amd64.deb
sudo apt-get install -f
sudo systemctl start keyrx
```

### Other Linux

```bash
VERSION=0.2.0
wget https://github.com/RyosukeMondo/keyrx/releases/download/v${VERSION}/keyrx-${VERSION}-linux-x86_64.tar.gz
tar -xzf keyrx-${VERSION}-linux-x86_64.tar.gz
cd keyrx-${VERSION}-linux-x86_64
sudo ./install.sh
```

## Troubleshooting

```bash
# Check build tools
./scripts/package/validate.sh

# Clean build
make clean
cargo clean
rm -rf target/debian target/tarball

# Rebuild
make package

# Check GitHub token
gh auth status

# View workflow logs
gh run view --log
```

## Files & Locations

```
Scripts:
  scripts/package/build-deb.sh
  scripts/package/build-tarball.sh
  scripts/package/build-all.sh
  scripts/package/validate.sh

Makefile:
  make package
  make package-deb
  make package-tar
  make release

Docs:
  docs/RELEASE.md (full guide)
  INSTALLER_SETUP.md (overview)
  scripts/package/README.md (details)

GitHub:
  .github/workflows/release.yml
  https://github.com/RyosukeMondo/keyrx/releases
  https://github.com/RyosukeMondo/keyrx/actions
```

## Quick Checks

```bash
# Current version
grep '^version' Cargo.toml | head -1

# Latest tag
git describe --tags --abbrev=0

# Latest release
gh release view --json tagName -q .tagName

# Build status
gh run list --workflow=release.yml --limit 3

# Package sizes
ls -lh target/debian/*.deb target/tarball/*.tar.gz 2>/dev/null
```
