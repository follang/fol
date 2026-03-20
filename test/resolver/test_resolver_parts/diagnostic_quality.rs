use super::{try_resolve_package_from_folder, unique_temp_root};
use fol_resolver::ResolverErrorKind;
use std::fs;

#[test]
fn test_resolver_unresolved_qualified_type_diagnostics_keep_exact_role_and_location() {
    let temp_root = unique_temp_root("resolver_diag_qualified_type");
    fs::create_dir_all(&temp_root).expect("Should create a temporary resolver fixture directory");
    fs::write(temp_root.join("main.fol"), "ali Broken: tools::Missing;\n")
        .expect("Should write the unresolved qualified type fixture");

    let errors = try_resolve_package_from_folder(
        temp_root
            .to_str()
            .expect("Temporary resolver fixture path should be valid UTF-8"),
    )
    .expect_err("Resolver should reject unresolved qualified type references");
    let error = errors
        .iter()
        .find(|error| error.kind() == ResolverErrorKind::UnresolvedName)
        .expect("Resolver should report an unresolved-name error");
    let origin = error
        .origin()
        .expect("Qualified unresolved type diagnostics should keep exact syntax origins");

    assert!(
        error
            .to_string()
            .contains("could not resolve qualified type 'tools::Missing'"),
        "Resolver should report the exact unresolved role and name"
    );
    assert_eq!(
        origin.file.as_deref(),
        Some(
            temp_root
                .join("main.fol")
                .to_str()
                .expect("Temporary resolver fixture path should be valid UTF-8")
        ),
        "Qualified unresolved diagnostics should keep the exact source file"
    );
    assert_eq!(origin.line, 1, "Qualified unresolved diagnostics should keep the exact line");
    assert_eq!(
        origin.column, 13,
        "Qualified unresolved diagnostics should point at the qualified type root token"
    );

    fs::remove_dir_all(&temp_root)
        .expect("Temporary resolver fixture directory should be removable after the test");
}

#[test]
fn test_resolver_unresolved_named_inquiry_target_diagnostics_use_target_role() {
    let temp_root = unique_temp_root("resolver_diag_inquiry_target");
    fs::create_dir_all(&temp_root).expect("Should create a temporary resolver fixture directory");
    fs::write(
        temp_root.join("main.fol"),
        "fun[] main(): int = {\n    return 0;\n    where(cache) {\n        0;\n    };\n};\n",
    )
    .expect("Should write the unresolved inquiry target fixture");

    let errors = try_resolve_package_from_folder(
        temp_root
            .to_str()
            .expect("Temporary resolver fixture path should be valid UTF-8"),
    )
    .expect_err("Resolver should reject unresolved inquiry targets");

    assert!(
        errors.iter().any(|error| {
            error.kind() == ResolverErrorKind::UnresolvedName
                && error
                    .to_string()
                    .contains("could not resolve inquiry target 'cache'")
        }),
        "Resolver should report the exact unresolved inquiry-target role"
    );

    fs::remove_dir_all(&temp_root)
        .expect("Temporary resolver fixture directory should be removable after the test");
}
