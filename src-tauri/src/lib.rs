use std::collections::BTreeMap;
use std::path::PathBuf;
use std::process::Command;
use std::sync::Mutex;

use base64::Engine;
use serde::{Deserialize, Serialize};
use tauri::{Emitter, Manager};
use tauri_plugin_global_shortcut::{Code, Modifiers, Shortcut, ShortcutState};
use tauri_plugin_notification::NotificationExt;
use walkdir::WalkDir;

#[derive(Serialize, Clone)]
struct Entry {
    id: String,
    title: String,
    subtitle: String,
    kind: String,   // "app" | "command" | "system" | "settings"
    action: String, // path for apps, keyword for system, "open-settings" etc.
}

/// A user-defined color theme. `theme` in Settings holds either a built-in
/// keyword (system/dark/light/custom) or the `id` of one of these.
#[derive(Serialize, Deserialize, Clone)]
#[serde(default)]
struct Theme {
    id: String,
    name: String,
    appearance: String, // "dark" | "light"
    bg_kind: String,    // "solid" | "gradient"
    bg_start: String,
    bg_end: String,
    text: String,
    selection: String,
    accent: String,
}

impl Default for Theme {
    fn default() -> Self {
        Theme {
            id: String::new(),
            name: "New Theme".into(),
            appearance: "dark".into(),
            bg_kind: "solid".into(),
            bg_start: "#0b0b10".into(),
            bg_end: "#1a1a2e".into(),
            text: "#f4f4f6".into(),
            selection: "#ffffff14".into(),
            accent: "#7882ff".into(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(default)]
struct Settings {
    pinned: Vec<String>,
    recent: Vec<String>,
    bg_color: String,
    collapsed: bool,
    act_pin: String,
    act_open: String,
    theme: String,
    custom_themes: Vec<Theme>,
    // Mirrored from config.toml so the frontend can use them without a
    // second round-trip. These are config-owned (edited via the TOML file).
    search_url: String,
    aliases: BTreeMap<String, String>,
}

/// Prism-managed runtime state, persisted under the `[prism]` table.
#[derive(Serialize, Deserialize, Clone)]
#[serde(default)]
struct PrismState {
    pinned: Vec<String>,
    recent: Vec<String>,
    bg_color: String,
    collapsed: bool,
    act_pin: String,
    act_open: String,
    theme: String,
}

impl Default for PrismState {
    fn default() -> Self {
        PrismState {
            pinned: Vec::new(),
            recent: Vec::new(),
            bg_color: "#222222".into(),
            collapsed: true,
            act_pin: "p".into(),
            act_open: "o".into(),
            theme: "system".into(),
        }
    }
}

/// The single source of truth: `config.toml` in the app config dir.
#[derive(Serialize, Deserialize, Clone)]
#[serde(default)]
struct Config {
    toggle_hotkey: String,
    clipboard_hotkey: String,
    search_url: String,
    clear_on_enter: bool,
    show_trayicon: bool,
    prism: PrismState,
    aliases: BTreeMap<String, String>,
    modes: BTreeMap<String, String>,
    custom_themes: Vec<Theme>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            toggle_hotkey: "ALT+SPACE".into(),
            clipboard_hotkey: String::new(),
            search_url: "https://www.google.com/search?q=%s".into(),
            clear_on_enter: true,
            show_trayicon: true,
            prism: PrismState::default(),
            aliases: BTreeMap::new(),
            modes: BTreeMap::new(),
            custom_themes: Vec::new(),
        }
    }
}

impl Config {
    fn to_settings(&self) -> Settings {
        Settings {
            pinned: self.prism.pinned.clone(),
            recent: self.prism.recent.clone(),
            bg_color: self.prism.bg_color.clone(),
            collapsed: self.prism.collapsed,
            act_pin: self.prism.act_pin.clone(),
            act_open: self.prism.act_open.clone(),
            theme: self.prism.theme.clone(),
            custom_themes: self.custom_themes.clone(),
            search_url: self.search_url.clone(),
            aliases: self.aliases.clone(),
        }
    }

    /// Fold frontend-editable state back in, leaving TOML-only keys intact.
    fn merge_settings(&mut self, s: &Settings) {
        self.prism.pinned = s.pinned.clone();
        self.prism.recent = s.recent.clone();
        self.prism.bg_color = s.bg_color.clone();
        self.prism.collapsed = s.collapsed;
        self.prism.act_pin = s.act_pin.clone();
        self.prism.act_open = s.act_open.clone();
        self.prism.theme = s.theme.clone();
        self.custom_themes = s.custom_themes.clone();
    }
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            pinned: Vec::new(),
            recent: Vec::new(),
            bg_color: "#222222".into(),
            collapsed: true,
            act_pin: "p".into(),
            act_open: "o".into(),
            theme: "system".into(),
            custom_themes: Vec::new(),
            search_url: "https://www.google.com/search?q=%s".into(),
            aliases: BTreeMap::new(),
        }
    }
}

fn config_dir(app: &tauri::AppHandle) -> PathBuf {
    let dir = app
        .path()
        .app_config_dir()
        .unwrap_or_else(|_| PathBuf::from("."));
    let _ = std::fs::create_dir_all(&dir);
    dir
}

fn config_path(app: &tauri::AppHandle) -> PathBuf {
    config_dir(app).join("config.toml")
}

/// A commented default config so first-time users can discover every key.
const DEFAULT_CONFIG_TOML: &str = r#"# Prism configuration — this file is the single source of truth.
# Edit it directly (the "Edit Config File" command opens it), then
# re-open Prism. Theme Studio writes the [[custom_themes]] section.

toggle_hotkey    = "ALT+SPACE"     # global launcher hotkey
clipboard_hotkey = ""              # e.g. "SUPER+SHIFT+V" (empty = disabled)
search_url       = "https://www.google.com/search?q=%s"
clear_on_enter   = true
show_trayicon    = true

# Aliases: type the alias to jump straight to an app/command.
[aliases]
# ff = "Firefox"
# code = "Visual Studio Code"

# Modes: run any shell script straight from the launcher.
[modes]
# focus = "~/scripts/focus.sh"

# Prism-managed state (pins, recents, active theme). Safe to leave as-is.
[prism]
"#;

/// Load config.toml, creating it (and migrating any legacy settings.json)
/// on first run.
fn load_config(app: &tauri::AppHandle) -> Config {
    let p = config_path(app);
    if let Ok(s) = std::fs::read_to_string(&p) {
        return toml::from_str(&s).unwrap_or_default();
    }
    // Migrate from the old settings.json if present.
    let legacy = config_dir(app).join("settings.json");
    if let Ok(s) = std::fs::read_to_string(&legacy) {
        if let Ok(old) = serde_json::from_str::<Settings>(&s) {
            let mut cfg = Config::default();
            cfg.merge_settings(&old);
            save_config(app, &cfg);
            return cfg;
        }
    }
    // Fresh install: write the commented template.
    let _ = std::fs::write(&p, DEFAULT_CONFIG_TOML);
    toml::from_str(DEFAULT_CONFIG_TOML).unwrap_or_default()
}

