//! Multi-display support (v0.2 Phase 7, F21).
//!
//! Documented scope for v0.2: the app is still ONE window, not one
//! window per display. What this phase ships instead:
//!
//! 1. At startup the panel opens on the display **with the notch**
//!    when one exists (previously it always opened on the main
//!    screen, which need not be the notched one when an external
//!    display is the main display). See `preferred_display`, used by
//!    `lib.rs` setup.
//! 2. When `notch.show_on_all_displays` OR
//!    `notch.follow_active_display` is on, the panel **follows the
//!    display the mouse is on** (`follow_active_display`, polled
//!    from the frontend). This is the v0.2 approximation of "show on
//!    all displays": the single panel is always on the display the
//!    user is working on. One window per display is a later phase —
//!    the Settings → Notch section says so in plain language, and
//!    the two toggles are not presented as doing more than this.
//! 3. `get_displays` exposes the monitor list (logical geometry +
//!    notch state) for future use and for debugging.
//!
//! All geometry comes from Tauri's `Monitor` (winit virtual-screen
//! coordinates, physical pixels) plus `notch::geometry_for_monitor`
//! for the per-display notch state. No new crates, no new
//! permissions.

use serde::Serialize;
use tauri::{AppHandle, Manager, Monitor, Window};

use crate::notch::{geometry_for_monitor, place_window_on_monitor};
use crate::settings::load_settings;

/// One display, as the frontend / future phases see it. All
/// coordinates are logical points in the virtual desktop (the
/// primary monitor's top-left is 0,0).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct DisplayInfo {
    pub index: usize,
    pub name: String,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub scale_factor: f64,
    pub is_primary: bool,
    pub has_notch: bool,
}

fn monitor_logical_rect(monitor: &Monitor) -> (f64, f64, f64, f64) {
    let scale = monitor.scale_factor();
    let scale = if scale > 0.0 && scale.is_finite() {
        scale
    } else {
        1.0
    };
    (
        monitor.position().x as f64 / scale,
        monitor.position().y as f64 / scale,
        monitor.size().width as f64 / scale,
        monitor.size().height as f64 / scale,
    )
}

fn display_info(app: &AppHandle, monitor: &Monitor, index: usize, is_primary: bool) -> DisplayInfo {
    let (x, y, width, height) = monitor_logical_rect(monitor);
    DisplayInfo {
        index,
        name: monitor.name().cloned().unwrap_or_default(),
        x,
        y,
        width,
        height,
        scale_factor: monitor.scale_factor(),
        is_primary,
        has_notch: geometry_for_monitor(app, monitor).has_notch,
    }
}

#[tauri::command]
pub fn get_displays(app: AppHandle) -> Vec<DisplayInfo> {
    let Ok(monitors) = app.available_monitors() else {
        return Vec::new();
    };
    let primary = app.primary_monitor().ok().flatten();
    monitors
        .iter()
        .enumerate()
        .map(|(i, m)| {
            let is_primary = primary.as_ref().is_some_and(|p| {
                p.position() == m.position() && p.size() == m.size()
            });
            display_info(&app, m, i, is_primary)
        })
        .collect()
}

/// The monitor the main window should open on: the first monitor
/// with a notch, else the primary monitor, else the first monitor.
/// `None` when no monitor can be enumerated at all.
pub fn preferred_display(app: &AppHandle) -> Option<Monitor> {
    let monitors = app.available_monitors().ok()?;
    if monitors.is_empty() {
        return None;
    }
    if let Some(notched) = monitors
        .iter()
        .find(|m| geometry_for_monitor(app, m).has_notch)
    {
        return Some(notched.clone());
    }
    if let Ok(Some(primary)) = app.primary_monitor() {
        return Some(primary);
    }
    monitors.into_iter().next()
}

/// Move the main window to the display the mouse cursor is on, if
/// it is not there already — but only when the user asked for it:
/// `notch.show_on_all_displays` or `notch.follow_active_display`.
/// The window keeps its current state (`expanded`): an expanded
/// panel re-opens at expanded size on the new display, a collapsed
/// pill at that display's idle size (which honours
/// `non_notch_height` on a notch-less display).
///
/// Returns Ok(true) when the window moved, Ok(false) when it was
/// already on the cursor's display (or when following is off, or
/// when there is only one display). The frontend polls this every
/// few seconds; a `false` result is the common, cheap case — the
/// cursor-position and current-monitor lookups are local calls.
#[tauri::command]
pub fn follow_active_display(window: Window, expanded: bool) -> Result<bool, String> {
    let app = window.app_handle().clone();
    let settings = load_settings(&app);
    if !settings.notch.show_on_all_displays && !settings.notch.follow_active_display {
        return Ok(false);
    }
    let monitors = app.available_monitors().map_err(|e| e.to_string())?;
    if monitors.len() < 2 {
        return Ok(false);
    }
    let cursor = window.cursor_position().map_err(|e| e.to_string())?;
    // Cursor position is in physical pixels in the same virtual
    // screen space as Monitor::position/size, so containment is a
    // direct physical comparison — no scale conversion.
    let target = monitors.iter().find(|m| {
        let px = cursor.x;
        let py = cursor.y;
        let ox = m.position().x as f64;
        let oy = m.position().y as f64;
        let w = m.size().width as f64;
        let h = m.size().height as f64;
        px >= ox && px < ox + w && py >= oy && py < oy + h
    });
    let Some(target) = target else {
        return Ok(false);
    };
    let current = window.current_monitor().map_err(|e| e.to_string())?;
    if let Some(current) = current {
        if current.position() == target.position() && current.size() == target.size() {
            return Ok(false);
        }
    }
    let geometry = geometry_for_monitor(&app, target);
    let (width, height) = if expanded {
        (geometry.expanded_width, geometry.expanded_height)
    } else {
        (geometry.idle_width, geometry.idle_height)
    };
    place_window_on_monitor(&window, target, width, height)?;
    Ok(true)
}
