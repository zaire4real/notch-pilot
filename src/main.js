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
const pillTimeEl = $("pill-time");
const pillBatteryEl = $("pill-battery");
const clockEl = $("clock");
const dateEl = $("date");
const batteryEl = $("battery");
const npTitleEl = $("np-title");
const npArtistEl = $("np-artist");
const playBtn = $("btn-play");
const shelfListEl = $("shelf-list");

let expanded = false;
let expandTimer = null;
let tauri = window.__TAURI__ || null;

// Last known dynamic state, kept so a language switch can re-render
// everything immediately without waiting for the next poll.
let lastBattery = null;
let lastNowPlaying = null;
let shelfFiles = [];

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

function renderNowPlaying() {
  const np = lastNowPlaying;
  if (!np || !np.title) {
    npTitleEl.textContent = t("notPlaying");
    npArtistEl.textContent = t("notPlayingHint");
    playBtn.textContent = "▶";
    playBtn.setAttribute("title", t("play"));
    return;
  }
  npTitleEl.textContent = np.title;
  npArtistEl.textContent = np.artist
    ? `${np.artist}${np.album ? ` · ${np.album}` : ""}`
    : np.album || np.app || "";
  playBtn.textContent = np.playing ? "⏸" : "▶";
  playBtn.setAttribute("title", np.playing ? t("pause") : t("play"));
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
}

async function setExpanded(next) {
  if (next === expanded) return;
  expanded = next;
  notchEl.classList.toggle("expanded", expanded);
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

// Language switcher: three buttons, instant update, persisted in
// localStorage by setLanguage() in i18n.js.
document.querySelectorAll("[data-lang]").forEach((btn) => {
  btn.addEventListener("click", () => applyLanguage(btn.getAttribute("data-lang")));
});

async function refreshTime() {
  try {
    const time = await invoke("get_time");
    // get_time returns "2026-11-10 11:33"
    const [date, hm] = String(time).split(" ");
    if (hm) pillTimeEl.textContent = hm;
    if (clockEl) clockEl.textContent = hm || time;
    if (dateEl && date) dateEl.textContent = date;
  } catch (e) {
    // Fallback: local JS clock, so the pill is never blank.
    const now = new Date();
    const hm = now.toTimeString().slice(0, 5);
    pillTimeEl.textContent = hm;
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
  } catch (e) {
    lastNowPlaying = null;
    npTitleEl.textContent = t("notPlaying");
    npArtistEl.textContent = t("musicLoadFailed");
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

async function init() {
  // Default language: stored choice, else system language
  // (zh-TW first, then zh-CN, otherwise en), set in i18n.js.
  initLanguage();
  applyI18n();
  renderBattery();
  renderNowPlaying();
  renderShelf();
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
  setInterval(refreshTime, 15000);
  setInterval(refreshBattery, 30000);
  setInterval(refreshNowPlaying, 5000);
}

init();
