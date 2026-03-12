use super::{resolve_package_from_folder, try_resolve_package_from_folder, unique_temp_root};
use fol_resolver::{ResolverErrorKind, ScopeKind};
use std::fs;

#[test]
fn test_resolver_resolves_use_loc_against_the_current_package_root() {
    let temp_root = unique_temp_root("use_loc_package_root");
    fs::create_dir_all(&temp_root).expect("Should create a temporary resolver fixture directory");
    let package = temp_root
        .file_name()
        .expect("Temp resolver root should have a folder name")
        .to_string_lossy()
        .to_string();
    fs::write(
        temp_root.join("main.fol"),
        format!("use pkg: loc = {{{package}}};\nfun[] main(): int = {{\n    return pkg;\n}}\n"),
    )
    .expect("Should write the package-root import fixture");

    let resolved = resolve_package_from_folder(
        temp_root
            .to_str()
            .expect("Temporary resolver fixture path should be valid UTF-8"),
    );
    let import = resolved
        .imports_in_scope(resolved.program_scope)
        .into_iter()
        .find(|import| import.alias_name == "pkg")
        .expect("Resolver should keep the package-root import record");

    assert_eq!(
        import.target_scope,
        Some(resolved.program_scope),
        "Package-root location imports should resolve to the program scope"
    );

    fs::remove_dir_all(&temp_root)
        .expect("Temporary resolver fixture directory should be removable after the test");
}

#[test]
fn test_resolver_resolves_use_loc_against_nested_namespaces() {
    let temp_root = unique_temp_root("use_loc_namespace");
    fs::create_dir_all(temp_root.join("net/http"))
        .expect("Should create a temporary resolver fixture directory");
    let package = temp_root
        .file_name()
        .expect("Temp resolver root should have a folder name")
        .to_string_lossy()
        .to_string();
    fs::write(
        temp_root.join("net/http/route.fol"),
        "var handler: int = 1;\n",
    )
    .expect("Should write the imported namespace fixture");
    fs::write(
        temp_root.join("main.fol"),
        "use http: loc = {net::http};\nfun[] main(): int = {\n    return http;\n}\n",
    )
    .expect("Should write the namespace import fixture");

    let resolved = resolve_package_from_folder(
        temp_root
            .to_str()
            .expect("Temporary resolver fixture path should be valid UTF-8"),
    );
    let expected_scope = resolved
        .namespace_scope(&format!("{package}::net::http"))
        .expect("Nested namespace scope should exist");
    let import = resolved
        .imports_in_scope(resolved.program_scope)
        .into_iter()
        .find(|import| import.alias_name == "http")
        .expect("Resolver should keep the namespace import record");

    assert_eq!(
        import.target_scope,
        Some(expected_scope),
        "Namespace location imports should resolve to the imported namespace scope"
    );
    assert!(
        matches!(
            resolved.scope(expected_scope).map(|scope| &scope.kind),
            Some(ScopeKind::NamespaceRoot { .. })
        ),
        "Resolved namespace imports should point at namespace scopes"
    );

    fs::remove_dir_all(&temp_root)
        .expect("Temporary resolver fixture directory should be removable after the test");
}

#[test]
fn test_resolver_reports_missing_use_loc_targets() {
    let temp_root = unique_temp_root("use_loc_missing_target");
    fs::create_dir_all(&temp_root).expect("Should create a temporary resolver fixture directory");
    fs::write(
        temp_root.join("main.fol"),
        "use missing: loc = {net::missing};\nfun[] main(): int = {\n    return 0;\n}\n",
    )
    .expect("Should write the missing import fixture");

    let errors = try_resolve_package_from_folder(
        temp_root
            .to_str()
            .expect("Temporary resolver fixture path should be valid UTF-8"),
    )
    .expect_err("Resolver should reject missing local location imports");

    assert!(
        errors.iter().any(|error| {
            error.kind() == ResolverErrorKind::UnresolvedName
                && error
                    .to_string()
                    .contains("could not resolve local import target 'net::missing'")
        }),
        "Resolver should report unresolved local import targets explicitly"
    );

    fs::remove_dir_all(&temp_root)
        .expect("Temporary resolver fixture directory should be removable after the test");
}

#[test]
fn test_resolver_reports_ambiguous_use_loc_targets() {
    let temp_root = unique_temp_root("use_loc_ambiguous_target");
    let package = temp_root
        .file_name()
        .expect("Temp resolver root should have a folder name")
        .to_string_lossy()
        .to_string();
    fs::create_dir_all(temp_root.join(&package))
        .expect("Should create a temporary resolver fixture directory");
    fs::write(
        temp_root.join(package.clone()).join("inner.fol"),
        "var value: int = 1;\n",
    )
    .expect("Should write the ambiguous namespace fixture");
    fs::write(
        temp_root.join("main.fol"),
        format!("use local: loc = {{{package}}};\nfun[] main(): int = {{\n    return 0;\n}}\n"),
    )
    .expect("Should write the ambiguous import fixture");

    let errors = try_resolve_package_from_folder(
        temp_root
            .to_str()
            .expect("Temporary resolver fixture path should be valid UTF-8"),
    )
    .expect_err("Resolver should reject ambiguous local location imports");

    assert!(
        errors.iter().any(|error| {
            error.kind() == ResolverErrorKind::AmbiguousReference
                && error
                    .to_string()
                    .contains(&format!("local import target '{package}' is ambiguous"))
        }),
        "Resolver should report ambiguous local import targets explicitly"
    );

    fs::remove_dir_all(&temp_root)
        .expect("Temporary resolver fixture directory should be removable after the test");
}
