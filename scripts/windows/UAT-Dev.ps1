# UAT-Dev.ps1 - Run daemon with development UI server
#
# Usage: .\scripts\windows\UAT-Dev.ps1
#
# This runs:
# 1. Daemon with production embedded UI (for API)
# 2. Vite dev server (for UI with hot reload and unminified errors)

param(
    [switch]$Stop
)

$ErrorActionPreference = "Stop"

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$ProjectRoot = (Get-Item (Join-Path $ScriptDir "..\..")).FullName
$UiDir = Join-Path $ProjectRoot "keyrx_ui"
$LayoutRhai = Join-Path $ProjectRoot "examples\user_layout.rhai"
$LayoutKrx = Join-Path $ProjectRoot "user_layout.krx"
$DaemonName = "keyrx_daemon"

function Write-Info($Message) {
    Write-Host "[INFO] $Message" -ForegroundColor Cyan
}

function Stop-Daemon {
    Write-Info "Stopping daemon..."
    Stop-Process -Name $DaemonName -Force -ErrorAction SilentlyContinue
    Start-Sleep -Seconds 1
}

if ($Stop) {
    Stop-Daemon
    Write-Info "Daemon stopped"
    exit 0
}

# Compile layout
Write-Info "Compiling layout..."
Set-Location $ProjectRoot
cargo run -p keyrx_compiler --quiet -- compile "$LayoutRhai" -o "$LayoutKrx"

# Build daemon (no UI needed, dev server will serve UI)
Write-Info "Building daemon..."
cargo build -p keyrx_daemon --features windows --quiet

# Stop existing daemon
Stop-Daemon

# Start daemon in background
Write-Info "Starting daemon in background..."
$DaemonPath = Join-Path $ProjectRoot "target\debug\keyrx_daemon.exe"
Start-Process -FilePath $DaemonPath -ArgumentList "run", "--config", "`"$LayoutKrx`"" -WindowStyle Hidden

Start-Sleep -Seconds 2

# Start Vite dev server (foreground)
Write-Info "Starting Vite dev server..."
Write-Host ""
Write-Host "=" * 60 -ForegroundColor Green
Write-Host "Development UI: http://localhost:5173" -ForegroundColor Green
Write-Host "Daemon API: http://localhost:9871" -ForegroundColor Green
Write-Host "=" * 60 -ForegroundColor Green
Write-Host ""
Write-Host "Press Ctrl+C to stop dev server" -ForegroundColor Yellow
Write-Host ""

Set-Location $UiDir
npm run dev
