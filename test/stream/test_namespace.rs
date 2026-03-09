// Tests for namespace functionality in fol-stream

use fol_stream::{Source, SourceType};

#[cfg(test)]
mod namespace_tests {
    use super::*;

    #[test]
    fn test_package_name_detection() {
        // Test that package name is correctly detected from Cargo.toml
        let sources = Source::init("test/legacy/main", SourceType::Folder)
            .expect("Should create sources with namespace");

        assert!(!sources.is_empty(), "Should have sources");

        // All sources should have the same package name
        let package_name = &sources[0].package;
        assert_eq!(
            package_name, "fol",
            "Package name should be 'fol' from Cargo.toml"
        );

        for source in &sources {
            assert_eq!(
                source.package, *package_name,
                "All sources should have same package name"
            );
        }

        println!("Detected package name: {}", package_name);
    }

    #[test]
    fn test_root_namespace() {
        // Test files in root directory get root namespace (package name)
        let sources = Source::init("test/legacy/main/main.fol", SourceType::File)
            .expect("Should create source");

        assert_eq!(sources.len(), 1, "Should have one source");
        let source = &sources[0];

        assert_eq!(source.package, "fol", "Package should be 'fol'");
        assert_eq!(
            source.namespace, "fol",
            "Root file should have root namespace"
        );

        println!("Root file namespace: {}", source.namespace);
    }

    #[test]
    fn test_subdirectory_namespace() {
        // Test files in subdirectories get proper namespace
        let sources =
            Source::init("test/legacy/main", SourceType::Folder).expect("Should create sources");

        // Find sources in subdirectories
        let subdir_sources: Vec<_> = sources
            .iter()
            .filter(|s| s.namespace != "fol") // Not root namespace
            .collect();

        assert!(
            !subdir_sources.is_empty(),
            "Should have sources in subdirectories"
        );

        for source in &subdir_sources {
            println!("File: {} -> Namespace: {}", source.path, source.namespace);

            // Namespace should start with package name
            assert!(
                source.namespace.starts_with("fol::"),
                "Namespace should start with package name: {}",
                source.namespace
            );

            // Should contain directory structure
            if source.path.contains("/var/") {
                assert!(
                    source.namespace.contains("var"),
                    "Should contain 'var' in namespace: {}",
                    source.namespace
                );
            }

            if source.path.contains("/single/") {
                assert!(
                    source.namespace.contains("single"),
                    "Should contain 'single' in namespace: {}",
                    source.namespace
                );
            }
        }
    }

    #[test]
    fn test_namespace_structure_mapping() {
        // Test specific directory to namespace mappings

        // Create a test directory structure for precise testing
        use std::fs;

        let test_dir = "test/stream/namespace_test";
        if !std::path::Path::new(test_dir).exists() {
            fs::create_dir_all(format!("{}/math/vector", test_dir)).ok();
            fs::create_dir_all(format!("{}/utils", test_dir)).ok();

            // Create test files
            fs::write(format!("{}/main.fol", test_dir), "// Root file").ok();
            fs::write(format!("{}/math/mod.fol", test_dir), "// Math module").ok();
            fs::write(
                format!("{}/math/vector/vec3.fol", test_dir),
                "// Vector3 implementation",
            )
            .ok();
            fs::write(
                format!("{}/utils/helper.fol", test_dir),
                "// Utility functions",
            )
            .ok();
        }

        let sources = Source::init_with_package(test_dir, SourceType::Folder, "myproject")
            .expect("Should create sources");

        // Verify namespace mappings
        for source in &sources {
            let filename = std::path::Path::new(&source.path)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");

            match filename {
                "main.fol" => {
                    assert_eq!(
                        source.namespace, "myproject",
                        "Root file should have root namespace"
                    );
                }
                "mod.fol" => {
                    assert_eq!(
                        source.namespace, "myproject::math",
                        "Math module should be in math namespace"
                    );
                }
                "vec3.fol" => {
                    assert_eq!(
                        source.namespace, "myproject::math::vector",
                        "Vector file should be in math::vector namespace"
                    );
                }
                "helper.fol" => {
                    assert_eq!(
                        source.namespace, "myproject::utils",
                        "Helper should be in utils namespace"
                    );
                }
                _ => {}
            }

            println!("File: {} -> Namespace: {}", filename, source.namespace);
        }

        // Clean up
        fs::remove_dir_all(test_dir).ok();
    }

