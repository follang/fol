use super::super::{
    evaluate_build_source, BuildEvaluationErrorKind, BuildEvaluationInputs, BuildEvaluationRequest,
};
use crate::artifact::BuildArtifactFolModel;
use crate::option::{BuildOptimizeMode, BuildTargetTriple};
use crate::runtime::{BuildRuntimeDependencyQueryKind, BuildRuntimeGeneratedFileKind};
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

fn temp_build_package(source: &str) -> (PathBuf, PathBuf) {
    static NEXT_ID: AtomicU64 = AtomicU64::new(0);

    let package_root = std::env::temp_dir().join(format!(
        "fol_build_eval_src_{}_{}",
        std::process::id(),
        NEXT_ID.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir_all(&package_root).expect("temp package root should be created");
    fs::write(package_root.join("build.fol"), source).expect("build source should be written");
    (package_root.clone(), package_root.join("build.fol"))
}

#[test]
fn build_source_evaluator_supports_object_style_dependency_configs() {
    let source = concat!(
        "pro[] build(): non = {\n",
        "    var graph = .build().graph();\n",
        "    var core = graph.dependency({ alias = \"core\", package = \"org/core\", mode = \"lazy\" });\n",
        "    return;\n",
        "}\n",
    );
    let (package_root, build_path) = temp_build_package(source);
    let request = BuildEvaluationRequest {
        package_root: package_root.display().to_string(),
        inputs: BuildEvaluationInputs {
            working_directory: package_root.display().to_string(),
            ..BuildEvaluationInputs::default()
        },
        operations: Vec::new(),
    };

    let evaluated = evaluate_build_source(&request, &build_path, source)
        .expect("dependency configs should evaluate")
        .expect("build body should produce a graph");

    assert_eq!(evaluated.result.dependency_requests.len(), 1);
    assert_eq!(evaluated.result.dependency_requests[0].alias, "core");
    assert_eq!(evaluated.result.dependency_requests[0].package, "org/core");
    assert_eq!(
        evaluated.result.dependency_requests[0].evaluation_mode,
        Some(crate::DependencyBuildEvaluationMode::Lazy)
    );
    assert_eq!(evaluated.evaluated.dependencies.len(), 1);
    assert_eq!(evaluated.evaluated.dependencies[0].alias, "core");
}

#[test]
fn build_source_evaluator_keeps_artifact_fol_models_in_evaluated_programs() {
    let source = concat!(
        "pro[] build(): non = {\n",
        "    var graph = .build().graph();\n",
        "    graph.add_exe({ name = \"app\", root = \"src/app.fol\", fol_model = \"core\" });\n",
        "    graph.add_static_lib({ name = \"corelib\", root = \"src/lib.fol\", fol_model = \"alloc\" });\n",
        "    graph.add_shared_lib({ name = \"plugin\", root = \"src/plugin.fol\", fol_model = \"std\" });\n",
        "    graph.add_test({ name = \"tests\", root = \"test/app.fol\", fol_model = \"alloc\" });\n",
        "    return;\n",
        "}\n",
    );
    let (package_root, build_path) = temp_build_package(source);
    let request = BuildEvaluationRequest {
        package_root: package_root.display().to_string(),
        inputs: BuildEvaluationInputs {
            working_directory: package_root.display().to_string(),
            ..BuildEvaluationInputs::default()
        },
        operations: Vec::new(),
    };

    let evaluated = evaluate_build_source(&request, &build_path, source)
        .expect("artifact fol_model configs should evaluate")
        .expect("build body should produce a graph");

    assert_eq!(evaluated.evaluated.artifacts.len(), 4);
    assert_eq!(
        evaluated.evaluated.artifacts[0].fol_model,
        BuildArtifactFolModel::Core
    );
    assert_eq!(
        evaluated.evaluated.artifacts[1].fol_model,
        BuildArtifactFolModel::Alloc
    );
    assert_eq!(
        evaluated.evaluated.artifacts[2].fol_model,
        BuildArtifactFolModel::Std
    );
    assert_eq!(
        evaluated.evaluated.artifacts[3].fol_model,
        BuildArtifactFolModel::Alloc
    );
    assert_eq!(
        evaluated.evaluated.artifacts[1].kind,
        crate::runtime::BuildRuntimeArtifactKind::StaticLibrary
    );
    assert_eq!(
        evaluated.evaluated.artifacts[2].kind,
        crate::runtime::BuildRuntimeArtifactKind::SharedLibrary
    );
    assert_eq!(
        evaluated.evaluated.artifacts[3].kind,
        crate::runtime::BuildRuntimeArtifactKind::Test
    );
}

#[test]
fn build_source_evaluator_rejects_unknown_artifact_fol_models() {
    let source = concat!(
        "pro[] build(): non = {\n",
        "    var graph = .build().graph();\n",
        "    graph.add_exe({ name = \"app\", root = \"src/app.fol\", fol_model = \"hosted\" });\n",
        "    return;\n",
        "}\n",
    );
    let (package_root, build_path) = temp_build_package(source);
    let request = BuildEvaluationRequest {
        package_root: package_root.display().to_string(),
        inputs: BuildEvaluationInputs {
            working_directory: package_root.display().to_string(),
            ..BuildEvaluationInputs::default()
        },
        operations: Vec::new(),
    };

    let error = evaluate_build_source(&request, &build_path, source)
        .expect_err("unknown fol_model values should fail build evaluation");

    assert_eq!(error.kind(), BuildEvaluationErrorKind::InvalidInput);
    assert_eq!(
        error.message(),
        "artifact fol_model must be one of: core, alloc, std (got 'hosted')"
    );
}

#[test]
fn build_source_evaluator_supports_object_style_write_file_configs() {
    let source = concat!(
        "pro[] build(): non = {\n",
        "    var graph = .build().graph();\n",
        "    var version = graph.write_file({ name = \"version\", path = \"gen/version.fol\", contents = \"generated\" });\n",
        "    return;\n",
        "}\n",
    );
    let (package_root, build_path) = temp_build_package(source);
    let request = BuildEvaluationRequest {
        package_root: package_root.display().to_string(),
        inputs: BuildEvaluationInputs {
            working_directory: package_root.display().to_string(),
            ..BuildEvaluationInputs::default()
        },
        operations: Vec::new(),
    };

    let evaluated = evaluate_build_source(&request, &build_path, source)
        .expect("write-file configs should evaluate")
        .expect("build body should produce a graph");

    assert!(matches!(
        evaluated.result.graph.generated_files()[0].kind,
        crate::BuildGeneratedFileKind::Write
    ));
    assert_eq!(
        evaluated.result.graph.generated_files()[0].name,
        "gen/version.fol"
    );
}

#[test]
fn build_source_evaluator_supports_object_style_copy_file_configs() {
    let source = concat!(
        "pro[] build(): non = {\n",
        "    var graph = .build().graph();\n",
        "    var logo = graph.file_from_root(\"assets/logo.svg\");\n",
        "    var asset = graph.copy_file({ name = \"asset\", source = logo, path = \"gen/logo.svg\" });\n",
        "    return;\n",
        "}\n",
    );
    let (package_root, build_path) = temp_build_package(source);
    let request = BuildEvaluationRequest {
        package_root: package_root.display().to_string(),
        inputs: BuildEvaluationInputs {
            working_directory: package_root.display().to_string(),
            ..BuildEvaluationInputs::default()
        },
        operations: Vec::new(),
    };

    let evaluated = evaluate_build_source(&request, &build_path, source)
        .expect("copy-file configs should evaluate")
        .expect("build body should produce a graph");

    assert!(matches!(
        evaluated.result.graph.generated_files()[0].kind,
        crate::BuildGeneratedFileKind::Copy
    ));
    assert_eq!(
        evaluated.result.graph.generated_files()[0].name,
        "gen/logo.svg"
    );
}

#[test]
fn build_source_evaluator_supports_object_style_system_tool_configs() {
    let source = concat!(
        "pro[] build(): non = {\n",
        "    var graph = .build().graph();\n",
        "    var bindings = graph.add_system_tool({ tool = \"flatc\", output = \"gen/schema.fol\" });\n",
        "    return;\n",
        "}\n",
    );
    let (package_root, build_path) = temp_build_package(source);
    let request = BuildEvaluationRequest {
        package_root: package_root.display().to_string(),
        inputs: BuildEvaluationInputs {
            working_directory: package_root.display().to_string(),
            ..BuildEvaluationInputs::default()
        },
        operations: Vec::new(),
    };

    let evaluated = evaluate_build_source(&request, &build_path, source)
        .expect("system-tool configs should evaluate")
        .expect("build body should produce a graph");

    assert!(matches!(
        evaluated.result.graph.generated_files()[0].kind,
        crate::BuildGeneratedFileKind::CaptureOutput
    ));
    assert_eq!(
        evaluated.result.graph.generated_files()[0].name,
        "gen/schema.fol"
    );
}

#[test]
fn build_source_evaluator_supports_object_style_codegen_configs() {
    let source = concat!(
        "pro[] build(): non = {\n",
        "    var graph = .build().graph();\n",
        "    var schema = graph.add_codegen({ kind = \"schema\", input = \"schema/api.yaml\", output = \"gen/api.fol\" });\n",
        "    return;\n",
        "}\n",
    );
    let (package_root, build_path) = temp_build_package(source);
    let request = BuildEvaluationRequest {
        package_root: package_root.display().to_string(),
        inputs: BuildEvaluationInputs {
            working_directory: package_root.display().to_string(),
            ..BuildEvaluationInputs::default()
        },
        operations: Vec::new(),
    };

    let evaluated = evaluate_build_source(&request, &build_path, source)
        .expect("codegen configs should evaluate")
        .expect("build body should produce a graph");

    assert!(matches!(
        evaluated.result.graph.generated_files()[0].kind,
        crate::BuildGeneratedFileKind::Write
    ));
    assert_eq!(
        evaluated.result.graph.generated_files()[0].name,
        "gen/api.fol"
    );
}

#[test]
fn build_source_evaluator_keeps_generated_outputs_in_evaluated_programs() {
    let source = concat!(
        "pro[] build(): non = {\n",
        "    var graph = .build().graph();\n",
        "    var version = graph.write_file({ name = \"version\", path = \"gen/version.fol\", contents = \"generated\" });\n",
        "    var logo = graph.file_from_root(\"assets/logo.svg\");\n",
        "    var asset = graph.copy_file({ name = \"asset\", source = logo, path = \"gen/logo.svg\" });\n",
        "    return;\n",
        "}\n",
    );
    let (package_root, build_path) = temp_build_package(source);
    let request = BuildEvaluationRequest {
        package_root: package_root.display().to_string(),
        inputs: BuildEvaluationInputs {
            working_directory: package_root.display().to_string(),
            ..BuildEvaluationInputs::default()
        },
        operations: Vec::new(),
    };

    let evaluated = evaluate_build_source(&request, &build_path, source)
        .expect("generated outputs should evaluate")
        .expect("build body should produce a graph");

    assert_eq!(evaluated.evaluated.generated_files.len(), 2);
    assert!(evaluated
        .evaluated
        .generated_files
        .iter()
        .any(|file| file.relative_path == "gen/version.fol"
            && file.kind == BuildRuntimeGeneratedFileKind::Write));
    assert!(evaluated
        .evaluated
        .generated_files
        .iter()
        .any(|file| file.relative_path == "gen/logo.svg"
            && file.kind == BuildRuntimeGeneratedFileKind::Copy));
}

#[test]
fn build_source_evaluator_keeps_mixed_generated_output_families() {
    let source = concat!(
        "pro[] build(): non = {\n",
        "    var graph = .build().graph();\n",
        "    var version = graph.write_file({ name = \"version\", path = \"gen/version.fol\", contents = \"generated\" });\n",
        "    var logo = graph.file_from_root(\"assets/logo.svg\");\n",
        "    var asset = graph.copy_file({ name = \"asset\", source = logo, path = \"gen/logo.svg\" });\n",
        "    var tool = graph.add_system_tool({ tool = \"flatc\", output = \"gen/schema.fol\" });\n",
        "    var codegen = graph.add_codegen({ kind = \"schema\", input = \"schema/api.yaml\", output = \"gen/api.fol\" });\n",
        "    return;\n",
        "}\n",
    );
    let (package_root, build_path) = temp_build_package(source);
    let request = BuildEvaluationRequest {
        package_root: package_root.display().to_string(),
        inputs: BuildEvaluationInputs {
            working_directory: package_root.display().to_string(),
            ..BuildEvaluationInputs::default()
        },
        operations: Vec::new(),
    };

    let evaluated = evaluate_build_source(&request, &build_path, source)
        .expect("mixed generated outputs should evaluate")
        .expect("build body should produce a graph");
    let kinds = evaluated
        .evaluated
        .generated_files
        .iter()
        .map(|file| file.kind)
        .collect::<Vec<_>>();

    assert_eq!(evaluated.evaluated.generated_files.len(), 4);
    assert!(kinds.contains(&BuildRuntimeGeneratedFileKind::Write));
    assert!(kinds.contains(&BuildRuntimeGeneratedFileKind::Copy));
    assert!(kinds.contains(&BuildRuntimeGeneratedFileKind::ToolOutput));
    assert!(kinds.contains(&BuildRuntimeGeneratedFileKind::CodegenOutput));
}

#[test]
fn build_source_evaluator_records_dependency_module_and_artifact_queries() {
    let source = concat!(
        "pro[] build(): non = {\n",
        "    var graph = .build().graph();\n",
        "    var core = graph.dependency({ alias = \"core\", package = \"org/core\" });\n",
        "    var module = core.module(\"root\");\n",
        "    var artifact = core.artifact(\"corelib\");\n",
        "    return;\n",
        "}\n",
    );
    let (package_root, build_path) = temp_build_package(source);
    let request = BuildEvaluationRequest {
        package_root: package_root.display().to_string(),
        inputs: BuildEvaluationInputs {
            working_directory: package_root.display().to_string(),
            ..BuildEvaluationInputs::default()
        },
        operations: Vec::new(),
    };

    let evaluated = evaluate_build_source(&request, &build_path, source)
        .expect("dependency queries should evaluate")
        .expect("build body should produce a graph");

    assert_eq!(evaluated.evaluated.dependency_queries.len(), 2);
    assert!(evaluated
        .evaluated
        .dependency_queries
        .iter()
        .any(|query| query.dependency_alias == "core"
            && query.query_name == "root"
            && query.kind == BuildRuntimeDependencyQueryKind::Module));
    assert!(evaluated
        .evaluated
        .dependency_queries
        .iter()
        .any(|query| query.dependency_alias == "core"
            && query.query_name == "corelib"
            && query.kind == BuildRuntimeDependencyQueryKind::Artifact));
}

#[test]
fn build_source_evaluator_records_dependency_step_and_generated_queries() {
    let source = concat!(
        "pro[] build(): non = {\n",
        "    var graph = .build().graph();\n",
        "    var core = graph.dependency({ alias = \"core\", package = \"org/core\" });\n",
        "    var step = core.step(\"check\");\n",
        "    var generated = core.generated(\"bindings\");\n",
        "    return;\n",
        "}\n",
    );
    let (package_root, build_path) = temp_build_package(source);
    let request = BuildEvaluationRequest {
        package_root: package_root.display().to_string(),
        inputs: BuildEvaluationInputs {
            working_directory: package_root.display().to_string(),
            ..BuildEvaluationInputs::default()
        },
        operations: Vec::new(),
    };

    let evaluated = evaluate_build_source(&request, &build_path, source)
        .expect("dependency queries should evaluate")
        .expect("build body should produce a graph");

    assert_eq!(evaluated.evaluated.dependency_queries.len(), 2);
    assert!(evaluated
        .evaluated
        .dependency_queries
        .iter()
        .any(|query| query.dependency_alias == "core"
            && query.query_name == "check"
            && query.kind == BuildRuntimeDependencyQueryKind::Step));
    assert!(evaluated
        .evaluated
        .dependency_queries
        .iter()
        .any(|query| query.dependency_alias == "core"
            && query.query_name == "bindings"
            && query.kind == BuildRuntimeDependencyQueryKind::GeneratedOutput));
}

#[test]
fn build_source_evaluator_records_step_descriptions() {
    let source = concat!(
        "pro[] build(): non = {\n",
        "    var graph = .build().graph();\n",
        "    var docs = graph.step(\"docs\", \"Generate documentation\");\n",
        "    return;\n",
        "}\n",
    );
    let (package_root, build_path) = temp_build_package(source);
    let request = BuildEvaluationRequest {
        package_root: package_root.display().to_string(),
        inputs: BuildEvaluationInputs {
            working_directory: package_root.display().to_string(),
            ..BuildEvaluationInputs::default()
        },
        operations: Vec::new(),
    };

    let evaluated = evaluate_build_source(&request, &build_path, source)
        .expect("step descriptions should evaluate")
        .expect("build body should produce a graph");

    let docs = evaluated
        .result
        .graph
        .steps()
        .iter()
        .find(|step| step.name == "docs")
        .expect("docs step should exist");
    assert_eq!(docs.description.as_deref(), Some("Generate documentation"));
}

#[test]
fn build_source_evaluator_keeps_full_dependency_surface_usage_together() {
    let source = concat!(
        "pro[] build(): non = {\n",
        "    var graph = .build().graph();\n",
        "    var dep = graph.dependency({ alias = \"core\", package = \"org/core\", mode = \"on-demand\" });\n",
        "    var module = dep.module(\"root\");\n",
        "    var artifact = dep.artifact(\"corelib\");\n",
        "    var step = dep.step(\"check\");\n",
        "    var generated = dep.generated(\"bindings\");\n",
        "    return;\n",
        "}\n",
    );
    let (package_root, build_path) = temp_build_package(source);
    let request = BuildEvaluationRequest {
        package_root: package_root.display().to_string(),
        inputs: BuildEvaluationInputs {
            working_directory: package_root.display().to_string(),
            ..BuildEvaluationInputs::default()
        },
        operations: Vec::new(),
    };

    let evaluated = evaluate_build_source(&request, &build_path, source)
        .expect("dependency surface should evaluate")
        .expect("build body should produce a graph");
    let query_kinds = evaluated
        .evaluated
        .dependency_queries
        .iter()
        .map(|query| query.kind)
        .collect::<Vec<_>>();

    assert_eq!(evaluated.evaluated.dependencies.len(), 1);
    assert_eq!(
        evaluated.evaluated.dependencies[0].evaluation_mode,
        Some(crate::DependencyBuildEvaluationMode::OnDemand)
    );
    assert_eq!(evaluated.evaluated.dependency_queries.len(), 4);
    assert!(query_kinds.contains(&BuildRuntimeDependencyQueryKind::Module));
    assert!(query_kinds.contains(&BuildRuntimeDependencyQueryKind::Artifact));
    assert!(query_kinds.contains(&BuildRuntimeDependencyQueryKind::Step));
    assert!(query_kinds.contains(&BuildRuntimeDependencyQueryKind::GeneratedOutput));
}

#[test]
fn build_source_evaluator_keeps_dependency_queries_precise_for_build_add_dep_handles() {
    let source = concat!(
        "pro[] build(): non = {\n",
        "    var build = .build();\n",
        "    build.meta({ name = \"demo\", version = \"0.1.0\" });\n",
        "    var dep = build.add_dep({ alias = \"core\", source = \"pkg\", target = \"core\" });\n",
        "    var module = dep.module(\"root\");\n",
        "    var artifact = dep.artifact(\"corelib\");\n",
        "    var step = dep.step(\"check\");\n",
        "    var generated = dep.generated(\"bindings\");\n",
        "    return;\n",
        "}\n",
    );
    let (package_root, build_path) = temp_build_package(source);
    let request = BuildEvaluationRequest {
        package_root: package_root.display().to_string(),
        inputs: BuildEvaluationInputs {
            working_directory: package_root.display().to_string(),
            ..BuildEvaluationInputs::default()
        },
        operations: Vec::new(),
    };

    let evaluated = evaluate_build_source(&request, &build_path, source)
        .expect("build.add_dep dependency queries should evaluate")
        .expect("build body should produce a graph");

    assert_eq!(evaluated.evaluated.dependencies.len(), 1);
    assert_eq!(evaluated.evaluated.dependencies[0].alias, "core");
    assert_eq!(evaluated.evaluated.dependency_queries.len(), 4);
    assert!(evaluated
        .evaluated
        .dependency_queries
        .iter()
        .any(|query| query.dependency_alias == "core"
            && query.query_name == "root"
            && query.kind == BuildRuntimeDependencyQueryKind::Module));
    assert!(evaluated
        .evaluated
        .dependency_queries
        .iter()
        .any(|query| query.dependency_alias == "core"
            && query.query_name == "corelib"
            && query.kind == BuildRuntimeDependencyQueryKind::Artifact));
    assert!(evaluated
        .evaluated
        .dependency_queries
        .iter()
        .any(|query| query.dependency_alias == "core"
            && query.query_name == "check"
            && query.kind == BuildRuntimeDependencyQueryKind::Step));
    assert!(evaluated
        .evaluated
        .dependency_queries
        .iter()
        .any(|query| query.dependency_alias == "core"
            && query.query_name == "bindings"
            && query.kind == BuildRuntimeDependencyQueryKind::GeneratedOutput));
}

#[test]
fn build_source_evaluator_resolves_deferred_artifact_option_values_into_runtime_metadata() {
    let source = concat!(
        "pro[] build(): non = {\n",
        "    var graph = .build().graph();\n",
        "    var root = graph.option({ name = \"root\", kind = \"path\", default = \"src/demo.fol\" });\n",
        "    var target = graph.standard_target();\n",
        "    var optimize = graph.standard_optimize();\n",
        "    graph.add_exe({ name = \"demo\", root = root, target = target, optimize = optimize });\n",
        "    return;\n",
        "}\n",
    );
    let (package_root, build_path) = temp_build_package(source);
    let request = BuildEvaluationRequest {
        package_root: package_root.display().to_string(),
        inputs: BuildEvaluationInputs {
            working_directory: package_root.display().to_string(),
            target: BuildTargetTriple::parse("x86_64-linux-gnu"),
            optimize: BuildOptimizeMode::parse("release-fast"),
            ..BuildEvaluationInputs::default()
        },
        operations: Vec::new(),
    };

    let evaluated = evaluate_build_source(&request, &build_path, source)
        .expect("deferred artifact configs should evaluate")
        .expect("build body should produce operations");

    let artifact = evaluated
        .evaluated
        .artifacts
        .iter()
        .find(|artifact| artifact.name == "demo")
        .expect("artifact should exist");

    assert_eq!(artifact.root_module, "src/demo.fol");
    assert_eq!(artifact.target.as_deref(), Some("x86_64-linux-gnu"));
    assert_eq!(artifact.optimize.as_deref(), Some("release-fast"));
}

#[test]
fn build_source_evaluator_applies_build_inputs_and_option_overrides_to_artifact_metadata() {
    let source = concat!(
        "pro[] build(): non = {\n",
        "    var graph = .build().graph();\n",
        "    var root = graph.option({ name = \"root\", kind = \"path\", default = \"src/default.fol\" });\n",
        "    var target = graph.standard_target();\n",
        "    var optimize = graph.standard_optimize();\n",
        "    var app = graph.add_exe({ name = \"demo\", root = root, target = target, optimize = optimize });\n",
        "    graph.add_run(app);\n",
        "    return;\n",
        "}\n",
    );
    let (package_root, build_path) = temp_build_package(source);
    let mut inputs = BuildEvaluationInputs {
        working_directory: package_root.display().to_string(),
        target: BuildTargetTriple::parse("aarch64-macos-gnu"),
        optimize: BuildOptimizeMode::parse("release-small"),
        ..BuildEvaluationInputs::default()
    };
    inputs
        .options
        .insert("root".to_string(), "src/cli-selected.fol".to_string());
    let request = BuildEvaluationRequest {
        package_root: package_root.display().to_string(),
        inputs,
        operations: Vec::new(),
    };

    let evaluated = evaluate_build_source(&request, &build_path, source)
        .expect("build inputs should flow into artifact metadata")
        .expect("build body should produce operations");

    let artifact = evaluated
        .evaluated
        .artifacts
        .iter()
        .find(|artifact| artifact.name == "demo")
        .expect("artifact should exist");

    assert_eq!(artifact.root_module, "src/cli-selected.fol");
    assert_eq!(artifact.target.as_deref(), Some("aarch64-macos-gnu"));
    assert_eq!(artifact.optimize.as_deref(), Some("release-small"));
}

#[test]
fn build_source_evaluator_rejects_invalid_artifact_target_shapes_with_exact_diagnostics() {
    let source = concat!(
        "pro[] build(): non = {\n",
        "    var graph = .build().graph();\n",
        "    graph.add_exe({ name = \"demo\", root = \"src/main.fol\", target = graph });\n",
        "    return;\n",
        "}\n",
    );
    let (package_root, build_path) = temp_build_package(source);
    let request = BuildEvaluationRequest {
        package_root: package_root.display().to_string(),
        inputs: BuildEvaluationInputs {
            working_directory: package_root.display().to_string(),
            ..BuildEvaluationInputs::default()
        },
        operations: Vec::new(),
    };

    let error = evaluate_build_source(&request, &build_path, source)
        .expect_err("invalid artifact target shape should fail");

    assert_eq!(
        error.message(),
        "add_exe config is invalid: artifact 'target' must be a target handle or string triple"
    );
}

#[test]
fn build_source_evaluator_rejects_empty_artifact_roots_with_exact_diagnostics() {
    let source = concat!(
        "pro[] build(): non = {\n",
        "    var graph = .build().graph();\n",
        "    graph.add_exe({ name = \"demo\", root = \"\" });\n",
        "    return;\n",
        "}\n",
    );
    let (package_root, build_path) = temp_build_package(source);
    let request = BuildEvaluationRequest {
        package_root: package_root.display().to_string(),
        inputs: BuildEvaluationInputs {
            working_directory: package_root.display().to_string(),
            ..BuildEvaluationInputs::default()
        },
        operations: Vec::new(),
    };

    let error = evaluate_build_source(&request, &build_path, source)
        .expect_err("empty artifact root should fail");

    assert_eq!(
        error.message(),
        "add_exe config is invalid: artifact 'root' must not be empty"
    );
}

#[test]
fn build_source_evaluator_keeps_explicit_dependency_exports_precise() {
    let source = concat!(
        "pro[] build(): non = {\n",
        "    var build = .build();\n",
        "    build.meta({ name = \"demo\", version = \"0.1.0\" });\n",
        "    var graph = build.graph();\n",
        "    var codec = graph.add_module({ name = \"codec\", root = \"src/codec.fol\" });\n",
        "    var app = graph.add_static_lib({ name = \"demo\", root = \"src/main.fol\" });\n",
        "    var docs = graph.step(\"docs\");\n",
        "    var bindings = graph.write_file({ name = \"bindings\", path = \"gen/bindings.fol\", contents = \"ok\" });\n",
        "    build.export_module({ name = \"api\", module = codec });\n",
        "    build.export_artifact({ name = \"runtime\", artifact = app });\n",
        "    build.export_step({ name = \"check\", step = docs });\n",
        "    build.export_output({ name = \"schema\", output = bindings });\n",
        "    return;\n",
        "}\n",
    );
    let (package_root, build_path) = temp_build_package(source);
    let request = BuildEvaluationRequest {
        package_root: package_root.display().to_string(),
        inputs: BuildEvaluationInputs {
            working_directory: package_root.display().to_string(),
            ..BuildEvaluationInputs::default()
        },
        operations: Vec::new(),
    };

    let evaluated = evaluate_build_source(&request, &build_path, source)
        .expect("explicit exports should evaluate")
        .expect("build body should produce operations");

    assert_eq!(evaluated.evaluated.dependency_exports.len(), 4);
    assert!(evaluated
        .evaluated
        .dependency_exports
        .iter()
        .any(|export| export.name == "api"
            && export.target_name == "codec"
            && export.kind == crate::runtime::BuildRuntimeDependencyExportKind::Module));
    assert!(evaluated
        .evaluated
        .dependency_exports
        .iter()
        .any(|export| export.name == "runtime"
            && export.target_name == "demo"
            && export.kind == crate::runtime::BuildRuntimeDependencyExportKind::Artifact));
    assert!(evaluated
        .evaluated
        .dependency_exports
        .iter()
        .any(|export| export.name == "check"
            && export.target_name == "docs"
            && export.kind == crate::runtime::BuildRuntimeDependencyExportKind::Step));
    assert!(evaluated
        .evaluated
        .dependency_exports
        .iter()
        .any(|export| export.name == "schema"
            && export.target_name == "bindings"
            && export.kind == crate::runtime::BuildRuntimeDependencyExportKind::GeneratedOutput));
}

#[test]
fn build_source_evaluator_rejects_duplicate_export_names_per_kind() {
    let source = concat!(
        "pro[] build(): non = {\n",
        "    var build = .build();\n",
        "    var graph = build.graph();\n",
        "    var codec = graph.add_module({ name = \"codec\", root = \"src/codec.fol\" });\n",
        "    var other = graph.add_module({ name = \"other\", root = \"src/other.fol\" });\n",
        "    build.export_module({ name = \"api\", module = codec });\n",
        "    build.export_module({ name = \"api\", module = other });\n",
        "    return;\n",
        "}\n",
    );
    let (package_root, build_path) = temp_build_package(source);
    let request = BuildEvaluationRequest {
        package_root: package_root.display().to_string(),
        inputs: BuildEvaluationInputs {
            working_directory: package_root.display().to_string(),
            ..BuildEvaluationInputs::default()
        },
        operations: Vec::new(),
    };

    let error = evaluate_build_source(&request, &build_path, source)
        .expect_err("duplicate export names should fail");

    assert_eq!(
        error.message(),
        "export_module config is invalid: duplicate exported module name 'api'"
    );
}

#[test]
fn build_source_evaluator_rejects_export_kind_handle_mismatches() {
    let source = concat!(
        "pro[] build(): non = {\n",
        "    var build = .build();\n",
        "    var graph = build.graph();\n",
        "    var codec = graph.add_module({ name = \"codec\", root = \"src/codec.fol\" });\n",
        "    build.export_artifact({ name = \"runtime\", artifact = codec });\n",
        "    return;\n",
        "}\n",
    );
    let (package_root, build_path) = temp_build_package(source);
    let request = BuildEvaluationRequest {
        package_root: package_root.display().to_string(),
        inputs: BuildEvaluationInputs {
            working_directory: package_root.display().to_string(),
            ..BuildEvaluationInputs::default()
        },
        operations: Vec::new(),
    };

    let error = evaluate_build_source(&request, &build_path, source)
        .expect_err("export kind mismatches should fail");

    assert_eq!(
        error.message(),
        "build.export_artifact config is invalid: build.export_artifact requires handle field 'artifact'"
    );
}

#[test]
fn build_source_evaluator_rejects_copy_file_with_source_dir_handle() {
    let source = concat!(
        "pro[] build(): non = {\n",
        "    var graph = .build().graph();\n",
        "    var assets = graph.dir_from_root(\"assets\");\n",
        "    graph.copy_file({ name = \"asset\", source = assets, path = \"gen/logo.svg\" });\n",
        "    return;\n",
        "}\n",
    );
    let (package_root, build_path) = temp_build_package(source);
    let request = BuildEvaluationRequest {
        package_root: package_root.display().to_string(),
        inputs: BuildEvaluationInputs {
            working_directory: package_root.display().to_string(),
            ..BuildEvaluationInputs::default()
        },
        operations: Vec::new(),
    };

    let error = evaluate_build_source(&request, &build_path, source)
        .expect_err("copy_file should reject source-dir handles");

    assert_eq!(
        error.message(),
        "copy_file config is invalid: 'source' must be a source-file handle, not a source-dir handle"
    );
}

#[test]
fn build_source_evaluator_rejects_install_dir_with_source_file_handle() {
    let source = concat!(
        "pro[] build(): non = {\n",
        "    var graph = .build().graph();\n",
        "    var defaults = graph.file_from_root(\"config/defaults.toml\");\n",
        "    graph.install_dir({ name = \"assets\", source = defaults });\n",
        "    return;\n",
        "}\n",
    );
    let (package_root, build_path) = temp_build_package(source);
    let request = BuildEvaluationRequest {
        package_root: package_root.display().to_string(),
        inputs: BuildEvaluationInputs {
            working_directory: package_root.display().to_string(),
            ..BuildEvaluationInputs::default()
        },
        operations: Vec::new(),
    };

    let error = evaluate_build_source(&request, &build_path, source)
        .expect_err("install_dir should reject source-file handles");

    assert_eq!(
        error.message(),
        "install_dir config is invalid: 'source' must be a source-dir handle, not a source-file handle"
    );
}

#[test]
fn build_source_evaluator_rejects_run_add_file_arg_with_source_dir_handle() {
    let source = concat!(
        "pro[] build(): non = {\n",
        "    var graph = .build().graph();\n",
        "    var app = graph.add_exe({ name = \"demo\", root = \"src/main.fol\" });\n",
        "    var run = graph.add_run(app);\n",
        "    var assets = graph.dir_from_root(\"assets\");\n",
        "    run.add_file_arg(assets);\n",
        "    return;\n",
        "}\n",
    );
    let (package_root, build_path) = temp_build_package(source);
    let request = BuildEvaluationRequest {
        package_root: package_root.display().to_string(),
        inputs: BuildEvaluationInputs {
            working_directory: package_root.display().to_string(),
            ..BuildEvaluationInputs::default()
        },
        operations: Vec::new(),
    };

    let error = evaluate_build_source(&request, &build_path, source)
        .expect_err("run.add_file_arg should reject source-dir handles");

    assert_eq!(
        error.message(),
        "add_file_arg config is invalid: run.add_file_arg requires a source-file handle or generated-output handle, not a source-dir handle"
    );
}
