use crate::config::Config;
use crate::events::EventManager;
use anyhow::Result;
use gtk4::glib;
use std::sync::Arc;
use tracing::info;

/// Global application state coordinator
pub struct AppState {
    /// Configuration
    pub config: Config,

    /// Event manager for broadcasting events
    pub events: EventManager,
}

impl AppState {
    /// Create new application state
    pub fn new(config: Config) -> Self {
        info!("Initializing application state");

        let events = EventManager::default();

        AppState { config, events }
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

            glib::ControlFlow::Continue
        });

        // Temperature monitoring
        let events = self.state.events.clone();
        glib::timeout_add_seconds_local(5, move || {
            if let Ok(temp) = read_cpu_temp() {
                events.emit(Event::TemperatureChanged { celsius: temp });
            }
            glib::ControlFlow::Continue
        });

        Ok(())
    }

    /// Start backend event listeners
    fn start_backend_listeners(&self) -> Result<()> {
        // In future phases, this will start:
        // - Niri IPC event listener
        // - D-Bus signal listeners for audio, network, bluetooth
        // - MPRIS media player listeners

        info!("Backend listeners will be initialized in Phase 3 & 4");

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
