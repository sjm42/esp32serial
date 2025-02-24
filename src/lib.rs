// lib.rs
#![warn(clippy::large_futures)]

pub use std::{pin::Pin, sync::Arc};
pub use tracing::*;

mod config;
pub use config::*;

mod state;
pub use state::*;

mod apiserver;
pub use apiserver::*;

mod wifi;
pub use wifi::*;

mod serial;
pub use serial::*;

pub const FW_VERSION: &str = env!("CARGO_PKG_VERSION");

// EOF
