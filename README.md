# NotchPilot

> **其他語言：** [簡體中文](README.zh-CN.md) · [English](README.en.md)

把 MacBook 的劉海變成可展開的動態島 — 用 Rust + Tauri 2 重寫。

> **這是一個乾淨重寫（clean-room rewrite）。**
> 它只根據 boringNotch 的「行為描述」重做功能與版面：滑鼠懸停劉海展開、音樂控制、時間／電量、檔案架，v0.2.0 起還包含專輯封面、音訊視覺化、日曆、相機鏡像、AirDrop、音量／亮度 HUD 與設定視窗。
> 本專案**沒有複製任何 boringNotch 的 Swift 原始碼**，也與 TheBoredTeam 無關。
> 原專案是 Swift / SwiftUI + AppKit、GPL-3.0；本專案是 Rust + Tauri 2、MIT。

## v0.2.1 更新日誌

v0.2.1：修復 v0.2.0 在真的 MacBook 上完全無法啟動的問題（見下方說明），其餘功能與 v0.2.0 相同。

- **啟動修復**：v0.2.0 在 MacBook Air M3 上一啟動就被 dyld 擋下（`Library not loaded: @rpath/libswift_Concurrency.dylib … no LC_RPATH's found`），連視窗都不會出現。原因是 v0.2.0 的音訊視覺化用的 `screencapturekit` 套件會編譯 Swift 橋接程式碼，執行檔因此連結了 Swift 執行環境；但 macOS 把核心 Swift 執行環境放在 dyld 共享快取裡，硬碟上的 `/usr/lib/swift/` 只有 Photos、Network 等少數檔案，補 RPATH 也找不到，所以 v0.2.1 直接拿掉 `screencapturekit` 依賴，程式不再連結 Swift。
- **音訊視覺化改為動畫**：即日起只顯示標示為「動畫」的模擬頻譜（8 條長條），不再假裝是真實音訊。真實頻譜要等以後用純 objc2 的 ScreenCaptureKit 實作（不經過 Swift）才能回來。
- 其餘功能、版面與設定與 v0.2.0 相同。

> 注意：v0.2.0 從未成功啟動過，請勿使用 v0.2.0 的 .dmg，改用 v0.2.1。

## v0.2.0 更新日誌（⚠️ 此版無法啟動，請用 v0.2.1）

v0.2.0：功能與版面皆對齊原版 boringNotch：專輯封面、播放進度、音訊視覺化、日曆與提醒事項、相機鏡像、檔案架（含 AirDrop）、音量／亮度 HUD、藍牙動態、手勢與劉海尺寸設定；語言選項移至設定視窗。

新增與變更一覽：

- **媒體**：專輯封面、播放進度條＋快進／快退、同步歌詞（LRCLIB，預設關閉）、音訊輸出選擇（喇叭／耳機／AirPods／藍牙）。
- **音訊視覺化**：原本計畫用 ScreenCaptureKit 取系統音訊做真實 8 頻段頻譜，但該做法需要 Swift 執行環境，導致 v0.2.0 完全無法啟動；v0.2.1 已改為只顯示標示為「動畫」的模擬頻譜（見上方 v0.2.1 更新日誌）。
- **版面**：展開後順序改為音樂 → 日曆 → 底部（相機鏡像＋檔案架入口）；檔案架改為獨立畫面；收起時不再顯示時鐘（只顯示動態內容與電量）；閒置時可開啟「眼睛」動畫（預設關閉）；音量／切歌等短暫訊息改用約 70px 的 sneak-peek 顯示，約 2 秒後自動收起。
- **日曆／提醒事項**：顯示今日行程、下一場倒數、明天的第一場、未完成提醒事項（可勾選完成）。用 AppleScript 讀 Calendar.app / Reminders.app，需要在「自動化」授權。
- **檔案架**：檔案會存檔，重開 App 還在；點一下用預設 App 打開；可移除。AirDrop 可把檔案傳給附近裝置（成功是指分享選單有打開）。
- **相機鏡像**：AVFoundation 擷取相機畫面（640×480、約 11fps），可在設定選圓形／圓角。
- **HUD**：音量、亮度變化會在劉海以 sneak-peek 顯示；系統自己的音量／亮度提示不會被關掉。鍵盤背光尚未支援。
- **手勢**：觸控板滑動開合面板、橫向滑動切歌。
- **設定視窗**：獨立視窗，11 個分區（Notch、Appearance、Media、Calendar、Shelf、Mirror、Battery、OSD、Notifications、Shortcuts、About）；語言選項在 Notch 分區最上方；選單列圖示（點一下開設定、右鍵選單）、Cmd+, 也可開設定；設定存在 `settings.json`，動畫速度、劉海尺寸等都可調。
- **多螢幕**：單一視窗跟隨使用中的螢幕（非每螢幕一個視窗）。
- **其他**：藍牙裝置連線動態、番茄鐘、天氣（Open-Meteo，預設關閉、預設城市台北；天氣是 NotchPilot 自加的功能，原版沒有）。

