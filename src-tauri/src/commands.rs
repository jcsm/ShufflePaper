use serde::Serialize;
use tauri::{AppHandle, State};
use tauri_plugin_dialog::DialogExt;

use crate::state::AppState;
use crate::settings::{AppSettings, save_settings as save_settings_to_disk};
use crate::scheduler::{advance_wallpaper, restore_previous_wallpaper};
use crate::scanner::scan_directories;

#[derive(Serialize)]
pub struct AppStatus {
    pub current_image: Option<String>,
    pub total_images: usize,
    pub is_paused: bool,
    pub time_remaining: u64,
    pub can_previous: bool,
}

#[tauri::command]
pub fn get_settings(state: State<'_, AppState>) -> AppSettings {
    state.settings.lock().unwrap().clone()
}

#[tauri::command]
pub fn save_settings(app: AppHandle, state: State<'_, AppState>, new_settings: AppSettings) -> Result<(), String> {
    {
        let mut settings = state.settings.lock().unwrap();
        if settings.folder_path != new_settings.folder_path {
            // History, redo, and the shuffle bag all point into the old folder.
            state.history.lock().unwrap().clear();
            state.redo.lock().unwrap().clear();
            state.shuffle_bag.lock().unwrap().clear();
        }
        *settings = new_settings.clone();
    }
    
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
    let can_previous = !state.history.lock().unwrap().is_empty();
    
    let total_images = if settings.folder_path.is_empty() {
        0
    } else {
        scan_directories(&settings.folder_path).len()
    };
    
    let time_remaining = if is_paused {
        0
    } else {
        let elapsed = state.last_change.lock().unwrap().elapsed().as_secs();
        let interval = (settings.interval_minutes as u64) * 60;
        interval.saturating_sub(elapsed)
    };
    
    AppStatus {
        current_image,
        total_images,
        is_paused,
        time_remaining,
        can_previous,
    }
}

#[tauri::command]
pub fn next_wallpaper(app: AppHandle) {
    advance_wallpaper(&app, true);
}

#[tauri::command]
pub fn previous_wallpaper(app: AppHandle) {
    restore_previous_wallpaper(&app);
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
