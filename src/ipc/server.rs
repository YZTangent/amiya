use crate::app::AppState;
use crate::error::{AmiyaError, Result};
use crate::events::Event;
use crate::ipc::protocol::{BrightnessAction, Command, PopupType, Response, VolumeAction};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixListener as TokioUnixListener;
use tracing::{debug, error, info, warn};

const DEFAULT_VOLUME_STEP: f64 = 5.0;
const DEFAULT_BRIGHTNESS_STEP: f64 = 5.0;

pub struct IpcServer {
    socket_path: PathBuf,
    state: Arc<AppState>,
    start_time: SystemTime,
}

impl IpcServer {
    /// Create a new IPC server
    pub fn new(state: Arc<AppState>) -> Result<Self> {
        let socket_path = Self::get_socket_path()?;

        // Remove old socket if it exists
        if socket_path.exists() {
            std::fs::remove_file(&socket_path)
                .map_err(|e| AmiyaError::Ipc(format!("Failed to remove old socket: {}", e)))?;
        }

        Ok(IpcServer {
            socket_path,
            state,
            start_time: SystemTime::now(),
        })
    }

    /// Get the Unix socket path
    fn get_socket_path() -> Result<PathBuf> {
        let runtime_dir = std::env::var("XDG_RUNTIME_DIR")
            .or_else(|_| std::env::var("TMPDIR"))
            .unwrap_or_else(|_| "/tmp".to_string());

        let socket_path = PathBuf::from(runtime_dir).join("amiya").join("amiya.sock");

        // Create parent directory if it doesn't exist
        if let Some(parent) = socket_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                AmiyaError::Ipc(format!("Failed to create socket directory: {}", e))
            })?;
        }

        Ok(socket_path)
    }

    /// Start the IPC server
    pub async fn start(self: Arc<Self>) -> Result<()> {
        let listener = TokioUnixListener::bind(&self.socket_path).map_err(|e| {
            AmiyaError::Ipc(format!("Failed to bind to socket: {}", e))
        })?;

        info!("IPC server listening on {:?}", self.socket_path);

        loop {
            match listener.accept().await {
                Ok((stream, _addr)) => {
                    let server = self.clone();
                    tokio::spawn(async move {
                        if let Err(e) = server.handle_client(stream).await {
                            error!("Error handling client: {}", e);
                        }
                    });
                }
                Err(e) => {
                    error!("Failed to accept connection: {}", e);
                }
            }
        }
    }

    /// Handle a client connection
    async fn handle_client(&self, stream: tokio::net::UnixStream) -> Result<()> {
        let (reader, mut writer) = stream.into_split();
        let mut reader = BufReader::new(reader);
        let mut line = String::new();

        match reader.read_line(&mut line).await {
            Ok(0) => {
                debug!("Client disconnected");
                return Ok(());
            }
            Ok(_) => {
                let trimmed = line.trim();
                debug!("Received command: {}", trimmed);

                let response = match serde_json::from_str::<Command>(trimmed) {
                    Ok(command) => self.handle_command(command).await,
                    Err(e) => Response::error(format!("Invalid command: {}", e)),
                };

                // Send response
                let response_json = serde_json::to_string(&response)
                    .map_err(|e| AmiyaError::Ipc(format!("Failed to serialize response: {}", e)))?;

                writer
                    .write_all(response_json.as_bytes())
                    .await
                    .map_err(|e| AmiyaError::Ipc(format!("Failed to write response: {}", e)))?;

                writer
                    .write_all(b"\n")
                    .await
                    .map_err(|e| AmiyaError::Ipc(format!("Failed to write newline: {}", e)))?;

                writer
                    .flush()
                    .await
                    .map_err(|e| AmiyaError::Ipc(format!("Failed to flush: {}", e)))?;
            }
            Err(e) => {
                error!("Error reading from client: {}", e);
                return Err(AmiyaError::Ipc(format!("Read error: {}", e)));
            }
        }

        Ok(())
    }

    /// Handle a command
    async fn handle_command(&self, command: Command) -> Response {
        match command {
            Command::ShowPopup { popup } => self.handle_show_popup(popup).await,
            Command::HidePopup { popup } => self.handle_hide_popup(popup).await,
            Command::TogglePopup { popup } => self.handle_toggle_popup(popup).await,
            Command::Volume { action } => self.handle_volume(action).await,
            Command::Brightness { action } => self.handle_brightness(action).await,
            Command::Status => self.handle_status().await,
            Command::Ping => Response::pong(),
        }
    }

    /// Handle show popup command
    async fn handle_show_popup(&self, popup: PopupType) -> Response {
        info!("Showing popup: {:?}", popup);

        let event = Event::PopupRequested { popup_type: popup };
        self.state.events.emit(event);

        Response::success_with_message(format!("Showing {:?} popup", popup))
    }

    /// Handle hide popup command
    async fn handle_hide_popup(&self, popup: PopupType) -> Response {
        info!("Hiding popup: {:?}", popup);

        let event = Event::PopupClosed { popup_type: popup };
        self.state.events.emit(event);

        Response::success_with_message(format!("Hiding {:?} popup", popup))
    }

    /// Handle toggle popup command
    async fn handle_toggle_popup(&self, popup: PopupType) -> Response {
        info!("Toggling popup: {:?}", popup);

        // For now, just emit show event
        // TODO: Track popup state and actually toggle
        let event = Event::PopupRequested { popup_type: popup };
        self.state.events.emit(event);

        Response::success_with_message(format!("Toggling {:?} popup", popup))
    }

    /// Handle volume command
    async fn handle_volume(&self, action: VolumeAction) -> Response {
        if let Some(audio) = &self.state.audio_control {
            let result = match action {
                VolumeAction::Up { amount } => {
                    let step = amount.unwrap_or(DEFAULT_VOLUME_STEP);
                    audio.increase_volume(step).await
                }
                VolumeAction::Down { amount } => {
                    let step = amount.unwrap_or(DEFAULT_VOLUME_STEP);
                    audio.decrease_volume(step).await
                }
                VolumeAction::Set { level } => audio.set_volume(level).await,
                VolumeAction::Mute => audio.set_mute(true).await,
                VolumeAction::Unmute => audio.set_mute(false).await,
                VolumeAction::ToggleMute => audio.toggle_mute().await,
            };

            match result {
                Ok(()) => Response::success_with_message("Volume adjusted".to_string()),
                Err(e) => Response::error(format!("Failed to adjust volume: {}", e)),
            }
        } else {
            Response::error("Audio control not available".to_string())
        }
    }

    /// Handle brightness command
    async fn handle_brightness(&self, action: BrightnessAction) -> Response {
        if let Some(backlight) = &self.state.backlight_control {
            let result = match action {
                BrightnessAction::Up { amount } => {
                    let step = amount.unwrap_or(DEFAULT_BRIGHTNESS_STEP);
                    backlight.increase_brightness(step).await
                }
                BrightnessAction::Down { amount } => {
                    let step = amount.unwrap_or(DEFAULT_BRIGHTNESS_STEP);
                    backlight.decrease_brightness(step).await
                }
                BrightnessAction::Set { level } => backlight.set_brightness(level).await,
            };

            match result {
                Ok(()) => Response::success_with_message("Brightness adjusted".to_string()),
                Err(e) => Response::error(format!("Failed to adjust brightness: {}", e)),
            }
        } else {
            Response::error("Backlight control not available".to_string())
        }
    }

    /// Handle status command
    async fn handle_status(&self) -> Response {
        let uptime = self
            .start_time
            .elapsed()
            .unwrap_or_default()
            .as_secs();

        Response::Status {
            version: env!("CARGO_PKG_VERSION").to_string(),
            uptime,
        }
    }
}

impl Drop for IpcServer {
    fn drop(&mut self) {
        // Clean up socket file
        if self.socket_path.exists() {
            if let Err(e) = std::fs::remove_file(&self.socket_path) {
                warn!("Failed to remove socket file: {}", e);
            }
        }
    }
}
