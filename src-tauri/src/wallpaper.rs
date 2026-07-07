#[cfg(target_os = "windows")]
use std::os::windows::ffi::OsStrExt;

pub fn set_wallpaper(path: &str) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        use windows::Win32::UI::WindowsAndMessaging::{
            SystemParametersInfoW, SPI_SETDESKWALLPAPER, SPIF_UPDATEINIFILE, SPIF_SENDWININICHANGE,
        };

        let mut path_u16: Vec<u16> = std::ffi::OsStr::new(path).encode_wide().collect();
        path_u16.push(0);

        let result = unsafe {
            SystemParametersInfoW(
                SPI_SETDESKWALLPAPER,
                0,
                Some(path_u16.as_mut_ptr() as *mut _),
                SPIF_UPDATEINIFILE | SPIF_SENDWININICHANGE,
            )
        };

        if result.is_ok() {
            Ok(())
        } else {
            Err("Failed to set wallpaper".to_string())
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        println!("Wallpaper setting only implemented for Windows in this version. Path: {}", path);
        Ok(())
    }
}
