use fol_intrinsics::{intrinsic_registry, IntrinsicStatus, IntrinsicSurface};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EditorIntrinsicInfo {
    pub name: &'static str,
    pub surface: IntrinsicSurface,
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

#[cfg(test)]
mod tests {
    use super::{
        editor_builtin_type_names, editor_container_type_names, editor_declaration_keywords,
        editor_implemented_intrinsics, editor_shell_type_names, editor_source_kind_names,
    };

    #[test]
    fn editor_metadata_api_exposes_nonempty_language_facts() {
        assert!(!editor_declaration_keywords().is_empty());
        assert!(!editor_builtin_type_names().is_empty());
        assert!(!editor_container_type_names().is_empty());
        assert!(!editor_shell_type_names().is_empty());
        assert!(!editor_source_kind_names().is_empty());
        assert!(!editor_implemented_intrinsics().is_empty());
    }
}
