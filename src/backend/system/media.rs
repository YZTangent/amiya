use crate::error::{AmiyaError, Result};
use crate::events::{Event, EventManager};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use zbus::Connection;

/// Media player information
#[derive(Debug, Clone)]
pub struct MediaPlayer {
    pub name: String,
    pub bus_name: String,
    pub identity: String,
}

/// Track metadata
#[derive(Debug, Clone)]
pub struct TrackMetadata {
    pub title: String,
    pub artist: String,
    pub album: Option<String>,
    pub art_url: Option<String>,
    pub track_id: Option<String>,
}

/// Playback status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaybackStatus {
    Playing,
    Paused,
    Stopped,
}

impl std::fmt::Display for PlaybackStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PlaybackStatus::Playing => write!(f, "Playing"),
            PlaybackStatus::Paused => write!(f, "Paused"),
            PlaybackStatus::Stopped => write!(f, "Stopped"),
        }
    }
}

/// Media control via MPRIS2
pub struct MediaControl {
    connection: Arc<RwLock<Option<Connection>>>,
    active_player: Arc<RwLock<Option<String>>>,
    players: Arc<RwLock<Vec<MediaPlayer>>>,
    playback_status: Arc<RwLock<PlaybackStatus>>,
    current_track: Arc<RwLock<Option<TrackMetadata>>>,
    volume: Arc<RwLock<f64>>,
    events: Option<EventManager>,
}

impl MediaControl {
    /// Create a new media control instance
    pub fn new() -> Self {
        MediaControl {
            connection: Arc::new(RwLock::new(None)),
            active_player: Arc::new(RwLock::new(None)),
            players: Arc::new(RwLock::new(Vec::new())),
            playback_status: Arc::new(RwLock::new(PlaybackStatus::Stopped)),
            current_track: Arc::new(RwLock::new(None)),
            volume: Arc::new(RwLock::new(1.0)),
            events: None,
        }
    }

    /// Create with event manager for reactive updates
    pub fn with_events(events: EventManager) -> Self {
        let mut media = Self::new();
        media.events = Some(events);
        media
    }

    /// Initialize connection to D-Bus
    pub async fn connect(&self) -> Result<()> {
        match Connection::session().await {
            Ok(conn) => {
                info!("Connected to D-Bus session bus for Media control");

                // Store connection
                {
                    let mut connection = self.connection.write().await;
                    *connection = Some(conn.clone());
                }

                // Discover media players
                if let Err(e) = self.discover_players(&conn).await {
                    warn!("Failed to discover media players: {}", e);
                }

                Ok(())
            }
            Err(e) => {
                warn!("Could not connect to D-Bus session bus: {}", e);
                Err(AmiyaError::Backend(format!(
                    "Failed to connect to D-Bus: {}",
                    e
                )))
            }
        }
    }

    /// Discover available media players
    async fn discover_players(&self, conn: &Connection) -> Result<()> {
        // List all names on the session bus
        let dbus_proxy = zbus::ProxyBuilder::new(conn)
            .interface("org.freedesktop.DBus")
            .path("/org/freedesktop/DBus")?
            .destination("org.freedesktop.DBus")?
            .build::<zbus::Proxy>()
            .await
            .map_err(|e| AmiyaError::Backend(format!("Failed to create D-Bus proxy: {}", e)))?;

        let names: Vec<String> = dbus_proxy
            .call_method("ListNames", &())
            .await
            .map_err(|e| AmiyaError::Backend(format!("Failed to list names: {}", e)))?
            .body()
            .deserialize()
            .map_err(|e| AmiyaError::Backend(format!("Failed to deserialize names: {}", e)))?;

        // Filter for MPRIS2 players
        let mut players = Vec::new();
        for name in names {
            if name.starts_with("org.mpris.MediaPlayer2.") {
                // Get player identity
                if let Ok(identity) = self.get_player_identity(conn, &name).await {
                    players.push(MediaPlayer {
                        name: name.strip_prefix("org.mpris.MediaPlayer2.")
                            .unwrap_or(&name)
                            .to_string(),
                        bus_name: name.clone(),
                        identity,
                    });

                    // Set as active player if we don't have one
                    let active = self.active_player.read().await;
                    if active.is_none() {
                        drop(active);
                        let mut active = self.active_player.write().await;
                        *active = Some(name.clone());

                        // Update player state
                        drop(active);
                        if let Err(e) = self.update_player_state(conn, &name).await {
                            debug!("Failed to update player state: {}", e);
                        }
                    }
                }
            }
        }

        {
            let mut players_state = self.players.write().await;
            *players_state = players.clone();
        }

        info!("Discovered {} media players", players.len());

        // Emit event
        if let Some(events) = &self.events {
            if let Some(active) = self.active_player.read().await.as_ref() {
                events.emit(Event::MediaPlayerChanged {
                    player: Some(active.clone()),
                });
            }
        }

        Ok(())
    }

