# NotchPilot

> **其他语言：** [繁體中文](README.md) · [English](README.en.md)

把 MacBook 的刘海变成可展开的动态岛 — 用 Rust + Tauri 2 重写。

> **这是一个干净重写（clean-room rewrite）。**
> 它只根据 boringNotch 的「行为描述」重做功能与版面：鼠标悬停刘海展开、音乐控制、时间／电量、文件架，v0.2.0 起还包含专辑封面、音频可视化、日历、相机镜像、AirDrop、音量／亮度 HUD 与设置窗口。
> 本项目**没有复制任何 boringNotch 的 Swift 原始代码**，也与 TheBoredTeam 无关。
> 原项目是 Swift / SwiftUI + AppKit、GPL-3.0；本项目是 Rust + Tauri 2、MIT。

## v0.2.1 更新日志

v0.2.1：修复 v0.2.0 在真的 MacBook 上完全无法启动的问题（见下方说明），其余功能与 v0.2.0 相同。

- **启动修复**：v0.2.0 在 MacBook Air M3 上一启动就被 dyld 拦下（`Library not loaded: @rpath/libswift_Concurrency.dylib … no LC_RPATH's found`），连窗口都不会出现。原因是 v0.2.0 的音频可视化用的 `screencapturekit` 包会编译 Swift 桥接代码，执行文件因此链接了 Swift 运行环境；但 macOS 把核心 Swift 运行环境放在 dyld 共享缓存里，硬盘上的 `/usr/lib/swift/` 只有 Photos、Network 等少数文件，补 RPATH 也找不到，所以 v0.2.1 直接拿掉 `screencapturekit` 依赖，程序不再链接 Swift。
- **音频可视化改为动画**：即日起只显示标示为「动画」的模拟频谱（8 条长条），不再假装是真实音频。真实频谱要等以后用纯 objc2 的 ScreenCaptureKit 实现（不经过 Swift）才能回来。
- 其余功能、版面与设置与 v0.2.0 相同。

> 注意：v0.2.0 从未成功启动过，请勿使用 v0.2.0 的 .dmg，改用 v0.2.1。

## v0.2.0 更新日志（⚠️ 此版无法启动，请用 v0.2.1）

v0.2.0：功能与版面皆对齐原版 boringNotch：专辑封面、播放进度、音频可视化、日历与提醒事项、相机镜像、文件架（含 AirDrop）、音量／亮度 HUD、蓝牙动态、手势与刘海尺寸设定；语言选项移至设置窗口。

新增与变更一览：

- **媒体**：专辑封面、播放进度条＋快进／快退、同步歌词（LRCLIB，默认关闭）、音频输出选择（喇叭／耳机／AirPods／蓝牙）。
- **音频可视化**：原本计划用 ScreenCaptureKit 取系统音频做真实 8 频段频谱，但该做法需要 Swift 运行环境，导致 v0.2.0 完全无法启动；v0.2.1 已改为只显示标示为「动画」的模拟频谱（见上方 v0.2.1 更新日志）。
- **版面**：展开后顺序改为音乐 → 日历 → 底部（相机镜像＋文件架入口）；文件架改为独立画面；收起时不再显示时钟（只显示动态内容与电量）；闲置时可开启「眼睛」动画（默认关闭）；音量／切歌等短暂讯息改用约 70px 的 sneak-peek 显示，约 2 秒后自动收起。
- **日历／提醒事项**：显示今日行程、下一场倒数、明天的第一场、未完成提醒事项（可勾选完成）。用 AppleScript 读 Calendar.app / Reminders.app，需要在「自动化」授权。
- **文件架**：文件会存档，重开 App 还在；点一下用默认 App 打开；可移除。AirDrop 可把文件传给附近装置（成功是指分享选单有打开）。
- **相机镜像**：AVFoundation 撷取相机画面（640×480、约 11fps），可在设定选圆形／圆角。
- **HUD**：音量、亮度变化会在刘海以 sneak-peek 显示；系统自己的音量／亮度提示不会被关掉。键盘背光尚未支持。
- **手势**：触控板滑动开合面板、横向滑动切歌。
- **设置窗口**：独立窗口，11 个分区（Notch、Appearance、Media、Calendar、Shelf、Mirror、Battery、OSD、Notifications、Shortcuts、About）；语言选项在 Notch 分区最上方；选单列图标（点一下开设定、右键选单）、Cmd+, 也可开设定；设定存在 `settings.json`，动画速度、刘海尺寸等都可调。
- **多屏幕**：单一窗口跟随使用中的屏幕（非每屏幕一个窗口）。
- **其他**：蓝牙装置连接动态、番茄钟、天气（Open-Meteo，默认关闭、默认城市台北；天气是 NotchPilot 自加的功能，原版没有）。