## 語言

軟體 UI 支援三種語言：**繁體中文（zh-TW，預設）、簡體中文（zh-CN）、英文（en）**。

- 第一次打開時，會先看系統語言：是繁體中文就用繁體中文，是其他中文就用簡體中文，否則用英文。
- 語言在**設定視窗**（按面板上的 ⚙、選單列圖示或 Cmd+, 開啟）→「Notch」分區最上方選擇（繁體中文 / 简体中文 / English），點一下就即時切換全部文字，不用重開 App。
- 你的選擇會存在 `settings.json`（`~/Library/Application Support/com.yichia.notch-pilot/settings.json`），下次打開還是用你選的語言。
- 所有 UI 字串都集中在 `src/i18n.js`，HTML / `main.js` 裡沒有硬編碼的中文。

## 這個版本能做什麼

| 功能 | 狀態 |
|---|---|
| 劉海 overlay：無邊框、透明、置頂、開機顯示在螢幕上方中央 | ✅ 有 |
| 懸停展開 / 離開收起：收起為劉海寬×36，展開 680×360（可在設定調整） | ✅ 有 |
| 讀取劉海幾何：`safeAreaInsets` + `auxiliaryTopLeftArea` / `auxiliaryTopRightArea`，無劉海時退回置中 pill | ✅ 有（macOS） |
| 時間／日期顯示 | ✅ 有（展開面板內；收起時不顯示時鐘） |
| 電量：解析 `pmset -g batt`，顯示百分比、充電中／使用電池、剩餘時間 | ✅ 有（macOS） |
| Now Playing：專輯封面、曲名／歌手、播放進度、播放／暫停／上一首／下一首、快進／快退 | ✅ 有（見下方限制） |
| 音訊視覺化（動畫頻譜） | ✅ 有（動畫長條，明確標示為動畫；真實頻譜因 Swift 執行環境問題已在 v0.2.1 移除，待純 objc2 實作） |
| 同步歌詞（LRCLIB） | ✅ 有（預設關閉） |
| 音訊輸出選擇 | ✅ 有（喇叭／耳機／AirPods／藍牙） |
| 日曆／提醒事項整合 | ✅ 有（AppleScript 讀 Calendar.app / Reminders.app） |
| 相機鏡像 | ✅ 有（AVFoundation，640×480、約 11fps） |
| 檔案架：拖入檔案會存檔、點開、移除 | ✅ 有 |
| AirDrop | ✅ 有（分享選單開啟即視為成功） |
| 音量／亮度 HUD（在劉海顯示） | ✅ 有（偵測＋顯示；不會關掉系統自己的提示；鍵盤背光未支援） |
| 手勢：滑動開合、橫滑切歌 | ✅ 有 |
| 設定視窗（11 個分區）、選單列圖示、Cmd+, | ✅ 有 |
| 多螢幕：跟隨使用中的螢幕 | ✅ 有（單一視窗，非每螢幕一個視窗） |
| 藍牙裝置連線動態、番茄鐘 | ✅ 有 |
| 天氣（Open-Meteo） | ✅ 有（NotchPilot 自加功能，預設關閉、預設城市台北） |
| 三語 UI：繁體中文／簡體中文／英文，即時切換 | ✅ 有（在設定視窗切換） |
| 鎖定畫面小工具、通知 | ❌ 沒有（原版也還沒做） |

