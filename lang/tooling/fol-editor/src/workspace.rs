use crate::{EditorConfig, EditorDocument, EditorError, EditorErrorKind, EditorResult};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditorWorkspaceMapping {
    pub document_path: PathBuf,
    pub package_root: Option<PathBuf>,
    pub workspace_root: Option<PathBuf>,
    pub analysis_root: PathBuf,
}

#[derive(Debug)]
pub struct EditorAnalysisOverlay {
    temp_root: PathBuf,
    analysis_root: PathBuf,
    package_root: Option<PathBuf>,
    document_path: PathBuf,
}

impl EditorAnalysisOverlay {
    pub fn analysis_root(&self) -> &Path {
        &self.analysis_root
    }

    pub fn package_root(&self) -> Option<&Path> {
        self.package_root.as_deref()
    }

    pub fn document_path(&self) -> &Path {
        &self.document_path
    }
}

impl Drop for EditorAnalysisOverlay {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.temp_root);
    }
}

pub fn map_document_workspace(
    path: &Path,
    config: &EditorConfig,
) -> EditorResult<EditorWorkspaceMapping> {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::fs::canonicalize(path).map_err(|error| {
            EditorError::new(
                EditorErrorKind::InvalidDocumentPath,
                format!("failed to resolve '{}': {error}", path.display()),
            )
        })?
    };
    let directory = absolute.parent().ok_or_else(|| {
        EditorError::new(
            EditorErrorKind::InvalidDocumentPath,
            format!("document '{}' has no parent directory", absolute.display()),
        )
    })?;
    let package_root = find_upward_marker(directory, "package.yaml");
    let workspace_root = config
        .root_markers
        .iter()
        .filter(|marker| marker.as_str() != "package.yaml")
        .find_map(|marker| find_upward_marker(directory, marker));
    let analysis_root = workspace_root
        .clone()
        .or_else(|| package_root.clone())
        .unwrap_or_else(|| directory.to_path_buf());
    Ok(EditorWorkspaceMapping {
        document_path: absolute,
        package_root,
        workspace_root,
        analysis_root,
    })
}

pub fn materialize_analysis_overlay(
    mapping: &EditorWorkspaceMapping,
    document: &EditorDocument,
) -> EditorResult<EditorAnalysisOverlay> {
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system time should be after epoch")
        .as_nanos();
    let temp_root = std::env::temp_dir().join(format!(
        "fol_editor_overlay_{}_{}_{}",
        std::process::id(),
        mapping
            .document_path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or("doc"),
        stamp
    ));
    fs::create_dir_all(&temp_root).map_err(|error| {
        EditorError::new(
            EditorErrorKind::Internal,
            format!("failed to create overlay root '{}': {error}", temp_root.display()),
        )
    })?;

    copy_directory_tree(&mapping.analysis_root, &temp_root)?;

    let relative_document = mapping
        .document_path
        .strip_prefix(&mapping.analysis_root)
        .map_err(|_| {
            EditorError::new(
                EditorErrorKind::Internal,
                format!(
                    "document '{}' is not inside analysis root '{}'",
                    mapping.document_path.display(),
                    mapping.analysis_root.display()
                ),
            )
        })?;
    let overlay_document = temp_root.join(relative_document);
    if let Some(parent) = overlay_document.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            EditorError::new(
                EditorErrorKind::Internal,
                format!("failed to create overlay parent '{}': {error}", parent.display()),
            )
        })?;
    }
    fs::write(&overlay_document, &document.text).map_err(|error| {
        EditorError::new(
            EditorErrorKind::Internal,
            format!(
                "failed to write overlay document '{}': {error}",
                overlay_document.display()
            ),
        )
    })?;

    let package_root = mapping
        .package_root
        .as_ref()
        .and_then(|package_root| package_root.strip_prefix(&mapping.analysis_root).ok())
        .map(|relative| temp_root.join(relative));

    Ok(EditorAnalysisOverlay {
        temp_root: temp_root.clone(),
        analysis_root: temp_root,
        package_root,
        document_path: overlay_document,
    })
}

