mod bar;
mod config;
mod ipc;
mod overlays;
mod popups;
mod system;
mod widgets;

use anyhow::Result;
use gtk4::prelude::*;
use gtk4::{glib, Application};
use tracing::{info, Level};
use tracing_subscriber;

const APP_ID: &str = "com.amiya.desktop";

fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    info!("Starting Amiya Desktop Environment");

    // Load configuration
    let config = config::Config::load()?;

    // Create GTK application
    let app = Application::builder().application_id(APP_ID).build();

    app.connect_activate(move |app| {
        if let Err(e) = activate(app, config.clone()) {
            eprintln!("Error activating application: {}", e);
        }
    });

    // Run the application
    app.run();

    Ok(())
}

fn activate(app: &Application, config: config::Config) -> Result<()> {
    info!("Activating Amiya");

    // Initialize the bar
    let bar = bar::Bar::new(app, &config)?;
    bar.show();

    // Set up hotkey listener in a separate thread
    let hotkey_config = config.clone();
    std::thread::spawn(move || {
        if let Err(e) = setup_hotkeys(hotkey_config) {
            eprintln!("Hotkey setup error: {}", e);
        }
    });

    Ok(())
}

fn setup_hotkeys(config: config::Config) -> Result<()> {
    // This will listen for hotkey events
    // In a real implementation, this would integrate with the compositor
    // or use something like keyd/swhkd
    info!("Hotkey system initialized");
    info!("Configured hotkeys: {:?}", config.hotkeys);

    // For now, we'll just log that hotkeys are configured
    // In a real implementation, you'd use something like:
    // - Direct Wayland protocol extension
    // - External hotkey daemon (swhkd)
    // - Niri's own hotkey system via IPC

    Ok(())
}
