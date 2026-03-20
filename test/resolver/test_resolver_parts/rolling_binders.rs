use super::{resolve_package_from_folder, unique_temp_root};
use fol_resolver::{ScopeKind, SymbolKind};
use std::fs;

#[test]
fn test_resolver_binds_rolling_names_inside_comprehension_scope() {
    let temp_root = unique_temp_root("rolling_binder_visible");
    fs::create_dir_all(&temp_root).expect("Should create a temporary resolver fixture directory");
    fs::write(
        temp_root.join("main.fol"),
        "fun[] main(items: seq[int]): seq[int] = {\n    return { item for item in items if item };\n};\n",
    )
    .expect("Should write the rolling-binder resolver fixture");

    let resolved = resolve_package_from_folder(
        temp_root
            .to_str()
            .expect("Temporary resolver fixture path should be valid UTF-8"),
    );
    let rolling_scope_id = resolved
        .scopes
        .iter_with_ids()
        .find_map(|(scope_id, scope)| {
            matches!(scope.kind, ScopeKind::RollingBinder).then_some(scope_id)
        })
        .expect("Resolver should create a rolling-binder scope");

    assert!(
        resolved
            .symbols_in_scope(rolling_scope_id)
            .into_iter()
            .any(|symbol| symbol.name == "item" && symbol.kind == SymbolKind::RollingBinder),
        "Rolling-binder scope should contain the comprehension binding"
    );
    assert!(
        resolved
            .references_in_scope(rolling_scope_id)
            .into_iter()
            .filter(|reference| reference.name == "item")
            .count()
            >= 2,
        "Rolling-binder scope should resolve binder references in both the element and the filter"
    );

    fs::remove_dir_all(&temp_root)
        .expect("Temporary resolver fixture directory should be removable after the test");
}

#[test]
fn test_resolver_allows_rolling_binders_to_shadow_outer_locals() {
    let temp_root = unique_temp_root("rolling_binder_shadowing");
    fs::create_dir_all(&temp_root).expect("Should create a temporary resolver fixture directory");
    fs::write(
        temp_root.join("main.fol"),
        "fun[] main(items: seq[int]): seq[int] = {\n    var item = 1;\n    return { item for item in items if item };\n};\n",
    )
    .expect("Should write the rolling shadowing fixture");

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
    let rolling_scope_id = resolved
        .scopes
        .iter_with_ids()
        .find_map(|(scope_id, scope)| {
            matches!(scope.kind, ScopeKind::RollingBinder).then_some(scope_id)
        })
        .expect("Resolver should create a rolling-binder scope");
    let outer_symbol = resolved
        .symbols_in_scope(routine_scope_id)
        .into_iter()
        .find(|symbol| symbol.name == "item" && symbol.kind == SymbolKind::ValueBinding)
        .expect("Routine scope should keep the outer local binding");
    let rolling_symbol = resolved
        .symbols_in_scope(rolling_scope_id)
        .into_iter()
        .find(|symbol| symbol.name == "item" && symbol.kind == SymbolKind::RollingBinder)
        .expect("Rolling scope should keep the shadowing comprehension binding");
    let rolling_reference = resolved
        .references_in_scope(rolling_scope_id)
        .into_iter()
        .find(|reference| reference.name == "item")
        .expect("Rolling scope should record a reference to the comprehension binding");

    assert_ne!(outer_symbol.id, rolling_symbol.id);
    assert_eq!(
        rolling_reference.resolved,
        Some(rolling_symbol.id),
        "Comprehension references should resolve to the rolling binder instead of the outer local"
    );

    fs::remove_dir_all(&temp_root)
        .expect("Temporary resolver fixture directory should be removable after the test");
}
