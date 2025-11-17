use super::client::NiriClient;
use super::protocol::NiriEvent;
use crate::events::{Event, EventManager, WorkspaceInfo};
use crate::error::Result;
use std::io::{BufRead, BufReader};
use std::sync::Arc;
use tracing::{debug, info, warn};

/// Event listener for niri compositor events
pub struct NiriEventListener {
    client: Arc<NiriClient>,
    events: EventManager,
}

impl NiriEventListener {
    pub fn new(client: Arc<NiriClient>, events: EventManager) -> Self {
        NiriEventListener { client, events }
    }

    /// Start listening for niri events
    /// This should be called in a background thread
    pub fn start_listening(&self) -> Result<()> {
        info!("Starting niri event listener");

        // TODO: Implement event subscription via niri's EventStream command
        // For now, we'll poll workspace state periodically
        // In Phase 3.3, we'll implement proper event streaming

        Ok(())
    }

    /// Poll workspace state and emit events
    /// This is a temporary solution until we implement proper event streaming
    pub fn poll_workspaces(&self) -> Result<()> {
        match self.client.get_workspaces() {
            Ok(niri_workspaces) => {
                // Convert niri workspaces to our event format
                let workspaces: Vec<WorkspaceInfo> = niri_workspaces
                    .iter()
                    .map(|ws| WorkspaceInfo {
                        id: ws.idx,
                        name: ws.name.clone(),
                        is_active: ws.is_active,
                        is_focused: ws.is_focused,
                    })
                    .collect();

                // Emit workspace update event
                self.events.emit(Event::WorkspacesUpdated { workspaces });

                Ok(())
            }
            Err(e) => {
                debug!("Failed to poll workspaces: {}", e);
                Ok(()) // Don't fail, just log
            }
        }
    }

    /// Handle a niri event and emit corresponding application events
    fn handle_niri_event(&self, niri_event: NiriEvent) {
        match niri_event {
            NiriEvent::WorkspaceActivated { id, focused } => {
                debug!("Workspace activated: id={}, focused={}", id, focused);
                // Convert from u64 to u32 (niri uses u64, we use u32)
                self.events.emit(Event::WorkspaceChanged { id: id as u32 });
            }
            NiriEvent::WorkspacesChanged { workspaces } => {
                debug!("Workspaces changed: {} workspaces", workspaces.len());
                let workspace_infos: Vec<WorkspaceInfo> = workspaces
                    .iter()
                    .map(|ws| WorkspaceInfo {
                        id: ws.idx,
                        name: ws.name.clone(),
                        is_active: ws.is_active,
                        is_focused: ws.is_focused,
                    })
                    .collect();
                self.events.emit(Event::WorkspacesUpdated {
                    workspaces: workspace_infos,
                });
            }
            NiriEvent::WorkspaceActiveWindowChanged {
                workspace_id,
                window_id,
            } => {
                debug!(
                    "Active window changed on workspace {}: {:?}",
                    workspace_id, window_id
                );
                // We don't have a specific event for this yet
            }
            NiriEvent::WindowOpenedOrChanged { window } => {
                debug!("Window opened or changed: {:?}", window.title);
                // Could emit an event if we add window tracking
            }
            NiriEvent::WindowClosed { id } => {
                debug!("Window closed: {}", id);
            }
            NiriEvent::WindowFocusChanged { id } => {
                debug!("Window focus changed: {:?}", id);
            }
            NiriEvent::KeyboardLayoutsChanged {
                keyboard_layouts,
                current_idx,
            } => {
                debug!(
                    "Keyboard layouts changed: {} layouts, current: {}",
                    keyboard_layouts.len(),
                    current_idx
                );
            }
            NiriEvent::KeyboardLayoutSwitched { idx } => {
                debug!("Keyboard layout switched to: {}", idx);
            }
        }
    }
}

/// Start polling workspace state periodically
/// This is a temporary solution until proper event streaming is implemented
pub fn start_workspace_polling(
    client: Arc<NiriClient>,
    events: EventManager,
    interval_seconds: u64,
) {
    use gtk4::glib;

    let listener = NiriEventListener::new(client, events);

    // Initial poll
    if let Err(e) = listener.poll_workspaces() {
        warn!("Initial workspace poll failed: {}", e);
    }

    // Poll periodically
    glib::timeout_add_seconds_local(interval_seconds, move || {
        if let Err(e) = listener.poll_workspaces() {
            warn!("Workspace poll failed: {}", e);
        }
        glib::ControlFlow::Continue
    });
}
