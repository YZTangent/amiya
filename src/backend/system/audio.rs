use crate::error::{AmiyaError, Result};
use crate::events::{Event, EventManager};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use zbus::Connection;

/// Audio control via PulseAudio/PipeWire
pub struct AudioControl {
    connection: Arc<RwLock<Option<Connection>>>,
    current_volume: Arc<RwLock<f64>>,
    current_mute: Arc<RwLock<bool>>,
    events: Option<EventManager>,
}

impl AudioControl {
    /// Create a new audio control instance
    pub fn new() -> Self {
        AudioControl {
            connection: Arc::new(RwLock::new(None)),
            current_volume: Arc::new(RwLock::new(50.0)),
            current_mute: Arc::new(RwLock::new(false)),
            events: None,
        }
    }

    /// Create with event manager for reactive updates
    pub fn with_events(events: EventManager) -> Self {
        AudioControl {
            connection: Arc::new(RwLock::new(None)),
            current_volume: Arc::new(RwLock::new(50.0)),
            current_mute: Arc::new(RwLock::new(false)),
            events: Some(events),
        }
    }

    /// Initialize connection to audio system
    pub async fn connect(&self) -> Result<()> {
        match Connection::session().await {
            Ok(conn) => {
                info!("Connected to D-Bus session for audio control");
                let mut connection = self.connection.write().await;
                *connection = Some(conn);

                // Try to get initial state
                if let Err(e) = self.update_state().await {
                    warn!("Failed to get initial audio state: {}", e);
                }

                Ok(())
            }
            Err(e) => {
                warn!("Could not connect to D-Bus for audio: {}", e);
                Err(AmiyaError::Backend(format!(
                    "Failed to connect to D-Bus: {}",
                    e
                )))
            }
        }
    }

    /// Get current volume level (0-100)
    pub async fn get_volume(&self) -> Result<f64> {
        let volume = *self.current_volume.read().await;
        Ok(volume)
    }

    /// Set volume level (0-100)
    pub async fn set_volume(&self, volume: f64) -> Result<()> {
        let volume = volume.clamp(0.0, 100.0);

        // For now, just update local state and emit event
        // In a full implementation, we'd send D-Bus commands to PulseAudio
        {
            let mut vol = self.current_volume.write().await;
            *vol = volume;
        }

        info!("Volume set to {}%", volume);

        // Emit event
        if let Some(events) = &self.events {
            let mute = *self.current_mute.read().await;
            events.emit(Event::VolumeChanged {
                level: volume,
                muted: mute,
            });
        }

        Ok(())
    }

    /// Increase volume by step
    pub async fn increase_volume(&self, step: f64) -> Result<()> {
        let current = self.get_volume().await?;
        let new_volume = (current + step).min(100.0);
        self.set_volume(new_volume).await
    }

    /// Decrease volume by step
    pub async fn decrease_volume(&self, step: f64) -> Result<()> {
        let current = self.get_volume().await?;
        let new_volume = (current - step).max(0.0);
        self.set_volume(new_volume).await
    }

    /// Get mute state
    pub async fn get_mute(&self) -> Result<bool> {
        let muted = *self.current_mute.read().await;
        Ok(muted)
    }

    /// Set mute state
    pub async fn set_mute(&self, muted: bool) -> Result<()> {
        {
            let mut mute = self.current_mute.write().await;
            *mute = muted;
        }

        info!("Audio mute: {}", muted);

        // Emit event
        if let Some(events) = &self.events {
            let volume = *self.current_volume.read().await;
            events.emit(Event::VolumeChanged {
                level: volume,
                muted,
            });
        }

        Ok(())
    }

    /// Toggle mute state
    pub async fn toggle_mute(&self) -> Result<()> {
        let current = self.get_mute().await?;
        self.set_mute(!current).await
    }

    /// Update internal state (for polling or after changes)
    async fn update_state(&self) -> Result<()> {
        // In a full implementation, we'd query PulseAudio via D-Bus here
        // For now, state is managed internally
        Ok(())
    }

    /// Start monitoring audio changes
    /// This would subscribe to PulseAudio D-Bus signals in a full implementation
    pub async fn start_monitoring(&self) -> Result<()> {
        debug!("Audio monitoring started (mock implementation)");
        // TODO: Subscribe to PulseAudio signals
        // - org.PulseAudio.Core1.Device.VolumeUpdated
        // - org.PulseAudio.Core1.Device.MuteUpdated
        Ok(())
    }
}

impl Default for AudioControl {
    fn default() -> Self {
        Self::new()
    }
}

// Helper function to run async audio control in GTK context
pub fn create_audio_control_sync(events: EventManager) -> Arc<AudioControl> {
    let audio = Arc::new(AudioControl::with_events(events));

    // Try to connect in background
    let audio_clone = audio.clone();
    tokio::spawn(async move {
        if let Err(e) = audio_clone.connect().await {
            warn!("Failed to connect audio control: {}", e);
        }
    });

    audio
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_volume_control() {
        let audio = AudioControl::new();

        audio.set_volume(75.0).await.unwrap();
        assert_eq!(audio.get_volume().await.unwrap(), 75.0);

        audio.increase_volume(10.0).await.unwrap();
        assert_eq!(audio.get_volume().await.unwrap(), 85.0);

        audio.decrease_volume(20.0).await.unwrap();
        assert_eq!(audio.get_volume().await.unwrap(), 65.0);
    }

    #[tokio::test]
    async fn test_mute_control() {
        let audio = AudioControl::new();

        assert_eq!(audio.get_mute().await.unwrap(), false);

        audio.toggle_mute().await.unwrap();
        assert_eq!(audio.get_mute().await.unwrap(), true);

        audio.toggle_mute().await.unwrap();
        assert_eq!(audio.get_mute().await.unwrap(), false);
    }

    #[tokio::test]
    async fn test_volume_clamping() {
        let audio = AudioControl::new();

        audio.set_volume(150.0).await.unwrap();
        assert_eq!(audio.get_volume().await.unwrap(), 100.0);

        audio.set_volume(-10.0).await.unwrap();
        assert_eq!(audio.get_volume().await.unwrap(), 0.0);
    }
}
