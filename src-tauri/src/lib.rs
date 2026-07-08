mod settings;
mod state;
mod scanner;
mod wallpaper;
mod scheduler;
mod tray;
mod commands;

use std::sync::Mutex;
use tauri::Manager;
use state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_autostart::init(tauri_plugin_autostart::MacosLauncher::LaunchAgent, Some(vec!["--autostart"])))
        .invoke_handler(tauri::generate_handler![
            commands::get_settings,
            commands::save_settings,
            commands::get_status,
            commands::next_wallpaper,
            commands::select_folder,
            commands::log_error
        ])
        .setup(|app| {
            let settings = settings::load_settings(app.handle());
            
            app.manage(AppState {
                settings: Mutex::new(settings),
                current_image: Mutex::new(None),
                is_paused: Mutex::new(false),
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
