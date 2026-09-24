use rand::seq::SliceRandom;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::Mutex;
use std::thread;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Manager};

use crate::context::active_pool;
use crate::fullscreen::is_fullscreen_active;
use crate::logging;
use crate::scanner::scan_directories;
use crate::settings::AppSettings;
use crate::state::{lock, AppState};
use crate::wallpaper::set_wallpaper;

const HISTORY_LIMIT: usize = 50;

/// How many different images one rotation tick may try before giving up.
/// One unusable file (corrupt, unsupported format) must not turn the whole
/// tick into a silent no-op.
const MAX_ATTEMPTS: usize = 3;

/// Last state reported to the tray, so a change is logged only once instead of
/// every second.
static LAST_STATE_LABEL: Mutex<Option<String>> = Mutex::new(None);

pub fn start_scheduler(app: AppHandle) {
    thread::spawn(move || loop {
        thread::sleep(Duration::from_secs(1));
        // A panic must never end this thread: previously a single unhandled
        // panic silently stopped all wallpaper changes until the app was
        // restarted, with nothing written anywhere.
        if catch_unwind(AssertUnwindSafe(|| tick(&app))).is_err() {
            logging::error("scheduler tick panicked (see the PANIC entry above)");
        }
    });
}

fn tick(app: &AppHandle) {
    let state = app.state::<AppState>();
    let manually_paused = *lock(&state.is_paused);
    if manually_paused {
        return;
    }

    let settings = lock(&state.settings).clone();
    let fullscreen = settings.pause_on_fullscreen && is_fullscreen_active();
    *lock(&state.is_fullscreen_paused) = fullscreen;
    update_tray_tooltip(app, fullscreen, manually_paused, &settings);
    if fullscreen || active_pool(&settings, chrono::Local::now()).is_empty() {
        return;
    }

    let elapsed = lock(&state.last_change).elapsed().as_secs();
    if elapsed >= settings.interval_minutes as u64 * 60 {
        advance_wallpaper(app, false);
    }
}

/// Apply the next wallpaper.
///
/// Returns `true` when a new wallpaper was applied. Tries up to
/// [`MAX_ATTEMPTS`] different images so one bad file cannot leave the desktop
/// (and the countdown) stuck, and backs off a full interval after a total
/// failure instead of retrying once per second.
pub fn advance_wallpaper(app: &AppHandle, from_manual: bool) -> bool {
    let state = app.state::<AppState>();
    let settings = lock(&state.settings).clone();
    let pool = active_pool(&settings, chrono::Local::now());
    if pool.is_empty() {
        return false;
    }

    if from_manual {
        if let Some(candidate) = pop_existing(&state.redo) {
            return commit_wallpaper(&state, candidate, false, false);
        }
    }

    let images = scan_directories(&pool);
    if images.is_empty() {
        return false;
    }
    let candidates = usable_images(&state, &images);

    let mut tried: Vec<String> = Vec::new();
    for _ in 0..MAX_ATTEMPTS {
        let candidate = if settings.mode == "Random" {
            pick_from_shuffle_bag(&state, &candidates, &tried)
        } else {
            select_sequential(&state, &candidates, &tried)
        };
        let Some(candidate) = candidate else {
            break;
        };
        tried.push(candidate.clone());
        if commit_wallpaper(&state, candidate, true, false) {
            return true;
        }
    }

    let message = format!(
        "no wallpaper could be applied from {pool} ({} candidate(s) tried)",
        tried.len()
    );
    logging::error(&message);
    *lock(&state.last_error) = Some(message);
    // Wait a whole interval before trying again so a bad folder does not become
    // a once-per-second retry loop.
    *lock(&state.last_change) = Instant::now();
    false
}

/// Restore the most recent wallpaper from history.
pub fn restore_previous_wallpaper(app: &AppHandle) -> bool {
    let state = app.state::<AppState>();
    let settings = lock(&state.settings).clone();
    if active_pool(&settings, chrono::Local::now()).is_empty() {
        return false;
    }
    match pop_existing(&state.history) {
        Some(candidate) => commit_wallpaper(&state, candidate, false, true),
        None => false,
    }
}

