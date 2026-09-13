#[cfg(target_os = "windows")]
pub fn is_fullscreen_active() -> bool {
    use windows::Win32::UI::Shell::{
        SHQueryUserNotificationState, QUNS_BUSY, QUNS_PRESENTATION_MODE,
        QUNS_RUNNING_D3D_FULL_SCREEN,
    };

    unsafe {
        SHQueryUserNotificationState()
            .map(|state| {
                state == QUNS_RUNNING_D3D_FULL_SCREEN
                    || state == QUNS_PRESENTATION_MODE
                    || state == QUNS_BUSY
            })
            .unwrap_or(false)
    }
}

#[cfg(not(target_os = "windows"))]
pub fn is_fullscreen_active() -> bool {
    false
}
