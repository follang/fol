//! Package discovery and acquisition for FOL.
//!
//! Current scope:
//! - package loading configuration and identity types
//! - package metadata loading
//! - package build-definition loading
//! - package root validation
//! - package graph preparation ahead of ordinary source resolution
//!
//! This crate intentionally does not perform name resolution for ordinary source code.

pub mod config;
pub mod errors;
pub mod identity;
pub mod model;

pub use config::PackageConfig;
pub use errors::{PackageError, PackageErrorKind};
pub use identity::{PackageIdentity, PackageSourceKind};
pub use model::PreparedPackage;
