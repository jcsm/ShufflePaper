#[cfg(target_os = "windows")]
use std::os::windows::ffi::OsStrExt;

use crate::logging;

/// Apply `path` as the desktop wallpaper.
///
/// Returns an error when the change definitely did not happen (missing file,
/// failed SVG rasterization, failed API call). Formats the Windows wallpaper
/// API cannot decode (notably `.webp`) are logged as warnings but cannot be
/// detected reliably, because the API reports success for them anyway.
///
/// This function is safe to call from any thread, but it must never be called
/// on the Tauri main/UI thread — see the note on the broadcast below.
pub fn set_wallpaper(path: &str) -> Result<(), String> {
    // SVG cannot be passed to the Windows wallpaper API; render it to a
    // cached PNG at screen resolution and apply that instead.
    let path = if path.to_lowercase().ends_with(".svg") {
        let (width, height) = screen_size();
        crate::svg::render_to_cached_png(path, width, height, crate::svg::FitMode::Cover)
            .map_err(|err| {
                let message = format!("SVG rasterization failed for {path}: {err}");
                logging::error(&message);
                message
            })?
            .to_string_lossy()
            .into_owned()
    } else {
        path.to_string()
    };

    if !std::path::Path::new(&path).exists() {
        return Err(format!("wallpaper file is missing: {path}"));
    }

    // The wallpaper API is documented for BMP and in practice decodes
    // JPEG/PNG/GIF through WIC. For anything else (webp, tiff, heic, ...) the
    // call can report success while the desktop stays unchanged or turns
    // black, so leave a trace that makes that diagnosable.
    let extension = std::path::Path::new(&path)
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_lowercase())
        .unwrap_or_default();
    if !matches!(extension.as_str(), "bmp" | "jpg" | "jpeg" | "png" | "gif") {
        logging::warn(format!(
            "{path} is a '.{extension}' file; the Windows wallpaper API may not decode it, \
             in which case the desktop stays unchanged even though the change reports success"
        ));
    }

    #[cfg(target_os = "windows")]
    {
        use windows::Win32::Foundation::{LPARAM, WPARAM};
        use windows::Win32::UI::WindowsAndMessaging::{
            SendNotifyMessageW, SystemParametersInfoW, HWND_BROADCAST, SPI_SETDESKWALLPAPER,
            SPIF_UPDATEINIFILE, WM_SETTINGCHANGE,
        };

        let mut path_u16: Vec<u16> = std::ffi::OsStr::new(&path).encode_wide().collect();
        path_u16.push(0);

        // Deliberately *without* SPIF_SENDWININICHANGE: that flag makes
        // SystemParametersInfo broadcast WM_SETTINGCHANGE with a blocking
        // SendMessage to HWND_BROADCAST, which only returns once every
        // top-level window on the system — our own webview windows included —
        // has answered. One busy or hung window therefore stalls the caller for
        // seconds or indefinitely; when the caller was the UI thread the whole
        // window stopped pumping messages and Windows replaced it with an
        // unresponsive "ghost" window (blank frame with a spinner), which is
        // what made the app look hung in a corner until it was restarted.
        let result = unsafe {
            SystemParametersInfoW(
                SPI_SETDESKWALLPAPER,
                0,
                Some(path_u16.as_mut_ptr() as *mut _),
                SPIF_UPDATEINIFILE,
            )
        };

        if let Err(err) = result {
            return Err(format!("SystemParametersInfoW failed for {path}: {err}"));
        }

        // Tell the rest of the system about the change without waiting for a
        // reply: SendNotifyMessage never blocks the calling thread, so an
        // unresponsive application can no longer hang us.
        let _ = unsafe {
            SendNotifyMessageW(
                HWND_BROADCAST,
                WM_SETTINGCHANGE,
                WPARAM(SPI_SETDESKWALLPAPER.0 as usize),
                LPARAM(0),
            )
        };

        Ok(())
    }
    #[cfg(not(target_os = "windows"))]
    {
        println!("Wallpaper setting only implemented for Windows in this version. Path: {}", path);
        Ok(())
    }
}

/// Primary-screen size in pixels, used to rasterize SVG wallpapers.
/// Falls back to 1920x1080 when the metrics are unavailable.
#[cfg(target_os = "windows")]
fn screen_size() -> (u32, u32) {
    use windows::Win32::UI::WindowsAndMessaging::{GetSystemMetrics, SM_CXSCREEN, SM_CYSCREEN};

    unsafe {
        let (w, h) = (GetSystemMetrics(SM_CXSCREEN), GetSystemMetrics(SM_CYSCREEN));
        if w > 0 && h > 0 {
            (w as u32, h as u32)
        } else {
            (1920, 1080)
        }
    }
}

#[cfg(not(target_os = "windows"))]
fn screen_size() -> (u32, u32) {
    (1920, 1080)
}
