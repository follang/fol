use crate::{
    BackendArtifact, BackendBuildPaths, BackendConfig, BackendError, BackendErrorKind,
    BackendMode, BackendResult, BackendSession,
};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use super::skeleton::emit_generated_crate_skeleton;

pub fn backend_build_paths(output_root: &Path) -> BackendBuildPaths {
    BackendBuildPaths {
        output_root: output_root.display().to_string(),
        build_root: output_root.join("fol-backend").display().to_string(),
        bin_root: output_root.join("bin").display().to_string(),
        runtime_root: output_root
            .join("fol-backend")
            .join("runtime")
            .display()
            .to_string(),
    }
}

pub fn prepare_backend_build_paths(output_root: &Path) -> BackendResult<BackendBuildPaths> {
    let paths = backend_build_paths(output_root);
    for dir in [&paths.build_root, &paths.bin_root, &paths.runtime_root] {
        fs::create_dir_all(dir).map_err(|error| {
            BackendError::new(
                BackendErrorKind::EmissionFailure,
                format!("failed to create backend output dir '{}': {error}", dir),
            )
        })?;
    }
    Ok(paths)
}

pub fn write_generated_crate(
    output_root: &Path,
    artifact: &BackendArtifact,
) -> BackendResult<PathBuf> {
    let BackendArtifact::RustSourceCrate { root, files } = artifact else {
        return Err(BackendError::new(
            BackendErrorKind::InvalidInput,
            "write_generated_crate expects a RustSourceCrate artifact",
        ));
    };

    let crate_root = output_root.join(root);
    if crate_root.exists() {
        fs::remove_dir_all(&crate_root).map_err(|error| {
            BackendError::new(
                BackendErrorKind::EmissionFailure,
                format!(
                    "failed to clean generated crate root '{}': {error}",
                    crate_root.display()
                ),
            )
        })?;
    }
    fs::create_dir_all(&crate_root).map_err(|error| {
        BackendError::new(
            BackendErrorKind::EmissionFailure,
            format!(
                "failed to create generated crate root '{}': {error}",
                crate_root.display()
            ),
        )
    })?;

    for file in files {
        let path = crate_root.join(&file.path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|error| {
                BackendError::new(
                    BackendErrorKind::EmissionFailure,
                    format!(
                        "failed to create generated module dir '{}': {error}",
                        parent.display()
                    ),
                )
            })?;
        }
        fs::write(&path, &file.contents).map_err(|error| {
            BackendError::new(
                BackendErrorKind::EmissionFailure,
                format!(
                    "failed to write generated file '{}': {error}",
                    path.display()
                ),
            )
        })?;
    }

    Ok(crate_root)
}

pub fn prepare_generated_build_dir(output_root: &Path) -> BackendResult<PathBuf> {
    Ok(PathBuf::from(prepare_backend_build_paths(output_root)?.build_root))
}

pub fn build_generated_crate(crate_root: &Path) -> BackendResult<PathBuf> {
    let manifest_path = crate_root.join("Cargo.toml");
    let output = Command::new("cargo")
        .arg("build")
        .arg("--manifest-path")
        .arg(&manifest_path)
        .arg("--release")
        .output()
        .map_err(|error| {
            BackendError::new(
                BackendErrorKind::BuildFailure,
                format!(
                    "failed to launch cargo build for '{}': {error}",
                    manifest_path.display()
                ),
            )
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        return Err(BackendError::new(
            BackendErrorKind::BuildFailure,
            format!(
                "cargo build failed for '{}'\nstdout:\n{}\nstderr:\n{}",
                manifest_path.display(),
                stdout.trim(),
                stderr.trim()
            ),
        ));
    }

    let package_name = crate_root
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| {
            BackendError::new(
                BackendErrorKind::BuildFailure,
                format!(
                    "generated crate root '{}' does not have a valid package name",
                    crate_root.display()
                ),
            )
        })?;
    let binary_path = crate_root.join("target").join("release").join(package_name);
    if !binary_path.exists() {
        return Err(BackendError::new(
            BackendErrorKind::BuildFailure,
            format!(
                "cargo build succeeded but '{}' is missing",
                binary_path.display()
            ),
        ));
    }

    Ok(binary_path)
}

pub fn emit_backend_artifact(
    session: &BackendSession,
    config: &BackendConfig,
    output_root: &Path,
) -> BackendResult<BackendArtifact> {
    let paths = prepare_backend_build_paths(output_root)?;
    let build_root = PathBuf::from(&paths.build_root);
    let source_artifact = emit_generated_crate_skeleton(session)?;
    let crate_root = write_generated_crate(&build_root, &source_artifact)?;

    if matches!(config.mode, BackendMode::EmitSource) {
        let BackendArtifact::RustSourceCrate { files, .. } = source_artifact else {
            return Err(BackendError::new(
                BackendErrorKind::InvalidInput,
                "generated crate skeleton produced an unexpected artifact type",
            ));
        };
        return Ok(BackendArtifact::RustSourceCrate {
            root: crate_root.display().to_string(),
            files,
        });
    }

    let built_binary = build_generated_crate(&crate_root)?;
    let final_binary_dir = PathBuf::from(&paths.bin_root);
    let final_binary = final_binary_dir.join(built_binary.file_name().ok_or_else(|| {
        BackendError::new(
            BackendErrorKind::BuildFailure,
            format!(
                "built binary '{}' does not have a file name",
                built_binary.display()
            ),
        )
    })?);
    fs::copy(&built_binary, &final_binary).map_err(|error| {
        BackendError::new(
            BackendErrorKind::BuildFailure,
            format!(
                "failed to copy built binary '{}' to '{}': {error}",
                built_binary.display(),
                final_binary.display()
            ),
        )
    })?;

    if !config.keep_build_dir {
        fs::remove_dir_all(&crate_root).map_err(|error| {
            BackendError::new(
                BackendErrorKind::BuildFailure,
                format!(
                    "failed to remove generated crate dir '{}': {error}",
                    crate_root.display()
                ),
            )
        })?;
    }

    Ok(BackendArtifact::CompiledBinary {
        crate_root: crate_root.display().to_string(),
        binary_path: final_binary.display().to_string(),
    })
}

pub fn summarize_emitted_artifact(artifact: &BackendArtifact) -> String {
    match artifact {
        BackendArtifact::RustSourceCrate { root, files } => format!(
            "generated Rust crate root={root} files={}",
            files
                .iter()
                .map(|file| file.path.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        ),
        BackendArtifact::CompiledBinary {
            crate_root,
            binary_path,
        } => format!("compiled backend artifact crate_root={crate_root} binary={binary_path}"),
    }
}
