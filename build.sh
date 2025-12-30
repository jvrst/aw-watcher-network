#!/usr/bin/env bash
set -euo pipefail

platform="$(uname -s)"
case "$platform" in
  Darwin)
    exec "$(dirname "$0")/run/macos/build.sh"
    ;;
  Linux)
    exec "$(dirname "$0")/run/linux/build.sh"
    ;;
  MINGW*|MSYS*|CYGWIN*)
    if command -v pwsh >/dev/null 2>&1; then
      exec pwsh -File "$(dirname "$0")/run/windows/build.ps"
    elif command -v powershell >/dev/null 2>&1; then
      exec powershell -File "$(dirname "$0")/run/windows/build.ps"
    else
      echo "PowerShell is required to run run/windows/build.ps"
      exit 1
    fi
    ;;
  *)
    echo "Unsupported platform: $platform"
    exit 1
    ;;
esac
