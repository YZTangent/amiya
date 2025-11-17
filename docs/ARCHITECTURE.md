# Amiya Architecture Documentation

## Overview

Amiya is a modern, integrated desktop environment for Wayland built around the [niri](https://github.com/YaLTeR/niri) scrollable-tiling compositor. It provides a status bar, system controls, and interactive popups all integrated into a cohesive desktop experience.

## Design Philosophy

1. **Modular Architecture**: Clean separation between UI, state management, and system backends
2. **Event-Driven**: Reactive updates through a central event bus
3. **Wayland-Native**: Built on layer-shell protocol for proper Wayland integration
4. **Composable**: Each component can function independently
5. **Performance**: Rust's zero-cost abstractions with minimal runtime overhead

## Architecture Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                    Amiya Desktop Environment                 │
├─────────────────────────────────────────────────────────────┤
│                          UI Layer                            │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐     │
│  │     Bar      │  │   Popups     │  │  Overlays    │     │
│  │ (Layer Top)  │  │(Layer Over)  │  │(Layer Over)  │     │
│  │              │  │              │  │              │     │
│  │ • Workspaces │  │ • Bluetooth  │  │ • Volume     │     │
│  │ • Clock      │  │ • WiFi       │  │ • Brightness │     │
│  │ • SysInfo    │  │ • Media      │  │              │     │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘     │
│         │                  │                  │             │
│         └──────────────────┴──────────────────┘             │
│                            │                                │
├────────────────────────────┼────────────────────────────────┤
│                   Application State Layer                    │
│                   ┌────────▼────────┐                       │
│                   │  Event Manager  │                       │
│                   │                 │                       │
│                   │ • Message Bus   │                       │
│                   │ • State Store   │                       │
│                   │ • Subscriptions │                       │
│                   └────────┬────────┘                       │
├────────────────────────────┼────────────────────────────────┤
│                      Backend Layer                           │
│         ┌──────────────────┼──────────────────┐            │
│         │                  │                  │             │
│  ┌──────▼──────┐  ┌────────▼────────┐  ┌─────▼──────┐    │
│  │   System    │  │   Niri IPC      │  │   Config   │    │
│  │  Backends   │  │                 │  │   Manager  │    │
│  │             │  │ • JSON-RPC      │  │            │    │
│  │ • Audio     │  │ • Workspaces    │  │ • TOML     │    │
│  │ • Backlight │  │ • Events        │  │ • Themes   │    │
│  │ • Bluetooth │  │ • Commands      │  │ • Hotkeys  │    │
│  │ • Network   │  │                 │  │            │    │
│  │ • Media     │  │                 │  │            │    │
│  └─────────────┘  └─────────────────┘  └────────────┘    │
└─────────┬──────────────────┬───────────────────────────────┘
          │                  │
    ┌─────▼─────┐      ┌─────▼─────┐
    │  D-Bus    │      │   Niri    │
    │ Services  │      │Compositor │
    │           │      │           │
    │ • PulseA  │      │ • IPC     │
    │ • BlueZ   │      │ • Hotkeys │
    │ • NM      │      │ • Events  │
    │ • MPRIS   │      │           │
    └───────────┘      └───────────┘
```

## Core Components

### 1. UI Layer

The UI layer consists of GTK4 windows using the layer-shell protocol to position themselves on the Wayland compositor.

#### Bar (`src/ui/bar.rs`)
- **Purpose**: Main status bar showing workspaces, time, and system information
- **Layer**: Top layer with exclusive zone
- **Position**: Configurable (top/bottom)
- **Components**:
  - Left: Workspace switcher
  - Center: Clock
  - Right: System monitors (CPU, RAM, temp, WiFi, BT)

#### Popups (`src/ui/popups/`)
- **Purpose**: Interactive control panels
- **Layer**: Overlay layer
- **Lifecycle**: Lazy creation, kept alive for performance
- **Types**:
  - **Bluetooth**: Device pairing, connection management
  - **WiFi**: Network selection and connection
  - **Media Control**: Playback controls and volume

#### Overlays (`src/ui/overlays/`)
- **Purpose**: Temporary on-screen displays
- **Layer**: Overlay layer (above everything)
- **Behavior**: Auto-hide after 2 seconds
- **Types**:
  - **Volume Slider**: Shows current volume level
  - **Brightness Slider**: Shows current brightness level

### 2. Application State Layer

The state layer manages application-wide state and event distribution.

#### Event Manager (`src/events.rs`)
- **Architecture**: Channel-based message bus using `tokio::sync::broadcast`
- **Purpose**: Decouple UI from backends, enable reactive updates
- **Message Types**:
  ```rust
  enum Event {
      // Workspace events
      WorkspaceChanged { id: u32 },
      WorkspaceCreated { id: u32 },
      WorkspaceRemoved { id: u32 },

      // System events
      VolumeChanged { level: f64, muted: bool },
      BrightnessChanged { level: f64 },

      // Network events
      WifiStateChanged { enabled: bool },
      WifiNetworkConnected { ssid: String },

      // Bluetooth events
      BluetoothStateChanged { enabled: bool },
      BluetoothDeviceConnected { address: String, name: String },

      // Media events
      MediaPlayerChanged { player: String },
      MediaTrackChanged { title: String, artist: String },
      MediaPlaybackChanged { playing: bool },
  }
  ```

#### Application State (`src/app.rs`)
- **Purpose**: Global application state and coordination
- **Responsibilities**:
  - Initialize all subsystems
  - Coordinate lifecycle of UI components
  - Manage event subscriptions
  - Handle graceful shutdown

### 3. Backend Layer

The backend layer interfaces with system services and the compositor.

#### Niri IPC (`src/backend/niri/`)
- **Protocol**: JSON-RPC 2.0 over Unix socket
- **Socket Location**: `$XDG_RUNTIME_DIR/niri/niri.<display>.sock`
- **Features**:
  - Query workspaces
  - Switch workspaces
  - Subscribe to workspace events
  - Query window information
- **Implementation Strategy**:
  1. Implement JSON-RPC client
  2. Parse niri's protocol from source code
  3. Document protocol for community benefit
  4. Handle connection failures gracefully

#### System Backends (`src/backend/system/`)

##### Audio Control (`audio.rs`)
- **Backend**: PulseAudio/PipeWire via D-Bus
- **D-Bus Interface**: `org.pulseaudio.core1` or `org.pipewire`
- **Operations**:
  - Get/set volume
  - Get/set mute state
  - Subscribe to volume changes
  - List audio devices

##### Backlight Control (`backlight.rs`)
- **Backend**: Direct sysfs access (`/sys/class/backlight/*`)
- **Requirements**: udev rules for non-root access
- **Operations**:
  - Get current brightness
  - Set brightness
  - Get max brightness

##### Bluetooth Control (`bluetooth.rs`)
- **Backend**: BlueZ via D-Bus
- **D-Bus Interface**: `org.bluez`
- **Operations**:
  - Enable/disable adapter
  - Scan for devices
  - Pair devices
  - Connect/disconnect devices
  - Subscribe to device events

##### Network Control (`network.rs`)
- **Backend**: NetworkManager via D-Bus
- **D-Bus Interface**: `org.freedesktop.NetworkManager`
- **Operations**:
  - Enable/disable WiFi
  - Scan for networks
  - Connect to network
  - Disconnect from network
  - Subscribe to connection events

##### Media Control (`media.rs`)
- **Backend**: MPRIS2 via D-Bus
- **D-Bus Interface**: `org.mpris.MediaPlayer2.*`
- **Operations**:
  - List active players
  - Get playback status
  - Get track metadata
  - Control playback (play/pause/next/previous)
  - Get/set volume
  - Subscribe to player events

## Event Flow Examples

### Volume Change Flow
```
User presses XF86AudioRaiseVolume (via niri hotkey)
  ↓
Niri executes: amiya-ctl volume up
  ↓
AudioControl.increase_volume()
  ↓
D-Bus call to PulseAudio
  ↓
PulseAudio emits VolumeChanged signal
  ↓
AudioControl receives signal
  ↓
EventManager.emit(VolumeChanged { level: 75.0, muted: false })
  ↓
┌────────────────────┬────────────────────┐
│                    │                    │
VolumeOverlay     SystemInfo          (any other
.show(75.0)       .update()            subscribers)
```

### Workspace Switch Flow
```
User switches workspace (via niri)
  ↓
Niri compositor updates internal state
  ↓
Niri IPC emits WorkspaceActivated event
  ↓
NiriClient receives event over Unix socket
  ↓
EventManager.emit(WorkspaceChanged { id: 2 })
  ↓
Workspaces widget updates highlight
```

### Bluetooth Connection Flow
```
User clicks "Connect" in Bluetooth popup
  ↓
BluetoothPopup calls BluetoothControl.connect(address)
  ↓
D-Bus call to BlueZ: Connect(address)
  ↓
BlueZ establishes connection
  ↓
BlueZ emits DeviceConnected signal
  ↓
BluetoothControl receives signal
  ↓
EventManager.emit(BluetoothDeviceConnected { address, name })
  ↓
BluetoothPopup updates UI state
```

## Key Design Decisions

### 1. State Management: Channel-Based Event Bus ✅

**Decision**: Implement a lightweight event bus using Tokio channels.

**Rationale**:
- Clean separation of concerns
- Easy to test (inject mock event streams)
- Reactive updates without tight coupling
- Naturally async-friendly
- Low overhead (lock-free channels)

**Alternative Considered**: Direct widget updates
- Simpler but creates tight coupling
- Hard to test
- Difficult to add new subscribers

### 2. Niri Integration: JSON-RPC Implementation ✅

**Decision**: Implement full JSON-RPC client for niri's IPC protocol.

**Rationale**:
- Proper event subscription support
- Future-proof as niri evolves
- Can document protocol for community
- More reliable than polling

**Alternative Considered**: Execute niri commands + polling
- Simpler but no event support
- Performance overhead from polling
- Can't detect external changes

### 3. Hotkey Strategy: Niri Native with Fallback ✅

**Decision**: Primary support for niri's native hotkey bindings, secondary support for swhkd.

**Rationale**:
- Best UX with niri (target compositor)
- No extra daemon required
- Fallback for other Wayland compositors
- Simple integration via `amiya-ctl` tool

**Niri Config Example**:
```kdl
binds {
    Mod+B { spawn "amiya-ctl" "popup" "bluetooth"; }
    Mod+W { spawn "amiya-ctl" "popup" "wifi"; }
    Mod+M { spawn "amiya-ctl" "popup" "media"; }

    XF86AudioRaiseVolume { spawn "amiya-ctl" "volume" "up"; }
    XF86AudioLowerVolume { spawn "amiya-ctl" "volume" "down"; }
    XF86AudioMute { spawn "amiya-ctl" "volume" "mute"; }

    XF86MonBrightnessUp { spawn "amiya-ctl" "brightness" "up"; }
    XF86MonBrightnessDown { spawn "amiya-ctl" "brightness" "down"; }
}
```

### 4. Popup Lifecycle: Lazy Creation ✅

**Decision**: Create popups on first use, keep alive, add idle cleanup.

**Rationale**:
- Fast response after first use (common case)
- Manageable memory overhead (3 popups)
- Can add cleanup timer if needed

**Implementation**:
- Use `lazy_static` or `OnceCell` for popup instances
- Create on first show
- Add 5-minute idle timer to destroy if unused

### 5. Project Structure: Backend/UI Separation ✅

**Decision**: Reorganize into `src/backend/` and `src/ui/` modules.

**New Structure**:
```
src/
├── main.rs              # Entry point
├── app.rs               # Application state coordinator
├── events.rs            # Event bus and message types
├── config.rs            # Configuration management
│
├── ui/                  # User interface layer
│   ├── mod.rs
│   ├── bar.rs
│   ├── widgets/
│   │   ├── clock.rs
│   │   ├── workspaces.rs
│   │   └── system_info.rs
│   ├── popups/
│   │   ├── bluetooth.rs
│   │   ├── wifi.rs
│   │   └── media_control.rs
│   └── overlays/
│       └── slider.rs
│
└── backend/             # System integration layer
    ├── mod.rs
    ├── niri/
    │   ├── mod.rs
    │   ├── client.rs     # JSON-RPC client
    │   ├── protocol.rs   # Protocol types
    │   └── events.rs     # Event handling
    └── system/
        ├── mod.rs
        ├── audio.rs      # PulseAudio/PipeWire
        ├── backlight.rs  # Brightness control
        ├── bluetooth.rs  # BlueZ
        ├── network.rs    # NetworkManager
        └── media.rs      # MPRIS2
```

## Error Handling Strategy

### Graceful Degradation
- If niri IPC unavailable → Show mock workspaces with warning
- If audio backend fails → Show "N/A" in UI, disable controls
- If network backend fails → Show last known state, disable controls
- If Bluetooth unavailable → Hide Bluetooth widget entirely

### Error Reporting
- Log all errors with `tracing` crate
- Show user-friendly notifications for critical errors
- Never crash the bar (catch panics in widgets)

### Recovery Mechanisms
- Retry D-Bus connections with exponential backoff
- Reconnect to niri socket on disconnect
- Refresh stale data on focus

## Performance Considerations

### Update Frequencies
- **Clock**: 1 second
- **CPU/Memory**: 2 seconds
- **Temperature**: 5 seconds
- **Network/Bluetooth status**: Event-driven (no polling)
- **Workspace state**: Event-driven (no polling)

### Resource Usage
- Single GTK application with multiple windows
- Lazy widget creation where possible
- Efficient channel broadcasting (no cloning for unsubscribed events)
- Connection pooling for D-Bus

### Optimization Opportunities
- Debounce rapid events (e.g., volume changes)
- Batch widget updates in single frame
- Use `gtk::idle_add` for non-critical updates
- Cache D-Bus proxy objects

## Security Considerations

### Privilege Separation
- No root privileges required
- Backlight control via udev rules
- Audio control via user session D-Bus
- Network/Bluetooth via system D-Bus with PolicyKit

### Input Validation
- Sanitize all config file inputs
- Validate D-Bus responses
- Bounds-check brightness/volume values
- Escape special characters in network SSIDs

### Attack Surface
- Minimal: Only listens on D-Bus session bus
- No network listeners
- No external command execution (except via amiya-ctl)
- All IPC over local Unix sockets

## Testing Strategy

### Unit Tests
- Config parsing and validation
- Event bus message passing
- Protocol serialization/deserialization
- Individual widget logic

### Integration Tests
- Mock D-Bus services for backend testing
- Mock niri IPC for workspace testing
- UI state transitions

### Manual Testing Checklist
- [ ] Bar displays on all monitors
- [ ] Workspace switching updates highlight
- [ ] Volume changes show overlay
- [ ] Brightness changes show overlay
- [ ] Bluetooth popup shows devices
- [ ] WiFi popup shows networks
- [ ] Media controls work with active player
- [ ] Configuration changes apply immediately
- [ ] System resume recovers all connections

## Future Enhancements

### Planned Features
- Battery/power widget
- Network speed monitor
- System notifications
- Application launcher
- System tray (XDG Status Notifier)
- Screenshot integration
- Screen recording integration
- Power menu (suspend/hibernate/shutdown)

### Plugin System
- Hot-reload widget plugins
- Custom widget API
- Scripting support (Lua/Rhai)
- Theme marketplace

### Multi-Monitor Support
- Per-monitor bars
- Workspace-per-monitor mode
- Monitor hotplug handling

## Dependencies

### Core
- `gtk4` - UI toolkit
- `gtk4-layer-shell` - Wayland layer protocol
- `tokio` - Async runtime
- `serde` - Serialization
- `toml` - Config parsing

### System Integration
- `zbus` - D-Bus client
- `sysinfo` - System monitoring
- `wayland-client` - Wayland protocol

### Utilities
- `anyhow` - Error handling
- `tracing` - Logging
- `chrono` - Time handling
- `lazy_static` - Lazy initialization

## References

- [Niri GitHub](https://github.com/YaLTeR/niri)
- [Layer Shell Protocol](https://wayland.app/protocols/wlr-layer-shell-unstable-v1)
- [GTK4 Documentation](https://docs.gtk.org/gtk4/)
- [D-Bus Specification](https://dbus.freedesktop.org/doc/dbus-specification.html)
- [MPRIS2 Specification](https://specifications.freedesktop.org/mpris-spec/latest/)
- [NetworkManager D-Bus API](https://networkmanager.dev/docs/api/latest/)
- [BlueZ D-Bus API](https://git.kernel.org/pub/scm/bluetooth/bluez.git/tree/doc)
