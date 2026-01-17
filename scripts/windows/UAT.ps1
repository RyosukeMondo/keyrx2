# UAT.ps1 - User Acceptance Test script for keyrx daemon on Windows
#
# Usage: ./scripts/windows/UAT.ps1
#
# This script manages the keyrx daemon for testing:
# - If daemon is running: stops it
# - If daemon is not running: FULL REBUILD (WASM → UI → Daemon) and start
#
# ALWAYS rebuilds WASM, UI, and daemon to ensure latest code is running.

$ErrorActionPreference = "Stop"

# Use Join-Path and Resolve-Path to get the project root accurately
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$ProjectRoot = (Get-Item (Join-Path $ScriptDir "..\..")).FullName
$LayoutRhai = Join-Path $ProjectRoot "examples\user_layout.rhai"
$LayoutKrx = Join-Path $ProjectRoot "user_layout.krx"
$DaemonName = "keyrx_daemon"
$DaemonExe = "keyrx_daemon.exe"
$UiDir = Join-Path $ProjectRoot "keyrx_ui"

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

# Build WASM module
function Build-Wasm {
    Write-Info "Building WASM module..."

    # Check if wasm-pack is installed
    if (-not (Get-Command wasm-pack -ErrorAction SilentlyContinue)) {
        Write-ErrorMsg "wasm-pack is not installed"
        Write-ErrorMsg "Install it with: cargo install wasm-pack"
        exit 1
    }

    $wasmPackVersion = & wasm-pack --version
    Write-Info "Using: $wasmPackVersion"

    # Paths
    $keyrxCoreDir = Join-Path $ProjectRoot "keyrx_core"
    $outputDir = Join-Path $ProjectRoot "keyrx_ui\src\wasm\pkg"

    if (-not (Test-Path $keyrxCoreDir)) {
        Write-ErrorMsg "keyrx_core directory not found: $keyrxCoreDir"
        exit 1
    }

    Write-Info "Building from: $keyrxCoreDir"
    Write-Info "Output to: $outputDir"

    # Build WASM with wasm-pack
    Set-Location $keyrxCoreDir

    $buildArgs = @(
        "build",
        "--target", "web",
        "--out-dir", $outputDir,
        "--release",
        "--",
        "--features", "wasm"
    )

    Write-Info "Running: wasm-pack $($buildArgs -join ' ')"
    & wasm-pack $buildArgs

    if ($LASTEXITCODE -ne 0) {
        Write-ErrorMsg "wasm-pack build failed"
        Set-Location $ProjectRoot
        exit 1
    }

    Set-Location $ProjectRoot

    # Verify output files
    $requiredFiles = @(
        (Join-Path $outputDir "keyrx_core_bg.wasm"),
        (Join-Path $outputDir "keyrx_core.js"),
        (Join-Path $outputDir "keyrx_core.d.ts")
    )

    Write-Info "Verifying output files..."
    $missingFiles = @()
    foreach ($file in $requiredFiles) {
        if (Test-Path $file) {
            Write-Info "  ✓ Found: $(Split-Path -Leaf $file)"
        } else {
            Write-ErrorMsg "  ✗ Missing: $file"
            $missingFiles += $file
        }
    }

    if ($missingFiles.Count -gt 0) {
        Write-ErrorMsg "Build verification failed - missing files"
        exit 1
    }

    # Get WASM file size
    $wasmFile = Join-Path $outputDir "keyrx_core_bg.wasm"
    $wasmSize = (Get-Item $wasmFile).Length
    $wasmSizeKB = [math]::Round($wasmSize / 1024, 2)
    $wasmSizeMB = [math]::Round($wasmSize / 1024 / 1024, 2)

    Write-Info "WASM file size: $wasmSizeKB KB ($wasmSizeMB MB)"

    if ($wasmSizeKB -lt 100) {
        Write-ErrorMsg "WASM file size too small: $wasmSizeKB KB < 100 KB"
        Write-ErrorMsg "Build may have failed or produced invalid output"
        exit 1
    }

    Write-Info "WASM build completed successfully"
}