## 语言

软件 UI 支持三种语言：**繁体中文（zh-TW，默认）、简体中文（zh-CN）、英文（en）**。

- 第一次打开时，会先看系统语言：是繁体中文就用繁体中文，是其他中文就用简体中文，否则用英文。
- 语言在**设置窗口**（按面板上的 ⚙、选单列图标或 Cmd+, 开启）→「Notch」分区最上方选择（繁體中文 / 简体中文 / English），点一下就即时切换全部文字，不用重开 App。
- 你的选择会存在 `settings.json`（`~/Library/Application Support/com.yichia.notch-pilot/settings.json`），下次打开还是用你选的语言。
- 所有 UI 字符串都集中在 `src/i18n.js`，HTML / `main.js` 里没有硬编码的中文。

## 这个版本能做什么

| 功能 | 状态 |
|---|---|
| 刘海 overlay：无边框、透明、置顶、开机显示在屏幕上方中央 | ✅ 有 |
| 悬停展开 / 离开展起：收起为刘海宽×36，展开 680×360（可在设定调整） | ✅ 有 |
| 读取刘海几何：`safeAreaInsets` + `auxiliaryTopLeftArea` / `auxiliaryTopRightArea`，无刘海时退回置中 pill | ✅ 有（macOS） |
| 时间／日期显示 | ✅ 有（展开面板内；收起时不显示时钟） |
| 电量：解析 `pmset -g batt`，显示百分比、充电中／使用电池、剩余时间 | ✅ 有（macOS） |
| Now Playing：专辑封面、曲名／歌手、播放进度、播放／暂停／上一首／下一首、快进／快退 | ✅ 有（见下方限制） |
| 音频可视化（动画频谱） | ✅ 有（动画长条，明确标示为动画；真实频谱因 Swift 运行环境问题已在 v0.2.1 移除，待纯 objc2 实现） |
| 同步歌词（LRCLIB） | ✅ 有（默认关闭） |
| 音频输出选择 | ✅ 有（喇叭／耳机／AirPods／蓝牙） |
| 日历／提醒事项整合 | ✅ 有（AppleScript 读 Calendar.app / Reminders.app） |
| 相机镜像 | ✅ 有（AVFoundation，640×480、约 11fps） |
| 文件架：拖入文件会存档、点开、移除 | ✅ 有 |
| AirDrop | ✅ 有（分享选单开启即视为成功） |
| 音量／亮度 HUD（在刘海显示） | ✅ 有（侦测＋显示；不会关掉系统自己的提示；键盘背光未支持） |
| 手势：滑动开合、横滑切歌 | ✅ 有 |
| 设置窗口（11 个分区）、选单列图标、Cmd+, | ✅ 有 |
| 多屏幕：跟随使用中的屏幕 | ✅ 有（单一窗口，非每屏幕一个窗口） |
| 蓝牙装置连接动态、番茄钟 | ✅ 有 |
| 天气（Open-Meteo） | ✅ 有（NotchPilot 自加功能，默认关闭、默认城市台北） |
| 三语 UI：繁体中文／简体中文／英文，即时切换 | ✅ 有（在设置窗口切换） |
| 锁定画面小工具、通知 | ❌ 没有（原版也还没做） |

## Now Playing 的限制（请先看）

本项目**没有用私有的 MediaRemote framework**。

做法是 AppleScript（`osascript`）：
1. 先问 Music.app 是否正在播放；
2. 如果没有，再问 Spotify 是否正在播放；
3. 都没有就回传「目前没有播放」。

所以：
- 只支持 Music.app 和 Spotify，其他播放器（YouTube Music、网页播放器等）不会显示。
- 需要在「系统设置 → 隐私权与安全性 → 自动化」允许本 App 控制 Music / Spotify。
- 曲名／歌手是轮询取得的，不是即时推送。
- 专辑封面从 AppleScript 读取；同步歌词从 LRCLIB 免费 API 抓（默认关闭，在设定 → Media 开启）；音频输出选择用 CoreAudio 切换装置。

## 系统需求（在你的 MacBook M3 上）

