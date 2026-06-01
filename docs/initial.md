Project name: `adbrowse`

Goal:
Create a cross-platform TUI app, optimized for macOS first, that lets users browse local files and Android `/sdcard` side-by-side, then copy, delete, rename, and sync files using ADB.

Core stack:

* Rust stable
* ratatui for UI
* crossterm for terminal backend/input
* tokio for async process execution and transfers
* clap for CLI args
* anyhow/thiserror for error handling
* tracing + tracing-subscriber for logging
* serde + toml for config
* tempfile for tests if needed

Do not use MTP. All Android access must go through the `adb` executable.

Functional requirements:

1. Startup

* Check that `adb` exists in PATH.
* Run `adb devices`.
* If no device is connected, show a clear TUI error screen with retry and quit options.
* If multiple devices are connected, show a device picker.
* Support `--serial <device_serial>` to select a device from CLI.
* Default Android path: `/sdcard`.
* Default local path: current working directory.
* Support CLI args:

  * `adbrowse`
  * `adbrowse --android-path /sdcard/Download`
  * `adbrowse --local-path ~/Downloads`
  * `adbrowse --serial <serial>`
  * `adbrowse --log-file ~/.adbrowse.log`

2. UI layout

* Use a dual-pane file manager layout.
* Left pane: local filesystem.
* Right pane: Android filesystem over ADB.
* Show current path at the top of each pane.
* Show file rows with:

  * name
  * directory marker
  * size
  * modified time if available
* Highlight active pane.
* Highlight selected row.
* Support scrolling.
* Show status bar at bottom with current action/help.
* Show transfer queue/progress area at bottom or as a popup.
* Show modal confirmations for destructive actions.

3. Keyboard controls

* `Tab`: switch active pane.
* `Up`/`Down` or `k`/`j`: move selection.
* `PageUp`/`PageDown`: scroll faster.
* `Enter` or `l`: enter directory.
* `Backspace` or `h`: go to parent directory.
* `Space`: toggle selection.
* `a`: select all in current directory.
* `Esc`: clear selection or close modal.
* `c`: copy selected files/directories to opposite pane.
* `d`: delete selected files/directories, with confirmation.
* `r`: rename selected single file/directory.
* `n`: create folder in active pane.
* `/`: filter/search current pane.
* `R`: refresh active pane.
* `q`: quit.
* `?`: help popup.

4. ADB filesystem operations
   Implement an `AdbClient` abstraction. Initially it may shell out to `adb`, but keep the interface clean so it can later be replaced.

Required methods:

* `list_dir(path: &str) -> Vec<FileEntry>`
* `stat(path: &str) -> FileEntry`
* `mkdir(path: &str)`
* `rename(from: &str, to: &str)`
* `delete(path: &str, recursive: bool)`
* `push(local: Path, remote: &str)`
* `pull(remote: &str, local: Path)`
* `device_list() -> Vec<Device>`
* `shell(args: &[&str])`

Important:

* Handle paths with spaces and special characters safely.
* Never concatenate unsanitized shell strings when avoidable.
* Prefer passing arguments directly to `Command`.
* For Android shell commands, implement robust quoting for remote paths.

5. Directory listing on Android
   Use `adb shell` for listing.

Prefer an implementation based on:

* `adb shell find <path> -maxdepth 1 -mindepth 1 -printf ...`

But Android toybox/find support may vary, so include fallback strategies:

* Try `find` with printf.
* If unavailable, fall back to `ls -la`.
* Parse best-effort.
* Directories must sort before files.
* Hidden files should be shown.
* Parent navigation should work.

6. Transfers
   Implement an async transfer queue.

Requirements:

* Queue multiple copy jobs.
* Show progress when possible.
* For single-file push/pull, use `adb push` and `adb pull`.
* Parse progress output if available.
* At minimum, show current file, source, destination, status, bytes transferred if known, and completed/failed state.
* Copying from local pane to Android uses `adb push`.
* Copying from Android pane to local uses `adb pull`.
* Recursive copy should work for directories.
* Do not block UI while transferring.
* Allow cancellation with a key such as `x` on the active transfer.

Optional but desirable:

* For pulling Android directories, support faster tar streaming:

  * `adb exec-out tar cf - <remote_dir>`
  * unpack locally
* For pushing directories, use `adb push` initially.
* Add TODO comments where high-performance transfer optimization should go.

7. Local filesystem operations
   Implement a `LocalFs` abstraction with:

* `list_dir`
* `stat`
* `mkdir`
* `rename`
* `delete`
* recursive copy where needed

Use std/tokio filesystem APIs as appropriate.

8. State architecture
   Use a clear app state model:

* `App`
* `PaneState`
* `FileEntry`
* `Selection`
* `TransferJob`
* `TransferQueue`
* `Modal`
* `InputMode`

Suggested module layout:

* `src/main.rs`
* `src/cli.rs`
* `src/app.rs`
* `src/ui.rs`
* `src/input.rs`
* `src/adb.rs`
* `src/local_fs.rs`
* `src/file_entry.rs`
* `src/transfer.rs`
* `src/config.rs`
* `src/error.rs`

9. Error handling

* Surface errors in the TUI status bar or modal.
* Do not panic for normal failures.
* Handle:

  * device disconnected
  * permission denied
  * missing adb
  * unauthorized device
  * failed transfer
  * invalid path
  * unsupported Android shell feature
* If device is unauthorized, instruct user to check the phone and accept USB debugging prompt.

10. Testing
    Add tests for:

* Android path quoting
* parsing `adb devices`
* parsing directory listing output
* local path handling
* selection behavior
* transfer queue state transitions

Use unit tests where possible. Mock `AdbClient` behavior instead of requiring a real Android device in tests.

11. UX polish

* The app should feel responsive.
* Use async tasks for slow operations.
* Show loading state when listing a directory.
* Debounce search/filter input if needed.
* Provide a help popup.
* Show useful empty states.
* Make terminal cleanup reliable on panic or normal exit.
* Restore terminal mode on exit.

12. README
    Create a complete README with:

* App overview
* Screenshot placeholder
* Installation
* macOS setup
* Installing ADB:

  * `brew install android-platform-tools`
* Android setup:

  * enable Developer Options
  * enable USB debugging
  * accept computer fingerprint prompt
* Usage examples
* Keyboard shortcuts
* Troubleshooting
* Roadmap

13. Deliverables
    Produce a complete Rust project that builds with:

```bash
cargo build
```

And runs with:

```bash
cargo run
```

The first version does not need to be perfect, but it must compile, open a TUI, detect ADB devices, browse local files, browse Android `/sdcard`, and support copy via ADB push/pull.

Prioritize correctness, clean architecture, and a working MVP over advanced features.

