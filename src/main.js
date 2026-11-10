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
const pillArtEl = $("pill-art");
const pillEyesEl = $("pill-eyes");
const calWeekdayEl = $("cal-weekday");
const calDateEl = $("cal-date");
const calTimeEl = $("cal-time");
const calNextEl = $("cal-next");
const calStatusEl = $("cal-status");
const calEventsEl = $("cal-events");
const calTomorrowEventEl = $("cal-tomorrow-event");
const calRemindersWrapEl = $("cal-reminders-wrap");
const remStatusEl = $("rem-status");
const remListEl = $("rem-list");
const remEmptyEl = $("rem-empty");
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
const airdropBtnEl = $("btn-airdrop");
const airdropStatusEl = $("airdrop-status");
const viewHomeEl = $("view-home");
const viewShelfEl = $("view-shelf");
const shelfChipEl = $("shelf-chip");
const peekEl = $("sneak-peek");
const peekArtEl = $("peek-art");
const peekTextEl = $("peek-text");

let expanded = false;
let expandTimer = null;
let tauri = window.__TAURI__ || null;

// Last known dynamic state, kept so a language switch can re-render
// everything immediately without waiting for the next poll.
let lastBattery = null;
let lastNowPlaying = null;
let lastSettings = null;
let lastCalendar = null;
let lastReminders = null;
let shelfFiles = [];

// AirDrop (v0.2 Phase 4b): `airdropAvailable` is null until the
// first `airdrop_available` check returns; false means the macOS
// sharing service is missing/unconfigured and the button stays
// disabled with a reason. `airdropBusy` disables the button while
// a send is in flight.
let airdropAvailable = null;
let airdropBusy = false;

// Home ↔ Shelf view state (Phase 2: the shelf is its own view;
// the calendar placeholder lives on Home only).
let currentView = "home";

// Sneak-peek state machine (Phase 2): collapsed → peek → collapsed
// (auto after ~2 s) or → expanded (user hovers). Later peek kinds
// (volume / charging / Bluetooth) plug into showSneakPeek the same
// way the track-change peek does below.
const SNEAK_PEEK_MS = 2000;
let peekTimer = null;
let peekKind = null;

// Idle "simple eyes" (F23): blink on a slow random timer, glance
// shifts with the mouse near the notch.
let eyesBlinkTimer = null;

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
      style === "circular" ? `rotate(${i * 45}deg) translateY(-46px)` : "";
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

// --- Collapsed pill state (Phase 2, §5.2) ------------------------
// Playing → art thumb (+ viz bars, handled by renderVizBars) on the
// left. Idle + show-eyes → the two eye capsules. Idle without eyes
// → empty left cluster; battery stays on the right either way.
function renderPillState() {
  const playing =
    lastNowPlaying && lastNowPlaying.title && lastNowPlaying.playing;
  const showEyes =
    lastSettings && lastSettings.appearance && lastSettings.appearance.show_eyes;
  if (pillArtEl) {
    const art =
      playing && lastNowPlaying
        ? lastNowPlaying.artwork || lastNowPlaying.artwork_url || ""
        : "";
    if (art) {
      if (pillArtEl.getAttribute("src") !== art) pillArtEl.src = art;
      pillArtEl.hidden = false;
    } else {
      pillArtEl.hidden = true;
      pillArtEl.removeAttribute("src");
    }
  }
  if (pillEyesEl) {
    pillEyesEl.hidden = !(!playing && showEyes);
  }
  scheduleEyesBlink();
}

// Blink timer: re-arms itself on a slow random interval (2.5–6 s)
// so the blink does not look metronomic. Only runs while the eyes
// are actually visible.
function scheduleEyesBlink() {
  if (eyesBlinkTimer !== null) {
    clearTimeout(eyesBlinkTimer);
    eyesBlinkTimer = null;
  }
  if (!pillEyesEl || pillEyesEl.hidden) return;
  eyesBlinkTimer = setTimeout(() => {
    if (pillEyesEl && !pillEyesEl.hidden) {
      pillEyesEl.classList.add("blink");
      setTimeout(() => pillEyesEl.classList.remove("blink"), 180);
    }
    scheduleEyesBlink();
  }, 2500 + Math.random() * 3500);
}

