#!/bin/bash
DIR="$(cd "$(dirname "$0")" && pwd)"

ARCH=$(uname -m)
if [ "$ARCH" = "arm64" ]; then
  BIN="$DIR/substack-scheduler-macos-arm64"
else
  BIN="$DIR/substack-scheduler-macos-x86_64"
fi

if [ ! -f "$BIN" ]; then
  osascript -e 'display alert "Substack Scheduler" message "Server binary not found. Make sure all downloaded files are in the same folder."'
  exit 1
fi

chmod +x "$BIN"

PLIST="$HOME/Library/LaunchAgents/com.substack-scheduler.plist"
LABEL="com.substack-scheduler"

# Write the LaunchAgent plist (always overwrite so paths stay correct if folder is moved)
cat > "$PLIST" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>$LABEL</string>
    <key>ProgramArguments</key>
    <array>
        <string>$BIN</string>
    </array>
    <key>WorkingDirectory</key>
    <string>$DIR</string>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
    <key>StandardOutPath</key>
    <string>/tmp/substack-scheduler.log</string>
    <key>StandardErrorPath</key>
    <string>/tmp/substack-scheduler.log</string>
</dict>
</plist>
EOF

# Stop any existing instance, then reload
launchctl unload "$PLIST" 2>/dev/null
launchctl load "$PLIST"

osascript -e 'display notification "Substack Scheduler is running and will start automatically at login." with title "Substack Scheduler"'
