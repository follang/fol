use super::{resolve_package_from_folder, try_resolve_package_from_folder, unique_temp_root};
use fol_resolver::{ResolverErrorKind, ScopeKind, SymbolKind};
use std::fs;

#[test]
fn test_resolver_resolves_plain_identifiers_against_outer_local_scopes() {
    let temp_root = unique_temp_root("identifier_resolution_outer");
    fs::create_dir_all(&temp_root).expect("Should create a temporary resolver fixture directory");
    fs::write(
        temp_root.join("main.fol"),
        "fun[] main(): int = {\n    var outer = 1;\n    fun inner(): int = {\n        return outer;\n    }\n    return 0;\n}\n",
    )
    .expect("Should write the outer-binding resolver fixture");

    let resolved = resolve_package_from_folder(
        temp_root
            .to_str()
            .expect("Temporary resolver fixture path should be valid UTF-8"),
    );
    let outer_scope_id = resolved
        .scopes
        .iter_with_ids()
        .find_map(|(scope_id, scope)| {
            (matches!(scope.kind, ScopeKind::Routine)
                && scope.parent == Some(resolved.program_scope))
            .then_some(scope_id)
        })
        .expect("Resolver should create an outer routine scope");
    let inner_scope_id = resolved
        .scopes
        .iter_with_ids()
        .find_map(|(scope_id, scope)| {
            (matches!(scope.kind, ScopeKind::Routine) && scope.parent == Some(outer_scope_id))
                .then_some(scope_id)
        })
        .expect("Resolver should create a nested routine scope");
    let outer_binding = resolved
        .symbols_in_scope(outer_scope_id)
        .into_iter()
        .find(|symbol| symbol.name == "outer" && symbol.kind == SymbolKind::ValueBinding)
        .expect("Outer routine scope should keep the local binding");
    let outer_reference = resolved
        .references_in_scope(inner_scope_id)
        .into_iter()
        .find(|reference| reference.name == "outer")
        .expect("Nested routine scope should keep the outer identifier reference");

    assert_eq!(
        outer_reference.resolved,
        Some(outer_binding.id),
        "Plain identifiers should resolve through enclosing routine scopes"
    );

    fs::remove_dir_all(&temp_root)
        .expect("Temporary resolver fixture directory should be removable after the test");
}

#[test]
fn test_resolver_resolves_plain_identifiers_against_import_alias_symbols() {
    let temp_root = unique_temp_root("identifier_resolution_import_alias");
    fs::create_dir_all(&temp_root).expect("Should create a temporary resolver fixture directory");
    fs::write(
        temp_root.join("main.fol"),
        "use math: loc = {core::math};\nfun[] main(): int = {\n    return math;\n}\n",
    )
    .expect("Should write the import-alias resolver fixture");

    let resolved = resolve_package_from_folder(
        temp_root
            .to_str()
            .expect("Temporary resolver fixture path should be valid UTF-8"),
    );
    let routine_scope_id = resolved
        .scopes
        .iter_with_ids()
        .find_map(|(scope_id, scope)| matches!(scope.kind, ScopeKind::Routine).then_some(scope_id))
        .expect("Resolver should create a routine scope");
    let import_symbol = resolved
        .symbols_in_scope(resolved.program_scope)
        .into_iter()
        .find(|symbol| symbol.name == "math" && symbol.kind == SymbolKind::ImportAlias)
        .expect("Top-level import aliases should be first-class symbols");
    let import_reference = resolved
        .references_in_scope(routine_scope_id)
        .into_iter()
        .find(|reference| reference.name == "math")
        .expect("Routine scope should record the import alias reference");

    assert_eq!(
        import_reference.resolved,
        Some(import_symbol.id),
        "Plain identifiers should resolve against visible import aliases"
    );

    fs::remove_dir_all(&temp_root)
        .expect("Temporary resolver fixture directory should be removable after the test");
}

#[test]
fn test_resolver_reports_unresolved_plain_identifier_names() {
    let temp_root = unique_temp_root("identifier_resolution_unresolved");
    fs::create_dir_all(&temp_root).expect("Should create a temporary resolver fixture directory");
    fs::write(
        temp_root.join("main.fol"),
        "fun[] main(): int = {\n    return missing;\n}\n",
    )
    .expect("Should write the unresolved-name resolver fixture");

    let errors = try_resolve_package_from_folder(
        temp_root
            .to_str()
            .expect("Temporary resolver fixture path should be valid UTF-8"),
    )
    .expect_err("Resolver should reject unresolved plain identifiers");

    assert!(
        errors
            .iter()
            .any(|error| error.kind() == ResolverErrorKind::UnresolvedName),
        "Resolver should report unresolved-name errors for plain identifier lookup"
    );

    fs::remove_dir_all(&temp_root)
        .expect("Temporary resolver fixture directory should be removable after the test");
}
