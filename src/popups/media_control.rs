use crate::app::AppState;
use crate::events::Event;
use gtk4::glib;
use gtk4::prelude::*;
use gtk4::{
    Application, ApplicationWindow, Box as GtkBox, Button, Label, Orientation, Scale,
};
use gtk4_layer_shell::{Edge, Layer, LayerShell};
use std::sync::Arc;
use tracing::{debug, info, warn};

pub struct MediaControlPopup {
    window: ApplicationWindow,
    track_name: Label,
    artist_name: Label,
    play_btn: Button,
    volume_scale: Scale,
    state: Arc<AppState>,
}

impl MediaControlPopup {
    pub fn new(app: &Application, state: Arc<AppState>) -> Self {
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
        let track_name = Label::new(Some("No track playing"));
        track_name.add_css_class("track-name");

        let artist_name = Label::new(Some(""));
        artist_name.add_css_class("artist-name");

        // Progress bar (TODO: implement seek functionality)
        let progress = Scale::with_range(gtk4::Orientation::Horizontal, 0.0, 100.0, 1.0);
        progress.set_value(0.0);
        progress.set_draw_value(false);
        progress.add_css_class("progress-bar");
        progress.set_sensitive(false); // Disabled for now

        // Time labels
        let time_box = GtkBox::new(Orientation::Horizontal, 0);
        let current_time = Label::new(Some("0:00"));
        current_time.set_halign(gtk4::Align::Start);
        current_time.set_hexpand(true);

        let total_time = Label::new(Some("0:00"));
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
        volume_scale.set_value(100.0);
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

        // Close on focus loss
        let window_clone = window.clone();
        window.connect_is_active_notify(move |win| {
            if !win.is_active() {
                window_clone.close();
            }
        });

        let popup = MediaControlPopup {
            window,
            track_name: track_name.clone(),
            artist_name: artist_name.clone(),
            play_btn: play_btn.clone(),
            volume_scale: volume_scale.clone(),
            state: state.clone(),
        };

        // Wire up playback controls
        if let Some(media) = &state.media_control {
            // Previous button
            let media_clone = media.clone();
            prev_btn.connect_clicked(move |_| {
                let media = media_clone.clone();
                glib::spawn_future_local(async move {
                    if let Err(e) = media.previous().await {
                        warn!("Failed to play previous track: {}", e);
                    } else {
                        info!("Playing previous track");
                    }
                });
            });

            // Play/Pause button
            let media_clone = media.clone();
            let play_btn_clone = play_btn.clone();
            play_btn.connect_clicked(move |_| {
                let media = media_clone.clone();
                let button = play_btn_clone.clone();

                glib::spawn_future_local(async move {
                    if let Err(e) = media.play_pause().await {
                        warn!("Failed to toggle playback: {}", e);
                    } else {
                        // Update button icon based on new state
                        if let Ok(status) = media.get_playback_status().await {
                            let icon = match status {
                                crate::backend::system::media::PlaybackStatus::Playing => "⏸",
                                _ => "▶",
                            };
                            button.set_label(icon);
                        }
                    }
                });
            });

            // Next button
            let media_clone = media.clone();
            next_btn.connect_clicked(move |_| {
                let media = media_clone.clone();
                glib::spawn_future_local(async move {
                    if let Err(e) = media.next().await {
                        warn!("Failed to play next track: {}", e);
                    } else {
                        info!("Playing next track");
                    }
                });
            });

            // Volume control
            let media_clone = media.clone();
            volume_scale.connect_value_changed(move |scale| {
                let media = media_clone.clone();
                let volume = scale.value() / 100.0; // Convert to 0.0-1.0 range

                glib::spawn_future_local(async move {
                    if let Err(e) = media.set_volume(volume).await {
                        debug!("Failed to set volume: {}", e);
                    }
                });
            });

            // Get initial volume
            let volume_scale_clone = volume_scale.clone();
            let media_clone = media.clone();
            glib::spawn_future_local(async move {
                let volume = media_clone.get_volume().await;
                volume_scale_clone.set_value(volume * 100.0);
            });

            // Get initial playback status
            let play_btn_clone = play_btn.clone();
            let media_clone = media.clone();
            glib::spawn_future_local(async move {
                let status = media_clone.get_playback_status().await;
                let icon = match status {
                    crate::backend::system::media::PlaybackStatus::Playing => "⏸",
                    _ => "▶",
                };
                play_btn_clone.set_label(icon);
            });
        }

        // Subscribe to media events
        let track_name_clone = track_name.clone();
        let artist_name_clone = artist_name.clone();
        let play_btn_clone = play_btn.clone();
        let volume_scale_clone = volume_scale.clone();
        let state_clone = state.clone();
        glib::spawn_future_local(async move {
            let mut receiver = state_clone.events.subscribe();

            loop {
                match receiver.recv().await {
                    Ok(Event::MediaTrackChanged {
                        title,
                        artist,
                        album,
                    }) => {
                        debug!("Track changed: {} - {}", artist, title);
                        track_name_clone.set_text(&title);

                        let artist_text = if let Some(alb) = album {
                            format!("{} • {}", artist, alb)
                        } else {
                            artist
                        };
                        artist_name_clone.set_text(&artist_text);
                    }
                    Ok(Event::MediaPlaybackChanged { playing }) => {
                        debug!("Playback changed: {}", playing);
                        let icon = if playing { "⏸" } else { "▶" };
                        play_btn_clone.set_label(icon);
                    }
                    Ok(Event::MediaVolumeChanged { volume }) => {
                        debug!("Volume changed: {:.2}", volume);
                        volume_scale_clone.set_value(volume * 100.0);
                    }
                    Ok(Event::MediaPlayerChanged { player }) => {
                        if let Some(p) = player {
                            info!("Active player changed: {}", p);
                        }
                    }
                    _ => {}
                }
            }
        });

        // Initial load of metadata
        if let Some(media) = &state.media_control {
            let track_name = track_name.clone();
            let artist_name = artist_name.clone();
            let media_clone = media.clone();
            glib::spawn_future_local(async move {
                if let Some(metadata) = media_clone.get_metadata().await {
                    track_name.set_text(&metadata.title);

                    let artist_text = if let Some(album) = metadata.album {
                        format!("{} • {}", metadata.artist, album)
                    } else {
                        metadata.artist
                    };
                    artist_name.set_text(&artist_text);
                }
            });
        }

        popup
    }

    pub fn show(&self) {
        // Refresh metadata when showing
        if let Some(media) = &self.state.media_control {
            let track_name = self.track_name.clone();
            let artist_name = self.artist_name.clone();
            let play_btn = self.play_btn.clone();
            let volume_scale = self.volume_scale.clone();
            let media_clone = media.clone();

            glib::spawn_future_local(async move {
                // Update metadata
                if let Some(metadata) = media_clone.get_metadata().await {
                    track_name.set_text(&metadata.title);

                    let artist_text = if let Some(album) = metadata.album {
                        format!("{} • {}", metadata.artist, album)
                    } else {
                        metadata.artist
                    };
                    artist_name.set_text(&artist_text);
                } else {
                    track_name.set_text("No track playing");
                    artist_name.set_text("");
                }

                // Update playback button
                let status = media_clone.get_playback_status().await;
                let icon = match status {
                    crate::backend::system::media::PlaybackStatus::Playing => "⏸",
                    _ => "▶",
                };
                play_btn.set_label(icon);

                // Update volume
                let volume = media_clone.get_volume().await;
                volume_scale.set_value(volume * 100.0);
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
