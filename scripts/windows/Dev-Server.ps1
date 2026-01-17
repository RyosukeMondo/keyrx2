# Dev-Server.ps1 - Start Vite dev server only
#
# Usage: .\scripts\windows\Dev-Server.ps1
#
# Assumes daemon is already running on port 9871

$ErrorActionPreference = "Stop"

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$ProjectRoot = (Get-Item (Join-Path $ScriptDir "..\..")).FullName
$UiDir = Join-Path $ProjectRoot "keyrx_ui"

Write-Host ""
Write-Host "=" * 60 -ForegroundColor Green
Write-Host "  Starting Vite Dev Server" -ForegroundColor Green
Write-Host "=" * 60 -ForegroundColor Green
Write-Host ""
Write-Host "Dev UI: http://localhost:5173 (UNMINIFIED ERRORS)" -ForegroundColor Yellow
Write-Host "Daemon: http://localhost:9871 (must be running)" -ForegroundColor Cyan
Write-Host ""
Write-Host "Press Ctrl+C to stop" -ForegroundColor Yellow
Write-Host ""

Set-Location $UiDir
npm run dev
