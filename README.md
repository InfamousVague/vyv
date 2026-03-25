# Vyv

A cross-platform keep-awake utility for macOS, Windows, and Linux. Like Caffeine or Amphetamine, but open source and cross-platform.

Named after Vyvanse — keeping your computer wide awake.

## Features

- **Keep your computer awake** — Prevent system and display sleep
- **Timed sessions** — 5m, 10m, 15m, 30m, 45m, 1h, 2h, 4h, 8h, or indefinite
- **Two wake modes** — Keep display + system awake, or system-only (display can sleep)
- **Prevent sleep when lid closed** — Override macOS clamshell sleep (same technique as Amphetamine)
- **Mouse jiggle** — Simulate mouse movement to prevent idle detection in Slack, Teams, Zoom, etc.
- **Panic button** — `Ctrl+Shift+Escape` instantly kills simulation and deactivates
- **Lifetime stats** — Tracks total uptime with "dosage" milestones
- **System tray only** — No dock icon, no windows, just a menu bar pill icon

## Screenshots

Click the pill icon in your menu bar to access the dropdown:

```
Vyv: Display + System — 1h 47m
──────────────────────────────
End Session
──────────────────────────────
Simulate (Off)          ▸
Settings                ▸
  ✓ Keep display awake
  ☐ Prevent sleep when lid closed
  ☐ Jiggle mouse to prevent idle
──────────────────────────────
Total: 23h (Regular dose)
──────────────────────────────
Quit Vyv
```

## Installation

### macOS

Download the `.dmg` from [Releases](../../releases), open it, and drag Vyv to Applications.

### Windows

Download the `.msi` installer from [Releases](../../releases) and run it.

### Linux

Download the `.deb` or `.AppImage` from [Releases](../../releases).

```bash
# Debian/Ubuntu
sudo dpkg -i vyv_0.1.0_amd64.deb

# AppImage
chmod +x vyv_0.1.0_amd64.AppImage
./vyv_0.1.0_amd64.AppImage
```

## How It Works

### Sleep Prevention

Vyv uses platform-native APIs to prevent sleep:

- **macOS**: IOPMAssertion (PreventUserIdleSystemSleep, PreventUserIdleDisplaySleep, PreventSystemSleep)
- **Windows**: SetThreadExecutionState (ES_CONTINUOUS | ES_SYSTEM_REQUIRED | ES_DISPLAY_REQUIRED)
- **Linux**: systemd-inhibit / D-Bus

### Lid Closed Mode (macOS)

To prevent sleep when the lid is closed, Vyv uses `pmset disablesleep` (the same approach as Amphetamine). On first use, it will prompt for your admin password to install a sudoers entry at `/etc/sudoers.d/vyv_power`. This is a one-time setup.

### Mouse Jiggle

When enabled, Vyv simulates small random mouse movements every 60-120 seconds to prevent apps like Slack, Teams, and Zoom from showing you as "away". The movements are tiny (a few pixels) and immediately reversed.

macOS requires Accessibility permission for this feature (System Settings > Privacy & Security > Accessibility).

## Building from Source

### Prerequisites

- [Rust](https://rustup.rs/) (1.77+)
- [Node.js](https://nodejs.org/) (18+)
- Platform-specific dependencies:
  - **macOS**: Xcode Command Line Tools
  - **Windows**: Visual Studio Build Tools
  - **Linux**: `sudo apt install libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf libxdo-dev`

### Build

```bash
git clone https://github.com/InfamousVague/vyv.git
cd vyv
npm install
npx tauri build
```

The built application will be in `src-tauri/target/release/bundle/`.

### Development

```bash
npm install
npx tauri dev
```

## Tech Stack

- **Backend**: Rust + [Tauri v2](https://v2.tauri.app/)
- **Sleep prevention**: [keepawake](https://crates.io/crates/keepawake) crate
- **Input simulation**: [enigo](https://crates.io/crates/enigo) crate
- **Process monitoring**: [sysinfo](https://crates.io/crates/sysinfo) crate
- **Battery monitoring**: [battery](https://crates.io/crates/battery) crate

## License

MIT
