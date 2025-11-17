use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, Box as GtkBox, Label, Orientation, ProgressBar};
use gtk4_layer_shell::{Edge, Layer, LayerShell};
use std::sync::Arc;
use std::time::Duration;

#[derive(Debug, Clone, Copy)]
pub enum SliderType {
    Volume,
    Brightness,
}

pub struct SliderOverlay {
    window: ApplicationWindow,
    progress: ProgressBar,
    label: Label,
    slider_type: SliderType,
}

impl SliderOverlay {
    pub fn new(app: &Application, slider_type: SliderType) -> Self {
        let window = ApplicationWindow::builder()
            .application(app)
            .title(match slider_type {
                SliderType::Volume => "Volume",
                SliderType::Brightness => "Brightness",
            })
            .default_width(300)
            .default_height(100)
            .build();

        // Initialize layer shell for overlay
        window.init_layer_shell();
        window.set_layer(Layer::Overlay);
        window.set_namespace("amiya-slider");

        // Center the overlay
        window.set_anchor(Edge::Top, false);
        window.set_anchor(Edge::Bottom, false);
        window.set_anchor(Edge::Left, false);
        window.set_anchor(Edge::Right, false);

        // Set margin from top
        window.set_margin(Edge::Top, 100);

        // Create container
        let container = GtkBox::new(Orientation::Vertical, 12);
        container.set_margin_start(24);
        container.set_margin_end(24);
        container.set_margin_top(24);
        container.set_margin_bottom(24);

        // Icon and label
        let icon = match slider_type {
            SliderType::Volume => "🔊",
            SliderType::Brightness => "☀️",
        };

        let label = Label::new(Some(&format!(
            "{} {}",
            icon,
            match slider_type {
                SliderType::Volume => "Volume",
                SliderType::Brightness => "Brightness",
            }
        )));
        label.set_halign(gtk4::Align::Center);

        // Progress bar
        let progress = ProgressBar::new();
        progress.set_show_text(true);
        progress.set_hexpand(true);

        container.append(&label);
        container.append(&progress);

        window.set_child(Some(&container));

        // Apply styling
        Self::apply_theme(&window);

        SliderOverlay {
            window,
            progress,
            label,
            slider_type,
        }
    }

    pub fn show(&self, value: f64) {
        self.progress.set_fraction(value / 100.0);
        self.progress.set_text(Some(&format!("{:.0}%", value)));
        self.window.present();

        // Auto-hide after 2 seconds
        let window = self.window.clone();
        glib::timeout_add_seconds_local(2, move || {
            window.close();
            glib::ControlFlow::Break
        });
    }

    pub fn update(&self, value: f64) {
        self.progress.set_fraction(value / 100.0);
        self.progress.set_text(Some(&format!("{:.0}%", value)));
    }

    fn apply_theme(window: &ApplicationWindow) {
        let provider = gtk4::CssProvider::new();
        let css = r#"
            window {
                background-color: rgba(30, 30, 46, 0.95);
                border-radius: 12px;
                color: #cdd6f4;
            }

            progressbar {
                min-height: 20px;
                border-radius: 10px;
            }

            progressbar trough {
                background-color: rgba(255, 255, 255, 0.1);
                border-radius: 10px;
            }

            progressbar progress {
                background-color: #89b4fa;
                border-radius: 10px;
            }

            label {
                font-size: 14pt;
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

// Global overlay manager
use std::sync::Mutex;

lazy_static::lazy_static! {
    static ref VOLUME_OVERLAY: Mutex<Option<Arc<SliderOverlay>>> = Mutex::new(None);
    static ref BRIGHTNESS_OVERLAY: Mutex<Option<Arc<SliderOverlay>>> = Mutex::new(None);
}

pub fn show_volume_overlay(app: &Application, value: f64) {
    let mut overlay = VOLUME_OVERLAY.lock().unwrap();
    if overlay.is_none() {
        *overlay = Some(Arc::new(SliderOverlay::new(app, SliderType::Volume)));
    }

    if let Some(o) = overlay.as_ref() {
        o.show(value);
    }
}

pub fn show_brightness_overlay(app: &Application, value: f64) {
    let mut overlay = BRIGHTNESS_OVERLAY.lock().unwrap();
    if overlay.is_none() {
        *overlay = Some(Arc::new(SliderOverlay::new(app, SliderType::Brightness)));
    }

    if let Some(o) = overlay.as_ref() {
        o.show(value);
    }
}
