// notch-pilot — vanilla frontend, no build step.
// Tauri 2 injects window.__TAURI__ because tauri.conf.json sets
// app.withGlobalTauri = true. Every invoke is guarded so the page
// still renders (with fallback text) if Tauri is unavailable.
//
// All visible UI text comes from ./i18n.js via t(). Do not put
// hardcoded UI strings in this file.

import {
  DATE_LOCALES,
  applyI18n,
  getLanguage,
  initLanguage,
  setLanguage,
  t,
} from "./i18n.js";

const $ = (id) => document.getElementById(id);

const notchEl = $("notch");
const pillBatteryEl = $("pill-battery");
const clockEl = $("clock");
const dateEl = $("date");
const batteryEl = $("battery");
const npTitleEl = $("np-title");
const npArtistEl = $("np-artist");
const playBtn = $("btn-play");
const npArtEl = $("np-art");
const npProgressRow = $("np-progress-row");
const npSeekEl = $("np-seek");
const npElapsedEl = $("np-elapsed");
const npTotalEl = $("np-total");
const npLyricEl = $("np-lyric");
const npOutputEl = $("np-output");
const shelfListEl = $("shelf-list");
const npVizEl = $("np-viz");
const npVizLabelEl = $("np-viz-label");
const pillVizEl = $("pill-viz");

let expanded = false;
let expandTimer = null;
let tauri = window.__TAURI__ || null;

// Last known dynamic state, kept so a language switch can re-render
// everything immediately without waiting for the next poll.
let lastBattery = null;
let lastNowPlaying = null;
let lastSettings = null;
let shelfFiles = [];

// Playback position tracking: the Rust poll returns the position at
// poll time; between polls the frontend interpolates while playing
// so the progress bar and lyrics move smoothly.
let lastPosition = 0;
let lastDuration = 0;
let lastPositionAt = 0;
let seeking = false;

// Synced lyrics (LRCLIB, frontend fetch — see fetchLyrics).
let currentTrackKey = "";
let lyricLines = [];

// Audio output devices from get_audio_outputs.
let audioOutputs = [];
let lastAppliedOutputUid = "";
let savedOutputApplied = false;

// --- Audio visualizer (Phase 1b) ------------------------------------
// Modes: "off" (not playing / disabled), "real" (ScreenCaptureKit
// spectrum from Rust), "animated" (honest fallback: bars animate
// while music plays, labelled 動畫視覺化 in the UI — never presented
// as real audio). The stream runs only while music is playing.
const VIZ_BANDS = 8;
const VIZ_POLL_MS = 140;
let vizMode = "off";
let vizTimer = null;
let vizStarting = false;
let vizLevels = new Array(VIZ_BANDS).fill(0);
let animLevels = new Array(VIZ_BANDS).fill(0);
let npVizBars = [];
let pillVizBars = [];

function buildVizBars() {
  if (npVizEl) {
    npVizEl.innerHTML = "";
    npVizBars = [];
    for (let i = 0; i < VIZ_BANDS; i++) {
      const s = document.createElement("span");
      npVizEl.appendChild(s);
      npVizBars.push(s);
    }
  }
  if (pillVizEl) {
    pillVizEl.innerHTML = "";
    pillVizBars = [];
    for (let i = 0; i < 4; i++) {
      const s = document.createElement("span");
      pillVizEl.appendChild(s);
      pillVizBars.push(s);
    }
  }
}

// Bar heights for the current style. "bars": straight levels.
// "wave": the same levels under a sine envelope (taller in the
// middle). "circular": bars radiate around the art — their rotation
// is set in applyVizStyle; height grows back towards the art.
function vizBarHeight(level, index, style) {
  if (style === "wave") {
    const env = 0.55 + 0.45 * Math.sin((Math.PI * (index + 0.5)) / VIZ_BANDS);
    return 2 + level * 46 * env;
  }
  if (style === "circular") {
    return 3 + level * 11;
  }
  return 2 + level * 54;
}

