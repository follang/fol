use super::super::{
    execute_workspace_build_route, plan_member_execution, plan_workspace_build_route,
    FrontendBuildWorkflowMode, FrontendMemberBuildRoute, FrontendStepExecutionKind,
    FrontendWorkspaceBuildRequest,
};
use crate::{FrontendConfig, FrontendProfile, FrontendWorkspace, PackageRoot, WorkspaceRoot};
use std::fs;

#[test]
fn cli_selected_custom_graph_steps_flow_into_the_routed_member_plan() {
    let root = std::env::temp_dir().join(format!(
        "fol_frontend_build_route_cli_step_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time before epoch")
            .as_nanos()
    ));
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("package.yaml"), "name: demo\nversion: 0.1.0\n").unwrap();
    fs::write(
        root.join("build.fol"),
        concat!(
            "pro[] build(graph: Graph): non = {\n",
            "    graph.step(\"docs\");\n",
            "    return graph\n",
            "};\n",
        ),
    )
    .unwrap();
    fs::write(
        root.join("src/main.fol"),
        "fun[] main(): int = {\n    return 0\n};\n",
    )
    .unwrap();
    let requested_step = super::super::requested_workspace_step(
        &crate::CodeSubcommand::Build(crate::BuildCommand::default()),
        Some("docs"),
    );
    let plan = plan_member_execution(&FrontendMemberBuildRoute {
        member_root: root.clone(),
        package_name: "demo".to_string(),
        mode: FrontendBuildWorkflowMode::Modern,
    }, &FrontendConfig::default())
    .expect("member planning should surface the custom docs step");

    assert_eq!(requested_step, "docs");
    assert!(plan.steps.iter().any(|step| step.name == "docs"));

    fs::remove_dir_all(root).ok();
}

#[test]
fn custom_run_steps_plan_as_run_execution() {
    let root = std::env::temp_dir().join(format!(
        "fol_frontend_build_route_custom_run_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time before epoch")
            .as_nanos()
    ));
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("package.yaml"), "name: demo\nversion: 0.1.0\n").unwrap();
    fs::write(
        root.join("build.fol"),
        concat!(
            "pro[] build(graph: Graph): non = {\n",
            "    graph.add_exe(\"app\", \"src/main.fol\");\n",
            "    graph.add_run(\"serve\", \"app\");\n",
            "    return graph\n",
            "};\n",
        ),
    )
    .unwrap();
    fs::write(
        root.join("src/main.fol"),
        "fun[] main(): int = {\n    return 0\n};\n",
    )
    .unwrap();
    let plan = plan_member_execution(&FrontendMemberBuildRoute {
        member_root: root.clone(),
        package_name: "demo".to_string(),
        mode: FrontendBuildWorkflowMode::Modern,
    }, &FrontendConfig::default())
    .expect("custom run step should plan successfully");

    let serve = plan
        .steps
        .iter()
        .find(|step| step.name == "serve")
        .expect("custom run step should be present");
    assert_eq!(serve.execution, Some(FrontendStepExecutionKind::Run));
    assert_eq!(
        serve
            .selection
            .as_ref()
            .map(|selection| selection.label.as_str()),
        Some("app")
    );

    fs::remove_dir_all(root).ok();
}

