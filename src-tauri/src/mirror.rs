//! Camera mirror (v0.2 Phase 5, F4).
//!
//! Mechanism — verified against the real `objc2-av-foundation`
//! 0.3.2 crate source (local cargo registry), not from memory:
//!
//! * `objc2-av-foundation` **does** exist for objc2 0.6 (risk R2
//!   resolved for AVFoundation), so the capture stack uses the
//!   generated bindings, not raw `msg_send!`.
//! * `AVCaptureSession` + `AVCaptureDeviceInput` +
//!   `AVCaptureVideoDataOutput`, with a `define_class!` delegate
//!   implementing `AVCaptureVideoDataOutputSampleBufferDelegate`.
//!   The delegate is a plain `NSObject` subclass; the trait impl
//!   the macro generates is empty — the real dispatch is the ObjC
//!   selector `captureOutput:didOutputSampleBuffer:fromConnection:`
//!   registered by `#[unsafe(method(...))]`, which is how objc2
//!   delegate classes work.
//! * The delegate object is retained in the module `STATE` (the
//!   output holds its delegate weakly), as is the serial dispatch
//!   queue the frames arrive on.
//! * Frames are requested as `kCVPixelFormatType_32BGRA` via the
//!   output's `videoSettings` (built with `NSDictionary::
//!   from_slices` + `NSNumber::numberWithUnsignedInt`; the key is
//!   the real `kCVPixelBufferPixelFormatTypeKey` CFString converted
//!   through its `Display` text, never a hardcoded string). Any
//!   frame in another format is skipped, never misread.
//! * Each frame is downscaled to at most 320 px wide (nearest
//!   neighbour), converted BGRA → RGBA, and **flipped
//!   horizontally** — the raw sensor frame is not mirrored, but a
//!   mirror preview is — then JPEG-encoded with
//!   `NSBitmapImageRep` (`representationUsingType:` JPEG) and
//!   base64-encoded with `NSData::base64EncodedStringWithOptions`
//!   (no hand-rolled base64). The frontend polls `mirror_frame`
//!   (~8x/sec) and puts the `data:` URL in an `<img>`.
//! * Frame rate is throttled in the delegate to one frame per
//!   90 ms (~11 fps); `alwaysDiscardsLateVideoFrames` is on, so a
//!   slow encode can never back up the capture queue.
//! * The session runs **only** while the panel is open: the
//!   frontend calls `mirror_start` on expand and `mirror_stop` on
//!   collapse. Stopping drops the session, which releases the
//!   camera (the green camera light goes out).
//! * Permission: creating the `AVCaptureDeviceInput` is what shows
//!   the first-use camera prompt (per the AVFoundation docs — the
//!   prompt is also what `requestAccessForMediaType:` shows, but
//!   that call is async and would need a block callback; relying
//!   on the input's automatic prompt keeps this phase to one
//!   code path). `NSCameraUsageDescription` is in
//!   `src-tauri/Info.plist`. If the user denies, the session runs
//!   but delivers no frames: `mirror_frame` re-checks the
//!   authorization status on every poll, stops the dead session,
//!   and reports `permission_denied` — the frontend shows a clear
//!   "enable in System Settings → Privacy & Security → Camera"
//!   message, never a silent blank tile (risk R7).
//! * Camera selection is supported: `mirror_cameras` lists video
//!   devices via `AVCaptureDeviceDiscoverySession` (built-in wide
//!   angle + external + Continuity Camera), and `mirror_start`
//!   takes a device unique ID (`deviceWithUniqueID`); an empty ID
//!   means the first discovered device (the default camera).
//!
//! Honest limits:
//!
//! * Nothing here has run on a real MacBook yet — in particular
//!   whether the camera prompt appears, whether frames actually
//!   arrive, and whether the session stops on collapse. The
//!   Actions build only proves it compiles.
//! * A `mirror_start` that races a `mirror_stop` (panel closed
//!   while `startRunning` is still blocking) is serialised with a
//!   generation counter: if the generation changed while starting,
//!   the just-started session is stopped again immediately.
//! * The rejected alternative from the plan — overlaying a native
//!   `AVCaptureVideoPreviewLayer` NSView on the Tauri webview —
//!   stays rejected; it remains a v0.3 option only.
//!
//! The non-macOS build is an explicit stub (same pattern as
//! `media.rs` / `airdrop.rs`).

