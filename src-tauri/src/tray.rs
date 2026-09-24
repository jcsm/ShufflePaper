use crate::context::Profile;
use crate::logging;
use crate::scheduler::{advance_wallpaper, restore_previous_wallpaper};
use crate::state::{lock, AppState};
use std::thread;
use tauri::{
    menu::{Menu, MenuItem, Submenu},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager,
};

/// Run a wallpaper action on a worker thread.
///
/// Tray and menu callbacks are delivered on the Tauri main thread. Applying a
/// wallpaper there used to broadcast a blocking system message from the UI
/// thread itself, which left the window unresponsive and the wallpaper
/// unchanged until the app was restarted.
fn run_off_main_thread(name: &str, task: impl FnOnce() + Send + 'static) {
    if let Err(err) = thread::Builder::new()
        .name(format!("shufflepaper-{name}"))
        .spawn(task)
    {
        logging::error(format!("could not spawn {name} task: {err}"));
    }
}

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
                "previous" => {
                    let app = app.clone();
                    run_off_main_thread("previous", move || {
                        restore_previous_wallpaper(&app);
                    });
                }
                "next" => {
                    let app = app.clone();
                    run_off_main_thread("next", move || {
                        advance_wallpaper(&app, true);
                    });
                }
                "toggle" => {
                    let state = app.state::<AppState>();
                    let mut paused = lock(&state.is_paused);
                    *paused = !*paused;
                    logging::info(if *paused {
                        "rotation paused from tray"
                    } else {
                        "rotation resumed from tray"
                    });
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
                "quit" => {
                    logging::info("exit requested from tray");
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
            .build(app)?;
        Ok(())
    } else {
        Err("No default window icon available for tray".into())
    }
}

fn set_force_mode(app: &AppHandle, mode: Option<Profile>) -> Result<(), String> {
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
    logging::info(format!(
        "profile override: {}",
        crate::context::profile_label(mode)
    ));
    crate::settings::save_settings(app, &settings)
}
