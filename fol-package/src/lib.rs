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
pub mod git;
pub mod identity;
pub mod locator;
pub mod lockfile;
pub mod metadata;
pub mod model;
pub mod paths;
pub mod session;

pub use build::{
    parse_package_build, BuildDependency, BuildExport, PackageBuildDefinition,
    PackageNativeArtifact, PackageNativeArtifactKind,
};
pub use config::PackageConfig;
pub use errors::{PackageError, PackageErrorKind};
pub use git::{wrap_git_failure, PackageGitMaterialization, PackageGitSourceSession};
pub use identity::{PackageIdentity, PackageSourceKind};
pub use locator::{
    parse_package_locator, PackageGitLocator, PackageGitSelector, PackageGitTransport,
    PackageLocator, PackageLocatorKind,
};
pub use lockfile::{
    parse_package_lockfile, render_package_lockfile, PackageLockEntry, PackageLockfile,
};
pub use metadata::{
    parse_package_metadata, PackageDependencyDecl, PackageDependencySourceKind, PackageMetadata,
};
pub use model::{PreparedExportMount, PreparedPackage};
pub use paths::{git_cache_path, git_store_path};
pub use session::{
    canonical_directory_root, infer_package_root, parse_directory_package_syntax,
    resolve_directory_path, PackageSession,
};
