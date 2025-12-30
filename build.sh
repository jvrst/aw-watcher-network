#!/usr/bin/env bash
set -e

echo "Building aw-watcher-network…"
cargo build --release

LABEL="net.activitywatch.aw-watcher-network"
PLIST="$HOME/Library/LaunchAgents/$LABEL.plist"
DOMAIN="gui/$UID"

# Bootstrap only if not already loaded
if ! launchctl print "$DOMAIN/$LABEL" >/dev/null 2>&1; then
  echo "Bootstrapping launch agent…"
  launchctl bootstrap "$DOMAIN" "$PLIST"
fi

echo "Restarting ActivityWatch Network Watcher…"
launchctl kickstart -k "$DOMAIN/$LABEL"
