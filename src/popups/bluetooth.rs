use crate::app::AppState;
use crate::events::{BluetoothDeviceInfo, Event};
use gtk4::glib;
use gtk4::prelude::*;
use gtk4::{
    Application, ApplicationWindow, Box as GtkBox, Button, Label, ListBox, Orientation,
    ScrolledWindow, Switch,
};
use gtk4_layer_shell::{Edge, Layer, LayerShell};
use std::sync::Arc;
use tracing::{debug, info, warn};

pub struct BluetoothPopup {
    window: ApplicationWindow,
    device_list: ListBox,
    toggle: Switch,
    state: Arc<AppState>,
}

impl BluetoothPopup {
    pub fn new(app: &Application, state: Arc<AppState>) -> Self {
        let window = ApplicationWindow::builder()
            .application(app)
            .title("Bluetooth")
            .default_width(400)
            .default_height(500)
            .build();

        // Initialize layer shell
        window.init_layer_shell();
        window.set_layer(Layer::Overlay);
        window.set_namespace("amiya-bluetooth");

        // Position in top-right
        window.set_anchor(Edge::Top, true);
        window.set_anchor(Edge::Right, true);
        window.set_margin(Edge::Top, 40);
        window.set_margin(Edge::Right, 10);

        // Create main container
        let container = GtkBox::new(Orientation::Vertical, 12);
        container.set_margin_start(16);
        container.set_margin_end(16);
        container.set_margin_top(16);
        container.set_margin_bottom(16);

        // Header with title and toggle
        let header = GtkBox::new(Orientation::Horizontal, 12);
        let title = Label::new(Some("🔵 Bluetooth"));
        title.set_halign(gtk4::Align::Start);
        title.set_hexpand(true);

        let toggle = Switch::new();
        toggle.set_valign(gtk4::Align::Center);

        // Set initial state from backend
        if let Some(bt) = &state.bluetooth_control {
            let bt_clone = bt.clone();
            glib::spawn_future_local(async move {
                if let Ok(powered) = bt_clone.is_powered().await {
                    glib::idle_add_once(move || {
                        // This will be set when we have the toggle reference
                    });
                }
            });
        }

        header.append(&title);
        header.append(&toggle);

        // Device list
        let scrolled = ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .min_content_height(300)
            .build();

        let device_list = ListBox::new();
        device_list.add_css_class("device-list");

        scrolled.set_child(Some(&device_list));

        // Scan button
        let scan_button = Button::with_label("Scan for Devices");
        scan_button.add_css_class("scan-button");

        container.append(&header);
        container.append(&scrolled);
        container.append(&scan_button);

        window.set_child(Some(&container));

        // Apply theme
        Self::apply_theme(&window);

        // Close on focus loss
        let window_clone = window.clone();
        window.connect_is_active_notify(move |win| {
            if !win.is_active() {
                window_clone.close();
            }
        });

        let popup = BluetoothPopup {
            window,
            device_list: device_list.clone(),
            toggle: toggle.clone(),
            state: state.clone(),
        };

        // Wire up toggle switch
        if let Some(bt) = &state.bluetooth_control {
            let bt_clone = bt.clone();
            toggle.connect_state_set(move |toggle, enabled| {
                let bt = bt_clone.clone();
                glib::spawn_future_local(async move {
                    if let Err(e) = bt.set_powered(enabled).await {
                        warn!("Failed to set Bluetooth powered state: {}", e);
                        // Revert toggle
                        glib::idle_add_once(move || {
                            toggle.set_active(!enabled);
                        });
                    } else {
                        info!("Bluetooth powered: {}", enabled);
                    }
                });
                glib::Propagation::Proceed
            });

            // Get initial powered state
            let toggle_clone = toggle.clone();
            let bt_clone = bt.clone();
            glib::spawn_future_local(async move {
                if let Ok(powered) = bt_clone.is_powered().await {
                    toggle_clone.set_active(powered);
                }
            });
        }

        // Wire up scan button
        if let Some(bt) = &state.bluetooth_control {
            let bt_clone = bt.clone();
            let scan_button_clone = scan_button.clone();
            scan_button.connect_clicked(move |_| {
                let bt = bt_clone.clone();
                let button = scan_button_clone.clone();
                button.set_sensitive(false);
                button.set_label("Scanning...");

                glib::spawn_future_local(async move {
                    if let Err(e) = bt.start_scan().await {
                        warn!("Failed to start Bluetooth scan: {}", e);
                    } else {
                        info!("Bluetooth scan started");

                        // Stop scan after 10 seconds
                        let bt_scan = bt.clone();
                        glib::timeout_add_seconds_local(10, move || {
                            let bt = bt_scan.clone();
                            glib::spawn_future_local(async move {
                                let _ = bt.stop_scan().await;
                            });
                            glib::ControlFlow::Break
                        });
                    }

                    // Re-enable button after 1 second
                    glib::timeout_add_seconds_local(1, move || {
                        button.set_sensitive(true);
                        button.set_label("Scan for Devices");
                        glib::ControlFlow::Break
                    });
                });
            });
        }

        // Subscribe to Bluetooth events
        let device_list_clone = device_list.clone();
        let state_clone = state.clone();
        glib::spawn_future_local(async move {
            let mut receiver = state_clone.events.subscribe();

            loop {
                match receiver.recv().await {
                    Ok(Event::BluetoothDevicesUpdated { devices }) => {
                        debug!("Received {} Bluetooth devices", devices.len());
                        Self::update_device_list(&device_list_clone, &devices, &state_clone);
                    }
                    Ok(Event::BluetoothStateChanged { enabled }) => {
                        debug!("Bluetooth state changed: {}", enabled);
                    }
                    Ok(Event::BluetoothDeviceConnected { address, name }) => {
                        info!("Device connected: {} ({})", name, address);
                    }
                    Ok(Event::BluetoothDeviceDisconnected { address }) => {
                        info!("Device disconnected: {}", address);
                    }
                    _ => {}
                }
            }
        });

        // Initial load of devices
        if let Some(bt) = &state.bluetooth_control {
            let device_list_clone = device_list.clone();
            let state_clone = state.clone();
            let bt_clone = bt.clone();
            glib::spawn_future_local(async move {
                if let Ok(devices) = bt_clone.get_devices().await {
                    Self::update_device_list(&device_list_clone, &devices, &state_clone);
                }
            });
        }

        popup
    }