    /// Get player identity
    async fn get_player_identity(&self, conn: &Connection, bus_name: &str) -> Result<String> {
        let proxy = zbus::ProxyBuilder::new(conn)
            .interface("org.mpris.MediaPlayer2")
            .path("/org/mpris/MediaPlayer2")?
            .destination(bus_name)?
            .build::<zbus::Proxy>()
            .await
            .map_err(|e| AmiyaError::Backend(format!("Failed to create player proxy: {}", e)))?;

        let identity: String = proxy
            .get_property("Identity")
            .await
            .map_err(|e| AmiyaError::Backend(format!("Failed to get identity: {}", e)))?;

        Ok(identity)
    }

    /// Update player state (status, metadata, etc.)
    async fn update_player_state(&self, conn: &Connection, bus_name: &str) -> Result<()> {
        let player_proxy = zbus::ProxyBuilder::new(conn)
            .interface("org.mpris.MediaPlayer2.Player")
            .path("/org/mpris/MediaPlayer2")?
            .destination(bus_name)?
            .build::<zbus::Proxy>()
            .await
            .map_err(|e| AmiyaError::Backend(format!("Failed to create player proxy: {}", e)))?;

        // Get playback status
        let status_str: String = player_proxy
            .get_property("PlaybackStatus")
            .await
            .unwrap_or_else(|_| "Stopped".to_string());

        let status = match status_str.as_str() {
            "Playing" => PlaybackStatus::Playing,
            "Paused" => PlaybackStatus::Paused,
            _ => PlaybackStatus::Stopped,
        };

        {
            let mut playback = self.playback_status.write().await;
            *playback = status;
        }

        // Emit playback event
        if let Some(events) = &self.events {
            events.emit(Event::MediaPlaybackChanged {
                playing: status == PlaybackStatus::Playing,
            });
        }

        // Get metadata
        if let Ok(metadata) = self.get_metadata_internal(&player_proxy).await {
            let mut track = self.current_track.write().await;
            *track = Some(metadata.clone());

            // Emit track event
            if let Some(events) = &self.events {
                events.emit(Event::MediaTrackChanged {
                    title: metadata.title,
                    artist: metadata.artist,
                    album: metadata.album,
                });
            }
        }

        // Get volume
        let volume: f64 = player_proxy
            .get_property("Volume")
            .await
            .unwrap_or(1.0);

        {
            let mut vol = self.volume.write().await;
            *vol = volume;
        }

        Ok(())
    }

