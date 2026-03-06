# Substack Notes Scheduler

Schedule your Substack Notes. Substack doesn't have native scheduling for Notes — this fills that gap.

## Setup

### 1. Start the server

**Download (easiest):** grab the latest release from [GitHub Releases](../../releases).

- **macOS** — download `substack-scheduler-macos-arm64` (M1/M2/M3) or `substack-scheduler-macos-x86_64` (Intel), and `start.command`. Put them in the same folder and double-click `start.command`. First time: right-click → Open to bypass Gatekeeper.
- **Windows** — download `substack-scheduler-windows-x86_64.exe` and `start.bat`. Put them in the same folder and double-click `start.bat`.
- **Linux** — download `substack-scheduler-linux-x86_64`, `chmod +x` it, run it.

**Build from source:**
```bash
cd server
cargo build --release
./target/release/substack-scheduler
```

Server runs at `http://localhost:6894`. Data is stored in `./data/`.

Optional env config:
```
PORT=6894
DATA_DIR=./data
```

### 2. Install the Chrome extension

1. Go to `chrome://extensions`
2. Enable **Developer mode**
3. Click **Load unpacked** → select the `extension/` folder

### 3. Connect

1. Make sure you're logged into [substack.com](https://substack.com) in Chrome
2. Click the extension icon → **Connect to Substack**

### 4. Schedule a note

Write your note, pick a date/time, click **Schedule**.

The extension checks for due notes every minute and publishes them directly from your browser. Chrome must be running at the scheduled time.

---

## Notes

- Uses Substack's unofficial internal API — they can change it at any time
- Don't abuse it — a few notes a day is fine
- The `./data/` folder contains your database and encryption key — don't delete it or you'll need to reconnect