use serde::Serialize;

/// One video capture device, as the Settings camera selector shows
/// it. `id` is the device's unique ID (what `settings.mirror.
/// camera` stores); empty means "default camera".
#[derive(Debug, Clone, Serialize)]
pub struct CameraInfo {
    pub id: String,
    pub name: String,
}

/// Result of `mirror_start`: `state` is one of "running",
/// "permission_denied", "no_camera" or "error".
#[derive(Debug, Clone, Serialize)]
pub struct MirrorStatus {
    pub state: String,
    pub message: String,
}

fn status(state: &str, message: &str) -> MirrorStatus {
    MirrorStatus {
        state: state.to_string(),
        message: message.to_string(),
    }
}

/// One `mirror_frame` poll result. `status` is one of "off",
/// "starting", "waiting_permission", "permission_denied", "running",
/// "no_camera" or "error". `frame` is a `data:image/jpeg;base64,…`
/// URL, present only while frames are arriving.
#[derive(Debug, Clone, Serialize)]
pub struct MirrorFrame {
    pub status: String,
    pub message: String,
    pub frame: Option<String>,
}

fn mk_frame(status: &str, message: &str, frame: Option<String>) -> MirrorFrame {
    MirrorFrame {
        status: status.to_string(),
        message: message.to_string(),
        frame,
    }
}

#[tauri::command]
pub fn mirror_cameras() -> Vec<CameraInfo> {
    #[cfg(target_os = "macos")]
    {
        macos::list_cameras()
    }
    #[cfg(not(target_os = "macos"))]
    {
        Vec::new()
    }
}

#[tauri::command]
pub fn mirror_start(camera: String) -> MirrorStatus {
    #[cfg(target_os = "macos")]
    {
        macos::start(camera)
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = camera;
        status("error", "Camera mirror is only available on macOS")
    }
}

#[tauri::command]
pub fn mirror_stop() {
    #[cfg(target_os = "macos")]
    {
        macos::stop()
    }
    #[cfg(not(target_os = "macos"))]
    {}
}

#[tauri::command]
pub fn mirror_frame() -> MirrorFrame {
    #[cfg(target_os = "macos")]
    {
        macos::frame()
    }
    #[cfg(not(target_os = "macos"))]
    {
        mk_frame("off", "", None)
    }
}

#[cfg(target_os = "macos")]
mod macos {
    use super::*;
    use dispatch2::DispatchQueue;
    use objc2::rc::Retained;
    use objc2::runtime::ProtocolObject;
    use objc2::{define_class, extern_methods, AnyThread};
    use objc2_app_kit::{NSBitmapImageFileType, NSBitmapImageRep, NSDeviceRGBColorSpace};
    use objc2_av_foundation::{
        AVCaptureConnection, AVCaptureDevice, AVCaptureDeviceDiscoverySession,
        AVCaptureDeviceInput, AVCaptureDevicePosition, AVCaptureDeviceTypeBuiltInWideAngleCamera,
        AVCaptureDeviceTypeContinuityCamera, AVCaptureDeviceTypeExternal,
        AVCaptureOutput, AVCaptureSession, AVCaptureSessionPreset640x480,
        AVCaptureVideoDataOutput, AVCaptureVideoDataOutputSampleBufferDelegate,
        AVAuthorizationStatus, AVMediaTypeVideo,
    };
    use objc2_core_media::CMSampleBuffer;
    use objc2_core_video::{
        kCVPixelBufferPixelFormatTypeKey, kCVPixelFormatType_32BGRA, CVPixelBuffer,
        CVPixelBufferGetBaseAddress, CVPixelBufferGetBytesPerRow, CVPixelBufferGetHeight,
        CVPixelBufferGetPixelFormatType, CVPixelBufferGetWidth, CVPixelBufferLockBaseAddress,
        CVPixelBufferLockFlags, CVPixelBufferUnlockBaseAddress,
    };
    use objc2_foundation::{NSArray, NSDictionary, NSNumber, NSObject, NSObjectProtocol, NSString};
    use std::ffi::c_uchar;
    use std::sync::Mutex;
    use std::time::{Duration, Instant};

