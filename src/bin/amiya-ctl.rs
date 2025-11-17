use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;

/// Amiya Control - CLI tool for controlling Amiya desktop environment
#[derive(Parser)]
#[command(name = "amiya-ctl")]
#[command(about = "Control Amiya desktop environment", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Control popups
    Popup {
        #[command(subcommand)]
        action: PopupAction,
    },

    /// Control volume
    Volume {
        #[command(subcommand)]
        action: VolumeAction,
    },

    /// Control brightness
    Brightness {
        #[command(subcommand)]
        action: BrightnessAction,
    },

    /// Get status
    Status,

    /// Ping the server
    Ping,
}

#[derive(Subcommand)]
enum PopupAction {
    /// Show a popup
    Show {
        /// Type of popup (bluetooth, wifi, media-control)
        popup: String,
    },

    /// Hide a popup
    Hide {
        /// Type of popup (bluetooth, wifi, media-control)
        popup: String,
    },

    /// Toggle a popup
    Toggle {
        /// Type of popup (bluetooth, wifi, media-control)
        popup: String,
    },
}

#[derive(Subcommand)]
enum VolumeAction {
    /// Increase volume
    Up {
        /// Amount to increase (default: 5.0)
        #[arg(short, long)]
        amount: Option<f64>,
    },

    /// Decrease volume
    Down {
        /// Amount to decrease (default: 5.0)
        #[arg(short, long)]
        amount: Option<f64>,
    },

    /// Set volume to specific level
    Set {
        /// Volume level (0-100)
        level: f64,
    },

    /// Mute audio
    Mute,

    /// Unmute audio
    Unmute,

    /// Toggle mute
    ToggleMute,
}

#[derive(Subcommand)]
enum BrightnessAction {
    /// Increase brightness
    Up {
        /// Amount to increase (default: 5.0)
        #[arg(short, long)]
        amount: Option<f64>,
    },

    /// Decrease brightness
    Down {
        /// Amount to decrease (default: 5.0)
        #[arg(short, long)]
        amount: Option<f64>,
    },

    /// Set brightness to specific level
    Set {
        /// Brightness level (0-100)
        level: f64,
    },
}

// Mirror the IPC protocol types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
enum Command {
    ShowPopup { popup: PopupType },
    HidePopup { popup: PopupType },
    TogglePopup { popup: PopupType },
    Volume { action: VolumeActionData },
    Brightness { action: BrightnessActionData },
    Status,
    Ping,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum PopupType {
    Bluetooth,
    Wifi,
    MediaControl,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "kebab-case")]
enum VolumeActionData {
    Up { amount: Option<f64> },
    Down { amount: Option<f64> },
    Set { level: f64 },
    Mute,
    Unmute,
    ToggleMute,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "kebab-case")]
enum BrightnessActionData {
    Up { amount: Option<f64> },
    Down { amount: Option<f64> },
    Set { level: f64 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "kebab-case")]
enum Response {
    Success { message: Option<String> },
    Error { message: String },
    Status { version: String, uptime: u64 },
    Pong,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let command = match cli.command {
        Commands::Popup { action } => match action {
            PopupAction::Show { popup } => Command::ShowPopup {
                popup: parse_popup_type(&popup)?,
            },
            PopupAction::Hide { popup } => Command::HidePopup {
                popup: parse_popup_type(&popup)?,
            },
            PopupAction::Toggle { popup } => Command::TogglePopup {
                popup: parse_popup_type(&popup)?,
            },
        },
        Commands::Volume { action } => Command::Volume {
            action: match action {
                VolumeAction::Up { amount } => VolumeActionData::Up { amount },
                VolumeAction::Down { amount } => VolumeActionData::Down { amount },
                VolumeAction::Set { level } => VolumeActionData::Set { level },
                VolumeAction::Mute => VolumeActionData::Mute,
                VolumeAction::Unmute => VolumeActionData::Unmute,
                VolumeAction::ToggleMute => VolumeActionData::ToggleMute,
            },
        },
        Commands::Brightness { action } => Command::Brightness {
            action: match action {
                BrightnessAction::Up { amount } => BrightnessActionData::Up { amount },
                BrightnessAction::Down { amount } => BrightnessActionData::Down { amount },
                BrightnessAction::Set { level } => BrightnessActionData::Set { level },
            },
        },
        Commands::Status => Command::Status,
        Commands::Ping => Command::Ping,
    };

    send_command(command)?;

    Ok(())
}

fn parse_popup_type(s: &str) -> anyhow::Result<PopupType> {
    match s.to_lowercase().as_str() {
        "bluetooth" | "bt" => Ok(PopupType::Bluetooth),
        "wifi" | "network" => Ok(PopupType::Wifi),
        "media-control" | "media" => Ok(PopupType::MediaControl),
        _ => Err(anyhow::anyhow!(
            "Invalid popup type: {}. Valid types: bluetooth, wifi, media-control",
            s
        )),
    }
}

fn get_socket_path() -> anyhow::Result<PathBuf> {
    let runtime_dir = std::env::var("XDG_RUNTIME_DIR")
        .or_else(|_| std::env::var("TMPDIR"))
        .unwrap_or_else(|_| "/tmp".to_string());

    let socket_path = PathBuf::from(runtime_dir).join("amiya").join("amiya.sock");

    if !socket_path.exists() {
        return Err(anyhow::anyhow!(
            "Amiya socket not found at {:?}. Is Amiya running?",
            socket_path
        ));
    }

    Ok(socket_path)
}

fn send_command(command: Command) -> anyhow::Result<()> {
    let socket_path = get_socket_path()?;

    // Connect to Unix socket
    let mut stream = UnixStream::connect(&socket_path)
        .map_err(|e| anyhow::anyhow!("Failed to connect to Amiya: {}. Is Amiya running?", e))?;

    // Serialize command
    let command_json = serde_json::to_string(&command)?;

    // Send command
    stream.write_all(command_json.as_bytes())?;
    stream.write_all(b"\n")?;
    stream.flush()?;

    // Read response
    let mut reader = BufReader::new(stream);
    let mut response_line = String::new();
    reader.read_line(&mut response_line)?;

    // Parse response
    let response: Response = serde_json::from_str(&response_line)?;

    // Print response
    match response {
        Response::Success { message } => {
            if let Some(msg) = message {
                println!("✓ {}", msg);
            } else {
                println!("✓ Success");
            }
        }
        Response::Error { message } => {
            eprintln!("✗ Error: {}", message);
            std::process::exit(1);
        }
        Response::Status { version, uptime } => {
            println!("Amiya Desktop Environment");
            println!("Version: {}", version);
            println!("Uptime: {} seconds", uptime);
        }
        Response::Pong => {
            println!("✓ Pong! Server is alive.");
        }
    }

    Ok(())
}
