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
}

impl Default for NotchSettings {
    fn default() -> Self {
        Self {
            idle_width: 0.0,
            expanded_width: 680.0,
            expanded_height: 200.0,
            non_notch_height: 36.0,
            hover_open_ms: 120,
            hover_close_ms: 250,
            animation_speed: "normal".to_string(),
            show_on_all_displays: false,
            follow_active_display: false,
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
}

impl Default for ShelfSettings {
    fn default() -> Self {
        Self {
            airdrop: true,
            open_on_drop: true,
            keep_files: true,
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