fn update_tray_tooltip(
    app: &AppHandle,
    fullscreen: bool,
    manually_paused: bool,
    settings: &AppSettings,
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
    let text = format!("ShufflePaper — {label} — {profile}");

    // Only touch the tray when the text actually changed: the tray API
    // round-trips to the UI thread, and this used to happen once per second.
    // The transition is also logged, because "rotation stopped" is usually a
    // pause state and that was previously invisible.
    {
        let mut last = lock(&LAST_STATE_LABEL);
        if last.as_deref() == Some(text.as_str()) {
            return;
        }
        logging::info(format!("state changed: {label} — profile {profile}"));
        *last = Some(text.clone());
    }

    if let Some(tray) = app.tray_by_id("main-tray") {
        let _ = tray.set_tooltip(Some(text));
    }
}

fn pop_existing(stack: &Mutex<Vec<String>>) -> Option<String> {
    let mut entries = lock(stack);
    while let Some(candidate) = entries.pop() {
        if std::path::Path::new(&candidate).exists() {
            return Some(candidate);
        }
    }
    None
}

/// Images that have not already failed to apply this session.
///
/// If every image failed (for example the whole folder is unreadable) the full
/// list is used again, so a transient error can never empty the rotation.
fn usable_images(state: &AppState, images: &[String]) -> Vec<String> {
    let failed = lock(&state.failed_images);
    if failed.is_empty() {
        return images.to_vec();
    }
    let usable: Vec<String> = images
        .iter()
        .filter(|image| !failed.iter().any(|failed| failed == *image))
        .cloned()
        .collect();
    if usable.is_empty() {
        images.to_vec()
    } else {
        usable
    }
}

fn pick_from_shuffle_bag(
    state: &AppState,
    images: &[String],
    exclude: &[String],
) -> Option<String> {
    let mut bag = lock(&state.shuffle_bag);
    for _ in 0..2 {
        if bag.is_empty() {
            *bag = images.to_vec();
            if let Some(current) = lock(&state.current_image).clone() {
                bag.retain(|image| image != &current);
            }
            bag.retain(|image| !exclude.contains(image));
            bag.shuffle(&mut rand::thread_rng());
        }
        while let Some(candidate) = bag.pop() {
            if std::path::Path::new(&candidate).exists() && !exclude.contains(&candidate) {
                return Some(candidate);
            }
        }
    }
    images
        .iter()
        .find(|image| !exclude.contains(image))
        .cloned()
}

fn select_sequential(state: &AppState, images: &[String], exclude: &[String]) -> Option<String> {
    let current = lock(&state.current_image).clone();
    let start = current
        .and_then(|c| images.iter().position(|image| image == &c))
        .map(|position| position + 1)
        .unwrap_or(0);
    (0..images.len())
        .map(|offset| &images[(start + offset) % images.len()])
        .find(|candidate| !exclude.contains(candidate))
        .cloned()
}

/// Apply a wallpaper and update the history, redo stack, and timer.
///
/// Returns `false` (without touching the timer) when the image could not be
/// applied, so the caller can fall back to another candidate.
pub(crate) fn commit_wallpaper(
    state: &AppState,
    img: String,
    fresh_selection: bool,
    restore_previous: bool,
) -> bool {
    if let Err(err) = set_wallpaper(&img) {
        let message = format!("failed to apply wallpaper {img}: {err}");
        logging::error(&message);
        *lock(&state.last_error) = Some(message);
        let mut failed = lock(&state.failed_images);
        if !failed.iter().any(|entry| entry == &img) {
            failed.push(img);
        }
        return false;
    }

    logging::info(format!("wallpaper applied: {}", file_name(&img)));

    let previous = lock(&state.current_image).replace(img);
    if let Some(prev) = previous {
        if restore_previous {
            lock(&state.redo).push(prev);
        } else {
            let mut history = lock(&state.history);
            history.push(prev);
            if history.len() > HISTORY_LIMIT {
                let excess = history.len() - HISTORY_LIMIT;
                history.drain(..excess);
            }
        }
    }
    if fresh_selection {
        lock(&state.redo).clear();
    }
    *lock(&state.last_error) = None;
    *lock(&state.last_change) = Instant::now();
    true
}

fn file_name(path: &str) -> String {
    std::path::Path::new(path)
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.to_string())
}
