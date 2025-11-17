use crate::error::{AmiyaError, Result};
use crate::events::{BluetoothDeviceInfo, Event, EventManager};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use zbus::Connection;

// Re-export for convenience
pub use crate::events::BluetoothDeviceInfo as BluetoothDevice;

/// Bluetooth control via BlueZ D-Bus
pub struct BluetoothControl {
    connection: Arc<RwLock<Option<Connection>>>,
    adapter_path: Arc<RwLock<Option<String>>>,
    devices: Arc<RwLock<HashMap<String, BluetoothDevice>>>,
    powered: Arc<RwLock<bool>>,
    scanning: Arc<RwLock<bool>>,
    events: Option<EventManager>,
}

impl BluetoothControl {
    /// Create a new bluetooth control instance
    pub fn new() -> Self {
        BluetoothControl {
            connection: Arc::new(RwLock::new(None)),
            adapter_path: Arc::new(RwLock::new(None)),
            devices: Arc::new(RwLock::new(HashMap::new())),
            powered: Arc::new(RwLock::new(false)),
            scanning: Arc::new(RwLock::new(false)),
            events: None,
        }
    }

    /// Create with event manager for reactive updates
    pub fn with_events(events: EventManager) -> Self {
        let mut bt = Self::new();
        bt.events = Some(events);
        bt
    }

