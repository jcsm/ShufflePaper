# Wallpaper Shuffle

> A tiny, open-source wallpaper rotator for Windows.

Wallpaper Shuffle is a lightweight desktop utility that automatically rotates wallpapers from a local folder at a configurable interval.

The goal is to provide the simplest possible experience while remaining fast, native-feeling, and privacy-friendly.

---

# Philosophy

Wallpaper Shuffle follows a few simple principles:

* Do one thing well.
* Zero telemetry.
* Zero ads.
* Zero accounts.
* Minimal resource usage.
* Native desktop experience.
* Open source forever.

The application should feel like a small Windows utility rather than a full desktop application.

---

# Tech Stack

## Frontend

* Vue 3
* TypeScript
* Vite
* Tailwind CSS

## Desktop

* Tauri v2

## Backend

* Rust

## Rust crates

* windows
* serde
* serde_json
* tauri-plugin-store (optional)
* tauri-plugin-dialog
* tauri-plugin-shell (if ever needed)

---

# MVP

The first release intentionally contains very few features.

## Wallpaper Folder

User can select a single folder containing wallpapers.

Supported formats:

* jpg
* jpeg
* png
* bmp
* webp

Subfolders are optional (disabled by default).

---

## Rotation Interval

User selects how often wallpapers change.

Supported values:

* 1 minute
* 5 minutes
* 10 minutes
* 30 minutes
* 1 hour
* 3 hours
* 6 hours
* 12 hours
* 24 hours

---

## Rotation Mode

* Shuffle
* Sequential

Shuffle should avoid repeating images until all images have been displayed once.

---

## Start with Windows

Checkbox.

Enabled by default.

---

## Manual Controls

* Next wallpaper
* Previous wallpaper

---

## Tray Icon

The application lives in the system tray.

Menu:

Next Wallpaper

Pause Rotation

Resume Rotation

Settings

Exit

Closing the settings window does not exit the application.

---

# User Interface

The UI should fit in a single window.

No tabs.

No sidebar.

No wizard.

No onboarding.

Example layout:

---

Wallpaper Folder

[ D:\Wallpapers               ] [ Browse ]

Rotation Interval

[ Every 30 minutes ▼ ]

Rotation Mode

(•) Shuffle

( ) Sequential

☑ Start with Windows

---

Current Status

Running

Images found: 436

Next change:

29 minutes

---

[ Save ]

---

---

# Settings

Stored locally.

Example:

{
"wallpaper_folder": "D:\Wallpapers",
"rotation_interval_minutes": 30,
"rotation_mode": "shuffle",
"start_with_windows": true
}

---

# Non Goals

The following are intentionally NOT part of the project.

* Online wallpaper services
* AI features
* Wallpaper downloads
* Wallpaper collections
* Accounts
* Cloud sync
* Multi-folder support
* Wallpaper editor
* Video wallpapers
* GIF wallpapers
* Slideshow effects
* Animated transitions

These may never be implemented.

---

# Architecture

Frontend

* Settings UI
* Status UI

Backend

WallpaperService

Responsible for changing wallpapers.

ImageScanner

Scans the selected folder.

Scheduler

Triggers wallpaper changes.

SettingsManager

Loads and saves configuration.

TrayManager

Handles tray icon and tray menu.

WindowsService

Windows-specific wallpaper API.

---

# Project Structure

wallpaper-shuffle/

src/

components/

views/

stores/

src-tauri/

src/

wallpaper.rs

scanner.rs

scheduler.rs

settings.rs

tray.rs

windows.rs

Cargo.toml

README.md

LICENSE

---

# Performance Goals

Executable

< 20 MB

RAM

< 40 MB

Idle CPU

0%

Cold startup

< 1 second

Wallpaper change

< 500 ms

---

# Privacy

Wallpaper Shuffle never sends data over the network.

No telemetry.

No analytics.

No tracking.

No advertisements.

Internet permission is never required.

---

# License

MIT License.

---

# Future Ideas

These features are intentionally postponed.

## v1.1

* Recursive folders
* Drag & Drop folder
* Recent folders

## v1.2

* Multi-monitor support

## v1.3

* Ignore small images
* Minimum resolution filter

## v2

* Plugin system

Only features aligned with the project's philosophy should be accepted.

The project should remain small, maintainable, and easy to understand.
