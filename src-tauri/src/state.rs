use std::sync::Mutex;
use crate::settings::AppSettings;

pub struct AppState {
    pub settings: Mutex<AppSettings>,
    pub current_image: Mutex<Option<String>>,
    pub is_paused: Mutex<bool>,
}
