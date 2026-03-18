//! Build system execution for FOL.
//!
//! Owns all build graph IR, build API types, and build execution logic.
//! The compiler (`fol-package`) handles only entry validation and package metadata.

pub mod api;
pub mod codegen;
pub mod dependency;
pub mod graph;
pub mod semantic;

pub use api::{
    validate_build_name, BuildApi, BuildApiError, BuildApiNameError, BuildArtifactHandle,
    BuildOptionValue, DependencyHandle, DependencyRequest, ExecutableRequest,
    InstallArtifactRequest, InstallDirRequest, InstallFileRequest, InstallHandle, RunHandle,
    RunRequest, SharedLibraryRequest, StandardOptimizeOption, StandardOptimizeRequest,
    StandardTargetOption, StandardTargetRequest, StaticLibraryRequest, StepHandle, StepRequest,
    TestArtifactRequest, UserOption, UserOptionRequest,
};
pub use codegen::{
    CodegenKind, CodegenRequest, CodegenResult, GeneratedFileAction, GeneratedFileDefinition,
    GeneratedFileInstallProjection, GeneratedFileSet, GeneratedOutputDependencySet,
    SystemToolRequest, SystemToolResult,
};
pub use dependency::{
    DependencyArtifactSurface, DependencyArtifactSurfaceSet, DependencyBuildEvaluationMode,
    DependencyBuildHandle, DependencyBuildSurface, DependencyBuildSurfaceSet,
    DependencyGeneratedOutputSurface, DependencyGeneratedOutputSurfaceSet, DependencyModuleSurface,
    DependencyModuleSurfaceSet, DependencySourceRootSurface, DependencyStepSurface,
    DependencyStepSurfaceSet,
};
pub use graph::{
    BuildArtifact, BuildArtifactDependency, BuildArtifactId, BuildArtifactInput, BuildArtifactKind,
    BuildGeneratedFile, BuildGeneratedFileId, BuildGeneratedFileKind, BuildGraph,
    BuildGraphValidationError, BuildGraphValidationErrorKind, BuildInstall, BuildInstallId,
    BuildInstallKind, BuildInstallTarget, BuildModule, BuildModuleId, BuildModuleKind, BuildOption,
    BuildOptionId, BuildOptionKind, BuildStep, BuildStepDependency, BuildStepId, BuildStepKind,
};
pub use semantic::{
    canonical_artifact_config_shapes, canonical_chain_metadata, canonical_graph_method_signatures,
    canonical_handle_method_signatures, canonical_option_config_shapes,
    canonical_option_value_kinds, BuildSemanticChainKind, BuildSemanticChainMetadata,
    BuildSemanticMethodParameter, BuildSemanticMethodSignature, BuildSemanticOptionValueKind,
    BuildSemanticParameterShape, BuildSemanticRecordField, BuildSemanticRecordShape,
    BuildSemanticRecordShapeKind, BuildSemanticType, BuildSemanticTypeFamily,
    BuildStdlibImportSurface, BuildStdlibModuleKind, BuildStdlibModulePath,
};