    /// One frame per 90 ms at most (~11 fps, inside the plan's
    /// 10–15 fps target).
    const FRAME_MIN_INTERVAL: Duration = Duration::from_millis(90);
    /// Frames are downscaled to at most this width before JPEG
    /// encoding, to keep each polled `data:` URL small.
    const MAX_FRAME_WIDTH: usize = 320;
    /// No frame this long after an authorised start means the
    /// capture is wedged — report "error" instead of "starting"
    /// forever.
    const NO_FRAME_TIMEOUT: Duration = Duration::from_secs(8);

    /// A running capture session, kept together because the session
    /// is only valid while its delegate and callback queue are
    /// alive (the output holds the delegate weakly).
    ///
    /// # Safety
    ///
    /// `Retained<AVCaptureSession>` is not `Send + Sync` on its
    /// own (objc2 gives no thread-safety promise for generated
    /// framework classes), so this wrapper asserts it — sound
    /// because:
    ///
    /// * `AVCaptureSession` is thread-safe per Apple's docs:
    ///   `startRunning` / `stopRunning` may be called from any
    ///   thread (they block until the session has started /
    ///   stopped), and session configuration is done before the
    ///   session starts here, never concurrently.
    /// * Every access to a `RunningSession` goes through the
    ///   `STATE` mutex, so no two threads ever touch the session,
    ///   delegate or queue at the same time.
    /// * `MirrorDelegate` has no ivars — its only method writes
    ///   through the same mutex — and a dispatch queue is
    ///   thread-safe by design.
    /// * Dropping the wrapper releases the ObjC objects, and
    ///   ObjC `release` is thread-safe.
    ///
    /// `delegate` / `queue` are never read — they exist only to
    /// keep the objects alive while the session runs.
    #[allow(dead_code)]
    struct RunningSession {
        session: Retained<AVCaptureSession>,
        delegate: Retained<MirrorDelegate>,
        queue: dispatch2::DispatchRetained<DispatchQueue>,
    }

    unsafe impl Send for RunningSession {}
    unsafe impl Sync for RunningSession {}

    struct MirrorState {
        running: Option<RunningSession>,
        camera_id: String,
        /// Bumped by every stop (and by a start that replaces a
        /// running session) so a start that finishes after a stop
        /// can tell it lost the race and tear its session down.
        generation: u64,
        started_at: Option<Instant>,
        last_processed_at: Option<Instant>,
        last_frame_at: Option<Instant>,
        latest_frame: Option<String>,
    }

    impl MirrorState {
        const fn new() -> Self {
            Self {
                running: None,
                camera_id: String::new(),
                generation: 0,
                started_at: None,
                last_processed_at: None,
                last_frame_at: None,
                latest_frame: None,
            }
        }
    }

    static STATE: Mutex<MirrorState> = Mutex::new(MirrorState::new());

    // ------------------------------------------------------------------
    // Delegate — receives frames on the serial dispatch queue.
    // ------------------------------------------------------------------

    define_class!(
        // SAFETY:
        // - The superclass NSObject has no subclassing requirements.
        // - `MirrorDelegate` does not implement `Drop`.
        #[unsafe(super(NSObject))]
        #[name = "NotchPilotMirrorDelegate"]
        struct MirrorDelegate;

        unsafe impl AVCaptureVideoDataOutputSampleBufferDelegate for MirrorDelegate {
            #[allow(non_snake_case)]
            #[unsafe(method(captureOutput:didOutputSampleBuffer:fromConnection:))]
            fn captureOutput_didOutputSampleBuffer_fromConnection(
                &self,
                _output: &AVCaptureOutput,
                sample_buffer: &CMSampleBuffer,
                _connection: &AVCaptureConnection,
            ) {
                process_frame(sample_buffer);
            }
        }

        unsafe impl NSObjectProtocol for MirrorDelegate {}
    );

