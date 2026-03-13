use super::{resolve_package_from_folder, try_resolve_package_from_folder, unique_temp_root};
use fol_resolver::{ReferenceKind, ResolverErrorKind, ScopeKind, SymbolKind};
use std::fs;

#[test]
fn test_resolver_resolves_plain_free_calls_against_visible_routines() {
    let temp_root = unique_temp_root("free_call_visible_routine");
    fs::create_dir_all(&temp_root).expect("Should create a temporary resolver fixture directory");
    fs::write(
        temp_root.join("main.fol"),
        "fun[] helper(value: int): int = {\n    return value;\n}\n\nfun[] main(input: int): int = {\n    return helper(input);\n}\n",
    )
    .expect("Should write the visible free-call resolver fixture");

    let resolved = resolve_package_from_folder(
        temp_root
            .to_str()
            .expect("Temporary resolver fixture path should be valid UTF-8"),
    );
    let helper_symbol = resolved
        .symbols_in_scope(resolved.program_scope)
        .into_iter()
        .find(|symbol| symbol.name == "helper" && symbol.kind == SymbolKind::Routine)
        .expect("Program scope should keep the helper routine symbol");
    let main_scope_id = resolved
        .scopes
        .iter_with_ids()
        .find_map(|(scope_id, scope)| {
            (matches!(scope.kind, ScopeKind::Routine)
                && resolved
                    .symbols_in_scope(scope_id)
                    .into_iter()
                    .any(|symbol| symbol.name == "input" && symbol.kind == SymbolKind::Parameter))
            .then_some(scope_id)
        })
        .expect("Resolver should create a routine scope for main");
    let helper_reference = resolved
        .references_in_scope(main_scope_id)
        .into_iter()
        .find(|reference| {
            reference.kind == ReferenceKind::FunctionCall && reference.name == "helper"
        })
        .expect("Routine scope should record a direct free-call callee reference");

    assert_eq!(
        helper_reference.resolved,
        Some(helper_symbol.id),
        "Plain free calls should resolve to visible routine symbols"
    );

    fs::remove_dir_all(&temp_root)
        .expect("Temporary resolver fixture directory should be removable after the test");
}

#[test]
fn test_resolver_reports_unresolved_plain_free_calls() {
    let temp_root = unique_temp_root("free_call_missing_routine");
    fs::create_dir_all(&temp_root).expect("Should create a temporary resolver fixture directory");
    fs::write(
        temp_root.join("main.fol"),
        "fun[] main(input: int): int = {\n    return helper(input);\n}\n",
    )
    .expect("Should write the missing free-call resolver fixture");

    let errors = try_resolve_package_from_folder(
        temp_root
            .to_str()
            .expect("Temporary resolver fixture path should be valid UTF-8"),
    )
    .expect_err("Resolver should reject unresolved plain free calls");
    let error = errors
        .iter()
        .find(|error| error.kind() == ResolverErrorKind::UnresolvedName)
        .expect("Resolver should report unresolved-name errors for missing free-call callees");
    let origin = error
        .origin()
        .expect("Plain unresolved free calls should keep exact syntax origins");

    assert!(
        error
            .to_string()
            .contains("could not resolve callable routine 'helper'"),
        "Resolver should report unresolved-name errors for missing free-call callees"
    );
    assert_eq!(origin.line, 2);
    assert_eq!(origin.column, 12);
    assert_eq!(
        origin.file.as_deref(),
        Some(
            temp_root
                .join("main.fol")
                .to_str()
                .expect("Temporary resolver fixture path should be valid UTF-8")
        ),
        "Plain unresolved free calls should retain their exact source file"
    );

    fs::remove_dir_all(&temp_root)
        .expect("Temporary resolver fixture directory should be removable after the test");
}
