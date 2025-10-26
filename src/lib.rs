// lib.rs
#![warn(clippy::large_futures)]

pub use anyhow::bail;
use serde::{Deserialize, Serialize};
pub use std::{
    net,
    pin::Pin,
    sync::{
        atomic::{AtomicU32, Ordering},
        Arc,
    },
};
pub use tokio::{
    sync::RwLock,
    task,
    time::{sleep, Duration},
};
pub use tracing::*;

pub use apiserver::*;
pub use config::*;
pub use serial::*;
pub use state::*;
pub use wifi::*;

pub const FW_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Deserialize)]
pub struct UpdateFirmware {
    url: String,
}

mod apiserver;
mod config;
mod serial;
mod state;
mod wifi;

// EOF
