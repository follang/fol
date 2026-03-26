use crate::api::PathHandleProvenance;
use crate::artifact::BuildArtifactFolModel;
use crate::eval::{BuildEvaluationError, BuildEvaluationErrorKind};
use fol_parser::ast::{AstNode, Literal, RecordInitField};
use std::collections::BTreeMap;

use super::core::BuildBodyExecutor;
use super::types::{ExecArtifact, ExecConfigValue, ExecValue, ResolvedPathHandle};

impl BuildBodyExecutor {
    fn generated_path_provenance(name: &str) -> PathHandleProvenance {
        if name.starts_with("dep::") && name.contains("::generated::") {
            PathHandleProvenance::DependencyGenerated
        } else if name.starts_with("dep::") && name.contains("::path::") {
            PathHandleProvenance::DependencyPath
        } else {
            PathHandleProvenance::Generated
        }
    }

    pub(super) fn resolve_path_handle(&self, node: &AstNode) -> Option<ResolvedPathHandle> {
        let AstNode::Identifier { name, .. } = node else {
            return None;
        };
        match self.scope.get(name.as_str()) {
            Some(ExecValue::SourceFile { path }) => Some(ResolvedPathHandle::file(
                path.clone(),
                PathHandleProvenance::Source,
            )),
            Some(ExecValue::SourceDir { path }) => Some(ResolvedPathHandle::dir(
                path.clone(),
                PathHandleProvenance::Source,
            )),
            Some(ExecValue::GeneratedFile { name, path, .. }) => {
                Some(ResolvedPathHandle::generated(
                    path.clone(),
                    Self::generated_path_provenance(name),
                    name.clone(),
                ))
            }
            _ => None,
        }
    }

    pub(super) fn resolve_string(&self, node: &AstNode) -> Option<String> {
        match node {
            AstNode::Literal(Literal::String(s)) => Some(s.clone()),
            AstNode::Identifier { name, .. } => match self.scope.get(name.as_str()) {
                Some(ExecValue::Target(s)) => Some(s.clone()),
                Some(ExecValue::Optimize(s)) => Some(s.clone()),
                Some(ExecValue::Str(s)) => Some(s.clone()),
                _ => None,
            },
            _ => None,
        }
    }

    pub(super) fn resolve_field_string(
        &self,
        fields: &[RecordInitField],
        field_name: &str,
    ) -> Option<String> {
        fields
            .iter()
            .find(|f| f.name == field_name)
            .and_then(|f| self.resolve_string(&f.value))
    }

    pub(super) fn parse_config_value(
        &self,
        node: &AstNode,
        _allowed_kinds: &[&str],
    ) -> Option<ExecConfigValue> {
        match node {
            AstNode::Literal(Literal::String(s)) => Some(ExecConfigValue::Literal(s.clone())),
            AstNode::Identifier { name, .. } => match self.scope.get(name.as_str()) {
                Some(ExecValue::Target(option_name))
                | Some(ExecValue::Optimize(option_name))
                | Some(ExecValue::OptionRef(option_name)) => {
                    Some(ExecConfigValue::OptionRef(option_name.clone()))
                }
                Some(ExecValue::Str(s)) => Some(ExecConfigValue::Literal(s.clone())),
                _ => None,
            },
            _ => None,
        }
    }

    pub(super) fn resolve_artifact_ref(&self, node: &AstNode) -> Option<ExecArtifact> {
        match node {
            AstNode::Literal(Literal::String(s)) => Some(ExecArtifact {
                name: s.clone(),
                root_module: ExecConfigValue::Literal(String::new()),
                fol_model: BuildArtifactFolModel::Std,
                target: None,
                optimize: None,
            }),
            AstNode::Identifier { name, .. } => match self.scope.get(name.as_str()) {
                Some(ExecValue::Artifact(a)) => Some(a.clone()),
                _ => None,
            },
            _ => None,
        }
    }

    pub(super) fn resolve_step_ref(&self, node: &AstNode) -> Option<String> {
        match node {
            AstNode::Literal(Literal::String(s)) => Some(s.clone()),
            AstNode::Identifier { name, .. } => match self.scope.get(name.as_str()) {
                Some(ExecValue::Step { name }) => Some(name.clone()),
                Some(ExecValue::Run { name }) => Some(name.clone()),
                Some(ExecValue::Install { name }) => Some(name.clone()),
                _ => None,
            },
            _ => None,
        }
    }

