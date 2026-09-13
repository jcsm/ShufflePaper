use crate::context::Profile;
use crate::scheduler::{advance_wallpaper, restore_previous_wallpaper};
use crate::state::AppState;
use tauri::{
    menu::{Menu, MenuItem, Submenu},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager,
};

pub fn create_tray(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let prev_i = MenuItem::with_id(app, "previous", "Previous Wallpaper", true, None::<&str>)?;
    let next_i = MenuItem::with_id(app, "next", "Next Wallpaper", true, None::<&str>)?;
    let toggle_i = MenuItem::with_id(app, "toggle", "Pause/Resume", true, None::<&str>)?;
    let work_i = MenuItem::with_id(app, "force_work", "Force: Work", true, None::<&str>)?;
    let personal_i =
        MenuItem::with_id(app, "force_personal", "Force: Personal", true, None::<&str>)?;
    let auto_i = MenuItem::with_id(app, "force_auto", "Automatic", true, None::<&str>)?;
    let profile_menu = Submenu::with_items(app, "Profile", true, &[&work_i, &personal_i, &auto_i])?;
    let settings_i = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?;
    let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let menu = Menu::with_items(
        app,
        &[
            &prev_i,
            &next_i,
            &toggle_i,
            &profile_menu,
            &settings_i,
            &quit_i,
        ],
    )?;

    let icon = app.default_window_icon().cloned();
    if let Some(icon) = icon {
        TrayIconBuilder::with_id("main-tray")
            .icon(icon)
            .tooltip("ShufflePaper")
            .menu(&menu)
            .show_menu_on_left_click(false)
            .on_menu_event(|app, event| match event.id.as_ref() {
                "previous" => restore_previous_wallpaper(app),
                "next" => advance_wallpaper(app, true),
                "toggle" => {
                    let state = app.state::<AppState>();
                    let mut paused = state.is_paused.lock().unwrap();
                    *paused = !*paused;
                }
                "force_work" => {
                    let _ = set_force_mode(app, Some(Profile::Work));
                }
                "force_personal" => {
                    let _ = set_force_mode(app, Some(Profile::Personal));
                }
                "force_auto" => {
                    let _ = set_force_mode(app, None);
                }
                "settings" => {
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
                "quit" => app.exit(0),
                _ => {}
            })
            .on_tray_icon_event(|tray, event| {
                if let TrayIconEvent::Click {
                    button: MouseButton::Left,
                    button_state: MouseButtonState::Up,
                    ..
                } = event
                {
                    let app = tray.app_handle();
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
            })
            .build(app)?;
        Ok(())
    } else {
        Err("No default window icon available for tray".into())
    }
}

fn set_force_mode(app: &AppHandle, mode: Option<Profile>) -> Result<(), String> {
    let state = app.state::<AppState>();
    let settings = {
        let mut settings = state.settings.lock().unwrap();
        settings.force_mode = mode;
        settings.clone()
    };
    state.history.lock().unwrap().clear();
    state.redo.lock().unwrap().clear();
    state.shuffle_bag.lock().unwrap().clear();
    crate::settings::save_settings(app, &settings)
}