## Now Playing 的限制（請先看）

本專案**沒有用私有的 MediaRemote framework**。

做法是 AppleScript（`osascript`）：
1. 先問 Music.app 是否正在播放；
2. 如果沒有，再問 Spotify 是否正在播放；
3. 都沒有就回傳「目前沒有播放」。

所以：
- 只支援 Music.app 和 Spotify，其他播放器（YouTube Music、網頁播放器等）不會顯示。
- 需要在「系統設定 → 隱私權與安全性 → 自動化」允許本 App 控制 Music / Spotify。
- 曲名／歌手是輪詢取得的，不是即時推播。
- 專輯封面從 AppleScript 讀取；同步歌詞從 LRCLIB 免費 API 抓（預設關閉，在設定 → Media 開啟）；音訊輸出選擇用 CoreAudio 切換裝置。

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

> 注意：本專案是在 Linux VM 上寫的，macOS 建置是在 GitHub Actions 的 macOS runner 上跑的。v0.2.0 已在真的 MacBook Air M3 上試過，但一啟動就被 dyld 擋下（Swift 執行環境問題）；v0.2.1 拿掉了 Swift 依賴，但尚未在真的 MacBook 上確認能打開。
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
src-tauri/target/release/bundle/dmg/NotchPilot_0.2.1_aarch64.dmg
src-tauri/target/release/bundle/macos/NotchPilot.app
```

## Gatekeeper 提醒

這個 App 沒有 Apple Developer 簽名（跟原版 boringNotch 一樣），第一次打開 macOS 會擋：

1. 打開後看到「無法打開，因為無法驗證開發者」是正常的，不是惡意軟體。
2. 到「系統設定 → 隱私權與安全性」，拉到最下面，點 NotchPilot 旁邊的「仍要打開」。
3. 之後再開一次就能正常用。

如果你的公司電腦只允許已公證（notarized）的 App，請先問 IT，不要硬裝。

## 怎麼用

- **收起狀態**：螢幕上方中央一個黑色小 pill，顯示動態內容（播放中的專輯小圖／視覺化、充電狀態）和電量；不顯示時鐘（時間請看選單列或展開面板）。
- **展開**：滑鼠移到 pill 上就會展開，移開就收起；展開後順序是音樂 → 日曆 → 底部（相機鏡像＋檔案架入口）。
- **語言**：在設定視窗（按面板上的 ⚙、選單列圖示或 Cmd+, 開啟）→「Notch」分區最上方切換，即時生效。
- **音樂**：展開後看到專輯封面、曲名／歌手、播放進度，按 ▶ / ⏸、⏮、⏭ 控制 Music 或 Spotify；進度條可快進／快退。
- **檔案架**：把檔案拖到展開的面板上，檔案會存檔，重開 App 還在；點一下用預設 App 打開，右鍵或按 × 移除；用 AirDrop 按鈕可傳給附近裝置。
- **設定**：獨立設定視窗，11 個分區，可調劉海尺寸、動畫速度、手勢、視覺化、鏡像形狀等；設定存在 `settings.json`。

## 疑難排解

| 問題 | 做法 |
|---|---|
| `npm run tauri dev` 說找不到 `cargo` | 重開終端機，或 `source "$HOME/.cargo/env"`，再跑 `cargo --version` |
| `npm run tauri dev` 說找不到 Xcode / `xcode-select` | 跑 `xcode-select --install`，裝完再試 |
| 音樂一直顯示「目前沒有播放」 | 先在 Music 或 Spotify 真的按下播放；再到「系統設定 → 隱私權與安全性 → 自動化」允許本 App 控制該 App |
| 視窗沒有出現在劉海位置 | 檢查是不是外接螢幕為主螢幕；本版是單一視窗跟隨使用中的螢幕 |
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
│   ├── main.js                 # 呼叫 Tauri commands、拖放檔案架、語言切換
│   ├── settings.html           # 設定視窗 UI
│   ├── settings.js             # 設定視窗邏輯
│   └── settings.css            # 設定視窗樣式
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
        ├── system.rs           # 電量（pmset）、時間
        ├── settings.rs         # 設定檔（settings.json）讀寫
        ├── audio.rs            # 音訊輸出裝置（CoreAudio）
        ├── visualizer.rs       # 音訊視覺化（動畫長條；真實頻譜已移除）
        ├── calendar.rs         # 日曆／提醒事項（AppleScript）
        ├── airdrop.rs          # AirDrop（NSSharingService）
        ├── mirror.rs           # 相機鏡像（AVFoundation）
        ├── hud.rs              # 音量／亮度偵測
        ├── displays.rs         # 多螢幕跟隨
        └── bluetooth.rs        # 藍牙裝置狀態
```