    pub(super) fn resolve_dependency_args(
        &self,
        fields: &[RecordInitField],
    ) -> Result<
        Option<BTreeMap<String, crate::api::DependencyArgValue>>,
        crate::eval::BuildEvaluationError,
    > {
        let Some(args_field) = fields.iter().find(|field| field.name == "args") else {
            return Ok(None);
        };
        let AstNode::RecordInit {
            fields: arg_fields, ..
        } = &args_field.value
        else {
            return Err(crate::eval::BuildEvaluationError::new(
                crate::eval::BuildEvaluationErrorKind::InvalidInput,
                "build.add_dep config is invalid: dependency 'args' must be a record".to_string(),
            ));
        };
        let mut args = BTreeMap::new();
        for field in arg_fields {
            let value = match &field.value {
                AstNode::Literal(Literal::Boolean(value)) => {
                    crate::api::DependencyArgValue::Bool(*value)
                }
                AstNode::Literal(Literal::Integer(value)) => {
                    crate::api::DependencyArgValue::Int(*value)
                }
                AstNode::Literal(Literal::String(value)) => {
                    crate::api::DependencyArgValue::String(value.clone())
                }
                AstNode::Identifier { name, .. } => match self.scope.get(name.as_str()) {
                    Some(ExecValue::Target(option_name))
                    | Some(ExecValue::Optimize(option_name))
                    | Some(ExecValue::OptionRef(option_name)) => {
                        crate::api::DependencyArgValue::OptionRef(option_name.clone())
                    }
                    Some(ExecValue::Str(value)) => {
                        crate::api::DependencyArgValue::String(value.clone())
                    }
                    _ => {
                        return Err(crate::eval::BuildEvaluationError::new(
                            crate::eval::BuildEvaluationErrorKind::InvalidInput,
                            format!(
                                "build.add_dep config is invalid: dependency arg '{}' must be bool, int, str, or an option handle",
                                field.name
                            ),
                        ))
                    }
                },
                _ => {
                    return Err(crate::eval::BuildEvaluationError::new(
                        crate::eval::BuildEvaluationErrorKind::InvalidInput,
                        format!(
                            "build.add_dep config is invalid: dependency arg '{}' must be bool, int, str, or an option handle",
                            field.name
                        ),
                    ))
                }
            };
            args.insert(field.name.clone(), value);
        }
        Ok(Some(args))
    }

    pub(super) fn resolve_field_string_list(
        &self,
        fields: &[RecordInitField],
        field_name: &str,
    ) -> Result<Vec<String>, BuildEvaluationError> {
        let Some(field) = fields.iter().find(|field| field.name == field_name) else {
            return Ok(Vec::new());
        };
        let items = self.eval_iterable(&field.value)?;
        let mut resolved = Vec::with_capacity(items.len());
        for item in items {
            match item {
                ExecValue::Str(value) => resolved.push(value),
                _ => {
                    return Err(BuildEvaluationError::new(
                        BuildEvaluationErrorKind::InvalidInput,
                        format!(
                        "build config is invalid: '{field_name}' must contain only string values"
                    ),
                    ))
                }
            }
        }
        Ok(resolved)
    }

    pub(super) fn resolve_field_path_list(
        &self,
        fields: &[RecordInitField],
        field_name: &str,
    ) -> Result<Vec<String>, BuildEvaluationError> {
        let Some(field) = fields.iter().find(|field| field.name == field_name) else {
            return Ok(Vec::new());
        };
        let items = self.eval_iterable(&field.value)?;
        let mut resolved = Vec::with_capacity(items.len());
        for item in items {
            match item {
                ExecValue::SourceFile { path } | ExecValue::GeneratedFile { path, .. } => {
                    resolved.push(path)
                }
                _ => {
                    return Err(BuildEvaluationError::new(
                        BuildEvaluationErrorKind::InvalidInput,
                        format!(
                            "build config is invalid: '{field_name}' must contain only source-file or generated-output handles"
                        ),
                    ))
                }
            }
        }
        Ok(resolved)
    }

    pub(super) fn resolve_field_string_map(
        &self,
        fields: &[RecordInitField],
        field_name: &str,
    ) -> Result<BTreeMap<String, String>, BuildEvaluationError> {
        let Some(field) = fields.iter().find(|field| field.name == field_name) else {
            return Ok(BTreeMap::new());
        };
        let AstNode::RecordInit {
            fields: map_fields, ..
        } = &field.value
        else {
            return Err(BuildEvaluationError::new(
                BuildEvaluationErrorKind::InvalidInput,
                format!("build config is invalid: '{field_name}' must be a record"),
            ));
        };
        let mut resolved = BTreeMap::new();
        for map_field in map_fields {
            let Some(value) = self.resolve_string(&map_field.value) else {
                return Err(BuildEvaluationError::new(
                    BuildEvaluationErrorKind::InvalidInput,
                    format!("build config is invalid: '{field_name}' values must be strings"),
                ));
            };
            resolved.insert(map_field.name.clone(), value);
        }
        Ok(resolved)
    }
}
