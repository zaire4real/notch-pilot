# NotchPilot

> **其他語言：** [簡體中文](README.zh-CN.md) · [English](README.en.md)

把 MacBook 的劉海變成可展開的動態島 — 用 Rust + Tauri 2 重寫的 MVP。

> **這是一個乾淨重寫（clean-room rewrite）的最小可行版本。**
> 它只根據 boringNotch 的「行為描述」重做：滑鼠懸停劉海展開、音樂控制、時間／電量、檔案架。
> 本專案**沒有複製任何 boringNotch 的 Swift 原始碼**，也與 TheBoredTeam 無關。
> 原專案是 Swift / SwiftUI + AppKit、GPL-3.0；本專案是 Rust + Tauri 2、MIT。

## 語言

軟體 UI 支援三種語言：**繁體中文（zh-TW，預設）、簡體中文（zh-CN）、英文（en）**。

- 第一次打開時，會先看系統語言：是繁體中文就用繁體中文，是其他中文就用簡體中文，否則用英文。
- 展開面板上方有語言切換器（繁體中文 / 简体中文 / English），點一下就即時切換全部文字，不用重開 App。
- 你的選擇會存在 `localStorage`，下次打開還是用你選的語言。
- 所有 UI 字串都集中在 `src/i18n.js`，HTML / `main.js` 裡沒有硬編碼的中文。

## 這個 MVP 能做什麼

| 功能 | 狀態 |
|---|---|
| 劉海 overlay：無邊框、透明、置頂、開機顯示在螢幕上方中央 | ✅ 有 |
| 懸停展開 / 離開收起：收起約 199×36，展開約 680×200（依劉海幾何計算） | ✅ 有 |
| 讀取劉海幾何：`safeAreaInsets` + `auxiliaryTopLeftArea` / `auxiliaryTopRightArea`，無劉海時退回置中 pill | ✅ 有（macOS） |
| 時間／日期顯示 | ✅ 有 |
| 電量：解析 `pmset -g batt`，顯示百分比、充電中／使用電池 | ✅ 有（macOS） |
| Now Playing：顯示曲名／歌手，播放／暫停／上一首／下一首 | ✅ 有（見下方限制） |
| 檔案架：把檔案拖到展開面板上，會列出檔名清單 | ✅ 有（僅列檔名） |
| 三語 UI：繁體中文／簡體中文／英文，即時切換 | ✅ 有 |
| 真實音訊視覺化（頻譜） | ❌ MVP 沒有 |
| AirDrop | ❌ MVP 沒有（只列出拖進來的檔名，不會真的傳送） |
| 相機鏡像 | ❌ MVP 沒有 |
| 日曆／提醒事項整合 | ❌ MVP 沒有 |
| 取代系統音量／亮度 HUD（按鍵攔截） | ❌ MVP 沒有 |
| 鎖定畫面小工具、天氣、通知 | ❌ MVP 沒有 |

## Now Playing 的限制（請先看）

MVP **沒有用私有的 MediaRemote framework**。

做法是 AppleScript（`osascript`）：
1. 先問 Music.app 是否正在播放；
2. 如果沒有，再問 Spotify 是否正在播放；
3. 都沒有就回傳「目前沒有播放」。

所以：
- 只支援 Music.app 和 Spotify，其他播放器（YouTube Music、網頁播放器等）不會顯示。
- 需要在「系統設定 → 隱私權與安全性 → 自動化」允許本 App 控制 Music / Spotify。
- 曲名／歌手是輪詢取得的，不是即時推播。

## 系統需求（在你的 MacBook M3 上）

- macOS 14 Sonoma 或以上（Apple Silicon / M3）
- Xcode Command Line Tools：`xcode-select --install`
- Rust（用 rustup 安裝）：https://rustup.rs
  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  ```
  裝完重開終端機，確認 `cargo --version` 有輸出。
- Node.js 20 以上（本機驗證時為 Node v24.22.1）
- Tauri CLI 是用 `npm install` 裝在本專案的 devDependency，不用全域安裝。

> 注意：本專案是在 Linux VM 上寫的，**沒有在 macOS 上編譯或執行過**。
> 下面的指令是在你的 MacBook M3 上跑的，第一次 `npm run tauri dev` 會花幾分鐘編譯 Rust。

## 安裝與執行

```bash
cd ~/workspace/notch-pilot
# 或把整個資料夾複製到你的 Mac 上之後 cd 進去

npm install

# 開發模式：開一個 overlay 視窗，改前端即時看得到
npm run tauri dev

