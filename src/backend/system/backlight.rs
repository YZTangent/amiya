use crate::error::{AmiyaError, Result};
use crate::events::{Event, EventManager};
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Backlight control via sysfs
pub struct BacklightControl {
    device_path: Option<PathBuf>,
    current_brightness: Arc<RwLock<f64>>,
    events: Option<EventManager>,
}

impl BacklightControl {
    /// Create a new backlight control instance
    pub fn new() -> Self {
        let device_path = Self::find_backlight_device();

        if device_path.is_none() {
            warn!("No backlight device found in /sys/class/backlight");
        } else {
            info!("Found backlight device: {:?}", device_path);
        }

        BacklightControl {
            device_path,
            current_brightness: Arc::new(RwLock::new(50.0)),
            events: None,
        }
    }

    /// Create with event manager for reactive updates
    pub fn with_events(events: EventManager) -> Self {
        let mut backlight = Self::new();
        backlight.events = Some(events);
        backlight
    }

    /// Find the first available backlight device
    fn find_backlight_device() -> Option<PathBuf> {
        let backlight_dir = PathBuf::from("/sys/class/backlight");
        if !backlight_dir.exists() {
            return None;
        }

        // Try to find preferred devices first (intel_backlight, amdgpu_bl0, etc.)
        let preferred = ["intel_backlight", "amdgpu_bl0", "radeon_bl0", "acpi_video0"];

        for device_name in &preferred {
            let path = backlight_dir.join(device_name);
            if path.exists() {
                return Some(path);
            }
        }

        // Fall back to first available device
        if let Ok(entries) = fs::read_dir(&backlight_dir) {
            for entry in entries.flatten() {
                return Some(entry.path());
            }
        }

        None
    }

    /// Check if backlight control is available
    pub fn is_available(&self) -> bool {
        self.device_path.is_some()
    }

    /// Get current brightness level (0-100)
    pub async fn get_brightness(&self) -> Result<f64> {
        // Try to read from sysfs first
        if let Some(device) = &self.device_path {
            match self.read_brightness_from_sysfs(device) {
                Ok(brightness) => {
                    // Update cached value
                    let mut current = self.current_brightness.write().await;
                    *current = brightness;
                    return Ok(brightness);
                }
                Err(e) => {
                    debug!("Failed to read brightness from sysfs: {}", e);
                }
            }
        }

        // Fall back to cached value
        let brightness = *self.current_brightness.read().await;
        Ok(brightness)
    }

    /// Read brightness directly from sysfs
    fn read_brightness_from_sysfs(&self, device: &PathBuf) -> Result<f64> {
        let current = fs::read_to_string(device.join("brightness"))
            .map_err(|e| AmiyaError::Backend(format!("Failed to read brightness: {}", e)))?
            .trim()
            .parse::<f64>()
            .map_err(|e| AmiyaError::Backend(format!("Failed to parse brightness: {}", e)))?;

        let max = fs::read_to_string(device.join("max_brightness"))
            .map_err(|e| AmiyaError::Backend(format!("Failed to read max_brightness: {}", e)))?
            .trim()
            .parse::<f64>()
            .map_err(|e| AmiyaError::Backend(format!("Failed to parse max_brightness: {}", e)))?;

        if max == 0.0 {
            return Err(AmiyaError::Backend("Invalid max_brightness: 0".to_string()));
        }

        Ok((current / max) * 100.0)
    }

    /// Set brightness level (0-100)
    pub async fn set_brightness(&self, brightness: f64) -> Result<()> {
        let brightness = brightness.clamp(0.0, 100.0);

        // Update cached value
        {
            let mut current = self.current_brightness.write().await;
            *current = brightness;
        }

        // Try to write to sysfs
        if let Some(device) = &self.device_path {
            if let Err(e) = self.write_brightness_to_sysfs(device, brightness) {
                warn!("Failed to set brightness via sysfs: {}. You may need to configure udev rules.", e);
                // Don't return error - we still updated cached value and will emit event
            } else {
                info!("Brightness set to {:.1}%", brightness);
            }
        } else {
            debug!("No backlight device available, using mock brightness: {:.1}%", brightness);
        }

        // Emit event
        if let Some(events) = &self.events {
            events.emit(Event::BrightnessChanged { level: brightness });
        }

        Ok(())
    }

