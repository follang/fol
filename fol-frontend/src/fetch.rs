use crate::{
    FrontendArtifactKind, FrontendArtifactSummary, FrontendCommandResult, FrontendError,
    FrontendResult, FrontendWorkspace, FrontendConfig,
};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrontendPreparedPackage {
    pub root: PathBuf,
    pub name: String,
    pub version: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrontendPackagePreparation {
    pub package_config: fol_package::PackageConfig,
    pub packages: Vec<FrontendPreparedPackage>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ResolvedDependencyPackage {
    root: PathBuf,
    name: String,
    version: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FetchResolution {
    preparation: FrontendPackagePreparation,
    resolved_packages: Vec<ResolvedDependencyPackage>,
    lockfile: fol_package::PackageLockfile,
    lockfile_path: PathBuf,
    package_store_root: PathBuf,
}

pub fn select_package_store_root(
    config: &FrontendConfig,
    workspace: &FrontendWorkspace,
) -> PathBuf {
    config
        .package_store_root_override
        .clone()
        .or_else(|| workspace.package_store_root_override.clone())
        .unwrap_or_else(|| workspace.root.root.join(".fol").join("pkg"))
}

fn select_git_store_root(config: &FrontendConfig, workspace: &FrontendWorkspace) -> PathBuf {
    select_package_store_root(config, workspace).join("git")
}

fn lockfile_path(workspace: &FrontendWorkspace) -> PathBuf {
    workspace.root.root.join("fol.lock")
}

pub fn prepare_workspace_packages(
    workspace: &FrontendWorkspace,
) -> FrontendResult<FrontendPackagePreparation> {
    let package_config = fol_package::PackageConfig {
        std_root: workspace
            .std_root_override
            .as_ref()
            .map(|path| path.to_string_lossy().to_string()),
        package_store_root: workspace
            .package_store_root_override
            .as_ref()
            .map(|path| path.to_string_lossy().to_string()),
        package_cache_root: Some(workspace.cache_root.to_string_lossy().to_string()),
        package_git_cache_root: Some(workspace.git_cache_root.to_string_lossy().to_string()),
    };

    let packages = workspace
        .members
        .iter()
        .map(|member| {
            let metadata = fol_package::parse_package_metadata(&member.manifest_file)
                .map_err(FrontendError::from)?;
            let build_path = member.root.join("build.fol");
            fol_package::parse_package_build(&build_path).map_err(FrontendError::from)?;

            Ok(FrontendPreparedPackage {
                root: member.root.clone(),
                name: metadata.name,
                version: metadata.version,
            })
        })
        .collect::<FrontendResult<Vec<_>>>()?;

    Ok(FrontendPackagePreparation {
        package_config,
        packages,
    })
}

pub fn fetch_workspace_with_config(
    workspace: &FrontendWorkspace,
    config: &FrontendConfig,
) -> FrontendResult<FrontendCommandResult> {
    let resolution = resolve_workspace_fetch(workspace, config)?;
    if !config.locked_fetch {
        std::fs::write(
            &resolution.lockfile_path,
            fol_package::render_package_lockfile(&resolution.lockfile),
        )
        .map_err(|error| FrontendError::new(
            crate::FrontendErrorKind::CommandFailed,
            format!(
                "failed to write lockfile '{}': {}",
                resolution.lockfile_path.display(),
                error
            ),
        ))?;
    }

    let mut result = FrontendCommandResult::new(
        "fetch",
        format!(
            "prepared {} workspace package(s) and resolved {} package root(s) into {}",
            resolution.preparation.packages.len(),
            resolution.resolved_packages.len(),
            resolution.package_store_root.display()
        ),
    );
    result.artifacts.push(FrontendArtifactSummary::new(
        FrontendArtifactKind::PackageRoot,
        "package-store-root",
        Some(resolution.package_store_root),
    ));
    result.artifacts.push(FrontendArtifactSummary::new(
        FrontendArtifactKind::PackageRoot,
        "package-cache-root",
        Some(workspace.cache_root.clone()),
    ));
    result.artifacts.push(FrontendArtifactSummary::new(
        FrontendArtifactKind::CacheRoot,
        "git-cache-root",
        Some(workspace.git_cache_root.clone()),
    ));
    result.artifacts.push(FrontendArtifactSummary::new(
        FrontendArtifactKind::PackageRoot,
        "lockfile",
        Some(resolution.lockfile_path),
    ));
    for package in resolution.resolved_packages {
        result.artifacts.push(FrontendArtifactSummary::new(
            FrontendArtifactKind::PackageRoot,
            package.name,
            Some(package.root),
        ));
    }
    Ok(result)
}

pub fn fetch_workspace(workspace: &FrontendWorkspace) -> FrontendResult<FrontendCommandResult> {
    fetch_workspace_with_config(workspace, &FrontendConfig::default())
}

pub fn update_workspace_with_config(
    workspace: &FrontendWorkspace,
    config: &FrontendConfig,
) -> FrontendResult<FrontendCommandResult> {
    let mut update_config = config.clone();
    update_config.refresh_fetch = true;
    update_config.locked_fetch = false;
    let mut result = fetch_workspace_with_config(workspace, &update_config)?;
    result.command = "update".to_string();
    result.summary = result.summary.replacen("prepared", "updated", 1);
    Ok(result)
}

pub fn update_workspace(workspace: &FrontendWorkspace) -> FrontendResult<FrontendCommandResult> {
    update_workspace_with_config(workspace, &FrontendConfig::default())
}

fn resolve_workspace_fetch(
    workspace: &FrontendWorkspace,
    config: &FrontendConfig,
) -> FrontendResult<FetchResolution> {
    let preparation = prepare_workspace_packages(workspace)?;
    let package_store_root = select_package_store_root(config, workspace);
    let git_store_root = select_git_store_root(config, workspace);
    let git_session =
        fol_package::PackageGitSourceSession::new(workspace.git_cache_root.clone(), git_store_root);
    let mut package_session = fol_package::PackageSession::with_config(
        preparation.package_config.clone(),
    );
    let mut queued_roots = preparation
        .packages
        .iter()
        .map(|package| package.root.clone())
        .collect::<Vec<_>>();
    let mut seen_roots = BTreeSet::new();
    let mut resolved_packages = Vec::new();
    let mut git_lock_entries = Vec::new();
    let mut seen_lock_keys = BTreeSet::new();
    let existing_lockfile = if config.locked_fetch {
        Some(load_existing_lockfile(workspace)?)
    } else {
        None
    };
    let mut seen_git_aliases = BTreeSet::new();
    let fetch_options = fol_package::PackageGitFetchOptions {
        offline: config.offline_fetch,
        refresh: config.refresh_fetch,
    };

    while let Some(root) = queued_roots.pop() {
        let canonical_root = std::fs::canonicalize(&root).unwrap_or(root.clone());
        let canonical_key = canonical_root.to_string_lossy().to_string();
        if !seen_roots.insert(canonical_key) {
            continue;
        }

        let metadata = fol_package::parse_package_metadata(&canonical_root.join("package.yaml"))
            .map_err(FrontendError::from)?;
        fol_package::parse_package_build(&canonical_root.join("build.fol"))
            .map_err(FrontendError::from)?;

        resolved_packages.push(ResolvedDependencyPackage {
            root: canonical_root.clone(),
            name: metadata.name.clone(),
            version: metadata.version.clone(),
        });

        for dependency in metadata.dependencies {
            match dependency.source_kind {
                fol_package::PackageDependencySourceKind::Local => {
                    let dependency_root = absolute_dependency_root(&canonical_root, &dependency.target);
                    fol_package::parse_package_metadata(&dependency_root.join("package.yaml"))
                        .map_err(FrontendError::from)?;
                    fol_package::parse_package_build(&dependency_root.join("build.fol"))
                        .map_err(FrontendError::from)?;
                    queued_roots.push(dependency_root);
                }
                fol_package::PackageDependencySourceKind::PackageStore => {
                    let locator = fol_package::parse_package_locator(&dependency.target)
                        .map_err(FrontendError::from)?;
                    let loaded = package_session
                        .load_package_from_store(
                            &package_store_root,
                            &locator_use_path_segments(&locator),
                        )
                        .map_err(FrontendError::from)?;
                    queued_roots.push(PathBuf::from(loaded.identity.canonical_root));
                }
                fol_package::PackageDependencySourceKind::Git => {
                    let locator = fol_package::parse_package_locator(&dependency.target)
                        .map_err(FrontendError::from)?;
                    let materialization = if let Some(lockfile) = &existing_lockfile {
                        let entry = lockfile
                            .entries
                            .iter()
                            .find(|entry| entry.alias == dependency.alias)
                            .ok_or_else(|| lock_mismatch_error(format!(
                                "missing git dependency '{}' in fol.lock",
                                dependency.alias
                            )))?;
                        if entry.locator != locator.raw {
                            return Err(lock_mismatch_error(format!(
                                "git dependency '{}' points to '{}' in package.yaml but '{}' in fol.lock",
                                dependency.alias, locator.raw, entry.locator
                            )));
                        }
                        git_session
                            .materialize_revision_with_options(
                                &locator,
                                &entry.selected_revision,
                                fetch_options,
                            )
                            .map_err(FrontendError::from)?
                    } else {
                        git_session
                            .materialize_selected_revision_with_options(&locator, fetch_options)
                            .map_err(FrontendError::from)?
                    };
                    seen_git_aliases.insert(dependency.alias.clone());
                    let loaded = package_session
                        .load_materialized_package(&materialization.store_root)
                        .map_err(FrontendError::from)?;
                    queued_roots.push(PathBuf::from(loaded.identity.canonical_root.clone()));

                    let lock_key = format!(
                        "{}@{}",
                        locator
                            .normalized_git_identity()
                            .unwrap_or_else(|| locator.raw.clone()),
                        materialization.selected_revision
                    );
                    if seen_lock_keys.insert(lock_key) {
                        git_lock_entries.push(fol_package::PackageLockEntry {
                            alias: dependency.alias,
                            source_kind: fol_package::PackageDependencySourceKind::Git,
                            locator: locator.raw.clone(),
                            selected_revision: materialization.selected_revision.clone(),
                            materialized_root: materialization.store_root.to_string_lossy().to_string(),
                        });
                    }
                }
            }
        }
    }

    git_lock_entries.sort_by(|left, right| left.alias.cmp(&right.alias));
    let lockfile = fol_package::PackageLockfile::new(git_lock_entries);
    if let Some(existing_lockfile) = &existing_lockfile {
        let existing_aliases = existing_lockfile
            .entries
            .iter()
            .map(|entry| entry.alias.as_str())
            .collect::<BTreeSet<_>>();
        let new_aliases = seen_git_aliases
            .iter()
            .map(|alias| alias.as_str())
            .collect::<BTreeSet<_>>();
        if existing_aliases != new_aliases {
            return Err(lock_mismatch_error(
                "package.yaml git dependencies do not match the aliases pinned in fol.lock",
            ));
        }
    }

    Ok(FetchResolution {
        preparation,
        resolved_packages,
        lockfile,
        lockfile_path: lockfile_path(workspace),
        package_store_root,
    })
}

fn load_existing_lockfile(workspace: &FrontendWorkspace) -> FrontendResult<fol_package::PackageLockfile> {
    let path = lockfile_path(workspace);
    let raw = std::fs::read_to_string(&path).map_err(|error| {
        FrontendError::new(
            crate::FrontendErrorKind::InvalidInput,
            format!(
                "locked fetch requires an existing fol.lock at '{}': {}",
                path.display(),
                error
            ),
        )
        .with_note("run `fol fetch` first to create fol.lock")
    })?;
    fol_package::parse_package_lockfile(&raw).map_err(FrontendError::from)
}

fn lock_mismatch_error(message: impl Into<String>) -> FrontendError {
    FrontendError::new(crate::FrontendErrorKind::InvalidInput, message.into())
        .with_note("run `fol fetch` or `fol update` to refresh fol.lock")
}

fn absolute_dependency_root(package_root: &Path, target: &str) -> PathBuf {
    let path = Path::new(target);
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        package_root.join(path)
    }
}

fn locator_use_path_segments(locator: &fol_package::PackageLocator) -> Vec<fol_parser::ast::UsePathSegment> {
    locator
        .path_segments
        .iter()
        .enumerate()
        .map(|(index, spelling)| fol_parser::ast::UsePathSegment {
            separator: (index > 0).then_some(fol_parser::ast::UsePathSeparator::Slash),
            spelling: spelling.clone(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{
        fetch_workspace, fetch_workspace_with_config, prepare_workspace_packages,
        select_package_store_root,
    };
    use crate::{FrontendConfig, FrontendWorkspace, PackageRoot, WorkspaceRoot};
    use std::{fs, path::{Path, PathBuf}, process::Command};

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
    fn package_preparation_reads_formal_workspace_members() {
        let root = std::env::temp_dir().join(format!("fol_frontend_prepare_{}", std::process::id()));
        let app = root.join("app");
        fs::create_dir_all(&app).unwrap();
        fs::write(app.join("package.yaml"), "name: app\nversion: 0.1.0\n").unwrap();
        fs::write(app.join("build.fol"), "def root: loc = \"src\"\n").unwrap();

        let workspace = FrontendWorkspace {
            root: WorkspaceRoot::new(root.clone()),
            members: vec![PackageRoot::new(app.clone())],
            std_root_override: Some(root.join("std")),
            package_store_root_override: Some(root.join(".fol/pkg")),
            build_root: root.join(".fol/build"),
            cache_root: root.join(".fol/cache"),
            git_cache_root: root.join(".fol/cache/git"),
        };

        let preparation = prepare_workspace_packages(&workspace).unwrap();

        assert_eq!(preparation.packages.len(), 1);
        assert_eq!(preparation.packages[0].root, app);
        assert_eq!(preparation.packages[0].name, "app");
        assert_eq!(
            preparation.package_config.package_store_root,
            Some(root.join(".fol/pkg").to_string_lossy().to_string())
        );

        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn package_preparation_rejects_members_without_formal_build_files() {
        let root = std::env::temp_dir().join(format!("fol_frontend_prepare_missing_{}", std::process::id()));
        let app = root.join("app");
        fs::create_dir_all(&app).unwrap();
        fs::write(app.join("package.yaml"), "name: app\nversion: 0.1.0\n").unwrap();

        let workspace = FrontendWorkspace {
            root: WorkspaceRoot::new(root.clone()),
            members: vec![PackageRoot::new(app)],
            std_root_override: None,
            package_store_root_override: None,
            build_root: root.join(".fol/build"),
            cache_root: root.join(".fol/cache"),
            git_cache_root: root.join(".fol/cache/git"),
        };

        let error = prepare_workspace_packages(&workspace).unwrap_err();

        assert_eq!(error.kind(), crate::FrontendErrorKind::PackageFailed);
        assert!(error.message().contains("build file"));

        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn fetch_workspace_returns_a_command_result_for_prepared_members() {
        let root = std::env::temp_dir().join(format!("fol_frontend_fetch_{}", std::process::id()));
        let app = root.join("app");
        fs::create_dir_all(&app).unwrap();
        fs::write(app.join("package.yaml"), "name: app\nversion: 0.1.0\n").unwrap();
        fs::write(app.join("build.fol"), "def root: loc = \"src\"\n").unwrap();

        let workspace = FrontendWorkspace {
            root: WorkspaceRoot::new(root.clone()),
            members: vec![PackageRoot::new(app.clone())],
            std_root_override: None,
            package_store_root_override: None,
            build_root: root.join(".fol/build"),
            cache_root: root.join(".fol/cache"),
            git_cache_root: root.join(".fol/cache/git"),
        };

        let result = fetch_workspace(&workspace).unwrap();

        assert_eq!(result.command, "fetch");
        assert!(result.summary.contains("prepared 1 workspace package(s) and resolved 1 package root(s) into"));
        assert_eq!(result.artifacts.len(), 5);
        assert_eq!(result.artifacts[4].path, Some(app));

        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn package_store_root_selection_prefers_frontend_config_then_workspace_then_default() {
        let workspace = FrontendWorkspace {
            root: WorkspaceRoot::new(PathBuf::from("/tmp/demo")),
            members: Vec::new(),
            std_root_override: None,
            package_store_root_override: Some(PathBuf::from("/tmp/demo/ws-pkg")),
            build_root: PathBuf::from("/tmp/demo/.fol/build"),
            cache_root: PathBuf::from("/tmp/demo/.fol/cache"),
            git_cache_root: PathBuf::from("/tmp/demo/.fol/cache/git"),
        };
        let config = FrontendConfig {
            package_store_root_override: Some(PathBuf::from("/tmp/demo/config-pkg")),
            ..FrontendConfig::default()
        };

        assert_eq!(
            select_package_store_root(&config, &workspace),
            PathBuf::from("/tmp/demo/config-pkg")
        );
        assert_eq!(
            select_package_store_root(&FrontendConfig::default(), &workspace),
            PathBuf::from("/tmp/demo/ws-pkg")
        );
        assert_eq!(
            select_package_store_root(
                &FrontendConfig::default(),
                &FrontendWorkspace {
                    package_store_root_override: None,
                    git_cache_root: PathBuf::from("/tmp/demo/.fol/cache/git"),
                    ..workspace
                }
            ),
            PathBuf::from("/tmp/demo/.fol/pkg")
        );
    }

    #[test]
    fn fetch_summary_prefers_configured_store_root_in_reported_artifacts() {
        let root = std::env::temp_dir().join(format!("fol_frontend_fetch_summary_{}", std::process::id()));
        let app = root.join("app");
        fs::create_dir_all(&app).unwrap();
        fs::write(app.join("package.yaml"), "name: app\nversion: 0.1.0\n").unwrap();
        fs::write(app.join("build.fol"), "def root: loc = \"src\"\n").unwrap();

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

        let result = fetch_workspace_with_config(&workspace, &config).unwrap();

        assert_eq!(result.artifacts[0].path, Some(root.join(".fol/config-pkg")));
        assert_eq!(result.artifacts[1].path, Some(root.join(".fol/cache")));
        assert_eq!(result.artifacts[2].path, Some(root.join(".fol/cache/git")));
        assert_eq!(result.artifacts[4].path, Some(app));

        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn fetch_workspace_materializes_git_manifest_dependencies_and_writes_lockfile() {
        let root = temp_root("git_dep");
        let app = root.join("app");
        let remote = root.join("remote-logtiny");
        create_package_repo(&remote, "logtiny", "0.1.0");
        fs::create_dir_all(app.join("src")).expect("should create app package");
        fs::write(
            app.join("package.yaml"),
            format!(
                "name: app\nversion: 0.1.0\ndep.logtiny: git:git+file://{}\n",
                remote.display()
            ),
        )
        .expect("should write app manifest");
        fs::write(app.join("build.fol"), "def root: loc = \"src\"\n").expect("should write app build");
        fs::write(app.join("src/main.fol"), "var[exp] answer: int = 1\n").expect("should write app source");

        let workspace = FrontendWorkspace {
            root: WorkspaceRoot::new(root.clone()),
            members: vec![PackageRoot::new(app.clone())],
            std_root_override: None,
            package_store_root_override: Some(root.join(".fol/pkg")),
            build_root: root.join(".fol/build"),
            cache_root: root.join(".fol/cache"),
            git_cache_root: root.join(".fol/cache/git"),
        };

        let result = fetch_workspace(&workspace).expect("fetch should succeed");

        assert!(root.join("fol.lock").is_file());
        let lockfile = fs::read_to_string(root.join("fol.lock")).expect("lockfile should be readable");
        assert!(lockfile.contains("alias: logtiny"));
        assert!(lockfile.contains("source: git"));
        assert!(result.artifacts.iter().any(|artifact| {
            artifact.label == "lockfile" && artifact.path == Some(root.join("fol.lock"))
        }));
        assert!(result.artifacts.iter().any(|artifact| {
            artifact.path
                .as_ref()
                .map(|path| path.to_string_lossy().contains("rev_"))
                .unwrap_or(false)
        }));

        fs::remove_dir_all(root).ok();
    }

    fn create_package_repo(root: &Path, name: &str, version: &str) {
        fs::create_dir_all(root.join("src")).expect("package repo should be creatable");
        fs::write(
            root.join("package.yaml"),
            format!("name: {name}\nversion: {version}\n"),
        )
        .expect("package metadata should be writable");
        fs::write(root.join("build.fol"), "def root: loc = \"src\"\n")
            .expect("package build should be writable");
        fs::write(root.join("src/lib.fol"), "var[exp] level: int = 1\n")
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
}
