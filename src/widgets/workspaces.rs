use crate::config::Config;
use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Button, Label, Orientation};
use std::sync::{Arc, Mutex};

pub struct Workspaces {
    container: GtkBox,
}

impl Workspaces {
    pub fn new(config: &Config) -> Self {
        let container = GtkBox::new(Orientation::Horizontal, 4);

        // Create workspace buttons (1-9 for now)
        // In a real implementation, this would query niri via IPC
        let active_workspace = Arc::new(Mutex::new(1));

        for i in 1..=9 {
            let button = Button::new();
            let label = Label::new(Some(&i.to_string()));
            button.set_child(Some(&label));
            button.add_css_class("workspace-button");

            if i == 1 {
                button.add_css_class("active");
            }

            let active = active_workspace.clone();
            let btn_clone = button.clone();
            button.connect_clicked(move |_| {
                // In real implementation: send IPC to niri to switch workspace
                let mut current = active.lock().unwrap();
                *current = i;
                println!("Switching to workspace {}", i);

                // Update button styles (simplified - in real app would update all buttons)
                btn_clone.add_css_class("active");
            });

            container.append(&button);
        }

        // Start workspace monitor
        Self::start_workspace_monitor(container.clone(), config.clone());

        Workspaces { container }
    }

    pub fn widget(&self) -> GtkBox {
        self.container.clone()
    }

    fn start_workspace_monitor(container: GtkBox, _config: Config) {
        // Spawn a task to monitor workspace changes from niri
        // This would use niri's IPC in a real implementation
        glib::spawn_future_local(async move {
            // Placeholder for niri IPC connection
            // In reality, you'd connect to niri's socket and listen for events
            loop {
                glib::timeout_future_seconds(1).await;
                // Update workspace display based on niri state
            }
        });
    }
}
