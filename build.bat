@echo off
setlocal enabledelayedexpansion

echo ============================================
echo   Claw Code - Windows Build Script
echo ============================================
echo.

REM Check Rust
rustc --version >nul 2>&1
if %ERRORLEVEL% neq 0 (
    echo [ERROR] Rust not found. Install from https://rustup.rs/
    pause
    exit /b 1
)

REM Check Cargo
cargo --version >nul 2>&1
if %ERRORLEVEL% neq 0 (
    echo [ERROR] Cargo not found. Install from https://rustup.rs/
    pause
    exit /b 1
)

echo [1/4] Checking Rust toolchain...
rustc --version
cargo --version
echo.

echo [2/4] Cleaning previous build...
cd rust

REM Kill any running claw.exe
taskkill /F /IM claw.exe >nul 2>&1

REM Wait for file handles to release
timeout /t 2 /nobreak >nul

REM Remove locked files
del /F /Q target\release\deps\claw.exe >nul 2>&1
del /F /Q target\release\deps\claw.exp >nul 2>&1
del /F /Q target\release\deps\claw.lib >nul 2>&1
del /F /Q target\release\deps\claw.d >nul 2>&1
del /F /Q target\release\claw.exe >nul 2>&1
del /F /Q target\release\claw.exe.old >nul 2>&1

REM Rename if delete fails (Windows file lock workaround)
ren target\release\deps\claw.exe claw.exe.locked >nul 2>&1
ren target\release\deps\claw.exp claw.exp.locked >nul 2>&1
ren target\release\deps\claw.lib claw.lib.locked >nul 2>&1
ren target\release\deps\claw.d claw.d.locked >nul 2>&1
ren target\release\claw.exe claw.exe.old >nul 2>&1

echo.
echo [3/4] Building release binary...
cargo build --release
if %ERRORLEVEL% neq 0 (
    echo.
    echo [ERROR] Build failed!
    pause
    exit /b 1
)
echo.

echo [4/4] Verifying binary...
if exist "target\release\claw.exe" (
    echo [OK] Binary built successfully!
    target\release\claw.exe --version
    echo.
    echo Binary location: %CD%\target\release\claw.exe
    echo.
    echo To make 'claw' available globally, run:
    echo   copy target\release\claw.exe %%USERPROFILE%%\.local\bin\
) else (
    echo [ERROR] Binary not found after build!
    pause
    exit /b 1
)

cd ..
echo.
echo ============================================
echo   Build Complete!
echo ============================================
pause
