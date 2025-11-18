use crate::app::AppState;
use crate::events::{Event, EventManager, PopupType};
use crate::popups::{BluetoothPopup, MediaControlPopup, PowerPopup, WifiPopup};
use gtk4::glib;
use gtk4::Application;
use std::sync::Arc;
use std::sync::Mutex;

pub struct PopupManager {
    bluetooth_popup: Arc<Mutex<Option<BluetoothPopup>>>,
    wifi_popup: Arc<Mutex<Option<WifiPopup>>>,
    media_control_popup: Arc<Mutex<Option<MediaControlPopup>>>,
    power_popup: Arc<Mutex<Option<PowerPopup>>>,
    app: Application,
    state: Arc<AppState>,
}

impl PopupManager {
    pub fn new(app: &Application, state: Arc<AppState>) -> Self {
        let manager = PopupManager {
            bluetooth_popup: Arc::new(Mutex::new(None)),
            wifi_popup: Arc::new(Mutex::new(None)),
            media_control_popup: Arc::new(Mutex::new(None)),
            power_popup: Arc::new(Mutex::new(None)),
            app: app.clone(),
            state: state.clone(),
        };

        // Subscribe to events
        Self::subscribe_to_events(
            state.events.clone(),
            manager.bluetooth_popup.clone(),
            manager.wifi_popup.clone(),
            manager.media_control_popup.clone(),
            manager.power_popup.clone(),
            app.clone(),
            state.clone(),
        );

        manager
    }

    fn subscribe_to_events(
        events: EventManager,
        bluetooth_popup: Arc<Mutex<Option<BluetoothPopup>>>,
        wifi_popup: Arc<Mutex<Option<WifiPopup>>>,
        media_control_popup: Arc<Mutex<Option<MediaControlPopup>>>,
        power_popup: Arc<Mutex<Option<PowerPopup>>>,
        app: Application,
        state: Arc<AppState>,
    ) {
        let mut receiver = events.subscribe();

        glib::spawn_future_local(async move {
            loop {
                match receiver.recv().await {
                    Ok(event) => match event {
                        Event::PopupRequested { popup_type } => {
                            match popup_type {
                                PopupType::Bluetooth => {
                                    let mut popup_guard = bluetooth_popup.lock().unwrap();
                                    if popup_guard.is_none() {
                                        *popup_guard =
                                            Some(BluetoothPopup::new(&app, state.clone()));
                                    }
                                    if let Some(popup) = popup_guard.as_ref() {
                                        popup.show();
                                    }
                                }
                                PopupType::Wifi => {
                                    let mut popup_guard = wifi_popup.lock().unwrap();
                                    if popup_guard.is_none() {
                                        *popup_guard = Some(WifiPopup::new(&app, state.clone()));
                                    }
                                    if let Some(popup) = popup_guard.as_ref() {
                                        popup.show();
                                    }
                                }
                                PopupType::MediaControl => {
                                    let mut popup_guard = media_control_popup.lock().unwrap();
                                    if popup_guard.is_none() {
                                        *popup_guard =
                                            Some(MediaControlPopup::new(&app, state.clone()));
                                    }
                                    if let Some(popup) = popup_guard.as_ref() {
                                        popup.show();
                                    }
                                }
                                PopupType::Power => {
                                    let mut popup_guard = power_popup.lock().unwrap();
                                    if popup_guard.is_none() {
                                        *popup_guard = Some(PowerPopup::new(&app, state.clone()));
                                    }
                                    if let Some(popup) = popup_guard.as_ref() {
                                        popup.show();
                                    }
                                }
                            }
                        }
                        Event::PopupClosed { popup_type } => {
                            match popup_type {
                                PopupType::Bluetooth => {
                                    if let Some(popup) = bluetooth_popup.lock().unwrap().as_ref() {
                                        popup.hide();
                                    }
                                }
                                PopupType::Wifi => {
                                    if let Some(popup) = wifi_popup.lock().unwrap().as_ref() {
                                        popup.hide();
                                    }
                                }
                                PopupType::MediaControl => {
                                    if let Some(popup) =
                                        media_control_popup.lock().unwrap().as_ref()
                                    {
                                        popup.hide();
                                    }
                                }
                                PopupType::Power => {
                                    if let Some(popup) = power_popup.lock().unwrap().as_ref() {
                                        popup.hide();
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
