use crate::artifact::BuildArtifactFolModel;
use crate::runtime::{BuildRuntimeGeneratedFileKind};

// ---- Extraction output types (public so eval.rs can build EvaluatedBuildProgram) ---

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecConfigValue {
    Literal(String),
    OptionRef(String),
}

impl ExecConfigValue {
    pub fn placeholder_string(&self) -> String {
        match self {
            Self::Literal(value) => value.clone(),
            Self::OptionRef(name) => name.clone(),
        }
    }

    pub fn resolve(&self, options: &crate::option::ResolvedBuildOptionSet) -> String {
        match self {
            Self::Literal(value) => value.clone(),
            Self::OptionRef(name) => options
                .get(name.as_str())
                .map(str::to_string)
                .unwrap_or_else(|| name.clone()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecArtifact {
    pub name: String,
    pub root_module: ExecConfigValue,
    pub fol_model: BuildArtifactFolModel,
    pub target: Option<ExecConfigValue>,
    pub optimize: Option<ExecConfigValue>,
}

// ---- Internal value type for the execution scope ---

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum ExecValue {
    Build,
    Graph,
    Target(String),
    Optimize(String),
    OptionRef(String),
    Str(String),
    Bool(bool),
    Artifact(ExecArtifact),
    Module {
        name: String,
    },
    GeneratedFile {
        name: String,
        path: String,
        kind: BuildRuntimeGeneratedFileKind,
    },
    Step {
        name: String,
    },
    Run {
        name: String,
    },
    Install {
        name: String,
    },
    Dependency {
        alias: String,
    },
    DependencyModule {
        alias: String,
        query_name: String,
    },
    DependencyArtifact {
        alias: String,
        query_name: String,
    },
    DependencyStep {
        alias: String,
        query_name: String,
    },
    List(Vec<ExecValue>),
}

// ---- Helper routine representation ---

pub(super) struct HelperRoutine {
    pub(super) params: Vec<String>,
    pub(super) body: Vec<fol_parser::ast::AstNode>,
}
