#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildExecutionRepresentation {
    RestrictedRuntimeIr,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildRuntimeProgram {
    representation: BuildExecutionRepresentation,
}

impl BuildRuntimeProgram {
    pub fn new(representation: BuildExecutionRepresentation) -> Self {
        Self { representation }
    }

    pub fn representation(&self) -> BuildExecutionRepresentation {
        self.representation
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildRuntimeHandleKind {
    Graph,
    Artifact,
    Step,
    Run,
    Install,
    Dependency,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildRuntimeHandle {
    pub kind: BuildRuntimeHandleKind,
    pub identity: String,
}

impl BuildRuntimeHandle {
    pub fn new(kind: BuildRuntimeHandleKind, identity: impl Into<String>) -> Self {
        Self {
            kind,
            identity: identity.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BuildRuntimeValue {
    Void,
    Bool(bool),
    Int(i64),
    String(String),
    Path(String),
    Target(String),
    Optimize(String),
    Handle(BuildRuntimeHandle),
}

#[cfg(test)]
mod tests {
    use super::{
        BuildExecutionRepresentation, BuildRuntimeHandle, BuildRuntimeHandleKind,
        BuildRuntimeProgram, BuildRuntimeValue,
    };

    #[test]
    fn runtime_programs_record_the_chosen_execution_representation() {
        let program = BuildRuntimeProgram::new(BuildExecutionRepresentation::RestrictedRuntimeIr);

        assert_eq!(
            program.representation(),
            BuildExecutionRepresentation::RestrictedRuntimeIr
        );
    }

    #[test]
    fn runtime_values_cover_the_initial_build_handle_and_option_surface() {
        let graph = BuildRuntimeValue::Handle(BuildRuntimeHandle::new(
            BuildRuntimeHandleKind::Graph,
            "graph",
        ));
        let target = BuildRuntimeValue::Target("x86_64-linux-gnu".to_string());
        let optimize = BuildRuntimeValue::Optimize("release-safe".to_string());

        assert!(matches!(
            graph,
            BuildRuntimeValue::Handle(BuildRuntimeHandle {
                kind: BuildRuntimeHandleKind::Graph,
                ..
            })
        ));
        assert_eq!(target, BuildRuntimeValue::Target("x86_64-linux-gnu".to_string()));
        assert_eq!(
            optimize,
            BuildRuntimeValue::Optimize("release-safe".to_string())
        );
    }
}
