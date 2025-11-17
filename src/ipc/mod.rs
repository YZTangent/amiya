pub mod niri;
pub mod protocol;
pub mod server;

pub use niri::NiriClient;
pub use protocol::{Command, PopupType, Response};
pub use server::IpcServer;