    /// Initialize connection to BlueZ
    pub async fn connect(&self) -> Result<()> {
        match Connection::system().await {
            Ok(conn) => {
                info!("Connected to D-Bus system bus for Bluetooth control");

                // Store connection
                {
                    let mut connection = self.connection.write().await;
                    *connection = Some(conn.clone());
                }

                // Find default adapter
                match self.find_adapter(&conn).await {
                    Ok(path) => {
                        info!("Found Bluetooth adapter: {}", path);
                        let mut adapter_path = self.adapter_path.write().await;
                        *adapter_path = Some(path.clone());

                        // Get initial state
                        if let Err(e) = self.update_adapter_state(&conn, &path).await {
                            warn!("Failed to get initial adapter state: {}", e);
                        }

                        // Get initial device list
                        if let Err(e) = self.update_device_list(&conn).await {
                            warn!("Failed to get initial device list: {}", e);
                        }
                    }
                    Err(e) => {
                        warn!("No Bluetooth adapter found: {}", e);
                        return Err(AmiyaError::Backend(
                            "No Bluetooth adapter available".to_string(),
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

    /// Find the default Bluetooth adapter
    async fn find_adapter(&self, conn: &Connection) -> Result<String> {
        // Try common adapter paths
        let common_paths = vec![
            "/org/bluez/hci0",
            "/org/bluez/hci1",
            "/org/bluez/hci2",
        ];

        for path in common_paths {
            // Try to create a proxy to the adapter
            let proxy_result = zbus::ProxyBuilder::new(conn)
                .interface("org.bluez.Adapter1")
                .path(path)?
                .destination("org.bluez")?
                .build::<zbus::Proxy>()
                .await;

            if proxy_result.is_ok() {
                return Ok(path.to_string());
            }
        }

        Err(AmiyaError::Backend("No Bluetooth adapter found".to_string()))
    }

    /// Update adapter state (powered, discovering, etc.)
    async fn update_adapter_state(&self, conn: &Connection, path: &str) -> Result<()> {
        let proxy = zbus::ProxyBuilder::new(conn)
            .interface("org.bluez.Adapter1")
            .path(path)?
            .destination("org.bluez")?
            .build::<zbus::Proxy>()
            .await
            .map_err(|e| AmiyaError::Backend(format!("Failed to create adapter proxy: {}", e)))?;

        // Get Powered property
        let powered: bool = proxy
            .get_property("Powered")
            .await
            .unwrap_or(false);

        {
            let mut powered_state = self.powered.write().await;
            *powered_state = powered;
        }

        // Get Discovering property
        let discovering: bool = proxy
            .get_property("Discovering")
            .await
            .unwrap_or(false);

        {
            let mut scanning_state = self.scanning.write().await;
            *scanning_state = discovering;
        }

        debug!("Adapter state - Powered: {}, Discovering: {}", powered, discovering);

        Ok(())
    }

    /// Update device list from BlueZ
    async fn update_device_list(&self, conn: &Connection) -> Result<()> {
        // In a full implementation, we'd use ObjectManager to enumerate devices
        // For now, just clear and emit event

        let devices = self.devices.read().await;
        let device_list: Vec<BluetoothDevice> = devices.values().cloned().collect();

        if let Some(events) = &self.events {
            events.emit(Event::BluetoothDevicesUpdated {
                devices: device_list,
            });
        }

        Ok(())
    }

    /// Check if Bluetooth is available
    pub fn is_available(&self) -> bool {
        // Check if we have a connection and adapter
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let conn = self.connection.read().await;
                let adapter = self.adapter_path.read().await;
                conn.is_some() && adapter.is_some()
            })
        })
    }

    /// Check if Bluetooth is powered on
    pub async fn is_powered(&self) -> Result<bool> {
        let powered = *self.powered.read().await;
        Ok(powered)
    }

    /// Set Bluetooth powered state
    pub async fn set_powered(&self, enabled: bool) -> Result<()> {
        let conn_guard = self.connection.read().await;
        let adapter_guard = self.adapter_path.read().await;

        let conn = conn_guard
            .as_ref()
            .ok_or_else(|| AmiyaError::Backend("Not connected to D-Bus".to_string()))?;

        let adapter_path = adapter_guard
            .as_ref()
            .ok_or_else(|| AmiyaError::Backend("No adapter available".to_string()))?;

        let proxy = zbus::ProxyBuilder::new(conn)
            .interface("org.bluez.Adapter1")
            .path(adapter_path.as_str())?
            .destination("org.bluez")?
            .build::<zbus::Proxy>()
            .await
            .map_err(|e| AmiyaError::Backend(format!("Failed to create adapter proxy: {}", e)))?;

        proxy
            .set_property("Powered", enabled)
            .await
            .map_err(|e| AmiyaError::Backend(format!("Failed to set powered: {}", e)))?;

        // Update local state
        {
            let mut powered_state = self.powered.write().await;
            *powered_state = enabled;
        }

        info!("Bluetooth powered: {}", enabled);

        // Emit event
        if let Some(events) = &self.events {
            events.emit(Event::BluetoothStateChanged { enabled });
        }

        Ok(())
    }

    /// Start device discovery/scanning
    pub async fn start_scan(&self) -> Result<()> {
        let conn_guard = self.connection.read().await;
        let adapter_guard = self.adapter_path.read().await;

        let conn = conn_guard
            .as_ref()
            .ok_or_else(|| AmiyaError::Backend("Not connected to D-Bus".to_string()))?;

        let adapter_path = adapter_guard
            .as_ref()
            .ok_or_else(|| AmiyaError::Backend("No adapter available".to_string()))?;

        let proxy = zbus::ProxyBuilder::new(conn)
            .interface("org.bluez.Adapter1")
            .path(adapter_path.as_str())?
            .destination("org.bluez")?
            .build::<zbus::Proxy>()
            .await
            .map_err(|e| AmiyaError::Backend(format!("Failed to create adapter proxy: {}", e)))?;

        proxy
            .call_method("StartDiscovery", &())
            .await
            .map_err(|e| AmiyaError::Backend(format!("Failed to start discovery: {}", e)))?;

        {
            let mut scanning = self.scanning.write().await;
            *scanning = true;
        }

        info!("Started Bluetooth device discovery");

        Ok(())
    }

    /// Stop device discovery/scanning
    pub async fn stop_scan(&self) -> Result<()> {
        let conn_guard = self.connection.read().await;
        let adapter_guard = self.adapter_path.read().await;

        let conn = conn_guard
            .as_ref()
            .ok_or_else(|| AmiyaError::Backend("Not connected to D-Bus".to_string()))?;

        let adapter_path = adapter_guard
            .as_ref()
            .ok_or_else(|| AmiyaError::Backend("No adapter available".to_string()))?;

        let proxy = zbus::ProxyBuilder::new(conn)
            .interface("org.bluez.Adapter1")
            .path(adapter_path.as_str())?
            .destination("org.bluez")?
            .build::<zbus::Proxy>()
            .await
            .map_err(|e| AmiyaError::Backend(format!("Failed to create adapter proxy: {}", e)))?;

        proxy
            .call_method("StopDiscovery", &())
            .await
            .map_err(|e| AmiyaError::Backend(format!("Failed to stop discovery: {}", e)))?;

        {
            let mut scanning = self.scanning.write().await;
            *scanning = false;
        }

        info!("Stopped Bluetooth device discovery");

        Ok(())
    }

    /// Get list of known devices
    pub async fn get_devices(&self) -> Result<Vec<BluetoothDevice>> {
        let devices = self.devices.read().await;
        Ok(devices.values().cloned().collect())
    }

    /// Connect to a device by address
    pub async fn connect_device(&self, address: &str) -> Result<()> {
        let conn_guard = self.connection.read().await;

        let conn = conn_guard
            .as_ref()
            .ok_or_else(|| AmiyaError::Backend("Not connected to D-Bus".to_string()))?;

        // Convert address to object path (e.g., AA:BB:CC:DD:EE:FF -> /org/bluez/hci0/dev_AA_BB_CC_DD_EE_FF)
        let device_path = format!(
            "/org/bluez/hci0/dev_{}",
            address.replace(':', "_")
        );

        let proxy = zbus::ProxyBuilder::new(conn)
            .interface("org.bluez.Device1")
            .path(device_path.as_str())?
            .destination("org.bluez")?
            .build::<zbus::Proxy>()
            .await
            .map_err(|e| AmiyaError::Backend(format!("Failed to create device proxy: {}", e)))?;

        proxy
            .call_method("Connect", &())
            .await
            .map_err(|e| AmiyaError::Backend(format!("Failed to connect device: {}", e)))?;

        info!("Connected to Bluetooth device: {}", address);

        // Update device state
        self.update_device_list(conn).await?;

        Ok(())
    }

    /// Disconnect from a device by address
    pub async fn disconnect_device(&self, address: &str) -> Result<()> {
        let conn_guard = self.connection.read().await;

        let conn = conn_guard
            .as_ref()
            .ok_or_else(|| AmiyaError::Backend("Not connected to D-Bus".to_string()))?;

        let device_path = format!(
            "/org/bluez/hci0/dev_{}",
            address.replace(':', "_")
        );

        let proxy = zbus::ProxyBuilder::new(conn)
            .interface("org.bluez.Device1")
            .path(device_path.as_str())?
            .destination("org.bluez")?
            .build::<zbus::Proxy>()
            .await
            .map_err(|e| AmiyaError::Backend(format!("Failed to create device proxy: {}", e)))?;

        proxy
            .call_method("Disconnect", &())
            .await
            .map_err(|e| AmiyaError::Backend(format!("Failed to disconnect device: {}", e)))?;

        info!("Disconnected from Bluetooth device: {}", address);

        // Update device state
        self.update_device_list(conn).await?;

        Ok(())
    }

    /// Pair with a device by address
    pub async fn pair_device(&self, address: &str) -> Result<()> {
        let conn_guard = self.connection.read().await;

        let conn = conn_guard
            .as_ref()
            .ok_or_else(|| AmiyaError::Backend("Not connected to D-Bus".to_string()))?;

        let device_path = format!(
            "/org/bluez/hci0/dev_{}",
            address.replace(':', "_")
        );

        let proxy = zbus::ProxyBuilder::new(conn)
            .interface("org.bluez.Device1")
            .path(device_path.as_str())?
            .destination("org.bluez")?
            .build::<zbus::Proxy>()
            .await
            .map_err(|e| AmiyaError::Backend(format!("Failed to create device proxy: {}", e)))?;

        proxy
            .call_method("Pair", &())
            .await
            .map_err(|e| AmiyaError::Backend(format!("Failed to pair device: {}", e)))?;

        info!("Paired with Bluetooth device: {}", address);

        // Update device state
        self.update_device_list(conn).await?;

        Ok(())
    }

    /// Remove/unpair a device by address
    pub async fn remove_device(&self, address: &str) -> Result<()> {
        let conn_guard = self.connection.read().await;
        let adapter_guard = self.adapter_path.read().await;

        let conn = conn_guard
            .as_ref()
            .ok_or_else(|| AmiyaError::Backend("Not connected to D-Bus".to_string()))?;

        let adapter_path = adapter_guard
            .as_ref()
            .ok_or_else(|| AmiyaError::Backend("No adapter available".to_string()))?;

        let device_path = format!(
            "/org/bluez/hci0/dev_{}",
            address.replace(':', "_")
        );

        let proxy = zbus::ProxyBuilder::new(conn)
            .interface("org.bluez.Adapter1")
            .path(adapter_path.as_str())?
            .destination("org.bluez")?
            .build::<zbus::Proxy>()
            .await
            .map_err(|e| AmiyaError::Backend(format!("Failed to create adapter proxy: {}", e)))?;

        proxy
            .call_method("RemoveDevice", &(device_path,))
            .await
            .map_err(|e| AmiyaError::Backend(format!("Failed to remove device: {}", e)))?;

        info!("Removed Bluetooth device: {}", address);

        // Update device state
        self.update_device_list(conn).await?;

        Ok(())
    }

    /// Start monitoring Bluetooth events
    /// This would subscribe to D-Bus signals in a full implementation
    pub async fn start_monitoring(&self) -> Result<()> {
        debug!("Bluetooth monitoring started (basic implementation)");
        // TODO: Subscribe to D-Bus signals:
        // - InterfacesAdded (new devices discovered)
        // - InterfacesRemoved (devices removed)
        // - PropertiesChanged (device/adapter state changes)
        Ok(())
    }
}

impl Default for BluetoothControl {
    fn default() -> Self {
        Self::new()
    }
}

// Helper function to create bluetooth control in GTK context
pub fn create_bluetooth_control_sync(events: EventManager) -> Arc<BluetoothControl> {
    let bluetooth = Arc::new(BluetoothControl::with_events(events));

    // Try to connect in background
    let bluetooth_clone = bluetooth.clone();
    tokio::spawn(async move {
        if let Err(e) = bluetooth_clone.connect().await {
            warn!("Failed to connect Bluetooth control: {}", e);
        }
    });

    bluetooth
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_bluetooth_creation() {
        let bt = BluetoothControl::new();
        // Should not panic
        assert!(!bt.is_available());
    }

    #[tokio::test]
    async fn test_powered_state() {
        let bt = BluetoothControl::new();
        // Default state should be false (not connected)
        assert_eq!(bt.is_powered().await.unwrap(), false);
    }
}
