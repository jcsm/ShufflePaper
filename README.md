<p align="center">
  <img src="src-tauri/icons/icon.png" alt="ShufflePaper icon" width="100" />
</p>

# ShufflePaper

![ShufflePaper screenshot](screenshot.png)

<p>
  <a href="https://github.com/jcsm/ShufflePaper/releases/latest">
    <img src="https://img.shields.io/github/v/release/jcsm/ShufflePaper" alt="Latest Release" />
  </a>
  <a href="https://github.com/jcsm/ShufflePaper/releases/latest">Download here</a>
</p>

## Features

- Select a folder of wallpapers
- Configurable rotation intervals (1 minute to 24 hours)
- Shuffle or Sequential rotation mode
- Shuffle mode never repeats an image until all have been shown once (Windows' built-in slideshow can't do this)
- Next / Previous manual controls with undo history (also in the tray menu)
- Start with Windows (togglable)
- Smart fullscreen pause for games, videos, and presentations
- Work / Personal context profiles with schedules and tray overrides
- Fixed-height settings window with internal scrolling
- Lives in the system tray with quick menu
- Privacy-friendly: no network, no telemetry

## Prerequisites

- [Node.js](https://nodejs.org/) (>= 16)
- [Rust](https://www.rust-lang.org/) (>= 1.70)
- Cargo
- Visual Studio Build Tools / MSVC (for Rust `windows` crate on Windows)

## Dependencies

- **Frontend**: Vue 3, TypeScript, Vite, Tailwind CSS
- **Backend**: Tauri v2, Rust
- **Rust crates**: `windows`, `serde`, `serde_json`, `rand`, `chrono`
- **Tauri plugins**: `tauri-plugin-dialog`, `tauri-plugin-autostart`

## Installation

```powershell
git clone https://github.com/jcsm/ShufflePaper.git
cd ShufflePaper
npm install
```

## Run in Development

```powershell
npm run tauri dev
```

This starts the Vite dev server and launches the Tauri app in debug mode.

## Build for Production

```powershell
npm run tauri build
```

The installer / executable will be generated in `src-tauri/target/release/`.

## Usage

1. Click **Browse** to select a folder containing wallpapers (jpg, jpeg, png, bmp, webp).
2. Choose a rotation interval from the dropdown.
3. Select **Shuffle** or **Sequential** mode.
4. Configure optional fullscreen pause and Work / Personal context rules.
5. Enable **Start with Windows** if you want the app to launch on boot.
6. Click **Save Configuration**.
7. Use the system tray icon for quick access: Next Wallpaper, Pause / Resume, profile override, Settings, Exit.

## Project Structure

```
ShufflePaper/
├── src-tauri/
│   └── src/
│       ├── commands.rs   # Tauri IPC commands
│       ├── main.rs
│       ├── lib.rs
│       ├── scheduler.rs  # Background rotation loop
│       ├── scanner.rs    # Folder image scanner
│       ├── settings.rs   # Config load / save
│       ├── tray.rs       # System tray menu
│       ├── state.rs      # App state (Mutex)
│       └── wallpaper.rs  # Windows wallpaper API
├── src/
│   └── App.vue           # Main Vue UI
├── package.json
├── vite.config.ts
├── tauri.conf.json
└── README.md
```

## License

MIT
