# navcat

<div align="center">
  <img src="navcat.png" width="300">
  <br><br>
  <img src="navcat.gif" width="800">
  <br><br>
</div>

A terminal UI for inspecting Android logcat output from TomTom navigation SDK. Filters by navigation category, highlights by tag type, and lets you toggle filters at runtime without restarting.

## Installation

```bash
cargo install --path .
```

## Usage

```bash
# Live mode — streams from a connected device via adb
navcat

# File mode — loads a saved logcat file into the TUI
navcat -f logcat.txt
```

## TUI Key Bindings

| Key | Action |
|-----|--------|
| `n` | Toggle navigation logs (progress, tracking, waypoints) |
| `g` | Toggle guidance logs |
| `r` | Toggle routing logs |
| `m` | Toggle map-matching logs |
| `/` | Open search bar — filters visible lines as you type |
| `Enter` | Lock search query and close bar |
| `Esc` | Clear search query |
| `↑` / `k` | Scroll up one line |
| `↓` / `j` | Scroll down one line |
| `PgUp` / `Ctrl+U` | Scroll up half a page |
| `PgDn` / `Ctrl+D` | Scroll down half a page |
| `f` / `End` | Resume follow mode (tail) |
| `?` | Toggle key binding hint in status bar |
| `q` `q` | Quit (double-press) |

## Filter Categories

The four toggles are independent — only categories that are on contribute logs to the visible set. All four off means nothing is shown.

| Toggle | Matches tags containing |
|--------|------------------------|
| `n` navigation | everything not in the other three categories |
| `g` guidance | `Guidance`, `Warning` |
| `r` routing | `Planner`, `Replan` |
| `m` map-matching | `Match`, `Project` |

Search (`/`) stacks on top of the category filters — e.g. routing-only logs narrowed to lines containing `"timeout"`.

## Tag Colors

| Color | Category |
|-------|----------|
| Blue | Navigation (default) |
| Magenta | Guidance |
| Bold red | Routing |
| Yellow | Map-matching |

## CLI Options

```
-f, --file <FILE>              Load a logcat file instead of live mode
-    --serial <SERIAL>         Target adb device serial for live mode
-l, --logcat-levels <LEVELS>   Log levels to show, comma-separated [default: I,D,E,W]
-t, --tags <TAGS>              Override the default tag filter list
-a, --add-tag <TAG>            Add tags on top of the default list
-n, --no-tag-filter            Show all tags (disable tag filtering)
-i, --highlighted-items <...>  Terms to highlight in yellow background
-s, --show-items <...>         Only show lines containing these terms
    --debug-level <LEVEL>       Internal logging: none/error/info/debug [default: none]
```

## Shell Completion

```bash
mkdir -p ~/.zfunc
navcat --completions zsh > ~/.zfunc/_navcat

# Add to ~/.zshrc if not already there:
# fpath=(~/.zfunc $fpath)
# autoload -Uz compinit && compinit
```

After reloading your shell, `navcat -f <tab>` completes to `.txt` files only.

## Alternatives

**Android Studio logcat** — good for correlating logs with stack traces and crash reports in one window. navcat adds runtime category toggles, stacked search, and a persistent buffer that survives device disconnects. If you're already in AS debugging a crash, stay there. If you're live on a device hunting a replan or map-matching event, navcat is faster.

**[Chipmunk](https://github.com/esrlabs/chipmunk)** — a full-featured desktop log analysis tool with DLT/SomeIP support, graphing, reusable filter sets, and no file size limits. Use Chipmunk for deep post-mortem analysis of large traces or when working with automotive formats. Use navcat for live adb sessions and quick TomTom nav SDK inspection without setup.

## Requirements

- Rust 1.70+
- `adb` on PATH (live mode only)
- Android device or emulator in `device` state (live mode only)

## License

MIT
