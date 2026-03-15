//! Backend foundations for turning lowered `V1` FOL workspaces into runnable artifacts.

mod config;
mod error;

pub const CRATE_NAME: &str = "fol-backend";

#[derive(Debug, Default)]
pub struct Backend;

impl Backend {
    pub fn new() -> Self {
        Self
    }
}

pub fn crate_name() -> &'static str {
    CRATE_NAME
}

pub use config::{BackendConfig, BackendMode, BackendTarget};
pub use error::{BackendError, BackendErrorKind};