#[test]
fn explicit_named_run_steps_select_the_requested_artifact_when_multiple_runnables_exist() {
    let root = std::env::temp_dir().join(format!(
        "fol_frontend_build_route_multi_run_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time before epoch")
            .as_nanos()
    ));
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("package.yaml"), "name: demo\nversion: 0.1.0\n").unwrap();
    fs::write(
        root.join("build.fol"),
        concat!(
            "pro[] build(graph: Graph): non = {\n",
            "    graph.add_exe(\"serve_app\", \"src/serve.fol\");\n",
            "    graph.add_exe(\"admin_app\", \"src/admin.fol\");\n",
            "    graph.add_run(\"serve\", \"serve_app\");\n",
            "    graph.add_run(\"admin\", \"admin_app\");\n",
            "    return graph\n",
            "};\n",
        ),
    )
    .unwrap();
    fs::write(
        root.join("src/serve.fol"),
        "fun[] main(): int = {\n    return 0\n};\n",
    )
    .unwrap();
    fs::write(
        root.join("src/admin.fol"),
        "fun[] main(): int = {\n    return 0\n};\n",
    )
    .unwrap();
    let plan = plan_member_execution(&FrontendMemberBuildRoute {
        member_root: root.clone(),
        package_name: "demo".to_string(),
        mode: FrontendBuildWorkflowMode::Modern,
    }, &FrontendConfig::default())
    .expect("member planning should keep named run step selections");

    let admin = plan
        .steps
        .iter()
        .find(|step| step.name == "admin")
        .expect("admin run step should be present");
    assert_eq!(admin.execution, Some(FrontendStepExecutionKind::Run));
    assert_eq!(
        admin
            .selection
            .as_ref()
            .map(|selection| selection.label.as_str()),
        Some("admin_app")
    );

    fs::remove_dir_all(root).ok();
}

#[test]
fn named_build_steps_can_target_matching_artifacts_when_multiple_builds_exist() {
    let root = std::env::temp_dir().join(format!(
        "fol_frontend_build_route_multi_build_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time before epoch")
            .as_nanos()
    ));
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("package.yaml"), "name: demo\nversion: 0.1.0\n").unwrap();
    fs::write(
        root.join("build.fol"),
        concat!(
            "pro[] build(graph: Graph): non = {\n",
            "    graph.add_exe(\"serve_app\", \"src/serve.fol\");\n",
            "    graph.add_exe(\"admin_app\", \"src/admin.fol\");\n",
            "    graph.step(\"serve_app\");\n",
            "    graph.step(\"admin_app\");\n",
            "    return graph\n",
            "};\n",
        ),
    )
    .unwrap();
    fs::write(
        root.join("src/serve.fol"),
        "fun[] main(): int = {\n    return 0\n};\n",
    )
    .unwrap();
    fs::write(
        root.join("src/admin.fol"),
        "fun[] main(): int = {\n    return 0\n};\n",
    )
    .unwrap();

    let plan = plan_member_execution(&FrontendMemberBuildRoute {
        member_root: root.clone(),
        package_name: "demo".to_string(),
        mode: FrontendBuildWorkflowMode::Modern,
    }, &FrontendConfig::default())
    .expect("member planning should keep named build step selections");

    let admin = plan
        .steps
        .iter()
        .find(|step| step.name == "admin_app")
        .expect("admin build step should be present");
    assert_eq!(admin.execution, Some(FrontendStepExecutionKind::Build));
    assert_eq!(
        admin
            .selection
            .as_ref()
            .map(|selection| selection.label.as_str()),
        Some("admin_app")
    );

    fs::remove_dir_all(root).ok();
}

#[test]
fn default_build_step_is_marked_ambiguous_when_multiple_executables_exist() {
    let root = std::env::temp_dir().join(format!(
        "fol_frontend_build_route_ambiguous_build_plan_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time before epoch")
            .as_nanos()
    ));
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("package.yaml"), "name: demo\nversion: 0.1.0\n").unwrap();
    fs::write(
        root.join("build.fol"),
        concat!(
            "pro[] build(graph: Graph): non = {\n",
            "    graph.add_exe(\"serve_app\", \"src/serve.fol\");\n",
            "    graph.add_exe(\"admin_app\", \"src/admin.fol\");\n",
            "    return graph\n",
            "};\n",
        ),
    )
    .unwrap();
    fs::write(
        root.join("src/serve.fol"),
        "fun[] main(): int = {\n    return 0\n};\n",
    )
    .unwrap();
    fs::write(
        root.join("src/admin.fol"),
        "fun[] main(): int = {\n    return 0\n};\n",
    )
    .unwrap();

    let plan = plan_member_execution(&FrontendMemberBuildRoute {
        member_root: root.clone(),
        package_name: "demo".to_string(),
        mode: FrontendBuildWorkflowMode::Modern,
    }, &FrontendConfig::default())
    .expect("member planning should succeed");

    let build = plan
        .steps
        .iter()
        .find(|step| step.name == "build")
        .expect("default build step should be present");
    assert_eq!(build.execution, Some(FrontendStepExecutionKind::Build));
    assert!(build.selection.is_none());
    assert!(build.ambiguous_selection);

    fs::remove_dir_all(root).ok();
}

