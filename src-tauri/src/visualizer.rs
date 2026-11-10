//! Audio visualizer — animated bars only (v0.2.1).
//!
//! History: v0.2 Phase 1b shipped a real system-audio spectrum via
//! ScreenCaptureKit (`SCStream` audio tap, Goertzel filter bank)
//! using the `screencapturekit` crate. That crate compiles a Swift
//! bridge (`Task { try await scStream.startCapture() }`), so the
//! final binary links the Swift runtime as
//! `@rpath/libswift_Concurrency.dylib` (plus libswiftCore /
//! libswiftFoundation) and needs an LC_RPATH to resolve them.
//!
//! That broke launch on real machines: on the user's MacBook Air
//! M3, `/usr/lib/swift/` contains only a handful of framework
//! Swift libs (Photos, Network, ShazamKit, …) — the core Swift
//! runtime lives only in the dyld shared cache, which `@rpath`
//! cannot point into — so dyld aborts at startup with
//! "Library not loaded: @rpath/libswift_Concurrency.dylib …
//! Reason: no LC_RPATH's found". Embedding the Swift dylibs was
//! considered and rejected: the transitive closure cannot be
//! enumerated from this Linux VM, so a missing dylib would just
//! move the crash to the next launch.
//!
//! So the real spectrum is REMOVED in v0.2.1 and the visualizer is
//! the honestly-labelled animated fallback (see `src/main.js`:
//! `visualizer_start` returning `Err` makes the frontend animate
//! the bars and show the "Animated visualizer" label — it never
//! presents them as real audio). The setting
//! `media.visualizer` / `media.visualizer_style` still controls
//! the bars and their style.
//!
//! A real spectrum may return later via a pure-objc2
//! ScreenCaptureKit implementation (no Swift) — that is the only
//! form that can launch on a machine without the Swift runtime
//! on disk.

/// Number of spectrum bands (and frontend bars).
pub const NUM_BANDS: usize = 8;

/// Start the visualizer. Always reports the real spectrum as
/// unavailable, so the frontend falls back to the animated bars.
/// Kept as a command (same signature as before) so the frontend
/// needs no changes.
#[tauri::command]
pub fn visualizer_start() -> Result<String, String> {
    Err("Real audio visualizer is unavailable in this build: the \
         ScreenCaptureKit spectrum needs the Swift runtime, which is \
         not bundled. Using the animated visualizer instead."
        .to_string())
}

/// Stop the visualizer. No-op: there is no stream to stop.
#[tauri::command]
pub fn visualizer_stop() -> Result<(), String> {
    Ok(())
}

/// Current band levels. All zeros — the frontend never polls this
/// in animated mode (it generates its own levels), but the command
/// stays so the API surface is unchanged.
#[tauri::command]
pub fn visualizer_levels() -> Vec<f32> {
    vec![0.0; NUM_BANDS]
}
