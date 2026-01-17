# Clean-All.ps1 - Remove ALL build artifacts
#
# Usage: .\scripts\windows\Clean-All.ps1

$ErrorActionPreference = "Stop"

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$ProjectRoot = (Get-Item (Join-Path $ScriptDir "..\..")).FullName
$UiDir = Join-Path $ProjectRoot "keyrx_ui"

function Write-Info($Message) {
    Write-Host "[CLEAN] $Message" -ForegroundColor Yellow
}

function Write-Success($Message) {
    Write-Host "[CLEAN] $Message" -ForegroundColor Green
}

Set-Location $ProjectRoot

Write-Host ""
Write-Host "=" * 60 -ForegroundColor Red
Write-Host "  CLEANING ALL BUILD ARTIFACTS" -ForegroundColor Red
Write-Host "=" * 60 -ForegroundColor Red
Write-Host ""

# 1. Clean UI artifacts
Write-Info "Cleaning UI build artifacts..."
Set-Location $UiDir

if (Test-Path "dist") {
    Remove-Item -Recurse -Force "dist"
    Write-Success "  ✓ Removed dist/"
}

if (Test-Path "node_modules\.vite") {
    Remove-Item -Recurse -Force "node_modules\.vite"
    Write-Success "  ✓ Removed Vite cache"
}

if (Test-Path ".tsbuildinfo") {
    Remove-Item -Force ".tsbuildinfo"
    Write-Success "  ✓ Removed TypeScript cache"
}

# 2. Clean WASM artifacts
Write-Info "Cleaning WASM artifacts..."
$wasmPkg = Join-Path $UiDir "src\wasm\pkg"
if (Test-Path $wasmPkg) {
    Remove-Item -Recurse -Force $wasmPkg
    Write-Success "  ✓ Removed WASM pkg/"
}

# 3. Clean Rust/Cargo artifacts
Write-Info "Cleaning Rust build artifacts..."
Set-Location $ProjectRoot

if (Test-Path "target") {
    Remove-Item -Recurse -Force "target"
    Write-Success "  ✓ Removed target/"
}

# 4. Clean compiled layout
if (Test-Path "user_layout.krx") {
    Remove-Item -Force "user_layout.krx"
    Write-Success "  ✓ Removed user_layout.krx"
}

Write-Host ""
Write-Host "=" * 60 -ForegroundColor Green
Write-Host "  ALL BUILD ARTIFACTS CLEANED" -ForegroundColor Green
Write-Host "=" * 60 -ForegroundColor Green
Write-Host ""
Write-Host "Next steps:" -ForegroundColor Cyan
Write-Host "  1. Run: .\scripts\windows\UAT.ps1" -ForegroundColor White
Write-Host "  2. This will rebuild everything from scratch" -ForegroundColor White
Write-Host ""
