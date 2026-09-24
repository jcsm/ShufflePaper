mod commands;
mod context;
mod fullscreen;
mod logging;
mod scanner;
mod scheduler;
mod settings;
mod state;
mod svg;
mod tray;
mod wallpaper;

use state::AppState;
use std::sync::Mutex;
use std::time::Instant;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Record panics before anything else can fail: without this a background
    // panic killed its thread and left no trace anywhere.
    logging::install_panic_hook();

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--autostart"]),
        ))
        .invoke_handler(tauri::generate_handler![
            commands::get_settings,
            commands::save_settings,
            commands::get_status,
            commands::next_wallpaper,
            commands::previous_wallpaper,
            commands::select_folder,
            commands::log_error,
            commands::hide_window,
            commands::resize_window,
            commands::start_drag,
            commands::set_force_mode,
            commands::open_log_file,
        ])
        .setup(|app| {
            let settings = settings::load_settings(app.handle());
            logging::info(format!(
                "ShufflePaper {} starting — mode={}, interval={} min, folder={}, context_rules={}, pause_on_fullscreen={}",
                env!("CARGO_PKG_VERSION"),
                settings.mode,
                settings.interval_minutes,
                settings.folder_path,
                settings.context_rules_enabled,
                settings.pause_on_fullscreen,
            ));

            app.manage(AppState {
                settings: Mutex::new(settings),
                current_image: Mutex::new(None),
                is_paused: Mutex::new(false),
                is_fullscreen_paused: Mutex::new(false),
                history: Mutex::new(Vec::new()),
                redo: Mutex::new(Vec::new()),
                shuffle_bag: Mutex::new(Vec::new()),
                last_change: Mutex::new(Instant::now()),
                failed_images: Mutex::new(Vec::new()),
                last_error: Mutex::new(None),
            });

            // A tray failure must not take the whole app down with a bare
            // panic and no window, which is what `.expect(...)` used to do.
            if let Err(err) = tray::create_tray(app.handle()) {
                logging::error(format!("failed to create tray icon: {err}"));
            }
            scheduler::start_scheduler(app.handle().clone());

            let args: Vec<String> = std::env::args().collect();
            if !args.contains(&"--autostart".to_string()) {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }

            Ok(())
        })
        .on_window_event(|window, event| match event {
            tauri::WindowEvent::CloseRequested { api, .. } => {
                if let Err(err) = window.hide() {
                    logging::error(format!("could not hide window: {err}"));
                }
                api.prevent_close();
            }
            _ => {}
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
