use super::{resolve_package_from_folder, try_resolve_package_from_folder, unique_temp_root};
use fol_resolver::{ResolverErrorKind, ScopeKind, SymbolKind};
use std::fs;

#[test]
fn test_resolver_binds_iteration_loop_names_inside_loop_scope() {
    let temp_root = unique_temp_root("loop_binder_visible");
    fs::create_dir_all(&temp_root).expect("Should create a temporary resolver fixture directory");
    fs::write(
        temp_root.join("main.fol"),
        "fun[] main(items: seq[int], limit: int): int = {\n    loop(item in items when item < limit) {\n        return item;\n    }\n    return limit;\n};\n",
    )
    .expect("Should write the loop-binder resolver fixture");

    let resolved = resolve_package_from_folder(
        temp_root
            .to_str()
            .expect("Temporary resolver fixture path should be valid UTF-8"),
    );
    let loop_scope_id = resolved
        .scopes
        .iter_with_ids()
        .find_map(|(scope_id, scope)| {
            matches!(scope.kind, ScopeKind::LoopBinder).then_some(scope_id)
        })
        .expect("Resolver should create a loop-binder scope");

    assert!(
        resolved
            .symbols_in_scope(loop_scope_id)
            .into_iter()
            .any(|symbol| symbol.name == "item" && symbol.kind == SymbolKind::LoopBinder),
        "Loop-binder scope should contain the iteration binder symbol"
    );
    assert!(
        resolved
            .references_in_scope(loop_scope_id)
            .into_iter()
            .filter(|reference| reference.name == "item")
            .count()
            >= 2,
        "Loop-binder scope should resolve binder references in both the guard and the body"
    );

    fs::remove_dir_all(&temp_root)
        .expect("Temporary resolver fixture directory should be removable after the test");
}

#[test]
fn test_resolver_rejects_iteration_loop_binders_outside_the_loop() {
    let temp_root = unique_temp_root("loop_binder_invisible");
    fs::create_dir_all(&temp_root).expect("Should create a temporary resolver fixture directory");
    fs::write(
        temp_root.join("main.fol"),
        "fun[] main(items: seq[int]): int = {\n    loop(item in items) {\n        return item;\n    }\n    return item;\n};\n",
    )
    .expect("Should write the loop-binder visibility fixture");

    let errors = try_resolve_package_from_folder(
        temp_root
            .to_str()
            .expect("Temporary resolver fixture path should be valid UTF-8"),
    )
    .expect_err("Resolver should reject iteration binders outside their loop scope");

    assert!(
        errors
            .iter()
            .any(|error| error.kind() == ResolverErrorKind::UnresolvedName),
        "Resolver should report unresolved-name errors when a loop binder escapes its scope"
    );

    fs::remove_dir_all(&temp_root)
        .expect("Temporary resolver fixture directory should be removable after the test");
}
