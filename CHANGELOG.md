# Changelog

All notable changes to ShufflePaper are documented here.

## [0.3.0] - 2026-09-13

### Added

- **Smart fullscreen pause** — wallpaper rotation pauses while Windows detects a fullscreen game, video, presentation, or other busy fullscreen activity.
- **Fullscreen status** — the app status panel and system tray tooltip show `Paused (fullscreen)` when automatic pausing is active.
- **Work / Personal profiles** — configure separate wallpaper folders for work and personal use.
- **Scheduled context rules** — automatically use the Work folder on selected weekdays and during configured working hours, then switch to Personal outside that schedule.
- **Manual profile override** — choose `Automatic`, `Force Work`, or `Force Personal` from the settings or system tray.
- **Context status** — the status panel shows the currently active profile.
- **Fixed-height settings layout** — advanced settings use an internal scroll area while the `Save Configuration` button remains visible.
- **Custom settings scrollbar** — added a compact scrollbar styled for both light and dark themes.
- **Information tooltips** — added explanatory tooltips for fullscreen pause and context rules.

### Changed

- Removed the experimental smooth crossfade after it caused unreliable black overlays during wallpaper changes. Wallpaper changes remain instant and reliable.
- Existing settings files remain compatible through defaults for all new options.
- All context-rule settings and profile actions use English labels.

## [0.2.0] - 2026-07-08

### Added

- **Previous button** — go back through your wallpaper history, up to 50 images, with the new **Previous** control in the app window or system tray menu.
- **Undo/redo navigation** — pressing **Next** right after **Previous** returns forward through skipped wallpapers instead of selecting a new one.
- **No-repeat shuffle** — Shuffle mode shows every image in the folder once before repeating any of them. Adding or removing images is picked up automatically on the next rotation cycle. Windows' built-in slideshow cannot do this.

### Fixed

- Manual **Next** correctly resets the auto-rotation timer, so the next-change countdown stays accurate after manual skips.

## [0.1.0] - 2026-06-28

First release of ShufflePaper: a lightweight, open-source wallpaper rotator for Windows. Zero telemetry, zero ads, and zero accounts.

### What's new

- Frameless window with a custom title bar and drag support.
- Auto-resize window based on content.
- Close-to-system-tray button.
- Dark/light theme toggle with system preference support.
- Configurable wallpaper rotation intervals from 1 minute to 24 hours.
- Shuffle and Sequential rotation modes.
- Manual **Next** / **Previous** controls.
- Start with Windows option.
- System tray with a quick menu.
- Local wallpaper-folder browser with image validation.
- Privacy-friendly operation with no network access and no telemetry.

Download `shuffle-paper.exe` and run it. No installation required.

[0.3.0]: https://github.com/jcsm/ShufflePaper/releases/tag/v0.3.0
[0.2.0]: https://github.com/jcsm/ShufflePaper/releases/tag/v0.2.0
[0.1.0]: https://github.com/jcsm/ShufflePaper/releases/tag/v0.1.0