function renderVizBars() {
  const style =
    lastSettings && lastSettings.media && lastSettings.media.visualizer_style
      ? lastSettings.media.visualizer_style
      : "bars";
  const show = vizMode !== "off";
  if (npVizEl) {
    npVizEl.hidden = !show;
    if (show) {
      npVizBars.forEach((bar, i) => {
        bar.style.height = `${vizBarHeight(vizLevels[i] || 0, i, style).toFixed(1)}px`;
      });
    }
  }
  if (npVizLabelEl) {
    // Honest label: shown ONLY while the animated fallback runs.
    npVizLabelEl.hidden = vizMode !== "animated";
  }
  if (pillVizEl) {
    pillVizEl.hidden = !show;
    if (show) {
      pillVizBars.forEach((bar, i) => {
        // Pill uses 4 bars: the louder of each band pair.
        const lvl = Math.max(vizLevels[i * 2] || 0, vizLevels[i * 2 + 1] || 0);
        bar.style.height = `${(2 + lvl * 12).toFixed(1)}px`;
      });
    }
  }
}

// Animated fallback levels: a smooth random walk (slow sine drift
// + noise), so the bars move while music plays without claiming to
// be the real spectrum.
function nextAnimatedLevels() {
  const time = Date.now() / 1000;
  for (let i = 0; i < VIZ_BANDS; i++) {
    const target =
      0.35 +
      0.3 * Math.sin(time * 1.7 + i * 0.9) +
      (Math.random() - 0.5) * 0.25;
    animLevels[i] += (target - animLevels[i]) * 0.35;
    animLevels[i] = Math.min(1, Math.max(0.06, animLevels[i]));
  }
  return animLevels;
}

async function vizTickReal() {
  try {
    const levels = await invoke("visualizer_levels");
    if (Array.isArray(levels) && levels.length >= VIZ_BANDS) {
      vizLevels = levels.slice(0, VIZ_BANDS);
    }
  } catch (e) {
    // A failed poll must not kill the timer; the next tick retries.
    console.warn("visualizer_levels failed:", e);
  }
  renderVizBars();
}

function vizTickAnimated() {
  vizLevels = nextAnimatedLevels().slice();
  renderVizBars();
}

function stopVisualizer() {
  if (vizTimer !== null) {
    clearInterval(vizTimer);
    vizTimer = null;
  }
  if (vizMode === "real") {
    invoke("visualizer_stop").catch((e) =>
      console.warn("visualizer_stop failed:", e),
    );
  }
  vizMode = "off";
  vizLevels = new Array(VIZ_BANDS).fill(0);
  animLevels = new Array(VIZ_BANDS).fill(0);
  renderVizBars();
}

// Start the visualizer for the current track: try the real
// ScreenCaptureKit stream first; on any failure (no permission,
// macOS < 13, …) fall back to the labelled animation.
async function startVisualizer() {
  if (vizMode !== "off" || vizStarting) return;
  vizStarting = true;
  try {
    const mode = await invoke("visualizer_start");
    if (mode === "real") {
      vizMode = "real";
      vizTimer = setInterval(vizTickReal, VIZ_POLL_MS);
      vizTickReal();
      return;
    }
  } catch (e) {
    console.warn("real visualizer unavailable, using animated fallback:", e);
  } finally {
    vizStarting = false;
  }
  vizMode = "animated";
  vizTimer = setInterval(vizTickAnimated, 120);
  vizTickAnimated();
}

// Apply the visualizer style class (bars / wave / circular) to the
// media-row container. For circular, each bar also gets its radial
// rotation (translateY(-34px) = container half-size, see styles.css).
function applyVizStyle() {
  if (!npVizEl) return;
  const style =
    lastSettings && lastSettings.media && lastSettings.media.visualizer_style
      ? lastSettings.media.visualizer_style
      : "bars";
  npVizEl.classList.toggle("viz-wave", style === "wave");
  npVizEl.classList.toggle("viz-circular", style === "circular");
  npVizBars.forEach((bar, i) => {
    bar.style.transform =
      style === "circular" ? `rotate(${i * 45}deg) translateY(-34px)` : "";
  });
}

