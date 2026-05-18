#![allow(non_snake_case)]

use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

static CSS: Asset = asset!("/assets/styles.css");
static PRISM_ICON: Asset = asset!("/assets/prism.png");
static CLIPBOARD_ICON: Asset = asset!("/assets/clipboard_icon.png");

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
}

async fn call<T: for<'de> Deserialize<'de>>(cmd: &str, args: &impl Serialize) -> Option<T> {
    let a = serde_wasm_bindgen::to_value(args).ok()?;
    let v = invoke(cmd, a).await;
    serde_wasm_bindgen::from_value(v).ok()
}

#[derive(Serialize)]
struct NoArgs {}

#[derive(Serialize)]
struct PathArg {
    path: String,
}

#[derive(Serialize)]
struct ActionArg {
    action: String,
}

#[derive(Serialize)]
struct SettingsArg {
    settings: Settings,
}

#[derive(Serialize)]
struct IdArg {
    id: String,
}

#[derive(Serialize)]
struct NameArg {
    name: String,
}

#[derive(Serialize)]
struct QueryArg {
    query: String,
}

#[derive(Serialize)]
struct ReminderAddArg {
    name: String,
    when: String,
    mode: String,
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
struct ReminderView {
    id: String,
    name: String,
    mode: String,
    when: String,
    overdue: bool,
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
struct Reminder {
    id: String,
    name: String,
    #[serde(default)]
    mode: String,
}

#[derive(Serialize)]
struct TagArg {
    tag: String,
}

#[derive(Deserialize, Clone, PartialEq, Default)]
struct UpdateStatus {
    current: String,
    latest: String,
    tag: String,
    has_update: bool,
    url: String,
}

#[derive(Deserialize, Clone, PartialEq)]
struct ReleaseRow {
    tag: String,
    name: String,
    notes: String,
    published: String,
    current: bool,
    installable: bool,
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
struct Entry {
    id: String,
    title: String,
    subtitle: String,
    kind: String,
    action: String,
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
struct ClipView {
    id: String,
    text: String,
    preview: String,
    source: String,
    group: String,
    chars: usize,
    words: usize,
    kind: String,
    image: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
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
            selection: "#2a2a3a".into(),
            accent: "#7882ff".into(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
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
    search_url: String,
    aliases: std::collections::BTreeMap<String, String>,
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
            aliases: std::collections::BTreeMap::new(),
        }
    }
}

fn icon_name(e: &Entry) -> &'static str {
    match e.kind.as_str() {
        "settings" | "config" => "gear",
        "system" => "monitor",
        "clipboard" => "clipboard",
        "mode" => "gem",
        "websearch" => "file",
        "themes" => "gem",
        _ => "app",
    }
}

/// Inline vector icons (Lucide path data), monochrome via currentColor.
#[component]
fn Ic(name: String) -> Element {
    rsx! {
        svg {
            class: "ic-svg",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            {
                match name.as_str() {
                    "gear" => rsx! {
                        circle { cx: "12", cy: "12", r: "3" }
                        path { d: "M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 1 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 1 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 1 1-2.83-2.83l.06-.06A1.65 1.65 0 0 0 4.6 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 1 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 1 1 2.83-2.83l.06.06A1.65 1.65 0 0 0 9 4.6a1.65 1.65 0 0 0 1-1.51V3a2 2 0 1 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 1 1 2.83 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9z" }
                    },
                    "monitor" => rsx! {
                        rect { x: "2", y: "3", width: "20", height: "14", rx: "2" }
                        line { x1: "8", y1: "21", x2: "16", y2: "21" }
                        line { x1: "12", y1: "17", x2: "12", y2: "21" }
                    },
                    "clipboard" => rsx! {
                        rect { x: "8", y: "2", width: "8", height: "4", rx: "1" }
                        path { d: "M16 4h2a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2h2" }
                    },
                    "pin" => rsx! {
                        line { x1: "12", y1: "17", x2: "12", y2: "22" }
                        path { d: "M9 4h6l-1 6 3 3H7l3-3z" }
                    },
                    "bell" => rsx! {
                        path { d: "M18 8a6 6 0 0 0-12 0c0 7-3 9-3 9h18s-3-2-3-9" }
                        path { d: "M13.73 21a2 2 0 0 1-3.46 0" }
                    },
                    "enter" => rsx! {
                        polyline { points: "9 10 4 15 9 20" }
                        path { d: "M20 4v7a4 4 0 0 1-4 4H4" }
                    },
                    "gem" => rsx! {
                        path { d: "M6 3h12l4 6-10 12L2 9z" }
                        path { d: "M2 9h20" }
                    },
                    "file" => rsx! {
                        path { d: "M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" }
                        polyline { points: "14 2 14 8 20 8" }
                    },
                    _ => rsx! {
                        rect { x: "3", y: "3", width: "18", height: "18", rx: "4" }
                        circle { cx: "12", cy: "12", r: "3" }
                    },
                }
            }
        }
    }
}

fn classify(c: &ClipView) -> &'static str {
    if c.kind == "image" {
        return "image";
    }
    let t = c.text.trim();
    let tl = t.to_lowercase();
    let video_ext = [".mp4", ".mov", ".webm", ".mkv", ".avi", ".m4v"];
    let is_url = tl.starts_with("http://")
        || tl.starts_with("https://")
        || tl.starts_with("www.");
    if (is_url
        && (tl.contains("youtube.com")
            || tl.contains("youtu.be")
            || tl.contains("vimeo.com")))
        || video_ext.iter().any(|e| tl.ends_with(e))
    {
        return "video";
    }
    if is_url {
        return "link";
    }
    let hex = t.starts_with('#')
        && t.len() >= 4
        && t[1..].chars().all(|ch| ch.is_ascii_hexdigit());
    if hex || tl.starts_with("rgb(") || tl.starts_with("rgba(") || tl.starts_with("hsl(") {
        return "color";
    }
    let has_ext = t.rsplit('.').next().map(|e| e.len() <= 5 && !e.is_empty()).unwrap_or(false);
    if (t.len() >= 3 && t.as_bytes().get(1) == Some(&b':') && t.contains('\\'))
        || (t.starts_with('/') && has_ext && !t.contains(' ') && !t.contains('\n'))
    {
        return "file";
    }
    "text"
}

fn fuzzy(q: &str, title: &str) -> bool {
    let mut ti = title.chars();
    for qc in q.chars() {
        loop {
            match ti.next() {
                Some(tc) if tc == qc => break,
                Some(_) => continue,
                None => return false,
            }
        }
    }
    true
}

/// CSS custom properties a custom Theme overrides on :root.
const THEME_VARS: &[&str] = &[
    "--surface-base",
    "--text-primary",
    "--text-strong",
    "--surface-active",
    "--accent",
    "--accent-strong",
];

fn apply_css(s: &Settings) {
    // A custom theme is selected when `theme` matches a stored theme id.
    if let Some(t) = s.custom_themes.iter().find(|t| t.id == s.theme) {
        let bg = if t.bg_kind == "gradient" {
            format!("linear-gradient(145deg, {} 0%, {} 100%)", t.bg_start, t.bg_end)
        } else {
            t.bg_start.clone()
        };
        let light = t.appearance == "light";
        let js = format!(
            r##"(function(){{
  var r=document.documentElement.style;
  r.setProperty('--surface-base',"{bg}");
  r.setProperty('--bg-color',"{start}");
  r.setProperty('--text-primary',"{text}");
  r.setProperty('--text-strong',"{text}");
  r.setProperty('--surface-active',"{sel}");
  r.setProperty('--accent',"{acc}");
  r.setProperty('--accent-strong',"{acc}");
  document.body.classList.toggle('light',{light});
  document.body.classList.toggle('dark',{dark});
}})();"##,
            bg = bg,
            start = t.bg_start,
            text = t.text,
            sel = t.selection,
            acc = t.accent,
            light = light,
            dark = !light,
        );
        let _ = document::eval(&js);
        return;
    }

    // Built-in keyword theme: clear any custom-theme overrides first.
    let clear = THEME_VARS
        .iter()
        .map(|v| format!("r.removeProperty('{v}');"))
        .collect::<String>();
    let _ = document::eval(&format!(
        "(function(){{var r=document.documentElement.style;{clear}}})();"
    ));

    // theme: system | dark | light | custom
    let js = format!(
        r##"(function(){{
  var t="{theme}", custom="{custom}", c;
  if(t==="dark") c="#222222";
  else if(t==="light") c="#ffffff";
  else if(t==="custom") c=custom;
  else c=(window.matchMedia&&window.matchMedia('(prefers-color-scheme: dark)').matches)?"#222222":"#ffffff";
  document.documentElement.style.setProperty('--bg-color', c);
  function lum(h){{h=h.replace('#','');if(h.length===3)h=h[0]+h[0]+h[1]+h[1]+h[2]+h[2];
    var r=parseInt(h.substr(0,2),16)/255,g=parseInt(h.substr(2,2),16)/255,b=parseInt(h.substr(4,2),16)/255;
    return 0.299*r+0.587*g+0.114*b;}}
  var isLight = c[0]==='#' ? lum(c)>0.6 : false;
  document.body.classList.toggle('light', isLight);
  document.body.classList.toggle('dark', !isLight);
}})();"##,
        theme = s.theme,
        custom = s.bg_color
    );
    let _ = document::eval(&js);
}

