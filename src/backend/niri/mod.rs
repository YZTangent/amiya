pub mod client;
pub mod events;
pub mod protocol;

pub use client::NiriClient;
pub use events::{start_workspace_polling, NiriEventListener};
pub use protocol::{NiriAction, NiriEvent, NiriWorkspace, WorkspaceReference};
