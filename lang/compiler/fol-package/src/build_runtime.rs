use std::collections::BTreeMap;

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildRuntimeRecordField {
    pub name: String,
    pub value: BuildRuntimeExpr,
}

impl BuildRuntimeRecordField {
    pub fn new(name: impl Into<String>, value: BuildRuntimeExpr) -> Self {
        Self {
            name: name.into(),
            value,
        }
    }
}

pub fn find_record_field<'a>(
    fields: &'a [BuildRuntimeRecordField],
    name: &str,
) -> Option<&'a BuildRuntimeExpr> {
    fields
        .iter()
        .find(|field| field.name == name)
        .map(|field| &field.value)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildRuntimeReceiverKind {
    Graph,
    Handle(BuildRuntimeHandleKind),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildRuntimeMethodCall {
    pub receiver: BuildRuntimeExpr,
    pub receiver_kind: BuildRuntimeReceiverKind,
    pub method: String,
    pub arguments: Vec<BuildRuntimeExpr>,
}

impl BuildRuntimeMethodCall {
    pub fn new(
        receiver: BuildRuntimeExpr,
        receiver_kind: BuildRuntimeReceiverKind,
        method: impl Into<String>,
        arguments: Vec<BuildRuntimeExpr>,
    ) -> Self {
        Self {
            receiver,
            receiver_kind,
            method: method.into(),
            arguments,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildRuntimeDiagnosticKind {
    MissingEntry,
    UnsupportedStatement,
    UnsupportedExpression,
    UnknownMethod,
    MissingField,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildRuntimeDiagnostic {
    pub kind: BuildRuntimeDiagnosticKind,
    pub message: String,
}

impl BuildRuntimeDiagnostic {
    pub fn new(kind: BuildRuntimeDiagnosticKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BuildRuntimeStmt {
    Bind {
        local: BuildRuntimeLocalId,
        value: BuildRuntimeExpr,
    },
    Expr(BuildRuntimeExpr),
    Return(BuildRuntimeExpr),
}

#[cfg(test)]
mod tests {
    use super::{
        BuildExecutionRepresentation, BuildRuntimeExpr, BuildRuntimeFrame, BuildRuntimeHandle,
        BuildRuntimeHandleKind, BuildRuntimeLocalId, BuildRuntimeMethodCall, BuildRuntimeProgram,
        BuildRuntimeDiagnostic, BuildRuntimeDiagnosticKind, BuildRuntimeReceiverKind,
        BuildRuntimeRecordField, BuildRuntimeStmt, BuildRuntimeValue, find_record_field,
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

    #[test]
    fn runtime_statements_cover_bind_effect_and_return_flow() {
        let bind = BuildRuntimeStmt::Bind {
            local: BuildRuntimeLocalId(0),
            value: BuildRuntimeExpr::Value(BuildRuntimeValue::Target(
                "x86_64-linux-gnu".to_string(),
            )),
        };
        let effect = BuildRuntimeStmt::Expr(BuildRuntimeExpr::Local(BuildRuntimeLocalId(0)));
        let ret = BuildRuntimeStmt::Return(BuildRuntimeExpr::Value(BuildRuntimeValue::Void));

        assert!(matches!(
            bind,
            BuildRuntimeStmt::Bind {
                local: BuildRuntimeLocalId(0),
                ..
            }
        ));
        assert!(matches!(
            effect,
            BuildRuntimeStmt::Expr(BuildRuntimeExpr::Local(BuildRuntimeLocalId(0)))
        ));
        assert!(matches!(
            ret,
            BuildRuntimeStmt::Return(BuildRuntimeExpr::Value(BuildRuntimeValue::Void))
        ));
    }

    #[test]
    fn runtime_record_helpers_find_named_fields_in_object_style_configs() {
        let fields = vec![
            BuildRuntimeRecordField::new(
                "name",
                BuildRuntimeExpr::Value(BuildRuntimeValue::String("demo".to_string())),
            ),
            BuildRuntimeRecordField::new(
                "root",
                BuildRuntimeExpr::Value(BuildRuntimeValue::Path("src/main.fol".to_string())),
            ),
        ];

        assert!(matches!(
            find_record_field(&fields, "root"),
            Some(BuildRuntimeExpr::Value(BuildRuntimeValue::Path(path)))
                if path == "src/main.fol"
        ));
        assert!(find_record_field(&fields, "missing").is_none());
    }

    #[test]
    fn runtime_method_calls_capture_graph_and_handle_receivers() {
        let graph_call = BuildRuntimeMethodCall::new(
            BuildRuntimeExpr::Local(BuildRuntimeLocalId(0)),
            BuildRuntimeReceiverKind::Graph,
            "add_exe",
            vec![BuildRuntimeExpr::Record(vec![(
                "name".to_string(),
                BuildRuntimeExpr::Value(BuildRuntimeValue::String("demo".to_string())),
            )])],
        );
        let handle_call = BuildRuntimeMethodCall::new(
            BuildRuntimeExpr::Local(BuildRuntimeLocalId(1)),
            BuildRuntimeReceiverKind::Handle(BuildRuntimeHandleKind::Step),
            "depend_on",
            vec![BuildRuntimeExpr::Local(BuildRuntimeLocalId(2))],
        );

        assert!(matches!(
            graph_call.receiver_kind,
            BuildRuntimeReceiverKind::Graph
        ));
        assert_eq!(graph_call.method, "add_exe");
        assert!(matches!(
            handle_call.receiver_kind,
            BuildRuntimeReceiverKind::Handle(BuildRuntimeHandleKind::Step)
        ));
        assert_eq!(handle_call.arguments.len(), 1);
    }

    #[test]
    fn runtime_diagnostics_cover_translation_and_evaluation_failures() {
        let unsupported_statement = BuildRuntimeDiagnostic::new(
            BuildRuntimeDiagnosticKind::UnsupportedStatement,
            "build runtime does not yet lower `when` statements",
        );
        let missing_field = BuildRuntimeDiagnostic::new(
            BuildRuntimeDiagnosticKind::MissingField,
            "artifact config is missing the required `root` field",
        );

        assert_eq!(
            unsupported_statement.kind,
            BuildRuntimeDiagnosticKind::UnsupportedStatement
        );
        assert!(unsupported_statement.message.contains("when"));
        assert_eq!(missing_field.kind, BuildRuntimeDiagnosticKind::MissingField);
        assert!(missing_field.message.contains("root"));
    }
}
