use fol_intrinsics::{intrinsic_registry, IntrinsicStatus, IntrinsicSurface};
use crate::TypecheckCapabilityModel;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EditorIntrinsicInfo {
    pub name: &'static str,
    pub surface: IntrinsicSurface,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorTypeFamily {
    Scalar,
    Array,
    RecordLike,
    OptionalShell,
    ErrorShell,
    String,
    Vector,
    Sequence,
    Set,
    Map,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EditorModelCapability {
    pub heap: bool,
    pub hosted_runtime: bool,
}

pub fn editor_declaration_keywords() -> &'static [&'static str] {
    fol_lexer::token::buildin::DECLARATION_KEYWORDS
}

pub fn editor_builtin_type_names() -> &'static [&'static str] {
    crate::BuiltinType::ALL_NAMES
}

pub fn editor_container_type_names() -> &'static [&'static str] {
    fol_parser::CONTAINER_TYPE_NAMES
}

pub fn editor_shell_type_names() -> &'static [&'static str] {
    fol_parser::SHELL_TYPE_NAMES
}

pub fn editor_source_kind_names() -> &'static [&'static str] {
    fol_parser::SOURCE_KIND_NAMES
}

pub fn editor_implemented_intrinsics() -> Vec<EditorIntrinsicInfo> {
    intrinsic_registry()
        .iter()
        .filter(|entry| entry.status == IntrinsicStatus::Implemented)
        .map(|entry| EditorIntrinsicInfo {
            name: entry.name,
            surface: entry.surface,
        })
        .collect()
}

pub fn editor_model_capability(model: TypecheckCapabilityModel) -> EditorModelCapability {
    match model {
        TypecheckCapabilityModel::Core => EditorModelCapability {
            heap: false,
            hosted_runtime: false,
        },
        TypecheckCapabilityModel::Alloc => EditorModelCapability {
            heap: true,
            hosted_runtime: false,
        },
        TypecheckCapabilityModel::Std => EditorModelCapability {
            heap: true,
            hosted_runtime: true,
        },
    }
}

pub fn editor_type_family_available_in_model(
    model: TypecheckCapabilityModel,
    family: EditorTypeFamily,
) -> bool {
    match family {
        EditorTypeFamily::Scalar
        | EditorTypeFamily::Array
        | EditorTypeFamily::RecordLike
        | EditorTypeFamily::OptionalShell
        | EditorTypeFamily::ErrorShell => true,
        EditorTypeFamily::String
        | EditorTypeFamily::Vector
        | EditorTypeFamily::Sequence
        | EditorTypeFamily::Set
        | EditorTypeFamily::Map => editor_model_capability(model).heap,
    }
}

pub fn editor_intrinsic_available_in_model(
    model: TypecheckCapabilityModel,
    intrinsic: EditorIntrinsicInfo,
) -> bool {
    if intrinsic.name == "echo" {
        return editor_model_capability(model).hosted_runtime;
    }
    true
}

#[cfg(test)]
mod tests {
    use super::{
        editor_builtin_type_names, editor_container_type_names, editor_declaration_keywords,
        editor_implemented_intrinsics, editor_model_capability, editor_shell_type_names,
        editor_source_kind_names, editor_type_family_available_in_model, EditorTypeFamily,
    };
    use crate::TypecheckCapabilityModel;

    #[test]
    fn editor_metadata_api_exposes_nonempty_language_facts() {
        assert!(!editor_declaration_keywords().is_empty());
        assert!(!editor_builtin_type_names().is_empty());
        assert!(!editor_container_type_names().is_empty());
        assert!(!editor_shell_type_names().is_empty());
        assert!(!editor_source_kind_names().is_empty());
        assert!(!editor_implemented_intrinsics().is_empty());
    }

    #[test]
    fn editor_model_capabilities_follow_core_alloc_std_shape() {
        assert_eq!(
            editor_model_capability(TypecheckCapabilityModel::Core),
            super::EditorModelCapability {
                heap: false,
                hosted_runtime: false,
            }
        );
        assert_eq!(
            editor_model_capability(TypecheckCapabilityModel::Alloc),
            super::EditorModelCapability {
                heap: true,
                hosted_runtime: false,
            }
        );
        assert_eq!(
            editor_model_capability(TypecheckCapabilityModel::Std),
            super::EditorModelCapability {
                heap: true,
                hosted_runtime: true,
            }
        );
        assert!(!editor_type_family_available_in_model(
            TypecheckCapabilityModel::Core,
            EditorTypeFamily::String
        ));
        assert!(editor_type_family_available_in_model(
            TypecheckCapabilityModel::Alloc,
            EditorTypeFamily::String
        ));
    }
}
