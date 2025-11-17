use crate::app::AppState;
use chrono::Local;
use gtk4::prelude::*;
use gtk4::{glib, Label};
use std::sync::Arc;

pub struct Clock {
    label: Label,
}

impl Clock {
    pub fn new(_state: &Arc<AppState>) -> Self {
        let label = Label::new(None);
        label.add_css_class("clock-label");

        // Update clock immediately
        Self::update_time(&label);

        // Update every second
        let label_clone = label.clone();
        glib::timeout_add_seconds_local(1, move || {
            Self::update_time(&label_clone);
            glib::ControlFlow::Continue
        });

        Clock { label }
    }

    fn update_time(label: &Label) {
        let now = Local::now();
        let time_str = now.format("%a %b %d  %H:%M:%S").to_string();
        label.set_text(&time_str);
    }

    pub fn widget(&self) -> Label {
        self.label.clone()
    }
}
