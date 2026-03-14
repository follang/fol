use super::{resolve_package_from_folder, try_resolve_package_from_folder, unique_temp_root};
use fol_resolver::{ReferenceKind, ResolverErrorKind, ScopeKind, SymbolKind};
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

#[test]
fn test_resolver_same_file_routines_can_resolve_hidden_routines() {
    let temp_root = unique_temp_root("file_private_hidden_routine");
    fs::create_dir_all(&temp_root).expect("Should create a temporary resolver fixture directory");
    fs::write(
        temp_root.join("main.fol"),
        "fun[hid] helper(value: int): int = {\n    return value;\n}\nfun[] main(input: int): int = {\n    return helper(input);\n}\n",
    )
    .expect("Should write the same-file hidden routine fixture");

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
    let helper_symbol = resolved
        .symbols_in_scope(source_unit.scope_id)
        .into_iter()
        .find(|symbol| symbol.name == "helper" && symbol.kind == SymbolKind::Routine)
        .expect("Source-unit scope should keep the hidden routine symbol");
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
        .expect("Main routine scope should record the same-file hidden routine call");

    assert_eq!(
        helper_reference.resolved,
        Some(helper_symbol.id),
        "Same-file routines should resolve hidden routines through source-unit scope"
    );

    fs::remove_dir_all(&temp_root)
        .expect("Temporary resolver fixture directory should be removable after the test");
}

#[test]
fn test_resolver_sibling_files_cannot_resolve_hidden_values() {
    let temp_root = unique_temp_root("file_private_hidden_value_negative");
    fs::create_dir_all(&temp_root).expect("Should create a temporary resolver fixture directory");
    fs::write(temp_root.join("a_hidden.fol"), "var[hid] hidden: int = 1;\n")
        .expect("Should write the hidden sibling fixture");
    fs::write(
        temp_root.join("b_main.fol"),
        "fun[] main(): int = {\n    return hidden;\n}\n",
    )
    .expect("Should write the sibling lookup fixture");

    let errors = try_resolve_package_from_folder(
        temp_root
            .to_str()
            .expect("Temporary resolver fixture path should be valid UTF-8"),
    )
    .expect_err("Resolver should reject hidden values across sibling files");

    assert!(
        errors.iter().any(|error| {
            error.kind() == ResolverErrorKind::UnresolvedName
                && error.to_string().contains("could not resolve name 'hidden'")
        }),
        "Hidden values should stay file-private even inside the same package"
    );

    fs::remove_dir_all(&temp_root)
        .expect("Temporary resolver fixture directory should be removable after the test");
}

#[test]
fn test_resolver_other_namespaces_cannot_resolve_hidden_routines() {
    let temp_root = unique_temp_root("file_private_hidden_routine_namespace_negative");
    fs::create_dir_all(temp_root.join("api"))
        .expect("Should create a temporary nested resolver fixture directory");
    fs::write(
        temp_root.join("api/helpers.fol"),
        "fun[hid] helper(value: int): int = {\n    return value;\n}\n",
    )
    .expect("Should write the hidden routine namespace fixture");
    fs::write(
        temp_root.join("main.fol"),
        "fun[] main(input: int): int = {\n    return api::helper(input);\n}\n",
    )
    .expect("Should write the cross-namespace hidden routine lookup fixture");

    let errors = try_resolve_package_from_folder(
        temp_root
            .to_str()
            .expect("Temporary resolver fixture path should be valid UTF-8"),
    )
    .expect_err("Resolver should reject hidden routines across namespaces");

    assert!(
        errors.iter().any(|error| {
            error.kind() == ResolverErrorKind::UnresolvedName
                && error
                    .to_string()
                    .contains("could not resolve qualified callable routine 'api::helper'")
        }),
        "Hidden routines should stay file-private even when addressed through namespace roots"
    );

    fs::remove_dir_all(&temp_root)
        .expect("Temporary resolver fixture directory should be removable after the test");
}