#[test]
fn ambiguous_default_multi_artifact_build_steps_fail_clearly() {
    let root = std::env::temp_dir().join(format!(
        "fol_frontend_build_route_ambiguous_build_exec_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time before epoch")
            .as_nanos()
    ));
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("package.yaml"), "name: demo\nversion: 0.1.0\n").unwrap();
    fs::write(
        root.join("build.fol"),
        concat!(
            "pro[] build(graph: Graph): non = {\n",
            "    graph.add_exe(\"serve_app\", \"src/serve.fol\");\n",
            "    graph.add_exe(\"admin_app\", \"src/admin.fol\");\n",
            "    return graph\n",
            "};\n",
        ),
    )
    .unwrap();
    fs::write(
        root.join("src/serve.fol"),
        "fun[] main(): int = {\n    return 0\n};\n",
    )
    .unwrap();
    fs::write(
        root.join("src/admin.fol"),
        "fun[] main(): int = {\n    return 0\n};\n",
    )
    .unwrap();
    let workspace = FrontendWorkspace {
        root: WorkspaceRoot::new(root.clone()),
        members: vec![PackageRoot::new(root.clone())],
        std_root_override: None,
        package_store_root_override: None,
        build_root: root.join(".fol/build"),
        cache_root: root.join(".fol/cache"),
        git_cache_root: root.join(".fol/cache/git"),
    };

    let error = execute_workspace_build_route(
        &workspace,
        &FrontendConfig::default(),
        &FrontendWorkspaceBuildRequest {
            requested_step: "build".to_string(),
            profile: FrontendProfile::Debug,
            run_args: Vec::new(),
        },
    )
    .expect_err("ambiguous default build step should fail");

    assert_eq!(error.kind(), crate::FrontendErrorKind::InvalidInput);
    assert!(error.message().contains("requires an explicit named step"));

    fs::remove_dir_all(root).ok();
}

#[test]
fn configured_executable_roots_drive_default_build_and_run_step_planning() {
    let root = std::env::temp_dir().join(format!(
        "fol_frontend_build_route_targeted_root_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time before epoch")
            .as_nanos()
    ));
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("package.yaml"), "name: demo\nversion: 0.1.0\n").unwrap();
    fs::write(
        root.join("build.fol"),
        concat!(
            "pro[] build(graph: Graph): non = {\n",
            "    graph.add_exe(\"app\", \"src/app.fol\");\n",
            "    graph.add_run(\"serve\", \"app\");\n",
            "    return graph\n",
            "};\n",
        ),
    )
    .unwrap();
    fs::write(root.join("src/main.fol"), "var[exp] ignored: int = 1;\n").unwrap();
    fs::write(
        root.join("src/app.fol"),
        "fun[] main(): int = {\n    return 0\n};\n",
    )
    .unwrap();
    let plan = plan_member_execution(&FrontendMemberBuildRoute {
        member_root: root.clone(),
        package_name: "demo".to_string(),
        mode: FrontendBuildWorkflowMode::Modern,
    }, &FrontendConfig::default())
    .expect("configured add_exe root should drive routed planning");

    let build = plan
        .steps
        .iter()
        .find(|step| step.name == "build")
        .expect("default build step should be present");
    assert_eq!(build.execution, Some(FrontendStepExecutionKind::Build));
    assert_eq!(
        build
            .selection
            .as_ref()
            .and_then(|selection| selection.root_module.as_deref()),
        Some("src/app.fol")
    );

    let serve = plan
        .steps
        .iter()
        .find(|step| step.name == "serve")
        .expect("custom serve step should be present");
    assert_eq!(serve.execution, Some(FrontendStepExecutionKind::Run));
    assert_eq!(
        serve
            .selection
            .as_ref()
            .and_then(|selection| selection.root_module.as_deref()),
        Some("src/app.fol")
    );

    fs::remove_dir_all(root).ok();
}

