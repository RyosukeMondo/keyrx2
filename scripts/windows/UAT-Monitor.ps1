# UAT-Monitor.ps1 - User Acceptance Test with Log Monitoring for Windows
#
# Usage:
#   .\scripts\windows\UAT-Monitor.ps1              # Full UAT with UI build and log monitoring
#   .\scripts\windows\UAT-Monitor.ps1 -Release     # Release build
#   .\scripts\windows\UAT-Monitor.ps1 -Rebuild     # Force clean rebuild
#   .\scripts\windows\UAT-Monitor.ps1 -SkipWasm    # Skip WASM build
#   .\scripts\windows\UAT-Monitor.ps1 -Headless    # Don't open browser
#
# Logs:
#   - Daemon logs: $env:TEMP\keyrx_daemon.log
#   - UAT build logs: $env:TEMP\keyrx_uat_<timestamp>.log

param(
    [switch]$Release,
    [switch]$Rebuild,
    [switch]$SkipWasm,
    [switch]$Headless,
    [switch]$Debug
)

$ErrorActionPreference = "Stop"

# Paths
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$ProjectRoot = (Get-Item (Join-Path $ScriptDir "..\..")).FullName
$LayoutRhai = Join-Path $ProjectRoot "examples\user_layout.rhai"
$LayoutKrx = Join-Path $ProjectRoot "user_layout.krx"
$DaemonName = "keyrx_daemon"
$DaemonExe = "keyrx_daemon.exe"
$DaemonLog = Join-Path $env:TEMP "keyrx_daemon.log"
$UatLog = Join-Path $env:TEMP "keyrx_uat_$(Get-Date -Format 'yyyyMMdd_HHmmss').log"
$WebUiUrl = "http://localhost:9867"
$UiDir = Join-Path $ProjectRoot "keyrx_ui"

# Build type
$BuildType = if ($Release) { "release" } else { "debug" }
$BuildFlag = if ($Release) { "--release" } else { "" }

# Colors
function Write-Info($Message) {
    Write-Host "[INFO] $Message" -ForegroundColor Cyan
}

function Write-Success($Message) {
    Write-Host "[SUCCESS] $Message" -ForegroundColor Green
}

function Write-Warn($Message) {
    Write-Host "[WARN] $Message" -ForegroundColor Yellow
}

function Write-ErrorMsg($Message) {
    Write-Host "[ERROR] $Message" -ForegroundColor Red
}

function Write-Step($Step, $Total, $Message) {
    Write-Host ""
    Write-Host "[$Step/$Total] $Message" -ForegroundColor Magenta
    Write-Host "=" * 60 -ForegroundColor Gray
}

# Logging function
function Write-Log($Message) {
    $timestamp = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
    "$timestamp - $Message" | Out-File -FilePath $UatLog -Append -Encoding UTF8
}

# Check if daemon is running
function Get-DaemonRunning {
    $process = Get-Process -Name $DaemonName -ErrorAction SilentlyContinue
    return $null -ne $process
}

# Stop daemon
function Stop-Daemon {
    Write-Info "Stopping keyrx daemon..."
    Write-Log "Stopping daemon"

    if (-not (Get-DaemonRunning)) {
        Write-Info "Daemon is not running"
        return
    }

    Stop-Process -Name $DaemonName -Force -ErrorAction SilentlyContinue
    Start-Sleep -Seconds 2

    if (Get-DaemonRunning) {
        Write-ErrorMsg "Failed to stop daemon"
        exit 1
    }

    Write-Success "Daemon stopped successfully"
}

# Clean build artifacts
function Clean-Build {
    Write-Info "Cleaning build artifacts..."
    Write-Log "Cleaning build artifacts"

    # Clean UI dist
    $distPath = Join-Path $UiDir "dist"
    if (Test-Path $distPath) {
        Remove-Item -Recurse -Force $distPath
        Write-Info "Removed UI dist/"
    }

    # Clean Vite cache
    $vitePath = Join-Path $UiDir "node_modules\.vite"
    if (Test-Path $vitePath) {
        Remove-Item -Recurse -Force $vitePath
        Write-Info "Removed Vite cache"
    }

    # Clean daemon build
    Set-Location $ProjectRoot
    cargo clean -p keyrx_daemon 2>$null
    Write-Info "Cleaned daemon build artifacts"
}