// Start/stop the visualizer from the current play state + setting.
// Called after every Now Playing refresh and every settings change.
function syncVisualizer() {
  const enabled =
    lastSettings &&
    lastSettings.media &&
    lastSettings.media.visualizer &&
    lastNowPlaying &&
    lastNowPlaying.title &&
    lastNowPlaying.playing;
  if (!enabled) {
    if (vizMode !== "off") stopVisualizer();
    return;
  }
  if (vizMode === "off") {
    applyVizStyle();
    startVisualizer();
  } else {
    applyVizStyle();
    renderVizBars();
  }
}

function invoke(cmd, args) {
  if (!tauri || !tauri.core || typeof tauri.core.invoke !== "function") {
    return Promise.reject(new Error("Tauri is not available"));
  }
  return tauri.core.invoke(cmd, args);
}

function batteryLabel(info) {
  if (!info || info.percent < 0) return t("batteryUnknown");
  return `🔋 ${info.percent}%${info.charging ? ` ${t("charging")}` : ""}`;
}

function renderBattery() {
  const label = batteryLabel(lastBattery);
  pillBatteryEl.textContent = label;
  if (batteryEl) batteryEl.textContent = label;
}

function formatTime(sec) {
  if (!Number.isFinite(sec) || sec < 0) sec = 0;
  const m = Math.floor(sec / 60);
  const s = Math.floor(sec % 60);
  return `${m}:${String(s).padStart(2, "0")}`;
}

// Position right now: interpolate from the last poll while playing,
// frozen at the last known position while paused.
function currentPosition() {
  if (!lastNowPlaying || !lastNowPlaying.playing || lastDuration <= 0) {
    return lastPosition;
  }
  return Math.min(
    lastDuration,
    lastPosition + (Date.now() - lastPositionAt) / 1000,
  );
}

function renderNowPlaying() {
  const np = lastNowPlaying;
  if (!np || !np.title) {
    npTitleEl.textContent = t("notPlaying");
    npArtistEl.textContent = t("notPlayingHint");
    playBtn.textContent = "▶";
    playBtn.setAttribute("title", t("play"));
    npArtEl.hidden = true;
    npArtEl.removeAttribute("src");
    npProgressRow.hidden = true;
    npLyricEl.hidden = true;
    npLyricEl.textContent = "";
    currentTrackKey = "";
    lyricLines = [];
    syncVisualizer();
    return;
  }
  npTitleEl.textContent = np.title;
  npArtistEl.textContent = np.artist
    ? `${np.artist}${np.album ? ` · ${np.album}` : ""}`
    : np.album || np.app || "";
  playBtn.textContent = np.playing ? "⏸" : "▶";
  playBtn.setAttribute("title", np.playing ? t("pause") : t("play"));
  // Album art: Music.app returns a data: URL, Spotify an artwork URL.
  const art = np.artwork || np.artwork_url || "";
  if (art) {
    if (npArtEl.getAttribute("src") !== art) npArtEl.src = art;
    npArtEl.hidden = false;
  } else {
    npArtEl.hidden = true;
    npArtEl.removeAttribute("src");
  }
  // Progress bar (only when the track reports a duration).
  lastDuration = np.duration || 0;
  if (lastDuration > 0) {
    npProgressRow.hidden = false;
    npSeekEl.max = String(lastDuration);
    npTotalEl.textContent = formatTime(lastDuration);
  } else {
    npProgressRow.hidden = true;
  }
  renderProgress();
  // Lyrics: refetch when the track changes.
  const key = `${np.app}|${np.title}|${np.artist}|${np.album}`;
  if (key !== currentTrackKey) {
    currentTrackKey = key;
    lyricLines = [];
    npLyricEl.hidden = true;
    npLyricEl.textContent = "";
    fetchLyrics(np);
    // A new track is a cheap moment to retry the real visualizer:
    // if the user granted Screen Recording since the animated
    // fallback started, the next track picks up the real spectrum.
    if (vizMode === "animated") {
      stopVisualizer();
    }
  } else {
    renderLyric();
  }
  syncVisualizer();
}

