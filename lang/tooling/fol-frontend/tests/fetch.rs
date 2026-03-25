use fol_frontend::{
    fetch_workspace, fetch_workspace_with_config, prepare_workspace_packages,
    update_workspace_with_config, FrontendConfig, FrontendWorkspace, PackageRoot, WorkspaceRoot,
};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn semantic_bin_build() -> &'static str {
    concat!(
        "pro[] build(): non = {\n",
        "    var build = .build();\n",
        "    build.meta({ name = \"app\", version = \"0.1.0\" });\n",
        "    var graph = build.graph();\n",
        "    var app = graph.add_exe({ name = \"app\", root = \"src/main.fol\" });\n",
        "    graph.install(app);\n",
        "    graph.add_run(app);\n",
        "};\n",
    )
}

fn semantic_lib_build(name: &str) -> String {
    format!(
        concat!(
            "pro[] build(): non = {{\n",
            "    var build = .build();\n",
            "    build.meta({{ name = \"{name}\", version = \"0.1.0\" }});\n",
            "    var graph = build.graph();\n",
            "    var lib = graph.add_static_lib({{ name = \"{name}\", root = \"src/lib.fol\" }});\n",
            "    graph.install(lib);\n",
            "}};\n",
        ),
        name = name
    )
}

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
    fs::write(app.join("build.fol"), semantic_bin_build()).expect("should write app build");
    fs::write(lib.join("build.fol"), semantic_lib_build("lib")).expect("should write lib build");

    let workspace = FrontendWorkspace {
        root: WorkspaceRoot::new(root.clone()),
        members: vec![PackageRoot::new(app.clone()), PackageRoot::new(lib.clone())],
        std_root_override: Some(root.join("std")),
        package_store_root_override: Some(root.join(".fol/pkg")),
        build_root: root.join(".fol/build"),
        cache_root: root.join(".fol/cache"),
        git_cache_root: root.join(".fol/cache/git"),
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
    fs::write(app.join("build.fol"), semantic_bin_build()).expect("should write app build");

    let workspace = FrontendWorkspace {
        root: WorkspaceRoot::new(root.clone()),
        members: vec![PackageRoot::new(app.clone())],
        std_root_override: None,
        package_store_root_override: Some(root.join(".fol/ws-pkg")),
        build_root: root.join(".fol/build"),
        cache_root: root.join(".fol/cache"),
        git_cache_root: root.join(".fol/cache/git"),
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

#[test]
fn fetch_locked_requires_existing_lockfile() {
    let root = temp_root("locked_missing");
    let app = root.join("app");
    fs::create_dir_all(&app).expect("should create app package");
    fs::write(app.join("build.fol"), semantic_bin_build()).expect("should write app build");

    let workspace = FrontendWorkspace {
        root: WorkspaceRoot::new(root.clone()),
        members: vec![PackageRoot::new(app)],
        std_root_override: None,
        package_store_root_override: Some(root.join(".fol/pkg")),
        build_root: root.join(".fol/build"),
        cache_root: root.join(".fol/cache"),
        git_cache_root: root.join(".fol/cache/git"),
    };
    let config = FrontendConfig {
        locked_fetch: true,
        ..FrontendConfig::default()
    };

    let error = fetch_workspace_with_config(&workspace, &config)
        .expect_err("locked fetch should require fol.lock");

    assert!(error
        .message()
        .contains("locked fetch requires an existing fol.lock"));
    fs::remove_dir_all(root).ok();
}

#[test]
fn fetch_offline_uses_warm_git_cache() {
    let root = temp_root("offline_warm");
    let app = root.join("app");
    let remote = root.join("remote-logtiny");
    create_git_package_repo(&remote, "logtiny", "0.1.0");
    create_app_with_git_dep(&app, &remote);

    let workspace = git_dep_workspace(&root, &app);
    fetch_workspace(&workspace).expect("initial fetch should warm the cache");

    let config = FrontendConfig {
        offline_fetch: true,
        ..FrontendConfig::default()
    };
    let result = fetch_workspace_with_config(&workspace, &config)
        .expect("offline fetch should use warm cache");

    assert_eq!(result.command, "fetch");
    assert!(root.join("fol.lock").is_file());
    fs::remove_dir_all(root).ok();
}

#[test]
fn fetch_offline_without_a_warm_cache_adds_guidance_notes() {
    let root = temp_root("offline_cold");
    let app = root.join("app");
    let remote = root.join("remote-logtiny");
    create_git_package_repo(&remote, "logtiny", "0.1.0");
    create_app_with_git_dep(&app, &remote);

    let workspace = git_dep_workspace(&root, &app);
    let error = fetch_workspace_with_config(
        &workspace,
        &FrontendConfig {
            offline_fetch: true,
            ..FrontendConfig::default()
        },
    )
    .expect_err("offline fetch should fail without a warm cache");

    assert!(error
        .notes()
        .iter()
        .any(|note| note.contains("offline mode only works")));
    fs::remove_dir_all(root).ok();
}

#[test]
fn update_workspace_refreshes_git_dependencies_through_public_api() {
    let root = temp_root("update");
    let app = root.join("app");
    let remote = root.join("remote-logtiny");
    create_git_package_repo(&remote, "logtiny", "0.1.0");
    create_app_with_git_dep(&app, &remote);
    let workspace = git_dep_workspace(&root, &app);

    fetch_workspace(&workspace).expect("initial fetch should succeed");
    let before = fs::read_to_string(root.join("fol.lock")).expect("lockfile should exist");

    fs::write(remote.join("src/lib.fol"), "var[exp] level: int = 2;\n")
        .expect("should update remote source");
    git(&remote, &["add", "."]);
    git(&remote, &["commit", "-m", "bump"]);

    let result = update_workspace_with_config(&workspace, &FrontendConfig::default())
        .expect("update should succeed");
    let after = fs::read_to_string(root.join("fol.lock")).expect("lockfile should still exist");

    assert_eq!(result.command, "update");
    assert_ne!(before, after);
    assert!(result.summary.contains("revisions changed: 1"));
    assert!(result.summary.contains("logtiny:"));
    fs::remove_dir_all(root).ok();
}

#[test]
fn update_prunes_stale_workspace_local_git_materializations() {
    let root = temp_root("update_prune");
    let app = root.join("app");
    let remote = root.join("remote-logtiny");
    create_git_package_repo(&remote, "logtiny", "0.1.0");
    create_app_with_git_dep(&app, &remote);
    let workspace = git_dep_workspace(&root, &app);

    fetch_workspace(&workspace).expect("initial fetch should succeed");
    let before = fol_package::parse_package_lockfile(
        &fs::read_to_string(root.join("fol.lock")).expect("lockfile should exist"),
    )
    .expect("lockfile should parse");
    let old_root = PathBuf::from(before.entries[0].materialized_root.clone());
    assert!(old_root.is_dir());

    fs::write(remote.join("src/lib.fol"), "var[exp] level: int = 3;\n")
        .expect("should update remote source");
    git(&remote, &["add", "."]);
    git(&remote, &["commit", "-m", "bump-again"]);

    let result = update_workspace_with_config(&workspace, &FrontendConfig::default())
        .expect("update should succeed");

    assert!(result
        .summary
        .contains("pruned 1 stale git materialization(s)"));
    assert!(!old_root.exists());
    fs::remove_dir_all(root).ok();
}

#[test]
fn locked_fetch_repairs_missing_pinned_materializations_from_warm_cache() {
    let root = temp_root("locked_repair");
    let app = root.join("app");
    let remote = root.join("remote-logtiny");
    create_git_package_repo(&remote, "logtiny", "0.1.0");
    create_app_with_git_dep(&app, &remote);
    let workspace = git_dep_workspace(&root, &app);

    fetch_workspace(&workspace).expect("initial fetch should succeed");
    let lockfile = fol_package::parse_package_lockfile(
        &fs::read_to_string(root.join("fol.lock")).expect("lockfile should exist"),
    )
    .expect("lockfile should parse");
    let materialized_root = PathBuf::from(lockfile.entries[0].materialized_root.clone());
    fs::remove_dir_all(&materialized_root).expect("should remove pinned materialization");

    let result = fetch_workspace_with_config(
        &workspace,
        &FrontendConfig {
            locked_fetch: true,
            offline_fetch: true,
            ..FrontendConfig::default()
        },
    )
    .expect("locked fetch should repair from the warm cache");

    assert!(result
        .summary
        .contains("repaired 1 missing pinned materialization(s)"));
    assert!(materialized_root.is_dir());
    fs::remove_dir_all(root).ok();
}

fn git_dep_workspace(root: &Path, app: &Path) -> FrontendWorkspace {
    FrontendWorkspace {
        root: WorkspaceRoot::new(root.to_path_buf()),
        members: vec![PackageRoot::new(app.to_path_buf())],
        std_root_override: None,
        package_store_root_override: Some(root.join(".fol/pkg")),
        build_root: root.join(".fol/build"),
        cache_root: root.join(".fol/cache"),
        git_cache_root: root.join(".fol/cache/git"),
    }
}

fn create_app_with_git_dep(app: &Path, remote: &Path) {
    fs::create_dir_all(app.join("src")).expect("should create app package");
    fs::write(
        app.join("build.fol"),
        format!(
            concat!(
                "pro[] build(): non = {{\n",
                "    var build = .build();\n",
                "    build.meta({{ name = \"app\", version = \"0.1.0\" }});\n",
                "    build.add_dep({{ alias = \"logtiny\", source = \"git\", target = \"git+file://{}\" }});\n",
                "    var graph = build.graph();\n",
                "    var app = graph.add_exe({{ name = \"app\", root = \"src/main.fol\" }});\n",
                "    graph.install(app);\n",
                "    graph.add_run(app);\n",
                "}};\n",
            ),
            remote.display()
        ),
    )
    .expect("should write app build");
    fs::write(
        app.join("src/main.fol"),
        "fun[] main(): int = {\n    return 0\n};\n",
    )
    .expect("should write app source");
}

fn create_git_package_repo(root: &Path, name: &str, version: &str) {
    fs::create_dir_all(root.join("src")).expect("package repo should be creatable");
    fs::write(root.join("build.fol"), semantic_lib_build(name).replace("0.1.0", version))
        .expect("package build should be writable");
    fs::write(root.join("src/lib.fol"), "var[exp] level: int = 1;\n")
        .expect("package source should be writable");
    git(root, &["init"]);
    git(root, &["config", "user.name", "FOL"]);
    git(root, &["config", "user.email", "fol@example.com"]);
    git(root, &["add", "."]);
    git(root, &["commit", "-m", "init"]);
}

fn git(root: &Path, args: &[&str]) {
    let status = Command::new("git")
        .args(args)
        .current_dir(root)
        .status()
        .expect("git command should run");
    assert!(status.success(), "git {:?} should succeed", args);
}
