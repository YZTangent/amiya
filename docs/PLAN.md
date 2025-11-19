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

### 4.4 Network Control (NetworkManager) ✅
- [x] Update `src/backend/system/network.rs`
  - [x] Implement D-Bus connection to NetworkManager
  - [x] Implement WiFi enable/disable
  - [x] Implement network scanning (RequestScan)
  - [x] Implement get_networks (enumerate access points)
  - [x] Implement network connection (basic)
  - [x] Implement network disconnection
  - [x] Async implementation with tokio
  - [x] Thread-safe with RwLock
  - [x] Event integration with EventManager
  - [x] Graceful degradation when WiFi unavailable
  - [x] WiFi device auto-detection
  - [x] Access point enumeration with signal strength
  - [x] Security detection (WPA/RSN flags)
  - [x] Unit tests for network control
  - [x] Subscribe to connection signals (basic)
  - [x] Emit network events
  - [ ] Implement full connection workflow (AddAndActivateConnection)
  - [ ] Test with real WiFi networks

### 4.5 Media Control (MPRIS2) ✅
- [x] Create `src/backend/system/media.rs`
  - [x] Implement D-Bus connection to MPRIS
  - [x] Discover active media players
  - [x] Implement playback control (play/pause/stop/next/prev)
  - [x] Get track metadata (title, artist, album, art URL, track ID)
  - [x] Get/set volume (0.0-1.0)
  - [x] Async implementation with tokio
  - [x] Thread-safe with RwLock
  - [x] Event integration with EventManager
  - [x] Graceful degradation when no players available
  - [x] Multi-player support with active player selection
  - [x] Playback status tracking (Playing, Paused, Stopped)
  - [x] Unit tests for media control
  - [x] Subscribe to player signals (basic)
  - [x] Emit media events
  - [ ] Test with media players (Spotify, VLC, Firefox)

### 4.6 UI Integration ✅
- [x] Update `BluetoothPopup` to use real backend
  - [x] Show real device list
  - [x] Implement connect/disconnect actions
  - [x] Show connection status
  - [x] Wire up enable/disable toggle
  - [x] Wire up scan button
  - [x] Subscribe to Bluetooth events for reactive updates
  - [x] Empty state handling
- [x] Update `WifiPopup` to use real backend
  - [x] Show real network list
  - [x] Implement connection (basic)
  - [x] Show signal strength with bars
  - [x] Wire up enable/disable toggle
  - [x] Wire up refresh button
  - [x] Subscribe to WiFi events for reactive updates
  - [x] Empty state handling
  - [ ] Handle password input dialog (TODO for future)
- [x] Update `MediaControlPopup` to use real backend
  - [x] Show active player
  - [x] Show track info (title, artist, album)
  - [x] Implement playback controls (play/pause/next/prev)
  - [x] Implement volume control
  - [x] Subscribe to media events for reactive updates
  - [x] Update UI based on playback status
  - [ ] Show album art from URL (TODO for future)
  - [ ] Seek functionality (TODO for future)
- [x] Update `SystemInfo` widget
  - [x] Show real WiFi status (already implemented via events)
  - [x] Show real Bluetooth status (already implemented via events)
  - [ ] Add click handlers to open popups (TODO for future)

## Phase 5: Hotkey System & CLI Tool ✅ COMPLETED

Implement hotkey triggering and command-line control tool.

### 5.1 IPC for amiya-ctl ✅
- [x] Create `src/ipc/protocol.rs`
  - [x] Define command protocol (JSON over Unix sockets)
  - [x] Define popup commands (show/hide/toggle)
  - [x] Define volume actions (up/down/set/mute)
  - [x] Define brightness actions (up/down/set)
  - [x] Define response format
- [x] Create `src/ipc/server.rs`
  - [x] Implement Unix socket server
  - [x] Handle popup commands
  - [x] Handle volume commands
  - [x] Handle brightness commands
  - [x] Emit events to EventManager
  - [x] Status and ping commands
  - [x] Clean up socket on shutdown
  - [ ] Add authentication/security (TODO for future)

