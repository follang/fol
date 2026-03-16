use crate::{FrontendError, FrontendErrorKind, FrontendResult, PackageRoot, WorkspaceRoot};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FrontendWorkspaceConfig {
    pub members: Vec<PathBuf>,
    pub std_root_override: Option<PathBuf>,
    pub package_store_root_override: Option<PathBuf>,
    pub build_root_override: Option<PathBuf>,
    pub cache_root_override: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrontendWorkspace {
    pub root: WorkspaceRoot,
    pub members: Vec<PackageRoot>,
    pub std_root_override: Option<PathBuf>,
    pub package_store_root_override: Option<PathBuf>,
    pub build_root: PathBuf,
    pub cache_root: PathBuf,
}

impl FrontendWorkspace {
    pub fn new(root: WorkspaceRoot) -> Self {
        Self {
            root,
            members: Vec::new(),
            std_root_override: None,
            package_store_root_override: None,
            build_root: PathBuf::from("/tmp/demo/.fol/build"),
            cache_root: PathBuf::from("/tmp/demo/.fol/cache"),
        }
    }

    pub fn with_members(root: WorkspaceRoot, member_paths: &[PathBuf]) -> FrontendResult<Self> {
        Ok(Self {
            members: enumerate_member_packages(&root, member_paths)?,
            build_root: default_build_root(&root.root),
            cache_root: default_cache_root(&root.root),
            root,
            std_root_override: None,
            package_store_root_override: None,
        })
    }

    pub fn from_config(root: WorkspaceRoot, config: &FrontendWorkspaceConfig) -> FrontendResult<Self> {
        Ok(Self {
            members: enumerate_member_packages(&root, &config.members)?,
            std_root_override: config
                .std_root_override
                .as_ref()
                .map(|path| absolute_member_root(&root.root, path)),
            package_store_root_override: config
                .package_store_root_override
                .as_ref()
                .map(|path| absolute_member_root(&root.root, path)),
            build_root: config
                .build_root_override
                .as_ref()
                .map(|path| absolute_member_root(&root.root, path))
                .unwrap_or_else(|| default_build_root(&root.root)),
            cache_root: config
                .cache_root_override
                .as_ref()
                .map(|path| absolute_member_root(&root.root, path))
                .unwrap_or_else(|| default_cache_root(&root.root)),
            root,
        })
    }

    pub fn info_summary_lines(&self) -> Vec<String> {
        let mut lines = vec![
            format!("root={}", self.root.root.display()),
            format!("members={}", self.members.len()),
            format!("build_root={}", self.build_root.display()),
            format!("cache_root={}", self.cache_root.display()),
        ];

        if let Some(std_root) = &self.std_root_override {
            lines.push(format!("std_root={}", std_root.display()));
        }
        if let Some(package_store_root) = &self.package_store_root_override {
            lines.push(format!("package_store_root={}", package_store_root.display()));
        }

        lines
    }
}

pub fn enumerate_member_packages(
    workspace_root: &WorkspaceRoot,
    member_paths: &[PathBuf],
) -> FrontendResult<Vec<PackageRoot>> {
    member_paths
        .iter()
        .map(|member| {
            let absolute = absolute_member_root(&workspace_root.root, member);
            let manifest_file = absolute.join(crate::PACKAGE_FILE_NAME);
            if !manifest_file.is_file() {
                return Err(FrontendError::new(
                    FrontendErrorKind::InvalidInput,
                    format!(
                        "workspace member '{}' is missing '{}'",
                        absolute.display(),
                        crate::PACKAGE_FILE_NAME
                    ),
                ));
            }
            Ok(PackageRoot::new(absolute))
        })
        .collect()
}

pub fn load_workspace_config(workspace_root: &WorkspaceRoot) -> FrontendResult<FrontendWorkspaceConfig> {
    let raw = std::fs::read_to_string(&workspace_root.config_file).map_err(|error| {
        FrontendError::new(
            FrontendErrorKind::CommandFailed,
            format!(
                "could not read workspace config '{}': {}",
                workspace_root.config_file.display(),
                error
            ),
        )
    })?;

    let mut config = FrontendWorkspaceConfig::default();
    let mut in_members = false;

    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        if in_members && trimmed.starts_with('-') {
            let member = trimmed.trim_start_matches('-').trim();
            if member.is_empty() {
                return Err(FrontendError::new(
                    FrontendErrorKind::InvalidInput,
                    format!(
                        "workspace config '{}' has an empty member entry",
                        workspace_root.config_file.display()
                    ),
                ));
            }
            config.members.push(PathBuf::from(strip_quotes(member)));
            continue;
        }

        in_members = false;
        let Some((key, value)) = trimmed.split_once(':') else {
            return Err(FrontendError::new(
                FrontendErrorKind::InvalidInput,
                format!(
                    "workspace config '{}' must use 'key: value' lines",
                    workspace_root.config_file.display()
                ),
            ));
        };

        let key = key.trim();
        let value = value.trim();

        match key {
            "members" => {
                if !value.is_empty() {
                    return Err(FrontendError::new(
                        FrontendErrorKind::InvalidInput,
                        format!(
                            "workspace config '{}' must declare members as a list",
                            workspace_root.config_file.display()
                        ),
                    ));
                }
                in_members = true;
            }
            "std_root" => config.std_root_override = Some(PathBuf::from(strip_quotes(value))),
            "package_store_root" => {
                config.package_store_root_override = Some(PathBuf::from(strip_quotes(value)))
            }
            "build_root" => config.build_root_override = Some(PathBuf::from(strip_quotes(value))),
            "cache_root" => config.cache_root_override = Some(PathBuf::from(strip_quotes(value))),
            _ => {
                return Err(FrontendError::new(
                    FrontendErrorKind::InvalidInput,
                    format!(
                        "workspace config '{}' has unsupported field '{}'",
                        workspace_root.config_file.display(),
                        key
                    ),
                ))
            }
        }
    }

    Ok(config)
}

