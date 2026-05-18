# Prism

Glassy, keyboard-first launcher for Windows — Raycast-style, built with
Tauri 2 + Dioxus (Rust/WASM frontend).

## Run

```
cargo tauri dev      # dev
cargo tauri build    # release bundle
```

Install the CLI once: `cargo install tauri-cli --version "^2"`. Needs the
`wasm32-unknown-unknown` target and dioxus-cli (`dx`).

## Keys

- **Alt+Space** — toggle launcher (configurable global hotkey). Loses focus → hides.
- Type to fuzzy-search apps + commands. **↑/↓** move, **Enter** activate.
- **Ctrl+1–9** — quick-launch the numbered row.
- **Ctrl+K** — actions popup (Pin / Unpin / Open).
- **Esc** — close popup/overlay, else hide.
- Append **`?`** to a query to search the web (`search_url`).

## Features

- **App search** — scans Start Menu (`ProgramData` + `AppData`), fuzzy-matched, with Pinned and Recent sections.
- **Calculator & converter** — typed in natural language, results copy to clipboard on Enter:
  - Math & percentages: `2+2*3`, `sqrt(144)`, `15% of 200`, `1200 - 10%`.
  - Units: `10 km to miles`, `100 f to c`, `2 cups to ml` (length, mass, temp, volume, speed, data, time, area, angle).
  - Currency: `100 usd to eur` — live rates, cached daily.
  - Date/time: `days until 2026-12-25`, `time in tokyo`, `9:30`, `920 am here to london` (timezone conversion using the real local offset).
- **Clipboard history** — searchable, classifies text / links / colors / images / files.
- **Prism Reminders** — schedule reminders that fire as a fullscreen alarm or a system notification.
- **Theme Studio** — create, preview and apply custom themes (solid/gradient, light/dark) alongside built-in system/dark/light.
- **Settings** — background image, blur, opacity, collapsed mode, Pin/Open action keys.
- **Check for Updates** — lists GitHub releases and installs newer builds in place.
- **System commands** — Lock, Sleep, Restart, Shut Down, Log Out, Empty Recycle Bin.
- **Aliases & Modes** — alias a query to an app/command; run a shell script straight from the launcher.
- Tray icon, custom PNG icons for built-in commands.

## Configuration

`config.toml` in the app config dir is the single source of truth (a legacy
`settings.json` is migrated automatically). The **Edit Config File** command
opens it; re-open Prism to apply.

```toml
toggle_hotkey    = "ALT+SPACE"     # global launcher hotkey
clipboard_hotkey = ""              # e.g. "SUPER+SHIFT+V" (empty = disabled)
search_url       = "https://www.google.com/search?q=%s"
clear_on_enter   = true
show_trayicon    = true

[aliases]
# ff = "Firefox"

[modes]
# focus = "~/scripts/focus.sh"

[prism]                            # Prism-managed state (pins, recents, themes)
```

## Roadmap

Snippet expansion, quick links, notes, indexed file search, window tiling,
script commands, and AI chat — layered onto this base incrementally.
