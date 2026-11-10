// notch-pilot — i18n dictionary.
// This is the ONLY file in src/ that contains translated UI strings.
// index.html and main.js must not contain hardcoded Chinese or English
// UI text; they read everything from here via t() / applyI18n().

export const SUPPORTED_LANGUAGES = ["zh-TW", "zh-CN", "en"];
export const DEFAULT_LANGUAGE = "zh-TW";
export const STORAGE_KEY = "notch-pilot-lang";

// Native name of each language, shown on the switcher buttons.
export const LANGUAGE_NAMES = {
  "zh-TW": "繁體中文",
  "zh-CN": "简体中文",
  en: "English",
};

// BCP 47 locale used for toLocaleDateString() per language.
export const DATE_LOCALES = {
  "zh-TW": "zh-TW",
  "zh-CN": "zh-CN",
  en: "en-US",
};

export const STRINGS = {
  "zh-TW": {
    appTitle: "NotchPilot",
    language: "語言",
    loading: "載入中…",
    time: "時間",
    battery: "電量",
    charging: "充電中",
    batteryUnknown: "🔋 --%",
    notPlaying: "目前沒有播放",
    notPlayingHint: "開啟 Music 或 Spotify 後會顯示在這裡",
    musicLoadFailed: "音樂資訊讀取失敗（只支援 Music / Spotify）",
    music: "音樂",
    play: "播放",
    pause: "暫停",
    playPause: "播放 / 暫停",
    previous: "上一首",
    next: "下一首",
    shelf: "檔案架",
    shelfHint: "— 把檔案拖到這裡，只會列出檔名（MVP 不會 AirDrop）",
    shelfEmpty: "還沒有檔案，拖一個檔案進來試試。",
    settings: "設定",
    settingsNote: "進階的劉海高度／手勢目前 MVP 沒有設定頁。",
    footerNote:
      "NotchPilot 0.1.0 · Rust + Tauri 2 乾淨重寫 MVP · 沒有視覺化、AirDrop、相機鏡像、日曆整合",
  },
  "zh-CN": {
    appTitle: "NotchPilot",
    language: "语言",
    loading: "加载中…",
    time: "时间",
    battery: "电量",
    charging: "充电中",
    batteryUnknown: "🔋 --%",
    notPlaying: "目前没有播放",
    notPlayingHint: "打开 Music 或 Spotify 后会显示在这里",
    musicLoadFailed: "音乐信息读取失败（只支持 Music / Spotify）",
    music: "音乐",
    play: "播放",
    pause: "暂停",
    playPause: "播放 / 暂停",
    previous: "上一首",
    next: "下一首",
    shelf: "文件架",
    shelfHint: "— 把文件拖到这里，只会列出文件名（MVP 不会 AirDrop）",
    shelfEmpty: "还没有文件，拖一个文件进来试试。",
    settings: "设置",
    settingsNote: "进阶的刘海高度／手势目前 MVP 没有设置页。",
    footerNote:
      "NotchPilot 0.1.0 · Rust + Tauri 2 干净重写 MVP · 没有可视化、AirDrop、相机镜像、日历整合",
  },
  en: {
    appTitle: "NotchPilot",
    language: "Language",
    loading: "Loading…",
    time: "Time",
    battery: "Battery",
    charging: "Charging",
    batteryUnknown: "🔋 --%",
    notPlaying: "Nothing playing",
    notPlayingHint: "Open Music or Spotify and it will show up here",
    musicLoadFailed: "Could not load music info (Music / Spotify only)",
    music: "Music",
    play: "Play",
    pause: "Pause",
    playPause: "Play / Pause",
    previous: "Previous",
    next: "Next",
    shelf: "Shelf",
    shelfHint: "— Drop files here; only file names are listed (no AirDrop in the MVP)",
    shelfEmpty: "No files yet. Drop a file here to try it.",
    settings: "Settings",
    settingsNote: "Advanced notch height / gestures have no settings page in the MVP.",
    footerNote:
      "NotchPilot 0.1.0 · Clean-room Rust + Tauri 2 MVP · No visualizer, AirDrop, camera mirror, or calendar integration",
  },
};

let currentLanguage = DEFAULT_LANGUAGE;

export function normalizeLanguage(value) {
  if (SUPPORTED_LANGUAGES.includes(value)) return value;
  return DEFAULT_LANGUAGE;
}

export function detectLanguage() {
  try {
    const stored = window.localStorage.getItem(STORAGE_KEY);
    if (stored && SUPPORTED_LANGUAGES.includes(stored)) return stored;
  } catch (_e) {
    // localStorage may be unavailable; fall through to system language.
  }
  const nav = (window.navigator && (window.navigator.language || window.navigator.userLanguage)) || "";
  const lang = String(nav);
  // zh-TW first, then any other zh-* as zh-CN, otherwise en.
  if (/^zh[-_](TW|Hant|HK|MO)/i.test(lang) || /^zh-Hant/i.test(lang)) return "zh-TW";
  if (/^zh/i.test(lang)) return "zh-CN";
  return "en";
}

export function getLanguage() {
  return currentLanguage;
}

export function t(key) {
  const dict = STRINGS[currentLanguage] || STRINGS[DEFAULT_LANGUAGE];
  if (Object.prototype.hasOwnProperty.call(dict, key)) return dict[key];
  const fallback = STRINGS[DEFAULT_LANGUAGE];
  return Object.prototype.hasOwnProperty.call(fallback, key) ? fallback[key] : key;
}

export function setLanguage(lang) {
  currentLanguage = normalizeLanguage(lang);
  try {
    window.localStorage.setItem(STORAGE_KEY, currentLanguage);
  } catch (_e) {
    // Ignore storage failures; the language still applies in memory.
  }
  document.documentElement.lang =
    currentLanguage === "zh-TW" ? "zh-Hant" : currentLanguage === "zh-CN" ? "zh-Hans" : "en";
  return currentLanguage;
}

export function initLanguage() {
  return setLanguage(detectLanguage());
}

// Apply all static strings: elements with data-i18n get textContent,
// data-i18n-title sets the title attribute, data-i18n-aria-label sets
// aria-label. Language switcher buttons get their native names.
export function applyI18n() {
  document.querySelectorAll("[data-i18n]").forEach((el) => {
    el.textContent = t(el.getAttribute("data-i18n"));
  });
  document.querySelectorAll("[data-i18n-title]").forEach((el) => {
    el.setAttribute("title", t(el.getAttribute("data-i18n-title")));
  });
  document.querySelectorAll("[data-i18n-aria-label]").forEach((el) => {
    el.setAttribute("aria-label", t(el.getAttribute("data-i18n-aria-label")));
  });
  document.querySelectorAll("[data-lang]").forEach((btn) => {
    const code = btn.getAttribute("data-lang");
    btn.textContent = LANGUAGE_NAMES[code] || code;
    btn.classList.toggle("active", code === currentLanguage);
    btn.setAttribute("aria-pressed", code === currentLanguage ? "true" : "false");
  });
}
