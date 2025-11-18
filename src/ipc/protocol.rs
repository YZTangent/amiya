use serde::{Deserialize, Serialize};

/// Command sent from amiya-ctl to amiya
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum Command {
    /// Show a popup
    ShowPopup { popup: PopupType },

    /// Hide a popup
    HidePopup { popup: PopupType },

    /// Toggle a popup
    TogglePopup { popup: PopupType },

    /// Volume control
    Volume { action: VolumeAction },

    /// Brightness control
    Brightness { action: BrightnessAction },

    /// Power management
    Power { action: PowerAction },

    /// Get current status
    Status,

    /// Ping to check if server is alive
    Ping,
}

/// Type of popup
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PopupType {
    Bluetooth,
    Wifi,
    MediaControl,
    Power,
}

/// Volume actions
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "kebab-case")]
pub enum VolumeAction {
    Up { amount: Option<f64> },
    Down { amount: Option<f64> },
    Set { level: f64 },
    Mute,
    Unmute,
    ToggleMute,
}

/// Brightness actions
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "kebab-case")]
pub enum BrightnessAction {
    Up { amount: Option<f64> },
    Down { amount: Option<f64> },
    Set { level: f64 },
}

/// Power actions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PowerAction {
    Shutdown,
    Reboot,
    Suspend,
    Hibernate,
    Lock,
}

/// Response from amiya to amiya-ctl
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "kebab-case")]
pub enum Response {
    Success {
        message: Option<String>,
    },
    Error {
        message: String,
    },
    Status {
        version: String,
        uptime: u64,
    },
    Pong,
}

impl Response {
    pub fn success() -> Self {
        Response::Success { message: None }
    }

    pub fn success_with_message(message: String) -> Self {
        Response::Success {
            message: Some(message),
        }
    }

    pub fn error(message: String) -> Self {
        Response::Error { message }
    }

    pub fn pong() -> Self {
        Response::Pong
    }
}
