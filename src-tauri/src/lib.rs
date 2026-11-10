//! notch-pilot — Tauri 2 app entry point.
//!
//! Clean-room MVP: the window is a transparent, borderless overlay that
//! sits at the top-centre of the main screen (over the notch, when there
//! is one). All behaviour is implemented in `notch`, `media`, `system`
//! and `settings`.
//!
//! v0.2 Phase 0 adds: a Settings window (`settings` module), a menu-bar
//! tray icon (F22) and a Cmd+, global shortcut that opens Settings.

pub mod airdrop;
pub mod audio;
pub mod calendar;
pub mod hud;
pub mod media;
pub mod mirror;
pub mod notch;
pub mod settings;
pub mod system;
pub mod visualizer;

use tauri::image::Image;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::Manager;

/// Build the menu-bar tray icon (a white notch pill, drawn in code so
/// no binary asset is needed) with its menu. Errors are returned so the
/// caller can decide whether a missing tray is fatal — it is not: the
/// gear button and Cmd+, still open Settings without it.
fn build_tray(app: &tauri::App) -> tauri::Result<()> {
    let open_settings =
        MenuItem::with_id(app, "open-settings", "Open Settings", true, None::<&str>)?;
    let toggle_panel =
        MenuItem::with_id(app, "toggle-panel", "Show/Hide Panel", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit NotchPilot", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&open_settings, &toggle_panel, &quit])?;

    TrayIconBuilder::with_id("main-tray")
        .icon(tray_icon_image())
        .icon_as_template(true)
        .tooltip("NotchPilot")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "open-settings" => {
                let _ = settings::open_settings_window(app);
            }
            "toggle-panel" => {
                if let Some(main) = app.get_webview_window("main") {
                    match main.is_visible() {
                        Ok(true) => {
                            let _ = main.hide();
                        }
                        _ => {
                            let _ = main.show();
                        }
                    }
                }
            }
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            // Left-click the tray icon opens Settings; right-click
            // shows the menu (the default behaviour).
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let _ = settings::open_settings_window(tray.app_handle());
            }
        })
        .build(app)?;
    Ok(())
}

/// White rounded pill (the collapsed notch bar) with a transparent
/// camera-dot hole, 32x32 RGBA, drawn in code. Used as a template
/// image so macOS renders it in menu-bar style.
fn tray_icon_image() -> Image<'static> {
    const W: usize = 32;
    const H: usize = 32;
    let mut rgba = vec![0u8; W * H * 4];
    // Pill: centred at (16, 16), half-size (14, 4), corner radius 4.
    // Camera dot: transparent hole of radius 2.5 at the centre.
    for y in 0..H {
        for x in 0..W {
            let px = x as f64 + 0.5;
            let py = y as f64 + 0.5;
            let qx = (px - 16.0).abs() - (14.0 - 4.0);
            let qy = (py - 16.0).abs() - (4.0 - 4.0);
            let ax = qx.max(0.0);
            let ay = qy.max(0.0);
            let dist = (ax * ax + ay * ay).sqrt() + qx.max(qy).min(0.0) - 4.0;
            let mut coverage = (0.5 - dist).clamp(0.0, 1.0);
            let dot = ((px - 16.0).powi(2) + (py - 16.0).powi(2)).sqrt();
            if dot < 2.5 {
                coverage = 0.0;
            }
            let i = (y * W + x) * 4;
            rgba[i] = 255;
            rgba[i + 1] = 255;
            rgba[i + 2] = 255;
            rgba[i + 3] = (coverage * 255.0) as u8;
        }
    }
    Image::new_owned(rgba, W as u32, H as u32)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        // Cmd+, opens Settings. Registered here (not via the plugin
        // builder) so a registration failure — e.g. the shortcut is
        // already taken by another app — is ignored instead of
        // failing the whole app start. The plugin's own setup runs
        // before this setup closure, so global_shortcut() state
        // exists by now.
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .setup(|app| {
            // The main window starts invisible in tauri.conf.json so we can
            // position it from the real notch geometry before showing it.
            // Tauri 2 removed get_window; get_webview_window returns a
            // WebviewWindow. Its AsRef<Webview> gives the Webview, whose
            // .window() returns an OWNED Window (cloned, not &Window) —
            // so bind it first and pass a reference to the binding.
            if let Some(window) = app.get_webview_window("main") {
                let native_window = window.as_ref().window();
                notch::apply_notch_window(&native_window);
                // Position at the collapsed pill size first, then show.
                if let Ok(geometry) = notch::get_notch_geometry(app.app_handle().clone()) {
                    let x = (geometry.screen_width - geometry.idle_width) / 2.0;
                    let _ = window.set_size(tauri::Size::Logical(tauri::LogicalSize {
                        width: geometry.idle_width,
                        height: geometry.idle_height,
                    }));
                    let _ = window.set_position(tauri::Position::Logical(tauri::LogicalPosition {
                        x,
                        y: 0.0,
                    }));
                }
                let _ = window.show();
            }

            // Menu-bar tray icon. Not fatal if it fails.
            if let Err(e) = build_tray(app) {
                eprintln!("notch-pilot: tray icon failed: {e}");
            }

            // Cmd+, (Super + Comma) opens Settings. Registration
            // failure is ignored — the gear button and tray menu
            // still open Settings.
            {
                use tauri_plugin_global_shortcut::{
                    Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState,
                };
                let _ = app.global_shortcut().on_shortcut(
                    Shortcut::new(Some(Modifiers::SUPER), Code::Comma),
                    |app, _shortcut, event| {
                        if event.state == ShortcutState::Pressed {
                            let _ = settings::open_settings_window(app);
                        }
                    },
                );
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            notch::get_notch_geometry,
            notch::set_notch_expanded,
            notch::set_notch_peek,
            media::get_now_playing,
            media::media_play_pause,
            media::media_next,
            media::media_previous,
            media::media_seek,
            audio::get_audio_outputs,
            audio::set_audio_output,
            calendar::get_calendar_data,
            calendar::get_reminders,
            calendar::complete_reminder,
            hud::get_hud_state,
            visualizer::visualizer_start,
            visualizer::visualizer_stop,
            visualizer::visualizer_levels,
            system::get_battery,
            system::get_time,
            settings::get_settings,
            settings::set_settings,
            settings::open_settings,
            settings::get_shelf,
            settings::add_shelf_files,
            settings::remove_shelf_file,
            settings::open_shelf_file,
            airdrop::airdrop_available,
            airdrop::airdrop_shelf_files,
            mirror::mirror_cameras,
            mirror::mirror_start,
            mirror::mirror_stop,
            mirror::mirror_frame,
        ])
        .run(tauri::generate_context!())
        .expect("error while running notch-pilot");
}