function renderProgress() {
  if (npProgressRow.hidden) return;
  const pos = currentPosition();
  if (!seeking) {
    npSeekEl.value = String(pos);
  }
  npElapsedEl.textContent = formatTime(pos);
}

function renderLyric() {
  if (
    lyricLines.length === 0 ||
    !lastSettings ||
    !lastSettings.media ||
    !lastSettings.media.lyrics
  ) {
    npLyricEl.hidden = true;
    return;
  }
  const pos = currentPosition();
  let line = "";
  for (const l of lyricLines) {
    if (l.time <= pos) line = l.text;
    else break;
  }
  if (!line) {
    npLyricEl.hidden = true;
    return;
  }
  npLyricEl.textContent = line;
  npLyricEl.hidden = false;
}

// Synced lyrics from LRCLIB (free, no key), fetched from the
// frontend — the webview can do HTTPS, so no new Rust dependency.
// A 404 or any failure hides the line silently: lyrics are a
// stretch feature and must never break the media row.
async function fetchLyrics(np) {
  if (!lastSettings || !lastSettings.media || !lastSettings.media.lyrics) {
    return;
  }
  const key = `${np.app}|${np.title}|${np.artist}|${np.album}`;
  try {
    const params = new URLSearchParams({
      artist_name: np.artist || "",
      track_name: np.title || "",
      album_name: np.album || "",
    });
    const res = await fetch(`https://lrclib.net/api/get?${params}`);
    if (!res.ok) return;
    const data = await res.json();
    if (key !== currentTrackKey) return; // track changed meanwhile
    lyricLines = parseLrc(data.syncedLyrics || "");
    renderLyric();
  } catch (e) {
    lyricLines = [];
    console.warn("lyrics fetch failed:", e);
  }
}

function parseLrc(lrc) {
  const lines = [];
  if (!lrc) return lines;
  for (const raw of String(lrc).split("\n")) {
    const m = raw.match(/^\[(\d+):(\d+(?:\.\d+)?)\]\s?(.*)$/);
    if (!m) continue;
    const text = (m[3] || "").trim();
    if (!text) continue;
    lines.push({
      time: parseInt(m[1], 10) * 60 + parseFloat(m[2]),
      text,
    });
  }
  lines.sort((a, b) => a.time - b.time);
  return lines;
}

// 1 s tick: keeps the progress bar and lyric line moving between
// the 5 s Rust polls.
function tickPosition() {
  if (!lastNowPlaying || !lastNowPlaying.title) return;
  renderProgress();
  renderLyric();
}

function renderShelf() {
  shelfListEl.innerHTML = "";
  if (shelfFiles.length === 0) {
    const li = document.createElement("li");
    li.className = "muted";
    li.textContent = t("shelfEmpty");
    shelfListEl.appendChild(li);
    return;
  }
  for (const p of shelfFiles) {
    const name = String(p).split("/").pop() || String(p);
    const li = document.createElement("li");
    li.textContent = `📄 ${name}`;
    li.title = String(p);
    shelfListEl.appendChild(li);
  }
}

function applyLanguage(lang) {
  setLanguage(lang);
  applyI18n();
  renderBattery();
  renderNowPlaying();
  renderShelf();
  refreshTime();
  // Rebuilds the output selector so its "System default" option is
  // translated too.
  refreshAudioOutputs();
}

