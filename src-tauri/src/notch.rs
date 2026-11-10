//! Notch geometry and expand / collapse.
//!
//! macOS has no public Dynamic Island API, so — like the original — this
//! MVP draws its own borderless overlay window and reads the notch area
//! from `NSScreen`.
//!
//! The macOS implementation uses `objc2` / `objc2-app-kit` /
//! `objc2-foundation` (macOS-only dependencies in Cargo.toml). The
//! non-macOS implementation is an explicit stub so `cargo check` on
//! Linux can still parse this file; it is NOT a macOS build.
//!
//! v0.2 Phase 7 (F21, multi-display): the window is still ONE window
//! (one window per display is not shipped in v0.2 — see
//! `displays.rs`), but it is now display-aware: `set_notch_expanded`
//! / `set_notch_peek` size and position the window on whichever
//! monitor it currently sits on, using that monitor's own notch
//! geometry (or `non_notch_height` on a notch-less display), and
//! placement uses physical coordinates so a move between monitors
//! with different scale factors stays centred.

use serde::Serialize;
use tauri::{AppHandle, LogicalPosition, LogicalSize, Manager, Monitor, Position, Size, Window};

use crate::settings::load_settings;

/// Sneak-peek window height (§5.2: ~70px transient one-line HUD).
/// Now a setting (`settings.notch.sneak_peek_height`, Phase 6);
/// `set_notch_peek` below reads it live and falls back to 70 when
/// the saved value is 0 or out of range.
pub const SNEAK_PEEK_HEIGHT_FALLBACK: f64 = 70.0;

/// Geometry returned to the frontend. All sizes are in logical points.
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct NotchGeometry {
    pub idle_width: f64,
    pub idle_height: f64,
    pub expanded_width: f64,
    pub expanded_height: f64,
    pub has_notch: bool,
    pub screen_width: f64,
    pub screen_height: f64,
}

impl NotchGeometry {
    /// Fallback used on non-notch screens and on non-macOS.
    /// 199x36 is the idle pill measured on a notched MacBook in earlier
    /// community reports; 680x360 is the v0.2 Phase 2 expanded panel
    /// size (media + calendar placeholder + bottom strip need the
    /// extra height over Phase 1's 320).
    pub fn fallback(screen_width: f64, screen_height: f64) -> Self {
        Self {
            idle_width: 199.0,
            idle_height: 36.0,
            expanded_width: 680.0,
            expanded_height: 360.0,
            has_notch: false,
            screen_width,
            screen_height,
        }
    }
}

/// Centre a window of `width` at the top of the screen (y = 0).
fn top_centre_x(geometry: &NotchGeometry, width: f64) -> f64 {
    (geometry.screen_width - width) / 2.0
}

/// Measured geometry with the user's sizing settings applied (F8
/// basics, Phase 2). The settings already exist in settings.json
/// (Settings → Notch, Phase 0); a value of 0 keeps the measured
/// default. On a non-notch display the idle height comes from
/// `non_notch_height`.
fn effective_geometry(app: &AppHandle) -> Result<NotchGeometry, String> {
    let settings = load_settings(app);
    let mut geometry = inner_get_notch_geometry()?;
    apply_sizing_settings(&mut geometry, &settings);
    Ok(geometry)
}

/// The sizing-settings half of `effective_geometry`, shared with the
/// per-monitor path so both apply the same rules.
fn apply_sizing_settings(geometry: &mut NotchGeometry, settings: &crate::settings::Settings) {
    if settings.notch.expanded_width > 0.0 {
        geometry.expanded_width = settings.notch.expanded_width;
    }
    if settings.notch.expanded_height > 0.0 {
        geometry.expanded_height = settings.notch.expanded_height;
    }
    if settings.notch.idle_width > 0.0 {
        geometry.idle_width = settings.notch.idle_width;
    }
    if !geometry.has_notch && settings.notch.non_notch_height > 0.0 {
        geometry.idle_height = settings.notch.non_notch_height;
    }
}

