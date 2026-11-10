//! Now Playing — macOS implementation via AppleScript (`osascript`).
//!
//! Honest limitation: this does NOT use the private MediaRemote
//! framework. It asks Music.app first, then Spotify, and reports
//! "nothing playing" for every other player (YouTube Music, web players,
//! etc.). The user must grant Automation permission the first time
//! (System Settings → Privacy & Security → Automation).
//!
//! v0.2 Phase 1 adds, on top of title/artist/album/playing:
//! - `position` / `duration` (seconds) via `player position` and
//!   `duration of current track`, plus a `media_seek` command.
//! - Album art: Music.app exposes `data of artwork 1 of current
//!   track`; the script writes those bytes to a temp file, Rust reads
//!   them back and returns a `data:` URL (base64). The result is
//!   cached per track so the 5 s poll does not re-fetch the art.
//!   Spotify exposes `artwork url of current track` instead, so its
//!   art is returned as a plain URL (`artwork_url`) and the frontend
//!   loads it with a normal `<img src>` (no CORS issue for images).

use serde::Serialize;
use std::sync::Mutex;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct NowPlaying {
    pub title: String,
    pub artist: String,
    pub album: String,
    pub playing: bool,
    pub app: String,
    /// Seconds into the track; 0 when unknown.
    pub position: f64,
    /// Track length in seconds; 0 when unknown.
    pub duration: f64,
    /// Album art for Music.app as a `data:` URL (base64); empty when
    /// the track has no artwork or the player is not Music.
    pub artwork: String,
    /// Artwork URL for Spotify; empty for Music.
    pub artwork_url: String,
}

impl NowPlaying {
    fn nothing() -> Self {
        Self {
            title: String::new(),
            artist: String::new(),
            album: String::new(),
            playing: false,
            app: String::new(),
            position: 0.0,
            duration: 0.0,
            artwork: String::new(),
            artwork_url: String::new(),
        }
    }
}

/// Cache of the last fetched artwork: (track key, `data:` URL).
/// Keyed on app + title + artist + album so a track change is the
/// only thing that triggers a fresh AppleScript artwork fetch.
#[cfg(target_os = "macos")]
static ARTWORK_CACHE: Mutex<Option<(String, String)>> = Mutex::new(None);

