#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeArtifactKind {
    Header,
    Object,
    StaticLibrary,
    SharedLibrary,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeSearchPathOrigin {
    PackageRoot,
    BuildRoot,
    System,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativeIncludePath {
    pub origin: NativeSearchPathOrigin,
    pub relative_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativeLibraryPath {
    pub origin: NativeSearchPathOrigin,
    pub relative_path: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeLinkMode {
    Static,
    Dynamic,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NativeLinkInput {
    Artifact(NativeArtifactDefinition),
    LibraryName(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativeLinkDirective {
    pub input: NativeLinkInput,
    pub mode: NativeLinkMode,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativeArtifactDefinition {
    pub name: String,
    pub kind: NativeArtifactKind,
    pub relative_path: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NativeArtifactSet {
    definitions: Vec<NativeArtifactDefinition>,
}

impl NativeArtifactSet {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn definitions(&self) -> &[NativeArtifactDefinition] {
        &self.definitions
    }

    pub fn add(&mut self, definition: NativeArtifactDefinition) {
        self.definitions.push(definition);
    }
}

#[cfg(test)]
mod tests {
    use super::{
        NativeArtifactDefinition, NativeArtifactKind, NativeArtifactSet, NativeIncludePath,
        NativeLibraryPath, NativeLinkDirective, NativeLinkInput, NativeLinkMode,
        NativeSearchPathOrigin,
    };

    #[test]
    fn native_artifact_set_starts_empty() {
        let set = NativeArtifactSet::new();

        assert!(set.definitions().is_empty());
    }

    #[test]
    fn native_artifact_set_preserves_inserted_shell_definitions() {
        let mut set = NativeArtifactSet::new();
        set.add(NativeArtifactDefinition {
            name: "api".to_string(),
            kind: NativeArtifactKind::Header,
            relative_path: "include/api.h".to_string(),
        });

        assert_eq!(set.definitions().len(), 1);
        assert_eq!(set.definitions()[0].name, "api");
        assert_eq!(set.definitions()[0].kind, NativeArtifactKind::Header);
        assert_eq!(set.definitions()[0].relative_path, "include/api.h");
    }

    #[test]
    fn native_artifact_kinds_cover_phase_ten_shapes() {
        assert_eq!(NativeArtifactKind::Header, NativeArtifactKind::Header);
        assert_eq!(NativeArtifactKind::Object, NativeArtifactKind::Object);
        assert_eq!(
            NativeArtifactKind::StaticLibrary,
            NativeArtifactKind::StaticLibrary
        );
        assert_eq!(
            NativeArtifactKind::SharedLibrary,
            NativeArtifactKind::SharedLibrary
        );
    }

    #[test]
    fn native_include_paths_keep_origin_and_relative_path() {
        let path = NativeIncludePath {
            origin: NativeSearchPathOrigin::PackageRoot,
            relative_path: "include".to_string(),
        };

        assert_eq!(path.origin, NativeSearchPathOrigin::PackageRoot);
        assert_eq!(path.relative_path, "include");
    }

    #[test]
    fn native_library_paths_cover_package_build_and_system_origins() {
        let package = NativeLibraryPath {
            origin: NativeSearchPathOrigin::PackageRoot,
            relative_path: "native/lib".to_string(),
        };
        let build = NativeLibraryPath {
            origin: NativeSearchPathOrigin::BuildRoot,
            relative_path: "out/lib".to_string(),
        };
        let system = NativeLibraryPath {
            origin: NativeSearchPathOrigin::System,
            relative_path: "/usr/lib".to_string(),
        };

        assert_eq!(package.origin, NativeSearchPathOrigin::PackageRoot);
        assert_eq!(build.origin, NativeSearchPathOrigin::BuildRoot);
        assert_eq!(system.origin, NativeSearchPathOrigin::System);
    }

    #[test]
    fn native_link_directives_keep_mode_and_library_inputs() {
        let directive = NativeLinkDirective {
            input: NativeLinkInput::LibraryName("ssl".to_string()),
            mode: NativeLinkMode::Dynamic,
        };

        assert_eq!(directive.mode, NativeLinkMode::Dynamic);
        assert_eq!(
            directive.input,
            NativeLinkInput::LibraryName("ssl".to_string())
        );
    }

    #[test]
    fn native_link_directives_can_reference_declared_native_artifacts() {
        let artifact = NativeArtifactDefinition {
            name: "crypto".to_string(),
            kind: NativeArtifactKind::StaticLibrary,
            relative_path: "native/libcrypto.a".to_string(),
        };
        let directive = NativeLinkDirective {
            input: NativeLinkInput::Artifact(artifact.clone()),
            mode: NativeLinkMode::Static,
        };

        assert_eq!(directive.mode, NativeLinkMode::Static);
        assert_eq!(directive.input, NativeLinkInput::Artifact(artifact));
    }
}
