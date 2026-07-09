use serde::Serialize;
use tauri::{AppHandle, State};
use tauri_plugin_dialog::DialogExt;

use crate::state::AppState;
use crate::settings::{AppSettings, save_settings as save_settings_to_disk};
use crate::scheduler::{force_next_wallpaper, last_change_time};
use crate::scanner::scan_directories;

#[derive(Serialize)]
pub struct AppStatus {
    pub current_image: Option<String>,
    pub total_images: usize,
    pub is_paused: bool,
    pub time_remaining: u64,
}

#[tauri::command]
pub fn get_settings(state: State<'_, AppState>) -> AppSettings {
    state.settings.lock().unwrap().clone()
}

#[tauri::command]
pub fn save_settings(app: AppHandle, state: State<'_, AppState>, new_settings: AppSettings) -> Result<(), String> {
    *state.settings.lock().unwrap() = new_settings.clone();
    
    use tauri_plugin_autostart::ManagerExt;
    let autolaunch = app.autolaunch();
    if new_settings.autostart {
        let _ = autolaunch.enable();
    } else {
        let _ = autolaunch.disable();
    }
    
    save_settings_to_disk(&app, &new_settings)
}

#[tauri::command]
pub fn get_status(state: State<'_, AppState>) -> AppStatus {
    let settings = state.settings.lock().unwrap().clone();
    let is_paused = *state.is_paused.lock().unwrap();
    let current_image = state.current_image.lock().unwrap().clone();
    
    let total_images = if settings.folder_path.is_empty() {
        0
    } else {
        scan_directories(&settings.folder_path).len()
    };
    
    let time_remaining = if is_paused {
        0
    } else {
        let elapsed = last_change_time().elapsed().as_secs();
        let interval = (settings.interval_minutes as u64) * 60;
        interval.saturating_sub(elapsed)
    };
    
    AppStatus {
        current_image,
        total_images,
        is_paused,
        time_remaining,
    }
}

#[tauri::command]
pub fn next_wallpaper(app: AppHandle) {
    force_next_wallpaper(&app);
}

#[tauri::command]
pub async fn select_folder(app: AppHandle) -> Result<Option<String>, String> {
    let folder = app.dialog().file().blocking_pick_folder();
    Ok(folder.map(|f| f.to_string()))
}

#[tauri::command]
pub fn start_drag(app: AppHandle) {
    if let Some(window) = tauri::Manager::get_webview_window(&app, "main") {
        let _ = window.start_dragging();
    }
}

#[tauri::command]
pub fn resize_window(app: AppHandle, width: f64, height: f64) {
    if let Some(window) = tauri::Manager::get_webview_window(&app, "main") {
        let _ = window.set_size(tauri::LogicalSize::new(width, height));
    }
}

#[tauri::command]
pub fn hide_window(app: AppHandle) {
    if let Some(window) = tauri::Manager::get_webview_window(&app, "main") {
        let _ = window.hide();
    }
}

#[tauri::command]
pub fn log_error(msg: String) {
    println!("FRONTEND ERROR: {}", msg);
    let _ = std::fs::write("frontend_error.log", msg);
}
