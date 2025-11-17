use super::protocol::{
    commands, JsonRpcRequest, JsonRpcResponse, NiriAction, NiriEvent, NiriWorkspace,
    NiriWorkspacesResponse, WorkspaceReference,
};
use crate::error::{AmiyaError, Result};
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use tracing::{debug, info, warn};

/// Niri IPC client for communicating with the compositor
pub struct NiriClient {
    socket_path: PathBuf,
    stream: Arc<Mutex<Option<UnixStream>>>,
    request_id: AtomicU64,
}

impl NiriClient {
    /// Create a new Niri client
    pub fn new() -> Result<Self> {
        let socket_path = Self::find_socket()?;
        info!("Found niri socket at: {:?}", socket_path);

        Ok(NiriClient {
            socket_path,
            stream: Arc::new(Mutex::new(None)),
            request_id: AtomicU64::new(1),
        })
    }

    /// Find the niri socket path
    fn find_socket() -> Result<PathBuf> {
        // Niri socket is typically at $XDG_RUNTIME_DIR/niri/niri-$WAYLAND_DISPLAY.sock
        let runtime_dir = std::env::var("XDG_RUNTIME_DIR")
            .or_else(|_| std::env::var("TMPDIR"))
            .unwrap_or_else(|_| "/tmp".to_string());

        let wayland_display = std::env::var("WAYLAND_DISPLAY").unwrap_or_else(|_| "wayland-0".to_string());

        // Try the standard path first
        let standard_path = PathBuf::from(&runtime_dir)
            .join("niri")
            .join(format!("niri-{}.sock", wayland_display));

        if standard_path.exists() {
            return Ok(standard_path);
        }

        // Fallback: search for any niri socket
        let niri_dir = PathBuf::from(&runtime_dir).join("niri");
        if niri_dir.exists() {
            for entry in std::fs::read_dir(&niri_dir)
                .map_err(|e| AmiyaError::Ipc(format!("Failed to read niri directory: {}", e)))?
            {
                let entry = entry.map_err(|e| AmiyaError::Ipc(e.to_string()))?;
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("sock") {
                    return Ok(path);
                }
            }
        }

        Err(AmiyaError::Ipc(
            "Could not find niri socket. Is niri running?".to_string(),
        ))
    }

    /// Connect to the niri socket
    fn connect(&self) -> Result<UnixStream> {
        let stream = UnixStream::connect(&self.socket_path)
            .map_err(|e| AmiyaError::Ipc(format!("Failed to connect to niri socket: {}", e)))?;

        // Set non-blocking mode for async operations
        stream
            .set_nonblocking(false)
            .map_err(|e| AmiyaError::Ipc(format!("Failed to set socket mode: {}", e)))?;

        debug!("Connected to niri socket");
        Ok(stream)
    }

    /// Ensure we have an active connection
    fn ensure_connected(&self) -> Result<()> {
        let mut stream = self.stream.lock().unwrap();
        if stream.is_none() {
            *stream = Some(self.connect()?);
        }
        Ok(())
    }

    /// Send a JSON-RPC request and receive response
    fn send_request(&self, request: JsonRpcRequest) -> Result<JsonRpcResponse> {
        self.ensure_connected()?;

        let mut stream = self.stream.lock().unwrap();
        let stream = stream.as_mut().ok_or_else(|| {
            AmiyaError::Ipc("Not connected to niri socket".to_string())
        })?;

        // Serialize and send request
        let request_json = serde_json::to_string(&request)
            .map_err(|e| AmiyaError::Ipc(format!("Failed to serialize request: {}", e)))?;

        debug!("Sending request: {}", request_json);

        writeln!(stream, "{}", request_json)
            .map_err(|e| AmiyaError::Ipc(format!("Failed to write to socket: {}", e)))?;

        stream.flush()
            .map_err(|e| AmiyaError::Ipc(format!("Failed to flush socket: {}", e)))?;

        // Read response
        let mut reader = BufReader::new(stream.try_clone().map_err(|e| {
            AmiyaError::Ipc(format!("Failed to clone stream: {}", e))
        })?);

        let mut response_line = String::new();
        reader.read_line(&mut response_line)
            .map_err(|e| AmiyaError::Ipc(format!("Failed to read response: {}", e)))?;

        debug!("Received response: {}", response_line);

        // Parse response
        let response: JsonRpcResponse = serde_json::from_str(&response_line)
            .map_err(|e| AmiyaError::Ipc(format!("Failed to parse response: {}", e)))?;

        // Check for errors
        if let Some(error) = response.error {
            return Err(AmiyaError::Ipc(format!(
                "Niri error: {} (code: {})",
                error.message, error.code
            )));
        }

        Ok(response)
    }

