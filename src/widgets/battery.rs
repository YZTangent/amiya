use crate::app::AppState;
use crate::events::Event;
use gtk4::prelude::*;
use gtk4::{glib, Box as GtkBox, Label, Orientation};
use std::sync::Arc;

pub struct Battery {
    container: GtkBox,
}

impl Battery {
    pub fn new(state: &Arc<AppState>) -> Self {
        let container = GtkBox::new(Orientation::Horizontal, 4);

        // Battery icon and percentage
        let battery_label = Label::new(Some("🔋 ---%"));
        battery_label.add_css_class("battery-label");
        container.append(&battery_label);

        // Subscribe to events
        Self::subscribe_to_events(state.events.clone(), battery_label.clone());

        // Get initial battery status
        if let Some(battery) = &state.battery_control {
            let battery_clone = battery.clone();
            let label_clone = battery_label.clone();
            glib::spawn_future_local(async move {
                let info = battery_clone.get_info().await;
                if info.is_present {
                    let text = Self::format_battery_text(
                        info.percentage,
                        &info.state.to_string(),
                        matches!(
                            info.state,
                            crate::backend::system::battery::BatteryState::Charging
                        ),
                    );
                    label_clone.set_text(&text);
                }
            });
        }

        Battery { container }
    }

    pub fn widget(&self) -> GtkBox {
        self.container.clone()
    }

    fn format_battery_text(percentage: f64, _state: &str, is_charging: bool) -> String {
        let icon = if is_charging {
            "⚡"
        } else if percentage >= 90.0 {
            "🔋"
        } else if percentage >= 60.0 {
            "🔋"
        } else if percentage >= 30.0 {
            "🔋"
        } else if percentage >= 15.0 {
            "🪫"
        } else {
            "🪫"
        };

        format!("{} {:.0}%", icon, percentage)
    }

    fn subscribe_to_events(events: crate::events::EventManager, battery_label: Label) {
        let mut receiver = events.subscribe();

        // Spawn event listener
        glib::spawn_future_local(async move {
            loop {
                match receiver.recv().await {
                    Ok(event) => match event {
                        Event::BatteryChanged {
                            percentage,
                            state,
                            is_charging,
                        } => {
                            let text = Self::format_battery_text(percentage, &state, is_charging);
                            battery_label.set_text(&text);

                            // Add CSS class based on battery level for styling
                            battery_label.remove_css_class("battery-low");
                            battery_label.remove_css_class("battery-critical");
                            battery_label.remove_css_class("battery-charging");

                            if is_charging {
                                battery_label.add_css_class("battery-charging");
                            } else if percentage < 15.0 {
                                battery_label.add_css_class("battery-critical");
                            } else if percentage < 30.0 {
                                battery_label.add_css_class("battery-low");
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
