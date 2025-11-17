use serde::{Deserialize, Serialize};

/// JSON-RPC 2.0 Request
#[derive(Debug, Clone, Serialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: u64,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,
}

impl JsonRpcRequest {
    pub fn new(id: u64, method: impl Into<String>) -> Self {
        JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id,
            method: method.into(),
            params: None,
        }
    }

    pub fn with_params(id: u64, method: impl Into<String>, params: serde_json::Value) -> Self {
        JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id,
            method: method.into(),
            params: Some(params),
        }
    }
}

/// JSON-RPC 2.0 Response
#[derive(Debug, Clone, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub id: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

/// JSON-RPC 2.0 Error
#[derive(Debug, Clone, Deserialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

/// Niri workspace information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NiriWorkspace {
    pub id: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub idx: u32,
    pub is_active: bool,
    pub is_focused: bool,
}

/// Niri workspaces response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NiriWorkspacesResponse {
    pub workspaces: Vec<NiriWorkspace>,
}

/// Niri window information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NiriWindow {
    pub id: u64,
    pub title: Option<String>,
    pub app_id: Option<String>,
    pub workspace_id: Option<u64>,
    pub is_focused: bool,
}

/// Niri event notification
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum NiriEvent {
    WorkspaceActivated {
        id: u64,
        focused: bool,
    },
    WorkspaceActiveWindowChanged {
        workspace_id: u64,
        window_id: Option<u64>,
    },
    WorkspacesChanged {
        workspaces: Vec<NiriWorkspace>,
    },
    WindowOpenedOrChanged {
        window: NiriWindow,
    },
    WindowClosed {
        id: u64,
    },
    WindowFocusChanged {
        id: Option<u64>,
    },
    KeyboardLayoutsChanged {
        keyboard_layouts: Vec<String>,
        current_idx: u32,
    },
    KeyboardLayoutSwitched {
        idx: u32,
    },
}

/// Niri commands
pub mod commands {
    pub const WORKSPACES: &str = "Workspaces";
    pub const FOCUSED_WINDOW: &str = "FocusedWindow";
    pub const ACTION: &str = "Action";
    pub const OUTPUT: &str = "Output";
    pub const OUTPUTS: &str = "Outputs";
    pub const KEYBOARD_LAYOUTS: &str = "KeyboardLayouts";
    pub const FOCUSED_OUTPUT: &str = "FocusedOutput";
    pub const VERSION: &str = "Version";
    pub const SUBSCRIBE: &str = "EventStream";
}

/// Niri actions that can be triggered
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum NiriAction {
    FocusWorkspace { reference: WorkspaceReference },
    FocusWorkspaceDown,
    FocusWorkspaceUp,
    MoveWindowToWorkspace { reference: WorkspaceReference },
    MoveWindowToWorkspaceDown,
    MoveWindowToWorkspaceUp,
    Quit,
    PowerOffMonitors,
}

/// Reference to a workspace by index or name
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum WorkspaceReference {
    Index(u32),
    Name(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_serialization() {
        let req = JsonRpcRequest::new(1, commands::WORKSPACES);
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("\"jsonrpc\":\"2.0\""));
        assert!(json.contains("\"id\":1"));
        assert!(json.contains("\"method\":\"Workspaces\""));
    }

    #[test]
    fn test_action_serialization() {
        let action = NiriAction::FocusWorkspace {
            reference: WorkspaceReference::Index(2),
        };
        let json = serde_json::to_value(&action).unwrap();
        assert!(json.is_object());
    }

    #[test]
    fn test_workspace_deserialization() {
        let json = r#"{
            "id": 123,
            "idx": 0,
            "is_active": true,
            "is_focused": true
        }"#;
        let workspace: NiriWorkspace = serde_json::from_str(json).unwrap();
        assert_eq!(workspace.id, 123);
        assert_eq!(workspace.idx, 0);
        assert!(workspace.is_active);
        assert!(workspace.is_focused);
    }
}