    /// Get next request ID
    fn next_id(&self) -> u64 {
        self.request_id.fetch_add(1, Ordering::SeqCst)
    }

    /// Get all workspaces
    pub fn get_workspaces(&self) -> Result<Vec<NiriWorkspace>> {
        let request = JsonRpcRequest::new(self.next_id(), commands::WORKSPACES);
        let response = self.send_request(request)?;

        let result = response.result.ok_or_else(|| {
            AmiyaError::Ipc("No result in workspaces response".to_string())
        })?;

        let workspaces_response: NiriWorkspacesResponse = serde_json::from_value(result)
            .map_err(|e| AmiyaError::Ipc(format!("Failed to parse workspaces: {}", e)))?;

        Ok(workspaces_response.workspaces)
    }

    /// Focus a workspace by index
    pub fn focus_workspace(&self, index: u32) -> Result<()> {
        let action = NiriAction::FocusWorkspace {
            reference: WorkspaceReference::Index(index),
        };

        let params = serde_json::json!({ "action": action });
        let request = JsonRpcRequest::with_params(self.next_id(), commands::ACTION, params);

        self.send_request(request)?;
        Ok(())
    }

    /// Focus a workspace by name
    pub fn focus_workspace_by_name(&self, name: String) -> Result<()> {
        let action = NiriAction::FocusWorkspace {
            reference: WorkspaceReference::Name(name),
        };

        let params = serde_json::json!({ "action": action });
        let request = JsonRpcRequest::with_params(self.next_id(), commands::ACTION, params);

        self.send_request(request)?;
        Ok(())
    }

    /// Get the niri version
    pub fn get_version(&self) -> Result<String> {
        let request = JsonRpcRequest::new(self.next_id(), commands::VERSION);
        let response = self.send_request(request)?;

        let result = response.result.ok_or_else(|| {
            AmiyaError::Ipc("No result in version response".to_string())
        })?;

        let version = result
            .as_str()
            .ok_or_else(|| AmiyaError::Ipc("Version is not a string".to_string()))?
            .to_string();

        Ok(version)
    }

    /// Disconnect from the socket
    pub fn disconnect(&self) {
        let mut stream = self.stream.lock().unwrap();
        *stream = None;
        debug!("Disconnected from niri socket");
    }
}

impl Default for NiriClient {
    fn default() -> Self {
        Self::new().unwrap_or_else(|e| {
            warn!("Failed to create NiriClient: {}", e);
            // Return a dummy client that won't work but won't crash
            NiriClient {
                socket_path: PathBuf::from("/tmp/niri-dummy.sock"),
                stream: Arc::new(Mutex::new(None)),
                request_id: AtomicU64::new(1),
            }
        })
    }
}

impl Drop for NiriClient {
    fn drop(&mut self) {
        self.disconnect();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_socket() {
        // This test will only pass if niri is running
        // In CI/CD, we'd mock this or skip the test
        match NiriClient::find_socket() {
            Ok(path) => {
                assert!(path.to_str().unwrap().contains("niri"));
                assert!(path.extension().and_then(|s| s.to_str()) == Some("sock"));
            }
            Err(e) => {
                // If niri is not running, that's expected
                assert!(e.to_string().contains("Could not find niri socket"));
            }
        }
    }
}
