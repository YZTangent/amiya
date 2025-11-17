use crate::config::Config;
use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Label, Orientation};
use std::sync::{Arc, Mutex};
use sysinfo::{CpuRefreshKind, RefreshKind, System};

pub struct SystemInfo {
    container: GtkBox,
}

impl SystemInfo {
    pub fn new(config: &Config) -> Self {
        let container = GtkBox::new(Orientation::Horizontal, 12);

        // CPU usage
        let cpu_label = Label::new(Some("CPU: ---%"));
        cpu_label.add_css_class("system-info-label");
        container.append(&cpu_label);

        // Memory usage
        let mem_label = Label::new(Some("MEM: ---%"));
        mem_label.add_css_class("system-info-label");
        container.append(&mem_label);

        // Temperature
        let temp_label = Label::new(Some("TEMP: --°C"));
        temp_label.add_css_class("system-info-label");
        container.append(&temp_label);

        // WiFi status
        let wifi_label = Label::new(Some("📶 WiFi"));
        wifi_label.add_css_class("system-info-label");
        container.append(&wifi_label);

        // Bluetooth status
        let bt_label = Label::new(Some("🔵 BT"));
        bt_label.add_css_class("system-info-label");
        container.append(&bt_label);

        // Start monitoring
        Self::start_monitoring(
            cpu_label.clone(),
            mem_label.clone(),
            temp_label.clone(),
            config.clone(),
        );

        SystemInfo { container }
    }

    pub fn widget(&self) -> GtkBox {
        self.container.clone()
    }

    fn start_monitoring(
        cpu_label: Label,
        mem_label: Label,
        temp_label: Label,
        _config: Config,
    ) {
        let sys = Arc::new(Mutex::new(System::new_with_specifics(
            RefreshKind::new()
                .with_cpu(CpuRefreshKind::everything())
                .with_memory(),
        )));

        // Update every 2 seconds
        glib::timeout_add_seconds_local(2, move || {
            let mut sys = sys.lock().unwrap();
            sys.refresh_cpu_all();
            sys.refresh_memory();

            // CPU usage
            let cpu_usage = sys.global_cpu_usage();
            cpu_label.set_text(&format!("CPU: {:.1}%", cpu_usage));

            // Memory usage
            let mem_used = sys.used_memory();
            let mem_total = sys.total_memory();
            let mem_percent = (mem_used as f64 / mem_total as f64) * 100.0;
            mem_label.set_text(&format!("MEM: {:.1}%", mem_percent));

            // Temperature (this is system-dependent)
            // On Linux, you'd typically read from /sys/class/thermal/thermal_zone*/temp
            if let Ok(temp) = Self::read_cpu_temp() {
                temp_label.set_text(&format!("TEMP: {}°C", temp));
            }

            glib::ControlFlow::Continue
        });
    }

    fn read_cpu_temp() -> Result<i32, std::io::Error> {
        // Try to read from common thermal zones
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

        Ok(0) // Default if no thermal zone found
    }
}
