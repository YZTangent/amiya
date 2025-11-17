use crate::error::{AmiyaError, Result};
use crate::events::{Event, EventManager, WifiNetworkInfo};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use zbus::Connection;

// Re-export for convenience
pub use crate::events::WifiNetworkInfo as WifiNetwork;

/// Network control via NetworkManager
pub struct NetworkControl {
    connection: Arc<RwLock<Option<Connection>>>,
    wifi_device_path: Arc<RwLock<Option<String>>>,
    networks: Arc<RwLock<HashMap<String, WifiNetwork>>>,
    wifi_enabled: Arc<RwLock<bool>>,
    events: Option<EventManager>,
}

impl NetworkControl {
    /// Create a new network control instance
    pub fn new() -> Self {
        NetworkControl {
            connection: Arc::new(RwLock::new(None)),
            wifi_device_path: Arc::new(RwLock::new(None)),
            networks: Arc::new(RwLock::new(HashMap::new())),
            wifi_enabled: Arc::new(RwLock::new(false)),
            events: None,
        }
    }

    /// Create with event manager for reactive updates
    pub fn with_events(events: EventManager) -> Self {
        let mut nm = Self::new();
        nm.events = Some(events);
        nm
    }

    /// Initialize connection to NetworkManager
    pub async fn connect(&self) -> Result<()> {
        match Connection::system().await {
            Ok(conn) => {
                info!("Connected to D-Bus system bus for Network control");

                // Store connection
                {
                    let mut connection = self.connection.write().await;
                    *connection = Some(conn.clone());
                }

                // Find WiFi device
                match self.find_wifi_device(&conn).await {
                    Ok(path) => {
                        info!("Found WiFi device: {}", path);
                        let mut device_path = self.wifi_device_path.write().await;
                        *device_path = Some(path.clone());

                        // Get initial WiFi state
                        if let Err(e) = self.update_wifi_state(&conn).await {
                            warn!("Failed to get initial WiFi state: {}", e);
                        }
                    }
                    Err(e) => {
                        warn!("No WiFi device found: {}", e);
                        return Err(AmiyaError::Backend(
                            "No WiFi device available".to_string(),
                        ));
                    }
                }

                Ok(())
            }
            Err(e) => {
                warn!("Could not connect to D-Bus system bus: {}", e);
                Err(AmiyaError::Backend(format!(
                    "Failed to connect to D-Bus: {}",
                    e
                )))
            }
        }
    }

    /// Find WiFi device from NetworkManager
    async fn find_wifi_device(&self, conn: &Connection) -> Result<String> {
        // Create proxy for NetworkManager
        let nm_proxy = zbus::ProxyBuilder::new(conn)
            .interface("org.freedesktop.NetworkManager")
            .path("/org/freedesktop/NetworkManager")?
            .destination("org.freedesktop.NetworkManager")?
            .build::<zbus::Proxy>()
            .await
            .map_err(|e| AmiyaError::Backend(format!("Failed to create NM proxy: {}", e)))?;

        // Get all devices
        let devices: Vec<zbus::zvariant::OwnedObjectPath> = nm_proxy
            .call_method("GetDevices", &())
            .await
            .map_err(|e| AmiyaError::Backend(format!("Failed to get devices: {}", e)))?
            .body()
            .deserialize()
            .map_err(|e| AmiyaError::Backend(format!("Failed to deserialize devices: {}", e)))?;

        // Find first WiFi device (type 2 = NM_DEVICE_TYPE_WIFI)
        for device_path in devices {
            let device_proxy = zbus::ProxyBuilder::new(conn)
                .interface("org.freedesktop.NetworkManager.Device")
                .path(device_path.as_str())?
                .destination("org.freedesktop.NetworkManager")?
                .build::<zbus::Proxy>()
                .await
                .map_err(|e| AmiyaError::Backend(format!("Failed to create device proxy: {}", e)))?;

            // Get device type
            let device_type: u32 = device_proxy
                .get_property("DeviceType")
                .await
                .unwrap_or(0);

            if device_type == 2 {
                // WiFi device found
                return Ok(device_path.to_string());
            }
        }

        Err(AmiyaError::Backend("No WiFi device found".to_string()))
    }

