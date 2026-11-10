// notch-pilot Settings window — vanilla frontend, no build step.
// Reuses ./i18n.js so the Settings window itself is trilingual.
//
// Every control carries data-setting="section.key" (dot path into the
// Settings object). Changes are saved immediately via the
// set_settings command; Rust persists settings.json and emits
// "settings-changed" so the main panel re-applies the new values.
// The language control is the segmented control at the top of the
// Notch section — the only language selector in the app.

import {
  applyI18n,
  getLanguage,
  initLanguage,
  setLanguage,
  t,
} from "./i18n.js";

const $ = (id) => document.getElementById(id);
const tauri = window.__TAURI__ || null;

let currentSettings = null;
let saveTimer = null;
let flashTimer = null;

function invoke(cmd, args) {
  if (!tauri || !tauri.core || typeof tauri.core.invoke !== "function") {
    return Promise.reject(new Error("Tauri is not available"));
  }
  return tauri.core.invoke(cmd, args);
}

function getPath(obj, path) {
  return path.split(".").reduce((o, k) => (o == null ? undefined : o[k]), obj);
}

function setPath(obj, path, value) {
  const keys = path.split(".");
  let o = obj;
  for (let i = 0; i < keys.length - 1; i++) {
    if (o[keys[i]] == null) o[keys[i]] = {};
    o = o[keys[i]];
  }
  o[keys[keys.length - 1]] = value;
}

// --- Read the form into a Settings object (clone of loaded) --------
function collectSettings() {
  const out = JSON.parse(JSON.stringify(currentSettings || {}));
  document.querySelectorAll("[data-setting]").forEach((el) => {
    const path = el.getAttribute("data-setting");
    if (el.tagName === "BUTTON") return; // segmented: handled below
    if (el.type === "checkbox") {
      setPath(out, path, el.checked);
    } else if (el.type === "number") {
      const n =
        el.getAttribute("data-number") === "int"
          ? parseInt(el.value, 10)
          : parseFloat(el.value);
      // Empty / invalid: idle width means "auto" (0); anything else
      // keeps the previously saved value.
      if (Number.isNaN(n)) {
        if (path === "notch.idle_width") setPath(out, path, 0);
      } else {
        setPath(out, path, n);
      }
    } else if (el.getAttribute("data-list") === "true") {
      setPath(
        out,
        path,
        String(el.value)
          .split(",")
          .map((s) => s.trim())
          .filter((s) => s.length > 0),
      );
    } else {
      setPath(out, path, el.value);
    }
  });
  // Segmented controls: the active button's data-value wins.
  document
    .querySelectorAll(".segmented [data-setting][data-value]")
    .forEach((btn) => {
      if (btn.classList.contains("active")) {
        setPath(out, btn.getAttribute("data-setting"), btn.getAttribute("data-value"));
      }
    });
  // Language comes from the i18n state, not a data-setting control.
  out.language = getLanguage();
  return out;
}

// --- Write a Settings object into the form --------------------------
function populateForm(settings) {
  document.querySelectorAll("[data-setting]").forEach((el) => {
    const path = el.getAttribute("data-setting");
    const value = getPath(settings, path);
    if (el.tagName === "BUTTON") {
      el.classList.toggle("active", String(value) === el.getAttribute("data-value"));
      el.setAttribute("aria-pressed", String(value) === el.getAttribute("data-value") ? "true" : "false");
      return;
    }
    if (el.type === "checkbox") {
      el.checked = value === true;
    } else if (el.getAttribute("data-list") === "true") {
      el.value = Array.isArray(value) ? value.join(", ") : "";
    } else if (value != null) {
      el.value = String(value);
    }
  });
}

function showSavedFlash() {
  const flash = $("saved-flash");
  flash.hidden = false;
  clearTimeout(flashTimer);
  flashTimer = setTimeout(() => {
    flash.hidden = true;
  }, 1200);
}

async function save() {
  if (!currentSettings) return;
  const next = collectSettings();
  try {
    currentSettings = await invoke("set_settings", { settings: next });
    showSavedFlash();
  } catch (e) {
    console.warn("set_settings failed:", e);
  }
}

// Debounce text/number typing so every keystroke is not a save.
function saveSoon() {
  clearTimeout(saveTimer);
  saveTimer = setTimeout(save, 400);
}

function applyLanguageToWindow(lang) {
  setLanguage(lang);
  applyI18n();
  document.documentElement.lang =
    lang === "zh-TW" ? "zh-Hant" : lang === "zh-CN" ? "zh-Hans" : "en";
  document.title = `NotchPilot ${t("settings")}`;
}

// --- Wiring -----------------------------------------------------------
function wireControls() {
  // Plain controls: save on change (checkbox/select/color) or on
  // input with debounce (text/number).
  document.querySelectorAll("[data-setting]").forEach((el) => {
    if (el.tagName === "BUTTON") return;
    if (el.type === "checkbox" || el.tagName === "SELECT" || el.type === "color") {
      el.addEventListener("change", save);
    } else {
      el.addEventListener("input", saveSoon);
      el.addEventListener("change", save);
    }
  });

  // Segmented controls (animation speed, mirror shape).
  document
    .querySelectorAll(".segmented [data-setting][data-value]")
    .forEach((btn) => {
      btn.addEventListener("click", () => {
        const group = btn.closest(".segmented");
        group.querySelectorAll("button").forEach((b) => b.classList.remove("active"));
        btn.classList.add("active");
        save();
      });
    });

  // Language segmented control — the only language selector.
  document.querySelectorAll("[data-lang]").forEach((btn) => {
    btn.addEventListener("click", () => {
      applyLanguageToWindow(btn.getAttribute("data-lang"));
      save();
    });
  });

  // Sidebar section switching.
  document.querySelectorAll(".side-item").forEach((item) => {
    item.addEventListener("click", () => {
      document.querySelectorAll(".side-item").forEach((i) => i.classList.remove("active"));
      item.classList.add("active");
      const section = item.getAttribute("data-section");
      document.querySelectorAll(".panel").forEach((p) => {
        p.classList.toggle("active", p.getAttribute("data-panel") === section);
      });
    });
  });
}

async function listenSettingsChanged() {
  if (!tauri || !tauri.event || typeof tauri.event.listen !== "function") {
    return;
  }
  try {
    // Another window saved settings: re-populate the form so it
    // never shows stale values.
    await tauri.event.listen("settings-changed", (event) => {
      if (event && event.payload) {
        currentSettings = event.payload;
        populateForm(currentSettings);
        if (currentSettings.language) {
          applyLanguageToWindow(currentSettings.language);
        }
      }
    });
  } catch (e) {
    console.warn("settings-changed listen failed:", e);
  }
}

async function init() {
  let settings = null;
  try {
    settings = await invoke("get_settings");
  } catch (e) {
    console.warn("get_settings failed:", e);
  }
  currentSettings = settings || {};
  // Language: saved choice wins, else the localStorage cache /
  // system detection (same rule as the main panel).
  if (settings && settings.language) {
    applyLanguageToWindow(settings.language);
  } else {
    initLanguage();
    applyI18n();
  }
  populateForm(currentSettings);
  wireControls();
  await listenSettingsChanged();
}

init();
