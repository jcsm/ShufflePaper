use std::sync::Mutex;
use std::thread;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Manager};
use rand::seq::SliceRandom;

use crate::state::AppState;
use crate::scanner::scan_directories;
use crate::wallpaper::set_wallpaper;

/// How many previously shown wallpapers can be walked back through.
const HISTORY_LIMIT: usize = 50;

pub fn start_scheduler(app: AppHandle) {
    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_secs(1));

            let state = app.state::<AppState>();
            let is_paused = *state.is_paused.lock().unwrap();

            if is_paused {
                continue;
            }

            let settings = state.settings.lock().unwrap().clone();
            if settings.folder_path.is_empty() {
                continue;
            }

            let elapsed = state.last_change.lock().unwrap().elapsed().as_secs();
            let interval_secs = (settings.interval_minutes as u64) * 60;

            if elapsed >= interval_secs {
                advance_wallpaper(&app, false);
            }
        }
    });
}

/// Pick the next wallpaper and apply it.
///
/// Random mode uses a shuffle bag: every image in the folder is shown once
/// before any of them can repeat. Sequential mode walks the folder cyclically.
///
/// Manual advances (`from_manual = true`) replay the redo stack first, so
/// pressing Next right after Previous returns to the skipped wallpaper without
/// consuming a shuffle-bag entry. Scheduled changes ignore the redo stack and
/// always make a fresh selection.
pub fn advance_wallpaper(app: &AppHandle, from_manual: bool) {
    let state = app.state::<AppState>();
    let settings = state.settings.lock().unwrap().clone();
    if settings.folder_path.is_empty() {
        return;
    }

    if from_manual {
        if let Some(candidate) = pop_existing(&state.redo) {
            commit_wallpaper(&state, candidate, false);
            return;
        }
    }

    let images = scan_directories(&settings.folder_path);
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
        let next_pos = (pos + 1) % images.len();
        images.get(next_pos).cloned()
    };

    if let Some(img) = selected {
        commit_wallpaper(&state, img, true);
    }
}

/// Walk back through the history stack to the most recent wallpaper that still
/// exists on disk and restore it. The current wallpaper is pushed onto the
/// redo stack so a subsequent Next returns forward through it.
pub fn restore_previous_wallpaper(app: &AppHandle) {
    let state = app.state::<AppState>();
    let settings = state.settings.lock().unwrap().clone();
    if settings.folder_path.is_empty() {
        return;
    }

    if let Some(candidate) = pop_existing(&state.history) {
        if set_wallpaper(&candidate).is_ok() {
            let previous = {
                let mut current_slot = state.current_image.lock().unwrap();
                current_slot.replace(candidate)
            };
            if let Some(prev) = previous {
                state.redo.lock().unwrap().push(prev);
            }
            *state.last_change.lock().unwrap() = Instant::now();
        } else {
            // The change failed; put the candidate back so it isn't lost.
            state.history.lock().unwrap().push(candidate);
        }
    }
}

/// Pop the most recent entry that still exists on disk, skipping deleted files.
fn pop_existing(stack: &Mutex<Vec<String>>) -> Option<String> {
    let mut entries = stack.lock().unwrap();
    while let Some(candidate) = entries.pop() {
        if std::path::Path::new(&candidate).exists() {
            return Some(candidate);
        }
    }
    None
}

/// Draw the next image from the shuffle bag.
///
/// The bag is refilled from a fresh folder scan whenever it runs empty, so
/// every image is shown once before any repeats and folder edits are picked
/// up on the next refill.
fn pick_from_shuffle_bag(state: &AppState, images: &[String]) -> Option<String> {
    let mut bag = state.shuffle_bag.lock().unwrap();

    // Two passes: drain stale entries first, then refill once more if needed.
    for _ in 0..2 {
        if bag.is_empty() {
            *bag = images.to_vec();
            // Don't immediately repeat the image that is currently shown.
            if let Some(current) = state.current_image.lock().unwrap().clone() {
                bag.retain(|i| i != &current);
            }
            bag.shuffle(&mut rand::thread_rng());
        }

        while let Some(candidate) = bag.pop() {
            if std::path::Path::new(&candidate).exists() {
                return Some(candidate);
            }
            // Stale entry (file deleted or moved); skip it.
        }
    }

    // Everything was stale (the folder emptied underneath us). Fall back to a
    // straight random pick so rotation still happens.
    images.choose(&mut rand::thread_rng()).cloned()
}

/// Apply a wallpaper and update the history / redo / timer bookkeeping.
///
/// `fresh_selection = false` marks a redo replay, which keeps the remaining
/// redo entries intact. Any fresh selection invalidates the redo stack.
fn commit_wallpaper(state: &AppState, img: String, fresh_selection: bool) {
    if set_wallpaper(&img).is_err() {
        return;
    }

    let previous = {
        let mut current_slot = state.current_image.lock().unwrap();
        current_slot.replace(img)
    };

    if let Some(prev) = previous {
        let mut history = state.history.lock().unwrap();
        history.push(prev);
        if history.len() > HISTORY_LIMIT {
            let excess = history.len() - HISTORY_LIMIT;
            history.drain(..excess);
        }
    }

    if fresh_selection {
        state.redo.lock().unwrap().clear();
    }
    *state.last_change.lock().unwrap() = Instant::now();
}
