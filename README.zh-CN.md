# NotchPilot

> **其他语言：** [繁體中文](README.md) · [English](README.en.md)

把 MacBook 的刘海变成可展开的动态岛 — 用 Rust + Tauri 2 重写的 MVP。

> **这是一个干净重写（clean-room rewrite）的最小可行版本。**
> 它只根据 boringNotch 的「行为描述」重做：鼠标悬停刘海展开、音乐控制、时间／电量、文件架。
> 本项目**没有复制任何 boringNotch 的 Swift 原始代码**，也与 TheBoredTeam 无关。
> 原项目是 Swift / SwiftUI + AppKit、GPL-3.0；本项目是 Rust + Tauri 2、MIT。

## 语言

软件 UI 支持三种语言：**繁体中文（zh-TW，默认）、简体中文（zh-CN）、英文（en）**。

- 第一次打开时，会先看系统语言：是繁体中文就用繁体中文，是其他中文就用简体中文，否则用英文。
- 展开面板上方有语言切换器（繁體中文 / 简体中文 / English），点一下就即时切换全部文字，不用重开 App。
- 你的选择会存在 `localStorage`，下次打开还是用你选的语言。
- 所有 UI 字符串都集中在 `src/i18n.js`，HTML / `main.js` 里没有硬编码的中文。

## 这个 MVP 能做什么

| 功能 | 状态 |
|---|---|
| 刘海 overlay：无边框、透明、置顶、开机显示在屏幕上方中央 | ✅ 有 |
| 悬停展开 / 离开展起：收起约 199×36，展开约 680×200（依刘海几何计算） | ✅ 有 |
| 读取刘海几何：`safeAreaInsets` + `auxiliaryTopLeftArea` / `auxiliaryTopRightArea`，无刘海时退回置中 pill | ✅ 有（macOS） |
| 时间／日期显示 | ✅ 有 |
| 电量：解析 `pmset -g batt`，显示百分比、充电中／使用电池 | ✅ 有（macOS） |
| Now Playing：显示曲名／歌手，播放／暂停／上一首／下一首 | ✅ 有（见下方限制） |
| 文件架：把文件拖到展开面板上，会列出文件名清单 | ✅ 有（仅列文件名） |
| 三语 UI：繁体中文／简体中文／英文，即时切换 | ✅ 有 |
| 真实音频可视化（频谱） | ❌ MVP 没有 |
| AirDrop | ❌ MVP 没有（只列出拖进来的文件名，不会真的传送） |
| 相机镜像 | ❌ MVP 没有 |
| 日历／提醒事项整合 | ❌ MVP 没有 |
| 取代系统音量／亮度 HUD（按键拦截） | ❌ MVP 没有 |
| 锁定画面小工具、天气、通知 | ❌ MVP 没有 |

## Now Playing 的限制（请先看）

MVP **没有用私有的 MediaRemote framework**。

做法是 AppleScript（`osascript`）：
1. 先问 Music.app 是否正在播放；
2. 如果没有，再问 Spotify 是否正在播放；
3. 都没有就回传「目前没有播放」。

所以：
- 只支持 Music.app 和 Spotify，其他播放器（YouTube Music、网页播放器等）不会显示。
- 需要在「系统设置 → 隐私权与安全性 → 自动化」允许本 App 控制 Music / Spotify。
- 曲名／歌手是轮询取得的，不是即时推送。

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

