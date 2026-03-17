use crate::build_graph::BuildGraph;
use crate::build_graph::{BuildOptionId, BuildOptionKind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StandardTargetRequest {
    pub name: String,
    pub default: Option<String>,
}

impl StandardTargetRequest {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            default: None,
        }
    }

    pub fn with_default(mut self, default: impl Into<String>) -> Self {
        self.default = Some(default.into());
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StandardOptimizeRequest {
    pub name: String,
    pub default: Option<String>,
}

impl StandardOptimizeRequest {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            default: None,
        }
    }

    pub fn with_default(mut self, default: impl Into<String>) -> Self {
        self.default = Some(default.into());
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StandardTargetOption {
    pub id: BuildOptionId,
    pub name: String,
    pub default: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StandardOptimizeOption {
    pub id: BuildOptionId,
    pub name: String,
    pub default: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BuildOptionValue {
    Bool(bool),
    String(String),
    Enum(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserOptionRequest {
    pub name: String,
    pub kind: BuildOptionKind,
    pub default: Option<BuildOptionValue>,
}

impl UserOptionRequest {
    pub fn bool(name: impl Into<String>, default: bool) -> Self {
        Self {
            name: name.into(),
            kind: BuildOptionKind::Bool,
            default: Some(BuildOptionValue::Bool(default)),
        }
    }

    pub fn string(name: impl Into<String>, default: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            kind: BuildOptionKind::String,
            default: Some(BuildOptionValue::String(default.into())),
        }
    }

    pub fn enumeration(name: impl Into<String>, default: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            kind: BuildOptionKind::Enum,
            default: Some(BuildOptionValue::Enum(default.into())),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserOption {
    pub id: BuildOptionId,
    pub name: String,
    pub kind: BuildOptionKind,
    pub default: Option<BuildOptionValue>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutableRequest {
    pub name: String,
    pub root_module: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StaticLibraryRequest {
    pub name: String,
    pub root_module: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SharedLibraryRequest {
    pub name: String,
    pub root_module: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TestArtifactRequest {
    pub name: String,
    pub root_module: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BuildApiNameError {
    Empty,
    InvalidCharacter(char),
}

pub fn validate_build_name(name: &str) -> Result<(), BuildApiNameError> {
    if name.is_empty() {
        return Err(BuildApiNameError::Empty);
    }

    for ch in name.chars() {
        if ch.is_ascii_lowercase() || ch.is_ascii_digit() || matches!(ch, '-' | '_' | '.') {
            continue;
        }
        return Err(BuildApiNameError::InvalidCharacter(ch));
    }

    Ok(())
}

#[derive(Debug)]
pub struct BuildApi<'a> {
    graph: &'a mut BuildGraph,
}

impl<'a> BuildApi<'a> {
    pub fn new(graph: &'a mut BuildGraph) -> Self {
        Self { graph }
    }

    pub fn graph(&self) -> &BuildGraph {
        self.graph
    }

    pub fn graph_mut(&mut self) -> &mut BuildGraph {
        self.graph
    }

    pub fn standard_target(&mut self, request: StandardTargetRequest) -> StandardTargetOption {
        let option_id = self.graph.add_option(BuildOptionKind::Target, request.name.clone());
        StandardTargetOption {
            id: option_id,
            name: request.name,
            default: request.default,
        }
    }

    pub fn standard_optimize(
        &mut self,
        request: StandardOptimizeRequest,
    ) -> StandardOptimizeOption {
        let option_id = self.graph.add_option(BuildOptionKind::Optimize, request.name.clone());
        StandardOptimizeOption {
            id: option_id,
            name: request.name,
            default: request.default,
        }
    }

    pub fn option(&mut self, request: UserOptionRequest) -> UserOption {
        let option_id = self.graph.add_option(request.kind, request.name.clone());
        UserOption {
            id: option_id,
            name: request.name,
            kind: request.kind,
            default: request.default,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        validate_build_name, BuildApi, BuildApiNameError, BuildOptionValue, ExecutableRequest,
        SharedLibraryRequest, StandardOptimizeRequest, StandardTargetRequest, StaticLibraryRequest,
        TestArtifactRequest, UserOptionRequest,
    };
    use crate::build_graph::BuildGraph;
    use crate::build_graph::BuildOptionKind;

    #[test]
    fn build_api_wraps_a_graph_reference() {
        let mut graph = BuildGraph::new();
        let api = BuildApi::new(&mut graph);

        assert!(api.graph().steps().is_empty());
    }

    #[test]
    fn build_api_exposes_mutable_graph_access() {
        let mut graph = BuildGraph::new();
        let mut api = BuildApi::new(&mut graph);

        api.graph_mut().add_step(crate::build_graph::BuildStepKind::Default, "build");

        assert_eq!(api.graph().steps().len(), 1);
    }

    #[test]
    fn build_api_records_standard_target_options_in_the_graph() {
        let mut graph = BuildGraph::new();
        let mut api = BuildApi::new(&mut graph);

        let option = api.standard_target(StandardTargetRequest::new("target").with_default("native"));

        assert_eq!(option.name, "target");
        assert_eq!(option.default.as_deref(), Some("native"));
        assert_eq!(api.graph().options()[0].id, option.id);
        assert_eq!(api.graph().options()[0].kind, BuildOptionKind::Target);
    }

    #[test]
    fn build_api_records_standard_optimize_options_in_the_graph() {
        let mut graph = BuildGraph::new();
        let mut api = BuildApi::new(&mut graph);

        let option =
            api.standard_optimize(StandardOptimizeRequest::new("optimize").with_default("debug"));

        assert_eq!(option.name, "optimize");
        assert_eq!(option.default.as_deref(), Some("debug"));
        assert_eq!(api.graph().options()[0].id, option.id);
        assert_eq!(api.graph().options()[0].kind, BuildOptionKind::Optimize);
    }

    #[test]
    fn build_api_records_boolean_user_options_in_the_graph() {
        let mut graph = BuildGraph::new();
        let mut api = BuildApi::new(&mut graph);

        let option = api.option(UserOptionRequest::bool("strip", false));

        assert_eq!(option.name, "strip");
        assert_eq!(option.kind, BuildOptionKind::Bool);
        assert_eq!(option.default, Some(BuildOptionValue::Bool(false)));
        assert_eq!(api.graph().options()[0].kind, BuildOptionKind::Bool);
    }

    #[test]
    fn build_api_records_string_and_enum_user_options_in_the_graph() {
        let mut graph = BuildGraph::new();
        let mut api = BuildApi::new(&mut graph);

        let prefix = api.option(UserOptionRequest::string("prefix", "/usr/local"));
        let flavor = api.option(UserOptionRequest::enumeration("flavor", "release"));

        assert_eq!(
            prefix.default,
            Some(BuildOptionValue::String("/usr/local".to_string()))
        );
        assert_eq!(
            flavor.default,
            Some(BuildOptionValue::Enum("release".to_string()))
        );
        assert_eq!(api.graph().options()[0].kind, BuildOptionKind::String);
        assert_eq!(api.graph().options()[1].kind, BuildOptionKind::Enum);
    }

    #[test]
    fn build_name_validation_accepts_the_draft_public_naming_rules() {
        assert_eq!(validate_build_name("app"), Ok(()));
        assert_eq!(validate_build_name("app-main"), Ok(()));
        assert_eq!(validate_build_name("app.main_1"), Ok(()));
    }

    #[test]
    fn build_name_validation_rejects_empty_and_mixed_case_names() {
        assert_eq!(validate_build_name(""), Err(BuildApiNameError::Empty));
        assert_eq!(
            validate_build_name("App"),
            Err(BuildApiNameError::InvalidCharacter('A'))
        );
    }

    #[test]
    fn structured_artifact_requests_keep_name_and_root_module_fields() {
        let exe = ExecutableRequest {
            name: "app".to_string(),
            root_module: "src/app.fol".to_string(),
        };
        let static_lib = StaticLibraryRequest {
            name: "support".to_string(),
            root_module: "src/support.fol".to_string(),
        };
        let shared_lib = SharedLibraryRequest {
            name: "plugin".to_string(),
            root_module: "src/plugin.fol".to_string(),
        };
        let tests = TestArtifactRequest {
            name: "app-tests".to_string(),
            root_module: "test/app.fol".to_string(),
        };

        assert_eq!(exe.root_module, "src/app.fol");
        assert_eq!(static_lib.name, "support");
        assert_eq!(shared_lib.name, "plugin");
        assert_eq!(tests.root_module, "test/app.fol");
    }
}
