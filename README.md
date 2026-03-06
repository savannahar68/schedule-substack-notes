# Substack Notes Scheduler

Schedule your Substack Notes to publish at a specific time. Substack doesn't have native scheduling for Notes — this fills that gap.

---

## How it works

There are two parts you need to set up once:

1. **The server** — a small program that runs in the background on your computer and keeps track of your scheduled notes. Once installed it starts automatically every time you log in.
2. **The Chrome extension** — adds the scheduling UI to your browser and publishes notes at the right time.

Once set up, you just open the extension, write your note, pick a time, and click Schedule.

> **Important:** Since the server runs on your own computer, your **laptop must be on and Chrome must be open** at the scheduled publish time. If your laptop is asleep or Chrome is closed, the note won't publish until the next time both are running.

---

## Step 1 — Download the files

Go to the [Releases page](../../releases) and download:

**macOS:**
- `substack-scheduler-macos-arm64.tar.gz` if you have an M1/M2/M3/M4 Mac
- `substack-scheduler-macos-x86_64.tar.gz` if you have an older Intel Mac
- `substack-scheduler-extension.zip`

Not sure which Mac you have? Click the Apple menu → **About This Mac**. If you see "Apple M1" (or M2, M3, M4) download the `arm64` version. If you see "Intel" download the `x86_64` version.

**Windows:**
- `substack-scheduler-windows-x86_64.exe.zip`
- `substack-scheduler-extension.zip`

---

## Step 2 — Install and start the server

### macOS

1. Double-click the `.tar.gz` file to extract it. You'll get a folder with the server binary, `start.command`, and `stop.command`.
2. Double-click `start.command`.
3. macOS will block it the first time — click **OK** to dismiss the alert.
4. Open **System Settings** → **Privacy & Security**. Scroll down and you'll see a message saying the file was blocked. Click **Allow Anyway**, then enter your password if prompted.
5. Double-click `start.command` again and click **Open** on the prompt.
6. A notification will pop up saying **"Substack Scheduler is running"**. That's it — the server is now running in the background and will start automatically every time you log in to your Mac.
7. Jump to Step 3 to install the Chrome extension.

To stop and uninstall the server: double-click `stop.command`.

### Windows

1. Right-click the downloaded `.zip` file → **Extract All**.
2. Open the extracted folder and double-click `start.bat`.
3. Windows may show a "Windows protected your PC" warning — click **More info** → **Run anyway**.
4. A popup will appear saying **"Substack Scheduler is running"**. The server is now running in the background and will start automatically at login.
5. Jump to Step 3 to install the Chrome extension.

To stop and uninstall the server: double-click `stop.bat`.

---

## Step 3 — Install the Chrome extension

1. Double-click the downloaded `substack-scheduler-extension.zip` to extract it. You'll get a folder called `extension`.
2. Open Chrome and go to `chrome://extensions` in the address bar.
3. Turn on **Developer mode** using the toggle in the top-right corner.
4. Click **Load unpacked**.
5. Select the `extension` folder you just extracted.
6. The Substack Notes Scheduler extension will appear in your list.

To pin it to your toolbar: click the puzzle piece icon (🧩) in Chrome's toolbar → click the pin icon next to Substack Notes Scheduler.

---

## Step 4 — Connect

1. Make sure you're logged into [substack.com](https://substack.com) in Chrome.
2. Click the Substack Notes Scheduler icon in your toolbar.
3. Click **Connect to Substack**.
4. You're ready to go.

---

## Step 5 — Schedule a note

1. Click the extension icon.
2. Write your note in the text box.
3. Pick the date and time you want it to publish.
4. Click **Schedule**.

The extension will automatically publish the note at the scheduled time. **Chrome must be open** at that time.

---

## Notes

- Uses Substack's unofficial internal API — they can change it at any time
- Don't schedule more than a handful of notes per day
- The `data/` folder next to your server binary holds your database and settings — don't delete it or you'll need to reconnect

---

## Building from source

Requirements: [Rust](https://rustup.rs)

```bash
git clone https://github.com/savannahar68/schedule-substack-notes.git
cd schedule-substack-notes/server
cargo build --release
./target/release/substack-scheduler
```

Then load the `extension/` folder as an unpacked extension in Chrome (Step 3 above).