## 技術選擇：為什麼是 Tauri 2？

- 最快能交出可用的版本：Rust 寫後端，UI 用原生 HTML/CSS/JS，不用再學一整套 SwiftUI。
- 劉海 overlay 的關鍵是 AppKit：Rust 端用 `objc2` / `objc2-app-kit` / `objc2-foundation` 讀 `NSScreen` 的 `safeAreaInsets` 和 `auxiliaryTopLeftArea` / `auxiliaryTopRightArea`，算出劉海寬度和展開尺寸；沒有劉海就退回置中 pill。
- macOS 沒有公開的 Dynamic Island API，所以跟原版一樣，本專案也是自己做一個 borderless overlay 視窗。

## 路線圖

v0.2.0 / v0.2.1 已完成（見上方更新日誌）：

- [x] 音訊視覺化（動畫長條，標示為動畫；真實頻譜因 Swift 執行環境問題已移除，待純 objc2 實作）
- [x] 日曆／提醒事項（AppleScript）
- [x] 相機鏡像（AVFoundation）
- [x] 檔案架存檔、打開檔案、AirDrop
- [x] 音量／亮度 HUD 顯示（偵測＋劉海顯示；不攔截系統提示）
- [x] 設定視窗：劉海尺寸、手勢、多螢幕跟隨、動畫速度
- [x] 藍牙動態、番茄鐘、天氣（預設關閉）

尚未做：

- [ ] 真實音訊頻譜（純 objc2 ScreenCaptureKit 實作，不經過 Swift）
- [ ] 鎖定畫面小工具、通知（原版也還沒做）
- [ ] 正式圖示（`src-tauri/icons/`）與簽名／公證
- [ ] 更多語言（視需求）

## 發布到 GitHub

本專案已上線，公開 repo 在 <https://github.com/zaire4real/notch-pilot>。

```bash
git clone https://github.com/zaire4real/notch-pilot.git
cd notch-pilot
npm install
npm run tauri dev
```

發布前請確認：`.gitignore` 已排除 `target/`、`node_modules/`、`.DS_Store`；`LICENSE`、三份 README（`README.md`、`README.zh-CN.md`、`README.en.md`）都在根目錄；不要把編譯出來的 `.app` / `.dmg` 推上去（它們在 `src-tauri/target/`，已被忽略）。

## 下載 .dmg（不用自己編譯）

直接下載就好，不用裝 Rust / Node 來編譯：

**下載位置：<https://github.com/zaire4real/notch-pilot/releases>**

1. 打開上面的 Releases 頁面，點最新的 release（目前是 **v0.2.1**）。
2. 在 **Assets** 裡下載 `NotchPilot_0.2.1_aarch64.dmg`。
3. 雙擊安裝，或拖到「應用程式」。

> 這個 `.dmg` **沒有簽名**（沒有 Apple 開發者帳號）。第一次打開時 macOS 會擋，請用右鍵 →「打開」，或到「系統設定 → 隱私權與安全性」點「仍要打開」。

備用方式：如果 Releases 還沒有你要的版本，可以到 repo 的 **Actions** 頁面 → **Build macOS DMG** → 選最新一次成功的執行，在最下面的 **Artifacts** 下載 **NotchPilot-macos-aarch64-dmg**（要登入 GitHub 才能下載，artifact 也會過期）。

## 授權

MIT，見 `LICENSE`。本專案不包含 boringNotch 的原始碼，也不是 TheBoredTeam 的官方專案。
