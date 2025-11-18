use crate::app::AppState;
use crate::backend::system::power::PowerAction;
use gtk4::glib;
use gtk4::prelude::*;
use gtk4::{
    Application, ApplicationWindow, Box as GtkBox, Button, Label, Orientation, Separator,
};
use gtk4_layer_shell::{Edge, Layer, LayerShell};
use std::sync::Arc;
use tracing::{info, warn};

pub struct PowerPopup {
    window: ApplicationWindow,
    state: Arc<AppState>,
}

impl PowerPopup {
    pub fn new(app: &Application, state: Arc<AppState>) -> Self {
        let window = ApplicationWindow::builder()
            .application(app)
            .title("Power Menu")
            .default_width(300)
            .default_height(400)
            .build();

        // Initialize layer shell
        window.init_layer_shell();
        window.set_layer(Layer::Overlay);
        window.set_namespace("amiya-power");

        // Position in center
        window.set_anchor(Edge::Top, false);
        window.set_anchor(Edge::Bottom, false);
        window.set_anchor(Edge::Left, false);
        window.set_anchor(Edge::Right, false);

        // Create main container
        let container = GtkBox::new(Orientation::Vertical, 16);
        container.set_margin_start(24);
        container.set_margin_end(24);
        container.set_margin_top(24);
        container.set_margin_bottom(24);

        // Header
        let header = Label::new(Some("⚡ Power Menu"));
        header.add_css_class("power-menu-header");
        header.set_halign(gtk4::Align::Center);
        container.append(&header);

        let separator1 = Separator::new(Orientation::Horizontal);
        container.append(&separator1);

        // Lock button
        let lock_button = Button::with_label("🔒 Lock");
        lock_button.add_css_class("power-button");
        lock_button.add_css_class("lock-button");

        // Suspend button
        let suspend_button = Button::with_label("🌙 Suspend");
        suspend_button.add_css_class("power-button");
        suspend_button.add_css_class("suspend-button");

        // Hibernate button
        let hibernate_button = Button::with_label("💤 Hibernate");
        hibernate_button.add_css_class("power-button");
        hibernate_button.add_css_class("hibernate-button");

        let separator2 = Separator::new(Orientation::Horizontal);

        // Reboot button
        let reboot_button = Button::with_label("🔄 Reboot");
        reboot_button.add_css_class("power-button");
        reboot_button.add_css_class("reboot-button");

        // Shutdown button
        let shutdown_button = Button::with_label("⏻ Shutdown");
        shutdown_button.add_css_class("power-button");
        shutdown_button.add_css_class("shutdown-button");

        // Add buttons to container
        container.append(&lock_button);
        container.append(&suspend_button);
        container.append(&hibernate_button);
        container.append(&separator2);
        container.append(&reboot_button);
        container.append(&shutdown_button);

        // Cancel button
        let cancel_button = Button::with_label("Cancel");
        cancel_button.add_css_class("cancel-button");
        container.append(&cancel_button);

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

        let popup = PowerPopup {
            window: window.clone(),
            state: state.clone(),
        };

        // Wire up buttons
        if let Some(power) = &state.power_control {
            // Lock
            let power_clone = power.clone();
            let window_clone = window.clone();
            lock_button.connect_clicked(move |_| {
                let power = power_clone.clone();
                let window = window_clone.clone();
                glib::spawn_future_local(async move {
                    info!("Locking screen...");
                    if let Err(e) = power.execute(PowerAction::Lock).await {
                        warn!("Failed to lock screen: {}", e);
                    }
                    window.close();
                });
            });

            // Suspend
            let power_clone = power.clone();
            let window_clone = window.clone();
            suspend_button.connect_clicked(move |_| {
                let power = power_clone.clone();
                let window = window_clone.clone();
                glib::spawn_future_local(async move {
                    info!("Suspending system...");
                    if let Err(e) = power.execute(PowerAction::Suspend).await {
                        warn!("Failed to suspend: {}", e);
                    }
                    window.close();
                });
            });

            // Hibernate
            let power_clone = power.clone();
            let window_clone = window.clone();
            hibernate_button.connect_clicked(move |_| {
                let power = power_clone.clone();
                let window = window_clone.clone();
                glib::spawn_future_local(async move {
                    info!("Hibernating system...");
                    if let Err(e) = power.execute(PowerAction::Hibernate).await {
                        warn!("Failed to hibernate: {}", e);
                    }
                    window.close();
                });
            });

            // Reboot
            let power_clone = power.clone();
            let window_clone = window.clone();
            reboot_button.connect_clicked(move |_| {
                let power = power_clone.clone();
                let window = window_clone.clone();
                glib::spawn_future_local(async move {
                    info!("Rebooting system...");
                    if let Err(e) = power.execute(PowerAction::Reboot).await {
                        warn!("Failed to reboot: {}", e);
                    }
                    window.close();
                });
            });

            // Shutdown
            let power_clone = power.clone();
            let window_clone = window.clone();
            shutdown_button.connect_clicked(move |_| {
                let power = power_clone.clone();
                let window = window_clone.clone();
                glib::spawn_future_local(async move {
                    info!("Shutting down system...");
                    if let Err(e) = power.execute(PowerAction::Shutdown).await {
                        warn!("Failed to shutdown: {}", e);
                    }
                    window.close();
                });
            });
        }

        // Cancel button closes the popup
        let window_clone = window.clone();
        cancel_button.connect_clicked(move |_| {
            window_clone.close();
        });

        popup
    }

    pub fn show(&self) {
        self.window.present();
    }

    pub fn hide(&self) {
        self.window.close();
    }

    pub fn toggle(&self) {
        if self.window.is_visible() {
            self.hide();
        } else {
            self.show();
        }
    }

    fn apply_theme(window: &ApplicationWindow) {
        let provider = gtk4::CssProvider::new();
        let css = r#"
        window {
            background-color: #1e1e2e;
            color: #cdd6f4;
            border-radius: 12px;
        }

        .power-menu-header {
            font-size: 18pt;
            font-weight: bold;
            margin-bottom: 8px;
        }

        .power-button {
            min-height: 48px;
            font-size: 14pt;
            border-radius: 8px;
            border: 2px solid transparent;
            background-color: #313244;
            color: #cdd6f4;
            padding: 8px 16px;
        }

        .power-button:hover {
            background-color: #45475a;
            border-color: #89b4fa;
        }

        .power-button:active {
            background-color: #585b70;
        }

        .lock-button:hover {
            border-color: #89dceb;
        }

        .suspend-button:hover {
            border-color: #89b4fa;
        }

        .hibernate-button:hover {
            border-color: #b4befe;
        }

        .reboot-button:hover {
            border-color: #f9e2af;
        }

        .shutdown-button:hover {
            border-color: #f38ba8;
        }

        .cancel-button {
            min-height: 40px;
            font-size: 12pt;
            border-radius: 8px;
            background-color: #45475a;
            color: #cdd6f4;
            margin-top: 8px;
        }

        .cancel-button:hover {
            background-color: #585b70;
        }

        separator {
            background-color: #45475a;
            min-height: 1px;
            margin: 8px 0;
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
