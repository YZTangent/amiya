use crate::error::{AmiyaError, Result};
use crate::events::{Event, EventManager};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use zbus::Connection;

/// Battery state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BatteryState {
    Charging,
    Discharging,
    FullyCharged,
    Empty,
    Unknown,
}

impl std::fmt::Display for BatteryState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BatteryState::Charging => write!(f, "Charging"),
            BatteryState::Discharging => write!(f, "Discharging"),
            BatteryState::FullyCharged => write!(f, "Fully Charged"),
            BatteryState::Empty => write!(f, "Empty"),
            BatteryState::Unknown => write!(f, "Unknown"),
        }
    }
}

impl From<u32> for BatteryState {
    fn from(value: u32) -> Self {
        match value {
            1 => BatteryState::Charging,
            2 => BatteryState::Discharging,
            3 => BatteryState::Empty,
            4 => BatteryState::FullyCharged,
            _ => BatteryState::Unknown,
        }
    }
}

/// Battery information
#[derive(Debug, Clone)]
pub struct BatteryInfo {
    pub percentage: f64,
    pub state: BatteryState,
    pub time_to_empty: Option<i64>, // seconds, None if charging or fully charged
    pub time_to_full: Option<i64>,  // seconds, None if discharging
    pub is_present: bool,
}

impl Default for BatteryInfo {
    fn default() -> Self {
        BatteryInfo {
            percentage: 0.0,
            state: BatteryState::Unknown,
            time_to_empty: None,
            time_to_full: None,
            is_present: false,
        }
    }
}

/// Battery control via UPower D-Bus
pub struct BatteryControl {
    connection: Arc<RwLock<Option<Connection>>>,
    device_path: Arc<RwLock<Option<String>>>,
    info: Arc<RwLock<BatteryInfo>>,
    events: Option<EventManager>,
}

impl BatteryControl {
    /// Create a new battery control instance
    pub fn new() -> Self {
        BatteryControl {
            connection: Arc::new(RwLock::new(None)),
            device_path: Arc::new(RwLock::new(None)),
            info: Arc::new(RwLock::new(BatteryInfo::default())),
            events: None,
        }
    }

    /// Create with event manager for reactive updates
    pub fn with_events(events: EventManager) -> Self {
        let mut battery = Self::new();
        battery.events = Some(events);
        battery
    }

