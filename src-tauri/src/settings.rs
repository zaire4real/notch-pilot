//! Settings persistence and the Settings window (v0.2 Phase 0).
//!
//! One JSON file, `settings.json`, in the app data dir
//! (`~/Library/Application Support/com.yichia.notch-pilot/` on macOS),
//! read/written by `get_settings` / `set_settings`. The language lives
//! here as the source of truth; the frontend keeps `localStorage` only
//! as a cache (see `src/i18n.js`).
//!
//! First-run note: when no settings file exists yet, `load_settings`
//! returns the defaults with an **empty** `language` string. The
//! frontend treats empty as "not chosen yet" and runs its system
//! language detection instead of forcing zh-TW on every user.
//! `set_settings` normalises an empty language to "zh-TW" before
//! saving, so a saved file always has a real language.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::{AppHandle, Emitter, Manager, WebviewUrl};

// ---------------------------------------------------------------------------
// Settings model — every field has a serde default so an older
// settings.json (missing newer keys) still loads.
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub language: String,
    pub notch: NotchSettings,
    pub appearance: AppearanceSettings,
    pub media: MediaSettings,
    pub calendar: CalendarSettings,
    pub shelf: ShelfSettings,
    pub mirror: MirrorSettings,
    pub battery: BatterySettings,
    pub osd: OsdSettings,
    pub shortcuts: ShortcutsSettings,
    pub weather: WeatherSettings,
    /// Pomodoro timer state (v0.2 Phase 7, F20). The panel owns the
    /// live countdown; this struct is only the persisted snapshot
    /// (mode + remaining seconds + whether it was running), restored
    /// on start. `running` restores as-is and the countdown resumes
    /// from the saved remaining time — the wall-clock time while the
    /// app was closed is not subtracted, so it can run a little long
    /// after a restart; that is documented, not hidden.
    pub pomodoro: PomodoroSettings,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            language: "zh-TW".to_string(),
            notch: NotchSettings::default(),
            appearance: AppearanceSettings::default(),
            media: MediaSettings::default(),
            calendar: CalendarSettings::default(),
            shelf: ShelfSettings::default(),
            mirror: MirrorSettings::default(),
            battery: BatterySettings::default(),
            osd: OsdSettings::default(),
            shortcuts: ShortcutsSettings::default(),
            weather: WeatherSettings::default(),
            pomodoro: PomodoroSettings::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct NotchSettings {
    /// 0 = auto (use the real notch width).
    pub idle_width: f64,
    pub expanded_width: f64,
    pub expanded_height: f64,
    pub non_notch_height: f64,
    pub hover_open_ms: u64,
    pub hover_close_ms: u64,
    /// "slow" | "normal" | "fast".
    pub animation_speed: String,
    pub show_on_all_displays: bool,
    pub follow_active_display: bool,
    /// Trackpad gestures (F6, Phase 6): swipe down on the pill =
    /// open, swipe up = close. Wired in main.js.
    pub gestures: bool,
    /// Sneak-peek HUD height in logical points (Phase 6; was a
    /// hard-coded 70 in notch.rs). 0 or out of range falls back
    /// to 70.
    pub sneak_peek_height: f64,
    /// Sneak-peek auto-collapse delay in milliseconds (Phase 6;
    /// was a hard-coded 2000 in main.js). 0 falls back to 2000.
    pub sneak_peek_ms: u64,
}