fn find_upward_marker(start: &Path, marker: &str) -> Option<PathBuf> {
    let mut current = Some(start);
    while let Some(path) = current {
        let candidate = path.join(marker);
        if candidate.is_file() || candidate.is_dir() {
            return Some(path.to_path_buf());
        }
        current = path.parent();
    }
    None
}

fn copy_directory_tree(from: &Path, to: &Path) -> EditorResult<()> {
    for entry in fs::read_dir(from).map_err(|error| {
        EditorError::new(
            EditorErrorKind::Internal,
            format!("failed to read analysis root '{}': {error}", from.display()),
        )
    })? {
        let entry = entry.map_err(|error| {
            EditorError::new(
                EditorErrorKind::Internal,
                format!("failed to enumerate analysis root '{}': {error}", from.display()),
            )
        })?;
        let source = entry.path();
        let target = to.join(entry.file_name());
        let file_type = entry.file_type().map_err(|error| {
            EditorError::new(
                EditorErrorKind::Internal,
                format!("failed to inspect '{}': {error}", source.display()),
            )
        })?;
        if file_type.is_dir() {
            fs::create_dir_all(&target).map_err(|error| {
                EditorError::new(
                    EditorErrorKind::Internal,
                    format!("failed to create '{}': {error}", target.display()),
                )
            })?;
            copy_directory_tree(&source, &target)?;
        } else if file_type.is_file() {
            fs::copy(&source, &target).map_err(|error| {
                EditorError::new(
                    EditorErrorKind::Internal,
                    format!(
                        "failed to copy '{}' to '{}': {error}",
                        source.display(),
                        target.display()
                    ),
                )
            })?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{map_document_workspace, materialize_analysis_overlay};
    use crate::{EditorConfig, EditorDocument, EditorDocumentUri};
    use std::fs;
    use std::path::PathBuf;

    fn temp_root(label: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "fol_editor_workspace_{}_{}_{}",
            label,
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system time should be after epoch")
                .as_nanos()
        ))
    }

    #[test]
    fn workspace_mapping_finds_package_and_workspace_roots() {
        let root = temp_root("mapping");
        let package = root.join("app");
        let src = package.join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(root.join("fol.work.yaml"), "members:\n  - app\n").unwrap();
        fs::write(package.join("package.yaml"), "name: app\nversion: 0.1.0\n").unwrap();
        fs::write(src.join("main.fol"), "fun[] main(): int = {\n    return 0\n}\n").unwrap();

        let mapping = map_document_workspace(&src.join("main.fol"), &EditorConfig::default())
            .expect("mapping should succeed");

        assert_eq!(mapping.package_root, Some(package.clone()));
        assert_eq!(mapping.workspace_root, Some(root.clone()));
        assert_eq!(mapping.analysis_root, root);

        fs::remove_dir_all(package.parent().unwrap()).ok();
    }

    #[test]
    fn overlay_materialization_rewrites_the_open_document_text() {
        let root = temp_root("overlay");
        let src = root.join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(root.join("package.yaml"), "name: app\nversion: 0.1.0\n").unwrap();
        fs::write(root.join("build.fol"), "def root: loc = \"src\"\n").unwrap();
        fs::write(src.join("main.fol"), "fun[] main(): int = {\n    return 0\n}\n").unwrap();

        let path = src.join("main.fol");
        let mapping =
            map_document_workspace(&path, &EditorConfig::default()).expect("mapping should work");
        let uri = EditorDocumentUri::from_file_path(path.clone()).unwrap();
        let document =
            EditorDocument::new(uri, 2, "fun[] main(): int = {\n    return 7\n}\n".to_string())
                .unwrap();
        let overlay = materialize_analysis_overlay(&mapping, &document).unwrap();
        let mirrored = overlay.analysis_root().join("src/main.fol");

        assert_eq!(
            fs::read_to_string(mirrored).unwrap(),
            "fun[] main(): int = {\n    return 7\n}\n"
        );
        assert_eq!(overlay.package_root(), Some(overlay.analysis_root()));
        assert_eq!(overlay.document_path(), overlay.analysis_root().join("src/main.fol"));

        fs::remove_dir_all(root).ok();
    }
}