// Glance: the eyes shift a few px towards the cursor while the
// mouse moves over the pill.
notchEl.addEventListener("mousemove", (event) => {
  if (!pillEyesEl || pillEyesEl.hidden) return;
  const rect = notchEl.getBoundingClientRect();
  const dx = (event.clientX - (rect.left + rect.width / 2)) / rect.width;
  pillEyesEl.style.setProperty("--glance-x", `${(dx * 6).toFixed(1)}px`);
});
notchEl.addEventListener("mouseleave", () => {
  if (pillEyesEl) pillEyesEl.style.setProperty("--glance-x", "0px");
});

// --- Home ↔ Shelf view switch (Phase 2, §5.2) ----------------------
function showHomeView() {
  currentView = "home";
  if (viewHomeEl) viewHomeEl.hidden = false;
  if (viewShelfEl) viewShelfEl.hidden = true;
}

function showShelfView() {
  currentView = "shelf";
  if (viewHomeEl) viewHomeEl.hidden = true;
  if (viewShelfEl) viewShelfEl.hidden = false;
}

// --- Sneak-peek state machine (Phase 2, §5.2) ----------------------
// One-line transient HUD. `kind` is recorded so later phases can
// add volume / charging / Bluetooth peeks without touching the
// machine: showSneakPeek({ kind, art, text }). Only fires while the
// panel is collapsed; hovering expands to the full panel instead.
function showSneakPeek({ kind, art, text }) {
  if (expanded || !peekEl || !peekTextEl) return;
  peekKind = kind || "generic";
  if (peekArtEl) {
    if (art) {
      if (peekArtEl.getAttribute("src") !== art) peekArtEl.src = art;
      peekArtEl.hidden = false;
    } else {
      peekArtEl.hidden = true;
      peekArtEl.removeAttribute("src");
    }
  }
  peekTextEl.textContent = text || "";
  peekEl.hidden = false;
  notchEl.classList.add("peek");
  invoke("set_notch_peek", { show: true }).catch((e) =>
    console.warn("set_notch_peek failed:", e),
  );
  if (peekTimer !== null) clearTimeout(peekTimer);
  peekTimer = setTimeout(hideSneakPeek, SNEAK_PEEK_MS);
}

