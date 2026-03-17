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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BuildRuntimeLocalId(pub u32);

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BuildRuntimeFrame {
    locals: BTreeMap<BuildRuntimeLocalId, BuildRuntimeValue>,
}

impl BuildRuntimeFrame {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn bind(&mut self, local: BuildRuntimeLocalId, value: BuildRuntimeValue) {
        self.locals.insert(local, value);
    }

    pub fn get(&self, local: BuildRuntimeLocalId) -> Option<&BuildRuntimeValue> {
        self.locals.get(&local)
    }

    pub fn alias(&mut self, target: BuildRuntimeLocalId, source: BuildRuntimeLocalId) -> bool {
        let Some(value) = self.locals.get(&source).cloned() else {
            return false;
        };
        self.locals.insert(target, value);
        true
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BuildRuntimeExpr {
    Local(BuildRuntimeLocalId),
    Value(BuildRuntimeValue),
    Record(Vec<(String, BuildRuntimeExpr)>),
}

#[cfg(test)]
mod tests {
    use super::{
        BuildExecutionRepresentation, BuildRuntimeExpr, BuildRuntimeFrame,
        BuildRuntimeHandle, BuildRuntimeHandleKind, BuildRuntimeLocalId, BuildRuntimeProgram,
        BuildRuntimeValue,
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

    #[test]
    fn runtime_frames_preserve_handle_aliases_across_repeated_local_flow() {
        let handle = BuildRuntimeValue::Handle(BuildRuntimeHandle::new(
            BuildRuntimeHandleKind::Artifact,
            "app",
        ));
        let mut frame = BuildRuntimeFrame::new();

        frame.bind(BuildRuntimeLocalId(0), handle.clone());
        assert!(frame.alias(BuildRuntimeLocalId(1), BuildRuntimeLocalId(0)));

        assert_eq!(frame.get(BuildRuntimeLocalId(0)), Some(&handle));
        assert_eq!(frame.get(BuildRuntimeLocalId(1)), Some(&handle));
        assert!(!frame.alias(BuildRuntimeLocalId(2), BuildRuntimeLocalId(99)));
    }

    #[test]
    fn runtime_expressions_cover_locals_literals_and_object_style_records() {
        let expression = BuildRuntimeExpr::Record(vec![
            (
                "name".to_string(),
                BuildRuntimeExpr::Value(BuildRuntimeValue::String("demo".to_string())),
            ),
            ("artifact".to_string(), BuildRuntimeExpr::Local(BuildRuntimeLocalId(3))),
        ]);

        assert!(matches!(
            expression,
            BuildRuntimeExpr::Record(fields)
            if fields[0].0 == "name"
                && matches!(fields[1].1, BuildRuntimeExpr::Local(BuildRuntimeLocalId(3)))
        ));
    }
}
use std::collections::BTreeMap;
