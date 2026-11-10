//! AirDrop send for the file shelf (v0.2 Phase 4b, F7).
//!
//! Mechanism — verified against the real objc2-app-kit 0.3.2 crate
//! source (local cargo registry), not from memory:
//!
//! * The service is looked up with the official AppKit constant
//!   `NSSharingServiceNameSendViaAirDrop` (an `extern "C"` static
//!   documented at developer.apple.com/documentation/appkit/
//!   nssharingservicenamesendviaairdrop). Its AppKit value is the
//!   community-documented string `com.apple.share.AirDrop.send`, but
//!   the code uses the constant, never a hardcoded string.
//! * `NSSharingService::sharingServiceNamed(&name)` returns
//!   `Option<Retained<NSSharingService>>` — `None` when the service
//!   is unavailable, so a missing AirDrop can never crash the app.
//! * `canPerformWithItems(Some(&items))` is checked before
//!   `performWithItems(&items)`; both take `&NSArray` (untyped) and
//!   are `unsafe` because the items generic must be correct — the
//!   items are `NSURL` file URLs, which the AppKit docs list as valid
//!   share items ("If an NSURL is a file URL … the content of the
//!   file will be shared").
//! * `performWithItems` displays the sharing window (the AirDrop
//!   device picker) — per the AppKit docs, "In most cases this will
//!   display a sharing window."
//! * `NSSharingService` is in objc2-app-kit 0.3.2's default features,
//!   so no Cargo.toml change was needed.
//!
//! Honest limits (per the plan, §7 Phase 4b / risk R3):
//!
//! * The call only *presents* the AirDrop picker. Whether the user
//!   picks a device and the transfer completes is not observable
//!   without an `NSSharingServiceDelegate` (a custom Objective-C
//!   class — out of scope for this phase), so a successful return
//!   means "the picker opened", not "the file arrived".
//! * Every failure mode returns `Err(String)` instead of failing
//!   silently: service missing, items not shareable, macOS-only.
//!   The frontend shows the error on the button status line and the
//!   shelf keeps working.
//! * The button sends **all** files currently on the shelf (there is
//!   no per-file selection model — a tile click already means "open").
//!
//! Main thread: AppKit UI must run on the main thread. Tauri
//! commands are not guaranteed to run there, so each macOS entry
//! point goes through `run_on_main`: it runs the work directly when
//! already on the main thread (`MainThreadMarker`), and otherwise
//! dispatches with `AppHandle::run_on_main_thread` and awaits the
//! result over an `mpsc` channel with a 5 s timeout (so a stuck
//! dispatch can never hang the command forever, and posting from
//! the main thread and then blocking — a deadlock — is avoided).

use tauri::AppHandle;

#[tauri::command]
pub fn airdrop_available(app: AppHandle) -> bool {
    #[cfg(target_os = "macos")]
    {
        macos::run_on_main(&app, macos::service_configured).unwrap_or(false)
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = app;
        false
    }
}

#[tauri::command]
pub fn airdrop_shelf_files(app: AppHandle, paths: Vec<String>) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        macos::airdrop_shelf_files_macos(&app, paths)
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = app;
        let _ = paths;
        Err("AirDrop is only available on macOS".to_string())
    }
}

#[cfg(target_os = "macos")]
mod macos {
    use super::*;
    use objc2::MainThreadMarker;
    use objc2_app_kit::{NSSharingService, NSSharingServiceNameSendViaAirDrop};
    use objc2_foundation::{NSArray, NSString, NSURL};
    use std::time::Duration;

    /// Run `f` on the macOS main thread and wait for its result.
    /// Runs directly when already on the main thread; otherwise
    /// dispatches via `AppHandle::run_on_main_thread` (which posts
    /// to the event loop) and blocks on a channel with a timeout.
    pub fn run_on_main<F, R>(app: &AppHandle, f: F) -> Result<R, String>
    where
        F: FnOnce() -> R + Send + 'static,
        R: Send + 'static,
    {
        if MainThreadMarker::new().is_some() {
            return Ok(f());
        }
        let (tx, rx) = std::sync::mpsc::channel();
        app.run_on_main_thread(move || {
            let _ = tx.send(f());
        })
        .map_err(|e| format!("could not dispatch to the main thread: {e}"))?;
        rx.recv_timeout(Duration::from_secs(5))
            .map_err(|_| "AirDrop did not respond on the main thread".to_string())
    }

    /// The launch-time check from the AppKit docs: `canPerformWithItems`
    /// with `nil` items returns YES when the service is configured.
    /// Must run on the main thread (callers go through `run_on_main`).
    pub fn service_configured() -> bool {
        // SAFETY: the constant is a valid AppKit NSString provided
        // by the system framework; reading the linked symbol is the
        // documented way to obtain it.
        let service_name = unsafe { NSSharingServiceNameSendViaAirDrop };
        let Some(service) = NSSharingService::sharingServiceNamed(service_name) else {
            return false;
        };
        // SAFETY: no items are passed (None), so there is no items
        // generic to get wrong; the method documents this exact use.
        unsafe { service.canPerformWithItems(None) }
    }

    pub fn airdrop_shelf_files_macos(app: &AppHandle, paths: Vec<String>) -> Result<(), String> {
        // Keep only files that still exist; a shelf entry deleted
        // since the list was loaded must not break the send.
        let existing: Vec<String> = paths
            .into_iter()
            .filter(|p| std::path::Path::new(p).exists())
            .collect();
        if existing.is_empty() {
            return Err("no files on the shelf to send".to_string());
        }
        run_on_main(app, move || send(existing))?
    }

    /// Present the AirDrop picker for these files. Must run on the
    /// main thread (callers go through `run_on_main`).
    fn send(paths: Vec<String>) -> Result<(), String> {
        // SAFETY: see service_configured above.
        let service_name = unsafe { NSSharingServiceNameSendViaAirDrop };
        let Some(service) = NSSharingService::sharingServiceNamed(service_name) else {
            return Err("AirDrop service is unavailable on this Mac".to_string());
        };
        let urls: Vec<_> = paths
            .iter()
            .map(|p| NSURL::fileURLWithPath(&NSString::from_str(p)))
            .collect();
        let items = NSArray::from_retained_slice(&urls);
        // The service methods take untyped `&NSArray` (= `NSArray<AnyObject>`).
        // SAFETY: `NSArray<NSURL>` and `NSArray<AnyObject>` have the
        // same layout (both are the same ObjC object), and every
        // element is an NSURL, which is a valid AnyObject.
        let items_any: &NSArray = unsafe { items.cast_unchecked() };
        // SAFETY: the items generic is correct — the array holds only
        // NSURL file URLs, which the AppKit docs list as valid share
        // items.
        if unsafe { !service.canPerformWithItems(Some(items_any)) } {
            return Err("AirDrop cannot share these files".to_string());
        }
        // SAFETY: same items as just validated above.
        unsafe { service.performWithItems(items_any) };
        Ok(())
    }
}
