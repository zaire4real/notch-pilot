# NotchPilot

> **Other languages:** [繁體中文](README.md) · [简体中文](README.zh-CN.md)

Turn your MacBook notch into an expandable Dynamic Island — a Rust + Tauri 2 rewrite.

> **This is a clean-room rewrite.**
> It reimplements boringNotch from a behavioural description only — feature and layout parity: hover the notch to expand, music controls, time / battery, file shelf, and since v0.2.0 also album art, audio visualizer, calendar, camera mirror, AirDrop, volume / brightness HUDs and a Settings window.
> This project **contains no Swift source code from boringNotch** and is not affiliated with TheBoredTeam.
> The original is Swift / SwiftUI + AppKit, GPL-3.0; this project is Rust + Tauri 2, MIT.

## What's new in v0.2.1

v0.2.1: fixes v0.2.0 failing to launch at all on a real MacBook (see below); everything else is the same as v0.2.0.

- **Launch fix**: v0.2.0 was aborted by dyld at startup on the MacBook Air M3 (`Library not loaded: @rpath/libswift_Concurrency.dylib … no LC_RPATH's found`) and never showed a window. The cause was the `screencapturekit` crate used for the audio visualizer: it compiles a Swift bridge, so the binary links the Swift runtime. macOS keeps the core Swift runtime only in the dyld shared cache — the on-disk `/usr/lib/swift/` holds just a few framework Swift libs (Photos, Network, …) — so adding an RPATH could not find it either. v0.2.1 drops the `screencapturekit` dependency entirely, so the app no longer links Swift.
- **Visualizer is animated-only**: it now shows only the simulated spectrum (8 bars) clearly labelled as animation, never presented as real audio. A real spectrum can only return later via a pure-objc2 ScreenCaptureKit implementation (no Swift).
- Everything else — features, layout and settings — is the same as v0.2.0.

> Note: v0.2.0 never launched successfully. Do not use the v0.2.0 .dmg; use v0.2.1 instead.

## What's new in v0.2.0 (⚠️ this version does not launch — use v0.2.1)

v0.2.0: feature and layout parity with the original boringNotch: album art, progress/seek, audio visualizer, calendar & reminders, camera mirror, file shelf with AirDrop, volume/brightness HUDs, Bluetooth live activity, gestures and notch sizing; language moved into the Settings window.

New and changed in detail:

- **Media**: album art, progress bar with seek, synced lyrics (LRCLIB, off by default), audio output selector (speakers / headphones / AirPods / Bluetooth).
- **Audio visualizer**: originally a real 8-band spectrum from system audio via ScreenCaptureKit — but that needed the Swift runtime and made v0.2.0 fail to launch entirely, so v0.2.1 shows only the simulated spectrum clearly labelled as animation (see the v0.2.1 changelog above).
- **Layout**: expanded panel order is now media → calendar → bottom strip (camera mirror + shelf entry); the shelf is a separate view; the collapsed pill no longer shows a clock (live activity + battery only); idle "eyes" animation is available (off by default); short messages (volume, track change) use a ~70px sneak-peek that auto-collapses after ~2s.
- **Calendar / Reminders**: today's events, countdown to the next event, the first event tomorrow, and incomplete reminders (check them off). Reads Calendar.app / Reminders.app via AppleScript — grant Automation permission.
- **Shelf**: files persist across restarts; click to open with the default app; remove with right-click or ×. AirDrop sends files to nearby devices (success means the share sheet opened).
- **Camera mirror**: AVFoundation capture (640×480, ~11fps); circle / rounded shape in Settings.
- **HUDs**: volume and brightness changes show in the notch as a sneak-peek; Apple's own volume/brightness OSD is not suppressed. Keyboard backlight is not supported yet.
- **Gestures**: trackpad swipe to open/close the panel, horizontal swipe to change tracks.
- **Settings window**: standalone window with 11 sections (Notch, Appearance, Media, Calendar, Shelf, Mirror, Battery, OSD, Notifications, Shortcuts, About); language is at the top of the Notch section; menu-bar icon (click to open Settings, right-click menu) and Cmd+, also open it; settings live in `settings.json`, including animation speed and notch sizing.
- **Multi-display**: a single window follows the active display (not one window per display).
- **Extras**: Bluetooth device live activity, Pomodoro timer, weather (Open-Meteo, off by default, default city Taipei; weather is a NotchPilot extra — the original does not have it).