    impl MirrorDelegate {
        // `+new` (NSObject's constructor), declared the way
        // objc2's own define_class tests do it. There is no
        // `msg_send![super(this), init]` here: that form needs a
        // `PartialInit` receiver, which only exists for classes
        // with ivars — and this delegate is stateless.
        extern_methods!(
            #[unsafe(method(new))]
            fn new() -> Retained<Self>;
        );
    }

    // ------------------------------------------------------------------
    // Camera list
    // ------------------------------------------------------------------

    fn video_media_type() -> Option<&'static objc2_av_foundation::AVMediaType> {
        // SAFETY: the constant is a valid AVFoundation media-type
        // string provided by the system framework.
        unsafe { AVMediaTypeVideo }
    }

    fn discovery_session() -> Retained<AVCaptureDeviceDiscoverySession> {
        // SAFETY: the device-type constants are valid AVFoundation
        // strings provided by the system framework.
        let types = NSArray::from_slice(&[
            unsafe { AVCaptureDeviceTypeBuiltInWideAngleCamera },
            unsafe { AVCaptureDeviceTypeExternal },
            unsafe { AVCaptureDeviceTypeContinuityCamera },
        ]);
        // SAFETY: discovery sessions are constructible with any
        // device-type list; `None` media type means "any media
        // type" per the docs, and Unspecified position means any
        // position.
        unsafe {
            AVCaptureDeviceDiscoverySession::discoverySessionWithDeviceTypes_mediaType_position(
                &types,
                video_media_type(),
                AVCaptureDevicePosition::Unspecified,
            )
        }
    }

    pub fn list_cameras() -> Vec<CameraInfo> {
        let session = discovery_session();
        // SAFETY: `devices` returns the devices available right now.
        let devices = unsafe { session.devices() };
        devices
            .iter()
            .map(|device| {
                // SAFETY: uniqueID / localizedName are plain getters.
                let id = unsafe { device.uniqueID() }.to_string();
                let name = unsafe { device.localizedName() }.to_string();
                CameraInfo { id, name }
            })
            .collect()
    }

    fn authorization_status() -> AVAuthorizationStatus {
        let Some(video) = video_media_type() else {
            // No video media type on this system at all — treat as
            // denied rather than crashing on an unwrap.
            return AVAuthorizationStatus::Denied;
        };
        // SAFETY: AVMediaTypeVideo is a valid media type.
        unsafe { AVCaptureDevice::authorizationStatusForMediaType(video) }
    }

    // ------------------------------------------------------------------
    // Start / stop
    // ------------------------------------------------------------------

    /// Stop and drop the running session, if any. The caller must
    /// hold the `STATE` lock. Dropping the session, delegate and
    /// queue releases the camera.
    fn stop_locked(state: &mut MirrorState) {
        if let Some(running) = &state.running {
            // SAFETY: stopRunning on a session we own.
            unsafe { running.session.stopRunning() };
        }
        state.running = None;
        state.camera_id = String::new();
        state.started_at = None;
        state.last_processed_at = None;
        state.last_frame_at = None;
        state.latest_frame = None;
        state.generation += 1;
    }

    pub fn stop() {
        let Ok(mut state) = STATE.lock() else {
            return;
        };
        stop_locked(&mut state);
    }

    pub fn start(camera: String) -> MirrorStatus {
        // Idempotent restart check + old-session teardown, under
        // the lock. The blocking `startRunning` happens below
        // without the lock (see the generation check).
        let generation = {
            let Ok(mut state) = STATE.lock() else {
                return status("error", "mirror state is unavailable");
            };
            if state.running.is_some() && state.camera_id == camera {
                return status("running", "");
            }
            if state.running.is_some() {
                stop_locked(&mut state);
            }
            // Remember the generation so a stop that lands while
            // startRunning blocks can be detected afterwards.
            state.generation
        };

        // Permission gate: a denial is final until the user changes
        // it in System Settings, so there is nothing to build.
        // NotDetermined is *not* gated — creating the device input
        // below is what shows the first-use prompt (per the
        // AVFoundation docs on requestAccessForMediaType:).
        let auth = authorization_status();
        if auth == AVAuthorizationStatus::Denied || auth == AVAuthorizationStatus::Restricted {
            return status(
                "permission_denied",
                "camera access is denied for NotchPilot",
            );
        }

        // Resolve the device: a saved unique ID, or the first
        // discovered video device (the default camera).
        let device = if camera.is_empty() {
            let session = discovery_session();
            // SAFETY: `devices` returns the devices available now.
            let devices = unsafe { session.devices() };
            devices
                .iter()
                .next()
                .map(|d| unsafe { d.uniqueID() }.to_string())
                .and_then(|id| unsafe {
                    AVCaptureDevice::deviceWithUniqueID(&NSString::from_str(&id))
                })
        } else {
            unsafe { AVCaptureDevice::deviceWithUniqueID(&NSString::from_str(&camera)) }
        };
        let Some(device) = device else {
            return status("no_camera", "no camera was found on this Mac");
        };

        // Creating the input opens the device for capture — and
        // shows the first-use camera prompt when the status is
        // NotDetermined. It fails when the camera is in use by
        // another app or has gone away.
        // SAFETY: the device came from the discovery session / a
        // unique-ID lookup, so it is a valid capture device.
        let input = match unsafe { AVCaptureDeviceInput::deviceInputWithDevice_error(&device) } {
            Ok(input) => input,
            Err(e) => {
                let desc = e.localizedDescription().to_string();
                return status("error", &format!("could not open the camera: {desc}"));
            }
        };

        // Session: 640x480 is plenty for a notch tile and keeps
        // the JPEG encode cheap.
        // SAFETY: `new` on AVCaptureSession is the documented
        // construction path.
        let session = unsafe { AVCaptureSession::new() };
        // SAFETY: the preset constant is a valid AVFoundation
        // preset string.
        let preset = unsafe { AVCaptureSessionPreset640x480 };
        if unsafe { session.canSetSessionPreset(preset) } {
            unsafe { session.setSessionPreset(preset) };
        }
        // SAFETY: canAdd* is checked before each add.
        unsafe {
            if !session.canAddInput(&input) {
                return status("error", "the camera cannot be added to a capture session");
            }
            session.addInput(&input);
        }

        // Output: BGRA frames on a serial queue, late frames
        // discarded rather than queued.
        // SAFETY: `new` on AVCaptureVideoDataOutput is the
        // documented construction path.
        let output = unsafe { AVCaptureVideoDataOutput::new() };
        let video_settings = video_settings_bgra();
        unsafe { output.setVideoSettings(video_settings.as_deref()) };
        unsafe { output.setAlwaysDiscardsLateVideoFrames(true) };
        let delegate = MirrorDelegate::new();
        let queue = DispatchQueue::new("com.yichia.notch-pilot.mirror", None);
        // SAFETY: the delegate conforms to the protocol (its class
        // implements it) and the queue is a valid serial queue.
        unsafe {
            output.setSampleBufferDelegate_queue(
                Some(ProtocolObject::from_ref(&*delegate)),
                Some(&queue),
            );
        }
        // SAFETY: canAddOutput is checked before adding.
        unsafe {
            if !session.canAddOutput(&output) {
                return status("error", "the camera output cannot be added to a capture session");
            }
            session.addOutput(&output);
        }

        // This blocks until the session has started (or failed) —
        // deliberately done without holding the STATE lock, so a
        // concurrent `mirror_stop` / `mirror_frame` is not stuck
        // behind it. The generation check below detects that race.
        // SAFETY: startRunning on a session we own.
        unsafe { session.startRunning() };

        let Ok(mut state) = STATE.lock() else {
            // The state lock is poisoned; stop the session we just
            // started rather than leaking a running camera.
            unsafe { session.stopRunning() };
            return status("error", "mirror state is unavailable");
        };
        if state.generation != generation {
            // A stop landed while startRunning blocked — tear the
            // just-started session down instead of leaking it.
            unsafe { session.stopRunning() };
            return status("off", "");
        }
        state.running = Some(RunningSession {
            session,
            delegate,
            queue,
        });
        state.camera_id = camera;
        state.started_at = Some(Instant::now());
        status("running", "")
    }

    /// `{ kCVPixelBufferPixelFormatTypeKey: 32BGRA }` — the output
    /// converts the camera's native format to BGRA for us. The key
    /// is the real `kCVPixelBufferPixelFormatTypeKey` CFString,
    /// converted through its text (toll-free bridging would also
    /// work, but a text round-trip needs no unsafe cast).
    fn video_settings_bgra() -> Option<Retained<NSDictionary<NSString, objc2::runtime::AnyObject>>>
    {
        let key_string = unsafe { kCVPixelBufferPixelFormatTypeKey }.to_string();
        let key = NSString::from_str(&key_string);
        let value = NSNumber::numberWithUnsignedInt(kCVPixelFormatType_32BGRA);
        // SAFETY: NSNumber is an AnyObject; `cast_unchecked` is the
        // documented way to upcast a retained object to AnyObject
        // (same pattern as airdrop.rs's NSArray cast).
        let value_any: Retained<objc2::runtime::AnyObject> =
            unsafe { Retained::cast_unchecked(value) };
        Some(NSDictionary::from_slices(&[&*key], &[&*value_any]))
    }

    // ------------------------------------------------------------------
    // Frame poll
    // ------------------------------------------------------------------

    pub fn frame() -> MirrorFrame {
        let Ok(mut state) = STATE.lock() else {
            return mk_frame("error", "mirror state is unavailable", None);
        };
        if state.running.is_none() {
            return mk_frame("off", "", None);
        }
        if let Some(latest) = &state.latest_frame {
            // A frame has arrived — the session is alive.
            return mk_frame("running", "", Some(latest.clone()));
        }
        // No frame yet. A denial that landed after start (the user
        // answered the prompt with "Don't Allow") leaves a running
        // session delivering nothing — stop it and say so.
        let auth = authorization_status();
        if auth == AVAuthorizationStatus::Denied || auth == AVAuthorizationStatus::Restricted {
            stop_locked(&mut state);
            return mk_frame(
                "permission_denied",
                "camera access is denied for NotchPilot",
                None,
            );
        }
        if auth == AVAuthorizationStatus::NotDetermined {
            return mk_frame("waiting_permission", "", None);
        }
        match state.started_at {
            Some(started) if started.elapsed() > NO_FRAME_TIMEOUT => {
                stop_locked(&mut state);
                mk_frame("error", "the camera produced no frames", None)
            }
            _ => mk_frame("starting", "", None),
        }
    }

    // ------------------------------------------------------------------
    // Frame processing (delegate thread)
    // ------------------------------------------------------------------

    /// Throttle, then encode one sample buffer to a JPEG `data:`
    /// URL and store it as the latest frame.
    fn process_frame(sample_buffer: &CMSampleBuffer) {
        let now = Instant::now();
        {
            let Ok(mut state) = STATE.lock() else {
                return;
            };
            if let Some(last) = state.last_processed_at {
                if now.duration_since(last) < FRAME_MIN_INTERVAL {
                    return;
                }
            }
            state.last_processed_at = Some(now);
        }
        let Some(data_url) = encode_frame(sample_buffer) else {
            return;
        };
        if let Ok(mut state) = STATE.lock() {
            state.latest_frame = Some(data_url);
            state.last_frame_at = Some(Instant::now());
        }
    }

    /// `CMSampleBuffer` → downscaled RGBA → JPEG `data:` URL.
    /// Returns `None` for any frame that cannot be used (no image
    /// buffer, wrong pixel format, lock failure, encode failure) —
    /// a skipped frame is invisible at 11 fps.
    fn encode_frame(sample_buffer: &CMSampleBuffer) -> Option<String> {
        // SAFETY: image_buffer is a plain getter on the sample
        // buffer we were handed by the capture output.
        let image_buffer = unsafe { sample_buffer.image_buffer() }?;
        // CVPixelBuffer is a type alias for CVImageBuffer, so no
        // cast is needed.
        let pixel_buffer: &CVPixelBuffer = &image_buffer;
        if CVPixelBufferGetPixelFormatType(pixel_buffer) != kCVPixelFormatType_32BGRA {
            return None;
        }
        let width = CVPixelBufferGetWidth(pixel_buffer);
        let height = CVPixelBufferGetHeight(pixel_buffer);
        if width == 0 || height == 0 {
            return None;
        }
        if unsafe { CVPixelBufferLockBaseAddress(pixel_buffer, CVPixelBufferLockFlags::ReadOnly) }
            != 0
        {
            return None;
        }
        let result = encode_locked(pixel_buffer, width, height);
        unsafe { CVPixelBufferUnlockBaseAddress(pixel_buffer, CVPixelBufferLockFlags::ReadOnly) };
        result
    }

    /// Read the locked pixel buffer, downscale, and JPEG-encode.
    /// The buffer stays locked for the whole call.
    fn encode_locked(pixel_buffer: &CVPixelBuffer, width: usize, height: usize) -> Option<String> {
        let base = CVPixelBufferGetBaseAddress(pixel_buffer) as *const u8;
        if base.is_null() {
            return None;
        }
        let row_bytes = CVPixelBufferGetBytesPerRow(pixel_buffer);
        let target_w = width.min(MAX_FRAME_WIDTH);
        let target_h = (height * target_w / width).max(1);

        // Downscale (nearest neighbour) + BGRA → RGBA + horizontal
        // flip in one pass. The flip is what makes the preview read
        // as a mirror: the sensor frame is not mirrored, the
        // FaceTime preview is.
        let mut rgba = vec![0u8; target_w * target_h * 4];
        for ty in 0..target_h {
            let sy = ty * height / target_h;
            for tx in 0..target_w {
                let sx = width - 1 - (tx * width / target_w);
                // SAFETY: sx < width, sy < height, and each row has
                // at least width * 4 bytes (row_bytes >= width * 4
                // for a 32-bit format).
                let src = unsafe { base.add(sy * row_bytes + sx * 4) };
                let dst = (ty * target_w + tx) * 4;
                unsafe {
                    rgba[dst] = *src.add(2); // R
                    rgba[dst + 1] = *src.add(1); // G
                    rgba[dst + 2] = *src; // B
                    rgba[dst + 3] = 255; // opaque
                }
            }
        }

        // JPEG via NSBitmapImageRep. The rep references our `rgba`
        // buffer (it does not copy), so `rgba` must outlive the
        // representation call — it does: it lives until this
        // function returns.
        let mut planes = [rgba.as_mut_ptr()];
        // SAFETY: `planes` points at a live `rgba` buffer of
        // target_w * target_h * 4 bytes; the geometry and format
        // arguments describe that buffer exactly (8-bit RGBA,
        // non-planar, 32 bits per pixel).
        //
        // `init…` methods take `this: Allocated<Self>` as a plain
        // first parameter (not `self`), so they are called as
        // associated functions — same shape as
        // `NSDictionary::from_slices`'s internal
        // `Self::initWithObjects_forKeys_count(Self::alloc(), …)`.
        let rep = unsafe {
            NSBitmapImageRep::initWithBitmapDataPlanes_pixelsWide_pixelsHigh_bitsPerSample_samplesPerPixel_hasAlpha_isPlanar_colorSpaceName_bytesPerRow_bitsPerPixel(
                NSBitmapImageRep::alloc(),
                planes.as_mut_ptr() as *mut *mut c_uchar,
                target_w as isize,
                target_h as isize,
                8,
                4,
                true,
                false,
                // SAFETY: the constant is a valid AppKit colour
                // space name provided by the system framework.
                NSDeviceRGBColorSpace,
                (target_w * 4) as isize,
                32,
            )
        }?;
        // SAFETY: JPEG encoding with default (empty) properties.
        let empty_props =
            NSDictionary::<NSString, objc2::runtime::AnyObject>::new();
        let jpeg = unsafe {
            rep.representationUsingType_properties(NSBitmapImageFileType::JPEG, &empty_props)
        }?;
        // `base64EncodedStringWithOptions` is a safe method; the
        // empty options mean one continuous base64 string.
        let b64 = jpeg.base64EncodedStringWithOptions(
            objc2_foundation::NSDataBase64EncodingOptions::empty(),
        );
        Some(format!("data:image/jpeg;base64,{b64}"))
    }
}