// Animation speed from settings: slow 0.5s / normal 0.25s / fast 0.1s.
// Applied as a CSS variable so the panel transition follows it.
function applyAnimationSpeed(speed) {
  const durations = { slow: "0.5s", normal: "0.25s", fast: "0.1s" };
  document.documentElement.style.setProperty(
    "--anim-duration",
    durations[speed] || durations.normal,
  );
}

function applySettings(settings) {
  if (!settings) return;
  const prevAudioOutput =
    lastSettings && lastSettings.media
      ? lastSettings.media.audio_output
      : undefined;
  lastSettings = settings;
  if (settings.language) {
    applyLanguage(settings.language);
  }
  if (settings.notch && settings.notch.animation_speed) {
    applyAnimationSpeed(settings.notch.animation_speed);
  }
  if (settings.media) {
    // Lyrics toggled off: hide the line. Toggled on: fetch for the
    // current track (if nothing is cached yet).
    if (!settings.media.lyrics) {
      lyricLines = [];
      npLyricEl.hidden = true;
    } else if (lastNowPlaying && lastNowPlaying.title) {
      const key = `${lastNowPlaying.app}|${lastNowPlaying.title}|${lastNowPlaying.artist}|${lastNowPlaying.album}`;
      if (key === currentTrackKey && lyricLines.length === 0) {
        fetchLyrics(lastNowPlaying);
      }
    }
    // Audio output changed (e.g. from the Settings window): apply it.
    const uid = settings.media.audio_output;
    if (uid && uid !== "system" && uid !== prevAudioOutput) {
      applyOutputUid(uid);
    }
    // Keep the panel selector in sync with the saved value.
    if (npOutputEl) {
      const known = audioOutputs.some((d) => d.uid === uid);
      npOutputEl.value = known ? uid : "system";
    }
    // Visualizer toggled or style changed: apply immediately.
    applyVizStyle();
    syncVisualizer();
  }
}

// Switch the system default output to the device with this UID.
// Best-effort: a missing/unplugged device is ignored silently.
async function applyOutputUid(uid) {
  if (!uid || uid === "system") return;
  if (lastAppliedOutputUid === uid) return;
  try {
    if (audioOutputs.length === 0) {
      audioOutputs = await invoke("get_audio_outputs");
    }
    const dev = audioOutputs.find((d) => d.uid === uid);
    if (dev) {
      await invoke("set_audio_output", { device_id: dev.id });
      lastAppliedOutputUid = uid;
    }
  } catch (e) {
    console.warn("applying saved audio output failed:", e);
  }
}

async function refreshAudioOutputs() {
  if (!npOutputEl) return;
  try {
    audioOutputs = await invoke("get_audio_outputs");
  } catch (e) {
    audioOutputs = [];
    console.warn("get_audio_outputs failed:", e);
  }
  const saved =
    lastSettings && lastSettings.media
      ? lastSettings.media.audio_output
      : "system";
  npOutputEl.innerHTML = "";
  const sysOpt = document.createElement("option");
  sysOpt.value = "system";
  sysOpt.textContent = t("audioOutputSystem");
  npOutputEl.appendChild(sysOpt);
  for (const dev of audioOutputs) {
    const opt = document.createElement("option");
    opt.value = dev.uid;
    opt.textContent = dev.name;
    npOutputEl.appendChild(opt);
  }
  const known = audioOutputs.some((d) => d.uid === saved);
  npOutputEl.value = known ? saved : "system";
}

async function persistAudioOutput(uid) {
  try {
    const settings = await invoke("get_settings");
    if (!settings) return;
    settings.media = settings.media || {};
    settings.media.audio_output = uid;
    lastSettings = await invoke("set_settings", { settings });
  } catch (e) {
    console.warn("persisting audio output failed:", e);
  }
}

