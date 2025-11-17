use gtk4::prelude::*;
use gtk4::{
    Application, ApplicationWindow, Box as GtkBox, Button, Entry, Label, ListBox, Orientation,
    ScrolledWindow, Switch,
};
use gtk4_layer_shell::{Edge, Layer, LayerShell};

pub struct WifiPopup {
    window: ApplicationWindow,
}

impl WifiPopup {
    pub fn new(app: &Application) -> Self {
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
        toggle.set_active(true);
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

        // Add some example networks
        Self::add_network(&network_list, "Home Network", "▂▄▆█", true, true);
        Self::add_network(&network_list, "Office WiFi", "▂▄▆_", false, true);
        Self::add_network(&network_list, "Guest Network", "▂▄__", false, false);
        Self::add_network(&network_list, "Coffee Shop", "▂___", false, true);

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

        WifiPopup { window }
    }

    fn add_network(
        list: &ListBox,
        name: &str,
        signal: &str,
        connected: bool,
        secured: bool,
    ) {
        let row = GtkBox::new(Orientation::Horizontal, 12);
        row.set_margin_start(8);
        row.set_margin_end(8);
        row.set_margin_top(8);
        row.set_margin_bottom(8);

        // Signal strength
        let signal_label = Label::new(Some(signal));
        signal_label.add_css_class("signal-strength");

        // Network info
        let network_info = GtkBox::new(Orientation::Vertical, 4);
        let name_label = Label::new(Some(name));
        name_label.set_halign(gtk4::Align::Start);
        name_label.add_css_class("network-name");

        let status = if connected {
            "Connected"
        } else if secured {
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
        if secured {
            let lock_label = Label::new(Some("🔒"));
            row.append(&lock_label);
        }

        let connect_btn = Button::with_label(if connected { "Disconnect" } else { "Connect" });
        connect_btn.set_valign(gtk4::Align::Center);

        row.append(&signal_label);
        row.append(&network_info);
        row.append(&connect_btn);

        list.append(&row);
    }

    pub fn show(&self) {
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
