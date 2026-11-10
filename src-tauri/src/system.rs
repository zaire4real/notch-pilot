//! Battery and time.
//!
//! macOS battery is parsed from `pmset -g batt`, e.g.
//!   " -InternalBattery-0 (id=1234567)  85%; charging; 0:42 remaining present: true"
//! No extra crates are used; parsing is done by hand so a format change
//! degrades to "unknown" instead of crashing.

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct BatteryInfo {
    /// 0..=100, or -1 when unknown.
    pub percent: i32,
    pub charging: bool,
    /// "AC Power" / "Battery Power" / "Unknown".
    pub source: String,
}

#[tauri::command]
pub fn get_battery() -> Result<BatteryInfo, String> {
    #[cfg(target_os = "macos")]
    {
        let output = std::process::Command::new("pmset")
            .args(["-g", "batt"])
            .output()
            .map_err(|e| e.to_string())?;
        let text = String::from_utf8_lossy(&output.stdout).to_string();
        Ok(parse_pmset(&text))
    }
    #[cfg(not(target_os = "macos"))]
    {
        Err("Battery info is only supported on macOS".to_string())
    }
}

#[tauri::command]
pub fn get_time() -> Result<String, String> {
    // `date` exists on macOS and Linux and already honours the user's
    // locale / timezone, so the frontend does not need its own clock math.
    let output = std::process::Command::new("date")
        .arg("+%Y-%m-%d %H:%M")
        .output()
        .map_err(|e| e.to_string())?;
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// Parse `pmset -g batt` output by hand. Returns percent -1 on failure.
fn parse_pmset(text: &str) -> BatteryInfo {
    let source = if text.contains("AC Power") {
        "AC Power".to_string()
    } else if text.contains("Battery Power") {
        "Battery Power".to_string()
    } else {
        "Unknown".to_string()
    };

    // NOTE: "discharging" contains the substring "charging", so check
    // for discharging first — otherwise a discharging battery would
    // be reported as charging.
    let charging = if text.contains("discharging") {
        false
    } else {
        text.contains("charging")
    };
    // The first line contains e.g. "  85%; charging; ..." — find the
    // digits immediately before the first '%'.
    let mut percent: i32 = -1;
    if let Some(pct_pos) = text.find('%') {
        let before = &text[..pct_pos];
        let digits: String = before
            .chars()
            .rev()
            .take_while(|c| c.is_ascii_digit())
            .collect::<Vec<char>>()
            .into_iter()
            .rev()
            .collect();
        if let Ok(n) = digits.parse::<i32>() {
            percent = n;
        }
    }

    BatteryInfo {
        percent,
        charging,
        source,
    }
}

#[cfg(test)]
mod tests {
    use super::parse_pmset;

    #[test]
    fn parses_charging_output() {
        let text = "Now drawing from 'AC Power'\n -InternalBattery-0 (id=1)\t85%; charging; 0:42 remaining present: true\n";
        let info = parse_pmset(text);
        assert_eq!(info.percent, 85);
        assert!(info.charging);
        assert_eq!(info.source, "AC Power");
    }

    #[test]
    fn parses_discharging_output() {
        let text = "Now drawing from 'Battery Power'\n -InternalBattery-0 (id=1)\t63%; discharging; 3:10 remaining present: true\n";
        let info = parse_pmset(text);
        assert_eq!(info.percent, 63);
        assert!(!info.charging);
        assert_eq!(info.source, "Battery Power");
    }
}