# 打包成 .app / .dmg（輸出在 src-tauri/target/release/bundle/）
npm run tauri build
```

`npm run tauri build` 產出的東西在：

```
src-tauri/target/release/bundle/dmg/NotchPilot_0.1.0_aarch64.dmg
src-tauri/target/release/bundle/macos/NotchPilot.app
```

## Gatekeeper 提醒

這個 App 沒有 Apple Developer 簽名（跟原版 boringNotch 一樣），第一次打開 macOS 會擋：

1. 打開後看到「無法打開，因為無法驗證開發者」是正常的，不是惡意軟體。
2. 到「系統設定 → 隱私權與安全性」，拉到最下面，點 NotchPilot 旁邊的「仍要打開」。
3. 之後再開一次就能正常用。

如果你的公司電腦只允許已公證（notarized）的 App，請先問 IT，不要硬裝。

## 怎麼用

- **收起狀態**：螢幕上方中央一個黑色小 pill，顯示時間和電量。
- **展開**：滑鼠移到 pill 上就會展開，移開就收起。
- **語言**：展開後點面板上方的「繁體中文 / 简体中文 / English」，即時切換。
- **音樂**：展開後看到曲名／歌手，按 ▶ / ⏸、⏮、⏭ 控制 Music 或 Spotify。
- **檔案架**：把檔案拖到展開的面板上，檔名會出現在「檔案架」區。MVP 只列檔名，不會移動、複製或 AirDrop 檔案。
- **設定**：面板下方有說明，進階的劉海高度／手勢目前 MVP 沒有設定頁。

## 疑難排解

| 問題 | 做法 |
|---|---|
| `npm run tauri dev` 說找不到 `cargo` | 重開終端機，或 `source "$HOME/.cargo/env"`，再跑 `cargo --version` |
| `npm run tauri dev` 說找不到 Xcode / `xcode-select` | 跑 `xcode-select --install`，裝完再試 |
| 音樂一直顯示「目前沒有播放」 | 先在 Music 或 Spotify 真的按下播放；再到「系統設定 → 隱私權與安全性 → 自動化」允許本 App 控制該 App |
| 視窗沒有出現在劉海位置 | 檢查是不是外接螢幕為主螢幕；MVP 只讀主螢幕（`NSScreen::main`）的劉海幾何 |
| 沒有劉海的螢幕（外接螢幕） | 會退回置中 pill 模式，功能一樣，只是沒有劉海形狀 |
| 語言切換後有文字沒變 | 那是 bug：所有字串都應該在 `src/i18n.js` 裡，請回報是哪一個字串沒翻到 |
| `tauri dev` 編譯很久 | 第一次正常，Rust 要編 Tauri 和 objc2，之後會快很多 |

## 專案結構

```
notch-pilot/
├── README.md                   # 繁體中文版（主版）
├── README.zh-CN.md             # 簡體中文版
├── README.en.md                # 英文版
├── LICENSE                     # MIT
├── .gitignore                  # target/、node_modules/、.DS_Store 等
├── package.json                # Tauri 2 CLI，純前端、無框架打包步驟
├── src/
│   ├── index.html              # 劉海面板 UI（無硬編碼文字）
│   ├── i18n.js                 # 三語字典：zh-TW / zh-CN / en
│   ├── styles.css              # 黑色 pill / 展開面板樣式
│   └── main.js                 # 呼叫 Tauri commands、拖放檔案架、語言切換
└── src-tauri/
    ├── Cargo.toml              # tauri 2、serde；objc2 只在 macOS 目標
    ├── tauri.conf.json         # 透明、無邊框、置頂、跳過 Dock
    ├── capabilities/
    │   └── default.json        # 最小權限
    ├── build.rs                # tauri_build::build()
    └── src/
        ├── main.rs             # 進入點，呼叫 lib::run()
        ├── lib.rs              # Tauri builder、註冊 commands、setup 呼叫 notch
        ├── notch.rs            # 劉海幾何（objc2 / AppKit）+ 展開／收起
        ├── media.rs            # Now Playing（AppleScript → Music / Spotify）
        └── system.rs           # 電量（pmset）、時間
```

## 技術選擇：為什麼是 Tauri 2？

- 最快能交出可用的版本：Rust 寫後端，UI 用原生 HTML/CSS/JS，不用再學一整套 SwiftUI。
- 劉海 overlay 的關鍵是 AppKit：Rust 端用 `objc2` / `objc2-app-kit` / `objc2-foundation` 讀 `NSScreen` 的 `safeAreaInsets` 和 `auxiliaryTopLeftArea` / `auxiliaryTopRightArea`，算出劉海寬度和展開尺寸；沒有劉海就退回置中 pill。
- macOS 沒有公開的 Dynamic Island API，所以跟原版一樣，本專案也是自己做一個 borderless overlay 視窗。

## 路線圖

- [ ] 真實音訊視覺化（用系統音訊取樣，不是假動畫）
- [ ] 日曆／提醒事項：讀 EventKit（需要授權）
- [ ] 相機鏡像：AVFoundation 預覽
- [ ] 檔案架真的能 AirDrop / 打開檔案
- [ ] 音量／亮度 HUD 取代（需要攔截系統按鍵，風險較高）
- [ ] 設定頁：劉海高度、手勢、要不要顯示在所有螢幕
- [ ] 更多語言（視需求）
- [ ] 正式圖示（`src-tauri/icons/`）與簽名／公證

## 發布到 GitHub

本專案已上線，公開 repo 在 <https://github.com/zaire4real/notch-pilot>。

```bash
git clone https://github.com/zaire4real/notch-pilot.git
cd notch-pilot
npm install
npm run tauri dev
```

發布前請確認：`.gitignore` 已排除 `target/`、`node_modules/`、`.DS_Store`；`LICENSE`、三份 README（`README.md`、`README.zh-CN.md`、`README.en.md`）都在根目錄；不要把編譯出來的 `.app` / `.dmg` 推上去（它們在 `src-tauri/target/`，已被忽略）。

## 授權

MIT，見 `LICENSE`。本專案不包含 boringNotch 的原始碼，也不是 TheBoredTeam 的官方專案。