fn save_config(app: &tauri::AppHandle, cfg: &Config) {
    if let Ok(body) = toml::to_string_pretty(cfg) {
        let _ = std::fs::write(config_path(app), body);
    }
}

#[tauri::command]
fn load_settings(app: tauri::AppHandle) -> Settings {
    load_config(&app).to_settings()
}

#[tauri::command]
fn save_settings(app: tauri::AppHandle, settings: Settings) -> Result<(), String> {
    let mut cfg = load_config(&app);
    cfg.merge_settings(&settings);
    save_config(&app, &cfg);
    Ok(())
}

/// Open config.toml in the user's default editor.
#[tauri::command]
fn open_config_file(app: tauri::AppHandle) -> Result<(), String> {
    let _ = load_config(&app); // ensure it exists
    let p = config_path(&app);
    tauri_plugin_opener::open_path(p.to_string_lossy().to_string(), None::<&str>)
        .map_err(|e| e.to_string())
}

/// Open the configured search engine with the query substituted for %s.
#[tauri::command]
fn web_search(app: tauri::AppHandle, query: String) -> Result<(), String> {
    let cfg = load_config(&app);
    let enc = urlencode(&query);
    let url = if cfg.search_url.contains("%s") {
        cfg.search_url.replace("%s", &enc)
    } else {
        format!("{}{}", cfg.search_url, enc)
    };
    tauri_plugin_opener::open_url(url, None::<&str>).map_err(|e| e.to_string())
}

/// Run a `[modes]` shell script by name.
#[tauri::command]
fn run_mode(app: tauri::AppHandle, name: String) -> Result<(), String> {
    let cfg = load_config(&app);
    let script = cfg
        .modes
        .get(&name)
        .ok_or_else(|| format!("unknown mode: {name}"))?
        .clone();
    let expanded = expand_tilde(&script);
    #[cfg(windows)]
    let r = Command::new("cmd").args(["/C", &expanded]).spawn();
    #[cfg(not(windows))]
    let r = Command::new("sh").args(["-c", &expanded]).spawn();
    r.map(|_| ()).map_err(|e| e.to_string())
}

fn expand_tilde(p: &str) -> String {
    if let Some(rest) = p.strip_prefix("~") {
        if let Some(home) = dirs::home_dir() {
            return format!("{}{}", home.display(), rest);
        }
    }
    p.to_string()
}

fn urlencode(s: &str) -> String {
    let mut out = String::with_capacity(s.len() * 3);
    for b in s.as_bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(*b as char)
            }
            b' ' => out.push('+'),
            _ => out.push_str(&format!("%{:02X}", b)),
        }
    }
    out
}

/// Recursively scan Windows Start Menu folders for .lnk / .url shortcuts.
#[tauri::command]
fn list_entries(app: tauri::AppHandle) -> Vec<Entry> {
    let mut entries: Vec<Entry> = Vec::new();

    let roots: Vec<PathBuf> = {
        let mut v = Vec::new();
        if let Some(pd) = std::env::var_os("ProgramData") {
            v.push(
                PathBuf::from(pd).join("Microsoft\\Windows\\Start Menu\\Programs"),
            );
        }
        if let Some(ad) = std::env::var_os("AppData") {
            v.push(
                PathBuf::from(ad).join("Microsoft\\Windows\\Start Menu\\Programs"),
            );
        }
        v
    };

    let mut seen = std::collections::HashSet::new();
    for root in roots {
        for e in WalkDir::new(&root).into_iter().filter_map(|e| e.ok()) {
            let path = e.path();
            let ext = path
                .extension()
                .and_then(|x| x.to_str())
                .map(|x| x.to_ascii_lowercase());
            if !matches!(ext.as_deref(), Some("lnk") | Some("url")) {
                continue;
            }
            let name = match path.file_stem().and_then(|s| s.to_str()) {
                Some(n) => n.to_string(),
                None => continue,
            };
            let key = name.to_ascii_lowercase();
            if !seen.insert(key) {
                continue;
            }
            entries.push(Entry {
                id: path.to_string_lossy().to_string(),
                title: name,
                subtitle: "Application".into(),
                kind: "app".into(),
                action: path.to_string_lossy().to_string(),
            });
        }
    }

    entries.sort_by(|a, b| a.title.to_lowercase().cmp(&b.title.to_lowercase()));

    // Built-in commands + system commands.
    let mut builtins = vec![
        Entry {
            id: "prism.clipboard".into(),
            title: "Clipboard History".into(),
            subtitle: "System".into(),
            kind: "clipboard".into(),
            action: "open-clipboard".into(),
        },
        Entry {
            id: "prism.reminders".into(),
            title: "Prism Reminders".into(),
            subtitle: "System".into(),
            kind: "reminders".into(),
            action: "open-reminders".into(),
        },
        Entry {
            id: "prism.updates".into(),
            title: "Check for Updates".into(),
            subtitle: "System".into(),
            kind: "updates".into(),
            action: "open-updates".into(),
        },
        Entry {
            id: "prism.settings".into(),
            title: "Prism Settings".into(),
            subtitle: "System".into(),
            kind: "settings".into(),
            action: "open-settings".into(),
        },
        Entry {
            id: "prism.themes".into(),
            title: "Theme Studio".into(),
            subtitle: "Customize, create and apply themes".into(),
            kind: "themes".into(),
            action: "open-themes".into(),
        },
        Entry {
            id: "sys.lock".into(),
            title: "Lock Screen".into(),
            subtitle: "System".into(),
            kind: "system".into(),
            action: "lock".into(),
        },
        Entry {
            id: "sys.sleep".into(),
            title: "Sleep".into(),
            subtitle: "System".into(),
            kind: "system".into(),
            action: "sleep".into(),
        },
        Entry {
            id: "sys.restart".into(),
            title: "Restart".into(),
            subtitle: "System".into(),
            kind: "system".into(),
            action: "restart".into(),
        },
        Entry {
            id: "sys.shutdown".into(),
            title: "Shut Down".into(),
            subtitle: "System".into(),
            kind: "system".into(),
            action: "shutdown".into(),
        },
        Entry {
            id: "sys.logout".into(),
            title: "Log Out".into(),
            subtitle: "System".into(),
            kind: "system".into(),
            action: "logout".into(),
        },
        Entry {
            id: "sys.trash".into(),
            title: "Empty Recycle Bin".into(),
            subtitle: "System".into(),
            kind: "system".into(),
            action: "trash".into(),
        },
        Entry {
            id: "prism.config".into(),
            title: "Edit Config File".into(),
            subtitle: "Open config.toml".into(),
            kind: "config".into(),
            action: "open-config".into(),
        },
    ];

    // User-defined modes from config.toml ([modes]).
    let cfg = load_config(&app);
    for (name, _script) in cfg.modes.iter() {
        builtins.push(Entry {
            id: format!("mode.{name}"),
            title: name.clone(),
            subtitle: "Mode".into(),
            kind: "mode".into(),
            action: name.clone(),
        });
    }

    builtins.append(&mut entries);
    builtins
}