function hideSneakPeek() {
  if (peekTimer !== null) {
    clearTimeout(peekTimer);
    peekTimer = null;
  }
  peekKind = null;
  if (peekEl) peekEl.hidden = true;
  notchEl.classList.remove("peek");
  // If the user hovered into the full panel meanwhile, the window
  // is already at expanded size — do not shrink it back to idle.
  if (expanded) return;
  invoke("set_notch_peek", { show: false }).catch((e) =>
    console.warn("set_notch_peek failed:", e),
  );
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
    renderPillState();
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
  const prevKey = currentTrackKey;
  const key = `${np.app}|${np.title}|${np.artist}|${np.album}`;
  if (key !== currentTrackKey) {
    currentTrackKey = key;
    lyricLines = [];
    npLyricEl.hidden = true;
    npLyricEl.textContent = "";
    fetchLyrics(np);
    // First sneak-peek kind: track changed. Only when there was a
    // previous track (not on first load) — showSneakPeek itself
    // no-ops while the panel is expanded.
    if (prevKey) {
      showSneakPeek({
        kind: "track",
        art: np.artwork || np.artwork_url || "",
        text: np.title,
      });
    }
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
  renderPillState();
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

// --- File shelf (v0.2 Phase 4a, F7 — shelf half) -----------------
// Paths persist in settings.json (Rust: settings.shelf.files) and
// survive a restart; missing files are silently pruned on load.
// shelfFiles holds the Rust ShelfItem objects { path, name, size }.
// Click a file = open it with its default app; the × button or a
// right-click removes it from the shelf (and from the persisted
// list). Icons are file-extension → emoji — no Finder icon
// extraction in v0.2. AirDrop is Phase 4b (below): the dropzone
// button sends every file on the shelf — there is no per-file
// selection model, because a tile click already means "open".
function shelfIconFor(name) {
  const ext = String(name).split(".").pop().toLowerCase();
  if (["png", "jpg", "jpeg", "gif", "webp", "heic", "svg", "bmp", "tiff"].includes(ext)) return "🖼️";
  if (["mp4", "mov", "mkv", "avi", "webm", "m4v"].includes(ext)) return "🎬";
  if (["mp3", "wav", "flac", "aac", "m4a", "ogg"].includes(ext)) return "🎵";
  if (["pdf"].includes(ext)) return "📕";
  if (["doc", "docx", "pages", "rtf"].includes(ext)) return "📝";
  if (["xls", "xlsx", "csv", "numbers"].includes(ext)) return "📊";
  if (["ppt", "pptx", "key"].includes(ext)) return "📽️";
  if (["zip", "rar", "7z", "tar", "gz", "dmg", "pkg"].includes(ext)) return "📦";
  if (["js", "ts", "rs", "py", "swift", "go", "java", "c", "cpp", "h", "html", "css", "json", "md", "sh"].includes(ext)) return "💻";
  if (["txt", "log"].includes(ext)) return "📃";
  return "📄";
}

function formatFileSize(size) {
  if (size == null || !Number.isFinite(size) || size < 0) return "";
  if (size < 1024) return `${size} B`;
  if (size < 1024 * 1024) return `${(size / 1024).toFixed(1)} KB`;
  if (size < 1024 * 1024 * 1024) return `${(size / 1024 / 1024).toFixed(1)} MB`;
  return `${(size / 1024 / 1024 / 1024).toFixed(1)} GB`;
}

function renderShelf() {
  if (shelfChipEl) {
    shelfChipEl.textContent = `${t("shelf")}: ${shelfFiles.length} ${t("shelfFiles")} ▸`;
  }
  updateAirdropButton();
  shelfListEl.innerHTML = "";
  if (shelfFiles.length === 0) {
    const li = document.createElement("li");
    li.className = "muted";
    li.textContent = t("shelfEmpty");
    shelfListEl.appendChild(li);
    return;
  }
  for (const item of shelfFiles) {
    const path = typeof item === "string" ? item : item.path;
    const name =
      (typeof item === "object" && item.name) ||
      String(path).split("/").pop() ||
      String(path);
    const size =
      typeof item === "object" ? formatFileSize(item.size) : "";
    const li = document.createElement("li");
    li.className = "shelf-item";
    li.title = String(path);
    const icon = document.createElement("span");
    icon.className = "shelf-icon";
    icon.textContent = shelfIconFor(name);
    const nameEl = document.createElement("span");
    nameEl.className = "shelf-name";
    nameEl.textContent = name;
    li.appendChild(icon);
    li.appendChild(nameEl);
    if (size) {
      const sizeEl = document.createElement("span");
      sizeEl.className = "shelf-size muted";
      sizeEl.textContent = size;
      li.appendChild(sizeEl);
    }
    const removeBtn = document.createElement("button");
    removeBtn.type = "button";
    removeBtn.className = "shelf-remove";
    removeBtn.textContent = "×";
    removeBtn.title = t("shelfRemove");
    removeBtn.setAttribute("aria-label", `${t("shelfRemove")}: ${name}`);
    removeBtn.addEventListener("click", (event) => {
      // The × must not also trigger the tile's open-on-click.
      event.stopPropagation();
      removeShelfFile(path);
    });
    li.appendChild(removeBtn);
    li.addEventListener("click", () => openShelfFile(path));
    li.addEventListener("contextmenu", (event) => {
      event.preventDefault();
      removeShelfFile(path);
    });
    shelfListEl.appendChild(li);
  }
}

// Load the persisted shelf (prunes missing files on the Rust side).
async function refreshShelf() {
  try {
    const items = await invoke("get_shelf");
    if (Array.isArray(items)) shelfFiles = items;
  } catch (e) {
    console.warn("get_shelf failed:", e);
  }
  renderShelf();
}

async function openShelfFile(path) {
  try {
    await invoke("open_shelf_file", { path });
  } catch (e) {
    // Most likely the file was deleted since the list was loaded:
    // re-fetch so the pruned list is shown.
    console.warn("open_shelf_file failed:", e);
    await refreshShelf();
  }
}

async function removeShelfFile(path) {
  try {
    const items = await invoke("remove_shelf_file", { path });
    if (Array.isArray(items)) shelfFiles = items;
  } catch (e) {
    console.warn("remove_shelf_file failed:", e);
  }
  renderShelf();
}

// --- AirDrop (v0.2 Phase 4b, F7) ---------------------------------
// Selection model (documented choice): the button sends ALL files
// currently on the shelf, in shelf order. The Rust side presents
// the macOS AirDrop picker (NSSharingService); success means the
// picker opened, not that the file arrived — no delegate reports
// the transfer result (see src-tauri/src/airdrop.rs). Any failure
// is shown on the status line below the button; the shelf itself
// keeps working either way.

function setAirdropStatus(text, isError) {
  if (!airdropStatusEl) return;
  airdropStatusEl.textContent = text || "";
  airdropStatusEl.classList.toggle("error", !!isError);
}

function updateAirdropButton() {
  if (!airdropBtnEl) return;
  // The label is the service brand name, the same in every language.
  airdropBtnEl.textContent = `${t("airdrop")} ⤴`;
  // Settings → Shelf → AirDrop off hides the button entirely (§6).
  // lastSettings is null before the first load; the default is on.
  const settingOn =
    !lastSettings ||
    !lastSettings.shelf ||
    lastSettings.shelf.airdrop !== false;
  airdropBtnEl.hidden = !settingOn;
  if (!settingOn) {
    setAirdropStatus("");
    return;
  }
  // Disabled states always carry a reason (button title); the
  // button is never a silent no-op.
  let reason = "";
  if (shelfFiles.length === 0) reason = t("airdropEmpty");
  else if (airdropAvailable === false) reason = t("airdropUnavailable");
  airdropBtnEl.disabled = airdropBusy || reason !== "";
  airdropBtnEl.title = reason || t("airdropTitle");
}

async function refreshAirdropAvailability() {
  try {
    airdropAvailable = await invoke("airdrop_available");
  } catch (e) {
    // A failed check must not disable the shelf: leave the button
    // enabled and let the send itself report the real error.
    airdropAvailable = null;
    console.warn("airdrop_available failed:", e);
  }
  updateAirdropButton();
}

async function sendShelfViaAirdrop() {
  if (airdropBusy || shelfFiles.length === 0) return;
  airdropBusy = true;
  updateAirdropButton();
  setAirdropStatus("");
  try {
    const paths = shelfFiles.map((f) =>
      typeof f === "string" ? f : f.path,
    );
    await invoke("airdrop_shelf_files", { paths });
    setAirdropStatus(t("airdropOpened"));
  } catch (e) {
    setAirdropStatus(`${t("airdropFailed")}: ${e}`, true);
  }
  airdropBusy = false;
  updateAirdropButton();
}

function applyLanguage(lang) {
  setLanguage(lang);
  applyI18n();
  renderBattery();
  renderNowPlaying();
  renderShelf();
  renderCalendar();
  renderReminders();
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
    // Appearance toggles (e.g. show-eyes) re-render the pill.
    renderPillState();
  }
  // Calendar/reminders settings may have changed (which calendars,
  // reminders on/off): re-fetch so the section follows immediately.
  refreshCalendar();
  // The shelf list can change from another window via set_settings
  // (the Settings window round-trips it); re-fetch when it differs.
  if (settings.shelf && Array.isArray(settings.shelf.files)) {
    const saved = settings.shelf.files.map(String);
    const current = shelfFiles.map((f) =>
      typeof f === "string" ? f : f.path,
    );
    if (JSON.stringify(saved) !== JSON.stringify(current)) {
      refreshShelf();
    }
  }
  // The AirDrop toggle (Settings → Shelf) hides/shows the button.
  updateAirdropButton();
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
  // A full expand cancels any sneak-peek; the resize below already
  // moves the window to the expanded size, so no peek invoke here.
  if (peekTimer !== null) {
    clearTimeout(peekTimer);
    peekTimer = null;
  }
  peekKind = null;
  if (peekEl) peekEl.hidden = true;
  notchEl.classList.remove("peek");
  notchEl.classList.toggle("expanded", expanded);
  if (expanded) {
    // Devices can change while the panel is closed (AirPods connect).
    refreshAudioOutputs();
    // Calendar/reminders can change while the panel is closed too.
    refreshCalendar();
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

// Shelf chip (Home bottom strip) ↔ Back button (Shelf view).
$("shelf-chip").addEventListener("click", () => showShelfView());
$("btn-shelf-back").addEventListener("click", () => showHomeView());

// AirDrop button (Phase 4b): sends every file on the shelf.
if (airdropBtnEl) {
  airdropBtnEl.addEventListener("click", () => sendShelfViaAirdrop());
}

async function refreshTime() {
  // The clock now lives as a small line in the calendar header
  // (§5.2): weekday + date + time. get_time returns "2026-11-10 11:33".
  const locale = DATE_LOCALES[getLanguage()] || "en-US";
  const render = (dateStr, hm) => {
    if (calTimeEl) calTimeEl.textContent = hm || "";
    if (!dateStr) return;
    const d = new Date(`${dateStr}T00:00:00`);
    if (Number.isNaN(d.getTime())) {
      if (calDateEl) calDateEl.textContent = dateStr;
      if (calWeekdayEl) calWeekdayEl.textContent = "";
      return;
    }
    if (calWeekdayEl) {
      calWeekdayEl.textContent = d.toLocaleDateString(locale, {
        weekday: "long",
      });
    }
    if (calDateEl) {
      calDateEl.textContent = d.toLocaleDateString(locale, {
        month: "short",
        day: "numeric",
      });
    }
  };
  try {
    const time = await invoke("get_time");
    const [date, hm] = String(time).split(" ");
    render(date, hm);
  } catch (e) {
    // Fallback: local JS clock, so the header is never blank.
    const now = new Date();
    render(
      `${now.getFullYear()}-${String(now.getMonth() + 1).padStart(2, "0")}-${String(now.getDate()).padStart(2, "0")}`,
      now.toTimeString().slice(0, 5),
    );
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

// --- Calendar + Reminders (Phase 3, F2/F3) ------------------------
// Data comes from calendar.rs (AppleScript to Calendar.app /
// Reminders.app). The header weekday/date/time is filled by
// refreshTime(); this section fills the event list, the TOMORROW
// row, the next-event countdown and the reminder checkboxes.
function formatCountdown(mins) {
  if (!Number.isFinite(mins) || mins <= 0) return t("calNow");
  const m = Math.floor(mins);
  if (m < 60) return `${m} ${t("calMinutesShort")}`;
  const h = Math.floor(m / 60);
  const rest = m % 60;
  return rest === 0
    ? `${h} ${t("calHoursShort")}`
    : `${h} ${t("calHoursShort")} ${rest} ${t("calMinutesShort")}`;
}

function eventTimeLabel(ev) {
  if (!ev) return "";
  if (ev.all_day) return t("calAllDay");
  return (ev.start || "").slice(11) || "";
}

function renderCalendar() {
  const data = lastCalendar;
  if (!data) return;
  if (calStatusEl) {
    if (data.status === "ok") {
      calStatusEl.hidden = true;
    } else {
      calStatusEl.hidden = false;
      calStatusEl.textContent =
        data.status === "not_running"
          ? t("calNotRunning")
          : data.status === "permission_denied"
            ? t("calPermission")
            : t("calLoadFailed");
    }
  }
  if (calEventsEl) {
    calEventsEl.innerHTML = "";
    const events = Array.isArray(data.events) ? data.events : [];
    if (events.length === 0) {
      const li = document.createElement("li");
      li.className = "muted";
      li.textContent = t("calNoEvents");
      calEventsEl.appendChild(li);
    } else {
      for (const ev of events) {
        const li = document.createElement("li");
        const time = document.createElement("span");
        time.className = "cal-ev-time";
        time.textContent = eventTimeLabel(ev);
        li.appendChild(time);
        li.appendChild(document.createTextNode(ev.title || ""));
        calEventsEl.appendChild(li);
      }
    }
  }
  if (calNextEl) {
    if (data.next && data.next.title) {
      calNextEl.textContent = `${t("calNext")}: ${data.next.title} · ${formatCountdown(data.next.minutes_until)}`;
    } else {
      calNextEl.textContent = t("calNoEvents");
    }
  }
  if (calTomorrowEventEl) {
    if (data.tomorrow) {
      calTomorrowEventEl.textContent = `${eventTimeLabel(data.tomorrow)}  ${data.tomorrow.title}`;
    } else {
      calTomorrowEventEl.textContent = t("calTomorrowNone");
    }
  }
}

function renderReminders() {
  const data = lastReminders;
  const showRem =
    !lastSettings ||
    !lastSettings.calendar ||
    lastSettings.calendar.reminders !== false;
  if (calRemindersWrapEl) {
    calRemindersWrapEl.hidden =
      !showRem || (data && data.status === "disabled");
  }
  if (!showRem || !data) return;
  if (remStatusEl) {
    if (data.status === "ok") {
      remStatusEl.hidden = true;
    } else {
      remStatusEl.hidden = false;
      remStatusEl.textContent =
        data.status === "not_running"
          ? t("remNotRunning")
          : data.status === "permission_denied"
            ? t("remPermission")
            : t("remLoadFailed");
    }
  }
  const items = Array.isArray(data.reminders) ? data.reminders : [];
  if (remEmptyEl) {
    remEmptyEl.hidden = !(data.status === "ok" && items.length === 0);
  }
  if (remListEl) {
    remListEl.innerHTML = "";
    for (const item of items) {
      const li = document.createElement("li");
      const cb = document.createElement("input");
      cb.type = "checkbox";
      cb.setAttribute("aria-label", item.title || "");
      cb.addEventListener("change", () => completeReminder(item.id));
      const label = document.createElement("label");
      label.textContent = item.title || "";
      li.appendChild(cb);
      li.appendChild(label);
      if (item.due) {
        const due = document.createElement("span");
        due.className = "rem-due";
        due.textContent = item.due.slice(5); // "11-10 18:00"
        li.appendChild(due);
      }
      remListEl.appendChild(li);
    }
  }
}

async function completeReminder(id) {
  try {
    await invoke("complete_reminder", { reminder_id: id });
  } catch (e) {
    console.warn("complete_reminder failed:", e);
  }
  // Re-fetch so the checked-off reminder disappears (or reappears
  // if the check-off failed).
  setTimeout(refreshCalendar, 400);
}

async function refreshCalendar() {
  try {
    lastCalendar = await invoke("get_calendar_data");
  } catch (e) {
    console.warn("get_calendar_data failed:", e);
    lastCalendar = {
      status: "unavailable",
      events: [],
      tomorrow: null,
      next: null,
    };
  }
  renderCalendar();
  const showRem =
    !lastSettings ||
    !lastSettings.calendar ||
    lastSettings.calendar.reminders !== false;
  if (showRem) {
    try {
      lastReminders = await invoke("get_reminders");
    } catch (e) {
      console.warn("get_reminders failed:", e);
      lastReminders = { status: "unavailable", reminders: [] };
    }
  } else {
    lastReminders = { status: "disabled", reminders: [] };
  }
  renderReminders();
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
// Dropped paths go through Rust so they are persisted in
// settings.json immediately (Phase 4a).
async function addShelfFiles(paths) {
  if (!Array.isArray(paths) || paths.length === 0) return;
  try {
    const items = await invoke("add_shelf_files", {
      paths: paths.map(String),
    });
    if (Array.isArray(items)) shelfFiles = items;
  } catch (e) {
    console.warn("add_shelf_files failed:", e);
    // Fall back to the in-memory list so the drop still shows up
    // this session even if persistence failed.
    const known = new Set(shelfFiles.map((f) => (typeof f === "string" ? f : f.path)));
    for (const p of paths.map(String)) {
      if (!known.has(p)) {
        shelfFiles.push({ path: p, name: p.split("/").pop() || p, size: null });
        known.add(p);
      }
    }
  }
  renderShelf();
}

async function listenDragDrop() {
  if (!tauri || !tauri.event || typeof tauri.event.listen !== "function") {
    return;
  }
  // Dragging a file over the notch auto-switches to the Shelf view
  // (as in the original), unless the user turned that off.
  const openOnDrop = () =>
    !lastSettings ||
    !lastSettings.shelf ||
    lastSettings.shelf.open_on_drop !== false;
  try {
    await tauri.event.listen("tauri://drag-enter", () => {
      if (openOnDrop()) {
        setExpanded(true);
        showShelfView();
      }
    });
  } catch (e) {
    console.warn("drag-enter listen failed:", e);
  }
  try {
    await tauri.event.listen("tauri://drag-drop", (event) => {
      // Tauri 2 drag-drop payload: { paths: string[], position: {...} }
      const paths = event && event.payload && event.payload.paths;
      addShelfFiles(paths);
      // Keep the panel open while the user is dropping files.
      setExpanded(true);
      if (openOnDrop()) showShelfView();
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
  await refreshShelf();
  await refreshAirdropAvailability();
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
  await refreshCalendar();
  await listenDragDrop();
  await listenSettingsChanged();
  setInterval(refreshTime, 15000);
  setInterval(refreshBattery, 30000);
  setInterval(refreshNowPlaying, 5000);
  // Calendar/reminders poll only while the panel is expanded, so a
  // closed panel never wakes Calendar.app / Reminders.app.
  setInterval(() => {
    if (expanded) refreshCalendar();
  }, 300000);
  setInterval(tickPosition, 1000);
}

init();
