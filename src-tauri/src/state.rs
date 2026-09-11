use std::sync::Mutex;
use std::time::Instant;
use crate::settings::AppSettings;

pub struct AppState {
    pub settings: Mutex<AppSettings>,
    pub current_image: Mutex<Option<String>>,
    pub is_paused: Mutex<bool>,
    /// Previously shown wallpapers (most recent last), capped at HISTORY_LIMIT.
    pub history: Mutex<Vec<String>>,
    /// Wallpapers skipped by Previous; replayed by Next until a new selection happens.
    pub redo: Mutex<Vec<String>>,
    /// Shuffle bag for no-repeat random rotation.
    pub shuffle_bag: Mutex<Vec<String>>,
    /// When the wallpaper last changed (shared across threads).
    pub last_change: Mutex<Instant>,
}