#[tauri::command]
fn launch(app: tauri::AppHandle, path: String) -> Result<(), String> {
    hide_main(&app);
    tauri_plugin_opener::open_path(path, None::<&str>).map_err(|e| e.to_string())
}

#[tauri::command]
fn run_system(app: tauri::AppHandle, action: String) -> Result<(), String> {
    hide_main(&app);
    let res = match action.as_str() {
        "lock" => Command::new("rundll32.exe")
            .args(["user32.dll,LockWorkStation"])
            .spawn(),
        "sleep" => Command::new("rundll32.exe")
            .args(["powrprof.dll,SetSuspendState", "0,1,0"])
            .spawn(),
        "restart" => Command::new("shutdown").args(["/r", "/t", "0"]).spawn(),
        "shutdown" => Command::new("shutdown").args(["/s", "/t", "0"]).spawn(),
        "logout" => Command::new("shutdown").args(["/l"]).spawn(),
        "trash" => Command::new("powershell")
            .args(["-NoProfile", "-Command", "Clear-RecycleBin -Force -ErrorAction SilentlyContinue"])
            .spawn(),
        other => return Err(format!("unknown system action: {other}")),
    };
    res.map(|_| ()).map_err(|e| e.to_string())
}

#[tauri::command]
fn image_data_url(path: String) -> Result<String, String> {
    let bytes = std::fs::read(&path).map_err(|e| e.to_string())?;
    let ext = std::path::Path::new(&path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("png")
        .to_ascii_lowercase();
    let mime = match ext.as_str() {
        "jpg" | "jpeg" => "image/jpeg",
        "webp" => "image/webp",
        "gif" => "image/gif",
        _ => "image/png",
    };
    let b64 = base64::engine::general_purpose::STANDARD.encode(bytes);
    Ok(format!("data:{mime};base64,{b64}"))
}

/// Resolve a .lnk shortcut to its target path so the extracted icon is the
/// real app icon (no shell "shortcut arrow" overlay).
#[cfg(windows)]
fn resolve_lnk(path: &str) -> Option<String> {
    use windows::core::{Interface, PCWSTR};
    use windows::Win32::Storage::FileSystem::WIN32_FIND_DATAW;
    use windows::Win32::System::Com::{
        CoCreateInstance, CoInitializeEx, IPersistFile, CLSCTX_INPROC_SERVER,
        COINIT_APARTMENTTHREADED, STGM_READ,
    };
    use windows::Win32::UI::Shell::{IShellLinkW, ShellLink};

    unsafe {
        let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
        let sl: IShellLinkW =
            CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER).ok()?;
        let pf: IPersistFile = sl.cast().ok()?;
        let wide: Vec<u16> =
            path.encode_utf16().chain(std::iter::once(0)).collect();
        pf.Load(PCWSTR(wide.as_ptr()), STGM_READ).ok()?;
        let mut buf = [0u16; 260];
        let mut wfd = WIN32_FIND_DATAW::default();
        sl.GetPath(&mut buf, &mut wfd, 0).ok()?;
        let end = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
        let s = String::from_utf16_lossy(&buf[..end]);
        let s = s.trim();
        if s.is_empty() {
            None
        } else {
            Some(s.to_string())
        }
    }
}

#[cfg(windows)]
fn extract_icon_png(path: &str) -> Option<Vec<u8>> {
    use std::ffi::c_void;
    use windows::core::PCWSTR;
    use windows::Win32::Graphics::Gdi::{
        DeleteObject, GetDC, GetDIBits, GetObjectW, ReleaseDC, BITMAP, BITMAPINFO,
        BITMAPINFOHEADER, DIB_RGB_COLORS, HGDIOBJ,
    };
    use windows::Win32::UI::Shell::{
        SHGetFileInfoW, SHFILEINFOW, SHGFI_ICON, SHGFI_LARGEICON,
    };
    use windows::Win32::UI::WindowsAndMessaging::{
        DestroyIcon, GetIconInfo, ICONINFO,
    };

    // For shortcuts, point at the resolved target → clean icon, no overlay.
    let target = if path.to_ascii_lowercase().ends_with(".lnk") {
        resolve_lnk(path).unwrap_or_else(|| path.to_string())
    } else {
        path.to_string()
    };

    unsafe {
        let wide: Vec<u16> = target
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();
        let mut info = SHFILEINFOW::default();
        let r = SHGetFileInfoW(
            PCWSTR(wide.as_ptr()),
            Default::default(),
            Some(&mut info),
            std::mem::size_of::<SHFILEINFOW>() as u32,
            SHGFI_ICON | SHGFI_LARGEICON,
        );
        if r == 0 || info.hIcon.is_invalid() {
            return None;
        }
        let hicon = info.hIcon;

        let mut ii = ICONINFO::default();
        if GetIconInfo(hicon, &mut ii).is_err() {
            let _ = DestroyIcon(hicon);
            return None;
        }

        let mut bm = BITMAP::default();
        GetObjectW(
            HGDIOBJ(ii.hbmColor.0),
            std::mem::size_of::<BITMAP>() as i32,
            Some(&mut bm as *mut _ as *mut c_void),
        );
        let w = bm.bmWidth;
        let h = bm.bmHeight;
        if w <= 0 || h <= 0 {
            let _ = DeleteObject(HGDIOBJ(ii.hbmColor.0));
            let _ = DeleteObject(HGDIOBJ(ii.hbmMask.0));
            let _ = DestroyIcon(hicon);
            return None;
        }

        let hdc = GetDC(None);
        let mut bi = BITMAPINFO::default();
        bi.bmiHeader.biSize = std::mem::size_of::<BITMAPINFOHEADER>() as u32;
        bi.bmiHeader.biWidth = w;
        bi.bmiHeader.biHeight = -h; // top-down
        bi.bmiHeader.biPlanes = 1;
        bi.bmiHeader.biBitCount = 32;
        bi.bmiHeader.biCompression = 0; // BI_RGB

        let mut buf = vec![0u8; (w * h * 4) as usize];
        let got = GetDIBits(
            hdc,
            ii.hbmColor,
            0,
            h as u32,
            Some(buf.as_mut_ptr() as *mut c_void),
            &mut bi,
            DIB_RGB_COLORS,
        );
        ReleaseDC(None, hdc);
        let _ = DeleteObject(HGDIOBJ(ii.hbmColor.0));
        let _ = DeleteObject(HGDIOBJ(ii.hbmMask.0));
        let _ = DestroyIcon(hicon);
        if got == 0 {
            return None;
        }

        // BGRA -> RGBA
        for px in buf.chunks_exact_mut(4) {
            px.swap(0, 2);
        }
        let img = image::RgbaImage::from_raw(w as u32, h as u32, buf)?;
        let mut out = std::io::Cursor::new(Vec::new());
        image::DynamicImage::ImageRgba8(img)
            .write_to(&mut out, image::ImageFormat::Png)
            .ok()?;
        Some(out.into_inner())
    }
}

