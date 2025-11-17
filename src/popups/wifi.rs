use crate::app::AppState;
use crate::events::{Event, WifiNetworkInfo};
use gtk4::glib;
use gtk4::prelude::*;
use gtk4::{
    Application, ApplicationWindow, Box as GtkBox, Button, Label, ListBox, Orientation,
    ScrolledWindow, Switch,
};
use gtk4_layer_shell::{Edge, Layer, LayerShell};
use std::sync::Arc;
use tracing::{debug, info, warn};

pub struct WifiPopup {
    window: ApplicationWindow,
    network_list: ListBox,
    toggle: Switch,
    state: Arc<AppState>,
}

impl WifiPopup {
    pub fn new(app: &Application, state: Arc<AppState>) -> Self {
        let window = ApplicationWindow::builder()
            .application(app)
            .title("WiFi")
            .default_width(400)
            .default_height(500)
            .build();

        // Initialize layer shell
        window.init_layer_shell();
        window.set_layer(Layer::Overlay);
        window.set_namespace("amiya-wifi");

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
        let title = Label::new(Some("📶 WiFi"));
        title.set_halign(gtk4::Align::Start);
        title.set_hexpand(true);

        let toggle = Switch::new();
        toggle.set_valign(gtk4::Align::Center);

        header.append(&title);
        header.append(&toggle);

        // Network list
        let scrolled = ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .min_content_height(300)
            .build();

        let network_list = ListBox::new();
        network_list.add_css_class("network-list");

        scrolled.set_child(Some(&network_list));

        // Refresh button
        let refresh_button = Button::with_label("Refresh Networks");
        refresh_button.add_css_class("refresh-button");

        container.append(&header);
        container.append(&scrolled);
        container.append(&refresh_button);

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

        let popup = WifiPopup {
            window,
            network_list: network_list.clone(),
            toggle: toggle.clone(),
            state: state.clone(),
        };

        // Wire up toggle switch
        if let Some(nm) = &state.network_control {
            let nm_clone = nm.clone();
            toggle.connect_state_set(move |toggle, enabled| {
                let nm = nm_clone.clone();
                glib::spawn_future_local(async move {
                    if let Err(e) = nm.set_wifi_enabled(enabled).await {
                        warn!("Failed to set WiFi enabled state: {}", e);
                        // Revert toggle
                        glib::idle_add_once(move || {
                            toggle.set_active(!enabled);
                        });
                    } else {
                        info!("WiFi enabled: {}", enabled);
                    }
                });
                glib::Propagation::Proceed
            });

            // Get initial WiFi state
            let toggle_clone = toggle.clone();
            let nm_clone = nm.clone();
            glib::spawn_future_local(async move {
                if let Ok(enabled) = nm_clone.is_wifi_enabled().await {
                    toggle_clone.set_active(enabled);
                }
            });
        }

        // Wire up refresh button
        if let Some(nm) = &state.network_control {
            let nm_clone = nm.clone();
            let refresh_button_clone = refresh_button.clone();
            refresh_button.connect_clicked(move |_| {
                let nm = nm_clone.clone();
                let button = refresh_button_clone.clone();
                button.set_sensitive(false);
                button.set_label("Scanning...");

                glib::spawn_future_local(async move {
                    // Start scan
                    if let Err(e) = nm.scan().await {
                        warn!("Failed to start WiFi scan: {}", e);
                    } else {
                        info!("WiFi scan started");

                        // Wait a moment, then get networks
                        glib::timeout_add_seconds_local(2, {
                            let nm = nm.clone();
                            move || {
                                let nm = nm.clone();
                                glib::spawn_future_local(async move {
                                    if let Err(e) = nm.get_networks().await {
                                        warn!("Failed to get networks: {}", e);
                                    }
                                });
                                glib::ControlFlow::Break
                            }
                        });
                    }

                    // Re-enable button
                    glib::timeout_add_seconds_local(1, move || {
                        button.set_sensitive(true);
                        button.set_label("Refresh Networks");
                        glib::ControlFlow::Break
                    });
                });
            });
        }

        // Subscribe to WiFi events
        let network_list_clone = network_list.clone();
        let state_clone = state.clone();
        glib::spawn_future_local(async move {
            let mut receiver = state_clone.events.subscribe();

            loop {
                match receiver.recv().await {
                    Ok(Event::WifiNetworksUpdated { networks }) => {
                        debug!("Received {} WiFi networks", networks.len());
                        Self::update_network_list(&network_list_clone, &networks, &state_clone);
                    }
                    Ok(Event::WifiStateChanged { enabled }) => {
                        debug!("WiFi state changed: {}", enabled);
                    }
                    Ok(Event::WifiNetworkConnected { ssid }) => {
                        info!("Connected to network: {}", ssid);
                    }
                    Ok(Event::WifiNetworkDisconnected) => {
                        info!("Disconnected from network");
                    }
                    _ => {}
                }
            }
        });

        // Initial load of networks
        if let Some(nm) = &state.network_control {
            let network_list_clone = network_list.clone();
            let state_clone = state.clone();
            let nm_clone = nm.clone();
            glib::spawn_future_local(async move {
                // Trigger a scan to get fresh data
                let _ = nm_clone.scan().await;

                // Wait a moment for scan to complete
                glib::timeout_add_seconds_local(2, {
                    let nm = nm_clone.clone();
                    let list = network_list_clone.clone();
                    let state = state_clone.clone();
                    move || {
                        let nm = nm.clone();
                        let list = list.clone();
                        let state = state.clone();
                        glib::spawn_future_local(async move {
                            if let Ok(networks) = nm.get_networks().await {
                                Self::update_network_list(&list, &networks, &state);
                            }
                        });
                        glib::ControlFlow::Break
                    }
                });
            });
        }

        popup
    }

