use anyhow::Result;
use std::fs;
use std::path::PathBuf;

pub struct BacklightControl {
    device_path: Option<PathBuf>,
}

impl BacklightControl {
    pub fn new() -> Self {
        let device_path = Self::find_backlight_device();
        BacklightControl { device_path }
    }

    fn find_backlight_device() -> Option<PathBuf> {
        let backlight_dir = PathBuf::from("/sys/class/backlight");
        if !backlight_dir.exists() {
            return None;
        }

        // Find first backlight device
        if let Ok(entries) = fs::read_dir(&backlight_dir) {
            for entry in entries.flatten() {
                return Some(entry.path());
            }
        }

        None
    }

    pub fn get_brightness(&self) -> Result<f64> {
        let device = self.device_path.as_ref()
            .ok_or_else(|| anyhow::anyhow!("No backlight device found"))?;

        let current = fs::read_to_string(device.join("brightness"))?
            .trim()
            .parse::<f64>()?;

        let max = fs::read_to_string(device.join("max_brightness"))?
            .trim()
            .parse::<f64>()?;

        Ok((current / max) * 100.0)
    }

    pub fn set_brightness(&self, percent: f64) -> Result<()> {
        let device = self.device_path.as_ref()
            .ok_or_else(|| anyhow::anyhow!("No backlight device found"))?;

        let max = fs::read_to_string(device.join("max_brightness"))?
            .trim()
            .parse::<f64>()?;

        let value = ((percent / 100.0) * max) as u32;

        // Writing to brightness requires root or proper udev rules
        fs::write(device.join("brightness"), value.to_string())?;

        tracing::info!("Setting brightness to {}%", percent);
        Ok(())
    }
}

impl Default for BacklightControl {
    fn default() -> Self {
        Self::new()
    }
}