#[cfg(not(windows))]
fn extract_icon_png(_path: &str) -> Option<Vec<u8>> {
    None
}

#[tauri::command]
fn icon_data_url(path: String) -> Option<String> {
    let png = extract_icon_png(&path)?;
    let b64 = base64::engine::general_purpose::STANDARD.encode(png);
    Some(format!("data:image/png;base64,{b64}"))
}

#[tauri::command]
async fn pick_image(app: tauri::AppHandle) -> Option<String> {
    use tauri_plugin_dialog::DialogExt;
    let (tx, rx) = std::sync::mpsc::channel();
    app.dialog()
        .file()
        .add_filter("Images", &["png", "jpg", "jpeg", "webp", "gif"])
        .pick_file(move |f| {
            let _ = tx.send(f.map(|p| p.to_string()));
        });
    rx.recv().ok().flatten()
}

#[tauri::command]
fn hide(app: tauri::AppHandle) {
    hide_main(&app);
}

#[tauri::command]
fn open_settings_window(app: tauri::AppHandle) {
    if let Some(w) = app.get_webview_window("settings") {
        let _ = w.show();
        let _ = w.unminimize();
        let _ = w.set_focus();
    } else {
        let _ = tauri::WebviewWindowBuilder::new(
            &app,
            "settings",
            tauri::WebviewUrl::App("?view=settings".into()),
        )
        .title("Prism Settings")
        .inner_size(560.0, 640.0)
        .min_inner_size(420.0, 420.0)
        .resizable(true)
        .decorations(true)
        .skip_taskbar(false)
        .center()
        .build();
    }
    hide_main(&app);
}

#[tauri::command]
fn close_settings_window(app: tauri::AppHandle) {
    if let Some(w) = app.get_webview_window("settings") {
        let _ = w.close();
    }
}

/// Open (or focus) the Theme Studio in its own decorated, resizable window.
#[tauri::command]
fn open_themes_window(app: tauri::AppHandle) {
    if let Some(w) = app.get_webview_window("themes") {
        let _ = w.show();
        let _ = w.unminimize();
        let _ = w.set_focus();
    } else {
        let _ = tauri::WebviewWindowBuilder::new(
            &app,
            "themes",
            tauri::WebviewUrl::App("?view=themes".into()),
        )
        .title("Prism Theme Studio")
        .inner_size(1040.0, 680.0)
        .min_inner_size(820.0, 560.0)
        .resizable(true)
        .decorations(true)
        .skip_taskbar(false)
        .center()
        .build();
    }
    hide_main(&app);
}

/// Broadcast to every window that the active theme/settings changed so the
/// launcher re-applies its styling live.
#[tauri::command]
fn theme_changed(app: tauri::AppHandle) {
    let _ = app.emit("prism:theme", ());
}

fn hide_main(app: &tauri::AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.hide();
    }
}

/// Center the launcher horizontally but place it higher than screen center
/// (Raycast-style: roughly the upper third of the monitor).
fn position_main(w: &tauri::WebviewWindow) {
    if let (Ok(Some(monitor)), Ok(win_size)) = (w.current_monitor(), w.outer_size()) {
        let mon_pos = monitor.position();
        let mon_size = monitor.size();
        let x = mon_pos.x + ((mon_size.width as i32 - win_size.width as i32) / 2);
        // ~15% from the top of the monitor instead of vertically centered.
        let y = mon_pos.y + (mon_size.height as i32 * 20 / 100);
        let _ = w.set_position(tauri::PhysicalPosition::new(x, y));
    }
}

fn toggle_main(app: &tauri::AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        if w.is_visible().unwrap_or(false) {
            let _ = w.hide();
        } else {
            position_main(&w);
            let _ = w.show();
            let _ = w.set_focus();
            let _ = w.emit("prism:focus", ());
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
struct ClipItem {
    id: String,
    text: String,
    source: String,
    ts: i64,
    #[serde(default)]
    kind: String, // "text" | "image"
    #[serde(default)]
    image: Option<String>, // base64 PNG (no data: prefix) for images
}

#[derive(Serialize, Clone)]
struct ClipView {
    id: String,
    text: String,
    preview: String,
    source: String,
    group: String,
    chars: usize,
    words: usize,
    kind: String,
    image: Option<String>, // full data URL for images
}

#[derive(Default)]
struct ClipState(Mutex<Vec<ClipItem>>);

#[cfg(windows)]
fn foreground_app() -> String {
    use windows::Win32::Foundation::CloseHandle;
    use windows::Win32::System::Threading::{
        OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_FORMAT,
        PROCESS_QUERY_LIMITED_INFORMATION,
    };
    use windows::Win32::UI::WindowsAndMessaging::{
        GetForegroundWindow, GetWindowThreadProcessId,
    };
    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.0.is_null() {
            return "Unknown".into();
        }
        let mut pid = 0u32;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
        if pid == 0 {
            return "Unknown".into();
        }
        let h = match OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) {
            Ok(h) => h,
            Err(_) => return "Unknown".into(),
        };
        let mut buf = [0u16; 260];
        let mut len = buf.len() as u32;
        let ok = QueryFullProcessImageNameW(
            h,
            PROCESS_NAME_FORMAT(0),
            windows::core::PWSTR(buf.as_mut_ptr()),
            &mut len,
        )
        .is_ok();
        let _ = CloseHandle(h);
        if !ok {
            return "Unknown".into();
        }
        let s = String::from_utf16_lossy(&buf[..len as usize]);
        std::path::Path::new(&s)
            .file_stem()
            .and_then(|x| x.to_str())
            .unwrap_or("Unknown")
            .to_string()
    }
}

#[cfg(not(windows))]
fn foreground_app() -> String {
    "Unknown".into()
}

fn clip_path(app: &tauri::AppHandle) -> PathBuf {
    let dir = app
        .path()
        .app_config_dir()
        .unwrap_or_else(|_| PathBuf::from("."));
    let _ = std::fs::create_dir_all(&dir);
    dir.join("clipboard.json")
}

fn persist_clips(path: &PathBuf, items: &[ClipItem]) {
    if let Ok(s) = serde_json::to_string(items) {
        let _ = std::fs::write(path, s);
    }
}

fn group_for(ts: i64) -> String {
    use chrono::{Datelike, Local, TimeZone};
    let now = Local::now();
    let dt = match Local.timestamp_opt(ts, 0).single() {
        Some(d) => d,
        None => return "Earlier".into(),
    };
    let days = now.date_naive().num_days_from_ce() - dt.date_naive().num_days_from_ce();
    match days {
        0 => "Today".into(),
        1 => "Yesterday".into(),
        _ => format!("{}", dt.format("%b %-d, %Y")),
    }
}

