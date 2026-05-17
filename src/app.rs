#![allow(non_snake_case)]

use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

static CSS: Asset = asset!("/assets/styles.css");

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
struct Settings {
    pinned: Vec<String>,
    recent: Vec<String>,
    bg_color: String,
    collapsed: bool,
    act_pin: String,
    act_open: String,
    theme: String,
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
        }
    }
}

fn icon_name(e: &Entry) -> &'static str {
    match e.kind.as_str() {
        "settings" => "gear",
        "system" => "monitor",
        "clipboard" => "clipboard",
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

fn apply_css(s: &Settings) {
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
    flat: Vec<Entry>,
    idx: usize,
) {
    let Some(e) = flat.get(idx).cloned() else {
        return;
    };
    match e.kind.as_str() {
        "settings" => show_settings.set(true),
        "clipboard" => show_clipboard.set(true),
        "reminders" => show_reminders.set(true),
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

    let mut entries = use_signal(Vec::<Entry>::new);
    let mut settings = use_signal(Settings::default);
    let mut query = use_signal(String::new);
    let mut sel = use_signal(|| 0usize);
    let mut show_settings = use_signal(|| false);
    let mut show_clipboard = use_signal(|| false);
    let mut show_reminders = use_signal(|| false);
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
            show_actions.set(false);
            let _ = document::eval(
                "setTimeout(function(){var x=document.getElementById('search');if(x)x.focus();},20);",
            );
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
        let ql = q.to_lowercase();
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
                    spawn(do_activate(settings, show_settings, show_clipboard, show_reminders, pv, i));
                } else {
                    let flat = flat_key.clone();
                    spawn(do_activate(settings, show_settings, show_clipboard, show_reminders, flat, cur));
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
                        spawn(do_activate(settings, show_settings, show_clipboard, show_reminders, flat, cur));
                        return;
                    }
                }
                if m.ctrl() {
                    if c == "k" {
                        ev.prevent_default();
                        show_actions.toggle();
                    } else if let Ok(n) = c.parse::<usize>() {
                        if (1..=9).contains(&n) && n <= total {
                            ev.prevent_default();
                            let flat = flat_key.clone();
                            spawn(do_activate(
                                settings,
                                show_settings,
                                show_clipboard,
                                show_reminders,
                                flat,
                                n - 1,
                            ));
                        }
                    }
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
            } else if show_settings() {
                SettingsView { settings, on_close: move |_| show_settings.set(false) }
            } else {
                div { class: if open { "dock-row" } else { "dock-row pill" },
                div {
                    class: if open { "panel" } else { "panel collapsed" },
                    onclick: move |e| e.stop_propagation(),
                    div { class: "backdrop" }
                    div { class: "searchbar",
                        svg {
                            class: "search-ic",
                            view_box: "0 0 24 24",
                            fill: "none",
                            stroke: "currentColor",
                            stroke_width: "2",
                            stroke_linecap: "round",
                            circle { cx: "11", cy: "11", r: "7" }
                            line { x1: "16.5", y1: "16.5", x2: "21", y2: "21" }
                        }
                        input {
                            id: "search",
                            autofocus: true,
                            autocomplete: "off",
                            autocorrect: "off",
                            autocapitalize: "off",
                            spellcheck: false,
                            placeholder: "Spotlight Search",
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
                                    let qkey = if idx < 9 { Some(idx + 1) } else { None };
                                    let flat_row = flat.clone();
                                    let icon = icons.read().get(&e.id).cloned();
                                    rsx! {
                                        div {
                                            id: "row-{idx}",
                                            class: if idx == cur { "row selected" } else { "row" },
                                            onclick: move |ev| {
                                                ev.stop_propagation();
                                                sel.set(idx);
                                                spawn(do_activate(settings, show_settings, show_clipboard, show_reminders, flat_row.clone(), idx));
                                            },
                                            if let Some(u) = icon {
                                                img { class: "icon-img", src: "{u}" }
                                            } else {
                                                div { class: "icon", Ic { name: icon_name(e).to_string() } }
                                            }
                                            div { class: "title", "{e.title}" }
                                            div { class: "subtitle", "{e.subtitle}" }
                                            div { class: "spacer" }
                                            div { class: "chips",
                                                if pinned {
                                                    div { class: "chip pin-chip", Ic { name: "pin".to_string() } }
                                                }
                                                if let Some(k) = qkey {
                                                    div { class: "chip", "Ctrl" }
                                                    div { class: "chip", "{k}" }
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
                                            spawn(do_activate(settings, show_settings, show_clipboard, show_reminders, flat_b.clone(), cur));
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
                                            spawn(do_activate(settings, show_settings, show_clipboard, show_reminders, pv.clone(), i));
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
