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

## Phase 2: Event System & Application State ✅ COMPLETED

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
- [x] Refactor `VolumeOverlay`
  - [x] Subscribe to `VolumeChanged` events
  - [x] Show overlay on event receive
  - [x] Support muted state
- [x] Refactor `BrightnessOverlay`
  - [x] Subscribe to `BrightnessChanged` events
  - [x] Show overlay on event receive
- [x] Create `OverlayManager` to coordinate overlays

### 2.4 Error Handling ✅
- [x] Add proper error types (`src/error.rs`)
- [x] Implement graceful degradation for missing backends
- [x] Add error logging with tracing
- [x] Add panic catching for system monitors
- [x] Backend availability status tracking

## Phase 3: Niri IPC Integration ✅ COMPLETED

Implement JSON-RPC client to communicate with niri compositor.

### 3.1 Protocol Implementation ✅
- [x] Create `src/backend/niri/protocol.rs`
  - [x] Define JSON-RPC request/response types
  - [x] Define workspace data structures
  - [x] Define event notification types
  - [x] Add serialization/deserialization
  - [x] Add unit tests for serialization

### 3.2 Client Implementation ✅
- [x] Create `src/backend/niri/client.rs`
  - [x] Implement Unix socket connection
  - [x] Implement JSON-RPC request sending
  - [x] Implement JSON-RPC response parsing
  - [x] Add connection management and cleanup
  - [x] Add socket path detection (XDG_RUNTIME_DIR)
  - [x] Atomic request ID generation

### 3.3 Event Handling ✅
- [x] Create `src/backend/niri/events.rs`
  - [x] Implement workspace polling
  - [x] Parse workspace events
  - [x] Emit to EventManager
  - [x] Handle niri unavailable gracefully
  - [x] Convert NiriWorkspace to WorkspaceInfo

### 3.4 Commands ✅
- [x] Implement workspace queries
  - [x] List all workspaces (get_workspaces)
  - [x] Get workspace with index and state
- [x] Implement workspace commands
  - [x] Switch to workspace by ID (focus_workspace)
  - [x] Switch to workspace by name (focus_workspace_by_name)
  - [x] Action-based command system
- [x] Implement version query for debugging

### 3.5 Integration ✅
- [x] Add NiriClient to AppState (optional)
- [x] Update `Workspaces` widget to use real niri data
- [x] Add click handlers to switch workspaces via IPC
- [x] Test workspace switching with fallback
- [x] Handle niri not running gracefully
- [x] Start workspace polling on app init
- [x] Backend availability status tracking

## Phase 4: D-Bus System Integration ⚡ IN PROGRESS

Implement D-Bus backends for system control.

### 4.1 Audio Control (PulseAudio/PipeWire) ✅
- [x] Create `src/backend/system/audio.rs`
  - [x] Implement async AudioControl with tokio
  - [x] Implement get_volume() and set_volume()
  - [x] Implement increase/decrease volume helpers
  - [x] Implement get_mute(), set_mute(), toggle_mute()
  - [x] Emit VolumeChanged events
  - [x] Add to AppState with event integration
  - [x] Thread-safe with RwLock
  - [x] Unit tests for volume and mute control
  - [x] Graceful connection handling

### 4.2 Backlight Control ✅
- [x] Update `src/backend/system/backlight.rs`
  - [x] Add error handling for missing backlight
  - [x] Implement increase/decrease helpers
  - [x] Add bounds checking
  - [x] Emit BrightnessChanged events
  - [x] Async implementation with tokio
  - [x] Thread-safe with RwLock
  - [x] Event integration with EventManager
  - [x] Graceful degradation when backlight unavailable
  - [x] Unit tests for brightness control
  - [x] Preferred device detection (intel_backlight, amdgpu_bl0, etc.)
  - [ ] Document udev rules in README
  - [ ] Test on real hardware

### 4.3 Bluetooth Control (BlueZ) ✅
- [x] Update `src/backend/system/bluetooth.rs`
  - [x] Implement D-Bus connection to BlueZ
  - [x] Implement adapter control (powered on/off)
  - [x] Implement device scanning (start/stop discovery)
  - [x] Implement device pairing
  - [x] Implement device connection/disconnection
  - [x] Implement device removal/unpairing
  - [x] Async implementation with tokio
  - [x] Thread-safe with RwLock
  - [x] Event integration with EventManager
  - [x] Graceful degradation when Bluetooth unavailable
  - [x] Adapter auto-detection (hci0, hci1, hci2)
  - [x] Unit tests for bluetooth control
  - [x] Subscribe to device signals (basic)
  - [x] Emit Bluetooth events
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