#[tauri::command]
fn clipboard_history(state: tauri::State<ClipState>) -> Vec<ClipView> {
    let items = state.0.lock().unwrap();
    items
        .iter()
        .map(|c| {
            let is_img = c.kind == "image";
            let one = c.text.replace(['\n', '\r', '\t'], " ");
            let preview: String = if is_img {
                "Image".to_string()
            } else {
                one.trim().chars().take(120).collect()
            };
            ClipView {
                id: c.id.clone(),
                text: c.text.clone(),
                preview,
                source: c.source.clone(),
                group: group_for(c.ts),
                chars: c.text.chars().count(),
                words: c.text.split_whitespace().count(),
                kind: if is_img { "image".into() } else { "text".into() },
                image: c
                    .image
                    .as_ref()
                    .map(|b| format!("data:image/png;base64,{b}")),
            }
        })
        .collect()
}

#[tauri::command]
fn clipboard_apply(
    app: tauri::AppHandle,
    state: tauri::State<ClipState>,
    id: String,
) -> Result<(), String> {
    let item = {
        let items = state.0.lock().unwrap();
        items
            .iter()
            .find(|c| c.id == id)
            .cloned()
            .ok_or_else(|| "not found".to_string())?
    };
    let mut cb = arboard::Clipboard::new().map_err(|e| e.to_string())?;
    if item.kind == "image" {
        let b64 = item.image.ok_or_else(|| "no image".to_string())?;
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(b64)
            .map_err(|e| e.to_string())?;
        let img = image::load_from_memory(&bytes).map_err(|e| e.to_string())?;
        let rgba = img.to_rgba8();
        let (w, h) = rgba.dimensions();
        cb.set_image(arboard::ImageData {
            width: w as usize,
            height: h as usize,
            bytes: std::borrow::Cow::Owned(rgba.into_raw()),
        })
        .map_err(|e| e.to_string())?;
    } else {
        cb.set_text(item.text).map_err(|e| e.to_string())?;
    }
    hide_main(&app);
    Ok(())
}

#[tauri::command]
fn clipboard_delete(
    app: tauri::AppHandle,
    state: tauri::State<ClipState>,
    id: String,
) {
    let mut items = state.0.lock().unwrap();
    items.retain(|c| c.id != id);
    persist_clips(&clip_path(&app), &items);
}

#[tauri::command]
fn clipboard_clear(app: tauri::AppHandle, state: tauri::State<ClipState>) {
    let mut items = state.0.lock().unwrap();
    items.clear();
    persist_clips(&clip_path(&app), &items);
}

// ===== Prism Reminders =====

#[derive(Serialize, Deserialize, Clone)]
struct Reminder {
    id: String,
    name: String,
    ts: i64,       // unix seconds when it should fire
    mode: String,  // "fullscreen" | "notification"
    #[serde(default)]
    fired: bool,
}

#[derive(Serialize, Clone)]
struct ReminderView {
    id: String,
    name: String,
    mode: String,
    when: String,  // human label, e.g. "May 17, 2026 · 14:30"
    overdue: bool,
}

#[derive(Default)]
struct RemindersState(Mutex<Vec<Reminder>>);

/// Fullscreen reminders that have fired and are waiting to be dismissed.
#[derive(Default)]
struct AlarmState(Mutex<Vec<Reminder>>);

fn reminders_path(app: &tauri::AppHandle) -> PathBuf {
    let dir = app
        .path()
        .app_config_dir()
        .unwrap_or_else(|_| PathBuf::from("."));
    let _ = std::fs::create_dir_all(&dir);
    dir.join("reminders.json")
}

fn persist_reminders(path: &PathBuf, items: &[Reminder]) {
    if let Ok(s) = serde_json::to_string_pretty(items) {
        let _ = std::fs::write(path, s);
    }
}

#[tauri::command]
fn reminders_list(state: tauri::State<RemindersState>) -> Vec<ReminderView> {
    use chrono::{Local, TimeZone};
    let now = Local::now().timestamp();
    let mut v = state.0.lock().unwrap().clone();
    v.sort_by_key(|r| r.ts);
    v.into_iter()
        .map(|r| {
            let when = Local
                .timestamp_opt(r.ts, 0)
                .single()
                .map(|d| d.format("%b %-d, %Y · %H:%M").to_string())
                .unwrap_or_else(|| "—".into());
            ReminderView {
                id: r.id,
                name: r.name,
                mode: r.mode,
                when,
                overdue: r.ts <= now,
            }
        })
        .collect()
}

#[tauri::command]
fn reminder_add(
    app: tauri::AppHandle,
    state: tauri::State<RemindersState>,
    name: String,
    when: String,
    mode: String,
) -> Result<(), String> {
    use chrono::{Local, NaiveDateTime, TimeZone};
    let name = name.trim().to_string();
    if name.is_empty() {
        return Err("name is empty".into());
    }
    // `when` is a browser datetime-local value: "YYYY-MM-DDTHH:MM".
    let naive = NaiveDateTime::parse_from_str(&when, "%Y-%m-%dT%H:%M")
        .map_err(|_| "invalid date/time".to_string())?;
    let ts = Local
        .from_local_datetime(&naive)
        .single()
        .ok_or_else(|| "ambiguous date/time".to_string())?
        .timestamp();
    let mode = if mode == "fullscreen" {
        "fullscreen"
    } else {
        "notification"
    }
    .to_string();
    let mut items = state.0.lock().unwrap();
    items.push(Reminder {
        id: format!("{}", chrono::Local::now().timestamp_millis()),
        name,
        ts,
        mode,
        fired: false,
    });
    persist_reminders(&reminders_path(&app), &items);
    Ok(())
}

#[tauri::command]
fn reminder_delete(
    app: tauri::AppHandle,
    state: tauri::State<RemindersState>,
    id: String,
) {
    let mut items = state.0.lock().unwrap();
    items.retain(|r| r.id != id);
    persist_reminders(&reminders_path(&app), &items);
}

/// Returns the fullscreen reminder currently demanding attention, if any.
#[tauri::command]
fn current_alarm(alarm: tauri::State<AlarmState>) -> Option<Reminder> {
    alarm.0.lock().unwrap().first().cloned()
}

/// Dismiss the active fullscreen alarm. Closes the overlay window when the
/// last pending alarm is cleared.
#[tauri::command]
fn dismiss_alarm(
    app: tauri::AppHandle,
    alarm: tauri::State<AlarmState>,
    reminders: tauri::State<RemindersState>,
) {
    let remaining = {
        let mut a = alarm.0.lock().unwrap();
        if !a.is_empty() {
            let done = a.remove(0);
            let mut rs = reminders.0.lock().unwrap();
            rs.retain(|r| r.id != done.id);
            persist_reminders(&reminders_path(&app), &rs);
        }
        a.len()
    };
    if remaining == 0 {
        if let Some(w) = app.get_webview_window("reminder") {
            let _ = w.hide();
            let _ = w.close();
        }
    } else if let Some(w) = app.get_webview_window("reminder") {
        let _ = w.emit("prism:alarm", ());
    }
}

