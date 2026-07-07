use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager,
};
use crate::state::AppState;
use crate::scheduler::force_next_wallpaper;

pub fn create_tray(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let next_i = MenuItem::with_id(app, "next", "Next Wallpaper", true, None::<&str>)?;
    let toggle_i = MenuItem::with_id(app, "toggle", "Pause/Resume", true, None::<&str>)?;
    let settings_i = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?;
    let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;

    let menu = Menu::with_items(app, &[&next_i, &toggle_i, &settings_i, &quit_i])?;

    let icon = app.default_window_icon().cloned();
    let _tray = if let Some(icon) = icon {
        TrayIconBuilder::new()
            .icon(icon)
            .menu(&menu)
            .show_menu_on_left_click(false)
            .on_menu_event(|app, event| match event.id.as_ref() {
                "next" => {
                    force_next_wallpaper(app);
                }
                "toggle" => {
                    let state = app.state::<AppState>();
                    let mut paused = state.is_paused.lock().unwrap();
                    *paused = !*paused;
                }
                "settings" => {
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
                "quit" => {
                    app.exit(0);
                }
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
            .build(app)?
    } else {
        return Err("No default window icon available for tray".into());
    };

    Ok(())
}
