use super::{resolve_package_from_folder, unique_temp_root};
use fol_resolver::{ReferenceKind, ScopeKind, SymbolKind};
use std::fs;

#[test]
fn test_resolver_same_file_routines_can_resolve_hidden_values() {
    let temp_root = unique_temp_root("file_private_hidden_value");
    fs::create_dir_all(&temp_root).expect("Should create a temporary resolver fixture directory");
    fs::write(
        temp_root.join("main.fol"),
        "var[hid] hidden: int = 1;\nfun[] main(): int = {\n    return hidden;\n}\n",
    )
    .expect("Should write the same-file hidden value fixture");

    let resolved = resolve_package_from_folder(
        temp_root
            .to_str()
            .expect("Temporary resolver fixture path should be valid UTF-8"),
    );
    let source_unit = resolved
        .source_units
        .iter()
        .next()
        .expect("Resolver should keep the source unit");
    let hidden_symbol = resolved
        .symbols_in_scope(source_unit.scope_id)
        .into_iter()
        .find(|symbol| symbol.name == "hidden" && symbol.kind == SymbolKind::ValueBinding)
        .expect("Source-unit scope should keep the hidden value binding");
    let routine_scope_id = resolved
        .scopes
        .iter_with_ids()
        .find_map(|(scope_id, scope)| matches!(scope.kind, ScopeKind::Routine).then_some(scope_id))
        .expect("Resolver should create a routine scope");
    let hidden_reference = resolved
        .references_in_scope(routine_scope_id)
        .into_iter()
        .find(|reference| {
            reference.kind == ReferenceKind::Identifier && reference.name == "hidden"
        })
        .expect("Routine scope should record the same-file hidden identifier reference");

    assert_eq!(
        hidden_reference.resolved,
        Some(hidden_symbol.id),
        "Same-file routines should resolve hidden values through source-unit scope"
    );

    fs::remove_dir_all(&temp_root)
        .expect("Temporary resolver fixture directory should be removable after the test");
}
