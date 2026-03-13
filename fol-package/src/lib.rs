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
pub mod build;
pub mod errors;
pub mod identity;
pub mod metadata;
pub mod model;
pub mod session;

pub use build::{parse_package_build, BuildDependency, BuildExport, PackageBuildDefinition};
pub use config::PackageConfig;
pub use errors::{PackageError, PackageErrorKind};
pub use identity::{PackageIdentity, PackageSourceKind};
pub use metadata::{parse_package_metadata, PackageMetadata};
pub use model::PreparedPackage;
pub use session::{
    canonical_directory_root, infer_package_root, parse_directory_package_syntax,
    resolve_directory_path, PackageSession,
};
