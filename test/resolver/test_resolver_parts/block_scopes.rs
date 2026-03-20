use super::{resolve_package_from_folder, try_resolve_package_from_folder, unique_temp_root};
use fol_resolver::{ResolverErrorKind, ScopeKind, SymbolKind};
use std::fs;

#[test]
fn test_resolver_builds_block_scopes_and_allows_shadowing() {
    let temp_root = unique_temp_root("block_scope_shadowing");
    fs::create_dir_all(&temp_root).expect("Should create a temporary resolver fixture directory");
    fs::write(
        temp_root.join("main.fol"),
        "fun[] main(): int = {\n    var value = 1;\n    {\n        var value = 2;\n        return value;\n    };\n};\n",
    )
    .expect("Should write the block shadowing fixture");

    let resolved = resolve_package_from_folder(
        temp_root
            .to_str()
            .expect("Temporary resolver fixture path should be valid UTF-8"),
    );
    let routine_scope_id = resolved
        .scopes
        .iter_with_ids()
        .find_map(|(scope_id, scope)| matches!(scope.kind, ScopeKind::Routine).then_some(scope_id))
        .expect("Resolver should create a routine scope for the fixture");
    let block_scope_id = resolved
        .scopes
        .iter_with_ids()
        .find_map(|(scope_id, scope)| {
            (matches!(scope.kind, ScopeKind::Block) && scope.parent == Some(routine_scope_id))
                .then_some(scope_id)
        })
        .expect("Resolver should create a nested block scope for the explicit block");

    assert!(
        resolved
            .symbols_in_scope(routine_scope_id)
            .into_iter()
            .any(|symbol| symbol.name == "value" && symbol.kind == SymbolKind::ValueBinding),
        "Routine scope should keep the outer local binding"
    );
    assert!(
        resolved
            .symbols_in_scope(block_scope_id)
            .into_iter()
            .any(|symbol| symbol.name == "value" && symbol.kind == SymbolKind::ValueBinding),
        "Nested block scope should keep the shadowing local binding"
    );
    assert!(
        resolved
            .references_in_scope(block_scope_id)
            .into_iter()
            .any(|reference| reference.name == "value" && reference.resolved.is_some()),
        "Identifier references inside the nested block should resolve against the shadowing binding"
    );

    fs::remove_dir_all(&temp_root)
        .expect("Temporary resolver fixture directory should be removable after the test");
}

#[test]
fn test_resolver_rejects_same_scope_duplicate_local_bindings() {
    let temp_root = unique_temp_root("block_scope_duplicates");
    fs::create_dir_all(&temp_root).expect("Should create a temporary resolver fixture directory");
    fs::write(
        temp_root.join("main.fol"),
        "fun[] main(): int = {\n    var value = 1;\n    var value = 2;\n    return value;\n};\n",
    )
    .expect("Should write the duplicate local binding fixture");

    let errors = try_resolve_package_from_folder(
        temp_root
            .to_str()
            .expect("Temporary resolver fixture path should be valid UTF-8"),
    )
    .expect_err("Resolver should reject duplicate same-scope local bindings");

    assert!(
        errors
            .iter()
            .any(|error| error.kind() == ResolverErrorKind::DuplicateSymbol),
        "Resolver should report duplicate local symbol errors for same-scope bindings"
    );

    fs::remove_dir_all(&temp_root)
        .expect("Temporary resolver fixture directory should be removable after the test");
}

#[test]
fn test_resolver_rejects_use_before_bind_in_local_initializers() {
    let temp_root = unique_temp_root("block_scope_use_before_bind");
    fs::create_dir_all(&temp_root).expect("Should create a temporary resolver fixture directory");
    fs::write(
        temp_root.join("main.fol"),
        "fun[] main(): int = {\n    var first = second;\n    var second = 2;\n    return first;\n};\n",
    )
    .expect("Should write the use-before-bind fixture");

    let errors = try_resolve_package_from_folder(
        temp_root
            .to_str()
            .expect("Temporary resolver fixture path should be valid UTF-8"),
    )
    .expect_err("Resolver should reject local use-before-bind references");

    assert!(
        errors
            .iter()
            .any(|error| error.kind() == ResolverErrorKind::UnresolvedName),
        "Resolver should report unresolved-name errors for local use-before-bind"
    );

    fs::remove_dir_all(&temp_root)
        .expect("Temporary resolver fixture directory should be removable after the test");
}