async function setExpanded(next) {
  if (next === expanded) return;
  expanded = next;
  notchEl.classList.toggle("expanded", expanded);
  if (expanded) {
    // Devices can change while the panel is closed (AirPods connect).
    refreshAudioOutputs();
  }
  try {
    await invoke("set_notch_expanded", { expanded });
  } catch (e) {
    // Window resize is a nice-to-have on first run; the CSS class
    // already switched the panel, so do not crash the UI.
    console.warn("set_notch_expanded failed:", e);
  }
}

// Hover to expand, with a small delay so brushing past the notch
// does not flash the panel open.
notchEl.addEventListener("mouseenter", () => {
  clearTimeout(expandTimer);
  expandTimer = setTimeout(() => setExpanded(true), 120);
});
notchEl.addEventListener("mouseleave", () => {
  clearTimeout(expandTimer);
  expandTimer = setTimeout(() => setExpanded(false), 250);
});

// Gear button opens the Settings window (the only place the
// language can be changed now — the panel-top switcher is gone).
$("btn-settings").addEventListener("click", () => {
  invoke("open_settings").catch((e) =>
    console.warn("open_settings failed:", e),
  );
});

async function refreshTime() {
  try {
    const time = await invoke("get_time");
    // get_time returns "2026-11-10 11:33"
    const [date, hm] = String(time).split(" ");
    if (clockEl) clockEl.textContent = hm || time;
    if (dateEl && date) dateEl.textContent = date;
  } catch (e) {
    // Fallback: local JS clock, so the panel clock is never blank.
    const now = new Date();
    const hm = now.toTimeString().slice(0, 5);
    if (clockEl) clockEl.textContent = hm;
    if (dateEl) {
      dateEl.textContent = now.toLocaleDateString(DATE_LOCALES[getLanguage()] || "en-US");
    }
    console.warn("get_time failed, using local clock:", e);
  }
}

async function refreshBattery() {
  try {
    lastBattery = await invoke("get_battery");
  } catch (e) {
    lastBattery = null;
    console.warn("get_battery failed:", e);
  }
  renderBattery();
}

async function refreshNowPlaying() {
  try {
    lastNowPlaying = await invoke("get_now_playing");
    lastPosition = lastNowPlaying ? lastNowPlaying.position || 0 : 0;
    lastDuration = lastNowPlaying ? lastNowPlaying.duration || 0 : 0;
    lastPositionAt = Date.now();
  } catch (e) {
    lastNowPlaying = null;
    npTitleEl.textContent = t("notPlaying");
    npArtistEl.textContent = t("musicLoadFailed");
    npArtEl.hidden = true;
    npArtEl.removeAttribute("src");
    npProgressRow.hidden = true;
    npLyricEl.hidden = true;
    console.warn("get_now_playing failed:", e);
    return;
  }
  renderNowPlaying();
}

async function mediaCommand(cmd) {
  try {
    await invoke(cmd);
  } catch (e) {
    console.warn(cmd + " failed:", e);
  }
  // Refresh after a short delay so the new state is shown.
  setTimeout(refreshNowPlaying, 400);
}

playBtn.addEventListener("click", () => mediaCommand("media_play_pause"));
$("btn-prev").addEventListener("click", () => mediaCommand("media_previous"));
$("btn-next").addEventListener("click", () => mediaCommand("media_next"));

// Seek bar: drag updates the elapsed label only; releasing the
// slider seeks (avoids spamming the player mid-drag).
npSeekEl.addEventListener("input", () => {
  seeking = true;
  npElapsedEl.textContent = formatTime(parseFloat(npSeekEl.value) || 0);
});
npSeekEl.addEventListener("change", async () => {
  const value = parseFloat(npSeekEl.value) || 0;
  seeking = false;
  lastPosition = value;
  lastPositionAt = Date.now();
  renderProgress();
  try {
    await invoke("media_seek", { position: value });
  } catch (e) {
    console.warn("media_seek failed:", e);
  }
  setTimeout(refreshNowPlaying, 400);
});

