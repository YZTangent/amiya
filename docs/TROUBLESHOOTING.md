# Troubleshooting Guide

Common issues and solutions for Amiya Desktop Environment.

## Table of Contents

- [IPC and amiya-ctl Issues](#ipc-and-amiya-ctl-issues)
- [Popup Issues](#popup-issues)
- [Volume Control Issues](#volume-control-issues)
- [Brightness Control Issues](#brightness-control-issues)
- [Bluetooth Issues](#bluetooth-issues)
- [WiFi Issues](#wifi-issues)
- [Media Control Issues](#media-control-issues)
- [Bar Not Showing](#bar-not-showing)
- [Performance Issues](#performance-issues)
- [Build Issues](#build-issues)

---

## IPC and amiya-ctl Issues

### "Amiya socket not found"

**Symptom**: `amiya-ctl` command fails with socket error

**Causes**:
- Amiya is not running
- Socket file was deleted
- `$XDG_RUNTIME_DIR` is not set correctly

**Solutions**:
```bash
# Check if Amiya is running
ps aux | grep amiya

# Check if socket exists
ls -la $XDG_RUNTIME_DIR/amiya/amiya.sock

# Check XDG_RUNTIME_DIR
echo $XDG_RUNTIME_DIR
# Should be something like /run/user/1000

# Restart Amiya
pkill amiya
amiya
```

### "Failed to connect to Amiya"

**Symptom**: Socket exists but connection fails

**Solutions**:
```bash
# Check socket permissions
ls -la $XDG_RUNTIME_DIR/amiya/

# Socket should be owned by your user
# If not, remove and restart Amiya
rm -rf $XDG_RUNTIME_DIR/amiya/
amiya
```

### Commands timeout or hang

**Symptom**: `amiya-ctl` commands don't respond

**Solutions**:
```bash
# Check Amiya logs for errors
journalctl --user -u amiya -n 50

# Try pinging
timeout 5 amiya-ctl ping

# If timeout occurs, restart Amiya
pkill -9 amiya
amiya
```

---

## Popup Issues

### Popup doesn't show

**Symptom**: `amiya-ctl popup show bluetooth` succeeds but nothing appears

**Solutions**:
```bash
# Check if compositor supports layer-shell
# Amiya requires wlr-layer-shell protocol

# Check Amiya logs
journalctl --user -u amiya | grep -i popup

# Try other popups
amiya-ctl popup toggle wifi
amiya-ctl popup toggle media-control

# Verify GTK4 is working
gtk4-demo
```

### Popup shows but is empty

**Symptom**: Popup appears but shows no content

**For Bluetooth popup**:
```bash
# Check BlueZ is running
systemctl status bluetooth

# Check D-Bus connection
bluetoothctl show
```

**For WiFi popup**:
```bash
# Check NetworkManager is running
systemctl status NetworkManager

# Check nmcli works
nmcli device wifi list
```

**For Media popup**:
```bash
# Check for media players
playerctl -l

# Play something and try again
```

### Popup appears in wrong position

**Symptom**: Popup not in top-right corner

**Solutions**:
- This might be a compositor issue
- Check niri/compositor configuration
- Try restarting Amiya

---

## Volume Control Issues

### Volume changes but no overlay shows

**Symptom**: `amiya-ctl volume up` works but no visual feedback

**Solutions**:
```bash
# Check overlays are working
# Volume change should show overlay automatically

# Check Amiya logs
journalctl --user -u amiya | grep -i volume

# Try manual volume set
amiya-ctl volume set 50
```

### "Audio control not available"

**Symptom**: Volume commands fail with this error

**Causes**:
- PulseAudio/PipeWire not running
- D-Bus connection failed

**Solutions**:
```bash
# Check audio server
pactl info  # PulseAudio
pw-cli info  # PipeWire

# Check D-Bus
dbus-send --session --dest=org.freedesktop.DBus \
  --type=method_call --print-reply /org/freedesktop/DBus \
  org.freedesktop.DBus.ListNames

# Restart audio server
systemctl --user restart pulseaudio
# or
systemctl --user restart pipewire pipewire-pulse
```

### Volume percentage incorrect

**Symptom**: Volume shows wrong value

**Solutions**:
- Check your audio mixer settings
- PulseAudio uses 0-100% scale
- Some devices might have different scaling

---

## Brightness Control Issues

### "Backlight control not available"

**Symptom**: Brightness commands fail

**Causes**:
- No backlight device found in `/sys/class/backlight`
- Permission issues
- Desktop PC without backlight

**Solutions**:
```bash
# Check for backlight device
ls /sys/class/backlight/

# If empty, you don't have a backlight
# (common on desktop PCs)

# Check current brightness (if device exists)
cat /sys/class/backlight/*/brightness
cat /sys/class/backlight/*/max_brightness

# Check permissions
ls -la /sys/class/backlight/*/brightness
```

### Brightness changes but display doesn't change

**Symptom**: Command succeeds but screen brightness unchanged

**Solutions**:
```bash
# Check if value actually changed
cat /sys/class/backlight/*/brightness

# Try writing directly (requires root)
echo 50 | sudo tee /sys/class/backlight/*/brightness

# If this doesn't work, your backlight might not be controllable
# via sysfs (try DDC/CI instead)
```

### "Failed to write brightness: Permission denied"

**Symptom**: No permission to change brightness

**Solutions**:
```bash
# Add udev rule to allow brightness control
sudo tee /etc/udev/rules.d/90-backlight.rules <<EOF
ACTION=="add", SUBSYSTEM=="backlight", RUN+="/bin/chgrp video /sys/class/backlight/%k/brightness"
ACTION=="add", SUBSYSTEM=="backlight", RUN+="/bin/chmod g+w /sys/class/backlight/%k/brightness"
EOF

# Add your user to video group
sudo usermod -aG video $USER

# Reload udev rules
sudo udevadm control --reload-rules
sudo udevadm trigger

# Log out and back in
```

---

## Bluetooth Issues

### No Bluetooth devices shown

**Symptom**: Bluetooth popup is empty

**Solutions**:
```bash
# Check BlueZ is running
systemctl status bluetooth
sudo systemctl start bluetooth

# Check adapter is available
bluetoothctl show

# Power on adapter
bluetoothctl power on

# Scan for devices
bluetoothctl scan on

# Check in Amiya
amiya-ctl popup show bluetooth
```

### Can't connect to devices

**Symptom**: Connect button does nothing

**Solutions**:
```bash
# Check device is paired
bluetoothctl devices
bluetoothctl info <MAC>

# Try connecting manually
bluetoothctl connect <MAC>

# Check Amiya logs
journalctl --user -u amiya | grep -i bluetooth
```

### Bluetooth toggle doesn't work

**Symptom**: Can't enable/disable Bluetooth

**Solutions**:
```bash
# Check rfkill
rfkill list bluetooth

# Unblock if blocked
sudo rfkill unblock bluetooth

# Check systemd service
sudo systemctl enable bluetooth
sudo systemctl start bluetooth
```

---

## WiFi Issues

### No WiFi networks shown

**Symptom**: WiFi popup is empty

**Solutions**:
```bash
# Check NetworkManager is running
systemctl status NetworkManager
sudo systemctl start NetworkManager

# Check WiFi device
nmcli device status

# Enable WiFi
nmcli radio wifi on

# Scan for networks
nmcli device wifi rescan

# List networks
nmcli device wifi list
```

### Can't connect to network

**Symptom**: Connect button does nothing

**Solutions**:
```bash
# Try connecting manually
nmcli device wifi connect "SSID" password "PASSWORD"

# Check device status
nmcli device show

# Check Amiya logs
journalctl --user -u amiya | grep -i wifi
```

### Password dialog missing

**Symptom**: Can't enter password for secured networks

**Solution**:
- Password dialog is not yet implemented
- Use `nmcli` to connect first time:
```bash
nmcli device wifi connect "Network Name" password "your-password"
```
- Once saved, Amiya can reconnect

---

## Media Control Issues

### No media player shown

**Symptom**: Media popup says "No track playing"

**Causes**:
- No MPRIS2-compatible player running
- Player not exposing MPRIS2 interface

**Solutions**:
```bash
# Check for MPRIS2 players
playerctl -l

# Start a compatible player:
# - Spotify
# - VLC
# - Firefox (with media playing)
# - Chrome (with media playing)
# - mpv
# - etc.

# Check MPRIS2 status
playerctl status
playerctl metadata
```

### Playback controls don't work

**Symptom**: Play/pause/next/prev buttons do nothing

**Solutions**:
```bash
# Test playerctl directly
playerctl play-pause
playerctl next
playerctl previous

# If playerctl works, check Amiya logs
journalctl --user -u amiya | grep -i media

# Try another player
```

### Wrong player selected

**Symptom**: Controlling different player than expected

**Solution**:
- Amiya selects first discovered player
- Close other players or control manually:
```bash
playerctl -p spotify play-pause
```

---

## Bar Not Showing

### Bar doesn't appear

**Symptom**: Amiya starts but no bar visible

**Solutions**:
```bash
# Check if running
ps aux | grep amiya

# Check logs
journalctl --user -u amiya | grep -i bar

# Check compositor supports wlr-layer-shell
# Required for niri, sway, Hyprland

# Try restarting
pkill amiya
amiya
```

### Bar appears but is blank

**Symptom**: Bar shows but no widgets

**Solutions**:
```bash
# Check configuration
cat ~/.config/amiya/config.toml

# Verify widgets are enabled:
# show_workspaces = true
# show_clock = true
# show_system_info = true

# Check logs for widget errors
journalctl --user -u amiya | grep -i widget
```

---

## Performance Issues

### High CPU usage

**Symptom**: Amiya using excessive CPU

**Solutions**:
```bash
# Check what's consuming CPU
top -p $(pgrep amiya)

# Check polling intervals in logs
journalctl --user -u amiya | grep -i poll

# Possible causes:
# - Rapid D-Bus events
# - System monitoring polling too fast
# - Event loop issues

# Report issue with logs
```

### High memory usage

**Symptom**: Amiya using too much RAM

**Solutions**:
```bash
# Check memory usage
ps aux | grep amiya

# Amiya should use <100MB typically
# If much higher, check for leaks

# Restart as workaround
pkill amiya
amiya
```

### Slow popup opening

**Symptom**: Popups take time to appear

**Solutions**:
- First open is slower (initialization)
- Subsequent opens should be fast
- Check D-Bus performance:
```bash
# Test D-Bus query speed
time bluetoothctl devices
time nmcli device wifi list
```

---

## Build Issues

### GTK4 not found

**Symptom**: Build fails with GTK4 library error

**Solutions**:
```bash
# Arch Linux
sudo pacman -S gtk4 gtk4-layer-shell

# Ubuntu/Debian
sudo apt install libgtk-4-dev gtk4-layer-shell

# Fedora
sudo dnf install gtk4-devel gtk4-layer-shell-devel
```

### zbus compilation errors

**Symptom**: D-Bus related build errors

**Solutions**:
```bash
# Update dependencies
cargo update

# Clean build
cargo clean
cargo build
```

### Missing dependencies

**Symptom**: Can't find various libraries

**Full dependency list**:
```bash
# Arch Linux
sudo pacman -S gtk4 gtk4-layer-shell rust

# Ubuntu/Debian (22.04+)
sudo apt install libgtk-4-dev gtk4-layer-shell \
  libdbus-1-dev pkg-config build-essential

# Fedora
sudo dnf install gtk4-devel gtk4-layer-shell-devel \
  dbus-devel pkg-config gcc
```

---

## General Debugging

### Enable debug logging

```bash
# Set log level
RUST_LOG=debug amiya
```

### Check all backends

```bash
# Bluetooth
bluetoothctl show

# WiFi
nmcli device wifi

# Audio
pactl info

# Backlight
ls /sys/class/backlight/

# Media
playerctl -l
```

### Report issues

When reporting bugs, include:

1. Amiya version: `amiya-ctl status`
2. Compositor: `echo $XDG_CURRENT_DESKTOP`
3. Relevant logs: `journalctl --user -u amiya -n 100`
4. Error message
5. Steps to reproduce

---

## Still Having Issues?

1. Check [HOTKEYS.md](./HOTKEYS.md) for hotkey setup
2. Check [ARCHITECTURE.md](./ARCHITECTURE.md) for system design
3. Check [README.md](../README.md) for general documentation
4. Open an issue on GitHub with debug information