- macOS 14 Sonoma 或以上（Apple Silicon / M3）
- Xcode Command Line Tools：`xcode-select --install`
- Rust（用 rustup 安装）：https://rustup.rs
  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  ```
  装完重开终端，确认 `cargo --version` 有输出。
- Node.js 20 以上（本机验证时为 Node v24.22.1）
- Tauri CLI 是用 `npm install` 装在本项目的 devDependency，不用全局安装。

> 注意：本项目是在 Linux VM 上写的，macOS 构建是在 GitHub Actions 的 macOS runner 上跑的。v0.2.0 已在真的 MacBook Air M3 上试过，但一启动就被 dyld 拦下（Swift 运行环境问题）；v0.2.1 拿掉了 Swift 依赖，但尚未在真的 MacBook 上确认能打开。
> 下面的指令是在你的 MacBook M3 上跑的，第一次 `npm run tauri dev` 会花几分钟编译 Rust。

## 安装与执行

```bash
cd ~/workspace/notch-pilot
# 或把整个文件夹复制到你的 Mac 上之后 cd 进去

npm install

# 开发模式：开一个 overlay 窗口，改前端即时看得到
npm run tauri dev

# 打包成 .app / .dmg（输出在 src-tauri/target/release/bundle/）
npm run tauri build
```

`npm run tauri build` 产出的东西在：

```
src-tauri/target/release/bundle/dmg/NotchPilot_0.2.1_aarch64.dmg
src-tauri/target/release/bundle/macos/NotchPilot.app
```

## Gatekeeper 提醒

这个 App 没有 Apple Developer 签名（跟原版 boringNotch 一样），第一次打开 macOS 会挡：

1. 打开后看到「无法打开，因为无法验证开发者」是正常的，不是恶意软件。
2. 到「系统设置 → 隐私权与安全性」，拉到最下面，点 NotchPilot 旁边的「仍要打开」。
3. 之后再开一次就能正常用。

如果你的公司电脑只允许已公证（notarized）的 App，请先问 IT，不要硬装。

## 怎么用

- **收起状态**：屏幕上方中央一个黑色小 pill，显示动态内容（播放中的专辑小图／可视化、充电状态）和电量；不显示时钟（时间请看选单列或展开面板）。
- **展开**：鼠标移到 pill 上就会展开，移开就收起；展开后顺序是音乐 → 日历 → 底部（相机镜像＋文件架入口）。
- **语言**：在设置窗口（按面板上的 ⚙、选单列图标或 Cmd+, 开启）→「Notch」分区最上方切换，即时生效。
- **音乐**：展开后看到专辑封面、曲名／歌手、播放进度，按 ▶ / ⏸、⏮、⏭ 控制 Music 或 Spotify；进度条可快进／快退。
- **文件架**：把文件拖到展开的面板上，文件会存档，重开 App 还在；点一下用默认 App 打开，右键或按 × 移除；用 AirDrop 按钮可传给附近装置。
- **设置**：独立设置窗口，11 个分区，可调刘海尺寸、动画速度、手势、可视化、镜像形状等；设定存在 `settings.json`。

## 疑难排解

| 问题 | 做法 |
|---|---|
| `npm run tauri dev` 说找不到 `cargo` | 重开终端，或 `source "$HOME/.cargo/env"`，再跑 `cargo --version` |
| `npm run tauri dev` 说找不到 Xcode / `xcode-select` | 跑 `xcode-select --install`，装完再试 |
| 音乐一直显示「目前没有播放」 | 先在 Music 或 Spotify 真的按下播放；再到「系统设置 → 隐私权与安全性 → 自动化」允许本 App 控制该 App |
| 窗口没有出现在刘海位置 | 检查是不是外接屏幕为主屏幕；本版是单一窗口跟随使用中的屏幕 |
| 没有刘海的屏幕（外接屏幕） | 会退回置中 pill 模式，功能一样，只是没有刘海形状 |
| 语言切换后有文字没变 | 那是 bug：所有字符串都应该在 `src/i18n.js` 里，请回报是哪一个字符串没翻到 |
| `tauri dev` 编译很久 | 第一次正常，Rust 要编 Tauri 和 objc2，之后会快很多 |

## 项目结构

```
notch-pilot/
├── README.md                   # 繁體中文版（主版）
├── README.zh-CN.md             # 简体中文版
├── README.en.md                # 英文版
├── LICENSE                     # MIT
├── .gitignore                  # target/、node_modules/、.DS_Store 等
├── package.json                # Tauri 2 CLI，纯前端、无框架打包步骤
├── src/
│   ├── index.html              # 刘海面板 UI（无硬编码文字）
│   ├── i18n.js                 # 三语字典：zh-TW / zh-CN / en
│   ├── styles.css              # 黑色 pill / 展开面板样式
│   ├── main.js                 # 调用 Tauri commands、拖放文件架、语言切换
│   ├── settings.html           # 设置窗口 UI
│   ├── settings.js             # 设置窗口逻辑
│   └── settings.css            # 设置窗口样式
└── src-tauri/
    ├── Cargo.toml              # tauri 2、serde；objc2 只在 macOS 目标
    ├── tauri.conf.json         # 透明、无边框、置顶、跳过 Dock
    ├── capabilities/
    │   └── default.json        # 最小权限
    ├── build.rs                # tauri_build::build()
    └── src/
        ├── main.rs             # 进入点，调用 lib::run()
        ├── lib.rs              # Tauri builder、注册 commands、setup 调用 notch
        ├── notch.rs            # 刘海几何（objc2 / AppKit）+ 展开／收起
        ├── media.rs            # Now Playing（AppleScript → Music / Spotify）
        ├── system.rs           # 电量（pmset）、时间
        ├── settings.rs         # 设定档（settings.json）读写
        ├── audio.rs            # 音频输出装置（CoreAudio）
        ├── visualizer.rs       # 音频可视化（动画长条；真实频谱已移除）
        ├── calendar.rs         # 日历／提醒事项（AppleScript）
        ├── airdrop.rs          # AirDrop（NSSharingService）
        ├── mirror.rs           # 相机镜像（AVFoundation）
        ├── hud.rs              # 音量／亮度侦测
        ├── displays.rs         # 多屏幕跟随
        └── bluetooth.rs        # 蓝牙装置状态
