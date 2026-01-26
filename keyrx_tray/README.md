# KeyRx System Tray Application

GNOME/KDE system tray application for KeyRx daemon control.

## Features

- System tray icon with status indicator
- Quick enable/disable remapping
- Profile switching
- Quick access to web UI
- Desktop notifications
- Auto-start support

## Architecture

```
┌─────────────────────────────────────────────────────┐
│  User Session (no root required)                    │
│                                                      │
│  ┌──────────────┐     ┌─────────────────┐          │
│  │ System Tray  │────▶│  Web UI         │          │
│  │ (keyrx-tray) │     │  (Browser)      │          │
│  └──────┬───────┘     └─────────────────┘          │
│         │                                            │
│         │ HTTP/WebSocket                            │
│         │ (localhost:9867)                          │
└─────────┼────────────────────────────────────────────┘
          │
          ▼
┌─────────────────────────────────────────────────────┐
│  System Service (root required for evdev)           │
│                                                      │
│  ┌──────────────────────────────────────┐          │
│  │  keyrx_daemon                        │          │
│  │  - Keyboard interception (evdev)     │          │
│  │  - Event remapping                   │          │
│  │  - Web server (port 9867)            │          │
│  │  - WebSocket events                  │          │
│  └──────────────────────────────────────┘          │
└─────────────────────────────────────────────────────┘
```

## Implementation Options

### Option 1: Python + GTK (Recommended for GNOME)
- Pros: Easy to develop, good GNOME integration
- Cons: Python dependency
- File: `keyrx-tray.py`

### Option 2: Rust + gtk-rs
- Pros: Single language, no runtime deps
- Cons: Larger binary, gtk-rs complexity
- Would need new Cargo crate

### Option 3: Electron (Web Technologies)
- Pros: Reuse React components from UI
- Cons: Heavy, overkill for tray

**Recommendation: Start with Python + GTK, migrate to Rust later if needed**

## Installation

The system tray is optional and installed separately from the daemon:

```bash
# Install dependencies (Ubuntu/Debian)
sudo apt-get install python3-gi gir1.2-appindicator3-0.1

# Copy tray script
sudo cp keyrx-tray.py /usr/local/bin/keyrx-tray
sudo chmod +x /usr/local/bin/keyrx-tray

# Install .desktop file (auto-start)
mkdir -p ~/.config/autostart
cp keyrx-tray.desktop ~/.config/autostart/
```

## Usage

```bash
# Start tray manually
keyrx-tray

# Auto-start (already configured if installed to ~/.config/autostart)
# Logs out and back in, or run:
nohup keyrx-tray &
```

## Dependencies

- Python 3.8+
- PyGObject (python3-gi)
- AppIndicator3 (gir1.2-appindicator3-0.1)
- python3-requests (for HTTP API)

## Development

```bash
# Install dev dependencies
pip3 install --user pygobject requests

# Run from source
python3 keyrx-tray.py

# Test without daemon (mock mode)
KEYRX_MOCK=1 python3 keyrx-tray.py
```
