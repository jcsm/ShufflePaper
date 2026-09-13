use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

use crate::context::Profile;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub folder_path: String,
    pub interval_minutes: u32,
    pub mode: String,
    pub autostart: bool,
    #[serde(default = "default_theme")]
    pub theme: String,
    #[serde(default = "default_true")]
    pub pause_on_fullscreen: bool,
    #[serde(default)]
    pub context_rules_enabled: bool,
    #[serde(default)]
    pub work_folder: Option<PathBuf>,
    #[serde(default)]
    pub personal_folder: Option<PathBuf>,
    #[serde(default = "default_work_days")]
    pub work_days: [bool; 7],
    #[serde(default = "default_work_start")]
    pub work_start_hour: u8,
    #[serde(default = "default_work_end")]
    pub work_end_hour: u8,
    #[serde(default)]
    pub force_mode: Option<Profile>,
}

fn default_theme() -> String {
    "system".to_string()
}
fn default_true() -> bool {
    true
}
fn default_work_days() -> [bool; 7] {
    [true, true, true, true, true, false, false]
}
fn default_work_start() -> u8 {
    9
}
fn default_work_end() -> u8 {
    18
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            folder_path: String::new(),
            interval_minutes: 15,
            mode: "Random".into(),
            autostart: false,
            theme: default_theme(),
            pause_on_fullscreen: true,
            context_rules_enabled: false,
            work_folder: None,
            personal_folder: None,
            work_days: default_work_days(),
            work_start_hour: default_work_start(),
            work_end_hour: default_work_end(),
            force_mode: None,
        }
    }
}

pub fn get_settings_path(app: &AppHandle) -> PathBuf {
    app.path().app_config_dir().unwrap().join("settings.json")
}

pub fn load_settings(app: &AppHandle) -> AppSettings {
    let path = get_settings_path(app);
    if path.exists() {
        if let Ok(content) = fs::read_to_string(&path) {
            if let Ok(settings) = serde_json::from_str(&content) {
                return settings;
            }
        }
    }
    AppSettings::default()
}

pub fn save_settings(app: &AppHandle, settings: &AppSettings) -> Result<(), String> {
    let path = get_settings_path(app);
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let content = serde_json::to_string_pretty(settings).map_err(|e| e.to_string())?;
    fs::write(path, content).map_err(|e| e.to_string())?;
    Ok(())
}