fn absolute_member_root(workspace_root: &Path, member: &Path) -> PathBuf {
    if member.is_absolute() {
        member.to_path_buf()
    } else {
        workspace_root.join(member)
    }
}

fn default_build_root(workspace_root: &Path) -> PathBuf {
    workspace_root.join(".fol").join("build")
}

fn default_cache_root(workspace_root: &Path) -> PathBuf {
    workspace_root.join(".fol").join("cache")
}

fn strip_quotes(raw: &str) -> &str {
    let bytes = raw.as_bytes();
    if bytes.len() >= 2 && bytes.first() == bytes.last() {
        match bytes[0] {
            b'"' | b'\'' => &raw[1..raw.len() - 1],
            _ => raw,
        }
    } else {
        raw
    }
}

#[cfg(test)]
mod tests {
    use super::{enumerate_member_packages, load_workspace_config, FrontendWorkspace, FrontendWorkspaceConfig};
    use crate::WorkspaceRoot;
    use std::{fs, path::PathBuf};

    #[test]
    fn frontend_workspace_starts_from_a_discovered_workspace_root() {
        let workspace = FrontendWorkspace::new(WorkspaceRoot::new(PathBuf::from("/tmp/demo")));

        assert_eq!(workspace.root.root, PathBuf::from("/tmp/demo"));
        assert_eq!(
            workspace.root.config_file,
            PathBuf::from("/tmp/demo/fol.work.yaml")
        );
        assert!(workspace.members.is_empty());
        assert!(workspace.std_root_override.is_none());
        assert!(workspace.package_store_root_override.is_none());
        assert_eq!(workspace.build_root, PathBuf::from("/tmp/demo/.fol/build"));
        assert_eq!(workspace.cache_root, PathBuf::from("/tmp/demo/.fol/cache"));
    }

