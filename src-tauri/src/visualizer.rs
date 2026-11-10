//! Audio visualizer — real system-audio spectrum via ScreenCaptureKit (v0.2 Phase 1b).
//!
//! macOS has no public "now-playing audio level" API, so the real
//! spectrum comes from a ScreenCaptureKit audio-only stream
//! (`SCStream`, macOS 13+): the stream taps the system audio output
//! at 48 kHz stereo float PCM, this module downmixes it to mono and
//! runs a Goertzel filter bank over 8 log-spaced bands, and the
//! frontend polls the levels with `visualizer_levels` (~7x/sec).
//!
//! Implementation notes, all verified against the actual
//! `screencapturekit` 11.1.2 / `apple-cf` 0.11.7 crate source
//! (not docs memory):
//! - `SCShareableContent::get()` errors when Screen Recording
//!   permission is not granted; `start_capture()` is what triggers
//!   the system permission prompt on first run.
//! - Only an `Audio` output handler is registered, so no video
//!   frames are delivered; the video size is kept at 16x16 to
//!   minimise the stream's cost. `SCStream` is `Send + Sync`, so it
//!   can live in a `static Mutex`.
//! - Audio buffers are float32 PCM (`AudioBuffer::data()` bytes,
//!   little-endian `f32`; one buffer, 2 interleaved channels, in
//!   practice). The Goertzel gain below is uncalibrated — no device
//!   test has been run yet; the user should report if the bars look
//!   dead or pinned at full height.
//! - The stream runs only while music is playing (the frontend
//!   starts/stops it from the play state). If the stream cannot
//!   start — permission denied, macOS < 13, no display —
//!   `visualizer_start` returns `Err` and the frontend falls back
//!   to the honestly-labelled animated bars (see `src/main.js`).
//!   Nothing here ever fakes a real spectrum.
//!
//! The non-macOS build is an explicit stub (same pattern as
//! `media.rs` / `audio.rs`).

use std::collections::VecDeque;
use std::sync::Mutex;
use std::time::Instant;

/// Number of spectrum bands (and frontend bars).
pub const NUM_BANDS: usize = 8;

/// ScreenCaptureKit audio sample rate (Hz). The stream is
/// configured for 48 kHz, which is what SCK delivers.
const SAMPLE_RATE: f32 = 48_000.0;

/// Goertzel window length in mono samples (~21 ms at 48 kHz).
const WINDOW: usize = 1024;

/// Goertzel centre frequencies (Hz), roughly log-spaced from bass
/// to brilliance.
const BAND_FREQS: [f32; NUM_BANDS] = [
    60.0, 160.0, 400.0, 1_000.0, 2_500.0, 6_000.0, 12_000.0, 16_000.0,
];

/// Amplitude-to-level gain. Goertzel magnitude for a component of
/// amplitude `A` is ~`A / 4` (Hann window); real music components
/// sit around 0.01–0.08, so x12 maps them to roughly 0.15–0.95.
/// Uncalibrated — see the module docs.
const LEVEL_GAIN: f32 = 12.0;

struct VizState {
    /// Latest mono samples (newest at the back), capped at 2 windows.
    samples: VecDeque<f32>,
    /// Smoothed band levels, 0..1.
    levels: [f32; NUM_BANDS],
    /// When the levels were last updated from real audio.
    updated_at: Option<Instant>,
}

impl VizState {
    const fn new() -> Self {
        Self {
            samples: VecDeque::new(),
            levels: [0.0; NUM_BANDS],
            updated_at: None,
        }
    }
}

static STATE: Mutex<VizState> = Mutex::new(VizState::new());

