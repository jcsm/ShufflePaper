use chrono::{Datelike, Timelike};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::settings::AppSettings;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Profile {
    Work,
    Personal,
}

/// Resolve the folder used for the next selection. Missing profile folders fall
/// back to the legacy single folder so old configurations keep working.
pub fn active_pool(settings: &AppSettings, now: chrono::DateTime<chrono::Local>) -> String {
    if !settings.context_rules_enabled {
        return settings.folder_path.clone();
    }

    let profile = settings.force_mode.unwrap_or_else(|| {
        let weekday = now.weekday().num_days_from_monday() as usize;
        let hour = now.hour() as u8;
        let start = settings.work_start_hour.min(23);
        let end = settings.work_end_hour.min(23);
        let in_hours = if start <= end {
            hour >= start && hour < end
        } else {
            hour >= start || hour < end
        };

        if settings.work_days.get(weekday).copied().unwrap_or(false) && in_hours {
            Profile::Work
        } else {
            Profile::Personal
        }
    });

    match profile {
        Profile::Work => settings.work_folder.as_ref(),
        Profile::Personal => settings.personal_folder.as_ref(),
    }
    .filter(|folder| !folder.as_os_str().is_empty())
    .cloned()
    .unwrap_or_else(|| PathBuf::from(&settings.folder_path))
    .to_string_lossy()
    .into_owned()
}

pub fn active_profile(settings: &AppSettings, now: chrono::DateTime<chrono::Local>) -> Profile {
    if let Some(profile) = settings.force_mode {
        return profile;
    }
    let weekday = now.weekday().num_days_from_monday() as usize;
    let hour = now.hour() as u8;
    let start = settings.work_start_hour.min(23);
    let end = settings.work_end_hour.min(23);
    let in_hours = if start <= end {
        hour >= start && hour < end
    } else {
        hour >= start || hour < end
    };
    if settings.context_rules_enabled
        && settings.work_days.get(weekday).copied().unwrap_or(false)
        && in_hours
    {
        Profile::Work
    } else {
        Profile::Personal
    }
}

pub fn profile_label(profile: Option<Profile>) -> &'static str {
    match profile {
        Some(Profile::Work) => "Work",
        Some(Profile::Personal) => "Personal",
        None => "Automatic",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Local, TimeZone};

    fn settings() -> AppSettings {
        AppSettings {
            context_rules_enabled: true,
            work_folder: Some(PathBuf::from("work")),
            personal_folder: Some(PathBuf::from("personal")),
            ..AppSettings::default()
        }
    }

    #[test]
    fn selects_work_profile_during_work_hours() {
        let now = Local.with_ymd_and_hms(2026, 9, 7, 10, 0, 0).unwrap();
        assert_eq!(active_pool(&settings(), now), "work");
    }

    #[test]
    fn selects_personal_profile_outside_work_hours() {
        let now = Local.with_ymd_and_hms(2026, 9, 7, 20, 0, 0).unwrap();
        assert_eq!(active_pool(&settings(), now), "personal");
    }
}
