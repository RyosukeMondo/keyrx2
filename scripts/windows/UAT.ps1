# UAT.ps1 - User Acceptance Test script for keyrx daemon on Windows
#
# Usage: ./scripts/windows/UAT.ps1
#
# This script manages the keyrx daemon for testing:
# - If daemon is running: stops it
# - If daemon is not running: compiles layout and starts daemon

$ErrorActionPreference = "Stop"

# Use Join-Path and Resolve-Path to get the project root accurately
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$ProjectRoot = (Get-Item (Join-Path $ScriptDir "..\..")).FullName
$LayoutRhai = Join-Path $ProjectRoot "examples\user_layout.rhai"
$LayoutKrx = Join-Path $env:TEMP "user_layout_uat.krx"
$DaemonName = "keyrx_daemon"
$DaemonExe = "keyrx_daemon.exe"

# Colors for output
function Write-Info($Message) {
    Write-Host "[INFO] $Message" -ForegroundColor Green
}

function Write-Warn($Message) {
    Write-Host "[WARN] $Message" -ForegroundColor Yellow
}

function Write-ErrorMsg($Message) {
    Write-Host "[ERROR] $Message" -ForegroundColor Red
}

# Check if daemon is running
function Get-DaemonRunning {
    $process = Get-Process -Name $DaemonName -ErrorAction SilentlyContinue
    return $process -ne $null
}

# Stop daemon
function Stop-Daemon {
    Write-Info "Stopping keyrx daemon..."
    Stop-Process -Name $DaemonName -Force -ErrorAction SilentlyContinue
    Start-Sleep -Seconds 1

    if (Get-DaemonRunning) {
        Write-ErrorMsg "Failed to stop daemon"
        exit 1
    }

    Write-Info "Daemon stopped successfully"
}

# Build daemon with windows feature
function Build-Daemon {
    Write-Info "Building keyrx_daemon with windows feature..."
    Set-Location $ProjectRoot
    cargo build -p keyrx_daemon --features windows --quiet
}

# Compile layout
function Compile-Layout {
    Write-Info "Compiling $LayoutRhai..."

    if (-not (Test-Path $LayoutRhai)) {
        Write-ErrorMsg "Layout file not found: $LayoutRhai"
        exit 1
    }

    Set-Location $ProjectRoot
    cargo run -p keyrx_compiler --quiet -- compile "$LayoutRhai" -o "$LayoutKrx"

    if (-not (Test-Path $LayoutKrx)) {
        Write-ErrorMsg "Compilation failed"
        exit 1
    }

    Write-Info "Compiled to $LayoutKrx"
}

# Start daemon
function Start-Daemon {
    Write-Info "Starting keyrx daemon..."
    Write-Info "Config: $LayoutKrx"
    Write-Host ""
    Write-Host "Press Ctrl+C to stop the daemon" -ForegroundColor Yellow
    Write-Host ""

    $DaemonPath = Join-Path $ProjectRoot "target\debug\$DaemonExe"
    if (-not (Test-Path $DaemonPath)) {
        Write-ErrorMsg "Daemon executable not found at $DaemonPath"
        exit 1
    }

    & $DaemonPath run --config "$LayoutKrx"
}

# Main
function Main {
    Set-Location $ProjectRoot

    Write-Host "========================================"
    Write-Host "  KeyRx UAT (User Acceptance Test)"
    Write-Host "========================================"
    Write-Host ""

    if (Get-DaemonRunning) {
        Write-Warn "Daemon is currently running"
        Stop-Daemon
        Write-Host ""
        Write-Info "Daemon stopped. Run this script again to start."
    }
    else {
        Write-Info "Daemon is not running"

        # Build daemon binary
        Build-Daemon

        # Compile layout
        Compile-Layout

        # Start daemon
        Start-Daemon
    }
}

Main
