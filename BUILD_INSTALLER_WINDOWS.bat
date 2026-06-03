@echo off
setlocal
cd /d "%~dp0"
title Build MyScreenshots - Windows Tauri

echo ============================================
echo Build Installer Windows - Tauri
echo ============================================
echo.

where node >nul 2>nul
if errorlevel 1 (
  echo [ERROR] Node.js LTS belum terinstall.
  echo Install dari: https://nodejs.org/
  start "" "https://nodejs.org/"
  pause
  exit /b 1
)

where cargo >nul 2>nul
if errorlevel 1 (
  echo [ERROR] Rust/Cargo belum terinstall.
  echo Install dari: https://rustup.rs/
  start "" "https://rustup.rs/"
  pause
  exit /b 1
)

if not exist "node_modules" (
  echo Install dependency frontend satu kali...
  call npm install
  if errorlevel 1 (
    echo [ERROR] npm install gagal.
    pause
    exit /b 1
  )
)

echo Membuat .exe installer...
call npm run dist:win
if errorlevel 1 (
  echo [ERROR] Build gagal.
  pause
  exit /b 1
)

echo.
echo Build selesai. Cek folder: src-tauri\target\release\bundle\nsis
start "" "%CD%\src-tauri\target\release\bundle\nsis"
pause