#[test]
fn object_style_artifact_build_bodies_drive_default_build_and_run_step_planning() {
    let root = std::env::temp_dir().join(format!(
        "fol_frontend_build_route_object_root_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time before epoch")
            .as_nanos()
    ));
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("package.yaml"), "name: demo\nversion: 0.1.0\n").unwrap();
    fs::write(
        root.join("build.fol"),
        concat!(
            "pro[] build(graph: Graph): non = {\n",
            "    var target = graph.standard_target();\n",
            "    var optimize = graph.standard_optimize();\n",
            "    var app = graph.add_exe({\n",
            "        name = \"demo\",\n",
            "        root = \"src/app.fol\",\n",
            "        target = target,\n",
            "        optimize = optimize,\n",
            "    });\n",
            "    graph.install(app);\n",
            "    graph.add_run(app);\n",
            "    return graph\n",
            "};\n",
        ),
    )
    .unwrap();
    fs::write(root.join("src/main.fol"), "var[exp] ignored: int = 1;\n").unwrap();
    fs::write(
        root.join("src/app.fol"),
        "fun[] main(): int = {\n    return 0\n};\n",
    )
    .unwrap();
    let plan = plan_member_execution(&FrontendMemberBuildRoute {
        member_root: root.clone(),
        package_name: "demo".to_string(),
        mode: FrontendBuildWorkflowMode::Modern,
    }, &FrontendConfig::default())
    .expect("object style add_exe should drive routed planning");

    let build = plan
        .steps
        .iter()
        .find(|step| step.name == "build")
        .expect("default build step should be present");
    assert_eq!(build.execution, Some(FrontendStepExecutionKind::Build));
    assert_eq!(
        build
            .selection
            .as_ref()
            .and_then(|selection| selection.root_module.as_deref()),
        Some("src/app.fol")
    );

    let run = plan
        .steps
        .iter()
        .find(|step| step.name == "run")
        .expect("default run step should be present");
    assert_eq!(run.execution, Some(FrontendStepExecutionKind::Run));
    assert_eq!(
        run.selection
            .as_ref()
            .and_then(|selection| selection.root_module.as_deref()),
        Some("src/app.fol")
    );

    fs::remove_dir_all(root).ok();
}

#[test]
fn workspace_route_plans_modern_build_members_through_default_graph_planning() {
    let root = std::env::temp_dir().join(format!(
        "fol_frontend_build_route_modern_exec_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time before epoch")
            .as_nanos()
    ));
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("package.yaml"), "name: demo\nversion: 0.1.0\n").unwrap();
    fs::write(
        root.join("build.fol"),
        concat!(
            "pro[] build(graph: Graph): non = {\n",
            "    graph.add_exe(\"demo\", \"src/main.fol\");\n",
            "    return graph\n",
            "};\n",
        ),
    )
    .unwrap();
    fs::write(
        root.join("src/main.fol"),
        "fun[] main(): int = {\n    return 0\n};\n",
    )
    .unwrap();
    let plan = plan_member_execution(&FrontendMemberBuildRoute {
        member_root: root.clone(),
        package_name: "demo".to_string(),
        mode: FrontendBuildWorkflowMode::Modern,
    }, &FrontendConfig::default())
    .expect("modern member should plan through the default graph");

    assert!(plan.steps.iter().any(|step| step.name == "build"));
    assert!(plan.steps.iter().any(|step| step.name == "run"));
    assert!(plan.steps.iter().any(|step| step.name == "check"));

    fs::remove_dir_all(root).ok();
}

