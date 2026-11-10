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

use serde::Serialize;
use tauri::{LogicalPosition, LogicalSize, Manager, Position, Size, Window};

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
    /// community reports; 680x200 is the MVP expanded panel size.
    pub fn fallback(screen_width: f64, screen_height: f64) -> Self {
        Self {
            idle_width: 199.0,
            idle_height: 36.0,
            expanded_width: 680.0,
            expanded_height: 200.0,
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

#[tauri::command]
pub fn get_notch_geometry() -> Result<NotchGeometry, String> {
    inner_get_notch_geometry()
}

#[tauri::command]
pub fn set_notch_expanded(window: Window, expanded: bool) -> Result<NotchGeometry, String> {
    let geometry = inner_get_notch_geometry()?;
    let (width, height) = if expanded {
        (geometry.expanded_width, geometry.expanded_height)
    } else {
        (geometry.idle_width, geometry.idle_height)
    };
    let x = top_centre_x(&geometry, width);
    window
        .set_size(Size::Logical(LogicalSize { width, height }))
        .map_err(|e| e.to_string())?;
    window
        .set_position(Position::Logical(LogicalPosition { x, y: 0.0 }))
        .map_err(|e| e.to_string())?;
    Ok(geometry)
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
mod macos {
    use super::NotchGeometry;
    use objc2::MainThreadMarker;
    use objc2_app_kit::{NSWindow, NSWindowCollectionBehavior, NSWindowLevel};
    use objc2_foundation::NSRect;
    use tauri::Window;

    pub fn get_notch_geometry_macos() -> Result<NotchGeometry, String> {
        let mtm = MainThreadMarker::new().ok_or("not on the main thread")?;
        let screen = objc2_app_kit::NSScreen::mainScreen(mtm)
            .ok_or("no main screen found")?;
        let frame: NSRect = screen.frame();
        let screen_width = frame.size.width;
        let screen_height = frame.size.height;

        // safeAreaInsets.top is > 0 on a notched display (the notch
        // intrudes into the menu-bar area). The auxiliary areas are the
        // two rectangles to the left and right of the notch, added in
        // macOS 14 for exactly this purpose.
        let insets = screen.safeAreaInsets();
        let left = screen.auxiliaryTopLeftArea();
        let right = screen.auxiliaryTopRightArea();

        let notch_left = left.origin.x + left.size.width;
        let notch_right = right.origin.x;
        let notch_width = notch_right - notch_left;

        let has_notch = insets.top > 0.0 && notch_width > 0.0;

        if has_notch {
            // Idle pill hugs the notch; expanded panel is wider and
            // taller, centred on the notch centre.
            let idle_width = notch_width.max(120.0).min(260.0);
            Ok(NotchGeometry {
                idle_width,
                idle_height: 36.0,
                expanded_width: 680.0,
                expanded_height: 200.0,
                has_notch: true,
                screen_width,
                screen_height,
            })
        } else {
            Ok(NotchGeometry::fallback(screen_width, screen_height))
        }
    }

    pub fn apply_notch_window_native(window: &Window) {
        // Tauri exposes the underlying NSWindow as a raw pointer on macOS.
        // Level 25 is above the normal window level and is what notch
        // overlays use so the panel stays visible over full-screen apps.
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
            ns_window.setLevel(NSWindowLevel(25));
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
