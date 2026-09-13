mod commands;
mod context;
mod fullscreen;
mod scanner;
mod scheduler;
mod settings;
mod state;
mod tray;
mod wallpaper;

use state::AppState;
use std::sync::Mutex;
use std::time::Instant;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
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
        ])
        .setup(|app| {
            let settings = settings::load_settings(app.handle());

            app.manage(AppState {
                settings: Mutex::new(settings),
                current_image: Mutex::new(None),
                is_paused: Mutex::new(false),
                is_fullscreen_paused: Mutex::new(false),
                history: Mutex::new(Vec::new()),
                redo: Mutex::new(Vec::new()),
                shuffle_bag: Mutex::new(Vec::new()),
                last_change: Mutex::new(Instant::now()),
            });

            tray::create_tray(app.handle()).expect("Failed to create tray");
            scheduler::start_scheduler(app.handle().clone());

            let args: Vec<String> = std::env::args().collect();
            if !args.contains(&"--autostart".to_string()) {
                if let Some(window) = app.get_webview_window("main") {
                    window.show().unwrap();
                    window.set_focus().unwrap();
                }
            }

            Ok(())
        })
        .on_window_event(|window, event| match event {
            tauri::WindowEvent::CloseRequested { api, .. } => {
                window.hide().unwrap();
                api.prevent_close();
            }
            _ => {}
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