    fn update_network_list(
        list: &ListBox,
        networks: &[WifiNetworkInfo],
        state: &Arc<AppState>,
    ) {
        // Clear existing rows
        while let Some(row) = list.first_child() {
            list.remove(&row);
        }

        // Add networks
        if networks.is_empty() {
            let label = Label::new(Some("No networks found"));
            label.set_margin_top(32);
            label.set_margin_bottom(32);
            label.add_css_class("empty-message");
            list.append(&label);
        } else {
            for network in networks {
                Self::add_network(list, network, state);
            }
        }
    }

    fn add_network(list: &ListBox, network: &WifiNetworkInfo, state: &Arc<AppState>) {
        let row = GtkBox::new(Orientation::Horizontal, 12);
        row.set_margin_start(8);
        row.set_margin_end(8);
        row.set_margin_top(8);
        row.set_margin_bottom(8);

        // Signal strength bars
        let signal_bars = Self::signal_to_bars(network.signal_strength);
        let signal_label = Label::new(Some(&signal_bars));
        signal_label.add_css_class("signal-strength");

        // Network info
        let network_info = GtkBox::new(Orientation::Vertical, 4);
        let name_label = Label::new(Some(&network.ssid));
        name_label.set_halign(gtk4::Align::Start);
        name_label.add_css_class("network-name");

        let status = if network.connected {
            "Connected"
        } else if network.secured {
            "Secured"
        } else {
            "Open"
        };
        let status_label = Label::new(Some(status));
        status_label.set_halign(gtk4::Align::Start);
        status_label.add_css_class("network-status");

        network_info.append(&name_label);
        network_info.append(&status_label);
        network_info.set_hexpand(true);

        // Lock icon for secured networks
        if network.secured {
            let lock_label = Label::new(Some("🔒"));
            row.append(&lock_label);
        }

        let connect_btn = Button::with_label(if network.connected {
            "Disconnect"
        } else {
            "Connect"
        });
        connect_btn.set_valign(gtk4::Align::Center);

        // Wire up connect/disconnect button
        if let Some(nm) = &state.network_control {
            let nm_clone = nm.clone();
            let ssid = network.ssid.clone();
            let is_connected = network.connected;
            let button_clone = connect_btn.clone();

            connect_btn.connect_clicked(move |_| {
                let nm = nm_clone.clone();
                let ssid = ssid.clone();
                let button = button_clone.clone();

                button.set_sensitive(false);

                glib::spawn_future_local(async move {
                    let result = if is_connected {
                        nm.disconnect().await
                    } else {
                        // TODO: Show password dialog for secured networks
                        nm.connect(&ssid, None).await
                    };

                    match result {
                        Ok(()) => {
                            info!(
                                "{} network: {}",
                                if is_connected { "Disconnected from" } else { "Connected to" },
                                ssid
                            );
                        }
                        Err(e) => {
                            warn!(
                                "Failed to {} network {}: {}",
                                if is_connected { "disconnect from" } else { "connect to" },
                                ssid,
                                e
                            );
                        }
                    }

                    // Re-enable button
                    button.set_sensitive(true);
                });
            });
        }

        row.append(&signal_label);
        row.append(&network_info);
        row.append(&connect_btn);

        list.append(&row);
    }

    fn signal_to_bars(strength: u8) -> String {
        match strength {
            0..=25 => "▂___".to_string(),
            26..=50 => "▂▄__".to_string(),
            51..=75 => "▂▄▆_".to_string(),
            _ => "▂▄▆█".to_string(),
        }
    }

    pub fn show(&self) {
        // Refresh network list when showing
        if let Some(nm) = &self.state.network_control {
            let network_list = self.network_list.clone();
            let state = self.state.clone();
            let nm_clone = nm.clone();

            glib::spawn_future_local(async move {
                // Trigger scan
                let _ = nm_clone.scan().await;

                // Wait for scan to complete, then get networks
                glib::timeout_add_seconds_local(2, {
                    let nm = nm_clone.clone();
                    let list = network_list.clone();
                    let state = state.clone();
                    move || {
                        let nm = nm.clone();
                        let list = list.clone();
                        let state = state.clone();
                        glib::spawn_future_local(async move {
                            if let Ok(networks) = nm.get_networks().await {
                                Self::update_network_list(&list, &networks, &state);
                            }
                        });
                        glib::ControlFlow::Break
                    }
                });
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

            .network-list {
                background-color: transparent;
            }

            .network-name {
                font-weight: bold;
            }

            .network-status {
                font-size: 10pt;
                color: #a6adc8;
            }

            .signal-strength {
                font-family: monospace;
                color: #89b4fa;
            }

            .empty-message {
                color: #a6adc8;
                font-style: italic;
            }

            button.refresh-button {
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
