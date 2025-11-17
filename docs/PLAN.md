# Amiya Implementation Plan

This document tracks the implementation progress of Amiya. Check off items as they are completed.

## Phase 1: Foundation ✅ COMPLETED

Basic project structure and UI components with mock data.

- [x] Project structure setup
- [x] Cargo.toml with dependencies
- [x] Basic GTK4 application skeleton
- [x] Configuration system (TOML-based)
- [x] Main status bar with layer-shell
- [x] Workspace widget (mock data)
- [x] Clock widget
- [x] System info widget (mock data)
- [x] Bluetooth popup (UI only)
- [x] WiFi popup (UI only)
- [x] Media control popup (UI only)
- [x] Volume overlay
- [x] Brightness overlay
- [x] README documentation
- [x] Example configuration file
- [x] MIT License

## Phase 2: Event System & Application State ⚡ IN PROGRESS

Implement the central event bus and refactor components to use reactive updates.

### 2.1 Event System Core ✅
- [x] Create `src/events.rs`
  - [x] Define `Event` enum with all message types
  - [x] Implement `EventManager` with broadcast channels
  - [x] Add subscription mechanism
  - [x] Add event emission helpers
  - [x] Write unit tests for event bus

### 2.2 Application State Manager ✅
- [x] Create `src/app.rs`
  - [x] Define `AppState` struct to hold global state
  - [x] Implement initialization logic
  - [x] Add component lifecycle management
  - [x] Add graceful shutdown handling
  - [x] Integrate system monitoring (CPU, memory, temperature)

### 2.3 Refactor UI to Use Events ✅
- [x] Refactor `Bar` to accept AppState
- [x] Refactor `Workspaces` widget
  - [x] Subscribe to `WorkspaceChanged` events
  - [x] Update UI on event receive
  - [x] Emit events on button clicks
- [x] Refactor `SystemInfo` widget
  - [x] Subscribe to system events
  - [x] Update CPU/RAM/temp on event receive
  - [x] Subscribe to WiFi/Bluetooth state changes
- [x] Refactor `Clock` widget to accept AppState
- [ ] Refactor `VolumeOverlay`
  - [ ] Subscribe to `VolumeChanged` events
  - [ ] Show overlay on event receive
- [ ] Refactor `BrightnessOverlay`
  - [ ] Subscribe to `BrightnessChanged` events
  - [ ] Show overlay on event receive

### 2.4 Error Handling
- [ ] Add proper error types
- [ ] Implement graceful degradation for missing backends
- [ ] Add error logging with tracing
- [ ] Add user-facing error notifications

## Phase 3: Niri IPC Integration

Implement JSON-RPC client to communicate with niri compositor.

### 3.1 Protocol Implementation
- [ ] Create `src/backend/niri/protocol.rs`
  - [ ] Define JSON-RPC request/response types
  - [ ] Define workspace data structures
  - [ ] Define event notification types
  - [ ] Add serialization/deserialization

### 3.2 Client Implementation
- [ ] Create `src/backend/niri/client.rs`
  - [ ] Implement Unix socket connection
  - [ ] Implement JSON-RPC request sending
  - [ ] Implement JSON-RPC response parsing
  - [ ] Add connection retry logic
  - [ ] Add timeout handling

### 3.3 Event Handling
- [ ] Create `src/backend/niri/events.rs`
  - [ ] Implement event subscription
  - [ ] Parse workspace events
  - [ ] Emit to EventManager
  - [ ] Handle reconnection on disconnect

### 3.4 Commands
- [ ] Implement workspace queries
  - [ ] List all workspaces
  - [ ] Get active workspace
  - [ ] Get workspace details
- [ ] Implement workspace commands
  - [ ] Switch to workspace by ID
  - [ ] Switch to workspace by name
- [ ] Implement window queries (nice-to-have)
  - [ ] List windows on workspace
  - [ ] Get focused window

### 3.5 Integration
- [ ] Update `Workspaces` widget to use real niri data
- [ ] Add click handlers to switch workspaces
- [ ] Test workspace switching
- [ ] Handle niri not running gracefully

## Phase 4: D-Bus System Integration

Implement D-Bus backends for system control.

### 4.1 Audio Control (PulseAudio/PipeWire)
- [ ] Update `src/backend/system/audio.rs`
  - [ ] Implement D-Bus connection to PulseAudio
  - [ ] Implement get_volume()
  - [ ] Implement set_volume()
  - [ ] Implement get_mute()
  - [ ] Implement toggle_mute()
  - [ ] Subscribe to volume change signals
  - [ ] Emit VolumeChanged events
  - [ ] Test with real audio system

### 4.2 Backlight Control
- [ ] Update `src/backend/system/backlight.rs`
  - [ ] Add error handling for missing backlight
  - [ ] Implement increase/decrease helpers
  - [ ] Add bounds checking
  - [ ] Emit BrightnessChanged events
  - [ ] Document udev rules in README
  - [ ] Test on real hardware

### 4.3 Bluetooth Control (BlueZ)
- [ ] Update `src/backend/system/bluetooth.rs`
  - [ ] Implement D-Bus connection to BlueZ
  - [ ] Implement adapter control (enable/disable)
  - [ ] Implement device scanning
  - [ ] Implement device pairing
  - [ ] Implement device connection/disconnection
  - [ ] Subscribe to device signals
  - [ ] Emit Bluetooth events
  - [ ] Test with real Bluetooth devices