/// Goertzel magnitude at each band frequency over `samples`
/// (Hann-windowed), mapped to 0..1 levels.
fn compute_bands(samples: &[f32]) -> [f32; NUM_BANDS] {
    let n = samples.len() as f32;
    let mut out = [0.0f32; NUM_BANDS];
    for (i, &freq) in BAND_FREQS.iter().enumerate() {
        let w = 2.0 * std::f32::consts::PI * freq / SAMPLE_RATE;
        let coeff = 2.0 * w.cos();
        let mut s1 = 0.0f32;
        let mut s2 = 0.0f32;
        for (k, &x) in samples.iter().enumerate() {
            // Hann window reduces spectral leakage between bands.
            let hann = 0.5
                * (1.0 - (2.0 * std::f32::consts::PI * k as f32 / n).cos());
            let v = x * hann;
            let s0 = v + coeff * s1 - s2;
            s2 = s1;
            s1 = s0;
        }
        let power = (s1 * s1 + s2 * s2 - coeff * s1 * s2).max(0.0);
        let amp = (power / (n * n)).sqrt();
        out[i] = (amp * LEVEL_GAIN).min(1.0);
    }
    out
}

/// Feed one decoded mono block into the ring buffer and, once a
/// full window is available, refresh the band levels (fast attack,
/// slow release so the bars fall smoothly).
fn push_mono(mono: &[f32]) {
    let Ok(mut state) = STATE.lock() else {
        return;
    };
    for &s in mono {
        state.samples.push_back(s);
    }
    let max = WINDOW * 2;
    while state.samples.len() > max {
        state.samples.pop_front();
    }
    if state.samples.len() >= WINDOW {
        let skip = state.samples.len() - WINDOW;
        let window: Vec<f32> = state.samples.iter().skip(skip).copied().collect();
        let fresh = compute_bands(&window);
        for i in 0..NUM_BANDS {
            let old = state.levels[i];
            let new = fresh[i];
            state.levels[i] = if new > old {
                old * 0.3 + new * 0.7
            } else {
                old * 0.85 + new * 0.15
            };
        }
        state.updated_at = Some(Instant::now());
    }
}

#[tauri::command]
pub fn visualizer_start() -> Result<String, String> {
    #[cfg(target_os = "macos")]
    {
        macos::start()
    }
    #[cfg(not(target_os = "macos"))]
    {
        Err("Real audio visualizer is only supported on macOS".to_string())
    }
}

#[tauri::command]
pub fn visualizer_stop() -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        macos::stop();
    }
    Ok(())
}

#[tauri::command]
pub fn visualizer_levels() -> Vec<f32> {
    #[cfg(target_os = "macos")]
    {
        macos::levels()
    }
    #[cfg(not(target_os = "macos"))]
    {
        vec![0.0; NUM_BANDS]
    }
}

#[cfg(target_os = "macos")]
mod macos {
    use super::{push_mono, VizState, NUM_BANDS, STATE};
    use screencapturekit::prelude::*;
    use std::sync::Mutex;

    /// The running ScreenCaptureKit stream. `SCStream` is `Send +
    /// Sync` (per the crate source), so it can live in a static.
    static STREAM: Mutex<Option<SCStream>> = Mutex::new(None);

