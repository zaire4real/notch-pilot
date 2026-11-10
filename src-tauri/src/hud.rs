//! HUD detection — volume + display brightness (v0.2 Phase 6, F9).
//!
//! Detection + notch display ONLY (risk R4): there is no supported
//! API to suppress Apple's own OSD, so this module never tries to
//! hide it — the frontend just shows a sneak-peek in the notch when
//! a value changes, alongside whatever macOS shows. The Settings →
//! OSD section says so in plain language, and no UI text promises
//! a full HUD replacement.
//!
//! Volume: one `osascript` call — `output volume of (get volume
//! settings)` plus the muted flag, joined with `|`.
//!
//! Brightness: on Apple Silicon there is no public
//! display-brightness getter — `IODisplayConnect` is gone since Big
//! Sur, and the DisplayServices framework that replaced it is
//! private, so it is not used. Instead the read-only system
//! diagnostic `/usr/libexec/corebrightnessdiag status-info` is
//! parsed for `DisplayServicesBrightness` (a 0..1 float). If the
//! tool is missing, slow, or its output does not parse, brightness
//! is `None` and the brightness peek simply never fires — volume
//! still works. No private API, no new crates, no entitlements.
//!
//! Keyboard backlight: DEFERRED to stretch (no stable public API).
//! Nothing in this module attempts it — see the OSD note in
//! Settings.

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct HudState {
    /// Output volume 0..=100, or None when it cannot be read.
    pub volume: Option<i32>,
    pub muted: bool,
    /// Display brightness 0..=100, or None when it cannot be read.
    pub brightness: Option<f64>,
}

#[tauri::command]
pub fn get_hud_state() -> HudState {
    #[cfg(target_os = "macos")]
    {
        let (volume, muted) = read_volume();
        HudState {
            volume,
            muted,
            brightness: read_brightness(),
        }
    }
    #[cfg(not(target_os = "macos"))]
    {
        HudState {
            volume: None,
            muted: false,
            brightness: None,
        }
    }
}

#[cfg(target_os = "macos")]
fn read_volume() -> (Option<i32>, bool) {
    let output = std::process::Command::new("osascript")
        .args([
            "-e",
            "((output volume of (get volume settings)) as string) & \"|\" & ((output muted of (get volume settings)) as string)",
        ])
        .output();
    let Ok(output) = output else {
        return (None, false);
    };
    if !output.status.success() {
        return (None, false);
    }
    parse_volume(&String::from_utf8_lossy(&output.stdout))
}

/// Parse `"72|false"` — the volume, then the muted flag. A volume
/// outside 0..=100 is treated as unreadable, never clamped into a
/// lie.
fn parse_volume(text: &str) -> (Option<i32>, bool) {
    let mut parts = text.trim().split('|');
    let volume = parts
        .next()
        .and_then(|s| s.trim().parse::<i32>().ok())
        .filter(|v| (0..=100).contains(v));
    let muted = parts.next().map(|s| s.trim() == "true").unwrap_or(false);
    (volume, muted)
}

#[cfg(target_os = "macos")]
fn read_brightness() -> Option<f64> {
    let output = std::process::Command::new("/usr/libexec/corebrightnessdiag")
        .args(["status-info"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    parse_brightness(&String::from_utf8_lossy(&output.stdout))
}

/// Find the `DisplayServicesBrightness` line and take the first
/// number after the `=`: a 0..1 float is a fraction (×100), a 1..100
/// number is already a percent. The first matching line wins (the
/// main display). Anything else → None.
fn parse_brightness(text: &str) -> Option<f64> {
    for line in text.lines() {
        if !line.contains("DisplayServicesBrightness") {
            continue;
        }
        let after = line.split('=').nth(1)?;
        let num = parse_first_number(after)?;
        if !(0.0..=100.0).contains(&num) {
            return None;
        }
        if num <= 1.0 {
            return Some(num * 100.0);
        }
        return Some(num);
    }
    None
}

/// First run of digits (with an optional decimal point) in `s`.
/// A lone "." with no digit is not a number.
fn parse_first_number(s: &str) -> Option<f64> {
    let mut current = String::new();
    for ch in s.chars() {
        if ch.is_ascii_digit() || ch == '.' {
            current.push(ch);
        } else if !current.is_empty() {
            break;
        }
    }
    if !current.chars().any(|c| c.is_ascii_digit()) {
        return None;
    }
    current.parse::<f64>().ok()
}

#[cfg(test)]
mod tests {
    use super::{parse_brightness, parse_first_number, parse_volume};

    #[test]
    fn parses_volume_and_muted() {
        assert_eq!(parse_volume("72|false\n"), (Some(72), false));
        assert_eq!(parse_volume("0|true"), (Some(0), true));
        assert_eq!(parse_volume("100|false"), (Some(100), false));
    }

    #[test]
    fn volume_out_of_range_is_unreadable() {
        assert_eq!(parse_volume("101|false"), (None, false));
        assert_eq!(parse_volume("abc|false"), (None, false));
        assert_eq!(parse_volume(""), (None, false));
    }

    #[test]
    fn parses_brightness_fraction() {
        let text = "DisplayServicesBrightness = 0.85\n";
        assert_eq!(parse_brightness(text), Some(85.0));
    }

    #[test]
    fn parses_brightness_percent_and_quoted() {
        let text = "  DisplayServicesBrightness = \"62.5\"\n";
        assert_eq!(parse_brightness(text), Some(62.5));
    }

    #[test]
    fn brightness_missing_or_garbage_is_none() {
        assert_eq!(parse_brightness("nothing here\n"), None);
        assert_eq!(parse_brightness("DisplayServicesBrightness = abc\n"), None);
        assert_eq!(
            parse_brightness("DisplayServicesBrightness = 250\n"),
            None
        );
    }

    #[test]
    fn first_number_scans_past_text() {
        assert_eq!(parse_first_number("value: 12.5 units"), Some(12.5));
        assert_eq!(parse_first_number("."), None);
        assert_eq!(parse_first_number("no digits"), None);
    }
}