    /// Initialize connection to UPower
    pub async fn connect(&self) -> Result<()> {
        match Connection::system().await {
            Ok(conn) => {
                info!("Connected to D-Bus system bus for Battery control");

                // Store connection
                {
                    let mut connection = self.connection.write().await;
                    *connection = Some(conn.clone());
                }

                // Find battery device
                if let Err(e) = self.find_battery_device(&conn).await {
                    warn!("Failed to find battery device: {}", e);
                    return Err(e);
                }

                // Update battery info
                if let Err(e) = self.update_battery_info().await {
                    warn!("Failed to update battery info: {}", e);
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

    /// Find the battery device via UPower
    async fn find_battery_device(&self, conn: &Connection) -> Result<()> {
        let upower_proxy = zbus::ProxyBuilder::new(conn)
            .interface("org.freedesktop.UPower")
            .path("/org/freedesktop/UPower")?
            .destination("org.freedesktop.UPower")?
            .build::<zbus::Proxy>()
            .await
            .map_err(|e| AmiyaError::Backend(format!("Failed to create UPower proxy: {}", e)))?;

        // Enumerate devices
        let devices: Vec<zbus::zvariant::OwnedObjectPath> = upower_proxy
            .call_method("EnumerateDevices", &())
            .await
            .map_err(|e| AmiyaError::Backend(format!("Failed to enumerate devices: {}", e)))?
            .body()
            .deserialize()
            .map_err(|e| AmiyaError::Backend(format!("Failed to deserialize devices: {}", e)))?;

        debug!("Found {} UPower devices", devices.len());

        // Find the first battery device (Type == 2 means battery)
        for device_path in devices {
            let device_proxy = zbus::ProxyBuilder::new(conn)
                .interface("org.freedesktop.UPower.Device")
                .path(device_path.clone())?
                .destination("org.freedesktop.UPower")?
                .build::<zbus::Proxy>()
                .await
                .map_err(|e| {
                    AmiyaError::Backend(format!("Failed to create device proxy: {}", e))
                })?;

            // Get device type
            if let Ok(device_type) = device_proxy.get_property::<u32>("Type").await {
                if device_type == 2 {
                    // Type 2 is battery
                    let path = device_path.to_string();
                    info!("Found battery device: {}", path);

                    let mut device_path_lock = self.device_path.write().await;
                    *device_path_lock = Some(path);

                    return Ok(());
                }
            }
        }

        Err(AmiyaError::Backend(
            "No battery device found".to_string(),
        ))
    }

    /// Update battery information from UPower
    async fn update_battery_info(&self) -> Result<()> {
        let conn_guard = self.connection.read().await;
        let conn = conn_guard
            .as_ref()
            .ok_or_else(|| AmiyaError::Backend("Not connected to D-Bus".to_string()))?;

        let device_path_guard = self.device_path.read().await;
        let device_path = device_path_guard
            .as_ref()
            .ok_or_else(|| AmiyaError::Backend("No battery device found".to_string()))?;

        let device_proxy = zbus::ProxyBuilder::new(conn)
            .interface("org.freedesktop.UPower.Device")
            .path(device_path.as_str())?
            .destination("org.freedesktop.UPower")?
            .build::<zbus::Proxy>()
            .await
            .map_err(|e| AmiyaError::Backend(format!("Failed to create device proxy: {}", e)))?;

        // Get battery properties
        let percentage: f64 = device_proxy
            .get_property("Percentage")
            .await
            .unwrap_or(0.0);

        let state_raw: u32 = device_proxy.get_property("State").await.unwrap_or(0);
        let state = BatteryState::from(state_raw);

        let time_to_empty: i64 = device_proxy.get_property("TimeToEmpty").await.unwrap_or(0);
        let time_to_full: i64 = device_proxy.get_property("TimeToFull").await.unwrap_or(0);

        let is_present: bool = device_proxy
            .get_property("IsPresent")
            .await
            .unwrap_or(false);

        let new_info = BatteryInfo {
            percentage,
            state,
            time_to_empty: if time_to_empty > 0 {
                Some(time_to_empty)
            } else {
                None
            },
            time_to_full: if time_to_full > 0 {
                Some(time_to_full)
            } else {
                None
            },
            is_present,
        };

        // Update stored info
        {
            let mut info = self.info.write().await;
            *info = new_info.clone();
        }

        debug!(
            "Battery: {}% - {} (present: {})",
            percentage, state, is_present
        );

        // Emit event
        if let Some(events) = &self.events {
            events.emit(Event::BatteryChanged {
                percentage,
                state: state.to_string(),
                is_charging: matches!(state, BatteryState::Charging),
            });
        }

        Ok(())
    }

    /// Get current battery information
    pub async fn get_info(&self) -> BatteryInfo {
        // Update from D-Bus
        let _ = self.update_battery_info().await;

        // Return cached info
        self.info.read().await.clone()
    }

    /// Get battery percentage (0-100)
    pub async fn get_percentage(&self) -> f64 {
        self.info.read().await.percentage
    }

    /// Get battery state
    pub async fn get_state(&self) -> BatteryState {
        self.info.read().await.state
    }

    /// Check if battery is charging
    pub async fn is_charging(&self) -> bool {
        matches!(self.info.read().await.state, BatteryState::Charging)
    }

    /// Check if battery is present
    pub async fn is_present(&self) -> bool {
        self.info.read().await.is_present
    }

    /// Get time to empty in seconds (None if charging or fully charged)
    pub async fn get_time_to_empty(&self) -> Option<i64> {
        self.info.read().await.time_to_empty
    }

    /// Get time to full in seconds (None if discharging)
    pub async fn get_time_to_full(&self) -> Option<i64> {
        self.info.read().await.time_to_full
    }

    /// Format time in seconds to human readable string (e.g., "2h 30m")
    pub fn format_time(seconds: i64) -> String {
        if seconds <= 0 {
            return "Unknown".to_string();
        }

        let hours = seconds / 3600;
        let minutes = (seconds % 3600) / 60;

        if hours > 0 {
            format!("{}h {}m", hours, minutes)
        } else {
            format!("{}m", minutes)
        }
    }
}

impl Default for BatteryControl {
    fn default() -> Self {
        Self::new()
    }
}

/// Create battery control with sync initialization (for GTK/glib event loop integration)
pub fn create_battery_control_sync(events: EventManager) -> Arc<BatteryControl> {
    let battery = Arc::new(BatteryControl::with_events(events));

    // Try to initialize in background
    let battery_clone = battery.clone();
    tokio::spawn(async move {
        if let Err(e) = battery_clone.connect().await {
            debug!("Failed to initialize battery: {}", e);
        }
    });

    battery
}
