#!/bin/bash
PLIST="$HOME/Library/LaunchAgents/com.substack-scheduler.plist"

if [ ! -f "$PLIST" ]; then
  osascript -e 'display alert "Substack Scheduler" message "The scheduler is not installed."'
  exit 0
fi

launchctl unload "$PLIST"
rm "$PLIST"

osascript -e 'display notification "Substack Scheduler has been stopped and removed from login items." with title "Substack Scheduler"'
