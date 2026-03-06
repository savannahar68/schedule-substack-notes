#!/bin/bash
# Double-click this file on macOS to start the scheduler.
# On first run, you may need to right-click → Open to bypass Gatekeeper.

DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$DIR"

# Pick the right binary for this Mac
ARCH=$(uname -m)
if [ "$ARCH" = "arm64" ]; then
  BIN="./substack-scheduler-macos-arm64"
else
  BIN="./substack-scheduler-macos-x86_64"
fi

if [ ! -f "$BIN" ]; then
  echo "Binary not found: $BIN"
  echo "Download the correct binary for your Mac from GitHub Releases."
  read -p "Press Enter to close..."
  exit 1
fi

chmod +x "$BIN"
"$BIN"
read -p "Server stopped. Press Enter to close..."
