# Linux UX Features & Integration - Complete Guide

This document answers all your questions about KeyRx daemon features, startup options, and desktop integration.

## 🎯 Your Questions Answered

### Q: How does the daemon work?
**A:** KeyRx daemon is a two-tier system:

1. **System Daemon** (runs as root via systemd):
   - Intercepts keyboard events using evdev (`/dev/input/eventX`)
   - Applies remapping rules from `.krx` config files
   - Provides HTTP API on port 9867
   - WebSocket for real-time updates

2. **User Interface** (runs as user):
   - Web UI (React, in browser) at http://localhost:9867
   - System tray app (Python + GTK, optional)
   - No root required for UI components

### Q: Launch on boot - daemon or startup registration?
**A: Both! Here's the recommendation:**

#### Recommended Architecture ✅

```
Boot → systemd starts daemon (root, always running)
         ↓
Login → Auto-start system tray (user, optional GUI)
         ↓
On-demand → Open web UI in browser
```

**Why this is best:**
- **Daemon as systemd service**: Proper system integration, survives logout, resource management
- **Tray as auto-start**: User-friendly GUI indicator without requiring root
- **Separation of concerns**: System functionality (root) vs user interface (user)

#### Installation Choices

**For Developers/Power Users:**
```bash
# System daemon only
sudo systemctl enable keyrx    # Auto-start on boot
sudo systemctl start keyrx     # Start now

# Access via web UI (http://localhost:9867) when needed
```

**For Desktop Users:**
```bash
# System daemon (as above) + System tray
sudo systemctl enable keyrx

# Auto-start tray on login
mkdir -p ~/.config/autostart
cp /usr/share/applications/keyrx-tray.desktop ~/.config/autostart/

# Or one-time start
keyrx-tray &
```

### Q: .desktop files for GNOME?
**A: Yes! Three .desktop files included:**

1. **keyrx.desktop** - Application launcher (menu entry)
   - Shows in GNOME applications menu
   - Opens system tray
   - Icon: `input-keyboard`

2. **keyrx-config.desktop** - Quick settings shortcut
   - Direct link to web UI
   - Shows in Settings category
   - One-click configuration access

3. **keyrx-tray.desktop** - Auto-start for tray
   - Goes in `~/.config/autostart/`
   - Hidden from menu (NoDisplay=true)
   - Starts system tray on login

### Q: System tray integration?
**A: Full GNOME/KDE system tray support using AppIndicator3!**

**Features:**
- Status indicator (enabled/disabled)
- Quick enable/disable remapping
- Profile switcher
- Open web UI button
- Desktop notifications (libnotify)
- About dialog

**Screenshot concept:**
```
┌─────────────────────────────┐
│ ⌨️  KeyRx                    │
├─────────────────────────────┤
│ Profile: Gaming             │
│ ──────────────────────────  │
│ ☑ Enable Remapping          │
│ ──────────────────────────  │
│ Switch Profile ▶            │
│   ├ Default        ☑        │
│   ├ Gaming                  │
│   └ Coding                  │
│ ──────────────────────────  │
│ Open Web UI                 │
│ Settings                    │
│ ──────────────────────────  │
│ About                       │
│ Quit Tray                   │
└─────────────────────────────┘
```

## 📦 What You Get

### 1. Debian Package (.deb)
```
/usr/bin/
  ├── keyrx_compiler          # Config compiler
  └── keyrx_daemon            # Main daemon

/usr/share/keyrx/             # Web UI files
/etc/systemd/system/
  └── keyrx.service           # systemd service

/usr/share/applications/
  ├── keyrx.desktop           # App launcher
  └── keyrx-config.desktop    # Settings shortcut

/usr/local/bin/
  └── keyrx-tray              # System tray (if available)
```

### 2. Tarball (.tar.gz)
```
bin/
  ├── keyrx_compiler
  ├── keyrx_daemon
  └── keyrx-tray              # System tray

share/
  ├── ui/                     # Web UI bundle
  ├── keyrx.service           # systemd template
  └── desktop/                # .desktop files

install.sh                    # Installation script
uninstall.sh                  # Removal script
```

## 🚀 Recommended Setup for Different Users

### For Linux Developers (Minimal)
```bash
# Install daemon only
sudo dpkg -i keyrx_*.deb
sudo systemctl enable --now keyrx

# Access when needed
xdg-open http://localhost:9867

# Control via CLI
curl -X POST http://localhost:9867/api/toggle
```

### For Desktop Users (Full Experience)
```bash
# Install package
sudo dpkg -i keyrx_*.deb

# Install tray dependencies
sudo apt-get install python3-gi gir1.2-appindicator3-0.1 python3-requests

# Enable auto-start
mkdir -p ~/.config/autostart
cp /usr/share/applications/keyrx-tray.desktop ~/.config/autostart/

# Start everything
sudo systemctl enable --now keyrx
keyrx-tray &
```

### For Gamers (Performance Focus)
```bash
# Install daemon
sudo dpkg -i keyrx_*.deb
sudo systemctl enable --now keyrx

# Create gaming profile via web UI
xdg-open http://localhost:9867

# Optional: GNOME shortcut to toggle
# Settings → Keyboard → Custom Shortcuts
# Command: curl -X POST http://localhost:9867/api/toggle
# Shortcut: Super+K
```

## 🔧 System Tray Features in Detail

### Status Indicators
- **Green keyboard icon**: Remapping active
- **Gray keyboard icon**: Remapping disabled
- **Red**: Daemon not running

### Quick Actions
1. **Enable/Disable**: One-click toggle
2. **Profile Switch**: Change profiles without opening UI
3. **Open Web UI**: Launches browser
4. **Settings**: Direct to settings page

