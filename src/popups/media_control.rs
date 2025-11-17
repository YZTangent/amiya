use gtk4::prelude::*;
use gtk4::{
    Application, ApplicationWindow, Box as GtkBox, Button, Label, Orientation, Scale,
};
use gtk4_layer_shell::{Edge, Layer, LayerShell};

pub struct MediaControlPopup {
    window: ApplicationWindow,
}

impl MediaControlPopup {
    pub fn new(app: &Application) -> Self {
        let window = ApplicationWindow::builder()
            .application(app)
            .title("Media Control")
            .default_width(400)
            .default_height(300)
            .build();

        // Initialize layer shell
        window.init_layer_shell();
        window.set_layer(Layer::Overlay);
        window.set_namespace("amiya-media");

        // Position in top-right
        window.set_anchor(Edge::Top, true);
        window.set_anchor(Edge::Right, true);
        window.set_margin(Edge::Top, 40);
        window.set_margin(Edge::Right, 10);

        // Create main container
        let container = GtkBox::new(Orientation::Vertical, 16);
        container.set_margin_start(24);
        container.set_margin_end(24);
        container.set_margin_top(24);
        container.set_margin_bottom(24);

        // Title
        let title = Label::new(Some("🎵 Now Playing"));
        title.add_css_class("title");

        // Album art placeholder
        let album_art = Label::new(Some("🎵"));
        album_art.add_css_class("album-art");

        // Track info
        let track_name = Label::new(Some("Track Name"));
        track_name.add_css_class("track-name");

        let artist_name = Label::new(Some("Artist Name"));
        artist_name.add_css_class("artist-name");

        // Progress bar
        let progress = Scale::with_range(gtk4::Orientation::Horizontal, 0.0, 100.0, 1.0);
        progress.set_value(45.0);
        progress.set_draw_value(false);
        progress.add_css_class("progress-bar");

        // Time labels
        let time_box = GtkBox::new(Orientation::Horizontal, 0);
        let current_time = Label::new(Some("1:23"));
        current_time.set_halign(gtk4::Align::Start);
        current_time.set_hexpand(true);

        let total_time = Label::new(Some("3:45"));
        total_time.set_halign(gtk4::Align::End);

        time_box.append(&current_time);
        time_box.append(&total_time);

        // Control buttons
        let controls = GtkBox::new(Orientation::Horizontal, 16);
        controls.set_halign(gtk4::Align::Center);

        let prev_btn = Button::with_label("⏮");
        prev_btn.add_css_class("control-button");

        let play_btn = Button::with_label("⏸");
        play_btn.add_css_class("control-button");
        play_btn.add_css_class("play-button");

        let next_btn = Button::with_label("⏭");
        next_btn.add_css_class("control-button");

        controls.append(&prev_btn);
        controls.append(&play_btn);
        controls.append(&next_btn);

        // Volume control
        let volume_box = GtkBox::new(Orientation::Horizontal, 8);
        let volume_icon = Label::new(Some("🔊"));
        let volume_scale = Scale::with_range(gtk4::Orientation::Horizontal, 0.0, 100.0, 1.0);
        volume_scale.set_value(75.0);
        volume_scale.set_draw_value(false);
        volume_scale.set_hexpand(true);

        volume_box.append(&volume_icon);
        volume_box.append(&volume_scale);

        // Add all elements
        container.append(&title);
        container.append(&album_art);
        container.append(&track_name);
        container.append(&artist_name);
        container.append(&progress);
        container.append(&time_box);
        container.append(&controls);
        container.append(&volume_box);

        window.set_child(Some(&container));

        // Apply theme
        Self::apply_theme(&window);

        MediaControlPopup { window }
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

            .title {
                font-size: 14pt;
                font-weight: bold;
            }

            .album-art {
                font-size: 72pt;
                margin: 16px;
            }

            .track-name {
                font-size: 16pt;
                font-weight: bold;
            }

            .artist-name {
                font-size: 12pt;
                color: #a6adc8;
            }

            .control-button {
                min-width: 48px;
                min-height: 48px;
                font-size: 18pt;
                border-radius: 24px;
                background-color: rgba(137, 180, 250, 0.2);
            }

            .control-button.play-button {
                background-color: #89b4fa;
                color: #1e1e2e;
            }

            scale trough {
                min-height: 6px;
                background-color: rgba(255, 255, 255, 0.1);
                border-radius: 3px;
            }

            scale highlight {
                background-color: #89b4fa;
                border-radius: 3px;
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
