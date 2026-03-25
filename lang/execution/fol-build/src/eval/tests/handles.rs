use super::super::{
    canonical_graph_construction_capabilities, evaluate_build_source,
    BuildEvaluationBoundary, BuildEvaluationErrorKind, BuildEvaluationInputs,
    BuildEvaluationRequest, BuildRuntimeCapabilityModel, BuildEvaluationResult,
    EvaluatedBuildProgram,
};
use crate::graph::BuildGraph;
use crate::option::{BuildOptionDeclarationSet, ResolvedBuildOptionSet};
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

fn temp_build_package(source: &str) -> (PathBuf, PathBuf) {
    static NEXT_ID: AtomicU64 = AtomicU64::new(0);

    let package_root = std::env::temp_dir().join(format!(
        "fol_build_eval_hdl_{}_{}",
        std::process::id(),
        NEXT_ID.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir_all(&package_root).expect("temp package root should be created");
    fs::write(
        package_root.join("build.fol"),
        "name: build-eval\nversion: 1.0.0\n",
    )
    .expect("package metadata should be written");
    fs::write(package_root.join("build.fol"), source).expect("build source should be written");
    (package_root.clone(), package_root.join("build.fol"))
}

#[test]
fn build_source_evaluator_accepts_build_metadata_and_dependency_calls() {
    let source = concat!(
        "pro[] build(): non = {\n",
        "    var build = .build();\n",
        "    build.meta({ name = \"demo\", version = \"0.1.0\" });\n",
        "    var logtiny = build.add_dep({ alias = \"logtiny\", source = \"git\", target = \"git+https://example.com/logtiny\" });\n",
        "    var graph = build.graph();\n",
        "    var app = graph.add_exe({ name = \"demo\", root = \"src/main.fol\" });\n",
        "    graph.install(app);\n",
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
        .expect("build metadata calls should evaluate")
        .expect("build body should produce operations");

    assert_eq!(evaluated.result.graph.artifacts().len(), 1);
    assert_eq!(evaluated.result.dependency_requests.len(), 1);
    assert_eq!(evaluated.result.dependency_requests[0].alias, "logtiny");
    assert_eq!(
        evaluated.result.dependency_requests[0].package,
        "git+https://example.com/logtiny"
    );
}

#[test]
fn build_source_evaluator_treats_add_dep_as_a_real_dependency_handle() {
    let source = concat!(
        "pro[] build(): non = {\n",
        "    var build = .build();\n",
        "    build.meta({ name = \"demo\", version = \"0.1.0\" });\n",
        "    var dep = build.add_dep({ alias = \"core\", source = \"pkg\", target = \"core\" });\n",
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
        .expect("build dependency handles should evaluate")
        .expect("build body should produce operations");

    assert_eq!(evaluated.result.dependency_requests.len(), 1);
    assert_eq!(evaluated.result.dependency_requests[0].alias, "core");
    assert!(evaluated
        .evaluated
        .dependency_queries
        .iter()
        .any(|query| query.dependency_alias == "core"
            && query.query_name == "bindings"
            && query.kind == crate::runtime::BuildRuntimeDependencyQueryKind::GeneratedOutput));
}

#[test]
fn build_source_evaluator_allows_dependency_modules_to_feed_back_into_artifact_imports() {
    let source = concat!(
        "pro[] build(): non = {\n",
        "    var build = .build();\n",
        "    build.meta({ name = \"demo\", version = \"0.1.0\" });\n",
        "    var dep = build.add_dep({ alias = \"core\", source = \"pkg\", target = \"core\" });\n",
        "    var graph = build.graph();\n",
        "    var app = graph.add_exe({ name = \"demo\", root = \"src/main.fol\" });\n",
        "    app.import(dep.module(\"root\"));\n",
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
        .expect("dependency module imports should evaluate")
        .expect("build body should produce operations");

    assert_eq!(evaluated.result.graph.artifacts().len(), 1);
    assert!(evaluated
        .evaluated
        .dependency_queries
        .iter()
        .any(|query| query.dependency_alias == "core"
            && query.query_name == "root"
            && query.kind == crate::runtime::BuildRuntimeDependencyQueryKind::Module));
    assert!(evaluated
        .result
        .graph
        .modules()
        .iter()
        .any(|module| module.name == "dep:core:root"));
}

#[test]
fn build_source_evaluator_keeps_generated_handles_through_local_helpers() {
    let source = concat!(
        "fun[] emit_cfg() = {\n",
        "    var graph = .build().graph();\n",
        "    return graph.write_file({ name = \"cfg\", path = \"config/generated.toml\", contents = \"ok\" });\n",
        "}\n",
        "pro[] build(): non = {\n",
        "    var graph = .build().graph();\n",
        "    var app = graph.add_exe({ name = \"demo\", root = \"src/main.fol\" });\n",
        "    var run = graph.add_run(app);\n",
        "    var cfg = emit_cfg();\n",
        "    app.add_generated(cfg);\n",
        "    run.add_file_arg(cfg);\n",
        "    graph.install_file({ name = \"install-cfg\", source = cfg });\n",
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
        .expect("helper output handles should evaluate")
        .expect("build body should produce operations");

    assert_eq!(evaluated.result.graph.generated_files().len(), 1);
    assert_eq!(evaluated.result.graph.installs().len(), 1);
    let artifact_inputs = evaluated
        .result
        .graph
        .artifact_inputs_for(crate::graph::BuildArtifactId(0))
        .collect::<Vec<_>>();
    assert!(artifact_inputs.iter().any(|input| matches!(
        input,
        crate::graph::BuildArtifactInput::GeneratedFile(crate::graph::BuildGeneratedFileId(0))
    )));
    let run_cfg = evaluated
        .result
        .graph
        .run_config_for(crate::graph::BuildStepId(0))
        .expect("run config should exist");
    assert!(run_cfg.args.iter().any(|arg| matches!(
        arg,
        crate::graph::BuildRunArg::GeneratedFile(crate::graph::BuildGeneratedFileId(0))
    )));
}

#[test]
fn build_source_evaluator_extracts_and_replays_restricted_build_bodies() {
    let source = concat!(
        "pro[] build(): non = {\n",
        "    var graph = .graph();\n",
        "    graph.add_exe(\"app\", \"src/app.fol\");\n",
        "    graph.add_test(\"app_test\", \"test/app.fol\");\n",
        "    graph.add_run(\"serve\", \"app\");\n",
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
        .expect("restricted build body should evaluate")
        .expect("build body should produce operations");

    assert_eq!(evaluated.evaluated.artifacts.len(), 2);
    assert!(evaluated
        .evaluated
        .artifacts
        .iter()
        .any(|artifact| artifact.root_module == "src/app.fol"));
    assert!(evaluated
        .evaluated
        .step_bindings
        .iter()
        .any(|binding| binding.step_name == "serve"));
    assert_eq!(evaluated.result.graph.artifacts().len(), 2);
    assert_eq!(evaluated.result.graph.steps().len(), 1);
}

#[test]
fn build_source_evaluator_supports_object_style_artifacts_and_handle_calls() {
    let source = concat!(
        "pro[] build(): non = {\n",
        "    var graph = .graph();\n",
        "    var target = graph.standard_target();\n",
        "    var optimize = graph.standard_optimize();\n",
        "    var app = graph.add_exe({\n",
        "        name = \"demo\",\n",
        "        root = \"src/demo.fol\",\n",
        "        target = target,\n",
        "        optimize = optimize,\n",
        "    });\n",
        "    graph.install(app);\n",
        "    graph.add_run(app);\n",
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
        .expect("object style build body should evaluate")
        .expect("build body should produce operations");

    assert_eq!(evaluated.evaluated.artifacts.len(), 1);
    assert!(evaluated
        .evaluated
        .artifacts
        .iter()
        .any(|artifact| artifact.name == "demo" && artifact.root_module == "src/demo.fol"));
    assert!(evaluated
        .evaluated
        .step_bindings
        .iter()
        .any(|binding| binding.step_name == "run"));
    assert_eq!(evaluated.result.graph.artifacts().len(), 1);
    assert_eq!(evaluated.result.graph.installs().len(), 1);
    let mut step_names = evaluated
        .result
        .graph
        .steps()
        .iter()
        .map(|step| step.name.as_str())
        .collect::<Vec<_>>();
    step_names.sort_unstable();
    assert_eq!(step_names, vec!["install", "run"]);
}

#[test]
fn build_source_evaluator_supports_user_option_record_configs() {
    let source = concat!(
        "pro[] build(): non = {\n",
        "    var graph = .graph();\n",
        "    var strip = graph.option({ name = \"strip\", kind = \"bool\", default = false });\n",
        "    var jobs = graph.option({ name = \"jobs\", kind = \"int\", default = 8 });\n",
        "    var flavor = graph.option({ name = \"flavor\", kind = \"enum\", default = \"fast\" });\n",
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
        .expect("user option configs should evaluate")
        .expect("build body should produce operations");

    assert_eq!(evaluated.result.option_declarations.declarations().len(), 3);
    assert_eq!(
        evaluated.result.resolved_options.get("strip"),
        Some("false")
    );
    assert_eq!(evaluated.result.resolved_options.get("jobs"), Some("8"));
    assert_eq!(
        evaluated.result.resolved_options.get("flavor"),
        Some("fast")
    );
}

#[test]
fn build_source_evaluator_reuses_bound_run_and_install_handles_as_step_dependencies() {
    let source = concat!(
        "pro[] build(): non = {\n",
        "    var graph = .graph();\n",
        "    var app = graph.add_exe(\"demo\", \"src/demo.fol\");\n",
        "    var run_app = graph.add_run(app);\n",
        "    var install_app = graph.install(app);\n",
        "    graph.step(\"bundle\", run_app, install_app);\n",
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
        .expect("bound step-like handles should evaluate")
        .expect("build body should produce operations");

    let bundle = evaluated
        .result
        .graph
        .steps()
        .iter()
        .find(|step| step.name == "bundle")
        .expect("bundle step should exist");
    let dependencies = evaluated
        .result
        .graph
        .step_dependencies_for(bundle.id)
        .collect::<Vec<_>>();
    let dependency_names = dependencies
        .iter()
        .filter_map(|id| evaluated.result.graph.steps().get(id.index()))
        .map(|step| step.name.as_str())
        .collect::<Vec<_>>();

    assert_eq!(dependency_names, vec!["run", "install"]);
}

#[test]
fn build_source_evaluator_rejects_unknown_handle_methods_explicitly() {
    let source = concat!(
        "pro[] build(): non = {\n",
        "    var graph = .graph();\n",
        "    var docs = graph.step(\"docs\");\n",
        "    docs.finish(docs);\n",
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
        .expect_err("unsupported handle methods should fail explicitly");

    assert_eq!(error.kind(), BuildEvaluationErrorKind::Unsupported);
    assert!(error.message().contains("finish"));
}

#[test]
fn build_source_evaluator_supports_step_handle_depend_on_chains() {
    let source = concat!(
        "pro[] build(): non = {\n",
        "    var graph = .graph();\n",
        "    var lint = graph.step(\"lint\");\n",
        "    graph.step(\"docs\").depend_on(lint);\n",
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
        .expect("step-handle chaining should evaluate")
        .expect("build body should produce operations");

    let docs = evaluated
        .result
        .graph
        .steps()
        .iter()
        .find(|step| step.name == "docs")
        .expect("docs step should exist");
    let lint = evaluated
        .result
        .graph
        .steps()
        .iter()
        .find(|step| step.name == "lint")
        .expect("lint step should exist");
    assert_eq!(
        evaluated
            .result
            .graph
            .step_dependencies_for(docs.id)
            .collect::<Vec<_>>(),
        vec![lint.id]
    );
}

#[test]
fn build_source_evaluator_supports_run_handle_depend_on_chains() {
    let source = concat!(
        "pro[] build(): non = {\n",
        "    var graph = .graph();\n",
        "    var lint = graph.step(\"lint\");\n",
        "    var app = graph.add_exe({ name = \"app\", root = \"src/app.fol\" });\n",
        "    graph.add_run(app).depend_on(lint);\n",
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
        .expect("run-handle chaining should evaluate")
        .expect("build body should produce operations");

    let run = evaluated
        .result
        .graph
        .steps()
        .iter()
        .find(|step| step.name == "run")
        .expect("run step should exist");
    let lint = evaluated
        .result
        .graph
        .steps()
        .iter()
        .find(|step| step.name == "lint")
        .expect("lint step should exist");
    assert_eq!(
        evaluated
            .result
            .graph
            .step_dependencies_for(run.id)
            .collect::<Vec<_>>(),
        vec![lint.id]
    );
}

#[test]
fn build_source_evaluator_supports_install_handle_depend_on_chains() {
    let source = concat!(
        "pro[] build(): non = {\n",
        "    var graph = .graph();\n",
        "    var lint = graph.step(\"lint\");\n",
        "    var app = graph.add_exe({ name = \"app\", root = \"src/app.fol\" });\n",
        "    graph.install(app).depend_on(lint);\n",
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
        .expect("install-handle chaining should evaluate")
        .expect("build body should produce operations");

    let install = evaluated
        .result
        .graph
        .steps()
        .iter()
        .find(|step| step.name == "install")
        .expect("install step should exist");
    let lint = evaluated
        .result
        .graph
        .steps()
        .iter()
        .find(|step| step.name == "lint")
        .expect("lint step should exist");
    assert_eq!(
        evaluated
            .result
            .graph
            .step_dependencies_for(install.id)
            .collect::<Vec<_>>(),
        vec![lint.id]
    );
}

#[test]
fn build_source_evaluator_keeps_step_like_handle_chains_stable() {
    let source = concat!(
        "pro[] build(): non = {\n",
        "    var graph = .graph();\n",
        "    var lint = graph.step(\"lint\");\n",
        "    var app = graph.add_exe({ name = \"app\", root = \"src/app.fol\" });\n",
        "    var run_app = graph.add_run(app);\n",
        "    var install_app = graph.install(app);\n",
        "    run_app.depend_on(lint);\n",
        "    install_app.depend_on(lint);\n",
        "    graph.step(\"bundle\", run_app, install_app, run_app).depend_on(lint);\n",
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
        .expect("combined handle chaining should evaluate")
        .expect("build body should produce operations");

    let lint = evaluated
        .result
        .graph
        .steps()
        .iter()
        .find(|step| step.name == "lint")
        .expect("lint step should exist");
    let run = evaluated
        .result
        .graph
        .steps()
        .iter()
        .find(|step| step.name == "run")
        .expect("run step should exist");
    let install = evaluated
        .result
        .graph
        .steps()
        .iter()
        .find(|step| step.name == "install")
        .expect("install step should exist");
    let bundle = evaluated
        .result
        .graph
        .steps()
        .iter()
        .find(|step| step.name == "bundle")
        .expect("bundle step should exist");

    assert_eq!(
        evaluated
            .result
            .graph
            .step_dependencies_for(run.id)
            .collect::<Vec<_>>(),
        vec![lint.id]
    );
    assert_eq!(
        evaluated
            .result
            .graph
            .step_dependencies_for(install.id)
            .collect::<Vec<_>>(),
        vec![lint.id]
    );
    assert_eq!(
        evaluated
            .result
            .graph
            .step_dependencies_for(bundle.id)
            .collect::<Vec<_>>(),
        vec![run.id, install.id, lint.id]
    );
}

#[test]
fn evaluated_build_program_surface_keeps_runtime_metadata_and_graph_result() {
    let result = BuildEvaluationResult::new(
        BuildEvaluationBoundary::GraphConstructionSubset,
        canonical_graph_construction_capabilities(),
        "/pkg",
        BuildOptionDeclarationSet::new(),
        ResolvedBuildOptionSet::new(),
        Vec::new(),
        BuildGraph::new(),
    );
    let evaluated = EvaluatedBuildProgram {
        program: crate::runtime::BuildRuntimeProgram::new(
            crate::runtime::BuildExecutionRepresentation::RestrictedRuntimeIr,
        ),
        artifacts: vec![crate::runtime::BuildRuntimeArtifact::new(
            "app",
            crate::runtime::BuildRuntimeArtifactKind::Executable,
            "src/app.fol",
        )],
        generated_files: vec![crate::runtime::BuildRuntimeGeneratedFile::new(
            "version",
            "gen/version.fol",
            crate::runtime::BuildRuntimeGeneratedFileKind::Write,
        )],
        dependencies: vec![crate::runtime::BuildRuntimeDependency {
            alias: "core".to_string(),
            package: "org/core".to_string(),
            evaluation_mode: None,
        }],
        dependency_queries: Vec::new(),
        step_bindings: vec![crate::runtime::BuildRuntimeStepBinding::new(
            "run",
            crate::runtime::BuildRuntimeStepBindingKind::DefaultRun,
            Some("app"),
        )],
        result,
    };

    assert_eq!(evaluated.artifacts.len(), 1);
    assert_eq!(evaluated.generated_files.len(), 1);
    assert_eq!(evaluated.dependencies.len(), 1);
    assert_eq!(evaluated.step_bindings.len(), 1);
    assert_eq!(evaluated.result.package_root, "/pkg");
}