### Notifications
- Profile switched: "Switched to profile: Gaming"
- Remapping toggled: "Remapping enabled"
- Errors: "Failed to connect to daemon"

### Requirements
```bash
# Ubuntu/Debian
sudo apt-get install \
    python3-gi \
    gir1.2-appindicator3-0.1 \
    gir1.2-notify-0.7 \
    python3-requests

# GNOME users (if tray doesn't show)
sudo apt-get install gnome-shell-extension-appindicator
gnome-extensions enable appindicatorsupport@rgcjonas.gmail.com
```

## 📋 Comparison Table

| Feature | Systemd Service | User Auto-start | System Tray |
|---------|----------------|-----------------|-------------|
| Runs as | root | user | user |
| Auto-start on boot | ✅ Yes | ❌ No (login) | ❌ No (login) |
| Survives logout | ✅ Yes | ❌ No | ❌ No |
| Visual indicator | ❌ No | ❌ No | ✅ Yes |
| Quick toggle | ❌ No | ❌ No | ✅ Yes |
| Resource limits | ✅ Yes | ❌ No | ❌ No |
| Security hardening | ✅ Yes | ❌ No | ❌ No |
| Needs root | ✅ Yes | ❌ No | ❌ No |

## 🎨 GNOME Integration Details

### Application Menu Entry
- Category: Utilities → System
- Keywords: keyboard, remap, input, hotkey, macro
- Shows icon from theme: `input-keyboard`

### Settings Integration
- keyrx-config.desktop appears in Settings
- Category: Hardware Settings
- Direct web UI access

### Notifications
Uses standard GNOME notifications (libnotify):
```python
notification = Notify.Notification.new(
    "KeyRx",
    "Profile switched to Gaming",
    "input-keyboard"
)
```

### Keyboard Shortcuts
Add custom shortcuts in GNOME Settings:
```bash
# Toggle remapping
curl -X POST http://localhost:9867/api/toggle

# Switch profile
curl -X POST http://localhost:9867/api/profiles/activate \
    -H "Content-Type: application/json" \
    -d '{"name":"gaming"}'
```

## 🔄 Auto-start Options Explained

### Option 1: systemd System Service (Recommended)
**File**: `/etc/systemd/system/keyrx.service`

**Pros:**
- Starts before user login
- Proper dependency management
- Resource limits and security
- Survives user logout

**Cons:**
- Requires root
- No GUI feedback

**Enable:**
```bash
sudo systemctl enable keyrx  # Auto-start on boot
sudo systemctl start keyrx   # Start immediately
```

### Option 2: XDG Autostart (.desktop)
**File**: `~/.config/autostart/keyrx-tray.desktop`

**Pros:**
- Per-user configuration
- No root needed
- Desktop environment integrated

**Cons:**
- Only starts on user login
- Stops on logout
- Not suitable for system services

**Enable:**
```bash
mkdir -p ~/.config/autostart
cp /usr/share/applications/keyrx-tray.desktop ~/.config/autostart/
```

### Option 3: systemd User Service (NOT Recommended for KeyRx)
**Why not:** Daemon needs root for evdev access. User services run without elevated privileges.

## 📝 Version & Build Info Display

The web UI footer shows:
```
v0.1.0 • 1/25/2026, 2:01:04 PM • main@71fb6e7f
```

**Components:**
- **v0.1.0**: Version from SSOT (Cargo.toml)
- **Build timestamp**: Actual build time (not runtime)
- **Git info**: Branch and commit hash

**SSOT (Single Source of Truth):**
- `Cargo.toml` (workspace version) → All Rust binaries
- Synced to `package.json` → UI version
- Generated to `version.ts` → Display in UI

See [docs/VERSION_MANAGEMENT.md](docs/VERSION_MANAGEMENT.md) for details.

## 📂 Files & Documentation

| File | Purpose |
|------|---------|
| `LINUX_UX_FEATURES.md` | This file (overview) |
| `docs/DESKTOP_INTEGRATION.md` | Detailed desktop integration guide |
| `docs/VERSION_MANAGEMENT.md` | Version SSOT and build info |
| `docs/RELEASE.md` | Release process guide |
| `keyrx_tray/README.md` | System tray documentation |

## 🚦 Quick Start Guide

### Minimal Setup (5 minutes)
```bash
# Install
sudo dpkg -i keyrx_*.deb
sudo systemctl enable --now keyrx

# Test
curl http://localhost:9867/api/status

# Done! Access UI at http://localhost:9867
```

### Full Desktop Experience (10 minutes)
```bash
# Install daemon
sudo dpkg -i keyrx_*.deb
sudo systemctl enable --now keyrx

# Install tray dependencies
sudo apt-get install python3-gi gir1.2-appindicator3-0.1 python3-requests

# For GNOME users (if needed)
sudo apt-get install gnome-shell-extension-appindicator
gnome-extensions enable appindicatorsupport@rgcjonas.gmail.com

# Setup auto-start
mkdir -p ~/.config/autostart
cp /usr/share/applications/keyrx-tray.desktop ~/.config/autostart/

# Start tray
keyrx-tray &

# Done! System tray icon should appear
```

## 🎁 What's Next?

See the files in your repo:
- `keyrx_tray/` - System tray application
- `scripts/package/` - Package builders (updated)
- `scripts/generate-version.js` - Version management
- `scripts/sync-version.sh` - Version synchronization

Build and test:
```bash
make package                    # Build all packages
./scripts/package/validate.sh  # Verify setup
```

Create release:
```bash
./scripts/sync-version.sh       # Sync versions
git tag v0.2.0 && git push --tags
```

Your Linux UX is now production-ready! 🚀
