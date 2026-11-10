//! Bluetooth live activity (v0.2 Phase 7, F10).
//!
//! `objc2-core-bluetooth` does exist (since objc2 0.2.1), but a
//! CoreBluetooth implementation needs a `CBCentralManager` +
//! delegate, an async state machine, and the right service UUIDs to
//! retrieve already-connected classic devices such as AirPods —
//! that is a whole native module for a low-priority item. The plan
//! (§3 F10) explicitly allows the simpler route, so this module
//! parses `system_profiler SPBluetoothDataType` instead: no new
//! crates, no entitlements, no delegate.
//!
//! Cost: the call takes ~1 s, so the frontend polls it on expand
//! only (plus a slow background tick while collapsed, see
//! `src/main.js`) — never on a hot path. A failed or unparseable
//! run reports `available: false` and an empty list; the panel then
//! hides the Bluetooth row instead of showing an error.
//!
//! `system_profiler` output is English regardless of the system
//! locale; the parser keys on the literal section/device/property
//! names, so it is not locale-sensitive.

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct BluetoothState {
    /// False when Bluetooth is absent or the profiler output has no
    /// Bluetooth section at all.
    pub available: bool,
    /// Names of devices whose `Connected:` property is `Yes`, in
    /// the order system_profiler lists them.
    pub devices: Vec<String>,
}

#[tauri::command]
pub fn get_bluetooth_devices() -> BluetoothState {
    #[cfg(target_os = "macos")]
    {
        let output = std::process::Command::new("system_profiler")
            .args(["SPBluetoothDataType"])
            .output();
        let Ok(output) = output else {
            return BluetoothState {
                available: false,
                devices: Vec::new(),
            };
        };
        if !output.status.success() {
            return BluetoothState {
                available: false,
                devices: Vec::new(),
            };
        }
        parse_bluetooth(&String::from_utf8_lossy(&output.stdout))
    }
    #[cfg(not(target_os = "macos"))]
    {
        BluetoothState {
            available: false,
            devices: Vec::new(),
        }
    }
}

/// Parse `system_profiler SPBluetoothDataType` output.
///
/// Layout (4-space indent per level):
/// ```text
/// Bluetooth:
///     ...
///     Devices:
///         AirPods Pro:
///             Address: XX-XX-XX-XX-XX-XX
///             Connected: Yes
///         Magic Mouse:
///             Connected: No
/// ```
/// A device is a line at exactly one indent deeper than `Devices:`
/// that ends in `:`; its `Connected: Yes` property (deeper still)
/// marks it connected. A top-level section other than `Bluetooth:`
/// ends the scan. A missing `Devices:` section means no paired
/// devices, not an error.
fn parse_bluetooth(text: &str) -> BluetoothState {
    let mut available = false;
    let mut devices: Vec<String> = Vec::new();
    let mut in_devices = false;
    let mut devices_indent = 0usize;
    let mut current: Option<String> = None;
    let mut current_connected = false;

    // Flush the device being collected: keep it only if connected.
    let flush = |current: &mut Option<String>, connected: bool, devices: &mut Vec<String>| {
        if let Some(name) = current.take() {
            if connected {
                devices.push(name);
            }
        }
    };

    for line in text.lines() {
        let indent = line.len() - line.trim_start_matches(' ').len();
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if indent == 0 {
            flush(&mut current, current_connected, &mut devices);
            current_connected = false;
            if trimmed == "Bluetooth:" {
                available = true;
                in_devices = false;
            } else {
                in_devices = false;
            }
            continue;
        }
        if trimmed == "Devices:" {
            flush(&mut current, current_connected, &mut devices);
            current_connected = false;
            in_devices = true;
            devices_indent = indent;
            continue;
        }
        if !in_devices {
            continue;
        }
        if indent <= devices_indent {
            // A sibling section of Devices: — the device list is over.
            flush(&mut current, current_connected, &mut devices);
            current_connected = false;
            in_devices = false;
            continue;
        }
        if indent == devices_indent + 4 && trimmed.ends_with(':') {
            flush(&mut current, current_connected, &mut devices);
            current_connected = false;
            current = Some(trimmed.trim_end_matches(':').to_string());
        } else if current.is_some() && trimmed == "Connected: Yes" {
            current_connected = true;
        }
    }
    flush(&mut current, current_connected, &mut devices);
    BluetoothState { available, devices }
}

#[cfg(test)]
mod tests {
    use super::parse_bluetooth;

    const SAMPLE: &str = "Bluetooth:\n\n    Apple Bluetooth Software Version: 9.0.0\n    Hardware, Features, and Settings:\n        Name: MacBook Pro\n        Address: 00-11-22-33-44-55\n    Devices:\n\n        AirPods Pro:\n            Address: AA-BB-CC-DD-EE-FF\n            Minor Type: Headphones\n            Connected: Yes\n            Paired: Yes\n\n        Magic Mouse:\n            Address: 11-22-33-44-55-66\n            Connected: No\n\n        WH-1000XM5:\n            Address: 22-33-44-55-66-77\n            Connected: Yes\n\n    Services:\n        ...\n";

    #[test]
    fn parses_connected_devices_only() {
        let state = parse_bluetooth(SAMPLE);
        assert!(state.available);
        assert_eq!(
            state.devices,
            vec!["AirPods Pro".to_string(), "WH-1000XM5".to_string()]
        );
    }

    #[test]
    fn no_devices_section_is_available_but_empty() {
        let state = parse_bluetooth("Bluetooth:\n\n    Apple Bluetooth Software Version: 9.0.0\n");
        assert!(state.available);
        assert!(state.devices.is_empty());
    }

    #[test]
    fn no_bluetooth_section_is_unavailable() {
        let state = parse_bluetooth("Wi-Fi:\n\n    Software Versions:\n        CoreWLAN: 1.0\n");
        assert!(!state.available);
        assert!(state.devices.is_empty());
    }

    #[test]
    fn empty_output_is_unavailable() {
        let state = parse_bluetooth("");
        assert!(!state.available);
        assert!(state.devices.is_empty());
    }

    #[test]
    fn device_without_connected_property_is_not_listed() {
        let text = "Bluetooth:\n    Devices:\n        Mystery Buds:\n            Address: 00-00-00-00-00-00\n";
        let state = parse_bluetooth(text);
        assert!(state.available);
        assert!(state.devices.is_empty());
    }
}
