#!/usr/bin/env bash
set -e

LABEL="net.activitywatch.aw-watcher-network"
DOMAIN="gui/$UID"
PLIST="$HOME/Library/LaunchAgents/$LABEL.plist"
BIN_NAME="aw-watcher-network"
BUILD_BIN="target/release/$BIN_NAME"
LINK_BIN="/usr/local/bin/$BIN_NAME"

echo "Building $BIN_NAME…"
cargo build --release

# Symlink binary if missing
if [ ! -e "$LINK_BIN" ]; then
  echo "Linking $BIN_NAME to /usr/local/bin (sudo required)…"
  sudo ln -s "$(pwd)/$BUILD_BIN" "$LINK_BIN"
else
  echo "Binary already linked at $LINK_BIN"
fi

# Bootstrap only if not already loaded
if ! launchctl print "$DOMAIN/$LABEL" >/dev/null 2>&1; then
  echo "Bootstrapping launch agent…"
  launchctl bootstrap "$DOMAIN" "$PLIST"
fi

echo "Restarting ActivityWatch Network Watcher…"
launchctl kickstart -k "$DOMAIN/$LABEL"