// Audio output selector: switch immediately, then persist the UID
// so the Settings window and the next start agree.
npOutputEl.addEventListener("change", async () => {
  const uid = npOutputEl.value;
  if (uid && uid !== "system") {
    const dev = audioOutputs.find((d) => d.uid === uid);
    if (dev) {
      try {
        await invoke("set_audio_output", { device_id: dev.id });
        lastAppliedOutputUid = uid;
      } catch (e) {
        console.warn("set_audio_output failed:", e);
      }
    }
  }
  await persistAudioOutput(uid);
});

// --- File shelf (MVP: list file names only, no AirDrop) ---
function addShelfFiles(paths) {
  if (!Array.isArray(paths) || paths.length === 0) return;
  shelfFiles = shelfFiles.concat(paths.map(String));
  renderShelf();
}

async function listenDragDrop() {
  if (!tauri || !tauri.event || typeof tauri.event.listen !== "function") {
    return;
  }
  try {
    await tauri.event.listen("tauri://drag-drop", (event) => {
      // Tauri 2 drag-drop payload: { paths: string[], position: {...} }
      const paths = event && event.payload && event.payload.paths;
      addShelfFiles(paths);
      // Keep the panel open while the user is dropping files.
      setExpanded(true);
    });
  } catch (e) {
    console.warn("drag-drop listen failed:", e);
  }
}

// The Settings window saves via set_settings, and Rust emits
// "settings-changed" to every window. Re-apply language and
// animation speed here so the panel updates instantly.
async function listenSettingsChanged() {
  if (!tauri || !tauri.event || typeof tauri.event.listen !== "function") {
    return;
  }
  try {
    await tauri.event.listen("settings-changed", (event) => {
      applySettings(event && event.payload);
    });
  } catch (e) {
    console.warn("settings-changed listen failed:", e);
  }
}

// Language source of truth is settings.json (Rust). On first run
// there is no file yet, so Rust returns an empty language and we
// fall back to the localStorage cache / system detection — then
// persist that detected choice so later starts use it.
async function loadLanguageFromSettings() {
  let settings = null;
  try {
    settings = await invoke("get_settings");
  } catch (e) {
    console.warn("get_settings failed:", e);
  }
  if (settings) lastSettings = settings;
  if (settings && settings.language) {
    setLanguage(settings.language);
    applyAnimationSpeed(
      settings.notch ? settings.notch.animation_speed : "normal",
    );
    return;
  }
  initLanguage();
  // Persist the detected language for the next start.
  try {
    const base = settings || (await invoke("get_settings"));
    if (base) {
      base.language = getLanguage();
      lastSettings = await invoke("set_settings", { settings: base });
    }
  } catch (e) {
    console.warn("persisting detected language failed:", e);
  }
}

async function init() {
  buildVizBars();
  await loadLanguageFromSettings();
  applyI18n();
  renderBattery();
  renderNowPlaying();
  renderShelf();
  await refreshAudioOutputs();
  // Re-apply a saved output device once at startup (best-effort:
  // the device may be unplugged). Not re-applied on expand, so a
  // change made in Control Centre is not fought.
  if (
    !savedOutputApplied &&
    lastSettings &&
    lastSettings.media &&
    lastSettings.media.audio_output &&
    lastSettings.media.audio_output !== "system"
  ) {
    savedOutputApplied = true;
    await applyOutputUid(lastSettings.media.audio_output);
  }
  // Ask Rust for the real notch geometry first; the Rust side also
  // positions the window in setup, so a failure here is not fatal.
  try {
    await invoke("get_notch_geometry");
  } catch (e) {
    console.warn("get_notch_geometry failed:", e);
  }
  await refreshTime();
  await refreshBattery();
  await refreshNowPlaying();
  await listenDragDrop();
  await listenSettingsChanged();
  setInterval(refreshTime, 15000);
  setInterval(refreshBattery, 30000);
  setInterval(refreshNowPlaying, 5000);
  setInterval(tickPosition, 1000);
}

init();