/// Pop the fullscreen alarm overlay on top of everything.
fn show_alarm_window(app: &tauri::AppHandle) {
    let app = app.clone();
    let _ = app.clone().run_on_main_thread(move || {
        if let Some(w) = app.get_webview_window("reminder") {
            let _ = w.show();
            let _ = w.set_fullscreen(true);
            let _ = w.set_always_on_top(true);
            let _ = w.set_focus();
            let _ = w.emit("prism:alarm", ());
        } else {
            let _ = tauri::WebviewWindowBuilder::new(
                &app,
                "reminder",
                tauri::WebviewUrl::App("?view=reminder".into()),
            )
            .title("Prism Reminder")
            .fullscreen(true)
            .always_on_top(true)
            .decorations(false)
            .skip_taskbar(true)
            .focused(true)
            .build();
        }
    });
}

// ===== Auto update =====

const GH_OWNER: &str = "moosausman07";
const GH_REPO: &str = "prism";
const UPDATE_INTERVAL_SECS: u64 = 3 * 60 * 60; // every 3 hours

fn pat() -> Option<String> {
    std::env::var("GH_PAT").ok().filter(|s| !s.trim().is_empty())
}

#[derive(Deserialize)]
struct GhAsset {
    name: String,
    url: String, // API asset URL (works for private repos)
}

#[derive(Deserialize)]
struct GhRelease {
    tag_name: String,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    body: Option<String>,
    #[serde(default)]
    draft: bool,
    #[serde(default)]
    prerelease: bool,
    html_url: String,
    #[serde(default)]
    published_at: Option<String>,
    #[serde(default)]
    assets: Vec<GhAsset>,
}

#[derive(Serialize, Clone)]
struct UpdateStatus {
    current: String,
    latest: String,
    tag: String,
    has_update: bool,
    url: String,
}

#[derive(Serialize, Clone)]
struct ReleaseRow {
    tag: String,
    name: String,
    notes: String,
    published: String,
    current: bool,
    installable: bool,
}

fn current_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

fn ver_of(tag: &str) -> Option<semver::Version> {
    semver::Version::parse(tag.trim_start_matches(['v', 'V'])).ok()
}

fn gh_client() -> reqwest::blocking::Client {
    reqwest::blocking::Client::builder()
        .user_agent("prism-updater")
        .build()
        .unwrap_or_default()
}

fn gh_get(url: &str) -> Result<reqwest::blocking::Response, String> {
    let mut req = gh_client()
        .get(url)
        .header("Accept", "application/vnd.github+json");
    if let Some(p) = pat() {
        req = req.bearer_auth(p);
    }
    let resp = req.send().map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err(format!("GitHub API error: {}", resp.status()));
    }
    Ok(resp)
}

fn fetch_releases() -> Result<Vec<GhRelease>, String> {
    let url = format!(
        "https://api.github.com/repos/{GH_OWNER}/{GH_REPO}/releases?per_page=100"
    );
    gh_get(&url)?.json().map_err(|e| e.to_string())
}

fn fetch_release_by_tag(tag: &str) -> Result<GhRelease, String> {
    let url = format!(
        "https://api.github.com/repos/{GH_OWNER}/{GH_REPO}/releases/tags/{tag}"
    );
    gh_get(&url)?.json().map_err(|e| e.to_string())
}

/// Newest non-draft, non-prerelease release (or None if none usable).
fn newest_release(rels: &[GhRelease]) -> Option<&GhRelease> {
    rels.iter()
        .filter(|r| !r.draft && !r.prerelease)
        .filter(|r| ver_of(&r.tag_name).is_some())
        .max_by(|a, b| {
            ver_of(&a.tag_name)
                .unwrap()
                .cmp(&ver_of(&b.tag_name).unwrap())
        })
}

fn do_update_check() -> Result<UpdateStatus, String> {
    let cur = current_version();
    let rels = fetch_releases()?;
    let (latest, tag, url, has) = match newest_release(&rels) {
        Some(r) => {
            let newer = match (ver_of(&r.tag_name), ver_of(&cur)) {
                (Some(rv), Some(cv)) => rv > cv,
                _ => false,
            };
            (
                r.tag_name.trim_start_matches(['v', 'V']).to_string(),
                r.tag_name.clone(),
                r.html_url.clone(),
                newer,
            )
        }
        None => (cur.clone(), String::new(), String::new(), false),
    };
    Ok(UpdateStatus {
        current: cur,
        latest,
        tag,
        has_update: has,
        url,
    })
}

/// Download the NSIS setup .exe of a release and launch it silently.
/// The running app is asked to quit so the installer can replace it.
fn do_install(app: &tauri::AppHandle, tag: &str) -> Result<(), String> {
    let rel = fetch_release_by_tag(tag)?;
    let asset = rel
        .assets
        .iter()
        .find(|a| a.name.to_lowercase().ends_with(".exe"))
        .ok_or_else(|| "no Windows installer in this release".to_string())?;

    // GitHub asset API: ask for the binary, handle the S3 redirect manually
    // so the Authorization header is not forwarded to storage.
    let no_redirect = reqwest::blocking::Client::builder()
        .user_agent("prism-updater")
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(|e| e.to_string())?;
    let mut req = no_redirect
        .get(&asset.url)
        .header("Accept", "application/octet-stream");
    if let Some(p) = pat() {
        req = req.bearer_auth(p);
    }
    let resp = req.send().map_err(|e| e.to_string())?;

    let bytes = if resp.status().is_redirection() {
        let loc = resp
            .headers()
            .get(reqwest::header::LOCATION)
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| "missing redirect location".to_string())?
            .to_string();
        gh_client()
            .get(&loc)
            .send()
            .map_err(|e| e.to_string())?
            .error_for_status()
            .map_err(|e| e.to_string())?
            .bytes()
            .map_err(|e| e.to_string())?
    } else if resp.status().is_success() {
        resp.bytes().map_err(|e| e.to_string())?
    } else {
        return Err(format!("download failed: {}", resp.status()));
    };

    let out = std::env::temp_dir().join(&asset.name);
    std::fs::write(&out, &bytes).map_err(|e| e.to_string())?;

    // Wait ~2s (so this process can exit and release its exe), then run the
    // NSIS installer silently.
    let cmd = format!(
        "ping 127.0.0.1 -n 3 >NUL & start \"\" \"{}\" /S",
        out.display()
    );
    Command::new("cmd")
        .args(["/C", &cmd])
        .spawn()
        .map_err(|e| e.to_string())?;

    app.exit(0);
    Ok(())
}