## Languages

The UI supports three languages: **Traditional Chinese (zh-TW, default), Simplified Chinese (zh-CN), and English (en)**.

- On first launch it follows the system language: Traditional Chinese if the system is Traditional Chinese, Simplified Chinese for other Chinese locales, otherwise English.
- Language is picked in the **Settings window** (open it with the ⚙ button in the panel, the menu-bar icon, or Cmd+,) → at the top of the "Notch" section (繁體中文 / 简体中文 / English). Click it and all text updates instantly — no restart needed.
- Your choice is stored in `settings.json` (`~/Library/Application Support/com.yichia.notch-pilot/settings.json`) and reused next time.
- All UI strings live in `src/i18n.js`; there is no hardcoded Chinese in the HTML or `main.js`.

## What this version can do

| Feature | Status |
|---|---|
| Notch overlay: borderless, transparent, always on top, centred at the top of the screen | ✅ Yes |
| Hover to expand / leave to collapse: collapsed is notch-width × 36, expanded 680×360 (adjustable in Settings) | ✅ Yes |
| Notch geometry: `safeAreaInsets` + `auxiliaryTopLeftArea` / `auxiliaryTopRightArea`, falls back to a centred pill on notch-less screens | ✅ Yes (macOS) |
| Time / date display | ✅ Yes (in the expanded panel; the collapsed pill shows no clock) |
| Battery: parsed from `pmset -g batt`, shows percent, charging / on battery, time remaining | ✅ Yes (macOS) |
| Now Playing: album art, track / artist, progress, Play / Pause / Previous / Next, seek | ✅ Yes (see limitations below) |
| Audio visualizer (animated spectrum) | ✅ Yes (animated bars, clearly labelled as animation; the real spectrum was removed in v0.2.1 because of the Swift runtime problem, pending a pure-objc2 implementation) |
| Synced lyrics (LRCLIB) | ✅ Yes (off by default) |
| Audio output selector | ✅ Yes (speakers / headphones / AirPods / Bluetooth) |
| Calendar / Reminders integration | ✅ Yes (AppleScript to Calendar.app / Reminders.app) |
| Camera mirror | ✅ Yes (AVFoundation, 640×480, ~11fps) |
| File shelf: dropped files persist, open, remove | ✅ Yes |
| AirDrop | ✅ Yes (success means the share sheet opened) |
| Volume / brightness HUDs (shown in the notch) | ✅ Yes (detection + display; Apple's own OSD is not suppressed; keyboard backlight unsupported) |
| Gestures: swipe to open/close, horizontal swipe to change tracks | ✅ Yes |
| Settings window (11 sections), menu-bar icon, Cmd+, | ✅ Yes |
| Multi-display: follows the active display | ✅ Yes (single window, not one window per display) |
| Bluetooth live activity, Pomodoro timer | ✅ Yes |
| Weather (Open-Meteo) | ✅ Yes (a NotchPilot extra, off by default, default city Taipei) |
| Trilingual UI: Traditional Chinese / Simplified Chinese / English, instant switching | ✅ Yes (switched in the Settings window) |
| Lock-screen widgets, notifications | ❌ No (the original has not shipped them either) |

## Now Playing limitations (read this first)

This project does **not** use the private MediaRemote framework.

It uses AppleScript (`osascript`):
1. First it asks Music.app whether it is playing;
2. If not, it asks Spotify;
3. If neither is playing, it reports "Nothing playing".

So:
- Only Music.app and Spotify are supported. Other players (YouTube Music, web players, etc.) will not show up.
- You need to allow the app to control Music / Spotify in "System Settings → Privacy & Security → Automation".
- Track / artist are polled, not pushed in real time.
- Album art comes from AppleScript; synced lyrics come from the free LRCLIB API (off by default, enable in Settings → Media); the audio output selector switches devices via CoreAudio.

## Requirements (on your MacBook M3)

- macOS 14 Sonoma or later (Apple Silicon / M3)
- Xcode Command Line Tools: `xcode-select --install`
- Rust (install via rustup): https://rustup.rs
  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  ```
  Restart the terminal afterwards and check `cargo --version`.
- Node.js 20+ (Node v24.22.1 was available when this was written on Linux)
- The Tauri CLI is installed as a devDependency via `npm install`; no global install needed.

> Note: this project was written on a Linux VM; macOS builds run on GitHub Actions macOS runners. v0.2.0 was tried on a real MacBook Air M3 but was aborted by dyld at launch (Swift runtime problem); v0.2.1 drops the Swift dependency but has not been confirmed to launch on a real MacBook yet.
> The commands below are meant to run on your MacBook M3; the first `npm run tauri dev` will take a few minutes to compile Rust.

## Install and run

```bash
cd ~/workspace/notch-pilot
# or cd into the folder after copying it to your Mac

npm install

# Dev mode: opens an overlay window; frontend changes show up live
npm run tauri dev

# Package as .app / .dmg (output in src-tauri/target/release/bundle/)
npm run tauri build
```

`npm run tauri build` produces:

```
src-tauri/target/release/bundle/dmg/NotchPilot_0.2.1_aarch64.dmg
src-tauri/target/release/bundle/macos/NotchPilot.app
```

## Gatekeeper note

This app is not signed with an Apple Developer account (same as the original boringNotch), so macOS will block it on first launch:

1. Seeing "cannot be opened because the developer cannot be verified" is normal — it is not malware.
2. Go to "System Settings → Privacy & Security", scroll to the bottom, and click "Open Anyway" next to NotchPilot.
3. Open it again and it will run normally.

If your work computer only allows notarized apps, ask IT first instead of forcing it.

## How to use

- **Collapsed:** a small black pill centred at the top of the screen, showing live activity (album thumbnail / visualizer while playing, charging state) and battery. No clock — check the menu bar or the expanded panel for the time.
- **Expand:** hover over the pill to expand it; move away to collapse it. The expanded order is media → calendar → bottom strip (camera mirror + shelf entry).
- **Language:** in the Settings window (⚙ button in the panel, menu-bar icon, or Cmd+,) → at the top of the "Notch" section; switches instantly.
- **Music:** the expanded panel shows album art, track / artist and progress; use ▶ / ⏸, ⏮, ⏭ to control Music or Spotify; the progress bar seeks.
- **Shelf:** drag files onto the expanded panel; they persist across restarts. Click to open with the default app, right-click or × to remove; the AirDrop button sends files to nearby devices.
- **Settings:** a standalone Settings window with 11 sections — notch sizing, animation speed, gestures, visualizer, mirror shape and more; settings live in `settings.json`.

## Troubleshooting

| Problem | What to do |
|---|---|
| `npm run tauri dev` says `cargo` is not found | Restart the terminal, or run `source "$HOME/.cargo/env"`, then check `cargo --version` |
| `npm run tauri dev` says Xcode / `xcode-select` is missing | Run `xcode-select --install` and try again |
| Music always shows "Nothing playing" | Actually press play in Music or Spotify first; then allow the app to control it in "System Settings → Privacy & Security → Automation" |
| The window is not over the notch | Check whether an external display is the main display; this version is a single window that follows the active display |
| A notch-less screen (external display) | It falls back to a centred pill; everything works the same, just without the notch shape |
| Some text does not change after switching language | That is a bug: every string should be in `src/i18n.js` — please report which string was missed |
| `tauri dev` takes a long time to compile | Normal the first time — Rust has to compile Tauri and objc2; later builds are much faster |

## Project structure

```
notch-pilot/
├── README.md                   # Traditional Chinese (main version)
├── README.zh-CN.md             # Simplified Chinese
├── README.en.md                # English
├── LICENSE                     # MIT
├── .gitignore                  # target/, node_modules/, .DS_Store, etc.
├── package.json                # Tauri 2 CLI, vanilla frontend, no framework build step
├── src/
│   ├── index.html              # Notch panel UI (no hardcoded text)
│   ├── i18n.js                 # Trilingual dictionary: zh-TW / zh-CN / en
│   ├── styles.css              # Black pill / expanded panel styling
│   ├── main.js                 # Tauri commands, drag-drop shelf, language switching
│   ├── settings.html           # Settings window UI
│   ├── settings.js             # Settings window logic
│   └── settings.css            # Settings window styling
└── src-tauri/
    ├── Cargo.toml              # tauri 2, serde; objc2 only on the macOS target
    ├── tauri.conf.json         # Transparent, borderless, always on top, skip taskbar
    ├── capabilities/
    │   └── default.json        # Minimal permissions
    ├── build.rs                # tauri_build::build()
    └── src/
        ├── main.rs             # Entry point, calls lib::run()
        ├── lib.rs              # Tauri builder, registered commands, setup calls notch
        ├── notch.rs            # Notch geometry (objc2 / AppKit) + expand / collapse
        ├── media.rs            # Now Playing (AppleScript → Music / Spotify)
        ├── system.rs           # Battery (pmset), time
        ├── settings.rs         # Settings file (settings.json) read/write
        ├── audio.rs            # Audio output devices (CoreAudio)
        ├── visualizer.rs       # Audio visualizer (animated bars; real spectrum removed)
        ├── calendar.rs         # Calendar / Reminders (AppleScript)
        ├── airdrop.rs          # AirDrop (NSSharingService)
        ├── mirror.rs           # Camera mirror (AVFoundation)
        ├── hud.rs              # Volume / brightness detection
        ├── displays.rs         # Multi-display following
        └── bluetooth.rs        # Bluetooth device status
```

## Why Tauri 2?

- Fastest way to ship something usable: Rust for the backend, plain HTML/CSS/JS for the UI — no need to learn a whole SwiftUI stack.
- The notch overlay hinges on AppKit: the Rust side uses `objc2` / `objc2-app-kit` / `objc2-foundation` to read `NSScreen`'s `safeAreaInsets` and `auxiliaryTopLeftArea` / `auxiliaryTopRightArea`, compute the notch width and expanded size, and fall back to a centred pill when there is no notch.
- macOS has no public Dynamic Island API, so — like the original — this project draws its own borderless overlay window.

## Roadmap

v0.2.0 / v0.2.1 is done (see the changelog above):

- [x] Audio visualizer (animated bars, labelled as animation; real spectrum removed in v0.2.1 over the Swift runtime problem, pending a pure-objc2 implementation)
- [x] Calendar / Reminders (AppleScript)
- [x] Camera mirror (AVFoundation)
- [x] Shelf persistence, opening files, AirDrop
- [x] Volume / brightness HUD display (detection + notch display; does not intercept the system OSD)
- [x] Settings window: notch sizing, gestures, multi-display following, animation speed
- [x] Bluetooth live activity, Pomodoro timer, weather (off by default)

Not done yet:

- [ ] Real audio spectrum (pure-objc2 ScreenCaptureKit implementation, no Swift)
- [ ] Lock-screen widgets, notifications (the original has not shipped them either)
- [ ] Proper icon (`src-tauri/icons/`) plus signing / notarization
- [ ] More languages (as needed)

## Publishing to GitHub

This project is live in the public repo <https://github.com/zaire4real/notch-pilot>.

```bash
git clone https://github.com/zaire4real/notch-pilot.git
cd notch-pilot
npm install
npm run tauri dev
```

Before publishing, check: `.gitignore` excludes `target/`, `node_modules/`, and `.DS_Store`; `LICENSE` and all three READMEs (`README.md`, `README.zh-CN.md`, `README.en.md`) are in the root; do not push the built `.app` / `.dmg` (they live in `src-tauri/target/` and are ignored).

## Download the .dmg (no need to build it yourself)

Just download it — no need to install Rust / Node to build:

**Download from: <https://github.com/zaire4real/notch-pilot/releases>**

1. Open the Releases page above and pick the latest release (currently **v0.2.1**).
2. Under **Assets**, download `NotchPilot_0.2.1_aarch64.dmg`.
3. Double-click it to install, or drag it to Applications.

> This `.dmg` is **unsigned** (there is no Apple Developer account). On first launch macOS will block it; right-click it and choose **Open**, or go to **System Settings > Privacy & Security** and click **Open Anyway**.

Fallback: if the version you want is not in Releases yet, go to the repo's **Actions** tab → **Build macOS DMG** → pick the latest successful run, and download **NotchPilot-macos-aarch64-dmg** from **Artifacts** at the bottom (requires a GitHub login, and artifacts expire).

## License

MIT, see `LICENSE`. This project contains no boringNotch source code and is not an official TheBoredTeam project.
