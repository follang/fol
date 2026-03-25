use fol_parser::ast::{AstNode, Literal, RecordInitField};
use crate::artifact::BuildArtifactFolModel;
use std::collections::BTreeMap;

use super::core::BuildBodyExecutor;
use super::types::{ExecArtifact, ExecConfigValue, ExecValue};

impl BuildBodyExecutor {
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

    pub(super) fn resolve_field_string(&self, fields: &[RecordInitField], field_name: &str) -> Option<String> {
        fields
            .iter()
            .find(|f| f.name == field_name)
            .and_then(|f| self.resolve_string(&f.value))
    }

    pub(super) fn parse_config_value(&self, node: &AstNode, _allowed_kinds: &[&str]) -> Option<ExecConfigValue> {
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
    ) -> Option<BTreeMap<String, crate::api::DependencyArgValue>> {
        let args_field = fields.iter().find(|field| field.name == "args")?;
        let AstNode::RecordInit { fields: arg_fields, .. } = &args_field.value else {
            return None;
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
                    _ => return None,
                },
                _ => return None,
            };
            args.insert(field.name.clone(), value);
        }
        Some(args)
    }
}