    #[test]
    fn workspace_member_enumeration_loads_declared_package_roots() {
        let root = std::env::temp_dir().join(format!("fol_frontend_members_{}", std::process::id()));
        let app = root.join("app");
        let lib = root.join("lib");
        fs::create_dir_all(&app).unwrap();
        fs::create_dir_all(&lib).unwrap();
        fs::write(root.join("fol.work.yaml"), "members:\n  - app\n  - lib\n").unwrap();
        fs::write(app.join("package.yaml"), "name: app\nversion: 0.1.0\n").unwrap();
        fs::write(lib.join("package.yaml"), "name: lib\nversion: 0.1.0\n").unwrap();

        let members = enumerate_member_packages(
            &WorkspaceRoot::new(root.clone()),
            &[PathBuf::from("app"), PathBuf::from("lib")],
        )
        .unwrap();

        assert_eq!(members.len(), 2);
        assert_eq!(members[0].root, app);
        assert_eq!(members[1].root, lib);

        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn workspace_member_enumeration_rejects_missing_package_roots() {
        let root = std::env::temp_dir().join(format!("fol_frontend_missing_member_{}", std::process::id()));
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join("fol.work.yaml"), "members:\n  - app\n").unwrap();

        let error = enumerate_member_packages(
            &WorkspaceRoot::new(root.clone()),
            &[PathBuf::from("app")],
        )
        .unwrap_err();

        assert_eq!(error.kind(), crate::FrontendErrorKind::InvalidInput);
        assert!(error.message().contains("missing 'package.yaml'"));

        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn workspace_config_loading_extracts_declared_member_paths() {
        let root = std::env::temp_dir().join(format!("fol_frontend_config_members_{}", std::process::id()));
        fs::create_dir_all(&root).unwrap();
        fs::write(
            root.join("fol.work.yaml"),
            "members:\n  - app\n  - libs/core\n",
        )
        .unwrap();

        let config = load_workspace_config(&WorkspaceRoot::new(root.clone())).unwrap();

        assert_eq!(
            config,
            FrontendWorkspaceConfig {
                members: vec![PathBuf::from("app"), PathBuf::from("libs/core")],
                ..FrontendWorkspaceConfig::default()
            }
        );

        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn workspace_config_loading_rejects_inline_members_scalars() {
        let root = std::env::temp_dir().join(format!("fol_frontend_config_invalid_{}", std::process::id()));
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join("fol.work.yaml"), "members: app\n").unwrap();

        let error = load_workspace_config(&WorkspaceRoot::new(root.clone())).unwrap_err();

        assert_eq!(error.kind(), crate::FrontendErrorKind::InvalidInput);
        assert!(error.message().contains("must declare members as a list"));

        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn workspace_from_config_resolves_std_and_package_store_overrides() {
        let root = std::env::temp_dir().join(format!("fol_frontend_config_overrides_{}", std::process::id()));
        let app = root.join("app");
        fs::create_dir_all(&app).unwrap();
        fs::write(app.join("package.yaml"), "name: app\nversion: 0.1.0\n").unwrap();

        let workspace = FrontendWorkspace::from_config(
            WorkspaceRoot::new(root.clone()),
            &FrontendWorkspaceConfig {
                members: vec![PathBuf::from("app")],
                std_root_override: Some(PathBuf::from("std")),
                package_store_root_override: Some(PathBuf::from(".fol/pkg")),
                ..FrontendWorkspaceConfig::default()
            },
        )
        .unwrap();

        assert_eq!(workspace.members.len(), 1);
        assert_eq!(workspace.std_root_override, Some(root.join("std")));
        assert_eq!(
            workspace.package_store_root_override,
            Some(root.join(".fol/pkg"))
        );
        assert_eq!(workspace.build_root, root.join(".fol/build"));
        assert_eq!(workspace.cache_root, root.join(".fol/cache"));

        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn workspace_config_loading_extracts_std_and_package_store_overrides() {
        let root = std::env::temp_dir().join(format!("fol_frontend_config_std_pkg_{}", std::process::id()));
        fs::create_dir_all(&root).unwrap();
        fs::write(
            root.join("fol.work.yaml"),
            "members:\n  - app\nstd_root: std\npackage_store_root: .fol/pkg\n",
        )
        .unwrap();

        let config = load_workspace_config(&WorkspaceRoot::new(root.clone())).unwrap();

        assert_eq!(config.std_root_override, Some(PathBuf::from("std")));
        assert_eq!(
            config.package_store_root_override,
            Some(PathBuf::from(".fol/pkg"))
        );

        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn workspace_config_loading_extracts_build_and_cache_overrides() {
        let root = std::env::temp_dir().join(format!("fol_frontend_config_build_cache_{}", std::process::id()));
        fs::create_dir_all(&root).unwrap();
        fs::write(
            root.join("fol.work.yaml"),
            "members:\n  - app\nbuild_root: .artifacts/build\ncache_root: .artifacts/cache\n",
        )
        .unwrap();

        let config = load_workspace_config(&WorkspaceRoot::new(root.clone())).unwrap();

        assert_eq!(config.build_root_override, Some(PathBuf::from(".artifacts/build")));
        assert_eq!(config.cache_root_override, Some(PathBuf::from(".artifacts/cache")));

        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn workspace_from_config_prefers_explicit_build_and_cache_roots() {
        let root = std::env::temp_dir().join(format!("fol_frontend_workspace_paths_{}", std::process::id()));
        let app = root.join("app");
        fs::create_dir_all(&app).unwrap();
        fs::write(app.join("package.yaml"), "name: app\nversion: 0.1.0\n").unwrap();

        let workspace = FrontendWorkspace::from_config(
            WorkspaceRoot::new(root.clone()),
            &FrontendWorkspaceConfig {
                members: vec![PathBuf::from("app")],
                build_root_override: Some(PathBuf::from(".artifacts/build")),
                cache_root_override: Some(PathBuf::from(".artifacts/cache")),
                ..FrontendWorkspaceConfig::default()
            },
        )
        .unwrap();

        assert_eq!(workspace.build_root, root.join(".artifacts/build"));
        assert_eq!(workspace.cache_root, root.join(".artifacts/cache"));

        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn workspace_info_summary_renders_stable_core_fields() {
        let workspace = FrontendWorkspace::new(WorkspaceRoot::new(PathBuf::from("/tmp/demo")));

        assert_eq!(
            workspace.info_summary_lines(),
            vec![
                "root=/tmp/demo".to_string(),
                "members=0".to_string(),
                "build_root=/tmp/demo/.fol/build".to_string(),
                "cache_root=/tmp/demo/.fol/cache".to_string(),
            ]
        );
    }

    #[test]
    fn workspace_info_summary_includes_override_roots_when_present() {
        let workspace = FrontendWorkspace {
            root: WorkspaceRoot::new(PathBuf::from("/tmp/demo")),
            members: Vec::new(),
            std_root_override: Some(PathBuf::from("/tmp/demo/std")),
            package_store_root_override: Some(PathBuf::from("/tmp/demo/.fol/pkg")),
            build_root: PathBuf::from("/tmp/demo/.fol/build"),
            cache_root: PathBuf::from("/tmp/demo/.fol/cache"),
        };

        let lines = workspace.info_summary_lines();
        assert!(lines.contains(&"std_root=/tmp/demo/std".to_string()));
        assert!(lines.contains(&"package_store_root=/tmp/demo/.fol/pkg".to_string()));
    }
}
