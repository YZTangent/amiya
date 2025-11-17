use crate::app::AppState;
use crate::events::Event;
use gtk4::prelude::*;
use gtk4::{glib, Box as GtkBox, Button, Label, Orientation};
use std::collections::HashMap;
use std::sync::Arc;

pub struct Workspaces {
    container: GtkBox,
}

impl Workspaces {
    pub fn new(state: &Arc<AppState>) -> Self {
        let container = GtkBox::new(Orientation::Horizontal, 4);

        // Create workspace buttons (1-9 for now)
        let mut buttons = HashMap::new();

        for i in 1..=9 {
            let button = Button::new();
            let label = Label::new(Some(&i.to_string()));
            button.set_child(Some(&label));
            button.add_css_class("workspace-button");

            if i == 1 {
                button.add_css_class("active");
            }

            // Store workspace ID in button data
            button.set_data("workspace_id", i);

            // Clone for click handler
            let events = state.events.clone();
            button.connect_clicked(move |btn| {
                let workspace_id = btn.data::<u32>("workspace_id").unwrap();
                // In Phase 3, this will send IPC to niri to switch workspace
                // For now, just emit an event
                tracing::info!("Switching to workspace {}", workspace_id);
                events.emit(Event::WorkspaceChanged { id: workspace_id });
            });

            buttons.insert(i, button.clone());
            container.append(&button);
        }

        // Subscribe to workspace events
        Self::subscribe_to_events(state.events.clone(), buttons);

        Workspaces { container }
    }

    pub fn widget(&self) -> GtkBox {
        self.container.clone()
    }

    fn subscribe_to_events(
        events: crate::events::EventManager,
        buttons: HashMap<u32, Button>,
    ) {
        let mut receiver = events.subscribe();

        glib::spawn_future_local(async move {
            loop {
                match receiver.recv().await {
                    Ok(event) => match event {
                        Event::WorkspaceChanged { id } => {
                            // Remove active class from all buttons
                            for button in buttons.values() {
                                button.remove_css_class("active");
                            }

                            // Add active class to the current workspace
                            if let Some(button) = buttons.get(&id) {
                                button.add_css_class("active");
                            }
                        }
                        Event::WorkspacesUpdated { workspaces } => {
                            // Update button visibility based on available workspaces
                            // For now, we'll just update active states
                            for workspace in workspaces {
                                if let Some(button) = buttons.get(&workspace.id) {
                                    if workspace.is_active {
                                        button.add_css_class("active");
                                    } else {
                                        button.remove_css_class("active");
                                    }
                                }
                            }
                        }
                        _ => {} // Ignore other events
                    },
                    Err(_) => {
                        // Channel closed, exit loop
                        break;
                    }
                }
            }
        });
    }
}
