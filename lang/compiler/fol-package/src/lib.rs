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
pub mod build_codegen;
pub mod build_dependency;
pub mod build_entry;
pub mod build_eval;
pub mod build_graph;
pub mod build_native;
pub mod build_option;
pub mod build_step;
pub mod build_semantic;
pub mod errors;
pub mod git;
pub mod identity;
pub mod locator;
pub mod lockfile;
pub mod metadata;
pub mod model;
pub mod paths;
pub mod session;

pub use fol_parser::ast::ParsedSourceUnitKind;
pub use build::{
    classify_semantic_build_mode, parse_package_build, BuildDependency, BuildExport,
    PackageBuildCompatibility,
    PackageBuildDefinition, PackageBuildMode, PackageNativeArtifact, PackageNativeArtifactKind,
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
pub use build_codegen::{
    CodegenKind, CodegenRequest, CodegenResult, GeneratedFileAction,
    GeneratedFileDefinition, GeneratedFileInstallProjection, GeneratedFileSet,
    GeneratedOutputDependencySet, SystemToolRequest, SystemToolResult,
};
pub use build_dependency::{
    dependency_modules_from_exports, DependencyArtifactSurface, DependencyArtifactSurfaceSet,
    DependencyBuildEvaluationMode, DependencyBuildHandle, DependencyBuildSurface,
    DependencyBuildSurfaceSet, DependencyGeneratedOutputSurface,
    DependencyGeneratedOutputSurfaceSet, DependencyModuleSurface, DependencyModuleSurfaceSet,
    DependencySourceRootSurface, DependencyStepSurface, DependencyStepSurfaceSet,
};
pub use build_entry::{
    collect_build_entry_candidates, validate_build_entry_cardinality,
    validate_build_entry_parameter_shape, validate_build_entry_parameter_type,
    validate_build_entry_return_type, validate_parsed_build_entry,
    BuildEntryCandidate, BuildEntrySignatureExpectation, BuildEntryValidationError,
    BuildEntryValidationErrorKind, ValidatedBuildEntry,
};
pub use build_eval::{
    evaluate_build_plan, evaluate_build_source, extract_build_program_from_source,
    AllowedBuildTimeOperation, BuildEvaluationBoundary, BuildEvaluationError,
    BuildEvaluationErrorKind, BuildEvaluationInputs, BuildEvaluationInstallArtifactRequest,
    BuildEvaluationOperation, BuildEvaluationOperationKind, BuildEvaluationRequest,
    BuildEvaluationResult, BuildEvaluationRunRequest, BuildEvaluationStepRequest,
    EvaluatedBuildSource, ExtractedBuildArtifact, ExtractedBuildProgram,
};
pub use build_graph::{
    BuildArtifact, BuildArtifactDependency, BuildArtifactId, BuildArtifactInput,
    BuildArtifactKind, BuildGeneratedFile, BuildGeneratedFileId, BuildGeneratedFileKind,
    BuildGraph, BuildGraphValidationError, BuildGraphValidationErrorKind, BuildInstall,
    BuildInstallId, BuildInstallKind, BuildInstallTarget, BuildModule, BuildModuleId,
    BuildModuleKind, BuildOption, BuildOptionId, BuildOptionKind, BuildStep,
    BuildStepDependency, BuildStepId, BuildStepKind,
};
pub use build_option::{
    BuildOptimizeMode, BuildOptionDeclaration, BuildOptionDeclarationSet, BuildOptionOverride,
    BuildOptionOverrideParseError, BuildTargetArch, BuildTargetEnvironment, BuildTargetOs,
    BuildTargetTriple, ResolvedBuildOptionSet, StandardOptimizeDeclaration,
    StandardTargetDeclaration, UserOptionDeclaration,
};
pub use build_native::{
    project_compatibility_native_artifact, project_compatibility_native_artifacts,
    NativeArtifactDefinition, NativeArtifactKind, NativeArtifactSet, NativeIncludePath,
    NativeLibraryPath, NativeLinkDirective, NativeLinkInput, NativeLinkMode, NativePlatform,
    NativeSearchPathOrigin,
};
pub use build_step::{
    plan_step_order, project_graph_steps, BuildDefaultStepKind, BuildRequestedStep,
    BuildStepCacheBoundary, BuildStepCacheKey, BuildStepDefinition, BuildStepEvent,
    BuildStepEventKind, BuildStepExecutionRequest, BuildStepExecutionResult, BuildStepPlanError,
    BuildStepReport,
};
pub use build_semantic::{
    canonical_artifact_config_shapes, canonical_chain_metadata,
    canonical_graph_method_signatures, canonical_handle_method_signatures,
    canonical_option_config_shapes, canonical_option_value_kinds, BuildSemanticChainKind,
    BuildSemanticChainMetadata, BuildSemanticMethodParameter, BuildSemanticMethodSignature,
    BuildSemanticOptionValueKind, BuildSemanticParameterShape, BuildSemanticRecordField,
    BuildSemanticRecordShape, BuildSemanticRecordShapeKind, BuildSemanticType,
    BuildSemanticTypeFamily, BuildStdlibImportSurface, BuildStdlibModuleKind,
    BuildStdlibModulePath,
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

#[cfg(test)]
mod tests {
    use super::{
        collect_build_entry_candidates, validate_parsed_build_entry, BuildEntrySignatureExpectation,
        canonical_chain_metadata, canonical_graph_method_signatures, canonical_handle_method_signatures,
        canonical_option_value_kinds, BuildSemanticChainKind, BuildSemanticType,
        BuildSemanticTypeFamily, ParsedSourceUnitKind,
        NativeArtifactDefinition, NativeArtifactKind, NativeArtifactSet, NativeLinkDirective,
        NativeLinkInput, NativeLinkMode,
    };

    #[test]
    fn crate_root_reexports_native_surface_types() {
        let mut set = NativeArtifactSet::new();
        let artifact = NativeArtifactDefinition {
            name: "ssl".to_string(),
            kind: NativeArtifactKind::StaticLibrary,
            relative_path: "native/libssl.a".to_string(),
        };
        set.add(artifact.clone());
        let directive = NativeLinkDirective {
            input: NativeLinkInput::Artifact(artifact),
            mode: NativeLinkMode::Static,
        };

        assert_eq!(set.definitions().len(), 1);
        assert_eq!(directive.mode, NativeLinkMode::Static);
    }

    #[test]
    fn crate_root_reexports_semantic_build_surface_types() {
        let graph = BuildSemanticType::graph();
        let methods = canonical_graph_method_signatures();
        let handles = canonical_handle_method_signatures();
        let chains = canonical_chain_metadata();
        let option_kinds = canonical_option_value_kinds();

        assert_eq!(graph.family, BuildSemanticTypeFamily::Graph);
        assert!(methods.iter().any(|method| method.name == "add_exe"));
        assert!(handles.iter().all(|method| method.name == "depend_on"));
        assert!(chains
            .iter()
            .any(|chain| chain.kind == BuildSemanticChainKind::RunDependency));
        assert!(option_kinds.len() >= 7);
    }

    #[test]
    fn crate_root_reexports_parsed_source_unit_kinds() {
        assert_eq!(ParsedSourceUnitKind::Build, fol_parser::ast::ParsedSourceUnitKind::Build);
    }

    #[test]
    fn crate_root_reexports_semantic_build_entry_surface() {
        let syntax = fol_parser::ast::ParsedPackage {
            package: "demo".to_string(),
            source_units: vec![fol_parser::ast::ParsedSourceUnit {
                path: "build.fol".to_string(),
                package: "demo".to_string(),
                namespace: "demo".to_string(),
                kind: ParsedSourceUnitKind::Build,
                items: vec![fol_parser::ast::ParsedTopLevel {
                    node_id: fol_parser::ast::SyntaxNodeId(1),
                    node: fol_parser::ast::AstNode::DefDecl {
                        options: Vec::new(),
                        name: "build".to_string(),
                        params: vec![fol_parser::ast::Parameter {
                            name: "graph".to_string(),
                            param_type: fol_parser::ast::FolType::Named {
                                syntax_id: None,
                                name: "Graph".to_string(),
                            },
                            is_borrowable: false,
                            is_mutex: false,
                            default: None,
                        }],
                        def_type: fol_parser::ast::FolType::Named {
                            syntax_id: None,
                            name: "Graph".to_string(),
                        },
                        body: Vec::new(),
                    },
                    meta: fol_parser::ast::ParsedTopLevelMeta::default(),
                }],
            }],
            syntax_index: fol_parser::ast::SyntaxIndex::default(),
        };

        let candidates = collect_build_entry_candidates(&syntax);
        let validated = validate_parsed_build_entry(&syntax, &BuildEntrySignatureExpectation::canonical())
            .expect("crate root should expose semantic build entry validation");

        assert_eq!(candidates.len(), 1);
        assert_eq!(validated.candidate.name, "build");
    }

    #[test]
    fn crate_root_reexports_semantic_build_modes() {
        let syntax = fol_parser::ast::ParsedPackage {
            package: "demo".to_string(),
            source_units: vec![fol_parser::ast::ParsedSourceUnit {
                path: "build.fol".to_string(),
                package: "demo".to_string(),
                namespace: "demo".to_string(),
                kind: ParsedSourceUnitKind::Build,
                items: vec![fol_parser::ast::ParsedTopLevel {
                    node_id: fol_parser::ast::SyntaxNodeId(1),
                    node: fol_parser::ast::AstNode::DefDecl {
                        options: Vec::new(),
                        name: "build".to_string(),
                        params: vec![fol_parser::ast::Parameter {
                            name: "graph".to_string(),
                            param_type: fol_parser::ast::FolType::Named {
                                syntax_id: None,
                                name: "Graph".to_string(),
                            },
                            is_borrowable: false,
                            is_mutex: false,
                            default: None,
                        }],
                        def_type: fol_parser::ast::FolType::Named {
                            syntax_id: None,
                            name: "Graph".to_string(),
                        },
                        body: Vec::new(),
                    },
                    meta: fol_parser::ast::ParsedTopLevelMeta::default(),
                }],
            }],
            syntax_index: fol_parser::ast::SyntaxIndex::default(),
        };

        assert_eq!(classify_semantic_build_mode(&syntax, false), PackageBuildMode::ModernOnly);
    }
}