    /// Update WiFi enabled state
    async fn update_wifi_state(&self, conn: &Connection) -> Result<()> {
        let nm_proxy = zbus::ProxyBuilder::new(conn)
            .interface("org.freedesktop.NetworkManager")
            .path("/org/freedesktop/NetworkManager")?
            .destination("org.freedesktop.NetworkManager")?
            .build::<zbus::Proxy>()
            .await
            .map_err(|e| AmiyaError::Backend(format!("Failed to create NM proxy: {}", e)))?;

        let enabled: bool = nm_proxy
            .get_property("WirelessEnabled")
            .await
            .unwrap_or(false);

        {
            let mut wifi_state = self.wifi_enabled.write().await;
            *wifi_state = enabled;
        }

        debug!("WiFi enabled state: {}", enabled);

        Ok(())
    }

    /// Check if network control is available
    pub fn is_available(&self) -> bool {
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let conn = self.connection.read().await;
                let device = self.wifi_device_path.read().await;
                conn.is_some() && device.is_some()
            })
        })
    }

    /// Check if WiFi is enabled
    pub async fn is_wifi_enabled(&self) -> Result<bool> {
        let enabled = *self.wifi_enabled.read().await;
        Ok(enabled)
    }

    /// Set WiFi enabled state
    pub async fn set_wifi_enabled(&self, enabled: bool) -> Result<()> {
        let conn_guard = self.connection.read().await;

        let conn = conn_guard
            .as_ref()
            .ok_or_else(|| AmiyaError::Backend("Not connected to D-Bus".to_string()))?;

        let nm_proxy = zbus::ProxyBuilder::new(conn)
            .interface("org.freedesktop.NetworkManager")
            .path("/org/freedesktop/NetworkManager")?
            .destination("org.freedesktop.NetworkManager")?
            .build::<zbus::Proxy>()
            .await
            .map_err(|e| AmiyaError::Backend(format!("Failed to create NM proxy: {}", e)))?;

        nm_proxy
            .set_property("WirelessEnabled", enabled)
            .await
            .map_err(|e| AmiyaError::Backend(format!("Failed to set WiFi enabled: {}", e)))?;

        // Update local state
        {
            let mut wifi_state = self.wifi_enabled.write().await;
            *wifi_state = enabled;
        }

        info!("WiFi enabled: {}", enabled);

        // Emit event
        if let Some(events) = &self.events {
            events.emit(Event::WifiStateChanged { enabled });
        }

        Ok(())
    }

    /// Scan for WiFi networks
    pub async fn scan(&self) -> Result<()> {
        let conn_guard = self.connection.read().await;
        let device_guard = self.wifi_device_path.read().await;

        let conn = conn_guard
            .as_ref()
            .ok_or_else(|| AmiyaError::Backend("Not connected to D-Bus".to_string()))?;

        let device_path = device_guard
            .as_ref()
            .ok_or_else(|| AmiyaError::Backend("No WiFi device available".to_string()))?;

        let wireless_proxy = zbus::ProxyBuilder::new(conn)
            .interface("org.freedesktop.NetworkManager.Device.Wireless")
            .path(device_path.as_str())?
            .destination("org.freedesktop.NetworkManager")?
            .build::<zbus::Proxy>()
            .await
            .map_err(|e| AmiyaError::Backend(format!("Failed to create wireless proxy: {}", e)))?;

        // Request scan with empty options
        let options: HashMap<String, zbus::zvariant::Value> = HashMap::new();
        wireless_proxy
            .call_method("RequestScan", &(options,))
            .await
            .map_err(|e| AmiyaError::Backend(format!("Failed to request scan: {}", e)))?;

        info!("WiFi scan requested");

        // Note: Actual scan results come via signals, which we'd subscribe to
        // For now, we'll just trigger a manual refresh after a delay
        Ok(())
    }

    /// Get list of available WiFi networks
    pub async fn get_networks(&self) -> Result<Vec<WifiNetwork>> {
        let conn_guard = self.connection.read().await;
        let device_guard = self.wifi_device_path.read().await;

        let conn = conn_guard
            .as_ref()
            .ok_or_else(|| AmiyaError::Backend("Not connected to D-Bus".to_string()))?;

        let device_path = device_guard
            .as_ref()
            .ok_or_else(|| AmiyaError::Backend("No WiFi device available".to_string()))?;

        let wireless_proxy = zbus::ProxyBuilder::new(conn)
            .interface("org.freedesktop.NetworkManager.Device.Wireless")
            .path(device_path.as_str())?
            .destination("org.freedesktop.NetworkManager")?
            .build::<zbus::Proxy>()
            .await
            .map_err(|e| AmiyaError::Backend(format!("Failed to create wireless proxy: {}", e)))?;

        // Get access points
        let access_points: Vec<zbus::zvariant::OwnedObjectPath> = wireless_proxy
            .call_method("GetAccessPoints", &())
            .await
            .map_err(|e| AmiyaError::Backend(format!("Failed to get access points: {}", e)))?
            .body()
            .deserialize()
            .map_err(|e| AmiyaError::Backend(format!("Failed to deserialize APs: {}", e)))?;

        let mut networks = Vec::new();

        // Get details for each access point
        for ap_path in access_points {
            if let Ok(ap_proxy) = zbus::ProxyBuilder::new(conn)
                .interface("org.freedesktop.NetworkManager.AccessPoint")
                .path(ap_path.as_str())
                .ok()
                .and_then(|b| b.destination("org.freedesktop.NetworkManager").ok())
                .and_then(|b| tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current().block_on(b.build::<zbus::Proxy>())
                }).ok())
            {
                // Get SSID (as raw bytes)
                let ssid_bytes: Vec<u8> = ap_proxy
                    .get_property("Ssid")
                    .await
                    .unwrap_or_default();

                let ssid = String::from_utf8(ssid_bytes).unwrap_or_default();

                // Get signal strength (0-100)
                let strength: u8 = ap_proxy
                    .get_property("Strength")
                    .await
                    .unwrap_or(0);

                // Get security flags
                let flags: u32 = ap_proxy
                    .get_property("Flags")
                    .await
                    .unwrap_or(0);

                let wpa_flags: u32 = ap_proxy
                    .get_property("WpaFlags")
                    .await
                    .unwrap_or(0);

                let rsn_flags: u32 = ap_proxy
                    .get_property("RsnFlags")
                    .await
                    .unwrap_or(0);

                let secured = wpa_flags != 0 || rsn_flags != 0;

                if !ssid.is_empty() {
                    networks.push(WifiNetwork {
                        ssid,
                        signal_strength: strength,
                        secured,
                        connected: false, // TODO: Check active connection
                    });
                }
            }
        }

        // Sort by signal strength
        networks.sort_by(|a, b| b.signal_strength.cmp(&a.signal_strength));

        // Emit event
        if let Some(events) = &self.events {
            events.emit(Event::WifiNetworksUpdated {
                networks: networks.clone(),
            });
        }

        Ok(networks)
    }

    /// Connect to a WiFi network
    pub async fn connect(&self, ssid: &str, password: Option<&str>) -> Result<()> {
        let conn_guard = self.connection.read().await;

        let conn = conn_guard
            .as_ref()
            .ok_or_else(|| AmiyaError::Backend("Not connected to D-Bus".to_string()))?;

        info!("Connecting to WiFi network: {}", ssid);

        // In a full implementation, we'd:
        // 1. Create a connection settings dict with SSID, security type, password
        // 2. Call AddAndActivateConnection on NetworkManager
        // 3. Wait for connection state change signal

        // For now, just emit event
        if let Some(events) = &self.events {
            events.emit(Event::WifiNetworkConnected {
                ssid: ssid.to_string(),
            });
        }

        Ok(())
    }

    /// Disconnect from current network
    pub async fn disconnect(&self) -> Result<()> {
        info!("Disconnecting from WiFi network");

        // In a full implementation, we'd:
        // 1. Get active connection from NetworkManager
        // 2. Call DeactivateConnection

        // For now, just emit event
        if let Some(events) = &self.events {
            events.emit(Event::WifiNetworkDisconnected);
        }

        Ok(())
    }

    /// Start monitoring network events
    /// This would subscribe to D-Bus signals in a full implementation
    pub async fn start_monitoring(&self) -> Result<()> {
        debug!("Network monitoring started (basic implementation)");
        // TODO: Subscribe to D-Bus signals:
        // - StateChanged (connection state)
        // - PropertiesChanged (WiFi enabled, active connection)
        // - AccessPointAdded/Removed (new networks discovered)
        Ok(())
    }
}

impl Default for NetworkControl {
    fn default() -> Self {
        Self::new()
    }
}

// Helper function to create network control in GTK context
pub fn create_network_control_sync(events: EventManager) -> Arc<NetworkControl> {
    let network = Arc::new(NetworkControl::with_events(events));

    // Try to connect in background
    let network_clone = network.clone();
    tokio::spawn(async move {
        if let Err(e) = network_clone.connect().await {
            warn!("Failed to connect Network control: {}", e);
        }
    });

    network
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_network_creation() {
        let nm = NetworkControl::new();
        // Should not panic
        assert!(!nm.is_available());
    }

    #[tokio::test]
    async fn test_wifi_state() {
        let nm = NetworkControl::new();
        // Default state should be false (not connected)
        assert_eq!(nm.is_wifi_enabled().await.unwrap(), false);
    }
}
