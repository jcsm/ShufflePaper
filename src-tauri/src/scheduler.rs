use std::cell::RefCell;
use std::thread;
use std::time::Duration;
use tauri::{AppHandle, Manager};
use rand::seq::SliceRandom;

use crate::state::AppState;
use crate::scanner::scan_directories;
use crate::wallpaper::set_wallpaper;

thread_local! {
    static LAST_CHANGE: RefCell<std::time::Instant> = RefCell::new(std::time::Instant::now());
}

pub fn last_change_time() -> std::time::Instant {
    LAST_CHANGE.with(|c| *c.borrow())
}

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

            let elapsed = last_change_time().elapsed().as_secs();
            let interval_secs = (settings.interval_minutes as u64) * 60;
            
            if elapsed >= interval_secs {
                let images = scan_directories(&settings.folder_path);
                if !images.is_empty() {
                    let mut rng = rand::thread_rng();
                    let selected = if settings.mode == "Random" {
                        images.choose(&mut rng).cloned()
                    } else {
                        let current = state.current_image.lock().unwrap().clone();
                        let pos = current.and_then(|c| images.iter().position(|i| i == &c)).unwrap_or(0);
                        let next_pos = (pos + 1) % images.len();
                        images.get(next_pos).cloned()
                    };

                    if let Some(img) = selected {
                        if set_wallpaper(&img).is_ok() {
                            *state.current_image.lock().unwrap() = Some(img);
                            LAST_CHANGE.with(|c| *c.borrow_mut() = std::time::Instant::now());
                        }
                    }
                }
            }
        }
    });
}

pub fn force_next_wallpaper(app: &AppHandle) {
    let state = app.state::<AppState>();
    let settings = state.settings.lock().unwrap().clone();
    if settings.folder_path.is_empty() {
        return;
    }
    
    let images = scan_directories(&settings.folder_path);
    if !images.is_empty() {
        let mut rng = rand::thread_rng();
        let selected = if settings.mode == "Random" {
            images.choose(&mut rng).cloned()
        } else {
            let current = state.current_image.lock().unwrap().clone();
            let pos = current.and_then(|c| images.iter().position(|i| i == &c)).unwrap_or(0);
            let next_pos = (pos + 1) % images.len();
            images.get(next_pos).cloned()
        };

        if let Some(img) = selected {
            if set_wallpaper(&img).is_ok() {
                *state.current_image.lock().unwrap() = Some(img);
                LAST_CHANGE.with(|c| *c.borrow_mut() = std::time::Instant::now());
            }
        }
    }
}
