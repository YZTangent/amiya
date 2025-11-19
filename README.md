# Amiya

A modern, integrated desktop environment for Wayland using [niri](https://github.com/YaLTeR/niri) as the tiling window manager.

## Features

### Status Bar
- **Workspace Display**: Shows all existing workspaces with visual highlighting of the active workspace
- **Clock**: Real-time clock display in the center of the bar
- **System Monitoring**:
  - CPU usage percentage
  - Memory usage percentage
  - Device temperature
  - WiFi status
  - Bluetooth status

### Interactive Popups
- **Bluetooth Management**: Full Bluetooth device management with pairing, connection, and scanning
- **WiFi Management**: Network selection, connection, and status monitoring
- **Media Control**: MPRIS media player control with playback, volume, and track information

### On-Screen Displays (OSD)
- **Volume Slider**: Beautiful overlay when volume is changed
- **Brightness Slider**: Visual feedback for brightness adjustments

### Battery Widget
- **Battery Status**: Shows battery percentage and charging state
- **Visual Indicators**: Color-coded battery levels (green when charging, orange/red when low)
- **Real-time Updates**: Monitors battery via UPower D-Bus

### Power Management
- **Power Menu**: Centered popup with lock, suspend, hibernate, reboot, shutdown
- **System Integration**: D-Bus systemd/logind integration for power actions
- **Hotkey Support**: Quick access via amiya-ctl

### Hotkey Control
Full control via `amiya-ctl` CLI tool:
- **Popup Control**: Show/hide/toggle Bluetooth, WiFi, and Media popups
- **Volume Control**: Adjust volume, mute/unmute via hotkeys
- **Brightness Control**: Adjust screen brightness via hotkeys
- **IPC Interface**: Unix socket-based command interface
- **External Integration**: Works with niri, swhkd, or any hotkey daemon

## Screenshots

*(Add screenshots here)*

## Installation

### Quick Install

```bash
# Clone the repository
git clone https://github.com/yourusername/amiya.git
cd amiya

# Run the installation script
./install.sh

# Enable auto-start
systemctl --user enable --now amiya.service
```

**For detailed installation instructions, troubleshooting, and manual installation, see [INSTALL.md](INSTALL.md).**

### Prerequisites

- **Rust** 1.70+ (install from [rustup.rs](https://rustup.rs/))
- **GTK4** 4.12+
- **Wayland compositor** (niri recommended)
- **D-Bus** system services

See [INSTALL.md](INSTALL.md) for distribution-specific package lists.

## Configuration

Amiya creates a default configuration file at `~/.config/amiya/config.toml` on first run.

### Example Configuration

```toml
[bar]
height = 32
position = "top"  # or "bottom"
show_workspaces = true
show_clock = true
show_system_info = true

[theme]
background = "#1e1e2e"
foreground = "#cdd6f4"
accent = "#89b4fa"
font = "Sans"
font_size = 11

[hotkeys]
"Super+B" = "show-bluetooth"
"Super+W" = "show-wifi"
"Super+M" = "show-media-control"
```

### Theme Customization

Amiya uses a simple color scheme that you can customize:

- `background`: Main bar background color
- `foreground`: Text and icon color
- `accent`: Highlight color for active elements
- `font`: Font family name
- `font_size`: Font size in points

Popular color schemes:
- **Catppuccin Mocha** (default): `#1e1e2e`, `#cdd6f4`, `#89b4fa`
- **Dracula**: `#282a36`, `#f8f8f2`, `#bd93f9`
- **Nord**: `#2e3440`, `#eceff4`, `#88c0d0`
- **Gruvbox Dark**: `#282828`, `#ebdbb2`, `#83a598`

## Usage

### Starting Amiya

Add Amiya to your niri autostart configuration:

```bash
# Edit your niri config
nano ~/.config/niri/config.kdl
```

Add this to the `spawn-at-startup` section:

```kdl
spawn-at-startup {
    command "amiya"
}
```

Or start it manually:

```bash
amiya
```

### Hotkey Integration

Amiya includes `amiya-ctl`, a command-line tool for controlling all desktop functions.

#### Using amiya-ctl

```bash
# Popup control
amiya-ctl popup toggle bluetooth
amiya-ctl popup show wifi
amiya-ctl popup hide media-control

# Volume control
amiya-ctl volume up
amiya-ctl volume down --amount 10
amiya-ctl volume toggle-mute

# Brightness control
amiya-ctl brightness up
amiya-ctl brightness down --amount 5
amiya-ctl brightness set 75

# Utility
amiya-ctl status
amiya-ctl ping
```

#### Option 1: Use niri's built-in hotkeys

Add to your niri config (`~/.config/niri/config.kdl`):

```kdl
binds {
    // Popups
    Mod+B { spawn "amiya-ctl" "popup" "toggle" "bluetooth"; }
    Mod+W { spawn "amiya-ctl" "popup" "toggle" "wifi"; }
    Mod+M { spawn "amiya-ctl" "popup" "toggle" "media-control"; }

    // Volume (media keys)
    XF86AudioRaiseVolume { spawn "amiya-ctl" "volume" "up"; }
    XF86AudioLowerVolume { spawn "amiya-ctl" "volume" "down"; }
    XF86AudioMute { spawn "amiya-ctl" "volume" "toggle-mute"; }

    // Brightness (media keys)
    XF86MonBrightnessUp { spawn "amiya-ctl" "brightness" "up"; }
    XF86MonBrightnessDown { spawn "amiya-ctl" "brightness" "down"; }
}
```

See [docs/niri-config-example.kdl](./docs/niri-config-example.kdl) for a complete example.

#### Option 2: Use an external hotkey daemon

Install and configure [swhkd](https://github.com/waycrate/swhkd):

```bash
# Install swhkd
yay -S swhkd-bin  # Arch
cargo install swhkd  # From source

# Configure (~/.config/swhkd/swhkdrc)
super + b
    amiya-ctl popup toggle bluetooth
super + w
    amiya-ctl popup toggle wifi
# ... see docs/swhkd-config-example for more
```

See [docs/HOTKEYS.md](./docs/HOTKEYS.md) for complete hotkey setup guide.

### Brightness Control

For brightness control to work without root, you need to set up udev rules:

```bash
# Create udev rule
sudo tee /etc/udev/rules.d/90-backlight.rules << EOF
SUBSYSTEM=="backlight", ACTION=="add", \
  RUN+="/bin/chgrp video /sys/class/backlight/%k/brightness", \
  RUN+="/bin/chmod g+w /sys/class/backlight/%k/brightness"
EOF

# Add your user to the video group
sudo usermod -a -G video $USER

# Reload udev rules
sudo udevadm control --reload-rules && sudo udevadm trigger
```

## Architecture

Amiya is built with:
- **GTK4**: Modern UI toolkit
- **gtk4-layer-shell**: Wayland layer shell protocol for bars and overlays
- **Rust**: Memory-safe systems programming
- **Niri IPC**: Direct communication with niri compositor

### Project Structure

```
amiya/
├── src/
│   ├── bar.rs              # Main status bar
│   ├── config.rs           # Configuration management
│   ├── main.rs             # Application entry point
│   ├── widgets/            # Bar widgets
│   │   ├── clock.rs        # Clock widget
│   │   ├── system_info.rs  # System monitoring
│   │   └── workspaces.rs   # Workspace switcher
│   ├── popups/             # Interactive popups
│   │   ├── bluetooth.rs    # Bluetooth management
│   │   ├── wifi.rs         # WiFi management
│   │   └── media_control.rs# Media player control
│   ├── overlays/           # On-screen displays
│   │   └── slider.rs       # Volume/brightness sliders
│   ├── system/             # System control backends
│   │   ├── audio.rs        # Audio/volume control
│   │   ├── backlight.rs    # Brightness control
│   │   ├── bluetooth.rs    # Bluetooth backend
│   │   └── network.rs      # Network management
│   └── ipc/                # Compositor communication
│       └── niri.rs         # Niri IPC client
└── Cargo.toml
```

## Roadmap

- [x] Basic status bar
- [x] Workspace display
- [x] System monitoring (CPU, RAM, temp)
- [x] Clock widget
- [x] Volume/brightness overlays
- [x] Bluetooth/WiFi/Media popups
- [ ] Full niri IPC integration
- [ ] D-Bus integration for Bluetooth/WiFi/Audio
- [ ] MPRIS media player integration
- [ ] Notification support
- [ ] System tray
- [ ] Battery indicator
- [ ] Network speed indicator
- [ ] Custom widget plugins
- [ ] Wayland screenshot integration
- [ ] Power menu
- [ ] Application launcher

## Contributing

Contributions are welcome! Please feel free to submit pull requests or open issues for bugs and feature requests.

### Development

```bash
# Run in debug mode with logging
RUST_LOG=debug cargo run

# Format code
cargo fmt

# Lint code
cargo clippy

# Run tests
cargo test
```

## License

MIT License - see LICENSE file for details

## Acknowledgments

- [niri](https://github.com/YaLTeR/niri) - The excellent scrollable-tiling Wayland compositor
- [gtk4-layer-shell](https://github.com/wmww/gtk4-layer-shell) - GTK4 bindings for layer shell
- [waybar](https://github.com/Alexays/Waybar) - Inspiration for feature set
- [eww](https://github.com/elkowar/eww) - Widget system inspiration

## Similar Projects

- [waybar](https://github.com/Alexays/Waybar) - Highly customizable Wayland bar
- [eww](https://github.com/elkowar/eww) - Widget system for Wayland
- [ags](https://github.com/Aylur/ags) - Aylur's GTK Shell

---

**Note**: This is early-stage software. While the core functionality is implemented, full system integration (D-Bus, MPRIS, etc.) requires additional implementation. Currently, many features display mock data and serve as a framework for full integration.