/// Geometry for one specific monitor (Phase 7, F21): the measured
/// notch state of that monitor, with the user's sizing settings
/// applied. On a non-notch monitor `has_notch` is false and the idle
/// height comes from `non_notch_height`, exactly as on the main
/// screen.
pub fn geometry_for_monitor(app: &AppHandle, monitor: &Monitor) -> NotchGeometry {
    let settings = load_settings(app);
    let mut geometry = inner_geometry_for_monitor(monitor);
    apply_sizing_settings(&mut geometry, &settings);
    geometry
}

/// Place `window` on `monitor`, centred at the top (y = 0), at a
/// logical size of `width` × `height`. Physical coordinates are used
/// for both size and position so the placement is exact even when
/// the window moves between monitors with different scale factors
/// (a logical position would be re-scaled by the window's *current*
/// monitor, which is the one it is leaving).
pub fn place_window_on_monitor(
    window: &Window,
    monitor: &Monitor,
    width: f64,
    height: f64,
) -> Result<(), String> {
    let scale = monitor.scale_factor();
    let scale = if scale > 0.0 && scale.is_finite() {
        scale
    } else {
        1.0
    };
    let phys_w = ((width * scale).round() as u32).max(1);
    let phys_h = ((height * scale).round() as u32).max(1);
    let phys_x = monitor.position().x
        + ((monitor.size().width as f64 - phys_w as f64) / 2.0).round() as i32;
    let phys_y = monitor.position().y;
    window
        .set_size(Size::Physical(tauri::PhysicalSize {
            width: phys_w,
            height: phys_h,
        }))
        .map_err(|e| e.to_string())?;
    window
        .set_position(Position::Physical(tauri::PhysicalPosition {
            x: phys_x,
            y: phys_y,
        }))
        .map_err(|e| e.to_string())?;
    Ok(())
}

/// Resize + reposition for the monitor the window currently sits on.
/// The size comes from THAT monitor's geometry (its idle width may
/// differ from the main screen's — e.g. the notch width vs the
/// 199 pt fallback on a notch-less display), so the geometry is
/// resolved first and the size derived from it, never the other way
/// round. When the current monitor cannot be determined (it should
/// not happen on macOS), fall back to the main-screen geometry and
/// the pre-Phase-7 logical placement so the panel still opens.
/// `peek_height`: Some(h) = sneak-peek size (expanded width, h
/// tall); None = expanded/collapsed size by `expanded`.
fn resize_window(
    window: &Window,
    expanded: bool,
    peek_height: Option<f64>,
) -> Result<NotchGeometry, String> {
    let app = window.app_handle().clone();
    let size_for = |g: &NotchGeometry| -> (f64, f64) {
        if let Some(h) = peek_height {
            (g.expanded_width, h)
        } else if expanded {
            (g.expanded_width, g.expanded_height)
        } else {
            (g.idle_width, g.idle_height)
        }
    };
    if let Ok(Some(monitor)) = window.current_monitor() {
        let geometry = geometry_for_monitor(&app, &monitor);
        let (width, height) = size_for(&geometry);
        place_window_on_monitor(window, &monitor, width, height)?;
        return Ok(geometry);
    }
    let geometry = effective_geometry(&app)?;
    let (width, height) = size_for(&geometry);
    let x = top_centre_x(&geometry, width);
    window
        .set_size(Size::Logical(LogicalSize { width, height }))
        .map_err(|e| e.to_string())?;
    window
        .set_position(Position::Logical(LogicalPosition { x, y: 0.0 }))
        .map_err(|e| e.to_string())?;
    Ok(geometry)
}

#[tauri::command]
pub fn get_notch_geometry(app: AppHandle) -> Result<NotchGeometry, String> {
    effective_geometry(&app)
}

#[tauri::command]
pub fn set_notch_expanded(window: Window, expanded: bool) -> Result<NotchGeometry, String> {
    resize_window(&window, expanded, None)
}

