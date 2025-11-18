use crate::error::{AmiyaError, Result};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use zbus::Connection;

/// Power action types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PowerAction {
    Shutdown,
    Reboot,
    Suspend,
    Hibernate,
    Lock,
}

impl std::fmt::Display for PowerAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PowerAction::Shutdown => write!(f, "Shutdown"),
            PowerAction::Reboot => write!(f, "Reboot"),
            PowerAction::Suspend => write!(f, "Suspend"),
            PowerAction::Hibernate => write!(f, "Hibernate"),
            PowerAction::Lock => write!(f, "Lock"),
        }
    }
}

/// Power management via systemd/logind D-Bus
pub struct PowerControl {
    connection: Arc<RwLock<Option<Connection>>>,
}

impl PowerControl {
    /// Create a new power control instance
    pub fn new() -> Self {
        PowerControl {
            connection: Arc::new(RwLock::new(None)),
        }
    }

    /// Initialize connection to systemd/logind
    pub async fn connect(&self) -> Result<()> {
        match Connection::system().await {
            Ok(conn) => {
                info!("Connected to D-Bus system bus for Power control");

                // Store connection
                {
                    let mut connection = self.connection.write().await;
                    *connection = Some(conn);
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

    /// Execute a power action
    pub async fn execute(&self, action: PowerAction) -> Result<()> {
        let conn_guard = self.connection.read().await;
        let conn = conn_guard
            .as_ref()
            .ok_or_else(|| AmiyaError::Backend("Not connected to D-Bus".to_string()))?;

        match action {
            PowerAction::Shutdown => self.shutdown(conn).await,
            PowerAction::Reboot => self.reboot(conn).await,
            PowerAction::Suspend => self.suspend(conn).await,
            PowerAction::Hibernate => self.hibernate(conn).await,
            PowerAction::Lock => self.lock(conn).await,
        }
    }

    /// Shutdown the system
    async fn shutdown(&self, conn: &Connection) -> Result<()> {
        info!("Initiating system shutdown");

        let proxy = zbus::ProxyBuilder::new(conn)
            .interface("org.freedesktop.login1.Manager")
            .path("/org/freedesktop/login1")?
            .destination("org.freedesktop.login1")?
            .build::<zbus::Proxy>()
            .await
            .map_err(|e| {
                AmiyaError::Backend(format!("Failed to create logind proxy: {}", e))
            })?;

        // PowerOff(interactive: bool)
        proxy
            .call_method("PowerOff", &(true,))
            .await
            .map_err(|e| AmiyaError::Backend(format!("Failed to shutdown: {}", e)))?;

        Ok(())
    }

    /// Reboot the system
    async fn reboot(&self, conn: &Connection) -> Result<()> {
        info!("Initiating system reboot");

        let proxy = zbus::ProxyBuilder::new(conn)
            .interface("org.freedesktop.login1.Manager")
            .path("/org/freedesktop/login1")?
            .destination("org.freedesktop.login1")?
            .build::<zbus::Proxy>()
            .await
            .map_err(|e| {
                AmiyaError::Backend(format!("Failed to create logind proxy: {}", e))
            })?;

        // Reboot(interactive: bool)
        proxy
            .call_method("Reboot", &(true,))
            .await
            .map_err(|e| AmiyaError::Backend(format!("Failed to reboot: {}", e)))?;

        Ok(())
    }

    /// Suspend the system
    async fn suspend(&self, conn: &Connection) -> Result<()> {
        info!("Suspending system");

        let proxy = zbus::ProxyBuilder::new(conn)
            .interface("org.freedesktop.login1.Manager")
            .path("/org/freedesktop/login1")?
            .destination("org.freedesktop.login1")?
            .build::<zbus::Proxy>()
            .await
            .map_err(|e| {
                AmiyaError::Backend(format!("Failed to create logind proxy: {}", e))
            })?;

        // Suspend(interactive: bool)
        proxy
            .call_method("Suspend", &(true,))
            .await
            .map_err(|e| AmiyaError::Backend(format!("Failed to suspend: {}", e)))?;

        Ok(())
    }

    /// Hibernate the system
    async fn hibernate(&self, conn: &Connection) -> Result<()> {
        info!("Hibernating system");

        let proxy = zbus::ProxyBuilder::new(conn)
            .interface("org.freedesktop.login1.Manager")
            .path("/org/freedesktop/login1")?
            .destination("org.freedesktop.login1")?
            .build::<zbus::Proxy>()
            .await
            .map_err(|e| {
                AmiyaError::Backend(format!("Failed to create logind proxy: {}", e))
            })?;

        // Hibernate(interactive: bool)
        proxy
            .call_method("Hibernate", &(true,))
            .await
            .map_err(|e| AmiyaError::Backend(format!("Failed to hibernate: {}", e)))?;

        Ok(())
    }

    /// Lock the screen
    async fn lock(&self, conn: &Connection) -> Result<()> {
        info!("Locking screen");

        // Try to get the current session
        let session_path = self.get_current_session(conn).await?;

        let proxy = zbus::ProxyBuilder::new(conn)
            .interface("org.freedesktop.login1.Session")
            .path(session_path)?
            .destination("org.freedesktop.login1")?
            .build::<zbus::Proxy>()
            .await
            .map_err(|e| {
                AmiyaError::Backend(format!("Failed to create session proxy: {}", e))
            })?;

        // Lock()
        proxy
            .call_method("Lock", &())
            .await
            .map_err(|e| AmiyaError::Backend(format!("Failed to lock screen: {}", e)))?;

        Ok(())
    }

    /// Get the current session path
    async fn get_current_session(&self, conn: &Connection) -> Result<String> {
        let proxy = zbus::ProxyBuilder::new(conn)
            .interface("org.freedesktop.login1.Manager")
            .path("/org/freedesktop/login1")?
            .destination("org.freedesktop.login1")?
            .build::<zbus::Proxy>()
            .await
            .map_err(|e| {
                AmiyaError::Backend(format!("Failed to create logind proxy: {}", e))
            })?;

        // Get the user's UID
        let uid = unsafe { libc::getuid() };

        // GetSessionByPID(pid: u32) -> session_path
        let pid = std::process::id();

        let session_path: zbus::zvariant::OwnedObjectPath = proxy
            .call_method("GetSessionByPID", &(pid,))
            .await
            .map_err(|e| AmiyaError::Backend(format!("Failed to get session: {}", e)))?
            .body()
            .deserialize()
            .map_err(|e| AmiyaError::Backend(format!("Failed to deserialize session: {}", e)))?;

        Ok(session_path.to_string())
    }

    /// Check if action is available
    pub async fn can_execute(&self, action: PowerAction) -> bool {
        let conn_guard = self.connection.read().await;
        let Some(conn) = conn_guard.as_ref() else {
            return false;
        };

        let Ok(proxy) = zbus::ProxyBuilder::new(conn)
            .interface("org.freedesktop.login1.Manager")
            .path("/org/freedesktop/login1")
            .ok()
            .and_then(|b| b.destination("org.freedesktop.login1").ok())
            .and_then(|b| {
                let rt = tokio::runtime::Handle::current();
                rt.block_on(b.build::<zbus::Proxy>()).ok()
            })
        else {
            return false;
        };

        let method = match action {
            PowerAction::Shutdown => "CanPowerOff",
            PowerAction::Reboot => "CanReboot",
            PowerAction::Suspend => "CanSuspend",
            PowerAction::Hibernate => "CanHibernate",
            PowerAction::Lock => return true, // Lock is always available if we have a session
        };

        // Call CanXXX method - returns "yes", "no", "challenge", or "na"
        if let Ok(response) = proxy.call_method::<_, String>(method, &()).await {
            let result: Result<String, _> = response.body().deserialize();
            if let Ok(can) = result {
                return can == "yes" || can == "challenge";
            }
        }

        false
    }
}

impl Default for PowerControl {
    fn default() -> Self {
        Self::new()
    }
}

/// Create power control with sync initialization (for GTK/glib event loop integration)
pub fn create_power_control_sync() -> Arc<PowerControl> {
    let power = Arc::new(PowerControl::new());

    // Try to initialize in background
    let power_clone = power.clone();
    tokio::spawn(async move {
        if let Err(e) = power_clone.connect().await {
            debug!("Failed to initialize power control: {}", e);
        }
    });

    power
}
