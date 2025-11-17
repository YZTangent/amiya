use crate::backend::NiriClient;
use crate::config::Config;
use crate::error::BackendStatus;
use crate::events::EventManager;
use anyhow::Result;
use gtk4::glib;
use std::sync::Arc;
use tracing::{info, warn};

/// Global application state coordinator
pub struct AppState {
    /// Configuration
    pub config: Config,

    /// Event manager for broadcasting events
    pub events: EventManager,

    /// Backend availability status
    pub backend_status: BackendStatus,

    /// Niri IPC client (optional - may be None if niri is not running)
    pub niri_client: Option<Arc<NiriClient>>,
}

impl AppState {
    /// Create new application state
    pub fn new(config: Config) -> Self {
        info!("Initializing application state");

        let events = EventManager::default();

        // Try to connect to niri
        let niri_client = match NiriClient::new() {
            Ok(client) => {
                info!("Successfully connected to niri compositor");
                Some(Arc::new(client))
            }
            Err(e) => {
                warn!("Could not connect to niri: {}. Workspace features will be limited.", e);
                None
            }
        };

        // Check backend availability
        let backend_status = if niri_client.is_some() {
            BackendStatus::Available
        } else {
            BackendStatus::Unavailable
        };

        AppState {
            config,
            events,
            backend_status,
            niri_client,
        }
    }

    /// Check if system backends are available
    fn check_backend_availability() -> BackendStatus {
        // Deprecated - status is now set during initialization
        BackendStatus::Available
    }

    /// Get an Arc-wrapped clone for sharing across threads
    pub fn shared(self) -> Arc<Self> {
        Arc::new(self)
    }
}

/// Application lifecycle manager
pub struct Application {
    state: Arc<AppState>,
}

impl Application {
    /// Create a new application instance
    pub fn new(config: Config) -> Self {
        let state = AppState::new(config).shared();

        Application { state }
    }

    /// Get a reference to the application state
    pub fn state(&self) -> &Arc<AppState> {
        &self.state
    }

    /// Initialize all subsystems
    pub fn initialize(&self) -> Result<()> {
        info!("Initializing application subsystems");

        // Start system monitors
        self.start_system_monitors()?;

        // Start backend listeners
        self.start_backend_listeners()?;

        Ok(())
    }

    /// Start system monitoring tasks
    fn start_system_monitors(&self) -> Result<()> {
        use crate::events::Event;
        use sysinfo::{CpuRefreshKind, RefreshKind, System};

        let events = self.state.events.clone();

        // CPU and Memory monitoring
        glib::timeout_add_seconds_local(2, move || {
            // Gracefully handle sysinfo errors
            let result = std::panic::catch_unwind(|| {
                let mut sys = System::new_with_specifics(
                    RefreshKind::new()
                        .with_cpu(CpuRefreshKind::everything())
                        .with_memory(),
                );

                sys.refresh_cpu_all();
                sys.refresh_memory();

                // CPU usage
                let cpu_usage = sys.global_cpu_usage() as f64;
                events.emit(Event::CpuUsageChanged { usage: cpu_usage });

                // Memory usage
                let used = sys.used_memory();
                let total = sys.total_memory();
                let percent = (used as f64 / total as f64) * 100.0;
                events.emit(Event::MemoryUsageChanged {
                    used,
                    total,
                    percent,
                });
            });

            if let Err(e) = result {
                warn!("System monitoring error: {:?}", e);
            }

            glib::ControlFlow::Continue
        });

        // Temperature monitoring
        let events = self.state.events.clone();
        glib::timeout_add_seconds_local(5, move || {
            match read_cpu_temp() {
                Ok(temp) => {
                    events.emit(Event::TemperatureChanged { celsius: temp });
                }
                Err(e) => {
                    // Don't spam logs - temperature read failures are common on some systems
                    tracing::debug!("Temperature read failed: {}", e);
                }
            }
            glib::ControlFlow::Continue
        });

        Ok(())
    }

    /// Start backend event listeners
    fn start_backend_listeners(&self) -> Result<()> {
        // Start niri workspace polling if client is available
        if let Some(niri_client) = &self.state.niri_client {
            info!("Starting niri workspace polling");
            crate::backend::niri::start_workspace_polling(
                niri_client.clone(),
                self.state.events.clone(),
                2, // Poll every 2 seconds
            );
        } else {
            info!("Niri client not available, skipping workspace polling");
        }

        // In Phase 4, this will also start:
        // - D-Bus signal listeners for audio, network, bluetooth
        // - MPRIS media player listeners

        Ok(())
    }

    /// Shutdown the application gracefully
    pub fn shutdown(&self) {
        info!("Shutting down application");
        // Cleanup resources, close connections, etc.
    }
}

/// Read CPU temperature from sysfs
fn read_cpu_temp() -> Result<i32> {
    let thermal_paths = [
        "/sys/class/thermal/thermal_zone0/temp",
        "/sys/class/thermal/thermal_zone1/temp",
    ];

    for path in &thermal_paths {
        if let Ok(content) = std::fs::read_to_string(path) {
            if let Ok(temp) = content.trim().parse::<i32>() {
                return Ok(temp / 1000); // Convert from millidegrees
            }
        }
    }

    anyhow::bail!("No thermal zone found")
}