```

## 技术选择：为什么是 Tauri 2？

- 最快能交出可用的版本：Rust 写后端，UI 用原生 HTML/CSS/JS，不用再学一整套 SwiftUI。
- 刘海 overlay 的关键是 AppKit：Rust 端用 `objc2` / `objc2-app-kit` / `objc2-foundation` 读 `NSScreen` 的 `safeAreaInsets` 和 `auxiliaryTopLeftArea` / `auxiliaryTopRightArea`，算出刘海宽度和展开尺寸；没有刘海就退回置中 pill。
- macOS 没有公开的 Dynamic Island API，所以跟原版一样，本项目也是自己做一个 borderless overlay 窗口。

## 路线图

v0.2.0 / v0.2.1 已完成（见上方更新日志）：

- [x] 音频可视化（动画长条，标示为动画；真实频谱因 Swift 运行环境问题已移除，待纯 objc2 实现）
- [x] 日历／提醒事项（AppleScript）
- [x] 相机镜像（AVFoundation）
- [x] 文件架存档、打开文件、AirDrop
- [x] 音量／亮度 HUD 显示（侦测＋刘海显示；不拦截系统提示）
- [x] 设置窗口：刘海尺寸、手势、多屏幕跟随、动画速度
- [x] 蓝牙动态、番茄钟、天气（默认关闭）

尚未做：

- [ ] 真实音频频谱（纯 objc2 ScreenCaptureKit 实现，不经过 Swift）
- [ ] 锁定画面小工具、通知（原版也还没做）
- [ ] 正式图标（`src-tauri/icons/`）与签名／公证
- [ ] 更多语言（视需求）

## 发布到 GitHub

本项目已上线，公开 repo 在 <https://github.com/zaire4real/notch-pilot>。

```bash
git clone https://github.com/zaire4real/notch-pilot.git
cd notch-pilot
npm install
npm run tauri dev
```

发布前请确认：`.gitignore` 已排除 `target/`、`node_modules/`、`.DS_Store`；`LICENSE`、三份 README（`README.md`、`README.zh-CN.md`、`README.en.md`）都在根目录；不要把编译出来的 `.app` / `.dmg` 推上去（它们在 `src-tauri/target/`，已被忽略）。

## 下载 .dmg（不用自己编译）

直接下载就好，不用装 Rust / Node 来编译：

**下载位置：<https://github.com/zaire4real/notch-pilot/releases>**

1. 打开上面的 Releases 页面，点最新的 release（目前是 **v0.2.1**）。
2. 在 **Assets** 里下载 `NotchPilot_0.2.1_aarch64.dmg`。
3. 双击安装，或拖到「应用程序」。

> 这个 `.dmg` **没有签名**（没有 Apple 开发者账号）。第一次打开时 macOS 会拦截，请用右键 →「打开」，或到「系统设置 → 隐私与安全性」点「仍要打开」。

备用方式：如果 Releases 还没有你要的版本，可以到 repo 的 **Actions** 页面 → **Build macOS DMG** → 选最新一次成功的执行，在最下面的 **Artifacts** 下载 **NotchPilot-macos-aarch64-dmg**（要登录 GitHub 才能下载，artifact 也会过期）。

## 授权

MIT，见 `LICENSE`。本项目不包含 boringNotch 的原始代码，也不是 TheBoredTeam 的官方项目。