fn scroll_to(idx: usize) {
    if idx == 0 {
        // First row: scroll its scrollable container fully to the top so the
        // section header ("All") above it also comes into view.
        let _ = document::eval(
            "var e=document.getElementById('row-0');\
             if(e){var p=e.parentElement;\
             while(p&&p.scrollHeight<=p.clientHeight)p=p.parentElement;\
             if(p)p.scrollTop=0;}",
        );
        return;
    }
    let _ = document::eval(&format!(
        "var e=document.getElementById('row-{idx}');\
         if(e)e.scrollIntoView({{block:'nearest'}});"
    ));
}

async fn save(mut settings: Signal<Settings>, st: Settings) {
    let _: Option<()> = call("save_settings", &SettingsArg { settings: st.clone() }).await;
    settings.set(st);
}

async fn do_activate(
    settings: Signal<Settings>,
    mut show_settings: Signal<bool>,
    mut show_clipboard: Signal<bool>,
    mut show_reminders: Signal<bool>,
    mut show_updates: Signal<bool>,
    flat: Vec<Entry>,
    idx: usize,
) {
    let Some(e) = flat.get(idx).cloned() else {
        return;
    };
    match e.kind.as_str() {
        "settings" => show_settings.set(true),
        "themes" => {
            let _: Option<()> = call("open_themes_window", &NoArgs {}).await;
        }
        "config" => {
            let _: Option<()> = call("open_config_file", &NoArgs {}).await;
        }
        "mode" => {
            let _: Option<()> = call("run_mode", &NameArg { name: e.action.clone() }).await;
        }
        "websearch" => {
            let _: Option<()> =
                call("web_search", &QueryArg { query: e.action.clone() }).await;
        }
        "clipboard" => show_clipboard.set(true),
        "reminders" => show_reminders.set(true),
        "updates" => show_updates.set(true),
        "system" => {
            let _: Option<()> =
                call("run_system", &ActionArg { action: e.action.clone() }).await;
        }
        _ => {
            let _: Option<()> =
                call("launch", &PathArg { path: e.action.clone() }).await;
        }
    }
    let mut st = (settings)();
    st.recent.retain(|x| x != &e.id);
    st.recent.insert(0, e.id.clone());
    st.recent.truncate(8);
    save(settings, st).await;
}

async fn do_toggle_pin(
    settings: Signal<Settings>,
    mut show_actions: Signal<bool>,
    flat: Vec<Entry>,
    idx: usize,
) {
    let Some(e) = flat.get(idx).cloned() else {
        return;
    };
    let mut st = (settings)();
    if st.pinned.contains(&e.id) {
        st.pinned.retain(|x| x != &e.id);
    } else {
        st.pinned.push(e.id.clone());
    }
    save(settings, st).await;
    show_actions.set(false);
}

fn url_query() -> String {
    web_sys::window()
        .and_then(|w| w.location().search().ok())
        .unwrap_or_default()
}

