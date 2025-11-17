use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_bar")]
    pub bar: BarConfig,

    #[serde(default)]
    pub hotkeys: HashMap<String, HotkeyAction>,

    #[serde(default)]
    pub theme: ThemeConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BarConfig {
    #[serde(default = "default_height")]
    pub height: i32,

    #[serde(default = "default_position")]
    pub position: Position,

    #[serde(default = "default_true")]
    pub show_workspaces: bool,

    #[serde(default = "default_true")]
    pub show_clock: bool,

    #[serde(default = "default_true")]
    pub show_system_info: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Position {
    Top,
    Bottom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum HotkeyAction {
    ShowBluetooth,
    ShowWifi,
    ShowMediaControl,
    IncreaseBrightness,
    DecreaseBrightness,
    IncreaseVolume,
    DecreaseVolume,
    Mute,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeConfig {
    #[serde(default = "default_background")]
    pub background: String,

    #[serde(default = "default_foreground")]
    pub foreground: String,

    #[serde(default = "default_accent")]
    pub accent: String,

    #[serde(default = "default_font")]
    pub font: String,

    #[serde(default = "default_font_size")]
    pub font_size: u32,
}

// Default values
fn default_bar() -> BarConfig {
    BarConfig {
        height: default_height(),
        position: default_position(),
        show_workspaces: true,
        show_clock: true,
        show_system_info: true,
    }
}

fn default_height() -> i32 {
    32
}

fn default_position() -> Position {
    Position::Top
}

fn default_true() -> bool {
    true
}

fn default_background() -> String {
    "#1e1e2e".to_string()
}

fn default_foreground() -> String {
    "#cdd6f4".to_string()
}

fn default_accent() -> String {
    "#89b4fa".to_string()
}

fn default_font() -> String {
    "Sans".to_string()
}

fn default_font_size() -> u32 {
    11
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            background: default_background(),
            foreground: default_foreground(),
            accent: default_accent(),
            font: default_font(),
            font_size: default_font_size(),
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path()?;

        if config_path.exists() {
            let content = fs::read_to_string(&config_path)
                .context("Failed to read config file")?;
            let config: Config = toml::from_str(&content)
                .context("Failed to parse config file")?;
            Ok(config)
        } else {
            // Create default config
            let config = Self::default();
            config.save()?;
            Ok(config)
        }
    }

    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path()?;

        // Create parent directory if it doesn't exist
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = toml::to_string_pretty(self)?;
        fs::write(&config_path, content)?;

        Ok(())
    }

    fn config_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .or_else(|| std::env::var("HOME").ok().map(|h| PathBuf::from(h).join(".config")))
            .context("Could not determine config directory")?;

        Ok(config_dir.join("amiya").join("config.toml"))
    }
}

impl Default for Config {
    fn default() -> Self {
        let mut hotkeys = HashMap::new();
        hotkeys.insert("Super+B".to_string(), HotkeyAction::ShowBluetooth);
        hotkeys.insert("Super+W".to_string(), HotkeyAction::ShowWifi);
        hotkeys.insert("Super+M".to_string(), HotkeyAction::ShowMediaControl);

        Self {
            bar: default_bar(),
            hotkeys,
            theme: ThemeConfig::default(),
        }
    }
}

// Helper module since dirs crate is not in dependencies
mod dirs {
    use std::env;
    use std::path::PathBuf;

    pub fn config_dir() -> Option<PathBuf> {
        env::var("XDG_CONFIG_HOME")
            .ok()
            .map(PathBuf::from)
            .or_else(|| {
                env::var("HOME")
                    .ok()
                    .map(|h| PathBuf::from(h).join(".config"))
            })
    }
}
