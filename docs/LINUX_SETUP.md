# Linux Setup Guide

Complete guide for setting up KeyRx keyboard remapping daemon on Linux.

## Table of Contents

- [Prerequisites](#prerequisites)
- [Quick Setup](#quick-setup)
- [Installation](#installation)
  - [Building from Source](#building-from-source)
  - [Installing the Binary](#installing-the-binary)
- [Permissions Setup](#permissions-setup)
  - [udev Rules](#udev-rules)
  - [User Groups](#user-groups)
- [Running the Daemon](#running-the-daemon)
  - [Manual Execution](#manual-execution)
  - [systemd Service (System-wide)](#systemd-service-system-wide)
  - [systemd Service (Per-user)](#systemd-service-per-user)
- [Configuration Management](#configuration-management)
  - [Hot Reload](#hot-reload)
  - [Multiple Devices](#multiple-devices)
- [Troubleshooting](#troubleshooting)
- [Security Considerations](#security-considerations)

## Prerequisites

- Linux kernel 2.6+ (evdev and uinput support)
- Rust 1.70+ (for building from source)
- systemd (optional, for service management)

### Verify Kernel Support

```bash
# Check evdev module
lsmod | grep evdev
# If not loaded: sudo modprobe evdev

# Check uinput module
lsmod | grep uinput
# If not loaded: sudo modprobe uinput

# Verify input devices exist
ls /dev/input/event*
```

## Quick Setup

For users who want to get started quickly:

```bash
# 1. Build the daemon
cargo build --release -p keyrx_daemon --features linux

# 2. Install udev rules
sudo cp keyrx_daemon/udev/99-keyrx.rules /etc/udev/rules.d/
sudo udevadm control --reload-rules && sudo udevadm trigger

# 3. Create uinput group and add your user
sudo groupadd -f uinput
sudo usermod -aG input,uinput $USER

# 4. Log out and back in (or reboot)

# 5. Verify setup
./target/release/keyrx_daemon list-devices

# 6. Run the daemon
./target/release/keyrx_daemon run --config your-config.krx
```

## Installation

### Building from Source

```bash
# Clone the repository
git clone https://github.com/keyrx/keyrx.git
cd keyrx

# Build with Linux features enabled
cargo build --release -p keyrx_daemon --features linux

# The binary is at: target/release/keyrx_daemon
```

### Installing the Binary

**System-wide installation (recommended for systemd service):**

```bash
sudo cp target/release/keyrx_daemon /usr/local/bin/
sudo chmod 755 /usr/local/bin/keyrx_daemon
```

**User installation:**

```bash
mkdir -p ~/.local/bin
cp target/release/keyrx_daemon ~/.local/bin/
chmod 755 ~/.local/bin/keyrx_daemon

# Ensure ~/.local/bin is in your PATH
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc
```

## Permissions Setup

KeyRx requires access to input devices and the ability to create virtual keyboards.

### udev Rules

Install the provided udev rules for non-root operation:

```bash
# Copy rules file
sudo cp keyrx_daemon/udev/99-keyrx.rules /etc/udev/rules.d/

# Reload udev rules
sudo udevadm control --reload-rules
sudo udevadm trigger
```

The rules grant:
- `input` group: Read access to `/dev/input/event*` devices
- `uinput` group: Read/write access to `/dev/uinput`

### User Groups

Add your user to the required groups:

```bash
# Create uinput group if it doesn't exist
sudo groupadd -f uinput

# Add user to both groups
sudo usermod -aG input,uinput $USER

# Verify group membership
groups $USER
```

**Important:** You must log out and log back in for group changes to take effect. Alternatively, use:

```bash
newgrp input
newgrp uinput
```

### Verify Permissions

```bash
# Check device permissions
ls -la /dev/input/event* /dev/uinput

# Should show:
# crw-rw---- 1 root input  ... /dev/input/event*
# crw-rw---- 1 root uinput ... /dev/uinput

# Test read access
cat /dev/input/event0
# (Press a key, you should see binary data, Ctrl+C to exit)

# List available keyboards
keyrx_daemon list-devices
```

## Running the Daemon

### Manual Execution

**List available devices:**

```bash
keyrx_daemon list-devices
```

Output:
```
Available keyboard devices:
PATH                    NAME                           SERIAL
/dev/input/event3       AT Translated Set 2 keyboard   -
/dev/input/event18      USB Keyboard                   USB-12345

Tip: Use these names in your config with device_start("USB Keyboard")
     or use device_start("*") to match all keyboards.
```

**Validate configuration (dry-run):**

```bash
keyrx_daemon validate --config my-config.krx
```

Output:
```
Step 1/3: Loading configuration...
  [OK] Configuration loaded successfully

Step 2/3: Enumerating input devices...
  Found 2 keyboard device(s)

Step 3/3: Matching devices to configuration...
  [MATCH] /dev/input/event3 (AT Translated Set 2 keyboard)
          Matched pattern: "*" (5 mappings)
  [MATCH] /dev/input/event18 (USB Keyboard)
          Matched pattern: "*" (5 mappings)

Validation successful! 2 device(s) will be remapped.
Run 'keyrx_daemon run --config my-config.krx' to start remapping.
```

**Run the daemon:**

```bash
# Normal operation
keyrx_daemon run --config my-config.krx

# With debug logging
keyrx_daemon run --config my-config.krx --debug
```

Press `Ctrl+C` to stop the daemon gracefully.

### systemd Service (System-wide)

For system-wide operation with automatic startup:

**1. Create keyrx user:**

```bash
sudo useradd -r -s /usr/sbin/nologin -M -d /nonexistent keyrx
sudo usermod -aG input,uinput keyrx
```

**2. Create configuration directory:**

```bash
sudo mkdir -p /etc/keyrx
sudo cp my-config.krx /etc/keyrx/config.krx
```

**3. Install the service:**

```bash
sudo cp keyrx_daemon/systemd/keyrx.service /etc/systemd/system/
sudo systemctl daemon-reload
```

**4. Enable and start:**

```bash
sudo systemctl enable keyrx.service
sudo systemctl start keyrx.service
```

**Service management commands:**

```bash
# Start/stop/restart
sudo systemctl start keyrx
sudo systemctl stop keyrx
sudo systemctl restart keyrx

# Reload configuration (without restart)
sudo systemctl reload keyrx

# Check status
sudo systemctl status keyrx

# View logs
journalctl -u keyrx -f
```

### systemd Service (Per-user)

For single-user operation without root privileges:

**1. Create directories:**

```bash
mkdir -p ~/.config/systemd/user
mkdir -p ~/.config/keyrx
mkdir -p ~/.local/bin
```

**2. Install files:**

```bash
cp target/release/keyrx_daemon ~/.local/bin/
cp my-config.krx ~/.config/keyrx/config.krx
cp keyrx_daemon/systemd/keyrx-user.service ~/.config/systemd/user/keyrx.service
```

**3. Enable and start:**

```bash
systemctl --user daemon-reload
systemctl --user enable keyrx.service
systemctl --user start keyrx.service
```

**4. (Optional) Start at boot without login:**

```bash
loginctl enable-linger $USER
```

**Service management commands:**

```bash
# Start/stop/restart
systemctl --user start keyrx
systemctl --user stop keyrx
systemctl --user restart keyrx

# Reload configuration
systemctl --user reload keyrx

# Check status
systemctl --user status keyrx

# View logs
journalctl --user -u keyrx -f
```

## Configuration Management

### Hot Reload

Reload configuration without restarting the daemon:

```bash
# For systemd (system service)
sudo systemctl reload keyrx

# For systemd (user service)
systemctl --user reload keyrx

# For manual daemon (send SIGHUP)
kill -HUP $(pgrep keyrx_daemon)
```

Hot reload:
- Preserves current modifier and lock states
- Keeps device grabs active (no interruption)
- Applies new mappings immediately

### Multiple Devices

Configure different mappings for different keyboards:

```rhai
// Built-in laptop keyboard
device_start("AT Translated Set 2");
    map("CapsLock", "VK_Escape");
device_end();

// External USB keyboard with different layout
device_start("USB Keyboard");
    map("CapsLock", "MD_00");  // Navigation layer
    when_start("MD_00");
        map("H", "VK_Left");
        map("J", "VK_Down");
        map("K", "VK_Up");
        map("L", "VK_Right");
    when_end();
device_end();

// Fallback for any other keyboard
device_start("*");
    map("CapsLock", "VK_Escape");
device_end();
```

Use `keyrx_daemon list-devices` to find device names.

## Troubleshooting

### Permission Denied

**Symptom:** `Error: Permission denied when accessing /dev/input/eventX`

**Solutions:**

1. Verify group membership:
   ```bash
   groups $USER  # Should include 'input' and 'uinput'
   ```

2. Check if you logged out after adding groups:
   ```bash
   # Quick fix without logout:
   newgrp input && newgrp uinput
   ```

3. Verify udev rules are installed:
   ```bash
   ls -la /etc/udev/rules.d/99-keyrx.rules
   ```

4. Reload udev rules:
   ```bash
   sudo udevadm control --reload-rules && sudo udevadm trigger
   ```

5. Check device permissions:
   ```bash
   ls -la /dev/input/event* /dev/uinput
   ```

### No Devices Found

**Symptom:** `keyrx_daemon list-devices` shows no keyboards

**Solutions:**

1. Check if input devices exist:
   ```bash
   ls /dev/input/event*
   ```

2. Verify evdev module is loaded:
   ```bash
   lsmod | grep evdev
   sudo modprobe evdev
   ```

3. Check device permissions (may need udev rules).

### uinput Not Available

**Symptom:** `Error: Failed to open /dev/uinput`

**Solutions:**

1. Load uinput module:
   ```bash
   sudo modprobe uinput
   ```

2. Make it persistent (load at boot):
   ```bash
   echo "uinput" | sudo tee /etc/modules-load.d/uinput.conf
   ```

3. Verify device exists:
   ```bash
   ls -la /dev/uinput
   ```

### Daemon Starts but Keys Not Remapped

**Solutions:**

1. Check if device is matched:
   ```bash
   keyrx_daemon validate --config your-config.krx
   ```

2. Verify configuration syntax:
   ```bash
   keyrx_compiler verify your-config.krx
   ```

3. Run with debug logging:
   ```bash
   keyrx_daemon run --config your-config.krx --debug
   ```

### Keys Stuck After Crash

If the daemon crashes while a key is held, it may appear "stuck."

**Solutions:**

1. Press and release the stuck key.
2. If using modifiers, press and release all modifier keys.
3. As last resort: `xdotool key --clearmodifiers Return`

### Service Fails to Start

**Symptom:** `systemctl status keyrx` shows failed

**Solutions:**

1. Check logs for details:
   ```bash
   journalctl -u keyrx --no-pager -n 50
   ```

2. Verify configuration path exists:
   ```bash
   ls -la /etc/keyrx/config.krx  # system service
   ls -la ~/.config/keyrx/config.krx  # user service
   ```

3. Test configuration manually:
   ```bash
   /usr/local/bin/keyrx_daemon validate --config /etc/keyrx/config.krx
   ```

4. Check keyrx user permissions (system service):
   ```bash
   sudo -u keyrx groups  # Should show input, uinput
   ```

## Security Considerations

### Group Access Risks

Users in the `input` group can:
- Read all input devices (keyboards, mice, etc.)
- Potentially capture sensitive input (passwords)

Only add trusted users to the `input` group.

### Virtual Keyboard Risks

Users in the `uinput` group can:
- Create virtual input devices
- Inject arbitrary keyboard/mouse events

This could be used to:
- Automate actions
- Simulate user input

Only add trusted users to the `uinput` group.

### systemd Security Hardening

The provided systemd service file includes security hardening:

- `NoNewPrivileges=yes` - Prevents privilege escalation
- `ProtectSystem=strict` - Read-only filesystem
- `ProtectHome=yes` - No home directory access
- `PrivateTmp=yes` - Isolated temporary directory
- `DeviceAllow=...` - Restricted device access
- `SystemCallFilter=...` - Limited system calls

### Best Practices

1. Use a dedicated `keyrx` user for the system service
2. Keep the daemon binary read-only
3. Store configuration in `/etc/keyrx/` (system) or `~/.config/keyrx/` (user)
4. Review and test configurations before deploying
5. Monitor daemon logs for unusual activity
