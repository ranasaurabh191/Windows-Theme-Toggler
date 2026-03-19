# dark-toggle

A lightweight Windows 11 system tray app written in Rust that toggles between dark and light mode with a single click — including the taskbar and Start menu.

![Rust](https://img.shields.io/badge/Rust-2021-orange) ![Platform](https://img.shields.io/badge/Platform-Windows%2011-blue) ![License](https://img.shields.io/badge/License-MIT-green)

---

## Features

- **One-click toggle** — left-click the tray icon to instantly switch between dark and light mode
- **Smooth animation** — icon crossfades between dark and light states
- **Taskbar aware** — changes the taskbar and Start menu color too, not just app windows
- **Correct startup icon** — detects your current Windows theme on launch and shows the right icon
- **Right-click to quit** — clean exit from the context menu
- **Zero dependencies at runtime** — single standalone `.exe`, no installer, no assets folder needed
- **Minimal CPU usage** — ~0% CPU when idle

---

## Download

Grab the latest release from the [Releases](../../releases) page and run `dark-toggle.exe` directly. No installation needed.

---

## Usage

| Action | Result |
|---|---|
| Left-click tray icon | Toggle dark / light mode |
| Right-click tray icon | Show context menu |
| Right-click → Quit | Exit the app |

The icon in the tray always shows what mode you will **switch to** on click.

---

## Auto-start with Windows

To make dark-toggle run automatically on login, open PowerShell and run:

```powershell
$exe = "C:\path\to\dark-toggle.exe"
Set-ItemProperty -Path "HKCU:\Software\Microsoft\Windows\CurrentVersion\Run" -Name "DarkToggle" -Value $exe
```

Replace `C:\path\to\dark-toggle.exe` with the actual path to your exe.

To remove it from startup:

```powershell
Remove-ItemProperty -Path "HKCU:\Software\Microsoft\Windows\CurrentVersion\Run" -Name "DarkToggle"
```

---

## Building from Source

### Prerequisites

- [Rust](https://rustup.rs/) (stable, 2021 edition)
- Windows 11 with AMD drivers installed

### Steps

```powershell
# Clone the repo
git clone https://github.com/yourname/dark-toggle
cd dark-toggle

# Debug build (shows console output, useful for development)
cargo build

# Release build (no console window, fully optimized)
cargo build --release
```

The release exe is at `target\release\dark-toggle.exe`. It is fully self-contained — the icons are embedded at compile time via `include_bytes!`.

---

## Project Structure

```
dark-toggle/
├── Cargo.toml          # Dependencies
├── assets/
│   ├── dark.png        # Dark mode icon (embedded into exe at compile time)
│   └── light.png       # Light mode icon (embedded into exe at compile time)
└── src/
    ├── main.rs         # Entry point, tray icon, event loop, animation
    └── theme.rs        # Windows registry read/write + WM_SETTINGCHANGE broadcast
```

---

## How It Works

### Theme switching

Windows stores the color mode in two registry keys under:

```
HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Themes\Personalize
```

| Key | Value | Meaning |
|---|---|---|
| `AppsUseLightTheme` | `0` | Dark mode |
| `AppsUseLightTheme` | `1` | Light mode |
| `SystemUsesLightTheme` | `0` | Dark taskbar |
| `SystemUsesLightTheme` | `1` | Light taskbar |

After writing the registry, the app broadcasts a `WM_SETTINGCHANGE` message with `ImmersiveColorSet` — this is exactly what Windows Settings does internally to make Explorer and the taskbar reload the theme immediately without a restart.

### Icon animation

On each toggle, the app generates intermediate frames by blending the two icon images pixel by pixel using **smoothstep easing** (`3t² - 2t³`), producing a natural-feeling crossfade over ~128ms.

### Event loop

Uses a proper Win32 `PeekMessageW` message loop instead of a simple sleep loop, which is required for tray icon events to fire correctly on Windows.

---

## Dependencies

| Crate | Version | Purpose |
|---|---|---|
| `tray-icon` | 0.20.1 | System tray icon and context menu |
| `winreg` | 0.56 | Windows registry read/write |
| `image` | 0.25 | PNG decoding and pixel blending |
| `windows-sys` | 0.59 | Win32 API (message loop, WM_SETTINGCHANGE) |

---

## License

MIT — do whatever you want with it.
