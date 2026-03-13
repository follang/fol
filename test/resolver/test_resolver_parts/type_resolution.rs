use super::{resolve_package_from_folder, try_resolve_package_from_folder, unique_temp_root};
use fol_resolver::{ReferenceKind, ResolverErrorKind, ScopeKind, SymbolKind};
use std::fs;

#[test]
fn test_resolver_resolves_named_types_against_top_level_type_symbols() {
    let temp_root = unique_temp_root("type_resolution_named");
    fs::create_dir_all(&temp_root).expect("Should create a temporary resolver fixture directory");
    fs::write(
        temp_root.join("main.fol"),
        "typ Item: int;\nali NamedItem: Item;\nfun[] main(value: Item): NamedItem = {\n    var local: Item = value;\n    return local;\n}\n",
    )
    .expect("Should write the named-type resolver fixture");

    let resolved = resolve_package_from_folder(
        temp_root
            .to_str()
            .expect("Temporary resolver fixture path should be valid UTF-8"),
    );
    let source_unit_scope = resolved
        .source_units
        .iter()
        .next()
        .expect("Resolver should keep the source unit")
        .scope_id;
    let item_symbol = resolved
        .symbols_in_scope(resolved.program_scope)
        .into_iter()
        .find(|symbol| symbol.name == "Item" && symbol.kind == SymbolKind::Type)
        .expect("Program scope should keep the Item type symbol");
    let named_item_symbol = resolved
        .symbols_in_scope(resolved.program_scope)
        .into_iter()
        .find(|symbol| symbol.name == "NamedItem" && symbol.kind == SymbolKind::Alias)
        .expect("Program scope should keep the NamedItem alias symbol");
    let routine_scope_id = resolved
        .scopes
        .iter_with_ids()
        .find_map(|(scope_id, scope)| matches!(scope.kind, ScopeKind::Routine).then_some(scope_id))
        .expect("Resolver should create a routine scope");

    assert!(
        resolved
            .references_in_scope(source_unit_scope)
            .into_iter()
            .any(|reference| {
                reference.kind == ReferenceKind::TypeName
                    && reference.name == "Item"
                    && reference.resolved == Some(item_symbol.id)
            }),
        "Alias declarations should record named type references in the enclosing scope"
    );
    assert!(
        resolved
            .references_in_scope(routine_scope_id)
            .into_iter()
            .any(|reference| {
                reference.kind == ReferenceKind::TypeName
                    && reference.name == "NamedItem"
                    && reference.resolved == Some(named_item_symbol.id)
            }),
        "Routine signatures should resolve alias return types as named type references"
    );
    assert!(
        resolved
            .references_in_scope(routine_scope_id)
            .into_iter()
            .filter(|reference| {
                reference.kind == ReferenceKind::TypeName
                    && reference.name == "Item"
                    && reference.resolved == Some(item_symbol.id)
            })
            .count()
            >= 2,
        "Routine signatures and local type hints should both resolve named Item references"
    );

    fs::remove_dir_all(&temp_root)
        .expect("Temporary resolver fixture directory should be removable after the test");
}

#[test]
fn test_resolver_prefers_generic_parameters_over_outer_type_symbols() {
    let temp_root = unique_temp_root("type_resolution_generics");
    fs::create_dir_all(&temp_root).expect("Should create a temporary resolver fixture directory");
    fs::write(
        temp_root.join("main.fol"),
        "typ T: int;\nfun identity(T)(value: T): T = {\n    var copy: T = value;\n    return copy;\n}\n",
    )
    .expect("Should write the generic type resolver fixture");

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
    let generic_symbol = resolved
        .symbols_in_scope(routine_scope_id)
        .into_iter()
        .find(|symbol| symbol.name == "T" && symbol.kind == SymbolKind::GenericParameter)
        .expect("Routine scope should keep the generic parameter symbol");

    assert!(
        resolved
            .references_in_scope(routine_scope_id)
            .into_iter()
            .filter(|reference| {
                reference.kind == ReferenceKind::TypeName
                    && reference.name == "T"
                    && reference.resolved == Some(generic_symbol.id)
            })
            .count()
            >= 3,
        "Routine type references should prefer the generic parameter over outer type symbols"
    );

    fs::remove_dir_all(&temp_root)
        .expect("Temporary resolver fixture directory should be removable after the test");
}

#[test]
fn test_resolver_reports_unresolved_named_types() {
    let temp_root = unique_temp_root("type_resolution_missing");
    fs::create_dir_all(&temp_root).expect("Should create a temporary resolver fixture directory");
    fs::write(
        temp_root.join("main.fol"),
        "fun[] main(value: Missing): int = {\n    return 0;\n}\n",
    )
    .expect("Should write the unresolved type fixture");

    let errors = try_resolve_package_from_folder(
        temp_root
            .to_str()
            .expect("Temporary resolver fixture path should be valid UTF-8"),
    )
    .expect_err("Resolver should reject unresolved named types");

    assert!(
        errors.iter().any(|error| {
            error.kind() == ResolverErrorKind::UnresolvedName
                && error
                    .to_string()
                    .contains("could not resolve type 'Missing'")
        }),
        "Resolver should report unresolved named type references explicitly"
    );

    fs::remove_dir_all(&temp_root)
        .expect("Temporary resolver fixture directory should be removable after the test");
}

#[test]
fn test_resolver_treats_str_as_a_builtin_type() {
    let temp_root = unique_temp_root("type_resolution_builtin_str");
    fs::create_dir_all(&temp_root).expect("Should create a temporary resolver fixture directory");
    fs::write(
        temp_root.join("main.fol"),
        "fun[] main(path: str): str = {\n    var local: str = path;\n    return local;\n}\n",
    )
    .expect("Should write the builtin str resolver fixture");

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

    assert!(
        resolved
            .references_in_scope(routine_scope_id)
            .into_iter()
            .all(|reference| {
                !(reference.kind == ReferenceKind::TypeName && reference.name == "str")
            }),
        "Builtin str should not enter named-type resolution as a user-defined symbol"
    );

    fs::remove_dir_all(&temp_root)
        .expect("Temporary resolver fixture directory should be removable after the test");
}
