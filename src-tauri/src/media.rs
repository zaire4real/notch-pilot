//! Now Playing — macOS implementation via AppleScript (`osascript`).
//!
//! Honest limitation: this MVP does NOT use the private MediaRemote
//! framework. It asks Music.app first, then Spotify, and reports
//! "nothing playing" for every other player (YouTube Music, web players,
//! etc.). The user must grant Automation permission the first time
//! (System Settings → Privacy & Security → Automation).

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct NowPlaying {
    pub title: String,
    pub artist: String,
    pub album: String,
    pub playing: bool,
    pub app: String,
}

impl NowPlaying {
    fn nothing() -> Self {
        Self {
            title: String::new(),
            artist: String::new(),
            album: String::new(),
            playing: false,
            app: String::new(),
        }
    }
}

#[tauri::command]
pub fn get_now_playing() -> Result<NowPlaying, String> {
    #[cfg(target_os = "macos")]
    {
        // Try Music.app, then Spotify. Each script returns
        // "title|artist|album|playing" or an empty string.
        for app in ["Music", "Spotify"] {
            if let Some(track) = query_app(app) {
                return Ok(track);
            }
        }
        Ok(NowPlaying::nothing())
    }
    #[cfg(not(target_os = "macos"))]
    {
        Err("Now Playing is only supported on macOS".to_string())
    }
}

#[tauri::command]
pub fn media_play_pause() -> Result<(), String> {
    control("playpause")
}

#[tauri::command]
pub fn media_next() -> Result<(), String> {
    control("next track")
}

#[tauri::command]
pub fn media_previous() -> Result<(), String> {
    control("previous track")
}

#[cfg(target_os = "macos")]
fn query_app(app: &str) -> Option<NowPlaying> {
    // `player state` is "playing" / "paused" / "stopped".
    // Delimit with "|" and trim the trailing newline from osascript.
    let script = format!(
        "tell application \"{app}\" to if it is running then \
         if player state is playing or player state is paused then \
         return (name of current track & \"|\" & artist of current track \
         & \"|\" & album of current track & \"|\" \
         & (player state as text)) end if end if"
    );
    let output = run_osascript(&script)?;
    let output = output.trim();
    if output.is_empty() {
        return None;
    }
    let mut parts = output.splitn(4, '|');
    let title = parts.next().unwrap_or("").to_string();
    let artist = parts.next().unwrap_or("").to_string();
    let album = parts.next().unwrap_or("").to_string();
    let state = parts.next().unwrap_or("").to_string();
    if title.is_empty() {
        return None;
    }
    Some(NowPlaying {
        title,
        artist,
        album,
        playing: state == "playing",
        app: app.to_string(),
    })
}

#[cfg(target_os = "macos")]
fn control(action: &str) -> Result<(), String> {
    // Send the same command to whichever app is currently playing;
    // if neither is, try Music.app so the button still does something
    // predictable instead of silently doing nothing.
    let target = if query_app("Music").is_some() {
        "Music"
    } else if query_app("Spotify").is_some() {
        "Spotify"
    } else {
        "Music"
    };
    let script = format!("tell application \"{target}\" to {action}");
    run_osascript(&script)
        .map(|_| ())
        .ok_or_else(|| format!("failed to send {action} to {target}"))
}

#[cfg(target_os = "macos")]
fn run_osascript(script: &str) -> Option<String> {
    let output = std::process::Command::new("osascript")
        .arg("-e")
        .arg(script)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&output.stdout).to_string())
}

#[cfg(not(target_os = "macos"))]
fn control(_action: &str) -> Result<(), String> {
    Err("Media control is only supported on macOS".to_string())
}