impl Default for NotchSettings {
    fn default() -> Self {
        Self {
            idle_width: 0.0,
            expanded_width: 680.0,
            expanded_height: 360.0,
            non_notch_height: 36.0,
            hover_open_ms: 120,
            hover_close_ms: 250,
            animation_speed: "normal".to_string(),
            show_on_all_displays: false,
            follow_active_display: false,
            gestures: true,
            sneak_peek_height: 70.0,
            sneak_peek_ms: 2000,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AppearanceSettings {
    pub accent: String,
    pub show_eyes: bool,
    /// "system" | "dark" | "light".
    pub theme: String,
}

impl Default for AppearanceSettings {
    fn default() -> Self {
        Self {
            accent: "#ffffff".to_string(),
            show_eyes: false,
            theme: "system".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct MediaSettings {
    pub visualizer: bool,
    /// "bars" | "circular" | "wave".
    pub visualizer_style: String,
    pub lyrics: bool,
    /// "system" = the current macOS output device.
    pub audio_output: String,
    pub media_gestures: bool,
}

impl Default for MediaSettings {
    fn default() -> Self {
        Self {
            visualizer: true,
            visualizer_style: "bars".to_string(),
            lyrics: false,
            audio_output: "system".to_string(),
            media_gestures: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct CalendarSettings {
    /// Calendar names to show; empty = all calendars.
    pub calendars: Vec<String>,
    pub reminders: bool,
}

impl Default for CalendarSettings {
    fn default() -> Self {
        Self {
            calendars: Vec::new(),
            reminders: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ShelfSettings {
    pub airdrop: bool,
    pub open_on_drop: bool,
    pub keep_files: bool,
    /// Persisted shelf file paths (v0.2 Phase 4a). The list lives in
    /// settings.json next to the other shelf toggles, so it survives
    /// an app restart. Entries whose files no longer exist are
    /// silently pruned on load (see `load_shelf_paths`).
    pub files: Vec<String>,
}

impl Default for ShelfSettings {
    fn default() -> Self {
        Self {
            airdrop: true,
            open_on_drop: true,
            keep_files: true,
            files: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct MirrorSettings {
    pub enabled: bool,
    /// "circle" | "rounded".
    pub shape: String,
    /// Empty = the default camera.
    pub camera: String,
}

impl Default for MirrorSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            shape: "rounded".to_string(),
            camera: String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct BatterySettings {
    pub show_percent: bool,
    /// "percent" | "time" | "both".
    pub charging_indicator: String,
}

impl Default for BatterySettings {
    fn default() -> Self {
        Self {
            show_percent: true,
            charging_indicator: "percent".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct OsdSettings {
    pub volume: bool,
    pub brightness: bool,
    pub backlight: bool,
}

impl Default for OsdSettings {
    fn default() -> Self {
        Self {
            volume: true,
            brightness: true,
            backlight: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ShortcutsSettings {
    pub open_settings: String,
    pub toggle_panel: String,
    pub play_pause: String,
    pub next: String,
    pub previous: String,
}

impl Default for ShortcutsSettings {
    fn default() -> Self {
        Self {
            // Cmd+, on macOS (CommandOrControl maps to Cmd on macOS).
            open_settings: "CommandOrControl+,".to_string(),
            toggle_panel: String::new(),
            play_pause: String::new(),
            next: String::new(),
            previous: String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct WeatherSettings {
    pub enabled: bool,
    pub city: String,
}

impl Default for WeatherSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            city: "Taipei".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct PomodoroSettings {
    /// "focus" | "break".
    pub mode: String,
    /// Seconds left in the current period.
    pub remaining_secs: u64,
    pub running: bool,
}

impl Default for PomodoroSettings {
    fn default() -> Self {
        Self {
            mode: "focus".to_string(),
            remaining_secs: 25 * 60,
            running: false,
        }
    }
}

// ---------------------------------------------------------------------------
// File persistence
// ---------------------------------------------------------------------------

pub fn settings_path(app: &AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_data_dir()
        .map(|dir| dir.join("settings.json"))
        .map_err(|e| e.to_string())
}

pub fn load_settings(app: &AppHandle) -> Settings {
    let Ok(path) = settings_path(app) else {
        return Settings::default();
    };
    let Ok(text) = std::fs::read_to_string(&path) else {
        // No file yet: defaults, but with an empty language so the
        // frontend runs its first-run system-language detection.
        let mut settings = Settings::default();
        settings.language = String::new();
        return settings;
    };
    match serde_json::from_str::<Settings>(&text) {
        Ok(settings) => settings,
        Err(_) => Settings::default(),
    }
}

pub fn save_settings(app: &AppHandle, settings: &Settings) -> Result<(), String> {
    let path = settings_path(app)?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let text = serde_json::to_string_pretty(settings).map_err(|e| e.to_string())?;
    std::fs::write(&path, text).map_err(|e| e.to_string())
}

// ---------------------------------------------------------------------------
// Tauri commands
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn get_settings(app: AppHandle) -> Settings {
    load_settings(&app)
}

#[tauri::command]
pub fn set_settings(app: AppHandle, settings: Settings) -> Result<Settings, String> {
    let mut settings = settings;
    if settings.language.trim().is_empty() {
        settings.language = "zh-TW".to_string();
    }
    save_settings(&app, &settings)?;
    // Tell every window (main panel + settings window) to re-apply.
    // The frontend listens for "settings-changed" (see src/main.js
    // and src/settings.js). A failed emit must not fail the save.
    let _ = app.emit("settings-changed", &settings);
    Ok(settings)
}

// ---------------------------------------------------------------------------
// Settings window — created at runtime so it only exists while open.
// ---------------------------------------------------------------------------

pub fn open_settings_window(app: &AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("settings") {
        let _ = window.show();
        let _ = window.set_focus();
        return Ok(());
    }
    tauri::WebviewWindowBuilder::new(app, "settings", WebviewUrl::App("settings.html".into()))
        .title("NotchPilot 設定")
        .inner_size(560.0, 640.0)
        .decorations(true)
        .resizable(true)
        .center()
        .build()
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn open_settings(app: AppHandle) -> Result<(), String> {
    open_settings_window(&app)
}

// ---------------------------------------------------------------------------
// Shelf persistence (v0.2 Phase 4a, F7 — shelf half only; AirDrop is
// Phase 4b and lives elsewhere).
//
// The shelf file list is `settings.shelf.files` in settings.json, so
// it survives an app restart with no extra file to keep in sync.
// The Settings window round-trips the whole Settings object
// (settings.js clones currentSettings), so it never wipes the list.
// Pruning rule: entries whose files no longer exist are silently
// dropped on load and the pruned list is saved back — the shelf
// never shows a dead entry, and there is no greyed-out state to
// explain. Opening a file uses the macOS `open` command (no new
// plugin dependency); file icons are extension → emoji in the
// frontend, no Finder icon extraction in v0.2.
// ---------------------------------------------------------------------------

/// One shelf entry as the frontend renders it: absolute path, file
/// name, and size in bytes (None when the size cannot be read — the
/// frontend then omits the size).
#[derive(Debug, Clone, Serialize)]
pub struct ShelfItem {
    pub path: String,
    pub name: String,
    pub size: Option<u64>,
}

fn shelf_item(path: &str) -> ShelfItem {
    let p = std::path::Path::new(path);
    ShelfItem {
        path: path.to_string(),
        name: p
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| path.to_string()),
        size: std::fs::metadata(p).ok().map(|m| m.len()),
    }
}

/// Load the persisted shelf paths, silently dropping entries whose
/// files no longer exist and saving the pruned list back when
/// anything was dropped.
fn load_shelf_paths(app: &AppHandle) -> Vec<String> {
    let mut settings = load_settings(app);
    let before = settings.shelf.files.len();
    settings
        .shelf
        .files
        .retain(|p| std::path::Path::new(p).exists());
    if settings.shelf.files.len() != before {
        if settings.language.trim().is_empty() {
            settings.language = "zh-TW".to_string();
        }
        let _ = save_settings(app, &settings);
    }
    settings.shelf.files
}

fn persist_shelf_paths(app: &AppHandle, files: Vec<String>) -> Vec<ShelfItem> {
    let mut settings = load_settings(app);
    if settings.language.trim().is_empty() {
        settings.language = "zh-TW".to_string();
    }
    settings.shelf.files = files;
    let _ = save_settings(app, &settings);
    settings
        .shelf
        .files
        .iter()
        .map(|p| shelf_item(p))
        .collect()
}

#[tauri::command]
pub fn get_shelf(app: AppHandle) -> Vec<ShelfItem> {
    load_shelf_paths(&app)
        .iter()
        .map(|p| shelf_item(p))
        .collect()
}

#[tauri::command]
pub fn add_shelf_files(app: AppHandle, paths: Vec<String>) -> Vec<ShelfItem> {
    let mut files = load_shelf_paths(&app);
    for path in paths {
        let path = path.trim().to_string();
        if path.is_empty() {
            continue;
        }
        // Skip duplicates and paths that do not exist (a drag-drop
        // of a deleted file must not leave a dead entry).
        if files.iter().any(|f| f == &path) {
            continue;
        }
        if !std::path::Path::new(&path).exists() {
            continue;
        }
        files.push(path);
    }
    persist_shelf_paths(&app, files)
}

#[tauri::command]
pub fn remove_shelf_file(app: AppHandle, path: String) -> Vec<ShelfItem> {
    let mut files = load_shelf_paths(&app);
    files.retain(|f| f != &path);
    persist_shelf_paths(&app, files)
}

/// Open a shelf file with its default app. The file must still
/// exist; a missing file is reported so the frontend can drop it
/// from the list (load_shelf_paths prunes it on the next read).
#[tauri::command]
pub fn open_shelf_file(path: String) -> Result<(), String> {
    if !std::path::Path::new(&path).exists() {
        return Err(format!("file no longer exists: {path}"));
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&path)
            .spawn()
            .map_err(|e| e.to_string())?;
        Ok(())
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = path;
        Err("opening files is only supported on macOS".to_string())
    }
}
