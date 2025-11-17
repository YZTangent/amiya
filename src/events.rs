use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::broadcast;

/// Event types that can be broadcast throughout the application
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Event {
    // Workspace events
    WorkspaceChanged {
        id: u32,
    },
    WorkspaceCreated {
        id: u32,
        name: Option<String>,
    },
    WorkspaceRemoved {
        id: u32,
    },
    WorkspacesUpdated {
        workspaces: Vec<WorkspaceInfo>,
    },

    // System events
    VolumeChanged {
        level: f64,
        muted: bool,
    },
    BrightnessChanged {
        level: f64,
    },
    CpuUsageChanged {
        usage: f64,
    },
    MemoryUsageChanged {
        used: u64,
        total: u64,
        percent: f64,
    },
    TemperatureChanged {
        celsius: i32,
    },

    // Network events
    WifiStateChanged {
        enabled: bool,
    },
    WifiNetworkConnected {
        ssid: String,
    },
    WifiNetworkDisconnected,
    WifiNetworksUpdated {
        networks: Vec<WifiNetworkInfo>,
    },

    // Bluetooth events
    BluetoothStateChanged {
        enabled: bool,
    },
    BluetoothDeviceConnected {
        address: String,
        name: String,
    },
    BluetoothDeviceDisconnected {
        address: String,
    },
    BluetoothDevicesUpdated {
        devices: Vec<BluetoothDeviceInfo>,
    },

    // Media events
    MediaPlayerChanged {
        player: Option<String>,
    },
    MediaTrackChanged {
        title: String,
        artist: String,
        album: Option<String>,
    },
    MediaPlaybackChanged {
        playing: bool,
    },
    MediaVolumeChanged {
        volume: f64,
    },

    // UI events
    PopupRequested {
        popup_type: PopupType,
    },
    PopupClosed {
        popup_type: PopupType,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceInfo {
    pub id: u32,
    pub name: Option<String>,
    pub is_active: bool,
    pub is_focused: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WifiNetworkInfo {
    pub ssid: String,
    pub signal_strength: u8,
    pub secured: bool,
    pub connected: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BluetoothDeviceInfo {
    pub address: String,
    pub name: String,
    pub connected: bool,
    pub paired: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PopupType {
    Bluetooth,
    Wifi,
    MediaControl,
}

/// Event manager handles broadcasting events throughout the application
pub struct EventManager {
    sender: broadcast::Sender<Event>,
}

impl EventManager {
    /// Create a new event manager with specified channel capacity
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        EventManager { sender }
    }

    /// Emit an event to all subscribers
    pub fn emit(&self, event: Event) {
        // Ignore send errors - they just mean no subscribers
        let _ = self.sender.send(event);
    }

    /// Subscribe to events
    pub fn subscribe(&self) -> broadcast::Receiver<Event> {
        self.sender.subscribe()
    }

    /// Get the number of active subscribers
    pub fn subscriber_count(&self) -> usize {
        self.sender.receiver_count()
    }
}

impl Clone for EventManager {
    fn clone(&self) -> Self {
        EventManager {
            sender: self.sender.clone(),
        }
    }
}

impl Default for EventManager {
    fn default() -> Self {
        Self::new(100) // Default capacity of 100 events
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_manager_creation() {
        let manager = EventManager::new(10);
        assert_eq!(manager.subscriber_count(), 0);
    }

    #[test]
    fn test_event_subscription() {
        let manager = EventManager::new(10);
        let _receiver = manager.subscribe();
        assert_eq!(manager.subscriber_count(), 1);
    }

    #[test]
    fn test_event_emission() {
        let manager = EventManager::new(10);
        let mut receiver = manager.subscribe();

        manager.emit(Event::VolumeChanged {
            level: 75.0,
            muted: false,
        });

        let received = receiver.try_recv();
        assert!(received.is_ok());

        if let Ok(Event::VolumeChanged { level, muted }) = received {
            assert_eq!(level, 75.0);
            assert_eq!(muted, false);
        } else {
            panic!("Expected VolumeChanged event");
        }
    }

    #[test]
    fn test_multiple_subscribers() {
        let manager = EventManager::new(10);
        let mut receiver1 = manager.subscribe();
        let mut receiver2 = manager.subscribe();

        assert_eq!(manager.subscriber_count(), 2);

        manager.emit(Event::WorkspaceChanged { id: 2 });

        // Both receivers should get the event
        assert!(receiver1.try_recv().is_ok());
        assert!(receiver2.try_recv().is_ok());
    }

    #[test]
    fn test_event_clone() {
        let event = Event::VolumeChanged {
            level: 50.0,
            muted: true,
        };

        let cloned = event.clone();
        match (event, cloned) {
            (
                Event::VolumeChanged { level: l1, muted: m1 },
                Event::VolumeChanged { level: l2, muted: m2 },
            ) => {
                assert_eq!(l1, l2);
                assert_eq!(m1, m2);
            }
            _ => panic!("Event clone failed"),
        }
    }
}
