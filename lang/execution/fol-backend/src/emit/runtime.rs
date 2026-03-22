use crate::{BackendBuildPaths, BackendBuildProfile, BackendError, BackendErrorKind, BackendResult};
use std::fs;
use std::path::{Path, PathBuf};

pub fn backend_runtime_source_root() -> PathBuf {
    if let Some(path) = std::env::var_os("FOL_BACKEND_RUNTIME_PATH") {
        return PathBuf::from(path);
    }
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap_or_else(|| Path::new(env!("CARGO_MANIFEST_DIR")))
        .join("fol-runtime")
}

pub fn backend_runtime_source_entry() -> PathBuf {
    backend_runtime_source_root().join("src").join("lib.rs")
}

pub fn backend_runtime_manifest_path() -> PathBuf {
    backend_runtime_source_root().join("Cargo.toml")
}

pub fn backend_runtime_build_dir(
    paths: &BackendBuildPaths,
    profile: BackendBuildProfile,
) -> PathBuf {
    PathBuf::from(&paths.runtime_root).join(profile.as_str())
}

pub fn prepare_backend_runtime_build_dir(
    paths: &BackendBuildPaths,
    profile: BackendBuildProfile,
) -> BackendResult<PathBuf> {
    let runtime_dir = backend_runtime_build_dir(paths, profile);
    fs::create_dir_all(&runtime_dir).map_err(|error| {
        BackendError::new(
            BackendErrorKind::EmissionFailure,
            format!(
                "failed to create backend runtime build dir '{}': {error}",
                runtime_dir.display()
            ),
        )
    })?;
    Ok(runtime_dir)
}
