use crate::config::{Config, Position};
use crate::widgets::{clock::Clock, system_info::SystemInfo, workspaces::Workspaces};
use anyhow::Result;
use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, Box as GtkBox, Orientation};
use gtk4_layer_shell::{Edge, Layer, LayerShell};

pub struct Bar {
    window: ApplicationWindow,
}

impl Bar {
    pub fn new(app: &Application, config: &Config) -> Result<Self> {
        let window = ApplicationWindow::builder()
            .application(app)
            .title("Amiya Bar")
            .build();

        // Initialize layer shell
        window.init_layer_shell();
        window.set_layer(Layer::Top);
        window.set_namespace("amiya-bar");

        // Anchor to edges
        window.set_anchor(Edge::Left, true);
        window.set_anchor(Edge::Right, true);

        match config.bar.position {
            Position::Top => {
                window.set_anchor(Edge::Top, true);
                window.set_anchor(Edge::Bottom, false);
            }
            Position::Bottom => {
                window.set_anchor(Edge::Bottom, true);
                window.set_anchor(Edge::Top, false);
            }
        }

        // Set exclusive zone (reserves space)
        window.set_exclusive_zone(config.bar.height);

        // Main container
        let main_box = GtkBox::new(Orientation::Horizontal, 0);
        main_box.set_hexpand(true);
        main_box.set_vexpand(true);

        // Apply theme
        apply_theme(&window, config);

        // Left section: Workspaces
        let left_box = GtkBox::new(Orientation::Horizontal, 8);
        left_box.set_margin_start(12);
        left_box.set_margin_end(12);
        left_box.set_margin_top(4);
        left_box.set_margin_bottom(4);

        if config.bar.show_workspaces {
            let workspaces = Workspaces::new(config);
            left_box.append(&workspaces.widget());
        }

        // Center section: Clock
        let center_box = GtkBox::new(Orientation::Horizontal, 0);
        center_box.set_halign(gtk4::Align::Center);
        center_box.set_hexpand(true);

        if config.bar.show_clock {
            let clock = Clock::new(config);
            center_box.append(&clock.widget());
        }

        // Right section: System info
        let right_box = GtkBox::new(Orientation::Horizontal, 12);
        right_box.set_margin_start(12);
        right_box.set_margin_end(12);
        right_box.set_margin_top(4);
        right_box.set_margin_bottom(4);
        right_box.set_halign(gtk4::Align::End);

        if config.bar.show_system_info {
            let system_info = SystemInfo::new(config);
            right_box.append(&system_info.widget());
        }

        // Add all sections to main box
        main_box.append(&left_box);
        main_box.append(&center_box);
        main_box.append(&right_box);

        window.set_child(Some(&main_box));

        Ok(Bar { window })
    }

    pub fn show(&self) {
        self.window.present();
    }
}

fn apply_theme(window: &ApplicationWindow, config: &Config) {
    let provider = gtk4::CssProvider::new();
    let css = format!(
        r#"
        window {{
            background-color: {};
            color: {};
            font-family: "{}";
            font-size: {}pt;
        }}

        .workspace-button {{
            background-color: transparent;
            border: 2px solid transparent;
            border-radius: 4px;
            padding: 4px 12px;
            margin: 0 2px;
            min-width: 30px;
            color: {};
        }}

        .workspace-button:hover {{
            background-color: alpha({}, 0.1);
        }}

        .workspace-button.active {{
            background-color: {};
            border-color: {};
            color: {};
        }}

        .system-info-label {{
            padding: 2px 8px;
            margin: 0 2px;
        }}

        .clock-label {{
            font-size: {}pt;
            font-weight: bold;
        }}

        .icon {{
            margin-right: 4px;
        }}
        "#,
        config.theme.background,
        config.theme.foreground,
        config.theme.font,
        config.theme.font_size,
        config.theme.foreground,
        config.theme.accent,
        config.theme.accent,
        config.theme.accent,
        config.theme.background,
        config.theme.font_size + 1,
    );

    provider.load_from_string(&css);

    gtk4::style_context_add_provider_for_display(
        &window.display(),
        &provider,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}
