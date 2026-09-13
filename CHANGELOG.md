# Changelog

All notable changes to ShufflePaper are documented here.

## [0.3.0] - 2026-09-13

### Added

- Smart fullscreen pause using Windows notification state detection.
- Optional pause while games, videos, presentations, or other fullscreen apps are active.
- Work and Personal wallpaper profiles with configurable folders.
- Automatic profile switching by work days and working hours.
- Manual profile override from the system tray or settings (`Automatic`, `Force Work`, and `Force Personal`).
- Status reporting for fullscreen pause state and the active profile.
- Custom styled settings scrollbar and informational tooltips.

### Changed

- Wallpaper rotation remains instant and reliable; the experimental smooth crossfade was removed.
- The settings window is constrained to 800px and scrolls internally when advanced settings are expanded.
- All context-rule labels and tray profile actions use English.
- Existing settings files remain compatible through defaults for new options.

[0.3.0]: https://github.com/jcsm/ShufflePaper/releases/tag/v0.3.0
