mod app;
mod bar;
mod config;
mod events;
mod ipc;
mod overlays;
mod popups;
mod system;
mod widgets;

use anyhow::Result;
use gtk4::prelude::*;
use gtk4::Application as GtkApplication;
use std::sync::Arc;
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

    // Create application state
    let amiya_app = app::Application::new(config.clone());

    // Initialize subsystems
    amiya_app.initialize()?;

    // Create GTK application
    let gtk_app = GtkApplication::builder().application_id(APP_ID).build();

    // Clone app state for the activate closure
    let app_state = amiya_app.state().clone();
    gtk_app.connect_activate(move |gtk_app| {
        if let Err(e) = activate(gtk_app, app_state.clone()) {
            eprintln!("Error activating application: {}", e);
        }
    });

    // Set up shutdown handler
    let app_state_shutdown = amiya_app.state().clone();
    gtk_app.connect_shutdown(move |_| {
        info!("GTK application shutting down");
        // Cleanup will be handled by Drop impls
    });

    // Run the application
    gtk_app.run();

    // Graceful shutdown
    amiya_app.shutdown();

    Ok(())
}

fn activate(gtk_app: &GtkApplication, app_state: Arc<app::AppState>) -> Result<()> {
    info!("Activating Amiya");

    // Initialize the bar with event manager
    let bar = bar::Bar::new(gtk_app, &app_state)?;
    bar.show();

    Ok(())
}
