# adbrowse

A cross-platform TUI file browser for local and Android filesystems, side-by-side.

Built with Rust, [ratatui](https://ratatui.rs/), and [crossterm](https://github.com/crossterm-rs/crossterm). Optimized for macOS first.

![screenshot placeholder](docs/screenshot.png)

## Features

- **Dual-pane layout** — local filesystem on the left, Android `/sdcard` on the right
- **ADB-powered** — all Android access goes through the `adb` executable (no MTP)
- **File operations** — copy, delete, rename, create folders
- **Async transfers** — non-blocking push/pull with progress indication
- **Keyboard-driven** — vim-style navigation with full shortcut set

## Installation

### Prerequisites

- Rust stable (install via [rustup](https://rustup.rs/))
- ADB (Android Debug Bridge)

#### macOS

```bash
# Install ADB
brew install android-platform-tools

# Build adbrowse
git clone <repo-url> adbrowse
cd adbrowse
cargo build --release
```

#### Android Setup

1. Enable **Developer Options** on your device:
   - Go to **Settings → About phone**
   - Tap **Build number** 7 times
2. Enable **USB Debugging**:
   - Go to **Settings → Developer options → USB debugging**
3. Connect your device via USB
4. Accept the **computer fingerprint prompt** on your phone

### Build & Run

```bash
cargo build
cargo run
```

## Usage

```bash
# Default: local cwd ↔ Android /sdcard
adbrowse

# Custom Android path
adbrowse --android-path /sdcard/Download

# Custom local path
adbrowse --local-path ~/Downloads

# Select specific device by serial
adbrowse --serial <device_serial>

# Enable file logging
adbrowse --log-file ~/.adbrowse.log
```

## Keyboard Shortcuts

| Key               | Action                          |
|-------------------|---------------------------------|
| `Tab`             | Switch active pane              |
| `↑`/`k` `↓`/`j`  | Move selection                  |
| `PgUp`/`PgDn`    | Scroll faster                   |
| `Enter`/`l`       | Enter directory                 |
| `Backspace`/`h`   | Go to parent directory          |
| `Space`           | Toggle file selection           |
| `a`               | Select all in current directory |
| `Esc`             | Clear selection / close modal   |
| `c`               | Copy selected to opposite pane  |
| `d`               | Delete selected (with confirm)  |
| `r`               | Rename selected                 |
| `n`               | Create new folder               |
| `t`               | View transfer queue             |
| `x`               | Cancel active transfer          |
| `/`               | Filter / search current pane    |
| `R`               | Refresh active pane             |
| `?`               | Help popup                      |
| `q`               | Quit                            |

## Architecture

```
src/
├── main.rs        # Entry point, terminal setup, event loop
├── cli.rs         # CLI argument parsing (clap)
├── app.rs         # Application state management
├── ui.rs          # ratatui rendering
├── input.rs       # Key event handling
├── adb.rs         # ADB client abstraction
├── local_fs.rs    # Local filesystem operations
├── file_entry.rs  # FileEntry / Device types
├── transfer.rs    # Async transfer queue
└── error.rs       # Error types (anyhow/thiserror)
```

Key abstractions:

- **`AdbClient`** — wraps all `adb` shell commands with a clean interface
- **`local_fs`** — local filesystem operations via std APIs (free functions)
- **`TransferQueue`** — async queue for non-blocking push/pull jobs

## Troubleshooting

| Problem | Solution |
|---------|----------|
| `adb: command not found` | Install ADB: `brew install android-platform-tools` |
| No device detected | Check USB connection, run `adb devices` manually |
| Device unauthorized | Accept the USB debugging prompt on your phone |
| Permission denied on `/sdcard` | Some paths require `adb shell ls` — try `/sdcard` first |
| Terminal looks broken after crash | Run `reset` to restore terminal state |

## Roadmap

- [ ] Tar-streaming for faster directory transfers
- [ ] File preview pane
- [ ] Configurable keybindings
- [ ] Bookmarks / favorites
- [ ] Windows & Linux testing
- [ ] Progress bars with byte counts

## License

MIT