#[test]
fn workspace_route_plans_modern_check_steps_even_without_a_runnable_binary() {
    let root = std::env::temp_dir().join(format!(
        "fol_frontend_build_route_modern_check_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time before epoch")
            .as_nanos()
    ));
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("package.yaml"), "name: demo\nversion: 0.1.0\n").unwrap();
    fs::write(
        root.join("build.fol"),
        concat!(
            "pro[] build(graph: Graph): non = {\n",
            "    graph.step(\"docs\");\n",
            "    return graph\n",
            "};\n",
        ),
    )
    .unwrap();
    fs::write(root.join("src/lib.fol"), "var[exp] answer: int = 42;\n").unwrap();
    let plan = plan_member_execution(&FrontendMemberBuildRoute {
        member_root: root.clone(),
        package_name: "demo".to_string(),
        mode: FrontendBuildWorkflowMode::Modern,
    }, &FrontendConfig::default())
    .expect("modern member without an executable should still plan check");

    let check = plan
        .steps
        .iter()
        .find(|step| step.name == "check")
        .expect("check step should be present");
    assert_eq!(check.execution, Some(FrontendStepExecutionKind::Check));
    assert!(check.selection.is_none());

    fs::remove_dir_all(root).ok();
}

#[test]
fn execute_workspace_build_route_rejects_echo_for_alloc_model_artifacts() {
    let root = std::env::temp_dir().join(format!(
        "fol_frontend_build_route_alloc_echo_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time before epoch")
            .as_nanos()
    ));
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("package.yaml"), "name: demo\nversion: 0.1.0\n").unwrap();
    fs::write(
        root.join("build.fol"),
        concat!(
            "pro[] build(graph: Graph): non = {\n",
            "    var app = graph.add_exe({\n",
            "        name = \"demo\",\n",
            "        root = \"src/main.fol\",\n",
            "        fol_model = \"alloc\",\n",
            "    });\n",
            "    graph.install(app);\n",
            "    return graph\n",
            "};\n",
        ),
    )
    .unwrap();
    fs::write(
        root.join("src/main.fol"),
        "fun[] main(): int = {\n    return .echo(1);\n};\n",
    )
    .unwrap();
    let workspace = FrontendWorkspace {
        root: WorkspaceRoot::new(root.clone()),
        members: vec![PackageRoot::new(root.clone())],
        std_root_override: None,
        package_store_root_override: None,
        build_root: root.join(".fol/build"),
        cache_root: root.join(".fol/cache"),
        git_cache_root: root.join(".fol/cache/git"),
    };

    let error = execute_workspace_build_route(
        &workspace,
        &FrontendConfig::default(),
        &FrontendWorkspaceBuildRequest {
            requested_step: "build".to_string(),
            profile: FrontendProfile::Debug,
            run_args: Vec::new(),
        },
    )
    .expect_err("alloc-model .echo should be rejected during routed build execution");

    assert_eq!(error.kind(), crate::FrontendErrorKind::CommandFailed);
    assert!(error.message().contains("compilation failed"));
    assert_eq!(error.diagnostics().len(), 1);
    assert!(error.diagnostics()[0]
        .message
        .contains("'.echo(...)' requires 'fol_model = std'"));
    assert!(error.diagnostics()[0]
        .message
        .contains("current artifact model is 'alloc'"));

    fs::remove_dir_all(root).ok();
}

#[test]
fn execute_workspace_build_route_rejects_heap_backed_types_for_core_model_artifacts() {
    let root = std::env::temp_dir().join(format!(
        "fol_frontend_build_route_core_heap_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time before epoch")
            .as_nanos()
    ));
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("package.yaml"), "name: demo\nversion: 0.1.0\n").unwrap();
    fs::write(
        root.join("build.fol"),
        concat!(
            "pro[] build(graph: Graph): non = {\n",
            "    var app = graph.add_exe({\n",
            "        name = \"demo\",\n",
            "        root = \"src/main.fol\",\n",
            "        fol_model = \"core\",\n",
            "    });\n",
            "    graph.install(app);\n",
            "    return graph\n",
            "};\n",
        ),
    )
    .unwrap();
    fs::write(
        root.join("src/main.fol"),
        "fun[] main(): str = {\n    return \"ok\";\n};\n",
    )
    .unwrap();
    let workspace = FrontendWorkspace {
        root: WorkspaceRoot::new(root.clone()),
        members: vec![PackageRoot::new(root.clone())],
        std_root_override: None,
        package_store_root_override: None,
        build_root: root.join(".fol/build"),
        cache_root: root.join(".fol/cache"),
        git_cache_root: root.join(".fol/cache/git"),
    };

    let error = execute_workspace_build_route(
        &workspace,
        &FrontendConfig::default(),
        &FrontendWorkspaceBuildRequest {
            requested_step: "build".to_string(),
            profile: FrontendProfile::Debug,
            run_args: Vec::new(),
        },
    )
    .expect_err("core-model heap-backed types should be rejected during routed build execution");

    assert_eq!(error.kind(), crate::FrontendErrorKind::CommandFailed);
    assert_eq!(error.diagnostics().len(), 1);
    assert!(error.diagnostics()[0]
        .message
        .contains("str requires heap support and is unavailable in 'fol_model = core'"));

    fs::remove_dir_all(root).ok();
}

