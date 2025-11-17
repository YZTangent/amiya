use crate::app::AppState;
use crate::events::Event;
use gtk4::prelude::*;
use gtk4::{glib, Box as GtkBox, Label, Orientation};
use std::sync::Arc;

pub struct SystemInfo {
    container: GtkBox,
}

impl SystemInfo {
    pub fn new(state: &Arc<AppState>) -> Self {
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

        // Subscribe to events
        Self::subscribe_to_events(
            state.events.clone(),
            cpu_label.clone(),
            mem_label.clone(),
            temp_label.clone(),
            wifi_label.clone(),
            bt_label.clone(),
        );

        SystemInfo { container }
    }

    pub fn widget(&self) -> GtkBox {
        self.container.clone()
    }

    fn subscribe_to_events(
        events: crate::events::EventManager,
        cpu_label: Label,
        mem_label: Label,
        temp_label: Label,
        wifi_label: Label,
        bt_label: Label,
    ) {
        let mut receiver = events.subscribe();

        // Spawn event listener
        glib::spawn_future_local(async move {
            loop {
                match receiver.recv().await {
                    Ok(event) => match event {
                        Event::CpuUsageChanged { usage } => {
                            cpu_label.set_text(&format!("CPU: {:.1}%", usage));
                        }
                        Event::MemoryUsageChanged { percent, .. } => {
                            mem_label.set_text(&format!("MEM: {:.1}%", percent));
                        }
                        Event::TemperatureChanged { celsius } => {
                            temp_label.set_text(&format!("TEMP: {}°C", celsius));
                        }
                        Event::WifiStateChanged { enabled } => {
                            let text = if enabled {
                                "📶 WiFi"
                            } else {
                                "📶 WiFi (Off)"
                            };
                            wifi_label.set_text(text);
                        }
                        Event::WifiNetworkConnected { ssid } => {
                            wifi_label.set_text(&format!("📶 {}", ssid));
                        }
                        Event::WifiNetworkDisconnected => {
                            wifi_label.set_text("📶 WiFi");
                        }
                        Event::BluetoothStateChanged { enabled } => {
                            let text = if enabled {
                                "🔵 BT"
                            } else {
                                "🔵 BT (Off)"
                            };
                            bt_label.set_text(text);
                        }
                        Event::BluetoothDeviceConnected { name, .. } => {
                            bt_label.set_text(&format!("🔵 {}", name));
                        }
                        Event::BluetoothDeviceDisconnected { .. } => {
                            bt_label.set_text("🔵 BT");
                        }
                        _ => {} // Ignore other events
                    },
                    Err(_) => {
                        // Channel closed, exit loop
                        break;
                    }
                }
            }
        });
    }
}