    /// Start the audio-only capture stream. Returns `"real"` when
    /// the stream is running. Any failure (no permission, macOS <
    /// 13, no display) is an `Err` — the caller falls back to the
    /// animated bars and never pretends this is real audio.
    pub fn start() -> Result<String, String> {
        {
            let guard = STREAM
                .lock()
                .map_err(|_| "visualizer lock poisoned".to_string())?;
            if guard.is_some() {
                return Ok("real".to_string());
            }
        }
        let content = SCShareableContent::get()
            .map_err(|e| format!("ScreenCaptureKit: {e}"))?;
        let display = content
            .displays()
            .into_iter()
            .next()
            .ok_or_else(|| "ScreenCaptureKit: no display found".to_string())?;
        let filter = SCContentFilter::create()
            .with_display(&display)
            .with_excluding_windows(&[])
            .build()
            .map_err(|e| format!("ScreenCaptureKit filter: {e}"))?;
        // Audio-only in effect: only an Audio output handler is
        // registered below, so no video frames are delivered. The
        // video size is kept at 16x16 to minimise the stream cost.
        // `excludes_current_process_audio` keeps NotchPilot's own
        // sounds (it plays none today) out of the tap.
        let config = SCStreamConfiguration::new()
            .with_width(16)
            .with_height(16)
            .with_captures_audio(true)
            .with_sample_rate(48_000)
            .with_channel_count(2)
            .with_excludes_current_process_audio(true);
        let mut stream = SCStream::new(&filter, &config)
            .map_err(|e| format!("ScreenCaptureKit stream: {e}"))?;
        // The closure captures nothing (it writes to the `STATE`
        // static), so it is `Send + Sync + 'static` as the trait
        // requires. It must never panic: a panic on a dispatch
        // queue would abort the process.
        stream
            .add_output_handler(
                |sample: CMSampleBuffer, _of_type: SCStreamOutputType| {
                    on_audio_sample(sample);
                },
                SCStreamOutputType::Audio,
            )
            .map_err(|e| format!("ScreenCaptureKit output: {e}"))?;
        // This is what triggers the system Screen Recording
        // permission prompt on first run.
        stream
            .start_capture()
            .map_err(|e| format!("ScreenCaptureKit start (is Screen Recording allowed?): {e}"))?;
        let mut guard = STREAM
            .lock()
            .map_err(|_| "visualizer lock poisoned".to_string())?;
        *guard = Some(stream);
        Ok("real".to_string())
    }

    pub fn stop() {
        let stream = {
            let Ok(mut guard) = STREAM.lock() else {
                return;
            };
            guard.take()
        };
        if let Some(s) = stream {
            let _ = s.stop_capture();
        }
        if let Ok(mut state) = STATE.lock() {
            *state = VizState::new();
        }
    }

    /// Current band levels, decayed towards zero when no audio has
    /// arrived recently (halved every ~180 ms of silence) so the
    /// bars fall instead of freezing mid-song.
    pub fn levels() -> Vec<f32> {
        let Ok(mut state) = STATE.lock() else {
            return vec![0.0; NUM_BANDS];
        };
        let mut out = state.levels;
        if let Some(t) = state.updated_at {
            let elapsed = t.elapsed().as_millis() as f32;
            if elapsed > 120.0 {
                let factor = 0.5f32.powf(elapsed / 180.0);
                for v in out.iter_mut() {
                    *v *= factor;
                }
                state.levels = out;
            }
        }
        out.to_vec()
    }

    /// One audio `CMSampleBuffer` from the stream: mix its buffers
    /// down to mono float samples and feed the ring buffer.
    /// SCK delivers 32-bit float PCM; in practice one buffer with
    /// 2 interleaved channels.
    fn on_audio_sample(sample: CMSampleBuffer) {
        let Ok(list) = sample.audio_buffer_list() else {
            return;
        };
        let mut mono: Vec<f32> = Vec::new();
        let mut n_buffers = 0usize;
        for buf in list.iter() {
            let bytes = buf.data();
            if bytes.len() < 4 {
                continue;
            }
            let mut ch: Vec<f32> = Vec::with_capacity(bytes.len() / 4);
            for chunk in bytes.chunks_exact(4) {
                ch.push(f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]));
            }
            if buf.number_channels == 2 {
                // Interleaved stereo -> mono by averaging pairs.
                let mut m = Vec::with_capacity(ch.len() / 2);
                for pair in ch.chunks_exact(2) {
                    m.push((pair[0] + pair[1]) * 0.5);
                }
                ch = m;
            }
            if ch.is_empty() {
                continue;
            }
            if mono.len() < ch.len() {
                mono.resize(ch.len(), 0.0);
            }
            for (i, s) in ch.iter().enumerate() {
                mono[i] += *s;
            }
            n_buffers += 1;
        }
        if n_buffers == 0 || mono.is_empty() {
            return;
        }
        let inv = 1.0 / n_buffers as f32;
        for s in mono.iter_mut() {
            *s *= inv;
        }
        push_mono(&mono);
    }
}