pub fn App() -> Element {
    // The fullscreen reminder overlay loads the same bundle with
    // `?view=reminder`; render the alarm screen instead of the launcher.
    if url_query().contains("view=reminder") {
        return rsx! { ReminderAlarm {} };
    }
    // The Theme Studio is its own decorated window (`?view=themes`).
    if url_query().contains("view=themes") {
        return rsx! { ThemeStudio {} };
    }

    let mut entries = use_signal(Vec::<Entry>::new);
    let mut settings = use_signal(Settings::default);
    let mut query = use_signal(String::new);
    let mut sel = use_signal(|| 0usize);
    let mut show_settings = use_signal(|| false);
    let mut show_clipboard = use_signal(|| false);
    let mut show_reminders = use_signal(|| false);
    let mut show_updates = use_signal(|| false);
    let mut show_actions = use_signal(|| false);
    let mut icons = use_signal(std::collections::HashMap::<String, String>::new);
    let mut expanded = use_signal(|| true);
    let mut nav = use_signal(|| false);
    // None = focus is in the search field; Some(i) = i-th pinned dock circle.
    let mut dock_sel = use_signal(|| None::<usize>);

    use_future(move || async move {
        if let Some(list) = call::<Vec<Entry>>("list_entries", &NoArgs {}).await {
            entries.set(list.clone());
            spawn(async move {
                for e in list.into_iter().filter(|e| e.kind == "app") {
                    if let Some(url) = call::<Option<String>>(
                        "icon_data_url",
                        &PathArg { path: e.action.clone() },
                    )
                    .await
                    .flatten()
                    {
                        icons.write().insert(e.id.clone(), url);
                    }
                }
            });
        }
        if let Some(s) = call::<Settings>("load_settings", &NoArgs {}).await {
            apply_css(&s);
            expanded.set(!s.collapsed);
            settings.set(s);
        } else {
            apply_css(&settings());
        }
        let _ = document::eval(
            "setTimeout(function(){var e=document.getElementById('search');if(e)e.focus();},40);",
        );
    });

    // Reset to collapsed state every time the launcher is summoned (Alt+Space).
    use_future(move || async move {
        let mut e = document::eval(
            "if(window.__TAURI__)window.__TAURI__.event.listen('prism:focus',function(){dioxus.send(1);});",
        );
        loop {
            if e.recv::<i32>().await.is_err() {
                break;
            }
            let c = settings().collapsed;
            query.set(String::new());
            sel.set(0);
            nav.set(false);
            dock_sel.set(None);
            expanded.set(!c);
            show_settings.set(false);
            show_clipboard.set(false);
            show_reminders.set(false);
            show_updates.set(false);
            show_actions.set(false);
            let _ = document::eval(
                "setTimeout(function(){var x=document.getElementById('search');if(x)x.focus();},20);",
            );
        }
    });

    // Clipboard hotkey (from config.toml) opens the clipboard view.
    use_future(move || async move {
        let mut e = document::eval(
            "if(window.__TAURI__)window.__TAURI__.event.listen('prism:clipboard',function(){dioxus.send(1);});",
        );
        loop {
            if e.recv::<i32>().await.is_err() {
                break;
            }
            expanded.set(true);
            show_clipboard.set(true);
        }
    });

    // Re-apply styling live when Theme Studio saves/applies a theme.
    use_future(move || async move {
        let mut e = document::eval(
            "if(window.__TAURI__)window.__TAURI__.event.listen('prism:theme',function(){dioxus.send(1);});",
        );
        loop {
            if e.recv::<i32>().await.is_err() {
                break;
            }
            if let Some(s) = call::<Settings>("load_settings", &NoArgs {}).await {
                apply_css(&s);
                expanded.set(!s.collapsed);
                settings.set(s);
            }
        }
    });

    // Re-apply theme when the OS light/dark preference changes (system mode).
    use_future(move || async move {
        let mut e = document::eval(
            "var m=window.matchMedia&&window.matchMedia('(prefers-color-scheme: dark)');\
             if(m&&m.addEventListener)m.addEventListener('change',function(){dioxus.send(1);});",
        );
        loop {
            if e.recv::<i32>().await.is_err() {
                break;
            }
            apply_css(&settings());
        }
    });

    let q = query();
    let all = entries();
    let s = settings();

    let mut sections: Vec<(String, Vec<Entry>)> = Vec::new();
    if q.trim().is_empty() {
        let pinned: Vec<Entry> = s
            .pinned
            .iter()
            .filter_map(|id| all.iter().find(|e| &e.id == id).cloned())
            .collect();
        let recent: Vec<Entry> = s
            .recent
            .iter()
            .filter(|id| !s.pinned.contains(*id))
            .filter_map(|id| all.iter().find(|e| &e.id == id).cloned())
            .collect();
        let used: std::collections::HashSet<&String> =
            s.pinned.iter().chain(s.recent.iter()).collect();
        let rest: Vec<Entry> = all
            .iter()
            .filter(|e| !used.contains(&e.id))
            .cloned()
            .collect();
        if !pinned.is_empty() {
            sections.push(("PINNED".into(), pinned));
        }
        // Recents first (no separate header), then everything else.
        let mut others = recent;
        others.extend(rest);
        if !others.is_empty() {
            sections.push(("ALL".into(), others));
        }
    } else {
        let qt = q.trim();
        let ql = qt.to_lowercase();

        // Alias: typing an alias key jumps straight to its target.
        if let Some(target) = s.aliases.get(&ql) {
            let tl = target.to_lowercase();
            if let Some(hit) = all
                .iter()
                .find(|e| e.title.to_lowercase().contains(&tl))
                .cloned()
            {
                sections.push((format!("ALIAS · {ql}"), vec![hit]));
            }
        }

        let mut hits: Vec<Entry> = all
            .iter()
            .filter(|e| fuzzy(&ql, &e.title.to_lowercase()))
            .cloned()
            .collect();
        hits.sort_by_key(|e| {
            let t = e.title.to_lowercase();
            (!t.starts_with(&ql), t.len())
        });
        sections.push(("RESULTS".into(), hits));

        // "add ? to search the web" (RustCast convention).
        if let Some(term) = qt.strip_suffix('?') {
            let term = term.trim();
            if !term.is_empty() {
                sections.push((
                    "WEB SEARCH".into(),
                    vec![Entry {
                        id: "prism.websearch".into(),
                        title: format!("Search the web for \u{201c}{term}\u{201d}"),
                        subtitle: "Web Search".into(),
                        kind: "websearch".into(),
                        action: term.to_string(),
                    }],
                ));
            }
        }
    }

    let flat: Vec<Entry> = sections.iter().flat_map(|(_, v)| v.clone()).collect();
    let total = flat.len();
    let cur = if total == 0 {
        0
    } else {
        (*sel.read()).min(total - 1)
    };

    let flat_key = flat.clone();
    let act_pin = s.act_pin.to_lowercase();
    let act_open = s.act_open.to_lowercase();

    // Pinned dock entries (mirrors the dock rendered beside the collapsed pill).
    let pinned_flat: Vec<Entry> = s
        .pinned
        .iter()
        .filter_map(|id| all.iter().find(|e| &e.id == id).cloned())
        .take(6)
        .collect();
    // The dock is only navigable while the launcher is collapsed.
    let dock_len = if expanded() || !q.trim().is_empty() {
        0
    } else {
        pinned_flat.len()
    };
    let pinned_key = pinned_flat.clone();

    let onkey = move |ev: KeyboardEvent| {
        let m = ev.modifiers();
        match ev.key() {
            Key::Escape => {
                ev.prevent_default();
                if show_actions() {
                    show_actions.set(false);
                } else if show_settings() {
                    show_settings.set(false);
                } else if show_reminders() {
                    show_reminders.set(false);
                } else if show_updates() {
                    show_updates.set(false);
                } else {
                    spawn(async move {
                        let _: Option<()> = call("hide", &NoArgs {}).await;
                    });
                }
            }
            Key::ArrowDown => {
                ev.prevent_default();
                expanded.set(true);
                dock_sel.set(None);
                if total > 0 {
                    let n = if !nav() { 0 } else { (cur + 1).min(total - 1) };
                    nav.set(true);
                    sel.set(n);
                    scroll_to(n);
                }
            }
            Key::ArrowUp => {
                ev.prevent_default();
                if total > 0 {
                    let n = if !nav() { 0 } else { cur.saturating_sub(1) };
                    nav.set(true);
                    sel.set(n);
                    scroll_to(n);
                }
            }
            Key::ArrowRight => {
                if dock_len > 0 {
                    ev.prevent_default();
                    let n = match dock_sel() {
                        None => 0,
                        Some(i) => (i + 1).min(dock_len - 1),
                    };
                    dock_sel.set(Some(n));
                }
            }
            Key::ArrowLeft => {
                if dock_len > 0 {
                    if let Some(i) = dock_sel() {
                        ev.prevent_default();
                        if i == 0 {
                            dock_sel.set(None);
                            let _ = document::eval(
                                "var e=document.getElementById('search');if(e)e.focus();",
                            );
                        } else {
                            dock_sel.set(Some(i - 1));
                        }
                    }
                }
            }
            Key::Enter => {
                ev.prevent_default();
                if let Some(i) = dock_sel() {
                    let pv = pinned_key.clone();
                    spawn(do_activate(settings, show_settings, show_clipboard, show_reminders, show_updates, pv, i));
                } else {
                    let flat = flat_key.clone();
                    spawn(do_activate(settings, show_settings, show_clipboard, show_reminders, show_updates, flat, cur));
                }
            }
            Key::Character(c) => {
                // Action-popup shortcuts: only while the popup is open.
                if show_actions() && !m.ctrl() {
                    let lc = c.to_lowercase();
                    if !act_pin.is_empty() && lc == act_pin {
                        ev.prevent_default();
                        let flat = flat_key.clone();
                        spawn(do_toggle_pin(settings, show_actions, flat, cur));
                        return;
                    }
                    if !act_open.is_empty() && lc == act_open {
                        ev.prevent_default();
                        show_actions.set(false);
                        let flat = flat_key.clone();
                        spawn(do_activate(settings, show_settings, show_clipboard, show_reminders, show_updates, flat, cur));
                        return;
                    }
                }
                if m.ctrl() && c == "k" {
                    ev.prevent_default();
                    show_actions.toggle();
                }
            }
            _ => {}
        }
    };

    let mut qk = 0usize;


    let collapsed_setting = s.collapsed;
    let open = expanded() || !q.trim().is_empty();

    // Pinned entries shown as a dock of circles beside the collapsed pill.
    let pinned_entries: Vec<Entry> = s
        .pinned
        .iter()
        .filter_map(|id| all.iter().find(|e| &e.id == id).cloned())
        .take(6)
        .collect();

    rsx! {
        link { rel: "stylesheet", href: CSS }
        div {
            class: "shell",
            tabindex: "0",
            oncontextmenu: move |e| e.prevent_default(),
            onkeydown: onkey,
            onclick: move |_| {
                spawn(async move {
                    let _: Option<()> = call("hide", &NoArgs {}).await;
                });
            },

            if show_clipboard() {
                ClipboardView { backdrop_cls: "backdrop".to_string(), on_close: move |_| show_clipboard.set(false) }
            } else if show_reminders() {
                RemindersView { on_close: move |_| show_reminders.set(false) }
            } else if show_updates() {
                UpdatesView { on_close: move |_| show_updates.set(false) }
            } else if show_settings() {
                SettingsView { settings, on_close: move |_| show_settings.set(false) }
            } else {
                div { class: if open { "dock-row" } else { "dock-row pill" },
                div {
                    class: if open { "panel" } else { "panel collapsed" },
                    onclick: move |e| e.stop_propagation(),
                    div { class: "backdrop" }
                    div { class: "searchbar",
                        img {
                            class: "search-ic",
                            src: PRISM_ICON,
                            alt: "Prism",
                        }
                        input {
                            id: "search",
                            autofocus: true,
                            autocomplete: "off",
                            autocorrect: "off",
                            autocapitalize: "off",
                            spellcheck: false,
                            placeholder: "Search for apps and commands",
                            value: "{query}",
                            oninput: move |e| {
                                let v = e.value();
                                let empty = v.trim().is_empty();
                                expanded.set(if empty { !collapsed_setting } else { true });
                                nav.set(!empty);
                                dock_sel.set(None);
                                query.set(v);
                                sel.set(0);
                            }
                        }
                    }
                    div { class: if open { "collapsible open" } else { "collapsible" },
                    div { class: "cwrap",
                    div { class: "divider" }
                    div { class: "list",
                        if total == 0 {
                            div { class: "empty", "No results" }
                        }
                        for (label , items) in sections.iter() {
                            div { class: "section-label", "{label}" }
                            for e in items.iter() {
                                {
                                    let idx = qk;
                                    qk += 1;
                                    let pinned = s.pinned.contains(&e.id);
                                    let flat_row = flat.clone();
                                    let icon = icons.read().get(&e.id).cloned();
                                    rsx! {
                                        div {
                                            id: "row-{idx}",
                                            class: if idx == cur { "row selected" } else { "row" },
                                            onclick: move |ev| {
                                                ev.stop_propagation();
                                                sel.set(idx);
                                                spawn(do_activate(settings, show_settings, show_clipboard, show_reminders, show_updates, flat_row.clone(), idx));
                                            },
                                            if let Some(u) = icon {
                                                img { class: "icon-img", src: "{u}" }
                                            } else if e.kind == "clipboard" {
                                                img { class: "icon-img", src: CLIPBOARD_ICON }
                                            } else {
                                                div { class: "icon", Ic { name: icon_name(e).to_string() } }
                                            }
                                            div { class: "title", "{e.title}" }
                                            div { class: "subtitle", "{e.subtitle}" }
                                            div { class: "spacer" }
                                            if pinned {
                                                div { class: "chips",
                                                    div { class: "chip pin-chip", Ic { name: "pin".to_string() } }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    div { class: "footer",
                        div { class: "brand",
                            Ic { name: "gem".to_string() }
                            span { "Prism" }
                        }
                        div { class: "acts",
                            span { class: "strong", "Open" }
                            div { class: "chip", "Enter" }
                            span { "Actions" }
                            div { class: "chip", "Ctrl" }
                            div { class: "chip", "K" }
                        }
                    }
                    if show_actions() {
                        {
                            let is_pinned = flat.get(cur).map(|e| s.pinned.contains(&e.id)).unwrap_or(false);
                            let flat_a = flat.clone();
                            let flat_b = flat.clone();
                            rsx! {
                                div { class: "actions-pop",
                                    div {
                                        class: "row",
                                        onclick: move |_| { spawn(do_toggle_pin(settings, show_actions, flat_a.clone(), cur)); },
                                        div { class: "icon", Ic { name: "pin".to_string() } }
                                        div { class: "title", if is_pinned { "Unpin" } else { "Pin" } }
                                        div { class: "spacer" }
                                        div { class: "chips",
                                            div { class: "chip", "{s.act_pin.to_uppercase()}" }
                                        }
                                    }
                                    div {
                                        class: "row",
                                        onclick: move |_| {
                                            show_actions.set(false);
                                            spawn(do_activate(settings, show_settings, show_clipboard, show_reminders, show_updates, flat_b.clone(), cur));
                                        },
                                        div { class: "icon", Ic { name: "enter".to_string() } }
                                        div { class: "title", "Open" }
                                        div { class: "spacer" }
                                        div { class: "chips",
                                            div { class: "chip", "{s.act_open.to_uppercase()}" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    }
                    }
                }
                if !open && !pinned_entries.is_empty() {
                    div { class: "dock",
                        for (i , pe) in pinned_entries.iter().enumerate() {
                            {
                                let ic = icons.read().get(&pe.id).cloned();
                                let pv = pinned_entries.clone();
                                rsx! {
                                    button {
                                        class: if dock_sel() == Some(i) { "circle selected" } else { "circle" },
                                        title: "{pe.title}",
                                        style: "animation-delay:{i*40}ms",
                                        onclick: move |e| {
                                            e.stop_propagation();
                                            spawn(do_activate(settings, show_settings, show_clipboard, show_reminders, show_updates, pv.clone(), i));
                                        },
                                        if let Some(u) = ic {
                                            img { class: "circle-img", src: "{u}" }
                                        } else {
                                            Ic { name: icon_name(pe).to_string() }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                }
            }
        }
    }
}

#[component]
fn SettingsView(settings: Signal<Settings>, on_close: EventHandler<()>) -> Element {
    let s = settings();

    rsx! {
        div {
            class: "panel",
            onclick: move |e| e.stop_propagation(),
            div { class: "backdrop" }
            div { class: "settings",
                div { class: "set-header",
                    button {
                        class: "cb-back",
                        onclick: move |e| { e.stop_propagation(); on_close.call(()); },
                        svg {
                            class: "back-ic",
                            view_box: "0 0 24 24",
                            fill: "none",
                            stroke: "currentColor",
                            stroke_width: "2.2",
                            stroke_linecap: "round",
                            stroke_linejoin: "round",
                            polyline { points: "15 18 9 12 15 6" }
                        }
                    }
                    h2 { "Prism Settings" }
                }
                div { class: "set-row",
                    div { class: "set-k",
                        div { class: "set-t", "Theme" }
                        div { class: "set-d", "System follows Windows. Dark = #222, Light = white." }
                    }
                    select {
                        class: "cb-type",
                        value: "{s.theme}",
                        onchange: move |e| async move {
                            let mut x = (settings)();
                            x.theme = e.value();
                            apply_css(&x);
                            save(settings, x).await;
                        },
                        option { value: "system", "System" }
                        option { value: "dark", "Dark" }
                        option { value: "light", "Light" }
                        option { value: "custom", "Custom" }
                    }
                }
                if s.theme == "custom" {
                    div { class: "set-row",
                        div { class: "set-k",
                            div { class: "set-t", "Custom color" }
                            div { class: "set-d", "Solid color behind the launcher." }
                        }
                        input {
                            r#type: "color",
                            class: "set-color",
                            value: "{s.bg_color}",
                            oninput: move |e| async move {
                                let mut x = (settings)();
                                x.bg_color = e.value();
                                apply_css(&x);
                                save(settings, x).await;
                            }
                        }
                    }
                }
                div { class: "set-row",
                    div { class: "set-k",
                        div { class: "set-t", "Start collapsed" }
                        div { class: "set-d", "Show only the search field until you type." }
                    }
                    label { class: "switch",
                        input {
                            r#type: "checkbox",
                            checked: s.collapsed,
                            onchange: move |e| async move {
                                let mut x = (settings)();
                                x.collapsed = e.checked();
                                save(settings, x).await;
                            }
                        }
                        span { class: "slider" }
                    }
                }
                div { class: "set-row",
                    div { class: "set-k",
                        div { class: "set-t", "Pin / Unpin shortcut" }
                        div { class: "set-d", "Key (while the Ctrl+K actions menu is open)." }
                    }
                    input {
                        r#type: "text",
                        class: "set-key",
                        maxlength: "1",
                        value: "{s.act_pin}",
                        oninput: move |e| async move {
                            let mut x = (settings)();
                            x.act_pin = e.value().chars().next().map(|c| c.to_string()).unwrap_or_default();
                            save(settings, x).await;
                        }
                    }
                }
                div { class: "set-row",
                    div { class: "set-k",
                        div { class: "set-t", "Open shortcut" }
                        div { class: "set-d", "Key (while the Ctrl+K actions menu is open)." }
                    }
                    input {
                        r#type: "text",
                        class: "set-key",
                        maxlength: "1",
                        value: "{s.act_open}",
                        oninput: move |e| async move {
                            let mut x = (settings)();
                            x.act_open = e.value().chars().next().map(|c| c.to_string()).unwrap_or_default();
                            save(settings, x).await;
                        }
                    }
                }
            }
        }
    }
}


#[component]
fn ClipboardView(backdrop_cls: String, on_close: EventHandler<()>) -> Element {
    let mut items = use_signal(Vec::<ClipView>::new);
    let mut sel = use_signal(|| 0usize);
    let mut filter = use_signal(String::new);
    let mut kind_filter = use_signal(|| "all".to_string());

    use_future(move || async move {
        if let Some(v) = call::<Vec<ClipView>>("clipboard_history", &NoArgs {}).await {
            items.set(v);
        }
        let _ = document::eval(
            "setTimeout(function(){var e=document.getElementById('cb-search');if(e)e.focus();},40);",
        );
    });

    let f = filter().to_lowercase();
    let kf = kind_filter();
    let all = items();
    let filtered: Vec<ClipView> = all
        .iter()
        .filter(|c| f.is_empty() || c.text.to_lowercase().contains(&f))
        .filter(|c| kf == "all" || classify(c) == kf)
        .cloned()
        .collect();
    let total = filtered.len();
    let cur = if total == 0 { 0 } else { (*sel.read()).min(total - 1) };
    let detail = filtered.get(cur).cloned();

    let filtered_k = filtered.clone();
    let onkey = move |ev: KeyboardEvent| {
        let filtered = filtered_k.clone();
        async move {
            match ev.key() {
                Key::Escape => {
                    ev.prevent_default();
                    on_close.call(());
                }
                Key::ArrowDown => {
                    ev.prevent_default();
                    if total > 0 {
                        let n = (cur + 1).min(total - 1);
                        sel.set(n);
                        let _ = document::eval(&format!(
                            "var e=document.getElementById('cb-{n}');if(e)e.scrollIntoView({{block:'nearest'}});"
                        ));
                    }
                }
                Key::ArrowUp => {
                    ev.prevent_default();
                    if total > 0 {
                        let n = cur.saturating_sub(1);
                        sel.set(n);
                        let _ = document::eval(&format!(
                            "var e=document.getElementById('cb-{n}');if(e)e.scrollIntoView({{block:'nearest'}});"
                        ));
                    }
                }
                Key::Enter => {
                    ev.prevent_default();
                    if let Some(c) = filtered.get(cur) {
                        let _: Option<()> =
                            call("clipboard_apply", &IdArg { id: c.id.clone() }).await;
                    }
                }
                _ => {}
            }
        }
    };

    let mut last_group = String::new();
    let mut idx = 0usize;

    rsx! {
        div {
            class: "panel",
            onclick: move |e| e.stop_propagation(),
            onkeydown: onkey,
            div { class: "{backdrop_cls}" }
            div { class: "cb",
                div { class: "cb-header",
                    button {
                        class: "cb-back",
                        onclick: move |e| { e.stop_propagation(); on_close.call(()); },
                        svg {
                            class: "back-ic",
                            view_box: "0 0 24 24",
                            fill: "none",
                            stroke: "currentColor",
                            stroke_width: "2.2",
                            stroke_linecap: "round",
                            stroke_linejoin: "round",
                            polyline { points: "15 18 9 12 15 6" }
                        }
                    }
                    input {
                        id: "cb-search",
                        class: "cb-search",
                        autofocus: true,
                        autocomplete: "off",
                        autocorrect: "off",
                        autocapitalize: "off",
                        spellcheck: false,
                        placeholder: "Type to filter entries...",
                        value: "{filter}",
                        oninput: move |e| { filter.set(e.value()); sel.set(0); }
                    }
                    select {
                        class: "cb-type",
                        value: "{kind_filter}",
                        onchange: move |e| { kind_filter.set(e.value()); sel.set(0); },
                        option { value: "all", "All Types" }
                        option { value: "text", "Text" }
                        option { value: "link", "Link" }
                        option { value: "image", "Image" }
                        option { value: "video", "Video" }
                        option { value: "color", "Color" }
                        option { value: "file", "File" }
                    }
                }
                div { class: "cb-body",
                    div { class: "cb-list",
                        if total == 0 {
                            div { class: "empty", "No clipboard history" }
                        }
                        for c in filtered.iter() {
                            {
                                let i = idx;
                                idx += 1;
                                let show_group = c.group != last_group;
                                last_group = c.group.clone();
                                let gl = c.group.clone();
                                rsx! {
                                    if show_group {
                                        div { class: "cb-group", "{gl}" }
                                    }
                                    div {
                                        id: "cb-{i}",
                                        class: if i == cur { "cb-item selected" } else { "cb-item" },
                                        onclick: {
                                            let cid = c.id.clone();
                                            move |e: MouseEvent| {
                                                e.stop_propagation();
                                                sel.set(i);
                                                let cid = cid.clone();
                                                spawn(async move {
                                                    let _: Option<()> = call(
                                                        "clipboard_apply",
                                                        &IdArg { id: cid },
                                                    )
                                                    .await;
                                                });
                                            }
                                        },
                                        if let Some(img) = c.image.clone() {
                                            img { class: "cb-thumb", src: "{img}" }
                                            span { class: "tx", "Image" }
                                        } else {
                                            span { class: "ic", Ic { name: "file".to_string() } }
                                            span { class: "tx", "{c.preview}" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    div { class: "cb-detail",
                        if let Some(d) = detail.clone() {
                            if let Some(img) = d.image.clone() {
                                div { class: "cb-imgwrap",
                                    img { class: "cb-bigimg", src: "{img}" }
                                }
                            } else {
                                div { class: "cb-pre", "{d.text}" }
                            }
                            div { class: "cb-info",
                                h3 { "Information" }
                                div { class: "kv",
                                    span { class: "k", "Source" }
                                    span { class: "v", "{d.source}" }
                                }
                                div { class: "kv",
                                    span { class: "k", "Type" }
                                    span { class: "v", if d.kind == "image" { "Image" } else { "Text" } }
                                }
                                div { class: "kv",
                                    span { class: "k", "Characters" }
                                    span { class: "v", "{d.chars}" }
                                }
                                div { class: "kv",
                                    span { class: "k", "Words" }
                                    span { class: "v", "{d.words}" }
                                }
                            }
                        } else {
                            div { class: "cb-pre", "" }
                        }
                    }
                }
                div { class: "footer",
                    div { class: "brand",
                        span { "\u{1f4cb}" }
                        span { "Clipboard History" }
                    }
                    div { class: "acts",
                        span { class: "strong", "Paste" }
                        div { class: "chip", "\u{21b5}" }
                        span { "Actions" }
                        div { class: "chip", "Ctrl" }
                        div { class: "chip", "K" }
                    }
                }
            }
        }
    }
}

#[component]
fn RemindersView(on_close: EventHandler<()>) -> Element {
    let mut items = use_signal(Vec::<ReminderView>::new);
    let mut name = use_signal(String::new);
    let mut when = use_signal(String::new);
    let mut mode = use_signal(|| "notification".to_string());
    let mut err = use_signal(String::new);

    let reload = move || {
        spawn(async move {
            if let Some(v) = call::<Vec<ReminderView>>("reminders_list", &NoArgs {}).await {
                items.set(v);
            }
        });
    };

    use_future(move || async move {
        if let Some(v) = call::<Vec<ReminderView>>("reminders_list", &NoArgs {}).await {
            items.set(v);
        }
        let _ = document::eval(
            "setTimeout(function(){var e=document.getElementById('rem-name');if(e)e.focus();},40);",
        );
    });

    let add = move |_| {
        let n = name();
        let w = when();
        let m = mode();
        if n.trim().is_empty() {
            err.set("Give the reminder a name.".into());
            return;
        }
        if w.is_empty() {
            err.set("Pick a date and time.".into());
            return;
        }
        spawn(async move {
            let ok: Option<()> = call(
                "reminder_add",
                &ReminderAddArg { name: n, when: w, mode: m },
            )
            .await;
            if ok.is_some() {
                err.set(String::new());
                name.set(String::new());
                when.set(String::new());
                if let Some(v) =
                    call::<Vec<ReminderView>>("reminders_list", &NoArgs {}).await
                {
                    items.set(v);
                }
            } else {
                err.set("Could not save reminder.".into());
            }
        });
    };

    let onkey = move |ev: KeyboardEvent| {
        if ev.key() == Key::Escape {
            ev.prevent_default();
            on_close.call(());
        }
    };

    rsx! {
        div {
            class: "panel",
            onclick: move |e| e.stop_propagation(),
            onkeydown: onkey,
            div { class: "backdrop" }
            div { class: "rem",
                div { class: "rem-header",
                    button {
                        class: "cb-back",
                        onclick: move |e| { e.stop_propagation(); on_close.call(()); },
                        svg {
                            class: "back-ic",
                            view_box: "0 0 24 24",
                            fill: "none",
                            stroke: "currentColor",
                            stroke_width: "2.2",
                            stroke_linecap: "round",
                            stroke_linejoin: "round",
                            polyline { points: "15 18 9 12 15 6" }
                        }
                    }
                    h2 { "Reminders" }
                }
                div { class: "rem-form",
                    input {
                        id: "rem-name",
                        class: "rem-in",
                        autocomplete: "off",
                        placeholder: "What should I remind you about?",
                        value: "{name}",
                        oninput: move |e| name.set(e.value()),
                    }
                    div { class: "rem-form-row",
                        input {
                            r#type: "datetime-local",
                            class: "rem-in rem-when",
                            value: "{when}",
                            oninput: move |e| when.set(e.value()),
                        }
                        select {
                            class: "cb-type",
                            value: "{mode}",
                            onchange: move |e| mode.set(e.value()),
                            option { value: "notification", "Notification" }
                            option { value: "fullscreen", "Full screen" }
                        }
                        button {
                            class: "rem-add",
                            onclick: add,
                            "Add"
                        }
                    }
                    if !err().is_empty() {
                        div { class: "rem-err", "{err}" }
                    }
                }
                div { class: "rem-list",
                    if items().is_empty() {
                        div { class: "empty", "No reminders yet" }
                    }
                    for r in items().iter() {
                        {
                            let id = r.id.clone();
                            rsx! {
                                div { class: "rem-item",
                                    div { class: "rem-bell",
                                        Ic { name: "bell".to_string() }
                                    }
                                    div { class: "rem-meta",
                                        div { class: "rem-name", "{r.name}" }
                                        div {
                                            class: if r.overdue { "rem-when-lbl overdue" } else { "rem-when-lbl" },
                                            "{r.when}"
                                        }
                                    }
                                    div { class: "rem-mode",
                                        if r.mode == "fullscreen" { "Full screen" } else { "Notification" }
                                    }
                                    button {
                                        class: "rem-del",
                                        onclick: move |e| {
                                            e.stop_propagation();
                                            let id = id.clone();
                                            spawn(async move {
                                                let _: Option<()> =
                                                    call("reminder_delete", &IdArg { id }).await;
                                            });
                                            reload();
                                        },
                                        "Delete"
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn ReminderAlarm() -> Element {
    let mut label = use_signal(String::new);

    use_future(move || async move {
        // Looping chime until dismissed.
        let _ = document::eval(
            r#"(function(){try{window.__prismStop=false;
var C=window.AudioContext||window.webkitAudioContext;if(!C)return;var x=new C();
function b(){if(window.__prismStop){try{x.close()}catch(e){}return;}
var o=x.createOscillator(),g=x.createGain();o.type='sine';o.frequency.value=880;
o.connect(g);g.connect(x.destination);g.gain.setValueAtTime(0.0001,x.currentTime);
g.gain.exponentialRampToValueAtTime(0.35,x.currentTime+0.05);
g.gain.exponentialRampToValueAtTime(0.0001,x.currentTime+0.5);
o.start();o.stop(x.currentTime+0.55);setTimeout(b,900);}b();}catch(e){}})();"#,
        );
        if let Some(r) = call::<Option<Reminder>>("current_alarm", &NoArgs {}).await.flatten() {
            label.set(r.name);
        }
        // Refresh the text if another alarm is queued behind this one.
        let mut e = document::eval(
            "if(window.__TAURI__)window.__TAURI__.event.listen('prism:alarm',function(){dioxus.send(1);});",
        );
        loop {
            if e.recv::<i32>().await.is_err() {
                break;
            }
            if let Some(r) =
                call::<Option<Reminder>>("current_alarm", &NoArgs {}).await.flatten()
            {
                label.set(r.name);
            }
        }
    });

    let dismiss = move |_| {
        spawn(async move {
            let _ = document::eval("window.__prismStop=true;");
            let _: Option<()> = call("dismiss_alarm", &NoArgs {}).await;
        });
    };

    rsx! {
        link { rel: "stylesheet", href: CSS }
        div { class: "alarm",
            div { class: "alarm-bell", Ic { name: "bell".to_string() } }
            div { class: "alarm-kicker", "Prism Reminder" }
            div { class: "alarm-title", "{label}" }
            button { class: "alarm-dismiss", onclick: dismiss, "Dismiss" }
        }
    }
}

#[component]
fn UpdatesView(on_close: EventHandler<()>) -> Element {
    let mut status = use_signal(UpdateStatus::default);
    let mut rows = use_signal(Vec::<ReleaseRow>::new);
    let mut loading = use_signal(|| true);
    let mut msg = use_signal(String::new);

    use_future(move || async move {
        loading.set(true);
        if let Some(s) = call::<UpdateStatus>("update_check", &NoArgs {}).await {
            status.set(s);
        }
        match call::<Vec<ReleaseRow>>("update_releases", &NoArgs {}).await {
            Some(v) => rows.set(v),
            None => msg.set("Could not reach GitHub. Check GH_PAT / network.".into()),
        }
        loading.set(false);
    });

    let install = move |tag: String| {
        msg.set(format!("Downloading {tag}… Prism will restart to finish installing."));
        spawn(async move {
            let ok: Option<()> = call("update_install", &TagArg { tag }).await;
            if ok.is_none() {
                msg.set("Install failed. See the release page for the installer.".into());
            }
        });
    };

    let onkey = move |ev: KeyboardEvent| {
        if ev.key() == Key::Escape {
            ev.prevent_default();
            on_close.call(());
        }
    };

    let st = status();
    let install_latest = {
        let tag = st.tag.clone();
        let mut install = install.clone();
        move |_| install(tag.clone())
    };

    rsx! {
        div {
            class: "panel",
            onclick: move |e| e.stop_propagation(),
            onkeydown: onkey,
            div { class: "backdrop" }
            div { class: "upd",
                div { class: "rem-header",
                    button {
                        class: "cb-back",
                        onclick: move |e| { e.stop_propagation(); on_close.call(()); },
                        svg {
                            class: "back-ic",
                            view_box: "0 0 24 24",
                            fill: "none",
                            stroke: "currentColor",
                            stroke_width: "2.2",
                            stroke_linecap: "round",
                            stroke_linejoin: "round",
                            polyline { points: "15 18 9 12 15 6" }
                        }
                    }
                    h2 { "Updates" }
                }
                div {
                    class: if st.has_update { "upd-banner has-update" } else { "upd-banner" },
                    div { class: "upd-cur",
                        div { class: "upd-cur-v", "Current · v{st.current}" }
                        div { class: "upd-cur-s",
                            if loading() {
                                "Checking for updates…"
                            } else if st.has_update {
                                "Update available: v{st.latest}"
                            } else {
                                "You're on the latest version."
                            }
                        }
                    }
                    if st.has_update {
                        button {
                            class: "rem-add",
                            onclick: install_latest,
                            "Update now"
                        }
                    }
                }
                if !msg().is_empty() {
                    div { class: "upd-msg", "{msg}" }
                }
                div { class: "rem-list",
                    if rows().is_empty() && !loading() {
                        div { class: "empty", "No releases found" }
                    }
                    for r in rows().iter() {
                        {
                            let tag = r.tag.clone();
                            let mut install = install.clone();
                            rsx! {
                                div {
                                    class: if r.current { "upd-rel current" } else { "upd-rel" },
                                    div { class: "upd-rel-head",
                                        div { class: "upd-rel-name",
                                            "{r.name}"
                                            if r.current {
                                                span { class: "upd-tag-cur", "INSTALLED" }
                                            }
                                        }
                                        div { class: "upd-rel-meta", "{r.tag} · {r.published}" }
                                    }
                                    if !r.notes.is_empty() {
                                        div { class: "upd-notes", "{r.notes}" }
                                    }
                                    if r.installable {
                                        button {
                                            class: "rem-del upd-install",
                                            onclick: move |e| {
                                                e.stop_propagation();
                                                install(tag.clone());
                                            },
                                            "Install this version"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

// ===== Theme Studio (separate `?view=themes` window) =====

fn theme_style(t: &Theme) -> String {
    let bg = if t.bg_kind == "gradient" {
        format!("linear-gradient(145deg, {} 0%, {} 100%)", t.bg_start, t.bg_end)
    } else {
        t.bg_start.clone()
    };
    format!(
        "--surface-base:{bg};--bg-color:{start};--text-primary:{text};\
         --text-strong:{text};--surface-active:{sel};--accent:{acc};\
         --accent-strong:{acc};",
        bg = bg,
        start = t.bg_start,
        text = t.text,
        sel = t.selection,
        acc = t.accent,
    )
}

async fn persist_themes(mut settings: Signal<Settings>, list: Vec<Theme>, active: Option<String>) {
    let mut st = settings();
    st.custom_themes = list;
    if let Some(id) = active {
        st.theme = id;
    }
    let _: Option<()> = call("save_settings", &SettingsArg { settings: st.clone() }).await;
    let _: Option<()> = call("theme_changed", &NoArgs {}).await;
    settings.set(st);
}

#[component]
fn ThemeStudio() -> Element {
    let mut settings = use_signal(Settings::default);
    let mut themes = use_signal(Vec::<Theme>::new);
    let mut draft = use_signal(Theme::default);
    // None = a built-in keyword theme is active (no inspector).
    let mut editing = use_signal(|| None::<String>);

    use_future(move || async move {
        if let Some(s) = call::<Settings>("load_settings", &NoArgs {}).await {
            themes.set(s.custom_themes.clone());
            if let Some(t) = s.custom_themes.iter().find(|t| t.id == s.theme) {
                draft.set(t.clone());
                editing.set(Some(t.id.clone()));
            }
            settings.set(s);
        }
    });

    let active = settings().theme;

    let select_custom = move |t: Theme| {
        let mut draft = draft;
        let mut editing = editing;
        draft.set(t.clone());
        editing.set(Some(t.id.clone()));
    };

    let new_theme = move |_| {
        let n = themes()
            .iter()
            .filter_map(|t| t.id.strip_prefix("theme-").and_then(|x| x.parse::<u32>().ok()))
            .max()
            .unwrap_or(0)
            + 1;
        let mut t = Theme::default();
        t.id = format!("theme-{n}");
        t.name = format!("New Theme {n}");
        let mut list = themes();
        list.push(t.clone());
        themes.set(list.clone());
        draft.set(t.clone());
        editing.set(Some(t.id.clone()));
        spawn(persist_themes(settings, list, None));
    };

    let save_apply = move |_| {
        let d = draft();
        let mut list = themes();
        if let Some(pos) = list.iter().position(|t| t.id == d.id) {
            list[pos] = d.clone();
        } else {
            list.push(d.clone());
        }
        themes.set(list.clone());
        spawn(persist_themes(settings, list, Some(d.id.clone())));
    };

    let delete_theme = move |_| {
        let mut themes = themes;
        let mut editing = editing;
        if let Some(id) = editing() {
            let list: Vec<Theme> = themes().into_iter().filter(|t| t.id != id).collect();
            themes.set(list.clone());
            editing.set(None);
            let reset = if settings().theme == id {
                Some("system".to_string())
            } else {
                None
            };
            spawn(persist_themes(settings, list, reset));
        }
    };

    let apply_builtin = move |kw: String| {
        let mut editing = editing;
        editing.set(None);
        spawn(persist_themes(settings, themes(), Some(kw)));
    };

    let set_field = move |f: &'static str, v: String| {
        let mut draft = draft;
        let mut d = draft();
        match f {
            "name" => d.name = v,
            "appearance" => d.appearance = v,
            "bg_kind" => d.bg_kind = v,
            "bg_start" => d.bg_start = v,
            "bg_end" => d.bg_end = v,
            "text" => d.text = v,
            "selection" => d.selection = v,
            "accent" => d.accent = v,
            _ => {}
        }
        draft.set(d);
    };

    let d = draft();
    let is_editing = editing().is_some();
    let prev_style = theme_style(&d);
    let count = themes().len() + 3;

    rsx! {
        link { rel: "stylesheet", href: CSS }
        div { class: "ts",
            aside { class: "ts-side",
                div { class: "ts-side-h", "Theme Studio" }
                div { class: "ts-side-sub",
                    span { "Installed Themes" }
                    span { class: "ts-count", "{count}" }
                }
                div { class: "ts-list",
                    button {
                        class: if active == "dark" { "ts-item active" } else { "ts-item" },
                        onclick: move |_| apply_builtin("dark".to_string()),
                        span { class: "ts-dot", style: "background:#0b0b10" }
                        span { class: "ts-item-n", "Prism Dark" }
                        span { class: "ts-lock", "🔒" }
                    }
                    button {
                        class: if active == "light" { "ts-item active" } else { "ts-item" },
                        onclick: move |_| apply_builtin("light".to_string()),
                        span { class: "ts-dot", style: "background:#ffffff" }
                        span { class: "ts-item-n", "Prism Light" }
                        span { class: "ts-lock", "🔒" }
                    }
                    button {
                        class: if active == "system" { "ts-item active" } else { "ts-item" },
                        onclick: move |_| apply_builtin("system".to_string()),
                        span { class: "ts-dot", style: "background:linear-gradient(90deg,#0b0b10 50%,#fff 50%)" }
                        span { class: "ts-item-n", "System" }
                        span { class: "ts-lock", "🔒" }
                    }
                    for t in themes() {
                        button {
                            key: "{t.id}",
                            class: if editing() == Some(t.id.clone()) { "ts-item active" } else { "ts-item" },
                            onclick: {
                                let t2 = t.clone();
                                move |_| select_custom(t2.clone())
                            },
                            span { class: "ts-dot", style: "background:{t.accent}" }
                            span { class: "ts-item-n", "{t.name}" }
                            if active == t.id {
                                span { class: "ts-check", "✓" }
                            }
                        }
                    }
                }
                button { class: "ts-new", onclick: new_theme, "+ New Theme" }
            }

            main { class: "ts-stage",
                div { class: "ts-beta", "Live Preview" }
                div { class: "ts-preview", style: "{prev_style}",
                    div { class: "panel",
                        div { class: "backdrop" }
                        div { class: "searchbar",
                            img { class: "search-ic", src: PRISM_ICON, alt: "Prism" }
                            div { class: "ts-fake-input", "Search for apps and commands..." }
                        }
                        div { class: "divider" }
                        div { class: "ts-prev-list",
                            div { class: "section-label", "SUGGESTIONS" }
                            div { class: "row selected",
                                div { class: "icon", Ic { name: "app".to_string() } }
                                div { class: "title", "Primary Text" }
                                div { class: "subtitle", "Selected row" }
                                div { class: "spacer" }
                                div { class: "chips", div { class: "chip", "↵" } }
                            }
                            div { class: "row",
                                div { class: "icon", Ic { name: "clipboard".to_string() } }
                                div { class: "title", "Clipboard History" }
                                div { class: "subtitle", "Command" }
                            }
                            div { class: "row",
                                div { class: "icon", Ic { name: "bell".to_string() } }
                                div { class: "title", "Prism Reminders" }
                                div { class: "subtitle", "Command" }
                            }
                            div { class: "section-label", "ACCENT" }
                            div { class: "row",
                                div { class: "icon", Ic { name: "gem".to_string() } }
                                div { class: "title", "Accent Button" }
                                div { class: "spacer" }
                                button { class: "rem-add", "Action" }
                            }
                        }
                        div { class: "footer",
                            div { class: "brand",
                                img { class: "search-ic", src: PRISM_ICON, alt: "" }
                                span { "Theme Studio Preview" }
                            }
                            div { class: "acts",
                                span { class: "strong", "Open Command" }
                                span { class: "chip", "↵" }
                            }
                        }
                    }
                }
            }

            aside { class: "ts-insp",
                if is_editing {
                    div { class: "ts-field",
                        label { "Theme Name" }
                        input {
                            class: "rem-in",
                            value: "{d.name}",
                            oninput: move |e| set_field("name", e.value()),
                        }
                    }
                    div { class: "ts-field",
                        label { "Theme Appearance" }
                        div { class: "ts-seg",
                            button {
                                class: if d.appearance == "light" { "ts-seg-b on" } else { "ts-seg-b" },
                                onclick: move |_| set_field("appearance", "light".to_string()),
                                "☀ Light"
                            }
                            button {
                                class: if d.appearance == "dark" { "ts-seg-b on" } else { "ts-seg-b" },
                                onclick: move |_| set_field("appearance", "dark".to_string()),
                                "☾ Dark"
                            }
                        }
                    }
                    div { class: "ts-group", "Background" }
                    div { class: "ts-field",
                        div { class: "ts-seg",
                            button {
                                class: if d.bg_kind == "solid" { "ts-seg-b on" } else { "ts-seg-b" },
                                onclick: move |_| set_field("bg_kind", "solid".to_string()),
                                "Solid"
                            }
                            button {
                                class: if d.bg_kind == "gradient" { "ts-seg-b on" } else { "ts-seg-b" },
                                onclick: move |_| set_field("bg_kind", "gradient".to_string()),
                                "Gradient"
                            }
                        }
                    }
                    ColorRow { label: "Start Color".to_string(), value: d.bg_start.clone(), on: move |v| set_field("bg_start", v) }
                    if d.bg_kind == "gradient" {
                        ColorRow { label: "End Color".to_string(), value: d.bg_end.clone(), on: move |v| set_field("bg_end", v) }
                    }
                    div { class: "ts-group", "Primary Colors" }
                    ColorRow { label: "Text".to_string(), value: d.text.clone(), on: move |v| set_field("text", v) }
                    ColorRow { label: "Selection".to_string(), value: d.selection.clone(), on: move |v| set_field("selection", v) }
                    ColorRow { label: "Accent".to_string(), value: d.accent.clone(), on: move |v| set_field("accent", v) }
                    div { class: "ts-actions",
                        button { class: "rem-add", onclick: save_apply, "Save & Apply" }
                        button { class: "rem-del", onclick: delete_theme, "Delete" }
                    }
                } else {
                    div { class: "ts-empty",
                        "Select a custom theme to edit it, or create a new one."
                    }
                }
            }
        }
    }
}

#[component]
fn ColorRow(label: String, value: String, on: EventHandler<String>) -> Element {
    rsx! {
        div { class: "ts-color-row",
            span { "{label}" }
            div { class: "ts-color-wrap",
                span { class: "ts-color-hex", "{value}" }
                input {
                    r#type: "color",
                    class: "ts-color",
                    value: "{value}",
                    oninput: move |e| on.call(e.value()),
                }
            }
        }
    }
}