# Clean UI build artifacts
function Clean-UiBuild {
    Write-Info "Cleaning UI build artifacts..."
    Set-Location $UiDir

    # Remove dist
    if (Test-Path "dist") {
        Remove-Item -Recurse -Force "dist"
        Write-Info "  ✓ Removed dist/"
    }

    # Remove Vite cache
    $viteCachePath = Join-Path "node_modules" ".vite"
    if (Test-Path $viteCachePath) {
        Remove-Item -Recurse -Force $viteCachePath
        Write-Info "  ✓ Removed Vite cache"
    }

    # Remove TypeScript build cache
    if (Test-Path ".tsbuildinfo") {
        Remove-Item -Force ".tsbuildinfo"
        Write-Info "  ✓ Removed .tsbuildinfo"
    }

    Write-Info "UI clean completed"
}

# Build Web UI
function Build-Ui {
    Write-Info "Building Web UI (production build)..."
    Set-Location $UiDir

    # Install dependencies if needed
    if (-not (Test-Path "node_modules")) {
        Write-Info "Installing npm dependencies..."
        npm install
    }

    # Build production bundle
    npx vite build
    if ($LASTEXITCODE -ne 0) {
        Write-ErrorMsg "UI build failed"
        exit 1
    }

    $indexHtml = Join-Path $UiDir "dist\index.html"
    if (-not (Test-Path $indexHtml)) {
        Write-ErrorMsg "UI build failed - dist/index.html not found"
        exit 1
    }

    Write-Info "UI build completed"
}

# Clean daemon build
function Clean-DaemonBuild {
    Write-Info "Cleaning daemon build artifacts..."
    Set-Location $ProjectRoot

    # Remove daemon build artifacts to force full rebuild
    $daemonBuildPath = "target\debug\keyrx_daemon.exe"
    if (Test-Path $daemonBuildPath) {
        Remove-Item -Force $daemonBuildPath
        Write-Info "  ✓ Removed daemon binary"
    }

    # Remove build artifacts (cargo outputs to stderr, so we need to handle it specially)
    $oldErrorAction = $ErrorActionPreference
    $ErrorActionPreference = "Continue"
    try {
        cargo clean -p keyrx_daemon 2>&1 | Out-Null
        Write-Info "  ✓ Cleaned daemon build cache"
    }
    finally {
        $ErrorActionPreference = $oldErrorAction
    }

    Write-Info "Daemon clean completed"
}

# Build daemon with windows feature
function Build-Daemon {
    Write-Info "Building keyrx_daemon with windows feature (embeds UI)..."
    Set-Location $ProjectRoot

    # Touch static_files.rs to force re-embedding UI
    $staticFiles = Join-Path $ProjectRoot "keyrx_daemon\src\web\static_files.rs"
    if (Test-Path $staticFiles) {
        (Get-Item $staticFiles).LastWriteTime = Get-Date
        Write-Info "Touched static_files.rs to re-embed UI"
    }

    # Force rebuild with clean
    cargo build -p keyrx_daemon --features windows
    if ($LASTEXITCODE -ne 0) {
        Write-ErrorMsg "Daemon build failed"
        exit 1
    }

    Write-Info "Daemon build completed"
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
    Write-Host "  CLEAN BUILD: WASM → UI → Daemon"
    Write-Host "========================================"
    Write-Host ""

    if (Get-DaemonRunning) {
        Write-Warn "Daemon is currently running"
        Stop-Daemon
        Write-Host ""
        Write-Info "Daemon stopped. Run this script again to start."
    }
    else {
        Write-Info "Starting full rebuild process..."
        Write-Host ""

        # Step 0: Clean UI build artifacts (to ensure fresh build)
        Write-Host "Step 0/6: Cleaning UI build cache" -ForegroundColor Cyan
        Clean-UiBuild
        Write-Host ""

        # Step 1: Build WASM module
        Write-Host "Step 1/6: Building WASM module" -ForegroundColor Cyan
        Build-Wasm
        Write-Host ""

        # Step 2: Build Web UI
        Write-Host "Step 2/6: Building Web UI" -ForegroundColor Cyan
        Build-Ui
        Write-Host ""

        # Step 3: Clean daemon build
        Write-Host "Step 3/6: Cleaning daemon build" -ForegroundColor Cyan
        Clean-DaemonBuild
        Write-Host ""

        # Step 4: Build daemon (embeds UI)
        Write-Host "Step 4/6: Building daemon with embedded UI" -ForegroundColor Cyan
        Build-Daemon
        Write-Host ""

        # Step 5: Compile layout
        Write-Host "Step 5/6: Compiling layout" -ForegroundColor Cyan
        Compile-Layout
        Write-Host ""

        # Step 6: Start daemon
        Write-Host "Step 6/6: Starting daemon" -ForegroundColor Cyan
        Start-Daemon
    }
}

Main
