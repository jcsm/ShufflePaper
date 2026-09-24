//! Persistent logging.
//!
//! A packaged Windows build runs with `windows_subsystem = "windows"`, so
//! there is no console and every `println!`/`eprintln!` is lost. A hang that
//! "leaves no log at all" is impossible to diagnose after the fact, so every
//! interesting event is also appended to a rotating file:
//!
//!   %LOCALAPPDATA%\ShufflePaper\logs\shufflepaper.log
//!
//! Failures are also mirrored into `AppState::last_error` so they show up in
//! the UI instead of only on disk.

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

/// Rotate once the log grows past this size. Small enough to read, large
/// enough to keep a few days of rotation history.
const MAX_LOG_BYTES: u64 = 1_000_000;

static LOG_PATH: OnceLock<PathBuf> = OnceLock::new();

/// Location of the log file, created on first use.
pub fn log_path() -> &'static Path {
    LOG_PATH.get_or_init(|| {
        let base = std::env::var_os("LOCALAPPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(std::env::temp_dir);
        let dir = base.join("ShufflePaper").join("logs");
        let _ = fs::create_dir_all(&dir);
        dir.join("shufflepaper.log")
    })
}

/// Append one timestamped line to the log (and to the console in dev builds).
pub fn log(level: &str, message: impl AsRef<str>) {
    let line = format!(
        "{} [{level}] {}\n",
        chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f"),
        message.as_ref()
    );

    // Visible when running `npm run tauri dev`.
    eprint!("{line}");

    let path = log_path();
    rotate_if_needed(path);
    match OpenOptions::new().create(true).append(true).open(path) {
        Ok(mut file) => {
            let _ = file.write_all(line.as_bytes());
        }
        Err(_) => {}
    }
}

pub fn info(message: impl AsRef<str>) {
    log("INFO", message);
}

pub fn warn(message: impl AsRef<str>) {
    log("WARN", message);
}

pub fn error(message: impl AsRef<str>) {
    log("ERROR", message);
}

/// Keep the log from growing without bound: one previous file is retained as
/// `shufflepaper.log.1`.
fn rotate_if_needed(path: &Path) {
    let too_big = fs::metadata(path)
        .map(|meta| meta.len() > MAX_LOG_BYTES)
        .unwrap_or(false);
    if !too_big {
        return;
    }
    let backup = path.with_extension("log.1");
    let _ = fs::remove_file(&backup);
    let _ = fs::rename(path, &backup);
}

/// Log panics instead of losing them.
///
/// Without this a panic in the scheduler thread or in a command kills that
/// thread silently, which is exactly the "wallpapers stop changing and there
/// is nothing in any log" symptom.
pub fn install_panic_hook() {
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let location = info
            .location()
            .map(|l| format!("{}:{}", l.file(), l.line()))
            .unwrap_or_else(|| "unknown location".to_string());
        let payload = info
            .payload()
            .downcast_ref::<&str>()
            .map(|s| (*s).to_string())
            .or_else(|| info.payload().downcast_ref::<String>().cloned())
            .unwrap_or_else(|| "non-string panic payload".to_string());

        error(format!("PANIC at {location}: {payload}"));
        default_hook(info);
    }));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn log_path_is_created_under_app_data() {
        let path = log_path();
        assert!(path.ends_with("shufflepaper.log"));
        assert!(path.parent().map(|p| p.exists()).unwrap_or(false));
    }

    #[test]
    fn log_appends_lines() {
        let before = fs::read_to_string(log_path()).unwrap_or_default().len();
        info("test entry");
        let after = fs::read_to_string(log_path()).unwrap_or_default();
        assert!(after.len() > before);
        assert!(after.contains("test entry"));
    }
}