# Compile layout
function Compile-Layout {
    Write-Info "Compiling layout: $LayoutRhai -> $LayoutKrx"
    Write-Log "Compiling layout"

    if (-not (Test-Path $LayoutRhai)) {
        Write-ErrorMsg "Layout file not found: $LayoutRhai"
        exit 1
    }

    Set-Location $ProjectRoot
    cargo run --bin keyrx_compiler --quiet -- compile "$LayoutRhai" -o "$LayoutKrx" 2>&1 | Tee-Object -FilePath $UatLog -Append

    if (-not (Test-Path $LayoutKrx)) {
        Write-ErrorMsg "Layout compilation failed"
        exit 1
    }

    Write-Success "Layout compiled successfully"
}

# Build WASM
function Build-Wasm {
    if ($SkipWasm) {
        Write-Info "Skipping WASM build (--SkipWasm flag)"
        return
    }

    Write-Info "Building WASM module..."
    Write-Log "Building WASM"

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
    & wasm-pack $buildArgs 2>&1 | Tee-Object -FilePath $UatLog -Append

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
            Write-Info "  Found: $(Split-Path -Leaf $file)"
        } else {
            Write-ErrorMsg "  Missing: $file"
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

    Write-Success "WASM build completed successfully"
}

# Build Web UI
function Build-Ui {
    Write-Info "Building Web UI..."
    Write-Log "Building UI"

    Set-Location $UiDir

    # Install dependencies if needed
    if (-not (Test-Path "node_modules")) {
        Write-Info "Installing npm dependencies..."
        npm install 2>&1 | Tee-Object -FilePath $UatLog -Append
    }

    # Build production bundle
    Write-Info "Building production bundle..."
    npx vite build 2>&1 | Tee-Object -FilePath $UatLog -Append

    if ($LASTEXITCODE -ne 0) {
        Write-ErrorMsg "UI build failed"
        exit 1
    }

    $indexHtml = Join-Path $UiDir "dist\index.html"
    if (-not (Test-Path $indexHtml)) {
        Write-ErrorMsg "UI build failed - dist/index.html not found"
        exit 1
    }

    Write-Success "UI build completed"
}

# Build daemon
function Build-Daemon {
    Write-Info "Building daemon ($BuildType mode)..."
    Write-Log "Building daemon ($BuildType)"

    Set-Location $ProjectRoot

    # Touch static_files.rs to force re-embedding UI
    $staticFiles = Join-Path $ProjectRoot "keyrx_daemon\src\web\static_files.rs"
    if (Test-Path $staticFiles) {
        (Get-Item $staticFiles).LastWriteTime = Get-Date
        Write-Info "Touched static_files.rs to re-embed UI"
    }

    # Build daemon
    $buildCmd = "cargo build --bin keyrx_daemon --features windows $BuildFlag"
    Write-Info "Running: $buildCmd"

    Invoke-Expression $buildCmd 2>&1 | Tee-Object -FilePath $UatLog -Append

    if ($LASTEXITCODE -ne 0) {
        Write-ErrorMsg "Daemon build failed"
        exit 1
    }

    $daemonPath = Join-Path $ProjectRoot "target\$BuildType\$DaemonExe"
    if (-not (Test-Path $daemonPath)) {
        Write-ErrorMsg "Daemon binary not found: $daemonPath"
        exit 1
    }

    Write-Success "Daemon build completed"
}

