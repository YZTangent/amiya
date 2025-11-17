use gtk4::prelude::*;
use gtk4::{
    Application, ApplicationWindow, Box as GtkBox, Button, Label, ListBox, Orientation,
    ScrolledWindow, Switch,
};
use gtk4_layer_shell::{Edge, Layer, LayerShell};

pub struct BluetoothPopup {
    window: ApplicationWindow,
}

impl BluetoothPopup {
    pub fn new(app: &Application) -> Self {
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
        toggle.set_active(true);
        toggle.set_valign(gtk4::Align::Center);

        header.append(&title);
        header.append(&toggle);

        // Device list
        let scrolled = ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .min_content_height(300)
            .build();

        let device_list = ListBox::new();
        device_list.add_css_class("device-list");

        // Add some example devices
        Self::add_device(&device_list, "Headphones", "Connected", true);
        Self::add_device(&device_list, "Keyboard", "Paired", false);
        Self::add_device(&device_list, "Mouse", "Available", false);

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
        window.connect_is_active_notify(move |_| {
            // In real implementation, close when focus is lost
        });

        BluetoothPopup { window }
    }

    fn add_device(list: &ListBox, name: &str, status: &str, connected: bool) {
        let row = GtkBox::new(Orientation::Horizontal, 12);
        row.set_margin_start(8);
        row.set_margin_end(8);
        row.set_margin_top(8);
        row.set_margin_bottom(8);

        let device_info = GtkBox::new(Orientation::Vertical, 4);
        let name_label = Label::new(Some(name));
        name_label.set_halign(gtk4::Align::Start);
        name_label.add_css_class("device-name");

        let status_label = Label::new(Some(status));
        status_label.set_halign(gtk4::Align::Start);
        status_label.add_css_class("device-status");

        device_info.append(&name_label);
        device_info.append(&status_label);
        device_info.set_hexpand(true);

        let connect_btn = Button::with_label(if connected { "Disconnect" } else { "Connect" });
        connect_btn.set_valign(gtk4::Align::Center);

        row.append(&device_info);
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
