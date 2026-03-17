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
pub mod build_api;
pub mod build_artifact;
pub mod build_eval;
pub mod build_graph;
pub mod build_option;
pub mod build_step;
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
    parse_package_build, BuildDependency, BuildExport, PackageBuildCompatibility,
    PackageBuildDefinition, PackageBuildEntryPoint, PackageBuildEntryPointKind, PackageBuildMode,
    PackageNativeArtifact, PackageNativeArtifactKind,
};
pub use build_api::{
    validate_build_name, BuildApi, BuildApiError, BuildApiNameError, BuildArtifactHandle,
    BuildOptionValue, DependencyHandle, DependencyRequest, ExecutableRequest,
    InstallArtifactRequest, InstallDirRequest, InstallFileRequest, InstallHandle, RunHandle,
    RunRequest, SharedLibraryRequest, StandardOptimizeOption, StandardOptimizeRequest,
    StandardTargetOption, StandardTargetRequest, StaticLibraryRequest, StepHandle, StepRequest,
    TestArtifactRequest, UserOption, UserOptionRequest,
};
pub use build_artifact::{
    project_graph_artifacts, BuildArtifactDefinition, BuildArtifactLinkage,
    BuildArtifactModelKind, BuildArtifactModuleConfig, BuildArtifactOutput,
    BuildArtifactPipelinePlan, BuildArtifactPipelineStage, BuildArtifactReport,
    BuildArtifactRootSource, BuildArtifactSet, BuildArtifactTargetConfig,
};
pub use build_eval::{
    evaluate_build_plan, AllowedBuildTimeOperation, BuildEvaluationBoundary, BuildEvaluationError,
    BuildEvaluationErrorKind, BuildEvaluationInputs, BuildEvaluationInstallArtifactRequest,
    BuildEvaluationOperation, BuildEvaluationOperationKind, BuildEvaluationRequest,
    BuildEvaluationResult, BuildEvaluationRunRequest, BuildEvaluationStepRequest,
};
pub use build_graph::{
    BuildArtifact, BuildArtifactDependency, BuildArtifactId, BuildArtifactInput,
    BuildArtifactKind, BuildGeneratedFile, BuildGeneratedFileId, BuildGeneratedFileKind,
    BuildGraph, BuildGraphValidationError, BuildGraphValidationErrorKind, BuildInstall,
    BuildInstallId, BuildInstallKind, BuildInstallTarget, BuildModule, BuildModuleId,
    BuildModuleKind, BuildOption, BuildOptionId, BuildOptionKind, BuildStep,
    BuildStepDependency, BuildStepId, BuildStepKind,
};
pub use build_step::{
    plan_step_order, project_graph_steps, BuildDefaultStepKind, BuildRequestedStep,
    BuildStepCacheBoundary, BuildStepCacheKey, BuildStepDefinition, BuildStepEvent,
    BuildStepEventKind, BuildStepExecutionRequest, BuildStepExecutionResult, BuildStepPlanError,
    BuildStepReport,
};
pub use config::PackageConfig;
pub use errors::{PackageError, PackageErrorKind};
pub use git::{
    wrap_git_failure, PackageGitFetchOptions, PackageGitMaterialization, PackageGitSourceSession,
};
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
