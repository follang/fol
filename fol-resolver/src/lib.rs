pub mod collect;
pub mod errors;
pub mod ids;
pub mod imports;
pub mod model;
pub mod traverse;

pub use errors::{ResolverError, ResolverErrorKind};
pub use ids::{IdTable, ImportId, ReferenceId, ScopeId, SourceUnitId, SymbolId};
pub use model::{
    ReferenceKind, ResolvedImport, ResolvedProgram, ResolvedReference, ResolvedScope,
    ResolvedSourceUnit, ResolvedSymbol, ScopeKind, SymbolKind,
};

pub type ResolverResult<T> = Result<T, Vec<ResolverError>>;

#[derive(Debug, Default)]
pub struct Resolver;

impl Resolver {
    pub fn new() -> Self {
        Self
    }

    pub fn resolve_package(
        &mut self,
        syntax: fol_parser::ast::ParsedPackage,
    ) -> ResolverResult<ResolvedProgram> {
        let mut program = ResolvedProgram::new(syntax);
        collect::collect_top_level_symbols(&mut program)?;
        traverse::collect_routine_scopes(&mut program)?;
        imports::validate_supported_import_kinds(&program)?;
        Ok(program)
    }
}

pub fn resolve_package(syntax: fol_parser::ast::ParsedPackage) -> ResolverResult<ResolvedProgram> {
    Resolver::new().resolve_package(syntax)
}

#[cfg(test)]
mod tests {
    use super::resolve_package;
    use fol_parser::ast::{AstParser, ParsedPackage};
    use fol_stream::FileStream;

    fn parse_package(path: &str) -> ParsedPackage {
        let mut stream = FileStream::from_file(path).expect("Should open resolver smoke fixture");
        let mut lexer = fol_lexer::lexer::stage3::Elements::init(&mut stream);
        let mut parser = AstParser::new();
        parser
            .parse_package(&mut lexer)
            .expect("Resolver smoke fixture should parse as a package")
    }

    #[test]
    fn resolver_smoke_can_lower_a_parsed_package() {
        let resolved = resolve_package(parse_package("../test/parser/simple_var.fol"))
            .expect("Resolver foundation should lower parsed packages");

        assert_eq!(resolved.package_name(), "parser");
        assert_eq!(resolved.source_units.len(), 1);
        assert_eq!(resolved.syntax().source_units.len(), 1);
    }
}