/// Sneak-peek resize (Phase 2, height made a setting in Phase 6):
/// the transient one-line HUD is as wide as the expanded panel but
/// only ~70px tall; hiding it returns to the idle pill. The
/// frontend drives this from its peek state machine and never calls
/// it while the panel is expanded.
#[tauri::command]
pub fn set_notch_peek(window: Window, show: bool) -> Result<NotchGeometry, String> {
    let app = window.app_handle().clone();
    let peek_height = {
        let h = load_settings(&app).notch.sneak_peek_height;
        if (40.0..=200.0).contains(&h) {
            h
        } else {
            SNEAK_PEEK_HEIGHT_FALLBACK
        }
    };
    if show {
        resize_window(&window, false, Some(peek_height))
    } else {
        resize_window(&window, false, None)
    }
}

/// Apply the overlay behaviour to the main window.
///
/// On macOS this raises the native window level and makes it join all
/// Spaces / sit above full-screen apps. On other platforms it is a
/// no-op — Tauri's `alwaysOnTop` / `skipTaskbar` settings in
/// `tauri.conf.json` already do the portable part.
pub fn apply_notch_window(window: &Window) {
    #[cfg(target_os = "macos")]
    {
        macos::apply_notch_window_native(window);
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = window;
    }
}

// ---------------------------------------------------------------------------
// macOS implementation
// ---------------------------------------------------------------------------

#[cfg(target_os = "macos")]
fn inner_get_notch_geometry() -> Result<NotchGeometry, String> {
    macos::get_notch_geometry_macos()
}

#[cfg(target_os = "macos")]
fn inner_geometry_for_monitor(monitor: &Monitor) -> NotchGeometry {
    macos::geometry_for_monitor_macos(monitor)
}

#[cfg(target_os = "macos")]
mod macos {
    use super::NotchGeometry;
    use objc2::MainThreadMarker;
    use objc2_app_kit::{NSStatusWindowLevel, NSScreen, NSWindow, NSWindowCollectionBehavior};
    use objc2_foundation::{NSArray, NSRect};
    use tauri::{Monitor, Window};

    /// Notch state of one screen: (has_notch, notch_width,
    /// screen_width, screen_height), all in logical points.
    /// safeAreaInsets.top is > 0 on a notched display (the notch
    /// intrudes into the menu-bar area). The auxiliary areas are the
    /// two rectangles to the left and right of the notch, added in
    /// macOS 14 for exactly this purpose.
    fn screen_notch_info(screen: &NSScreen) -> (bool, f64, f64, f64) {
        let frame: NSRect = screen.frame();
        let screen_width = frame.size.width;
        let screen_height = frame.size.height;
        let insets = screen.safeAreaInsets();
        let left = screen.auxiliaryTopLeftArea();
        let right = screen.auxiliaryTopRightArea();
        let notch_left = left.origin.x + left.size.width;
        let notch_right = right.origin.x;
        let notch_width = notch_right - notch_left;
        let has_notch = insets.top > 0.0 && notch_width > 0.0;
        (has_notch, notch_width, screen_width, screen_height)
    }

    fn geometry_from_screen(screen: &NSScreen) -> NotchGeometry {
        let (has_notch, notch_width, screen_width, screen_height) = screen_notch_info(screen);
        if has_notch {
            // Idle pill hugs the notch; expanded panel is wider and
            // taller, centred on the notch centre.
            let idle_width = notch_width.max(120.0).min(260.0);
            NotchGeometry {
                idle_width,
                idle_height: 36.0,
                expanded_width: 680.0,
                expanded_height: 360.0,
                has_notch: true,
                screen_width,
                screen_height,
            }
        } else {
            NotchGeometry::fallback(screen_width, screen_height)
        }
    }

    pub fn get_notch_geometry_macos() -> Result<NotchGeometry, String> {
        let mtm = MainThreadMarker::new().ok_or("not on the main thread")?;
        let screen = NSScreen::mainScreen(mtm).ok_or("no main screen found")?;
        Ok(geometry_from_screen(&screen))
    }

