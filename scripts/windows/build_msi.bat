@echo off
REM Build KeyRx MSI Installer
REM Usage: build_msi.bat

echo ================================
echo  KeyRx MSI Build
echo ================================
echo.

REM Check for WiX
set WIX_PATH=C:\Program Files (x86)\WiX Toolset v3.14\bin
if not exist "%WIX_PATH%\candle.exe" (
    echo ERROR: WiX Toolset not found!
    echo Install with: choco install wixtoolset
    exit /b 1
)

REM Build release binaries
echo [1/4] Building release binaries...
cargo build --release -p keyrx_daemon -p keyrx_compiler
if errorlevel 1 (
    echo ERROR: Cargo build failed!
    exit /b 1
)

REM Create output directory
if not exist "target\installer" mkdir "target\installer"

REM Compile WiX
echo [2/4] Compiling WiX source...
"%WIX_PATH%\candle.exe" -nologo -ext WixUIExtension -ext WixUtilExtension -out "target\installer\keyrx.wixobj" "keyrx_daemon\keyrx_installer.wxs"
if errorlevel 1 (
    echo ERROR: WiX compilation failed!
    exit /b 1
)

REM Link MSI
echo [3/4] Linking MSI...
"%WIX_PATH%\light.exe" -nologo -ext WixUIExtension -ext WixUtilExtension -spdb -out "target\installer\KeyRx-0.1.0-x64.msi" "target\installer\keyrx.wixobj"
if errorlevel 1 (
    echo ERROR: MSI linking failed!
    exit /b 1
)

echo [4/4] Done!
echo.
echo ================================
echo  Build Complete!
echo ================================
echo.
echo Installer: target\installer\KeyRx-0.1.0-x64.msi
echo.
echo To install: msiexec /i "target\installer\KeyRx-0.1.0-x64.msi"
echo To test:    target\installer\KeyRx-0.1.0-x64.msi