> 注意：本项目是在 Linux VM 上写的，**没有在 macOS 上编译或执行过**。
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
src-tauri/target/release/bundle/dmg/NotchPilot_0.1.0_aarch64.dmg
src-tauri/target/release/bundle/macos/NotchPilot.app
```

## Gatekeeper 提醒

这个 App 没有 Apple Developer 签名（跟原版 boringNotch 一样），第一次打开 macOS 会挡：

1. 打开后看到「无法打开，因为无法验证开发者」是正常的，不是恶意软件。
2. 到「系统设置 → 隐私权与安全性」，拉到最下面，点 NotchPilot 旁边的「仍要打开」。
3. 之后再开一次就能正常用。

如果你的公司电脑只允许已公证（notarized）的 App，请先问 IT，不要硬装。

## 怎么用

- **收起状态**：屏幕上方中央一个黑色小 pill，显示时间和电量。
- **展开**：鼠标移到 pill 上就会展开，移开就收起。
- **语言**：展开后点面板上方的「繁體中文 / 简体中文 / English」，即时切换。
- **音乐**：展开后看到曲名／歌手，按 ▶ / ⏸、⏮、⏭ 控制 Music 或 Spotify。
- **文件架**：把文件拖到展开的面板上，文件名会出现在「文件架」区。MVP 只列文件名，不会移动、复制或 AirDrop 文件。
- **设置**：面板下方有说明，进阶的刘海高度／手势目前 MVP 没有设置页。

## 疑难排解

| 问题 | 做法 |
|---|---|
| `npm run tauri dev` 说找不到 `cargo` | 重开终端，或 `source "$HOME/.cargo/env"`，再跑 `cargo --version` |
| `npm run tauri dev` 说找不到 Xcode / `xcode-select` | 跑 `xcode-select --install`，装完再试 |
| 音乐一直显示「目前没有播放」 | 先在 Music 或 Spotify 真的按下播放；再到「系统设置 → 隐私权与安全性 → 自动化」允许本 App 控制该 App |
| 窗口没有出现在刘海位置 | 检查是不是外接屏幕为主屏幕；MVP 只读主屏幕（`NSScreen::main`）的刘海几何 |
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
│   └── main.js                 # 调用 Tauri commands、拖放文件架、语言切换
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
        └── system.rs           # 电量（pmset）、时间
```

## 技术选择：为什么是 Tauri 2？

- 最快能交出可用的版本：Rust 写后端，UI 用原生 HTML/CSS/JS，不用再学一整套 SwiftUI。
- 刘海 overlay 的关键是 AppKit：Rust 端用 `objc2` / `objc2-app-kit` / `objc2-foundation` 读 `NSScreen` 的 `safeAreaInsets` 和 `auxiliaryTopLeftArea` / `auxiliaryTopRightArea`，算出刘海宽度和展开尺寸；没有刘海就退回置中 pill。
- macOS 没有公开的 Dynamic Island API，所以跟原版一样，本项目也是自己做一个 borderless overlay 窗口。

## 路线图

- [ ] 真实音频可视化（用系统音频取样，不是假动画）
- [ ] 日历／提醒事项：读 EventKit（需要授权）
- [ ] 相机镜像：AVFoundation 预览
- [ ] 文件架真的能 AirDrop / 打开文件
- [ ] 音量／亮度 HUD 取代（需要拦截系统按键，风险较高）
- [ ] 设置页：刘海高度、手势、要不要显示在所有屏幕
- [ ] 更多语言（视需求）
- [ ] 正式图标（`src-tauri/icons/`）与签名／公证

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

1. 打开上面的 Releases 页面，点最新的 release（目前是 **v0.1.0**）。
2. 在 **Assets** 里下载 `NotchPilot_0.1.0_aarch64.dmg`。
3. 双击安装，或拖到「应用程序」。

> 这个 `.dmg` **没有签名**（没有 Apple 开发者账号）。第一次打开时 macOS 会拦截，请用右键 →「打开」，或到「系统设置 → 隐私与安全性」点「仍要打开」。

备用方式：如果 Releases 还没有你要的版本，可以到 repo 的 **Actions** 页面 → **Build macOS DMG** → 选最新一次成功的执行，在最下面的 **Artifacts** 下载 **NotchPilot-macos-aarch64-dmg**（要登录 GitHub 才能下载，artifact 也会过期）。

## 授权

MIT，见 `LICENSE`。本项目不包含 boringNotch 的原始代码，也不是 TheBoredTeam 的官方项目。