    /// Geometry for the NSScreen that matches `monitor`. Matching is
    /// by logical size (physical size / scale factor, within 1 pt):
    /// the Tauri monitor's position is in winit's virtual-screen
    /// coordinates (y-down from the primary monitor's top-left),
    /// while NSScreen frames are y-up from the main display's
    /// bottom-left, so positions do not compare directly — but sizes
    /// do, and in practice the built-in and external displays differ
    /// in size. If two screens share a size, the notched one wins
    /// (only the built-in display has a notch, and the idle width is
    /// the only value that differs between same-size screens). No
    /// match at all falls back to the monitor's own logical size
    /// with no notch.
    pub fn geometry_for_monitor_macos(monitor: &Monitor) -> NotchGeometry {
        let Ok(mtm) = MainThreadMarker::new().ok_or(()) else {
            return monitor_fallback(monitor);
        };
        let scale = monitor.scale_factor();
        let scale = if scale > 0.0 && scale.is_finite() {
            scale
        } else {
            1.0
        };
        let logical_w = monitor.size().width as f64 / scale;
        let logical_h = monitor.size().height as f64 / scale;
        let screens: &NSArray<NSScreen> = &NSScreen::screens(mtm);
        let mut best: Option<NotchGeometry> = None;
        for i in 0..screens.count() {
            let screen = screens.objectAtIndex(i);
            let frame: NSRect = screen.frame();
            if (frame.size.width - logical_w).abs() > 1.0
                || (frame.size.height - logical_h).abs() > 1.0
            {
                continue;
            }
            let geometry = geometry_from_screen(&screen);
            // A notched match beats a notch-less one of the same size.
            if geometry.has_notch {
                return geometry;
            }
            if best.is_none() {
                best = Some(geometry);
            }
        }
        best.unwrap_or_else(|| monitor_fallback(monitor))
    }

    fn monitor_fallback(monitor: &Monitor) -> NotchGeometry {
        let scale = monitor.scale_factor();
        let scale = if scale > 0.0 && scale.is_finite() {
            scale
        } else {
            1.0
        };
        NotchGeometry::fallback(
            monitor.size().width as f64 / scale,
            monitor.size().height as f64 / scale,
        )
    }

    pub fn apply_notch_window_native(window: &Window) {
        // Tauri exposes the underlying NSWindow as a raw pointer on macOS.
        // NSStatusWindowLevel is 25: above the normal window level, which
        // is what notch overlays use so the panel stays visible over
        // full-screen apps. (NSWindowLevel is only a type alias for
        // NSInteger in objc2-app-kit, so it cannot be constructed as
        // NSWindowLevel(25); use the named level constant instead.)
        // If the pointer is unavailable we simply keep Tauri's
        // alwaysOnTop setting and do not fail the app start.
        let Ok(raw) = window.ns_window() else {
            return;
        };
        if raw.is_null() {
            return;
        }
        unsafe {
            let ns_window: &NSWindow = &*(raw as *const NSWindow);
            ns_window.setLevel(NSStatusWindowLevel);
            let behaviour = NSWindowCollectionBehavior::CanJoinAllSpaces
                | NSWindowCollectionBehavior::Stationary
                | NSWindowCollectionBehavior::FullScreenAuxiliary;
            ns_window.setCollectionBehavior(behaviour);
            // The window must not steal keyboard focus when it appears.
            ns_window.setHidesOnDeactivate(false);
        }
    }
}

// ---------------------------------------------------------------------------
// Non-macOS stub — explicitly NOT a macOS implementation.
// ---------------------------------------------------------------------------

#[cfg(not(target_os = "macos"))]
fn inner_get_notch_geometry() -> Result<NotchGeometry, String> {
    // No NSScreen here. Report a common 1512x982 logical screen (14" MacBook
    // Pro) with the fallback pill so the frontend still renders on Linux.
    Ok(NotchGeometry::fallback(1512.0, 982.0))
}

#[cfg(not(target_os = "macos"))]
fn inner_geometry_for_monitor(monitor: &Monitor) -> NotchGeometry {
    // Same fallback, sized to the monitor's own logical size.
    let scale = monitor.scale_factor();
    let scale = if scale > 0.0 && scale.is_finite() {
        scale
    } else {
        1.0
    };
    NotchGeometry::fallback(
        monitor.size().width as f64 / scale,
        monitor.size().height as f64 / scale,
    )
}