#[test]
fn execute_workspace_build_route_rejects_dynamic_len_for_core_model_artifacts() {
    let root = std::env::temp_dir().join(format!(
        "fol_frontend_build_route_core_len_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time before epoch")
            .as_nanos()
    ));
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("package.yaml"), "name: demo\nversion: 0.1.0\n").unwrap();
    fs::write(
        root.join("build.fol"),
        concat!(
            "pro[] build(graph: Graph): non = {\n",
            "    var app = graph.add_exe({\n",
            "        name = \"demo\",\n",
            "        root = \"src/main.fol\",\n",
            "        fol_model = \"core\",\n",
            "    });\n",
            "    graph.install(app);\n",
            "    return graph\n",
            "};\n",
        ),
    )
    .unwrap();
    fs::write(
        root.join("src/main.fol"),
        "fun[] main(): int = {\n    return .len(\"Ada\");\n};\n",
    )
    .unwrap();
    let workspace = FrontendWorkspace {
        root: WorkspaceRoot::new(root.clone()),
        members: vec![PackageRoot::new(root.clone())],
        std_root_override: None,
        package_store_root_override: None,
        build_root: root.join(".fol/build"),
        cache_root: root.join(".fol/cache"),
        git_cache_root: root.join(".fol/cache/git"),
    };

    let error = execute_workspace_build_route(
        &workspace,
        &FrontendConfig::default(),
        &FrontendWorkspaceBuildRequest {
            requested_step: "build".to_string(),
            profile: FrontendProfile::Debug,
            run_args: Vec::new(),
        },
    )
    .expect_err("core-model dynamic .len should be rejected during routed build execution");

    assert_eq!(error.kind(), crate::FrontendErrorKind::CommandFailed);
    assert_eq!(error.diagnostics().len(), 1);
    assert!(error.diagnostics()[0]
        .message
        .contains("string literals require heap support and are unavailable in 'fol_model = core'"));

    fs::remove_dir_all(root).ok();
}

#[test]
fn execute_workspace_build_route_accepts_dynamic_len_for_alloc_model_artifacts() {
    let root = std::env::temp_dir().join(format!(
        "fol_frontend_build_route_alloc_len_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time before epoch")
            .as_nanos()
    ));
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("package.yaml"), "name: demo\nversion: 0.1.0\n").unwrap();
    fs::write(
        root.join("build.fol"),
        concat!(
            "pro[] build(graph: Graph): non = {\n",
            "    var app = graph.add_exe({\n",
            "        name = \"demo\",\n",
            "        root = \"src/main.fol\",\n",
            "        fol_model = \"alloc\",\n",
            "    });\n",
            "    graph.install(app);\n",
            "    return graph\n",
            "};\n",
        ),
    )
    .unwrap();
    fs::write(
        root.join("src/main.fol"),
        concat!(
            "fun[] main(): int = {\n",
            "    return .len(\"Ada\");\n",
            "};\n",
        ),
    )
    .unwrap();
    let workspace = FrontendWorkspace {
        root: WorkspaceRoot::new(root.clone()),
        members: vec![PackageRoot::new(root.clone())],
        std_root_override: None,
        package_store_root_override: None,
        build_root: root.join(".fol/build"),
        cache_root: root.join(".fol/cache"),
        git_cache_root: root.join(".fol/cache/git"),
    };

    execute_workspace_build_route(
        &workspace,
        &FrontendConfig::default(),
        &FrontendWorkspaceBuildRequest {
            requested_step: "build".to_string(),
            profile: FrontendProfile::Debug,
            run_args: Vec::new(),
        },
    )
    .expect("alloc-model dynamic .len should remain buildable during routed execution");

    fs::remove_dir_all(root).ok();
}

