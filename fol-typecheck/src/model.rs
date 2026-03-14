use crate::{BuiltinTypeIds, TypeTable};

#[derive(Debug, Clone, PartialEq)]
pub struct TypedProgram {
    resolved: fol_resolver::ResolvedProgram,
    type_table: TypeTable,
    builtins: BuiltinTypeIds,
}

impl TypedProgram {
    pub fn from_resolved(resolved: fol_resolver::ResolvedProgram) -> Self {
        let mut type_table = TypeTable::new();
        let builtins = BuiltinTypeIds::install(&mut type_table);

        Self {
            resolved,
            type_table,
            builtins,
        }
    }

    pub fn package_name(&self) -> &str {
        self.resolved.package_name()
    }

    pub fn resolved(&self) -> &fol_resolver::ResolvedProgram {
        &self.resolved
    }

    pub fn type_table(&self) -> &TypeTable {
        &self.type_table
    }

    pub fn builtin_types(&self) -> BuiltinTypeIds {
        self.builtins
    }
}

#[cfg(test)]
mod tests {
    use super::TypedProgram;
    use crate::{BuiltinType, CheckedType};
    use fol_parser::ast::AstParser;
    use fol_resolver::resolve_package;
    use fol_stream::FileStream;

    #[test]
    fn typed_program_shell_installs_builtin_types_for_resolved_programs() {
        let fixture_path = concat!(env!("CARGO_MANIFEST_DIR"), "/../test/parser/simple_var.fol");
        let mut stream = FileStream::from_file(fixture_path).expect("Should open typecheck fixture");
        let mut lexer = fol_lexer::lexer::stage3::Elements::init(&mut stream);
        let mut parser = AstParser::new();
        let syntax = parser
            .parse_package(&mut lexer)
            .expect("Typecheck fixture should parse");
        let resolved = resolve_package(syntax).expect("Typecheck fixture should resolve");

        let typed = TypedProgram::from_resolved(resolved);

        assert_eq!(typed.package_name(), "parser");
        assert_eq!(
            typed.type_table().get(typed.builtin_types().str_),
            Some(&CheckedType::Builtin(BuiltinType::Str))
        );
    }
}