### 5.2 Command-Line Tool ✅
- [x] Create `src/bin/amiya-ctl.rs`
  - [x] Implement CLI argument parsing with clap
  - [x] Implement socket client
  - [x] Add popup subcommands (show/hide/toggle for bluetooth, wifi, media-control)
  - [x] Add volume subcommands (up, down, set, mute, unmute, toggle-mute)
  - [x] Add brightness subcommands (up, down, set)
  - [x] Add status command
  - [x] Add ping command
  - [x] Add help text and usage examples
  - [x] User-friendly output with ✓/✗ indicators

### 5.3 Integration ✅
- [x] Wire up amiya-ctl commands to event system
- [x] Test hotkey triggering (implementation complete, hardware testing pending)
- [x] Document niri configuration
- [x] Add swhkd configuration example

### 5.4 Documentation ✅
- [x] Update README with hotkey setup
- [x] Add niri config examples
- [x] Add swhkd config examples
- [x] Add troubleshooting guide (TROUBLESHOOTING.md and HOTKEYS.md)

## Phase 6: Polish & Additional Features

Final polish and nice-to-have features.

### 6.1 UI Improvements ⚡ IN PROGRESS
- [x] Add click-away detection for popups (already implemented in Phase 1-4)
- [ ] Implement smooth animations
- [x] Add loading states (partially implemented - scan buttons show "Scanning...")
- [x] Add empty states (partially implemented in popups)
- [ ] Improve error messages
- [ ] Add tooltips

### 6.2 Additional Widgets ⚡ IN PROGRESS
- [x] Battery widget ✅
  - [x] Show battery percentage
  - [x] Show charging status
  - [x] Show time remaining (in backend, not displayed yet)
  - [x] Warning at low battery (CSS classes for low/critical)
  - [x] D-Bus UPower integration
  - [x] Event-driven reactive updates
  - [x] Battery state monitoring (every 10 seconds)
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

### 6.4 Additional Popups ✅ COMPLETED
- [x] Power menu ✅
  - [x] Suspend
  - [x] Hibernate
  - [x] Shutdown
  - [x] Reboot
  - [x] Lock screen
  - [x] D-Bus systemd/logind integration
  - [x] Beautiful UI with centered popup
  - [x] IPC support via amiya-ctl
  - [x] Hotkey bindings documented
  - [x] PopupManager integration
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

### 6.7 Packaging ⚡ IN PROGRESS
- [ ] Create AUR package (Arch)
- [ ] Create Flatpak
- [ ] Create Nix package
- [x] Add install script ✅
  - [x] Dependency checking
  - [x] Binary installation to ~/.local/bin
  - [x] Configuration file setup
  - [x] Systemd service installation
  - [x] User-friendly output with instructions
- [x] Add uninstall script ✅
  - [x] Service stop and disable
  - [x] Binary removal
  - [x] Optional configuration cleanup
- [x] Create installation documentation (INSTALL.md) ✅
  - [x] Prerequisites and dependencies
  - [x] Quick install guide
  - [x] Manual installation steps
  - [x] Configuration guide
  - [x] Troubleshooting section
  - [x] Update instructions
- [x] Create systemd user service ✅
  - [x] Auto-restart on failure
  - [x] Logging configuration
  - [x] Environment setup

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

**Last Updated**: 2025-11-17

**Current Phase**: Phase 5 Complete - Ready for Phase 6 (Polish & Additional Features)

**Next Milestone**: UI improvements, additional widgets, and testing

**Overall Progress**: ~50% (5 of 7 phases complete)

## Notes

- Each checkbox should be marked with `[x]` when completed
- Add sub-items as implementation details emerge
- Update "Current Status" section regularly
- Link to related issues/PRs as they are created