    fn update_device_list(
        list: &ListBox,
        devices: &[BluetoothDeviceInfo],
        state: &Arc<AppState>,
    ) {
        // Clear existing rows
        while let Some(row) = list.first_child() {
            list.remove(&row);
        }

        // Add devices
        if devices.is_empty() {
            let label = Label::new(Some("No devices found"));
            label.set_margin_top(32);
            label.set_margin_bottom(32);
            label.add_css_class("empty-message");
            list.append(&label);
        } else {
            for device in devices {
                Self::add_device(list, device, state);
            }
        }
    }

    fn add_device(list: &ListBox, device: &BluetoothDeviceInfo, state: &Arc<AppState>) {
        let row = GtkBox::new(Orientation::Horizontal, 12);
        row.set_margin_start(8);
        row.set_margin_end(8);
        row.set_margin_top(8);
        row.set_margin_bottom(8);

        let device_info = GtkBox::new(Orientation::Vertical, 4);
        let name_label = Label::new(Some(&device.name));
        name_label.set_halign(gtk4::Align::Start);
        name_label.add_css_class("device-name");

        let status = if device.connected {
            "Connected"
        } else if device.paired {
            "Paired"
        } else {
            "Available"
        };
        let status_label = Label::new(Some(status));
        status_label.set_halign(gtk4::Align::Start);
        status_label.add_css_class("device-status");

        device_info.append(&name_label);
        device_info.append(&status_label);
        device_info.set_hexpand(true);

        let connect_btn = Button::with_label(if device.connected {
            "Disconnect"
        } else {
            "Connect"
        });
        connect_btn.set_valign(gtk4::Align::Center);

        // Wire up connect/disconnect button
        if let Some(bt) = &state.bluetooth_control {
            let bt_clone = bt.clone();
            let address = device.address.clone();
            let is_connected = device.connected;
            let button_clone = connect_btn.clone();

            connect_btn.connect_clicked(move |_| {
                let bt = bt_clone.clone();
                let addr = address.clone();
                let button = button_clone.clone();

                button.set_sensitive(false);

                glib::spawn_future_local(async move {
                    let result = if is_connected {
                        bt.disconnect_device(&addr).await
                    } else {
                        bt.connect_device(&addr).await
                    };

                    match result {
                        Ok(()) => {
                            info!(
                                "{} device: {}",
                                if is_connected { "Disconnected" } else { "Connected" },
                                addr
                            );
                        }
                        Err(e) => {
                            warn!(
                                "Failed to {} device {}: {}",
                                if is_connected { "disconnect" } else { "connect" },
                                addr,
                                e
                            );
                        }
                    }

                    // Re-enable button
                    button.set_sensitive(true);
                });
            });
        }

        row.append(&device_info);
        row.append(&connect_btn);

        list.append(&row);
    }

    pub fn show(&self) {
        // Refresh device list when showing
        if let Some(bt) = &self.state.bluetooth_control {
            let device_list = self.device_list.clone();
            let state = self.state.clone();
            let bt_clone = bt.clone();

            glib::spawn_future_local(async move {
                if let Ok(devices) = bt_clone.get_devices().await {
                    Self::update_device_list(&device_list, &devices, &state);
                }
            });
        }

        self.window.present();
    }

    pub fn hide(&self) {
        self.window.close();
    }

    fn apply_theme(window: &ApplicationWindow) {
        let provider = gtk4::CssProvider::new();
        let css = r#"
            window {
                background-color: rgba(30, 30, 46, 0.98);
                border-radius: 12px;
                color: #cdd6f4;
            }

            .device-list {
                background-color: transparent;
            }

            .device-name {
                font-weight: bold;
            }

            .device-status {
                font-size: 10pt;
                color: #a6adc8;
            }

            .empty-message {
                color: #a6adc8;
                font-style: italic;
            }

            button.scan-button {
                background-color: #89b4fa;
                color: #1e1e2e;
                border-radius: 6px;
                padding: 8px;
                font-weight: bold;
            }
        "#;

        provider.load_from_string(css);

        gtk4::style_context_add_provider_for_display(
            &window.display(),
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
}
