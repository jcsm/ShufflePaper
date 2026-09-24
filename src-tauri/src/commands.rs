use serde::Serialize;
use tauri::{AppHandle, Manager, State};
use tauri_plugin_dialog::DialogExt;

use crate::context::{active_profile, profile_label, Profile};
use crate::logging;
use crate::scanner::scan_directories;
use crate::scheduler::{advance_wallpaper, restore_previous_wallpaper};
use crate::settings::{save_settings as save_settings_to_disk, AppSettings};
use crate::state::{lock, AppState};

#[derive(Serialize)]
pub struct AppStatus {
    pub current_image: Option<String>,
    pub total_images: usize,
    pub is_paused: bool,
    pub paused_fullscreen: bool,
    pub time_remaining: u64,
    pub can_previous: bool,
    pub active_profile: String,
    /// Last wallpaper failure, so the UI can show it instead of failing silently.
    pub last_error: Option<String>,
}

#[tauri::command]
pub fn get_settings(state: State<'_, AppState>) -> AppSettings {
    lock(&state.settings).clone()
}

#[tauri::command(async)]
pub fn save_settings(
    app: AppHandle,
    new_settings: AppSettings,
) -> Result<(), String> {
    let state = app.state::<AppState>();
    {
        let mut settings = lock(&state.settings);
        if settings.folder_path != new_settings.folder_path
            || settings.work_folder != new_settings.work_folder
            || settings.personal_folder != new_settings.personal_folder
            || settings.context_rules_enabled != new_settings.context_rules_enabled
        {
            lock(&state.history).clear();
            lock(&state.redo).clear();
            lock(&state.shuffle_bag).clear();
            // Folders changed: forget which files previously failed.
            lock(&state.failed_images).clear();
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

/// Runs off the main thread (`async`): it touches the file system and the shell
/// notification state, and the frontend polls it every two seconds. Doing that
/// work on the UI thread is what makes a window stop responding.
#[tauri::command(async)]
pub fn get_status(app: AppHandle) -> AppStatus {
    let state = app.state::<AppState>();
    let settings = lock(&state.settings).clone();
    let paused_fullscreen =
        settings.pause_on_fullscreen && crate::fullscreen::is_fullscreen_active();
    *lock(&state.is_fullscreen_paused) = paused_fullscreen;
    let manually_paused = *lock(&state.is_paused);
    let is_paused = manually_paused || paused_fullscreen;
    let current_image = lock(&state.current_image).clone();
    let can_previous = !lock(&state.history).is_empty();
    let now = chrono::Local::now();
    let pool = crate::context::active_pool(&settings, now);
    let total_images = scan_directories(&pool).len();
    let time_remaining = if is_paused {
        0
    } else {
        let elapsed = lock(&state.last_change).elapsed().as_secs();
        let interval = settings.interval_minutes as u64 * 60;
        interval.saturating_sub(elapsed)
    };
    let last_error = lock(&state.last_error).clone();

    AppStatus {
        current_image,
        total_images,
        is_paused,
        paused_fullscreen,
        time_remaining,
        can_previous,
        active_profile: profile_label(if settings.context_rules_enabled {
            Some(active_profile(&settings, now))
        } else {
            None
        })
        .to_string(),
        last_error,
    }
}

/// `async` keeps the blocking wallpaper call off the Tauri main thread, so the
/// window keeps responding while the wallpaper is applied.
#[tauri::command(async)]
pub fn next_wallpaper(app: AppHandle) -> bool {
    advance_wallpaper(&app, true)
}

#[tauri::command(async)]
pub fn previous_wallpaper(app: AppHandle) -> bool {
    restore_previous_wallpaper(&app)
}

#[tauri::command]
pub async fn select_folder(app: AppHandle) -> Result<Option<String>, String> {
    let folder = app.dialog().file().blocking_pick_folder();
    Ok(folder.map(|f| f.to_string()))
}

#[tauri::command(async)]
pub fn set_force_mode(app: AppHandle, mode: Option<Profile>) -> Result<(), String> {
    let state = app.state::<AppState>();
    let settings = {
        let mut settings = lock(&state.settings);
        settings.force_mode = mode;
        settings.clone()
    };
    lock(&state.history).clear();
    lock(&state.redo).clear();
    lock(&state.shuffle_bag).clear();
    lock(&state.failed_images).clear();
    save_settings_to_disk(&app, &settings)
}

#[tauri::command]
pub fn start_drag(app: AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.start_dragging();
    }
}

#[tauri::command]
pub fn resize_window(app: AppHandle, width: f64, height: f64) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.set_size(tauri::LogicalSize::new(width, height));
    }
}

#[tauri::command]
pub fn hide_window(app: AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
    }
}

/// Frontend errors are appended to the same log file as the backend ones
/// instead of a stray `frontend_error.log` next to the working directory, which
/// nobody could find.
#[tauri::command(async)]
pub fn log_error(msg: String) {
    logging::error(format!("frontend: {msg}"));
}

/// Reveal the log file in the system file manager and return its path.
#[tauri::command(async)]
pub fn open_log_file() -> Result<String, String> {
    let path = logging::log_path().to_path_buf();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    logging::info(format!("opening log file at {}", path.display()));
    reveal_in_file_manager(&path).map_err(|err| format!("could not open {}: {err}", path.display()))?;
    Ok(path.to_string_lossy().into_owned())
}

#[cfg(target_os = "windows")]
fn reveal_in_file_manager(path: &std::path::Path) -> std::io::Result<()> {
    std::process::Command::new("explorer")
        .arg(format!("/select,{}", path.display()))
        .spawn()
        .map(|_| ())
}

#[cfg(target_os = "macos")]
fn reveal_in_file_manager(path: &std::path::Path) -> std::io::Result<()> {
    std::process::Command::new("open")
        .arg("-R")
        .arg(path)
        .spawn()
        .map(|_| ())
}

#[cfg(all(unix, not(target_os = "macos")))]
fn reveal_in_file_manager(path: &std::path::Path) -> std::io::Result<()> {
    let target = path.parent().unwrap_or(path);
    std::process::Command::new("xdg-open")
        .arg(target)
        .spawn()
        .map(|_| ())
}