#[tauri::command]
pub fn get_now_playing() -> Result<NowPlaying, String> {
    #[cfg(target_os = "macos")]
    {
        // Try Music.app, then Spotify.
        for app in ["Music", "Spotify"] {
            if let Some(mut track) = query_app(app) {
                if app == "Music" {
                    track.artwork = artwork_for_music(&track);
                }
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

#[tauri::command]
pub fn media_seek(position: f64) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        let position = if position.is_finite() {
            position.max(0.0)
        } else {
            0.0
        };
        let target = active_app();
        let script = format!("tell application \"{target}\" to set player position to {position}");
        run_osascript(&script)
            .map(|_| ())
            .ok_or_else(|| format!("failed to seek in {target}"))
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = position;
        Err("Media seek is only supported on macOS".to_string())
    }
}

#[cfg(target_os = "macos")]
fn query_app(app: &str) -> Option<NowPlaying> {
    // `player state` is "playing" / "paused" / "stopped".
    // Delimit with "|" and trim the trailing newline from osascript.
    // Spotify's `artwork url` is wrapped in try/on error: a track
    // without artwork must not fail the whole query.
    let script = if app == "Spotify" {
        "tell application \"Spotify\"\n\
         \tif it is running then\n\
         \t\tif player state is playing or player state is paused then\n\
         \t\t\ttry\n\
         \t\t\t\tset artUrl to artwork url of current track\n\
         \t\t\ton error\n\
         \t\t\t\tset artUrl to \"\"\n\
         \t\t\tend try\n\
         \t\t\treturn (name of current track & \"|\" & artist of current track \
         & \"|\" & album of current track & \"|\" \
         & (player state as text) & \"|\" & (player position as text) \
         & \"|\" & (duration of current track as text) & \"|\" & artUrl)\n\
         \t\tend if\n\
         \tend if\n\
         end tell\n\
         return \"\""
            .to_string()
    } else {
        "tell application \"Music\"\n\
         \tif it is running then\n\
         \t\tif player state is playing or player state is paused then\n\
         \t\t\treturn (name of current track & \"|\" & artist of current track \
         & \"|\" & album of current track & \"|\" \
         & (player state as text) & \"|\" & (player position as text) \
         & \"|\" & (duration of current track as text))\n\
         \t\tend if\n\
         \tend if\n\
         end tell\n\
         return \"\""
            .to_string()
    };
    let output = run_osascript(&script)?;
    let output = output.trim();
    if output.is_empty() {
        return None;
    }
    let mut parts = output.splitn(7, '|');
    let title = parts.next().unwrap_or("").to_string();
    let artist = parts.next().unwrap_or("").to_string();
    let album = parts.next().unwrap_or("").to_string();
    let state = parts.next().unwrap_or("").to_string();
    let position = parts
        .next()
        .unwrap_or("")
        .parse::<f64>()
        .unwrap_or(0.0)
        .max(0.0);
    let duration = parts
        .next()
        .unwrap_or("")
        .parse::<f64>()
        .unwrap_or(0.0)
        .max(0.0);
    let artwork_url = parts.next().unwrap_or("").to_string();
    if title.is_empty() {
        return None;
    }
    Some(NowPlaying {
        title,
        artist,
        album,
        playing: state == "playing",
        app: app.to_string(),
        position,
        duration,
        artwork: String::new(),
        artwork_url,
    })
}

/// Which app is currently playing (or Music as the predictable
/// fallback when neither is). Used by control() and media_seek().
#[cfg(target_os = "macos")]
fn active_app() -> &'static str {
    if query_app("Music").is_some() {
        "Music"
    } else if query_app("Spotify").is_some() {
        "Spotify"
    } else {
        "Music"
    }
}

#[cfg(target_os = "macos")]
fn control(action: &str) -> Result<(), String> {
    // Send the command to whichever app is currently playing;
    // if neither is, try Music.app so the button still does something
    // predictable instead of silently doing nothing.
    let target = active_app();
    let script = format!("tell application \"{target}\" to {action}");
    run_osascript(&script)
        .map(|_| ())
        .ok_or_else(|| format!("failed to send {action} to {target}"))
}

/// Album art for the current Music.app track, as a `data:` URL.
/// Cached per track; returns an empty string when the track has no
/// artwork or the artwork fetch fails.
#[cfg(target_os = "macos")]
fn artwork_for_music(track: &NowPlaying) -> String {
    let key = format!(
        "Music|{}|{}|{}",
        track.title, track.artist, track.album
    );
    if let Ok(cache) = ARTWORK_CACHE.lock() {
        if let Some((cached_key, cached_url)) = cache.as_ref() {
            if *cached_key == key {
                return cached_url.clone();
            }
        }
    }
    let url = fetch_music_artwork().unwrap_or_default();
    if let Ok(mut cache) = ARTWORK_CACHE.lock() {
        *cache = Some((key, url.clone()));
    }
    url
}

/// Ask Music.app for `data of artwork 1 of current track` and write
/// the bytes to a temp file (osascript cannot return raw bytes on
/// stdout). Rust reads the file back and base64-encodes it.
#[cfg(target_os = "macos")]
fn fetch_music_artwork() -> Option<String> {
    let path = std::env::temp_dir().join("notch-pilot-artwork.bin");
    let path_str = path
        .to_string_lossy()
        .replace('\\', "\\\\")
        .replace('"', "\\\"");
    let script = format!(
        "tell application \"Music\"\n\
         \tif it is running then\n\
         \t\tif player state is playing or player state is paused then\n\
         \t\t\ttry\n\
         \t\t\t\tset artData to data of artwork 1 of current track\n\
         \t\t\t\tset f to open for access (POSIX file \"{path_str}\") with write permission\n\
         \t\t\t\ttry\n\
         \t\t\t\t\tset eof of f to 0\n\
         \t\t\t\t\twrite artData to f\n\
         \t\t\t\t\tclose access f\n\
         \t\t\t\ton error\n\
         \t\t\t\t\ttry\n\
         \t\t\t\t\t\tclose access f\n\
         \t\t\t\t\tend try\n\
         \t\t\t\tend try\n\
         \t\t\t\treturn \"ok\"\n\
         \t\t\ton error\n\
         \t\t\t\treturn \"\"\n\
         \t\t\tend try\n\
         \t\tend if\n\
         \tend if\n\
         end tell\n\
         return \"\""
    );
    let output = run_osascript(&script)?;
    if output.trim() != "ok" {
        return None;
    }
    let bytes = std::fs::read(&path).ok()?;
    if bytes.is_empty() {
        return None;
    }
    let mime = sniff_artwork_mime(&bytes);
    Some(format!("data:{mime};base64,{}", base64_encode(&bytes)))
}

#[cfg(target_os = "macos")]
fn sniff_artwork_mime(data: &[u8]) -> &'static str {
    if data.starts_with(&[0xFF, 0xD8, 0xFF]) {
        "image/jpeg"
    } else if data.starts_with(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]) {
        "image/png"
    } else {
        // Music.app artwork is JPEG in practice; fall back to JPEG
        // rather than failing the whole Now Playing response.
        "image/jpeg"
    }
}

/// Minimal base64 (standard alphabet) so no new crate is needed.
#[cfg(target_os = "macos")]
fn base64_encode(data: &[u8]) -> String {
    const ALPHABET: &[u8; 64] =
        b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity((data.len() + 2) / 3 * 4);
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = *chunk.get(1).unwrap_or(&0) as u32;
        let b2 = *chunk.get(2).unwrap_or(&0) as u32;
        let n = (b0 << 16) | (b1 << 8) | b2;
        out.push(ALPHABET[((n >> 18) & 63) as usize] as char);
        out.push(ALPHABET[((n >> 12) & 63) as usize] as char);
        out.push(if chunk.len() > 1 {
            ALPHABET[((n >> 6) & 63) as usize] as char
        } else {
            '='
        });
        out.push(if chunk.len() > 2 {
            ALPHABET[(n & 63) as usize] as char
        } else {
            '='
        });
    }
    out
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
