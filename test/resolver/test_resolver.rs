use fol_lexer::lexer::stage3::Elements;
use fol_parser::ast::AstParser;
use fol_resolver::{resolve_package, ResolvedProgram};
use fol_stream::FileStream;

fn resolve_package_from_file(path: &str) -> ResolvedProgram {
    let mut file_stream = FileStream::from_file(path).expect("Should read resolver test file");
    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let parsed = parser
        .parse_package(&mut lexer)
        .expect("Resolver test fixture should parse as a package");

    resolve_package(parsed).expect("Resolver foundation should lower parsed packages")
}

#[cfg(test)]
#[path = "test_resolver_parts/foundation.rs"]
mod foundation;
