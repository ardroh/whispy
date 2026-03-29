# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Run

```bash
cargo build                    # Dev build
cargo run                      # Run with default logging
RUST_LOG=info cargo run        # Run with info-level tracing
cargo build --release          # Release build
bash scripts/install.sh        # Build release, create .app bundle, register LaunchAgent
bash scripts/uninstall.sh      # Remove app and LaunchAgent
```

No tests currently exist. The app requires macOS permissions (Microphone, Accessibility) so manual testing is necessary.

## Code Conventions

**Hard limit: 400 lines per file.** If a file approaches this limit, split it into logical submodules. For example, `preferences.rs` delegates its HTML template to `preferences_html.rs`.

## Architecture

Whispy is a macOS menu bar app that records audio via global hotkey, transcribes via OpenAI Whisper API, and pastes the result at the cursor.

### State Machine

`Idle → Recording (hotkey) → Transcribing (hotkey) → Idle (API response + paste)`

The `Phase` enum in `app.rs` drives all transitions. `AppState` is the central coordinator — it owns the audio recorder, HTTP client, and event proxy.

### Threading Model

- **Main thread**: `tao` event loop polling at 16ms. Owns tray icon, hotkey handler, and preferences window. All `AppState` mutations happen here.
- **Audio thread**: OS-managed `cpal` stream callback pushes samples into `Arc<Mutex<Vec<f32>>>`.
- **Transcription thread**: Spawned per-request in `stop_recording()`. Creates its own single-threaded `tokio` runtime for async `reqwest` call. Posts result back via `EventLoopProxy<UserEvent>`.
- **Clipboard restore thread**: Spawned after Cmd+V paste, sleeps 200ms then restores original clipboard.

### Audio Pipeline

```
cpal input (f32/i16/u16) → normalize to f32[-1,1] → mix to mono → resample to 16kHz → encode WAV (hound, 16-bit)
```

Resampling uses linear interpolation. WAV bytes are sent directly to the Whisper API as multipart form data.

### Preferences Window IPC

The preferences UI is an HTML page rendered in a `wry` WebView. Communication uses a custom async protocol (`whispy://localhost/{endpoint}`). Three endpoints: `save`, `fetch_models`, `test`. Each handler spawns a thread with its own tokio runtime.

Models are fetched live from `GET /v1/models` and filtered for IDs containing "whisper" or "transcri".

### Key Integration Points

- **Config** (`~/Library/Application Support/whispy/config.json`): Loaded at startup and reloaded on preferences window close and before each recording.
- **Paste mechanism**: Writes to clipboard via `arboard`, simulates Cmd+V via `enigo`. Requires macOS Accessibility permission.
- **Notifications**: Errors shown via `osascript` → native macOS notifications.
- **No dock icon**: `Info.plist` has `LSUIElement=true`.

### Crate Ecosystem

The tray, hotkey, and event loop crates (`tray-icon`, `global-hotkey`, `tao`) are all from the Tauri ecosystem and share the same event channel pattern — poll receivers with `try_recv()` each frame.
