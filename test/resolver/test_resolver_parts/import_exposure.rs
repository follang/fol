use super::{resolve_package_from_folder, unique_temp_root};
use fol_resolver::{ReferenceKind, ScopeKind, SymbolKind};
use std::fs;

#[test]
fn test_resolver_resolves_plain_identifiers_against_imported_exported_values() {
    let temp_root = unique_temp_root("import_exposure_value");
    fs::create_dir_all(temp_root.join("math"))
        .expect("Should create a temporary resolver fixture directory");
    fs::write(temp_root.join("math/values.fol"), "var[exp] answer: int = 42;\n")
        .expect("Should write the imported exported value fixture");
    fs::write(
        temp_root.join("main.fol"),
        "use math: loc = {math};\nfun[] main(): int = {\n    return answer;\n}\n",
    )
    .expect("Should write the plain imported-value lookup fixture");

    let resolved = resolve_package_from_folder(
        temp_root
            .to_str()
            .expect("Temporary resolver fixture path should be valid UTF-8"),
    );
    let import = resolved
        .imports_in_scope(resolved.program_scope)
        .into_iter()
        .find(|import| import.alias_name == "math")
        .expect("Program scope should keep the imported namespace alias");
    let answer_symbol = resolved
        .symbols_in_scope(import.target_scope.expect("Import target should resolve"))
        .into_iter()
        .find(|symbol| symbol.name == "answer" && symbol.kind == SymbolKind::ValueBinding)
        .expect("Imported namespace scope should keep the exported value binding");
    let routine_scope_id = resolved
        .scopes
        .iter_with_ids()
        .find_map(|(scope_id, scope)| matches!(scope.kind, ScopeKind::Routine).then_some(scope_id))
        .expect("Resolver should create a routine scope");
    let answer_reference = resolved
        .references_in_scope(routine_scope_id)
        .into_iter()
        .find(|reference| {
            reference.kind == ReferenceKind::Identifier && reference.name == "answer"
        })
        .expect("Routine scope should record the imported plain identifier reference");

    assert_eq!(
        answer_reference.resolved,
        Some(answer_symbol.id),
        "Plain identifiers should resolve against imported exported value members"
    );

    fs::remove_dir_all(&temp_root)
        .expect("Temporary resolver fixture directory should be removable after the test");
}
