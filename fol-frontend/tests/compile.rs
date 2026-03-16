use fol_frontend::{
    build_workspace, check_workspace, run_workspace, FrontendArtifactKind, FrontendWorkspace,
    PackageRoot, WorkspaceRoot,
};
use std::fs;
use std::path::PathBuf;

fn temp_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "fol_frontend_compile_{}_{}_{}",
        label,
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time should be after epoch")
            .as_nanos()
    ))
}

fn sample_workspace(root: &PathBuf) -> FrontendWorkspace {
    let app = root.join("app");
    let src = app.join("src");
    fs::create_dir_all(&src).expect("should create source tree");
    fs::write(app.join("package.yaml"), "name: app\nversion: 0.1.0\n")
        .expect("should write manifest");
    fs::write(app.join("build.fol"), "def root: loc = \"src\"\n")
        .expect("should write build file");
    fs::write(src.join("main.fol"), "fun[] main(): int = {\n    return 0\n}\n")
        .expect("should write main");

    FrontendWorkspace {
        root: WorkspaceRoot::new(root.clone()),
        members: vec![PackageRoot::new(app)],
        std_root_override: None,
        package_store_root_override: None,
        build_root: root.join(".fol/build"),
        cache_root: root.join(".fol/cache"),
    }
}

#[test]
fn check_command_round_trips_workspace_members_through_public_api() {
    let root = temp_root("check");
    let workspace = sample_workspace(&root);

    let result = check_workspace(&workspace).expect("check should succeed");

    assert_eq!(result.command, "check");
    assert_eq!(result.artifacts.len(), 1);
    assert_eq!(result.artifacts[0].kind, FrontendArtifactKind::PackageRoot);

    fs::remove_dir_all(root).ok();
}

#[test]
fn build_command_reports_emitted_crate_and_binary_through_public_api() {
    let root = temp_root("build");
    let workspace = sample_workspace(&root);

    let result = build_workspace(&workspace).expect("build should succeed");

    assert_eq!(result.command, "build");
    assert_eq!(result.artifacts.len(), 2);
    assert_eq!(result.artifacts[0].kind, FrontendArtifactKind::EmittedRust);
    assert_eq!(result.artifacts[1].kind, FrontendArtifactKind::Binary);
    assert!(result.artifacts[1]
        .path
        .as_ref()
        .expect("binary path should exist")
        .is_file());

    fs::remove_dir_all(root).ok();
}

#[test]
fn run_command_executes_single_workspace_members_through_public_api() {
    let root = temp_root("run");
    let workspace = sample_workspace(&root);

    let result = run_workspace(&workspace).expect("run should succeed");

    assert_eq!(result.command, "run");
    assert_eq!(result.artifacts.len(), 1);
    assert_eq!(result.artifacts[0].kind, FrontendArtifactKind::Binary);

    fs::remove_dir_all(root).ok();
}
