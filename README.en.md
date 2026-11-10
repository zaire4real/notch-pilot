# NotchPilot

> **Other languages:** [繁體中文](README.md) · [简体中文](README.zh-CN.md)

Turn your MacBook notch into an expandable Dynamic Island — a Rust + Tauri 2 rewrite, MVP.

> **This is a clean-room rewrite, minimal viable version.**
> It reimplements boringNotch from a behavioural description only: hover the notch to expand, music controls, time / battery, file shelf.
> This project **contains no Swift source code from boringNotch** and is not affiliated with TheBoredTeam.
> The original is Swift / SwiftUI + AppKit, GPL-3.0; this project is Rust + Tauri 2, MIT.

## Languages

The UI supports three languages: **Traditional Chinese (zh-TW, default), Simplified Chinese (zh-CN), and English (en)**.

- On first launch it follows the system language: Traditional Chinese if the system is Traditional Chinese, Simplified Chinese for other Chinese locales, otherwise English.
- The expanded panel has a language switcher at the top (繁體中文 / 简体中文 / English). Click it and all text updates instantly — no restart needed.
- Your choice is stored in `localStorage` and reused next time.
- All UI strings live in `src/i18n.js`; there is no hardcoded Chinese in the HTML or `main.js`.

## What this MVP can do

| Feature | Status |
|---|---|
| Notch overlay: borderless, transparent, always on top, centred at the top of the screen | ✅ Yes |
| Hover to expand / leave to collapse: collapsed ~199×36, expanded ~680×200 (computed from notch geometry) | ✅ Yes |
| Notch geometry: `safeAreaInsets` + `auxiliaryTopLeftArea` / `auxiliaryTopRightArea`, falls back to a centred pill on notch-less screens | ✅ Yes (macOS) |
| Time / date display | ✅ Yes |
| Battery: parsed from `pmset -g batt`, shows percent, charging / on battery | ✅ Yes (macOS) |
| Now Playing: track / artist, Play / Pause / Previous / Next | ✅ Yes (see limitations below) |
| File shelf: drop files onto the expanded panel to list their names | ✅ Yes (names only) |
| Trilingual UI: Traditional Chinese / Simplified Chinese / English, instant switching | ✅ Yes |
| Real audio visualizer (spectrum) | ❌ Not in the MVP |
| AirDrop | ❌ Not in the MVP (only lists dropped file names, does not send anything) |
| Camera mirror | ❌ Not in the MVP |
| Calendar / Reminders integration | ❌ Not in the MVP |
| Volume / brightness HUD replacement (key interception) | ❌ Not in the MVP |
| Lock-screen widgets, weather, notifications | ❌ Not in the MVP |

## Now Playing limitations (read this first)

The MVP does **not** use the private MediaRemote framework.

It uses AppleScript (`osascript`):
1. First it asks Music.app whether it is playing;
2. If not, it asks Spotify;
3. If neither is playing, it reports "Nothing playing".

So:
- Only Music.app and Spotify are supported. Other players (YouTube Music, web players, etc.) will not show up.
- You need to allow the app to control Music / Spotify in "System Settings → Privacy & Security → Automation".
- Track / artist are polled, not pushed in real time.

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

> Note: this project was written on a Linux VM and **has not been compiled or run on macOS**.
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
src-tauri/target/release/bundle/dmg/NotchPilot_0.1.0_aarch64.dmg
src-tauri/target/release/bundle/macos/NotchPilot.app
```

## Gatekeeper note

This app is not signed with an Apple Developer account (same as the original boringNotch), so macOS will block it on first launch:

1. Seeing "cannot be opened because the developer cannot be verified" is normal — it is not malware.
2. Go to "System Settings → Privacy & Security", scroll to the bottom, and click "Open Anyway" next to NotchPilot.
3. Open it again and it will run normally.

If your work computer only allows notarized apps, ask IT first instead of forcing it.

## How to use

- **Collapsed:** a small black pill centred at the top of the screen, showing time and battery.
- **Expand:** hover over the pill to expand it; move away to collapse it.
- **Language:** click "繁體中文 / 简体中文 / English" at the top of the expanded panel to switch instantly.
- **Music:** the expanded panel shows track / artist; use ▶ / ⏸, ⏮, ⏭ to control Music or Spotify.
- **Shelf:** drag files onto the expanded panel and their names appear under "Shelf". The MVP only lists names — it does not move, copy, or AirDrop files.
- **Settings:** the footer explains that advanced notch height / gestures have no settings page in the MVP.

## Troubleshooting

| Problem | What to do |
|---|---|
| `npm run tauri dev` says `cargo` is not found | Restart the terminal, or run `source "$HOME/.cargo/env"`, then check `cargo --version` |
| `npm run tauri dev` says Xcode / `xcode-select` is missing | Run `xcode-select --install` and try again |
| Music always shows "Nothing playing" | Actually press play in Music or Spotify first; then allow the app to control it in "System Settings → Privacy & Security → Automation" |
| The window is not over the notch | Check whether an external display is the main display; the MVP only reads the main screen's (`NSScreen::main`) notch geometry |
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
│   └── main.js                 # Tauri commands, drag-drop shelf, language switching
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
        └── system.rs           # Battery (pmset), time
```

## Why Tauri 2?

- Fastest way to ship something usable: Rust for the backend, plain HTML/CSS/JS for the UI — no need to learn a whole SwiftUI stack.
- The notch overlay hinges on AppKit: the Rust side uses `objc2` / `objc2-app-kit` / `objc2-foundation` to read `NSScreen`'s `safeAreaInsets` and `auxiliaryTopLeftArea` / `auxiliaryTopRightArea`, compute the notch width and expanded size, and fall back to a centred pill when there is no notch.
- macOS has no public Dynamic Island API, so — like the original — this project draws its own borderless overlay window.

## Roadmap

- [ ] Real audio visualizer (sampled from system audio, not a fake animation)
- [ ] Calendar / Reminders via EventKit (needs permission)
- [ ] Camera mirror via AVFoundation preview
- [ ] Shelf that can actually AirDrop / open files
- [ ] Volume / brightness HUD replacement (needs system key interception — higher risk)
- [ ] Settings page: notch height, gestures, show on all displays
- [ ] More languages (as needed)
- [ ] Proper icon (`src-tauri/icons/`) plus signing / notarization

## Publishing to GitHub

This project is live in the public repo <https://github.com/zaire4real/notch-pilot>.

```bash
git clone https://github.com/zaire4real/notch-pilot.git
cd notch-pilot
npm install
npm run tauri dev
```

Before publishing, check: `.gitignore` excludes `target/`, `node_modules/`, and `.DS_Store`; `LICENSE` and all three READMEs (`README.md`, `README.zh-CN.md`, `README.en.md`) are in the root; do not push the built `.app` / `.dmg` (they live in `src-tauri/target/` and are ignored).

## License

MIT, see `LICENSE`. This project contains no boringNotch source code and is not an official TheBoredTeam project.
