use fol_lexer::lexer::stage3::Elements;
use fol_parser::ast::AstParser;
use fol_resolver::{resolve_package, ResolvedProgram, ResolverError};
use fol_stream::FileStream;
use std::time::{SystemTime, UNIX_EPOCH};

fn resolve_package_from_file(path: &str) -> ResolvedProgram {
    let mut file_stream = FileStream::from_file(path).expect("Should read resolver test file");
    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let parsed = parser
        .parse_package(&mut lexer)
        .expect("Resolver test fixture should parse as a package");

    resolve_package(parsed).expect("Resolver foundation should lower parsed packages")
}

fn resolve_package_from_folder(path: &str) -> ResolvedProgram {
    try_resolve_package_from_folder(path)
        .expect("Resolver test fixture should resolve successfully")
}

fn try_resolve_package_from_folder(path: &str) -> Result<ResolvedProgram, Vec<ResolverError>> {
    let mut file_stream = FileStream::from_folder(path).expect("Should read resolver test folder");
    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let parsed = parser
        .parse_package(&mut lexer)
        .expect("Resolver test fixture should parse as a package");

    resolve_package(parsed)
}

fn unique_temp_root(label: &str) -> std::path::PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("System time should be after unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "fol_resolver_{}_{}_{}",
        label,
        std::process::id(),
        stamp
    ))
}

#[cfg(test)]
#[path = "test_resolver_parts/block_scopes.rs"]
mod block_scopes;
#[cfg(test)]
#[path = "test_resolver_parts/foundation.rs"]
mod foundation;
#[cfg(test)]
#[path = "test_resolver_parts/routine_scopes.rs"]
mod routine_scopes;
#[cfg(test)]
#[path = "test_resolver_parts/source_units.rs"]
mod source_units;
#[cfg(test)]
#[path = "test_resolver_parts/top_level_collection.rs"]
mod top_level_collection;
#[cfg(test)]
#[path = "test_resolver_parts/top_level_duplicates.rs"]
mod top_level_duplicates;
