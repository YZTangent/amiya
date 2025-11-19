# Installation Guide

This guide covers installing Amiya on your system.

## Prerequisites

### Required Dependencies

- **Rust** (1.70+): Install from [rustup.rs](https://rustup.rs/)
- **GTK4** (4.12+): Graphical toolkit
- **D-Bus**: System message bus (usually pre-installed)

### System Libraries

The following development libraries are required:

#### Arch Linux
```bash
sudo pacman -S gtk4 gtk4-layer-shell dbus
```

#### Ubuntu/Debian
```bash
sudo apt install libgtk-4-dev libdbus-1-dev pkg-config build-essential
```

#### Fedora
```bash
sudo dnf install gtk4-devel dbus-devel gcc
```

### Optional Dependencies

- **niri**: Wayland compositor (recommended)
- **swhkd**: Hotkey daemon (alternative to niri bindings)
- **UPower**: For battery widget
- **BlueZ**: For Bluetooth control
- **NetworkManager**: For WiFi control
- **PulseAudio/PipeWire**: For audio control

## Quick Install (Recommended)

1. Clone the repository:
```bash
git clone https://github.com/yourusername/amiya.git
cd amiya
```

2. Run the installation script:
```bash
./install.sh
```

3. Add `~/.local/bin` to your PATH (if not already):
```bash
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc
```

4. Enable auto-start:
```bash
systemctl --user enable --now amiya.service
```

## Manual Installation

### 1. Build from Source

```bash
cargo build --release
```

### 2. Install Binaries

```bash
mkdir -p ~/.local/bin
cp target/release/amiya ~/.local/bin/
cp target/release/amiya-ctl ~/.local/bin/
chmod +x ~/.local/bin/amiya ~/.local/bin/amiya-ctl
```

### 3. Install Configuration

```bash
mkdir -p ~/.config/amiya
cp amiya.toml ~/.config/amiya/config.toml
```

### 4. Install Systemd Service (Optional)

```bash
mkdir -p ~/.config/systemd/user
cp amiya.service ~/.config/systemd/user/
systemctl --user daemon-reload
systemctl --user enable --now amiya.service
```

## Configuration

### Amiya Configuration

Edit `~/.config/amiya/config.toml` to customize:
- Bar position and height
- Theme colors and fonts
- Widget visibility

See `amiya.toml` for available options.

### Hotkey Configuration

#### For niri Users

Add hotkey bindings to `~/.config/niri/config.kdl`:

```kdl
binds {
    // Popups
    Mod+B { spawn "amiya-ctl" "popup" "toggle" "bluetooth"; }
    Mod+W { spawn "amiya-ctl" "popup" "toggle" "wifi"; }
    Mod+M { spawn "amiya-ctl" "popup" "toggle" "media-control"; }
    Mod+Escape { spawn "amiya-ctl" "popup" "toggle" "power"; }

    // Volume
    XF86AudioRaiseVolume { spawn "amiya-ctl" "volume" "up"; }
    XF86AudioLowerVolume { spawn "amiya-ctl" "volume" "down"; }
    XF86AudioMute { spawn "amiya-ctl" "volume" "toggle-mute"; }

    // Brightness
    XF86MonBrightnessUp { spawn "amiya-ctl" "brightness" "up"; }
    XF86MonBrightnessDown { spawn "amiya-ctl" "brightness" "down"; }
}
```

See `docs/niri-config-example.kdl` for the complete example.

#### For swhkd Users

Add bindings to `~/.config/swhkd/swhkdrc`:

```
super + b
    amiya-ctl popup toggle bluetooth

super + w
    amiya-ctl popup toggle wifi

super + escape
    amiya-ctl popup toggle power

XF86AudioRaiseVolume
    amiya-ctl volume up
```

See `docs/swhkd-config-example` for the complete example.

## Verification

### Test the Installation

1. **Check if Amiya is running:**
```bash
amiya-ctl status
```

2. **Test popup controls:**
```bash
amiya-ctl popup toggle bluetooth
amiya-ctl popup toggle wifi
amiya-ctl popup toggle power
```

3. **Test system controls:**
```bash
amiya-ctl volume up
amiya-ctl brightness up
```

### View Logs

```bash
# If using systemd
journalctl --user -u amiya -f

# If running manually
# Logs will appear in the terminal
```

## Troubleshooting

### Amiya won't start

1. **Check dependencies:**
```bash
pkg-config --exists gtk4 && echo "GTK4 OK" || echo "GTK4 missing"
```

2. **Check logs:**
```bash
journalctl --user -u amiya --no-pager | tail -50
```

3. **Try running manually:**
```bash
RUST_LOG=debug amiya
```

### Bar doesn't appear

- Ensure you're using a Wayland compositor
- Check that layer-shell protocol is supported
- Verify Amiya is running: `amiya-ctl status`

### Popups don't work

1. **Test IPC connection:**
```bash
amiya-ctl ping
```

2. **Check socket exists:**
```bash
ls -la $XDG_RUNTIME_DIR/amiya/amiya.sock
```

3. **Verify event emission:**
```bash
RUST_LOG=debug amiya-ctl popup toggle bluetooth
```

### System controls don't work

**Audio:**
- Check PulseAudio/PipeWire is running: `pactl info`

**Bluetooth:**
- Check BlueZ is running: `systemctl status bluetooth`

**WiFi:**
- Check NetworkManager is running: `systemctl status NetworkManager`

**Battery:**
- Check UPower is available: `upower --dump`

**Power:**
- Check systemd-logind: `systemctl status systemd-logind`

For more troubleshooting, see `docs/TROUBLESHOOTING.md`.

## Updating

```bash
cd amiya
git pull
./install.sh
systemctl --user restart amiya.service
```

## Uninstallation

```bash
./uninstall.sh
```

This will:
- Stop and disable the systemd service
- Remove binaries from `~/.local/bin`
- Optionally remove configuration files

## Next Steps

- Read `docs/HOTKEYS.md` for all available commands
- Check `docs/TROUBLESHOOTING.md` for common issues
- Customize your theme in `~/.config/amiya/config.toml`
- Join the community (Discord/Matrix link)

## Getting Help

- **Documentation:** Check the `docs/` directory
- **Issues:** https://github.com/yourusername/amiya/issues
- **Discussions:** https://github.com/yourusername/amiya/discussions

Enjoy using Amiya!