#[test]
fn execute_workspace_build_route_rejects_run_for_selected_core_model_artifacts() {
    let root = std::env::temp_dir().join(format!(
        "fol_frontend_build_route_core_run_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time before epoch")
            .as_nanos()
    ));
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("package.yaml"), "name: demo\nversion: 0.1.0\n").unwrap();
    fs::write(
        root.join("build.fol"),
        concat!(
            "pro[] build(graph: Graph): non = {\n",
            "    var app = graph.add_exe({\n",
            "        name = \"demo\",\n",
            "        root = \"src/main.fol\",\n",
            "        fol_model = \"core\",\n",
            "    });\n",
            "    graph.add_run(\"serve\", app);\n",
            "    return graph\n",
            "};\n",
        ),
    )
    .unwrap();
    fs::write(
        root.join("src/main.fol"),
        "fun[] main(): int = {\n    return 0;\n};\n",
    )
    .unwrap();
    let workspace = FrontendWorkspace {
        root: WorkspaceRoot::new(root.clone()),
        members: vec![PackageRoot::new(root.clone())],
        std_root_override: None,
        package_store_root_override: None,
        build_root: root.join(".fol/build"),
        cache_root: root.join(".fol/cache"),
        git_cache_root: root.join(".fol/cache/git"),
    };

    let error = execute_workspace_build_route(
        &workspace,
        &FrontendConfig::default(),
        &FrontendWorkspaceBuildRequest {
            requested_step: "serve".to_string(),
            profile: FrontendProfile::Debug,
            run_args: Vec::new(),
        },
    )
    .expect_err("core-model selected run should be rejected during routed execution");

    assert_eq!(error.kind(), crate::FrontendErrorKind::InvalidInput);
    assert!(error
        .message()
        .contains("run command requires 'fol_model = std'"));
    assert!(error.message().contains("'demo'"));
    assert!(error.message().contains("'core'"));

    fs::remove_dir_all(root).ok();
}

#[test]
fn execute_workspace_build_route_rejects_test_for_selected_alloc_model_artifacts() {
    let root = std::env::temp_dir().join(format!(
        "fol_frontend_build_route_alloc_test_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time before epoch")
            .as_nanos()
    ));
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("package.yaml"), "name: demo\nversion: 0.1.0\n").unwrap();
    fs::write(
        root.join("build.fol"),
        concat!(
            "pro[] build(graph: Graph): non = {\n",
            "    graph.add_test({\n",
            "        name = \"demo_test\",\n",
            "        root = \"src/main.fol\",\n",
            "        fol_model = \"alloc\",\n",
            "    });\n",
            "    return graph\n",
            "};\n",
        ),
    )
    .unwrap();
    fs::write(
        root.join("src/main.fol"),
        "fun[] main(): int = {\n    return 0;\n};\n",
    )
    .unwrap();
    let workspace = FrontendWorkspace {
        root: WorkspaceRoot::new(root.clone()),
        members: vec![PackageRoot::new(root.clone())],
        std_root_override: None,
        package_store_root_override: None,
        build_root: root.join(".fol/build"),
        cache_root: root.join(".fol/cache"),
        git_cache_root: root.join(".fol/cache/git"),
    };

    let error = execute_workspace_build_route(
        &workspace,
        &FrontendConfig::default(),
        &FrontendWorkspaceBuildRequest {
            requested_step: "test".to_string(),
            profile: FrontendProfile::Debug,
            run_args: Vec::new(),
        },
    )
    .expect_err("alloc-model selected test should be rejected during routed execution");

    assert_eq!(error.kind(), crate::FrontendErrorKind::InvalidInput);
    assert!(error
        .message()
        .contains("test command requires 'fol_model = std'"));
    assert!(error.message().contains("'demo_test'"));
    assert!(error.message().contains("'alloc'"));

    fs::remove_dir_all(root).ok();
}