# Start daemon in background
function Start-Daemon {
    $daemonPath = Join-Path $ProjectRoot "target\$BuildType\$DaemonExe"

    Write-Info "Starting daemon: $daemonPath"
    Write-Info "Config: $LayoutKrx"
    Write-Info "Logs: $DaemonLog"
    Write-Log "Starting daemon"

    # Clear old log
    if (Test-Path $DaemonLog) {
        Remove-Item $DaemonLog
    }

    # Build daemon arguments
    $daemonArgs = @("run", "--config", "`"$LayoutKrx`"")
    if ($Debug) {
        $daemonArgs += "--debug"
    }

    # Start daemon process with output redirection
    $processInfo = New-Object System.Diagnostics.ProcessStartInfo
    $processInfo.FileName = $daemonPath
    $processInfo.Arguments = $daemonArgs -join " "
    $processInfo.UseShellExecute = $false
    $processInfo.RedirectStandardOutput = $true
    $processInfo.RedirectStandardError = $true
    $processInfo.WorkingDirectory = $ProjectRoot

    $process = New-Object System.Diagnostics.Process
    $process.StartInfo = $processInfo

    # Set up output redirection to file
    $outputHandler = {
        if (-not [String]::IsNullOrEmpty($EventArgs.Data)) {
            $EventArgs.Data | Out-File -FilePath $DaemonLog -Append -Encoding UTF8
        }
    }

    Register-ObjectEvent -InputObject $process -EventName OutputDataReceived -Action $outputHandler | Out-Null
    Register-ObjectEvent -InputObject $process -EventName ErrorDataReceived -Action $outputHandler | Out-Null

    $process.Start() | Out-Null
    $process.BeginOutputReadLine()
    $process.BeginErrorReadLine()

    $daemonPid = $process.Id
    Write-Success "Daemon started (PID: $daemonPid)"

    # Wait a moment for daemon to start
    Start-Sleep -Seconds 3

    # Check if daemon is still running
    if ($process.HasExited) {
        Write-ErrorMsg "Daemon failed to start"
        Write-ErrorMsg "Exit code: $($process.ExitCode)"
        if (Test-Path $DaemonLog) {
            Write-ErrorMsg "Last 20 lines of log:"
            Get-Content $DaemonLog -Tail 20
        }
        exit 1
    }

    # Open browser if not headless
    if (-not $Headless) {
        Start-Sleep -Seconds 1
        Write-Info "Opening browser..."
        Start-Process $WebUiUrl
    }

    Write-Success "Daemon is running"
    Write-Info "Web UI: $WebUiUrl"
    Write-Info ""
}

# Monitor daemon log
function Monitor-DaemonLog {
    Write-Host ""
    Write-Host "=" * 60 -ForegroundColor Cyan
    Write-Host "  Monitoring Daemon Logs" -ForegroundColor Cyan
    Write-Host "  Press Ctrl+C to stop monitoring (daemon keeps running)" -ForegroundColor Yellow
    Write-Host "=" * 60 -ForegroundColor Cyan
    Write-Host ""

    # Wait for log file to appear
    $timeout = 10
    $elapsed = 0
    while (-not (Test-Path $DaemonLog) -and $elapsed -lt $timeout) {
        Start-Sleep -Seconds 1
        $elapsed++
    }

    if (-not (Test-Path $DaemonLog)) {
        Write-Warn "Daemon log file not found at: $DaemonLog"
        Write-Warn "The daemon may not be logging to this location."
        return
    }

    # Tail the log file
    try {
        Get-Content $DaemonLog -Wait -Tail 20
    }
    catch {
        Write-Warn "Log monitoring stopped"
    }
}

# Main execution
function Main {
    Write-Host ""
    Write-Host "=" * 60 -ForegroundColor Green
    Write-Host "  KeyRX UAT Monitor" -ForegroundColor Green
    Write-Host "=" * 60 -ForegroundColor Green
    Write-Host ""
    Write-Info "UAT log: $UatLog"
    Write-Info "Daemon log: $DaemonLog"
    Write-Host ""

    Write-Log "=== UAT Monitor Started ==="
    Write-Log "Build type: $BuildType"
    Write-Log "Skip WASM: $SkipWasm"
    Write-Log "Headless: $Headless"

    Set-Location $ProjectRoot

    $step = 1
    $totalSteps = 7

    # Step 1: Clean (if requested)
    if ($Rebuild) {
        Write-Step $step $totalSteps "Clean build artifacts"
        Clean-Build
        $step++
    }

    # Step 1/2: Stop existing daemon
    Write-Step $step $totalSteps "Stop existing daemon"
    Stop-Daemon
    $step++

    # Step 2/3: Compile layout
    Write-Step $step $totalSteps "Compile layout"
    Compile-Layout
    $step++

    # Step 3/4: Build WASM
    Write-Step $step $totalSteps "Build WASM module"
    Build-Wasm
    $step++

    # Step 4/5: Build UI
    Write-Step $step $totalSteps "Build Web UI"
    Build-Ui
    $step++

    # Step 5/6: Build daemon
    Write-Step $step $totalSteps "Build daemon with embedded UI"
    Build-Daemon
    $step++

    # Step 6/7: Start daemon
    Write-Step $step $totalSteps "Start daemon"
    Start-Daemon

    # Monitor logs
    Write-Host ""
    Write-Host "=" * 60 -ForegroundColor Green
    Write-Host "  UAT Complete!" -ForegroundColor Green
    Write-Host "=" * 60 -ForegroundColor Green
    Write-Host ""
    Write-Info "Web UI: $WebUiUrl"
    Write-Info "Daemon log: $DaemonLog"
    Write-Info "UAT build log: $UatLog"
    Write-Host ""
    Write-Info "To stop the daemon: .\scripts\windows\UAT.ps1"
    Write-Host ""

    Write-Log "=== UAT Build Complete ==="

    # Start monitoring
    Monitor-DaemonLog
}

# Run main
try {
    Main
}
catch {
    Write-ErrorMsg "UAT failed: $_"
    Write-Log "ERROR: $_"
    exit 1
}
