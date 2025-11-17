use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workspace {
    pub id: u32,
    pub name: Option<String>,
    pub is_active: bool,
    pub is_focused: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspacesResponse {
    pub workspaces: Vec<Workspace>,
}

pub struct NiriClient {
    socket_path: PathBuf,
}

impl NiriClient {
    pub fn new() -> Result<Self> {
        let socket_path = Self::find_socket()?;
        Ok(NiriClient { socket_path })
    }

    fn find_socket() -> Result<PathBuf> {
        // Try to find niri socket
        // Niri typically creates socket at $XDG_RUNTIME_DIR/niri/niri.<display>.sock
        let runtime_dir = std::env::var("XDG_RUNTIME_DIR")
            .or_else(|_| std::env::var("TMPDIR"))
            .unwrap_or_else(|_| "/tmp".to_string());

        let niri_dir = PathBuf::from(runtime_dir).join("niri");

        // Find the first .sock file
        if niri_dir.exists() {
            for entry in std::fs::read_dir(&niri_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("sock") {
                    return Ok(path);
                }
            }
        }

        anyhow::bail!("Could not find niri socket")
    }

    pub fn get_workspaces(&self) -> Result<Vec<Workspace>> {
        // This is a simplified implementation
        // In reality, you'd send a JSON-RPC request to niri's IPC socket
        // and parse the response

        // For now, return mock data
        Ok(vec![
            Workspace {
                id: 1,
                name: Some("1".to_string()),
                is_active: true,
                is_focused: true,
            },
            Workspace {
                id: 2,
                name: Some("2".to_string()),
                is_active: false,
                is_focused: false,
            },
            Workspace {
                id: 3,
                name: Some("3".to_string()),
                is_active: false,
                is_focused: false,
            },
        ])
    }

    pub fn switch_workspace(&self, workspace_id: u32) -> Result<()> {
        // Send IPC message to niri to switch workspace
        // This is a placeholder implementation
        tracing::info!("Switching to workspace {}", workspace_id);
        Ok(())
    }

    // TODO: Implement actual IPC communication
    // Niri's IPC protocol uses JSON-RPC 2.0 over a Unix socket
    // You would need to:
    // 1. Connect to the socket
    // 2. Send JSON-RPC requests
    // 3. Parse JSON-RPC responses
    // 4. Listen for events (workspace changes, etc.)
}

impl Default for NiriClient {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| {
            tracing::warn!("Could not connect to niri, using mock client");
            NiriClient {
                socket_path: PathBuf::from("/tmp/niri-mock.sock"),
            }
        })
    }
}
