use fol_frontend::{clean_workspace_with_config, FrontendConfig, FrontendWorkspace, WorkspaceRoot};
use std::fs;
use std::path::PathBuf;

fn temp_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "fol_frontend_clean_{}_{}_{}",
        label,
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time should be after epoch")
            .as_nanos()
    ))
}

#[test]
fn clean_command_removes_workspace_build_and_cache_roots_through_public_api() {
    let root = temp_root("roots");
    let build_root = root.join(".fol/build");
    let cache_root = root.join(".fol/cache");
    fs::create_dir_all(&build_root).expect("should create build root");
    fs::create_dir_all(&cache_root).expect("should create cache root");

    let mut workspace = FrontendWorkspace::new(WorkspaceRoot::new(root.clone()));
    workspace.build_root = build_root.clone();
    workspace.cache_root = cache_root.clone();

    let result = clean_workspace_with_config(&workspace, &FrontendConfig::default())
        .expect("clean should succeed");

    assert_eq!(result.command, "clean");
    assert!(!build_root.exists());
    assert!(!cache_root.exists());

    fs::remove_dir_all(root).ok();
}

#[test]
fn clean_command_skips_external_package_store_roots_through_public_api() {
    let root = temp_root("external_store");
    let external_store = temp_root("pkg_store");
    fs::create_dir_all(&external_store).expect("should create external package store");

    let workspace = FrontendWorkspace::new(WorkspaceRoot::new(root.clone()));
    let result = clean_workspace_with_config(
        &workspace,
        &FrontendConfig {
            package_store_root_override: Some(external_store.clone()),
            ..FrontendConfig::default()
        },
    )
    .expect("clean should succeed");

    assert!(result.summary.contains("skipped external package store"));
    assert!(external_store.exists());

    fs::remove_dir_all(root).ok();
    fs::remove_dir_all(external_store).ok();
}
