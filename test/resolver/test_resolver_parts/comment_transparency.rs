use super::{resolve_package_from_folder, unique_temp_root};
use fol_resolver::{ReferenceKind, ScopeKind, SymbolKind};
use std::fs;

#[test]
fn test_resolver_keeps_inline_comment_wrapped_identifiers_resolvable() {
    let temp_root = unique_temp_root("comment_wrapped_identifier");
    fs::create_dir_all(&temp_root).expect("Should create a temporary resolver fixture directory");
    fs::write(
        temp_root.join("main.fol"),
        "fun[] main(input: int): int = {\n    var value = input;\n    return\n        `[doc] wrapped value`\n        value;\n}\n",
    )
    .expect("Should write the commented identifier resolver fixture");

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
    let value_symbol = resolved
        .symbols_in_scope(routine_scope_id)
        .into_iter()
        .find(|symbol| symbol.name == "value" && symbol.kind == SymbolKind::ValueBinding)
        .expect("Routine scope should keep the local value binding");
    let value_reference = resolved
        .references_in_scope(routine_scope_id)
        .into_iter()
        .find(|reference| reference.kind == ReferenceKind::Identifier && reference.name == "value")
        .expect("Comment-wrapped identifier should still become a reference record");

    assert_eq!(
        value_reference.resolved,
        Some(value_symbol.id),
        "Inline comment wrappers should not block identifier resolution"
    );

    fs::remove_dir_all(&temp_root)
        .expect("Temporary resolver fixture directory should be removable after the test");
}

#[test]
fn test_resolver_keeps_inline_comment_wrapped_initializers_in_scope_order() {
    let temp_root = unique_temp_root("comment_wrapped_initializer");
    fs::create_dir_all(&temp_root).expect("Should create a temporary resolver fixture directory");
    fs::write(
        temp_root.join("main.fol"),
        "fun[] main(input: int): int = {\n    var outer = input;\n    var alias =\n        `[doc] wrapped initializer`\n        outer;\n    return alias;\n}\n",
    )
    .expect("Should write the commented initializer resolver fixture");

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
    let outer_symbol = resolved
        .symbols_in_scope(routine_scope_id)
        .into_iter()
        .find(|symbol| symbol.name == "outer" && symbol.kind == SymbolKind::ValueBinding)
        .expect("Routine scope should keep the outer binding");
    let outer_reference = resolved
        .references_in_scope(routine_scope_id)
        .into_iter()
        .find(|reference| reference.kind == ReferenceKind::Identifier && reference.name == "outer")
        .expect("Comment-wrapped initializer should still record its identifier reference");

    assert_eq!(
        outer_reference.resolved,
        Some(outer_symbol.id),
        "Inline comment wrappers should not disturb declaration-order lookup"
    );

    fs::remove_dir_all(&temp_root)
        .expect("Temporary resolver fixture directory should be removable after the test");
}
