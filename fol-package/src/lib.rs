//! Package discovery and acquisition for FOL.
//!
//! Current scope:
//! - package metadata loading
//! - package build-definition loading
//! - package root validation
//! - package graph preparation ahead of ordinary source resolution
//!
//! This crate intentionally does not perform name resolution for ordinary source code.

pub mod errors;

pub use errors::{PackageError, PackageErrorKind};
