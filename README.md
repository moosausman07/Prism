# Prism

Glassy keyboard launcher for Windows (Tauri 2 + Dioxus). Raycast-style.

## Run

```
cargo tauri dev      # dev
cargo tauri build    # release bundle
```

(Install CLI once: `cargo install tauri-cli --version "^2"`. Needs `wasm32-unknown-unknown` target + dioxus-cli `dx`.)

## Use

- **Alt+Space** — toggle launcher (global hotkey). Loses focus → hides.
- Type to fuzzy-search apps + commands. **↑/↓** move, **Enter** open.
- **Ctrl+1–9** — quick-launch numbered rows.
- **Ctrl+K** — actions popup (Pin / Unpin / Open).
- **Esc** — close popup / settings, else hide.
- **Prism Settings** entry — background image + blur + opacity. Persists to app config dir.

## Foundation scope (this pass)

Glass UI, global hotkey, Start-Menu app scan, fuzzy search, Pinned/Recent
sections, Ctrl+1–9, Ctrl+K actions, Windows system commands (Lock/Sleep/
Restart/Shut Down/Log Out/Empty Recycle Bin), settings, JSON persistence.

## Not yet built

Clipboard history, snippet expansion, quick links, notes, canvas, indexed
file search, calendar, window tiling, hyper key, script commands,
auto-updates, AI cursor/chat, dictation. Added incrementally on this base.
