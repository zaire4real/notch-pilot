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
    settingsNote: "按 ⚙ 開啟設定。",
    footerNote:
      "NotchPilot 0.1.0 · Rust + Tauri 2 · 設定請按 ⚙",
    secNotch: "瀏海",
    secAppearance: "外觀",
    secMedia: "媒體",
    secCalendar: "日曆",
    secShelf: "檔案架",
    secMirror: "鏡像",
    secBattery: "電池",
    secOsd: "螢幕顯示",
    secNotifications: "通知",
    secShortcuts: "快捷鍵",
    secAbout: "關於",
    notchIdleWidth: "閒置寬度（0 = 自動）",
    notchExpandedWidth: "展開寬度",
    notchExpandedHeight: "展開高度",
    notchNonNotchHeight: "無瀏海螢幕高度",
    notchHoverOpen: "滑入開啟延遲（毫秒）",
    notchHoverClose: "滑出關閉延遲（毫秒）",
    animationSpeed: "動畫速度",
    animSlow: "慢",
    animNormal: "一般",
    animFast: "快",
    showOnAllDisplays: "在所有顯示器顯示",
    followActiveDisplay: "跟隨使用中的顯示器",
    accentColor: "強調色",
    showEyes: "閒置時顯示眼睛",
    theme: "主題",
    themeSystem: "系統",
    themeDark: "深色",
    themeLight: "淺色",
    visualizer: "音訊視覺化",
    visualizerStyle: "視覺化樣式",
    visualizerBars: "長條",
    visualizerCircular: "圓形",
    visualizerWave: "波形",
    vizAnimated: "動畫視覺化",
    vizNote:
      "即時頻譜使用 ScreenCaptureKit 抓取系統音訊，首次播放時會要求「螢幕錄製」權限；若未授權，會改以動畫呈現（非即時音訊）。",
    lyrics: "同步歌詞",
    audioOutput: "音訊輸出",
    audioOutputSystem: "系統預設",
    audioOutputLabel: "音訊輸出裝置",
    seek: "跳轉",
    mediaGestures: "媒體手勢",
    showCalendars: "顯示的日曆（空白 = 全部）",
    reminders: "提醒事項",
    airdrop: "AirDrop",
    openOnDrop: "拖入檔案時開啟檔案架",
    keepFiles: "保留檔案",
    mirrorEnabled: "啟用相機鏡像",
    mirrorShape: "鏡像形狀",
    shapeCircle: "圓形",
    shapeRounded: "圓角矩形",
    camera: "相機（空白 = 預設）",
    showPercent: "顯示電量百分比",
    chargingIndicator: "充電指示",
    chargingPercent: "百分比",
    chargingTime: "剩餘時間",
    chargingBoth: "兩者都顯示",
    osdVolume: "音量 HUD",
    osdBrightness: "亮度 HUD",
    osdBacklight: "鍵盤背光 HUD",
    notificationsComingSoon: "通知功能尚未推出（原版也還沒有）。",
    shortcutOpenSettings: "開啟設定",
    shortcutTogglePanel: "顯示／隱藏面板",
    shortcutPlayPause: "播放／暫停",
    shortcutNext: "下一首",
    shortcutPrev: "上一首",
    aboutVersion: "版本",
    aboutRepo: "原始碼",
    aboutCleanRoom: "Rust + Tauri 2 乾淨重寫，與 TheBoredTeam 無關。",
    aboutGatekeeper: "未簽名：首次開啟請右鍵「打開」，或到「系統設定 → 隱私權與安全性」按「仍要打開」。",
    settingsSaved: "已儲存",
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
    settingsNote: "按 ⚙ 打开设置。",
    footerNote:
      "NotchPilot 0.1.0 · Rust + Tauri 2 · 设置请按 ⚙",
    secNotch: "刘海",
    secAppearance: "外观",
    secMedia: "媒体",
    secCalendar: "日历",
    secShelf: "文件架",
    secMirror: "镜像",
    secBattery: "电池",
    secOsd: "屏幕显示",
    secNotifications: "通知",
    secShortcuts: "快捷键",
    secAbout: "关于",
    notchIdleWidth: "闲置宽度（0 = 自动）",
    notchExpandedWidth: "展开宽度",
    notchExpandedHeight: "展开高度",
    notchNonNotchHeight: "无刘海屏幕高度",
    notchHoverOpen: "滑入打开延迟（毫秒）",
    notchHoverClose: "滑出关闭延迟（毫秒）",
    animationSpeed: "动画速度",
    animSlow: "慢",
    animNormal: "一般",
    animFast: "快",
    showOnAllDisplays: "在所有显示器显示",
    followActiveDisplay: "跟随使用中的显示器",
    accentColor: "强调色",
    showEyes: "闲置时显示眼睛",
    theme: "主题",
    themeSystem: "系统",
    themeDark: "深色",
    themeLight: "浅色",
    visualizer: "音频可视化",
    visualizerStyle: "可视化样式",
    visualizerBars: "长条",
    visualizerCircular: "圆形",
    visualizerWave: "波形",
    vizAnimated: "动画可视化",
    vizNote:
      "实时频谱使用 ScreenCaptureKit 抓取系统音频，首次播放时会要求「屏幕录制」权限；若未授权，会改以动画呈现（非实时音频）。",
    lyrics: "同步歌词",
    audioOutput: "音频输出",
    audioOutputSystem: "系统默认",
    audioOutputLabel: "音频输出设备",
    seek: "跳转",
    mediaGestures: "媒体手势",
    showCalendars: "显示的日历（空白 = 全部）",
    reminders: "提醒事项",
    airdrop: "AirDrop",
    openOnDrop: "拖入文件时打开文件架",
    keepFiles: "保留文件",
    mirrorEnabled: "启用相机镜像",
    mirrorShape: "镜像形状",
    shapeCircle: "圆形",
    shapeRounded: "圆角矩形",
    camera: "相机（空白 = 默认）",
    showPercent: "显示电量百分比",
    chargingIndicator: "充电指示",
    chargingPercent: "百分比",
    chargingTime: "剩余时间",
    chargingBoth: "两者都显示",
    osdVolume: "音量 HUD",
    osdBrightness: "亮度 HUD",
    osdBacklight: "键盘背光 HUD",
    notificationsComingSoon: "通知功能尚未推出（原版也还没有）。",
    shortcutOpenSettings: "打开设置",
    shortcutTogglePanel: "显示／隐藏面板",
    shortcutPlayPause: "播放／暂停",
    shortcutNext: "下一首",
    shortcutPrev: "上一首",
    aboutVersion: "版本",
    aboutRepo: "源代码",
    aboutCleanRoom: "Rust + Tauri 2 干净重写，与 TheBoredTeam 无关。",
    aboutGatekeeper: "未签名：首次打开请右键「打开」，或到「系统设置 → 隐私权与安全性」按「仍要打开」。",
    settingsSaved: "已保存",
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
    settingsNote: "Press ⚙ to open Settings.",
    footerNote:
      "NotchPilot 0.1.0 · Rust + Tauri 2 · Press ⚙ for Settings",
    secNotch: "Notch",
    secAppearance: "Appearance",
    secMedia: "Media",
    secCalendar: "Calendar",
    secShelf: "Shelf",
    secMirror: "Mirror",
    secBattery: "Battery",
    secOsd: "On-Screen Display",
    secNotifications: "Notifications",
    secShortcuts: "Shortcuts",
    secAbout: "About",
    notchIdleWidth: "Idle width (0 = auto)",
    notchExpandedWidth: "Expanded width",
    notchExpandedHeight: "Expanded height",
    notchNonNotchHeight: "Height on non-notch displays",
    notchHoverOpen: "Hover to open delay (ms)",
    notchHoverClose: "Hover to close delay (ms)",
    animationSpeed: "Animation speed",
    animSlow: "Slow",
    animNormal: "Normal",
    animFast: "Fast",
    showOnAllDisplays: "Show on all displays",
    followActiveDisplay: "Follow the active display",
    accentColor: "Accent colour",
    showEyes: "Show simple eyes while idle",
    theme: "Theme",
    themeSystem: "System",
    themeDark: "Dark",
    themeLight: "Light",
    visualizer: "Audio visualizer",
    visualizerStyle: "Visualizer style",
    visualizerBars: "Bars",
    visualizerCircular: "Circular",
    visualizerWave: "Wave",
    vizAnimated: "Animated visualizer",
    vizNote:
      "The live spectrum taps system audio via ScreenCaptureKit and asks for Screen Recording permission on first play; without it, the bars fall back to animation (not live audio).",
    lyrics: "Synced lyrics",
    audioOutput: "Audio output",
    audioOutputSystem: "System default",
    audioOutputLabel: "Audio output device",
    seek: "Seek",
    mediaGestures: "Media gestures",
    showCalendars: "Calendars to show (blank = all)",
    reminders: "Reminders",
    airdrop: "AirDrop",
    openOnDrop: "Open shelf when a file is dropped",
    keepFiles: "Keep files",
    mirrorEnabled: "Enable camera mirror",
    mirrorShape: "Mirror shape",
    shapeCircle: "Circle",
    shapeRounded: "Rounded rectangle",
    camera: "Camera (blank = default)",
    showPercent: "Show battery percentage",
    chargingIndicator: "Charging indicator",
    chargingPercent: "Percentage",
    chargingTime: "Time remaining",
    chargingBoth: "Both",
    osdVolume: "Volume HUD",
    osdBrightness: "Brightness HUD",
    osdBacklight: "Keyboard backlight HUD",
    notificationsComingSoon: "Notifications are not available yet (the original does not have them either).",
    shortcutOpenSettings: "Open Settings",
    shortcutTogglePanel: "Show / hide panel",
    shortcutPlayPause: "Play / pause",
    shortcutNext: "Next track",
    shortcutPrev: "Previous track",
    aboutVersion: "Version",
    aboutRepo: "Source code",
    aboutCleanRoom: "Clean-room Rust + Tauri 2 rewrite, not affiliated with TheBoredTeam.",
    aboutGatekeeper: "Unsigned: on first launch right-click > Open, or go to System Settings > Privacy & Security > Open Anyway.",
    settingsSaved: "Saved",
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