    /// Get metadata from player proxy
    async fn get_metadata_internal(&self, player_proxy: &zbus::Proxy<'_>) -> Result<TrackMetadata> {
        use zbus::zvariant;

        let metadata: HashMap<String, zvariant::OwnedValue> = player_proxy
            .get_property("Metadata")
            .await
            .map_err(|e| AmiyaError::Backend(format!("Failed to get metadata: {}", e)))?;

        // Extract fields from metadata dict
        let title = metadata
            .get("xesam:title")
            .and_then(|v| v.downcast_ref::<str>().ok())
            .unwrap_or("Unknown")
            .to_string();

        let artist = metadata
            .get("xesam:artist")
            .and_then(|v| {
                // Artist can be a string or array of strings
                if let Ok(s) = v.downcast_ref::<str>() {
                    Some(s.to_string())
                } else if let Ok(arr) = v.downcast_ref::<zvariant::Array>() {
                    arr.get(0)
                        .and_then(|v| v.downcast_ref::<str>().ok())
                        .map(|s| s.to_string())
                } else {
                    None
                }
            })
            .unwrap_or_else(|| "Unknown".to_string());

        let album = metadata
            .get("xesam:album")
            .and_then(|v| v.downcast_ref::<str>().ok())
            .map(|s| s.to_string());

        let art_url = metadata
            .get("mpris:artUrl")
            .and_then(|v| v.downcast_ref::<str>().ok())
            .map(|s| s.to_string());

        let track_id = metadata
            .get("mpris:trackid")
            .and_then(|v| v.downcast_ref::<zvariant::ObjectPath>().ok())
            .map(|p| p.to_string());

        Ok(TrackMetadata {
            title,
            artist,
            album,
            art_url,
            track_id,
        })
    }

