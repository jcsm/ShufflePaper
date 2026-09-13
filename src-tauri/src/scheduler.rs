use rand::seq::SliceRandom;
use std::sync::Mutex;
use std::thread;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Manager};

use crate::context::active_pool;
use crate::fullscreen::is_fullscreen_active;
use crate::scanner::scan_directories;
use crate::state::AppState;
use crate::wallpaper::set_wallpaper;

const HISTORY_LIMIT: usize = 50;

pub fn start_scheduler(app: AppHandle) {
    thread::spawn(move || loop {
        thread::sleep(Duration::from_secs(1));
        let state = app.state::<AppState>();
        if *state.is_paused.lock().unwrap() {
            continue;
        }

        let settings = state.settings.lock().unwrap().clone();
        let fullscreen = settings.pause_on_fullscreen && is_fullscreen_active();
        *state.is_fullscreen_paused.lock().unwrap() = fullscreen;
        update_tray_tooltip(
            &app,
            fullscreen,
            *state.is_paused.lock().unwrap(),
            &settings,
        );
        if fullscreen || active_pool(&settings, chrono::Local::now()).is_empty() {
            continue;
        }

        let elapsed = state.last_change.lock().unwrap().elapsed().as_secs();
        if elapsed >= settings.interval_minutes as u64 * 60 {
            advance_wallpaper(&app, false);
        }
    });
}

pub fn advance_wallpaper(app: &AppHandle, from_manual: bool) {
    let state = app.state::<AppState>();
    let settings = state.settings.lock().unwrap().clone();
    let pool = active_pool(&settings, chrono::Local::now());
    if pool.is_empty() {
        return;
    }

    if from_manual {
        if let Some(candidate) = pop_existing(&state.redo) {
            commit_wallpaper(&state, candidate, false, false);
            return;
        }
    }

    let images = scan_directories(&pool);
    if images.is_empty() {
        return;
    }
    let selected = if settings.mode == "Random" {
        pick_from_shuffle_bag(&state, &images)
    } else {
        let current = state.current_image.lock().unwrap().clone();
        let pos = current
            .and_then(|c| images.iter().position(|i| i == &c))
            .unwrap_or(0);
        images.get((pos + 1) % images.len()).cloned()
    };
    if let Some(img) = selected {
        commit_wallpaper(&state, img, true, false);
    }
}

pub fn restore_previous_wallpaper(app: &AppHandle) {
    let state = app.state::<AppState>();
    let settings = state.settings.lock().unwrap().clone();
    if active_pool(&settings, chrono::Local::now()).is_empty() {
        return;
    }
    if let Some(candidate) = pop_existing(&state.history) {
        commit_wallpaper(&state, candidate, false, true);
    }
}

fn update_tray_tooltip(
    app: &AppHandle,
    fullscreen: bool,
    manually_paused: bool,
    settings: &crate::settings::AppSettings,
) {
    let label = if fullscreen {
        "Paused (fullscreen)"
    } else if manually_paused {
        "Paused"
    } else {
        "Active"
    };
    let profile = if settings.context_rules_enabled {
        crate::context::profile_label(Some(crate::context::active_profile(
            settings,
            chrono::Local::now(),
        )))
    } else {
        "Automatic"
    };
    if let Some(tray) = app.tray_by_id("main-tray") {
        let _ = tray.set_tooltip(Some(format!("ShufflePaper — {label} — {profile}")));
    }
}

fn pop_existing(stack: &Mutex<Vec<String>>) -> Option<String> {
    let mut entries = stack.lock().unwrap();
    while let Some(candidate) = entries.pop() {
        if std::path::Path::new(&candidate).exists() {
            return Some(candidate);
        }
    }
    None
}

fn pick_from_shuffle_bag(state: &AppState, images: &[String]) -> Option<String> {
    let mut bag = state.shuffle_bag.lock().unwrap();
    for _ in 0..2 {
        if bag.is_empty() {
            *bag = images.to_vec();
            if let Some(current) = state.current_image.lock().unwrap().clone() {
                bag.retain(|i| i != &current);
            }
            bag.shuffle(&mut rand::thread_rng());
        }
        while let Some(candidate) = bag.pop() {
            if std::path::Path::new(&candidate).exists() {
                return Some(candidate);
            }
        }
    }
    images.choose(&mut rand::thread_rng()).cloned()
}

/// Apply a wallpaper and update the history, redo stack, and timer.
pub(crate) fn commit_wallpaper(
    state: &AppState,
    img: String,
    fresh_selection: bool,
    restore_previous: bool,
) {
    if set_wallpaper(&img).is_err() {
        return;
    }
    let previous = {
        let mut current_slot = state.current_image.lock().unwrap();
        current_slot.replace(img)
    };
    if let Some(prev) = previous {
        if restore_previous {
            state.redo.lock().unwrap().push(prev);
        } else {
            let mut history = state.history.lock().unwrap();
            history.push(prev);
            if history.len() > HISTORY_LIMIT {
                let excess = history.len() - HISTORY_LIMIT;
                history.drain(..excess);
            }
        }
    }
    if fresh_selection {
        state.redo.lock().unwrap().clear();
    }
    *state.last_change.lock().unwrap() = Instant::now();
}
