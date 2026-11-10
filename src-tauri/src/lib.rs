//! notch-pilot — Tauri 2 app entry point.
//!
//! Clean-room MVP: the window is a transparent, borderless overlay that
//! sits at the top-centre of the main screen (over the notch, when there
//! is one). All behaviour is implemented in `notch`, `media` and `system`.

pub mod media;
pub mod notch;
pub mod system;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
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
                if let Ok(geometry) = notch::get_notch_geometry() {
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
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            notch::get_notch_geometry,
            notch::set_notch_expanded,
            media::get_now_playing,
            media::media_play_pause,
            media::media_next,
            media::media_previous,
            system::get_battery,
            system::get_time,
        ])
        .run(tauri::generate_context!())
        .expect("error while running notch-pilot");
}