    /// Check if media control is available
    pub fn is_available(&self) -> bool {
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let conn = self.connection.read().await;
                let player = self.active_player.read().await;
                conn.is_some() && player.is_some()
            })
        })
    }

    /// Get list of available players
    pub async fn get_players(&self) -> Result<Vec<MediaPlayer>> {
        let players = self.players.read().await;
        Ok(players.clone())
    }

    /// Get active player name
    pub async fn get_active_player(&self) -> Option<String> {
        self.active_player.read().await.clone()
    }

    /// Set active player
    pub async fn set_active_player(&self, bus_name: &str) -> Result<()> {
        let conn_guard = self.connection.read().await;

        let conn = conn_guard
            .as_ref()
            .ok_or_else(|| AmiyaError::Backend("Not connected to D-Bus".to_string()))?;

        // Verify player exists
        let players = self.players.read().await;
        if !players.iter().any(|p| p.bus_name == bus_name) {
            return Err(AmiyaError::Backend(format!(
                "Player not found: {}",
                bus_name
            )));
        }

        {
            let mut active = self.active_player.write().await;
            *active = Some(bus_name.to_string());
        }

        // Update player state
        self.update_player_state(conn, bus_name).await?;

        info!("Active player set to: {}", bus_name);

        // Emit event
        if let Some(events) = &self.events {
            events.emit(Event::MediaPlayerChanged {
                player: Some(bus_name.to_string()),
            });
        }

        Ok(())
    }

    /// Play
    pub async fn play(&self) -> Result<()> {
        self.call_player_method("Play").await
    }

    /// Pause
    pub async fn pause(&self) -> Result<()> {
        self.call_player_method("Pause").await
    }

    /// Play/Pause toggle
    pub async fn play_pause(&self) -> Result<()> {
        self.call_player_method("PlayPause").await
    }

    /// Stop
    pub async fn stop(&self) -> Result<()> {
        self.call_player_method("Stop").await
    }

    /// Next track
    pub async fn next(&self) -> Result<()> {
        self.call_player_method("Next").await
    }

    /// Previous track
    pub async fn previous(&self) -> Result<()> {
        self.call_player_method("Previous").await
    }

    /// Call a method on the active player
    async fn call_player_method(&self, method: &str) -> Result<()> {
        let conn_guard = self.connection.read().await;
        let player_guard = self.active_player.read().await;

        let conn = conn_guard
            .as_ref()
            .ok_or_else(|| AmiyaError::Backend("Not connected to D-Bus".to_string()))?;

        let bus_name = player_guard
            .as_ref()
            .ok_or_else(|| AmiyaError::Backend("No active player".to_string()))?;

        let player_proxy = zbus::ProxyBuilder::new(conn)
            .interface("org.mpris.MediaPlayer2.Player")
            .path("/org/mpris/MediaPlayer2")?
            .destination(bus_name.as_str())?
            .build::<zbus::Proxy>()
            .await
            .map_err(|e| AmiyaError::Backend(format!("Failed to create player proxy: {}", e)))?;

        player_proxy
            .call_method(method, &())
            .await
            .map_err(|e| AmiyaError::Backend(format!("Failed to call {}: {}", method, e)))?;

        info!("Called {} on player", method);

        // Update state after action
        drop(conn_guard);
        drop(player_guard);

        if let Some(active) = self.active_player.read().await.as_ref() {
            if let Some(conn) = self.connection.read().await.as_ref() {
                let _ = self.update_player_state(conn, active).await;
            }
        }

        Ok(())
    }

    /// Get current playback status
    pub async fn get_playback_status(&self) -> PlaybackStatus {
        *self.playback_status.read().await
    }

    /// Get current track metadata
    pub async fn get_metadata(&self) -> Option<TrackMetadata> {
        self.current_track.read().await.clone()
    }

    /// Get volume (0.0-1.0)
    pub async fn get_volume(&self) -> f64 {
        *self.volume.read().await
    }

    /// Set volume (0.0-1.0)
    pub async fn set_volume(&self, volume: f64) -> Result<()> {
        let volume = volume.clamp(0.0, 1.0);

        let conn_guard = self.connection.read().await;
        let player_guard = self.active_player.read().await;

        let conn = conn_guard
            .as_ref()
            .ok_or_else(|| AmiyaError::Backend("Not connected to D-Bus".to_string()))?;

        let bus_name = player_guard
            .as_ref()
            .ok_or_else(|| AmiyaError::Backend("No active player".to_string()))?;

        let player_proxy = zbus::ProxyBuilder::new(conn)
            .interface("org.mpris.MediaPlayer2.Player")
            .path("/org/mpris/MediaPlayer2")?
            .destination(bus_name.as_str())?
            .build::<zbus::Proxy>()
            .await
            .map_err(|e| AmiyaError::Backend(format!("Failed to create player proxy: {}", e)))?;

        player_proxy
            .set_property("Volume", volume)
            .await
            .map_err(|e| AmiyaError::Backend(format!("Failed to set volume: {}", e)))?;

        {
            let mut vol = self.volume.write().await;
            *vol = volume;
        }

        info!("Media volume set to {:.2}", volume);

        // Emit event
        if let Some(events) = &self.events {
            events.emit(Event::MediaVolumeChanged { volume });
        }

        Ok(())
    }

    /// Start monitoring media events
    pub async fn start_monitoring(&self) -> Result<()> {
        debug!("Media monitoring started (basic implementation)");
        // TODO: Subscribe to D-Bus signals:
        // - PropertiesChanged (playback status, metadata, volume)
        // - NameOwnerChanged (new/removed players)
        Ok(())
    }
}

impl Default for MediaControl {
    fn default() -> Self {
        Self::new()
    }
}

// Helper function to create media control in GTK context
pub fn create_media_control_sync(events: EventManager) -> Arc<MediaControl> {
    let media = Arc::new(MediaControl::with_events(events));

    // Try to connect in background
    let media_clone = media.clone();
    tokio::spawn(async move {
        if let Err(e) = media_clone.connect().await {
            warn!("Failed to connect Media control: {}", e);
        }
    });

    media
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_media_creation() {
        let media = MediaControl::new();
        // Should not panic
        assert_eq!(media.get_playback_status().await, PlaybackStatus::Stopped);
    }

    #[tokio::test]
    async fn test_volume() {
        let media = MediaControl::new();
        assert_eq!(media.get_volume().await, 1.0);
    }
}