#[tauri::command]
async fn update_check() -> Result<UpdateStatus, String> {
    tauri::async_runtime::spawn_blocking(do_update_check)
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn update_releases() -> Result<Vec<ReleaseRow>, String> {
    tauri::async_runtime::spawn_blocking(|| {
        let cur = current_version();
        let mut rels = fetch_releases()?;
        rels.sort_by(|a, b| match (ver_of(&b.tag_name), ver_of(&a.tag_name)) {
            (Some(bv), Some(av)) => bv.cmp(&av),
            _ => b.tag_name.cmp(&a.tag_name),
        });
        Ok(rels
            .into_iter()
            .filter(|r| !r.draft)
            .map(|r| {
                let tagv = r.tag_name.trim_start_matches(['v', 'V']).to_string();
                let is_cur = tagv == cur;
                let has_exe = r
                    .assets
                    .iter()
                    .any(|a| a.name.to_lowercase().ends_with(".exe"));
                ReleaseRow {
                    name: r.name.clone().filter(|s| !s.is_empty()).unwrap_or_else(
                        || r.tag_name.clone(),
                    ),
                    notes: r.body.clone().unwrap_or_default(),
                    published: r
                        .published_at
                        .clone()
                        .map(|s| s.chars().take(10).collect())
                        .unwrap_or_default(),
                    current: is_cur,
                    installable: has_exe && !is_cur,
                    tag: r.tag_name,
                }
            })
            .collect())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn update_install(app: tauri::AppHandle, tag: String) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || do_install(&app, &tag))
        .await
        .map_err(|e| e.to_string())?
}

/// Parsed global hotkeys, populated at setup from config.toml.
#[derive(Default)]
struct HotkeyState(Mutex<(Option<Shortcut>, Option<Shortcut>)>);

fn key_code(k: &str) -> Option<Code> {
    Some(match k {
        "SPACE" => Code::Space,
        "ENTER" | "RETURN" => Code::Enter,
        "TAB" => Code::Tab,
        "ESC" | "ESCAPE" => Code::Escape,
        "BACKSPACE" => Code::Backspace,
        "0" => Code::Digit0,
        "1" => Code::Digit1,
        "2" => Code::Digit2,
        "3" => Code::Digit3,
        "4" => Code::Digit4,
        "5" => Code::Digit5,
        "6" => Code::Digit6,
        "7" => Code::Digit7,
        "8" => Code::Digit8,
        "9" => Code::Digit9,
        "A" => Code::KeyA,
        "B" => Code::KeyB,
        "C" => Code::KeyC,
        "D" => Code::KeyD,
        "E" => Code::KeyE,
        "F" => Code::KeyF,
        "G" => Code::KeyG,
        "H" => Code::KeyH,
        "I" => Code::KeyI,
        "J" => Code::KeyJ,
        "K" => Code::KeyK,
        "L" => Code::KeyL,
        "M" => Code::KeyM,
        "N" => Code::KeyN,
        "O" => Code::KeyO,
        "P" => Code::KeyP,
        "Q" => Code::KeyQ,
        "R" => Code::KeyR,
        "S" => Code::KeyS,
        "T" => Code::KeyT,
        "U" => Code::KeyU,
        "V" => Code::KeyV,
        "W" => Code::KeyW,
        "X" => Code::KeyX,
        "Y" => Code::KeyY,
        "Z" => Code::KeyZ,
        "F1" => Code::F1,
        "F2" => Code::F2,
        "F3" => Code::F3,
        "F4" => Code::F4,
        "F5" => Code::F5,
        "F6" => Code::F6,
        "F7" => Code::F7,
        "F8" => Code::F8,
        "F9" => Code::F9,
        "F10" => Code::F10,
        "F11" => Code::F11,
        "F12" => Code::F12,
        _ => return None,
    })
}

/// Parse strings like "ALT+SPACE", "SUPER+SHIFT+V", "CTRL+ALT+1".
fn parse_hotkey(s: &str) -> Option<Shortcut> {
    if s.trim().is_empty() {
        return None;
    }
    let mut mods = Modifiers::empty();
    let mut code: Option<Code> = None;
    for raw in s.split('+') {
        match raw.trim().to_uppercase().as_str() {
            "ALT" | "OPTION" => mods |= Modifiers::ALT,
            "CTRL" | "CONTROL" => mods |= Modifiers::CONTROL,
            "SHIFT" => mods |= Modifiers::SHIFT,
            "SUPER" | "CMD" | "COMMAND" | "WIN" | "META" => mods |= Modifiers::SUPER,
            other => code = Some(key_code(other)?),
        }
    }
    Some(Shortcut::new(
        if mods.is_empty() { None } else { Some(mods) },
        code?,
    ))
}

/// Handle prism:// URLs: open?target=, show, quit.
fn handle_deeplink(app: &tauri::AppHandle, url: &str) {
    let rest = url
        .strip_prefix("prism://")
        .or_else(|| url.strip_prefix("prism:"))
        .unwrap_or(url);
    let (action, qs) = rest.split_once('?').unwrap_or((rest, ""));
    match action.trim_matches('/') {
        "show" => {
            if let Some(w) = app.get_webview_window("main") {
                let _ = w.show();
                let _ = w.set_focus();
                let _ = w.emit("prism:focus", ());
            }
        }
        "quit" => app.exit(0),
        "open" => {
            let target = qs.split('&').find_map(|kv| {
                kv.strip_prefix("target=")
                    .map(|v| v.replace('+', " ").replace("%20", " "))
            });
            if let Some(t) = target {
                let _ = tauri_plugin_opener::open_path(t, None::<&str>);
            }
        }
        _ => {}
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(ClipState::default())
        .manage(RemindersState::default())
        .manage(AlarmState::default())
        .manage(HotkeyState::default())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_deep_link::init())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, shortcut, event| {
                    if event.state() != ShortcutState::Pressed {
                        return;
                    }
                    let (toggle, clip) = {
                        let st = app.state::<HotkeyState>();
                        let g = st.0.lock().unwrap();
                        g.clone()
                    };
                    if toggle.as_ref() == Some(shortcut) {
                        toggle_main(app);
                    } else if clip.as_ref() == Some(shortcut) {
                        if let Some(w) = app.get_webview_window("main") {
                            let _ = w.show();
                            let _ = w.set_focus();
                            let _ = w.emit("prism:focus", ());
                            let _ = w.emit("prism:clipboard", ());
                        }
                    }
                })
                .build(),
        )
        .setup(|app| {
            use tauri_plugin_global_shortcut::GlobalShortcutExt;

            // Global hotkeys come from config.toml.
            let cfg = load_config(&app.handle());
            let toggle = parse_hotkey(&cfg.toggle_hotkey)
                .or_else(|| parse_hotkey("ALT+SPACE"));
            let clip = parse_hotkey(&cfg.clipboard_hotkey);
            if let Some(t) = &toggle {
                let _ = app.global_shortcut().register(t.clone());
            }
            if let Some(c) = &clip {
                let _ = app.global_shortcut().register(c.clone());
            }
            *app.state::<HotkeyState>().0.lock().unwrap() = (toggle, clip);

            // Register the prism:// deep-link scheme at runtime (Windows).
            {
                use tauri_plugin_deep_link::DeepLinkExt;
                let _ = app.deep_link().register("prism");
                let h = app.handle().clone();
                app.deep_link().on_open_url(move |ev| {
                    for url in ev.urls() {
                        handle_deeplink(&h, url.as_str());
                    }
                });
            }

            // Load persisted clipboard history, then poll for changes.
            {
                let handle = app.handle().clone();
                let path = clip_path(&handle);
                if let Ok(s) = std::fs::read_to_string(&path) {
                    if let Ok(v) = serde_json::from_str::<Vec<ClipItem>>(&s) {
                        let st = handle.state::<ClipState>();
                        *st.0.lock().unwrap() = v;
                    }
                }
                std::thread::spawn(move || {
                    let mut cb = match arboard::Clipboard::new() {
                        Ok(c) => c,
                        Err(_) => return,
                    };
                    let mut last = String::new();
                    let mut last_img: u64 = 0;
                    let mut counter: u64 = 0;
                    loop {
                        std::thread::sleep(std::time::Duration::from_millis(700));

                        let mut new_item: Option<ClipItem> = None;

                        // Text clipboard.
                        if let Ok(text) = cb.get_text() {
                            if !text.trim().is_empty() && text != last {
                                last = text.clone();
                                counter += 1;
                                new_item = Some(ClipItem {
                                    id: format!(
                                        "{}-{}",
                                        chrono::Local::now().timestamp_millis(),
                                        counter
                                    ),
                                    text,
                                    source: foreground_app(),
                                    ts: chrono::Local::now().timestamp(),
                                    kind: "text".into(),
                                    image: None,
                                });
                            }
                        }

                        // Image clipboard.
                        if new_item.is_none() {
                            if let Ok(img) = cb.get_image() {
                                use std::hash::{Hash, Hasher};
                                let mut h = std::collections::hash_map::DefaultHasher::new();
                                img.width.hash(&mut h);
                                img.height.hash(&mut h);
                                img.bytes.hash(&mut h);
                                let hv = h.finish();
                                if hv != last_img && img.width > 0 && img.height > 0 {
                                    last_img = hv;
                                    if let Some(buf) = image::RgbaImage::from_raw(
                                        img.width as u32,
                                        img.height as u32,
                                        img.bytes.into_owned(),
                                    ) {
                                        let mut out = std::io::Cursor::new(Vec::new());
                                        if image::DynamicImage::ImageRgba8(buf)
                                            .write_to(&mut out, image::ImageFormat::Png)
                                            .is_ok()
                                        {
                                            let b64 = base64::engine::general_purpose::STANDARD
                                                .encode(out.into_inner());
                                            counter += 1;
                                            new_item = Some(ClipItem {
                                                id: format!(
                                                    "{}-{}",
                                                    chrono::Local::now().timestamp_millis(),
                                                    counter
                                                ),
                                                text: String::new(),
                                                source: foreground_app(),
                                                ts: chrono::Local::now().timestamp(),
                                                kind: "image".into(),
                                                image: Some(b64),
                                            });
                                        }
                                    }
                                }
                            }
                        }

                        let Some(item) = new_item else { continue };
                        let st = handle.state::<ClipState>();
                        let mut items = st.0.lock().unwrap();
                        if item.kind == "image" {
                            items.retain(|c| c.image != item.image);
                        } else {
                            items.retain(|c| c.text != item.text || c.kind == "image");
                        }
                        items.insert(0, item);
                        items.truncate(200);
                        persist_clips(&clip_path(&handle), &items);
                    }
                });
            }

            // Load persisted reminders, then poll for ones that come due.
            {
                let handle = app.handle().clone();
                let rpath = reminders_path(&handle);
                if let Ok(s) = std::fs::read_to_string(&rpath) {
                    if let Ok(v) = serde_json::from_str::<Vec<Reminder>>(&s) {
                        let st = handle.state::<RemindersState>();
                        *st.0.lock().unwrap() = v;
                    }
                }
                std::thread::spawn(move || loop {
                    std::thread::sleep(std::time::Duration::from_secs(5));
                    let now = chrono::Local::now().timestamp();
                    let mut due: Vec<Reminder> = Vec::new();
                    {
                        let st = handle.state::<RemindersState>();
                        let mut items = st.0.lock().unwrap();
                        for r in items.iter_mut() {
                            if !r.fired && r.ts <= now {
                                r.fired = true;
                                due.push(r.clone());
                            }
                        }
                        if !due.is_empty() {
                            // Notification reminders are one-shot: drop them
                            // once shown. Fullscreen ones stay until dismissed.
                            items.retain(|r| {
                                !(r.fired && r.mode != "fullscreen")
                            });
                            persist_reminders(&reminders_path(&handle), &items);
                        }
                    }
                    let mut want_window = false;
                    for r in due {
                        if r.mode == "fullscreen" {
                            handle
                                .state::<AlarmState>()
                                .0
                                .lock()
                                .unwrap()
                                .push(r);
                            want_window = true;
                        } else {
                            let _ = handle
                                .notification()
                                .builder()
                                .title("Prism Reminder")
                                .body(&r.name)
                                .show();
                        }
                    }
                    if want_window {
                        show_alarm_window(&handle);
                    }
                });
            }

            // Auto-update: check GitHub every 3h and silently install a
            // newer release when one is available.
            {
                let handle = app.handle().clone();
                std::thread::spawn(move || {
                    // Small initial delay so startup isn't blocked.
                    std::thread::sleep(std::time::Duration::from_secs(30));
                    loop {
                        if let Ok(st) = do_update_check() {
                            if st.has_update && !st.tag.is_empty() {
                                let _ = do_install(&handle, &st.tag);
                            }
                        }
                        std::thread::sleep(std::time::Duration::from_secs(
                            UPDATE_INTERVAL_SECS,
                        ));
                    }
                });
            }

            // Hide when the launcher loses focus (lightweight, Raycast-style).
            if let Some(w) = app.get_webview_window("main") {
                let wc = w.clone();
                w.on_window_event(move |ev| {
                    if let tauri::WindowEvent::Focused(false) = ev {
                        let _ = wc.hide();
                    }
                });
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            list_entries,
            launch,
            run_system,
            load_settings,
            save_settings,
            image_data_url,
            icon_data_url,
            pick_image,
            hide,
            open_settings_window,
            close_settings_window,
            open_themes_window,
            theme_changed,
            open_config_file,
            web_search,
            run_mode,
            clipboard_history,
            clipboard_apply,
            clipboard_delete,
            clipboard_clear,
            reminders_list,
            reminder_add,
            reminder_delete,
            current_alarm,
            dismiss_alarm,
            update_check,
            update_releases,
            update_install
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