### 4.4 Network Control (NetworkManager)
- [ ] Update `src/backend/system/network.rs`
  - [ ] Implement D-Bus connection to NetworkManager
  - [ ] Implement WiFi enable/disable
  - [ ] Implement network scanning
  - [ ] Implement network connection
  - [ ] Implement network disconnection
  - [ ] Subscribe to connection signals
  - [ ] Emit network events
  - [ ] Test with real WiFi networks

### 4.5 Media Control (MPRIS2)
- [ ] Create `src/backend/system/media.rs`
  - [ ] Implement D-Bus connection to MPRIS
  - [ ] Discover active media players
  - [ ] Implement playback control (play/pause/next/prev)
  - [ ] Get track metadata
  - [ ] Get/set volume
  - [ ] Subscribe to player signals
  - [ ] Emit media events
  - [ ] Test with media players (Spotify, VLC, Firefox)

### 4.6 UI Integration
- [ ] Update `BluetoothPopup` to use real backend
  - [ ] Show real device list
  - [ ] Implement connect/disconnect actions
  - [ ] Show connection status
- [ ] Update `WifiPopup` to use real backend
  - [ ] Show real network list
  - [ ] Implement connection dialog
  - [ ] Show signal strength
  - [ ] Handle password input
- [ ] Update `MediaControlPopup` to use real backend
  - [ ] Show active player
  - [ ] Show track info
  - [ ] Implement playback controls
  - [ ] Show album art (if available)
- [ ] Update `SystemInfo` widget
  - [ ] Show real WiFi status
  - [ ] Show real Bluetooth status
  - [ ] Add click handlers to open popups

## Phase 5: Hotkey System & CLI Tool

Implement hotkey triggering and command-line control tool.

### 5.1 IPC for amiya-ctl
- [ ] Create `src/ipc/server.rs`
  - [ ] Implement Unix socket server
  - [ ] Define command protocol
  - [ ] Handle popup commands
  - [ ] Handle volume commands
  - [ ] Handle brightness commands
  - [ ] Add authentication/security

### 5.2 Command-Line Tool
- [ ] Create `src/bin/amiya-ctl.rs`
  - [ ] Implement CLI argument parsing
  - [ ] Implement socket client
  - [ ] Add popup subcommands (bluetooth, wifi, media)
  - [ ] Add volume subcommands (up, down, mute)
  - [ ] Add brightness subcommands (up, down)
  - [ ] Add help text and usage examples

### 5.3 Integration
- [ ] Wire up amiya-ctl commands to event system
- [ ] Test hotkey triggering
- [ ] Document niri configuration
- [ ] Add swhkd configuration example

### 5.4 Documentation
- [ ] Update README with hotkey setup
- [ ] Add niri config examples
- [ ] Add swhkd config examples
- [ ] Add troubleshooting guide

## Phase 6: Polish & Additional Features

Final polish and nice-to-have features.

### 6.1 UI Improvements
- [ ] Add click-away detection for popups
- [ ] Implement smooth animations
- [ ] Add loading states
- [ ] Add empty states (no devices, no networks)
- [ ] Improve error messages
- [ ] Add tooltips

### 6.2 Additional Widgets
- [ ] Battery widget
  - [ ] Show battery percentage
  - [ ] Show charging status
  - [ ] Show time remaining
  - [ ] Warning at low battery
- [ ] Network speed widget
  - [ ] Show upload/download speeds
  - [ ] Click to show details
- [ ] Notification support
  - [ ] Listen to desktop notifications
  - [ ] Show notification count
  - [ ] Click to show notification center

### 6.3 System Tray
- [ ] Implement Status Notifier Item protocol
- [ ] Show tray icons in bar
- [ ] Handle icon clicks
- [ ] Support tray menus

### 6.4 Additional Popups
- [ ] Power menu
  - [ ] Suspend
  - [ ] Hibernate
  - [ ] Shutdown
  - [ ] Reboot
  - [ ] Lock screen
- [ ] Calendar popup
  - [ ] Show calendar on clock click
  - [ ] Show upcoming events

### 6.5 Performance Optimization
- [ ] Profile with perf/flamegraph
- [ ] Optimize event handling
- [ ] Reduce memory usage
- [ ] Debounce rapid events
- [ ] Lazy-load heavy widgets

### 6.6 Testing
- [ ] Write unit tests for all backends
- [ ] Write integration tests
- [ ] Add CI/CD pipeline
- [ ] Test on multiple distributions
- [ ] Test with multiple monitors
- [ ] Test with different themes

### 6.7 Packaging
- [ ] Create AUR package (Arch)
- [ ] Create Flatpak
- [ ] Create Nix package
- [ ] Add install script
- [ ] Add uninstall script

## Phase 7: Documentation & Community

Complete documentation and prepare for community contributions.

### 7.1 Documentation
- [ ] Complete API documentation
- [ ] Add code examples
- [ ] Create user guide
- [ ] Create developer guide
- [ ] Add screenshots/videos
- [ ] Create wiki

### 7.2 Configuration
- [ ] Add more theme presets
- [ ] Add widget customization options
- [ ] Add position/size configuration
- [ ] Add module enable/disable toggles

### 7.3 Community
- [ ] Set up issue templates
- [ ] Add contribution guidelines
- [ ] Add code of conduct
- [ ] Create Discord/Matrix channel
- [ ] Write blog post announcement

## Current Status

**Last Updated**: 2024-11-17

**Current Phase**: Phase 2 (Event System)

**Next Milestone**: Complete event bus and refactor widgets

## Notes

- Each checkbox should be marked with `[x]` when completed
- Add sub-items as implementation details emerge
- Update "Current Status" section regularly
- Link to related issues/PRs as they are created
