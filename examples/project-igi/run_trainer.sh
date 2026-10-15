#!/usr/bin/env bash
# ==============================================================================
# Project IGI - Standalone Trainer Launcher for macOS / CrossOver
# ==============================================================================
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"

BOTTLE_NAME="project-igi"
WINE_BIN="/Users/haseeb-mir/Applications/CrossOver.app/Contents/SharedSupport/CrossOver/bin/wine"
BOTTLE_DIR="/Users/haseeb-mir/Library/Application Support/CrossOver/Bottles/${BOTTLE_NAME}"
TRAINER_EXE="${BOTTLE_DIR}/drive_c/IGI/igi_trainer.exe"

# If trainer exe does not exist in bottle, build and copy it
if [ ! -f "${TRAINER_EXE}" ]; then
    echo "[+] Compiling igi_trainer.exe for Win32..."
    /opt/homebrew/bin/i686-w64-mingw32-g++ -std=c++20 -static -O2 "${SCRIPT_DIR}/igi_trainer.cpp" -o "${REPO_ROOT}/build/examples/igi_trainer.exe"
    cp "${REPO_ROOT}/build/examples/igi_trainer.exe" "${TRAINER_EXE}"
fi

echo "[+] Launching Project IGI Trainer inside CrossOver bottle '${BOTTLE_NAME}'..."
"${WINE_BIN}" --bottle "${BOTTLE_NAME}" "C:\\IGI\\igi_trainer.exe"