    /// Write brightness to sysfs
    fn write_brightness_to_sysfs(&self, device: &PathBuf, percent: f64) -> Result<()> {
        let max = fs::read_to_string(device.join("max_brightness"))
            .map_err(|e| AmiyaError::Backend(format!("Failed to read max_brightness: {}", e)))?
            .trim()
            .parse::<f64>()
            .map_err(|e| AmiyaError::Backend(format!("Failed to parse max_brightness: {}", e)))?;

        let value = ((percent / 100.0) * max).round() as u32;

        // Writing to brightness requires root or proper udev rules
        fs::write(device.join("brightness"), value.to_string())
            .map_err(|e| AmiyaError::Backend(format!("Failed to write brightness: {}", e)))?;

        Ok(())
    }

    /// Increase brightness by step
    pub async fn increase_brightness(&self, step: f64) -> Result<()> {
        let current = self.get_brightness().await?;
        let new_brightness = (current + step).min(100.0);
        self.set_brightness(new_brightness).await
    }

    /// Decrease brightness by step
    pub async fn decrease_brightness(&self, step: f64) -> Result<()> {
        let current = self.get_brightness().await?;
        let new_brightness = (current - step).max(0.0);
        self.set_brightness(new_brightness).await
    }

    /// Initialize backlight state (read current value)
    pub async fn initialize(&self) -> Result<()> {
        if let Ok(brightness) = self.get_brightness().await {
            info!("Initial brightness: {:.1}%", brightness);
        }
        Ok(())
    }
}

impl Default for BacklightControl {
    fn default() -> Self {
        Self::new()
    }
}

// Helper function to create backlight control in GTK context
pub fn create_backlight_control_sync(events: EventManager) -> Arc<BacklightControl> {
    let backlight = Arc::new(BacklightControl::with_events(events));

    // Try to initialize in background
    let backlight_clone = backlight.clone();
    tokio::spawn(async move {
        if let Err(e) = backlight_clone.initialize().await {
            debug!("Failed to initialize backlight: {}", e);
        }
    });

    backlight
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_brightness_control() {
        let backlight = BacklightControl::new();

        backlight.set_brightness(75.0).await.unwrap();
        assert_eq!(backlight.get_brightness().await.unwrap(), 75.0);

        backlight.increase_brightness(10.0).await.unwrap();
        assert_eq!(backlight.get_brightness().await.unwrap(), 85.0);

        backlight.decrease_brightness(20.0).await.unwrap();
        assert_eq!(backlight.get_brightness().await.unwrap(), 65.0);
    }

    #[tokio::test]
    async fn test_brightness_clamping() {
        let backlight = BacklightControl::new();

        backlight.set_brightness(150.0).await.unwrap();
        assert_eq!(backlight.get_brightness().await.unwrap(), 100.0);

        backlight.set_brightness(-10.0).await.unwrap();
        assert_eq!(backlight.get_brightness().await.unwrap(), 0.0);
    }

    #[tokio::test]
    async fn test_brightness_bounds() {
        let backlight = BacklightControl::new();

        backlight.set_brightness(95.0).await.unwrap();
        backlight.increase_brightness(10.0).await.unwrap();
        assert_eq!(backlight.get_brightness().await.unwrap(), 100.0);

        backlight.set_brightness(5.0).await.unwrap();
        backlight.decrease_brightness(10.0).await.unwrap();
        assert_eq!(backlight.get_brightness().await.unwrap(), 0.0);
    }
}
