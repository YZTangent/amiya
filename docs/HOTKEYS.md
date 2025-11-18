# Hotkey Setup Guide

Amiya supports hotkey triggering through the `amiya-ctl` command-line tool. You can use any hotkey daemon that can execute shell commands.

## Table of Contents

- [Supported Hotkey Daemons](#supported-hotkey-daemons)
- [Setup with niri](#setup-with-niri)
- [Setup with swhkd](#setup-with-swhkd)
- [Setup with Other Daemons](#setup-with-other-daemons)
- [Available Commands](#available-commands)
- [Testing](#testing)

## Supported Hotkey Daemons

Amiya works with any hotkey daemon that can execute shell commands:

- **niri** (built-in bindings) - Recommended for niri users
- **swhkd** - Simple Wayland HotKey Daemon
- **keyd** - Key remapping daemon
- **wshowkeys** - For testing
- **Any custom solution** that can run `amiya-ctl`

## Setup with niri

If you're using niri as your compositor, you can add hotkeys directly to your niri configuration.

### 1. Open niri config

```bash
$EDITOR ~/.config/niri/config.kdl
```

### 2. Add Amiya bindings

See [niri-config-example.kdl](./niri-config-example.kdl) for a complete example.

Basic setup:

```kdl
binds {
    // Popups
    Mod+B { spawn "amiya-ctl" "popup" "toggle" "bluetooth"; }
    Mod+W { spawn "amiya-ctl" "popup" "toggle" "wifi"; }
    Mod+M { spawn "amiya-ctl" "popup" "toggle" "media-control"; }

    // Volume
    XF86AudioRaiseVolume { spawn "amiya-ctl" "volume" "up"; }
    XF86AudioLowerVolume { spawn "amiya-ctl" "volume" "down"; }
    XF86AudioMute { spawn "amiya-ctl" "volume" "toggle-mute"; }

    // Brightness
    XF86MonBrightnessUp { spawn "amiya-ctl" "brightness" "up"; }
    XF86MonBrightnessDown { spawn "amiya-ctl" "brightness" "down"; }
}
```

### 3. Reload niri config

```bash
niri msg action reload-config
```

## Setup with swhkd

swhkd is a simple hotkey daemon for Wayland.

### 1. Install swhkd

```bash
# Arch Linux
yay -S swhkd-bin

# From source
git clone https://github.com/waycrate/swhkd
cd swhkd
cargo install --path swhkd
```

### 2. Create config

```bash
mkdir -p ~/.config/swhkd
$EDITOR ~/.config/swhkd/swhkdrc
```

See [swhkd-config-example](./swhkd-config-example) for a complete example.

### 3. Start swhkd

```bash
# Start once
sudo swhkd

# Or use systemd
systemctl --user start swhkd
systemctl --user enable swhkd
```

### 4. Reload config

```bash
pkill -USR1 swhkd
```

## Setup with Other Daemons

### keyd

```ini
[main]
# Volume
volumeup = spawn amiya-ctl volume up
volumedown = spawn amiya-ctl volume down
mute = spawn amiya-ctl volume toggle-mute

# Brightness
brightnessup = spawn amiya-ctl brightness up
brightnessdown = spawn amiya-ctl brightness down
```

### Custom shell script

You can create your own hotkey handler:

```bash
#!/bin/bash
# ~/bin/my-hotkeys.sh

case "$1" in
    volume-up)
        amiya-ctl volume up
        ;;
    volume-down)
        amiya-ctl volume down
        ;;
    bluetooth)
        amiya-ctl popup toggle bluetooth
        ;;
esac
```

Then bind it with your compositor or window manager.

## Available Commands

### Popup Control

```bash
# Show popup
amiya-ctl popup show <bluetooth|wifi|media-control|power>

# Hide popup
amiya-ctl popup hide <bluetooth|wifi|media-control|power>

# Toggle popup
amiya-ctl popup toggle <bluetooth|wifi|media-control|power>
```

### Volume Control

```bash
# Increase volume by 5% (default)
amiya-ctl volume up

# Increase by custom amount
amiya-ctl volume up --amount 10

# Decrease volume
amiya-ctl volume down [--amount <percent>]

# Set specific level (0-100)
amiya-ctl volume set 75

# Mute/unmute
amiya-ctl volume mute
amiya-ctl volume unmute
amiya-ctl volume toggle-mute
```

### Brightness Control

```bash
# Increase brightness by 5% (default)
amiya-ctl brightness up

# Increase by custom amount
amiya-ctl brightness up --amount 10

# Decrease brightness
amiya-ctl brightness down [--amount <percent>]

# Set specific level (0-100)
amiya-ctl brightness set 50
```

### Power Management

```bash
# Shutdown the system
amiya-ctl power shutdown

# Reboot the system
amiya-ctl power reboot

# Suspend the system (sleep)
amiya-ctl power suspend

# Hibernate the system
amiya-ctl power hibernate

# Lock the screen
amiya-ctl power lock

# Show power menu popup
amiya-ctl popup toggle power
```

### Utility Commands

```bash
# Check server status
amiya-ctl status

# Ping server (health check)
amiya-ctl ping

# Get help
amiya-ctl --help
amiya-ctl popup --help
amiya-ctl volume --help
```

## Testing

### 1. Check if Amiya is running

```bash
amiya-ctl ping
```

Expected output:
```
✓ Pong! Server is alive.
```

### 2. Test popup commands

```bash
amiya-ctl popup show bluetooth
amiya-ctl popup hide bluetooth
amiya-ctl popup toggle wifi
```

### 3. Test volume/brightness

```bash
amiya-ctl volume up
amiya-ctl volume down
amiya-ctl brightness up
amiya-ctl brightness down
```

### 4. Check status

```bash
amiya-ctl status
```

Expected output:
```
Amiya Desktop Environment
Version: 0.1.0
Uptime: 123 seconds
```

### 5. Monitor logs

```bash
# If using systemd
journalctl --user -u amiya -f

# Or check amiya output
```

## Troubleshooting

### "Amiya socket not found"

- Make sure Amiya is running
- Check socket exists: `ls -la $XDG_RUNTIME_DIR/amiya/amiya.sock`
- Try: `amiya-ctl ping`

### "Failed to connect to Amiya"

- Amiya might not be running
- Check permissions on socket file
- Try restarting Amiya

### Hotkeys not working

1. Test command manually: `amiya-ctl popup toggle bluetooth`
2. Check hotkey daemon is running: `ps aux | grep swhkd`
3. Check hotkey daemon logs
4. Verify hotkey binding syntax
5. Try reloading hotkey daemon config

### Commands work but no visual response

- Check Amiya logs for errors
- Verify backends are available:
  - Bluetooth: `bluetoothctl show`
  - WiFi: `nmcli device wifi`
  - Media: `playerctl status`
- Try showing popup manually: `amiya-ctl popup show bluetooth`

## Media Keys

Most keyboards have dedicated media keys. Common key names:

- **Volume**: `XF86AudioRaiseVolume`, `XF86AudioLowerVolume`, `XF86AudioMute`
- **Brightness**: `XF86MonBrightnessUp`, `XF86MonBrightnessDown`
- **Media**: `XF86AudioPlay`, `XF86AudioNext`, `XF86AudioPrev`

You can find key names using:

```bash
# With wev (Wayland event viewer)
wev

# With xev (X11)
xev
```

## Best Practices

1. **Use toggle for popups** - Easier than separate show/hide bindings
2. **Default amounts work well** - 5% steps for volume/brightness
3. **Bind media keys** - Use hardware keys for volume/brightness
4. **Test before committing** - Verify commands work manually first
5. **Keep bindings simple** - Avoid complex modifier combinations
6. **Document your setup** - Comment your hotkey config file

## Example Workflows

### Minimal Setup (Media Keys Only)

```kdl
binds {
    XF86AudioRaiseVolume { spawn "amiya-ctl" "volume" "up"; }
    XF86AudioLowerVolume { spawn "amiya-ctl" "volume" "down"; }
    XF86AudioMute { spawn "amiya-ctl" "volume" "toggle-mute"; }
    XF86MonBrightnessUp { spawn "amiya-ctl" "brightness" "up"; }
    XF86MonBrightnessDown { spawn "amiya-ctl" "brightness" "down"; }
}
```

### Full Setup (All Features)

See [niri-config-example.kdl](./niri-config-example.kdl) or [swhkd-config-example](./swhkd-config-example)

### Power User Setup (Custom Amounts)

```kdl
binds {
    // Fine-grained control
    Mod+Plus { spawn "amiya-ctl" "volume" "up" "--amount" "2"; }
    Mod+Minus { spawn "amiya-ctl" "volume" "down" "--amount" "2"; }

    // Coarse control
    Mod+Shift+Plus { spawn "amiya-ctl" "volume" "up" "--amount" "10"; }
    Mod+Shift+Minus { spawn "amiya-ctl" "volume" "down" "--amount" "10"; }

    // Presets
    Mod+1 { spawn "amiya-ctl" "volume" "set" "30"; }
    Mod+5 { spawn "amiya-ctl" "volume" "set" "50"; }
    Mod+9 { spawn "amiya-ctl" "volume" "set" "90"; }
}
```

## See Also

- [ARCHITECTURE.md](./ARCHITECTURE.md) - System architecture
- [TROUBLESHOOTING.md](./TROUBLESHOOTING.md) - Common issues
- [README.md](../README.md) - Main documentation
