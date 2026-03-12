use super::{try_resolve_package_from_folder, unique_temp_root};
use fol_resolver::ResolverErrorKind;
use std::fs;

#[test]
fn test_resolver_rejects_non_location_import_kinds_explicitly() {
    let fixtures = [
        ("unsupported_mod", "use fmt: mod = {core::fmt};\n"),
        ("unsupported_std", "use fmt: std = {fmt/log};\n"),
        (
            "unsupported_url",
            "use remote: url = {\"https://example.com/api\"};\n",
        ),
    ];

    for (label, source) in fixtures {
        let temp_root = unique_temp_root(label);
        fs::create_dir_all(&temp_root)
            .expect("Should create a temporary resolver fixture directory");
        fs::write(temp_root.join("main.fol"), source)
            .expect("Should write the unsupported import fixture");

        let errors = try_resolve_package_from_folder(
            temp_root
                .to_str()
                .expect("Temporary resolver fixture path should be valid UTF-8"),
        )
        .expect_err("Resolver should reject unsupported import kinds explicitly");

        assert!(
            errors
                .iter()
                .any(|error| error.kind() == ResolverErrorKind::Unsupported),
            "Resolver should report unsupported import kinds explicitly for fixture {}",
            label
        );

        fs::remove_dir_all(&temp_root)
            .expect("Temporary resolver fixture directory should be removable after the test");
    }
}