    #[test]
    fn test_mod_directories_excluded_from_namespace() {
        // Test that .mod directories don't appear in namespaces
        let sources =
            Source::init("test/legacy/main", SourceType::Folder).expect("Should create sources");

        // No namespace should contain ".mod"
        for source in &sources {
            assert!(
                !source.namespace.contains(".mod"),
                "Namespace should not contain .mod: {}",
                source.namespace
            );

            // But the files from .mod directories should not be present at all
            assert!(
                !source.path.contains(".mod/"),
                "Should not have files from .mod directories: {}",
                source.path
            );
        }

        println!("All namespaces are clean of .mod references");
    }

    #[test]
    fn test_namespace_consistency() {
        // Test that all files in the same directory have the same namespace
        let sources =
            Source::init("test/legacy/main", SourceType::Folder).expect("Should create sources");

        use std::collections::HashMap;
        let mut dir_to_namespace: HashMap<String, String> = HashMap::new();

        for source in &sources {
            let dir = std::path::Path::new(&source.path)
                .parent()
                .unwrap_or_else(|| std::path::Path::new("."))
                .to_string_lossy()
                .to_string();

            if let Some(existing_namespace) = dir_to_namespace.get(&dir) {
                assert_eq!(
                    existing_namespace, &source.namespace,
                    "All files in same directory should have same namespace: {} vs {} in {}",
                    existing_namespace, source.namespace, dir
                );
            } else {
                dir_to_namespace.insert(dir, source.namespace.clone());
            }
        }

        println!("Directory to namespace mappings:");
        for (dir, namespace) in &dir_to_namespace {
            println!("  {} -> {}", dir, namespace);
        }
    }

    #[test]
    fn test_explicit_package_name() {
        // Test using explicit package name
        let sources =
            Source::init_with_package(
                "test/legacy/main/main.fol",
                SourceType::File,
                "custom_package",
            )
            .expect("Should create source with custom package");

        assert_eq!(sources.len(), 1, "Should have one source");
        let source = &sources[0];

        assert_eq!(
            source.package, "custom_package",
            "Should use explicit package name"
        );
        assert_eq!(
            source.namespace, "custom_package",
            "Should use custom package as root namespace"
        );
    }

    #[test]
    fn test_namespace_output_integration() {
        // Test that the namespace information is properly integrated
        let sources =
            Source::init("test/legacy/main", SourceType::Folder).expect("Should create sources");

        // Verify all expected properties are present
        for source in &sources {
            assert!(
                !source.namespace.is_empty(),
                "Namespace should not be empty"
            );
            assert!(!source.package.is_empty(), "Package should not be empty");
            assert!(!source.path.is_empty(), "Path should not be empty");

            // Namespace should be valid format
            assert!(
                source
                    .namespace
                    .chars()
                    .all(|c| c.is_alphanumeric() || c == ':' || c == '_'),
                "Namespace should only contain valid characters: {}",
                source.namespace
            );

            // If namespace contains ::, it should have at least package::something
            if source.namespace.contains("::") {
                let parts: Vec<&str> = source.namespace.split("::").collect();
                assert!(
                    parts.len() >= 2,
                    "Namespaced item should have at least 2 parts"
                );
                assert_eq!(
                    parts[0], source.package,
                    "First part should be package name"
                );
            }
        }

        println!("All sources have valid namespace information");
    }
}
