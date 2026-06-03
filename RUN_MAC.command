#!/bin/bash
cd "$(dirname "$0")" || exit 1
clear

echo "============================================"
echo "MyScreenshots - Tauri"
echo "============================================"
echo

if ! command -v node >/dev/null 2>&1; then
  echo "[ERROR] Node.js LTS belum terinstall."
  echo "Install dari: https://nodejs.org/"
  open "https://nodejs.org/"
  read -r -p "Tekan Enter untuk keluar..."
  exit 1
fi

if ! command -v cargo >/dev/null 2>&1; then
  echo "[ERROR] Rust/Cargo belum terinstall."
  echo "Install dari: https://rustup.rs/"
  open "https://rustup.rs/"
  read -r -p "Tekan Enter untuk keluar..."
  exit 1
fi

if [ ! -d "node_modules" ]; then
  echo "Install dependency frontend satu kali..."
  npm install || { echo "[ERROR] npm install gagal."; read -r -p "Tekan Enter..."; exit 1; }
fi

echo "Membuka aplikasi..."
npm run dev
