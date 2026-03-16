use fol_frontend::{
    fetch_workspace, fetch_workspace_with_config, prepare_workspace_packages, FrontendConfig,
    FrontendWorkspace, PackageRoot, WorkspaceRoot,
};
use std::fs;
use std::path::PathBuf;

fn temp_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "fol_frontend_fetch_{}_{}_{}",
        label,
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time should be after epoch")
            .as_nanos()
    ))
}

#[test]
fn fetch_round_trip_prepares_and_reports_local_workspace_packages() {
    let root = temp_root("round_trip");
    let app = root.join("app");
    let lib = root.join("lib");
    fs::create_dir_all(&app).expect("should create app package");
    fs::create_dir_all(&lib).expect("should create lib package");
    fs::write(app.join("package.yaml"), "name: app\nversion: 0.1.0\n").expect("should write app manifest");
    fs::write(app.join("build.fol"), "def root: loc = \"src\"\n").expect("should write app build");
    fs::write(lib.join("package.yaml"), "name: lib\nversion: 0.1.0\n").expect("should write lib manifest");
    fs::write(lib.join("build.fol"), "def root: loc = \"src\"\n").expect("should write lib build");

    let workspace = FrontendWorkspace {
        root: WorkspaceRoot::new(root.clone()),
        members: vec![PackageRoot::new(app.clone()), PackageRoot::new(lib.clone())],
        std_root_override: Some(root.join("std")),
        package_store_root_override: Some(root.join(".fol/pkg")),
        build_root: root.join(".fol/build"),
        cache_root: root.join(".fol/cache"),
    };

    let preparation = prepare_workspace_packages(&workspace).expect("preparation should succeed");
    let result = fetch_workspace(&workspace).expect("fetch should succeed");

    assert_eq!(preparation.packages.len(), 2);
    assert!(result.summary.contains("prepared 2 workspace package(s)"));
    assert_eq!(result.artifacts.len(), 4);
    assert_eq!(result.artifacts[2].path, Some(app));
    assert_eq!(result.artifacts[3].path, Some(lib));

    fs::remove_dir_all(root).ok();
}

#[test]
fn fetch_round_trip_prefers_frontend_config_store_root_in_artifacts() {
    let root = temp_root("config_store");
    let app = root.join("app");
    fs::create_dir_all(&app).expect("should create app package");
    fs::write(app.join("package.yaml"), "name: app\nversion: 0.1.0\n").expect("should write app manifest");
    fs::write(app.join("build.fol"), "def root: loc = \"src\"\n").expect("should write app build");

    let workspace = FrontendWorkspace {
        root: WorkspaceRoot::new(root.clone()),
        members: vec![PackageRoot::new(app.clone())],
        std_root_override: None,
        package_store_root_override: Some(root.join(".fol/ws-pkg")),
        build_root: root.join(".fol/build"),
        cache_root: root.join(".fol/cache"),
    };
    let config = FrontendConfig {
        package_store_root_override: Some(root.join(".fol/config-pkg")),
        ..FrontendConfig::default()
    };

    let result = fetch_workspace_with_config(&workspace, &config).expect("fetch should succeed");

    assert_eq!(result.artifacts[0].path, Some(root.join(".fol/config-pkg")));
    assert_eq!(result.artifacts[1].path, Some(root.join(".fol/cache")));
    assert_eq!(result.artifacts[2].path, Some(app));

    fs::remove_dir_all(root).ok();
}
