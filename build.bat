@echo off
setlocal DisableDelayedExpansion
chcp 65001 >nul
title PingPort Build

echo ============================================
echo   PingPort Build
echo ============================================
echo.

cd /d "%~dp0"

set "PINGPORT_BUILD_EXE=%~dp0src-tauri\target\release\pingport.exe"
echo Checking for a running build executable...
powershell.exe -NoProfile -NonInteractive -ExecutionPolicy Bypass -File "%~dp0scripts\stop-build-exe.ps1"
if errorlevel 1 (
    echo [X] Build cancelled. Close PingPort manually and retry.
    pause
    exit /b 1
)
set "PINGPORT_BUILD_EXE="
echo.

echo Building (tauri build --no-bundle)...
echo This compiles frontend + Rust and embeds assets correctly.
echo Skips installer packaging for a faster build.
echo.

call npx tauri build --no-bundle
if %errorlevel% neq 0 (
    echo.
    echo [X] Build failed.
    pause
    exit /b 1
)

echo.
echo ============================================
echo   Build succeeded!
echo   Output: src-tauri\target\release\pingport.exe
echo ============================================
pause
