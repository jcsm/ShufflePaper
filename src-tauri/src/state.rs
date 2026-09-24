use crate::settings::AppSettings;
use std::sync::{Mutex, MutexGuard};
use std::time::Instant;

pub struct AppState {
    pub settings: Mutex<AppSettings>,
    pub current_image: Mutex<Option<String>>,
    pub is_paused: Mutex<bool>,
    pub is_fullscreen_paused: Mutex<bool>,
    /// Previously shown wallpapers (most recent last), capped at HISTORY_LIMIT.
    pub history: Mutex<Vec<String>>,
    /// Wallpapers skipped by Previous; replayed by Next until a new selection happens.
    pub redo: Mutex<Vec<String>>,
    /// Shuffle bag for no-repeat random rotation.
    pub shuffle_bag: Mutex<Vec<String>>,
    /// When the wallpaper last changed (shared across threads).
    pub last_change: Mutex<Instant>,
    /// Wallpapers that could not be applied this session, so an unusable file
    /// is not picked again on every tick.
    pub failed_images: Mutex<Vec<String>>,
    /// Last wallpaper failure, surfaced to the UI so errors are not invisible.
    pub last_error: Mutex<Option<String>>,
}

/// Lock a mutex, recovering its data even if a previous holder panicked.
///
/// With the usual `.lock().unwrap()` a single panic poisons the mutex and every
/// later lock panics too, turning one recoverable error into a permanently dead
/// app that only a restart could fix. Everything stored here is independent
/// plain data (settings, paths, timestamps) with no invariant that a panic
/// could leave half-updated, so using the inner value is safe.
pub(crate) fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}
