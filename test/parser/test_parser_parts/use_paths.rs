use super::*;
use fol_parser::ast::UsePathSeparator;
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

fn unique_temp_root(label: &str) -> std::path::PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("System time should be after unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "fol_use_paths_{}_{}_{}",
        label,
        std::process::id(),
        stamp
    ))
}

fn use_decl_path_segments<'a>(
    declarations: &'a [AstNode],
    expected_name: &str,
) -> &'a [fol_parser::ast::UsePathSegment] {
    declarations
        .iter()
        .find_map(|node| match node {
            AstNode::UseDecl {
                name,
                path_segments,
                ..
            } if name == expected_name => Some(path_segments.as_slice()),
            _ => None,
        })
        .expect("Expected use declaration to expose structured path segments")
}

#[test]
fn test_use_declaration_accepts_direct_quoted_paths() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_use_direct_quoted_path.fol")
            .expect("Should read direct quoted use-path fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should accept direct quoted use paths");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| {
                matches!(node, AstNode::UseDecl { name, path, .. } if name == "warn" && path == "std/warn")
            }));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_use_declaration_accepts_multiple_direct_quoted_paths() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_use_multi_direct_quoted_paths.fol")
            .expect("Should read multi direct quoted use-path fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should accept multiple direct quoted use paths");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| {
                matches!(node, AstNode::UseDecl { name, path, .. } if name == "warn" && path == "std/warn")
            }));
            assert!(declarations.iter().any(|node| {
                matches!(node, AstNode::UseDecl { name, path, .. } if name == "trace" && path == "std/trace")
            }));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_use_declaration_accepts_direct_bare_paths() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_use_direct_bare_path.fol")
            .expect("Should read direct bare use-path fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should accept direct bare use paths");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| {
                matches!(node, AstNode::UseDecl { name, path, .. } if name == "file" && path == "std::fs::File")
            }));
            assert_eq!(
                use_decl_path_segments(&declarations, "file"),
                &[
                    fol_parser::ast::UsePathSegment {
                        separator: None,
                        spelling: "std".to_string(),
                    },
                    fol_parser::ast::UsePathSegment {
                        separator: Some(UsePathSeparator::DoubleColon),
                        spelling: "fs".to_string(),
                    },
                    fol_parser::ast::UsePathSegment {
                        separator: Some(UsePathSeparator::DoubleColon),
                        spelling: "File".to_string(),
                    },
                ],
                "Direct bare use paths should retain segment spelling and separators"
            );
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_use_declaration_accepts_multiple_direct_bare_paths() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_use_multi_direct_bare_paths.fol")
            .expect("Should read multi direct bare use-path fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should accept multiple direct bare use paths");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| {
                matches!(node, AstNode::UseDecl { name, path, .. } if name == "file" && path == "std::fs::File")
            }));
            assert!(declarations.iter().any(|node| {
                matches!(node, AstNode::UseDecl { name, path, .. } if name == "warn" && path == "fmt::log")
            }));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_use_declaration_direct_quoted_paths_preserve_inner_opposite_quotes() {
    let temp_root = unique_temp_root("direct_inner_quotes");
    fs::create_dir_all(&temp_root).expect("Should create temp use-path fixture dir");
    let fixture = temp_root.join("direct_inner_quotes.fol");
    fs::write(
        &fixture,
        "use warn: loc = \"std/'warn'\";\nuse trace: loc = 'std/\"trace\"';\n",
    )
    .expect("Should write temp direct quoted use-path fixture");

    let mut file_stream = FileStream::from_file(
        fixture
            .to_str()
            .expect("Use-path fixture path should be UTF-8"),
    )
    .expect("Should read temp direct quoted use-path fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should preserve inner opposite-family quotes in direct use paths");

    fs::remove_dir_all(&temp_root).ok();

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| {
                matches!(node, AstNode::UseDecl { name, path, .. } if name == "warn" && path == "std/'warn'")
            }));
            assert!(declarations.iter().any(|node| {
                matches!(node, AstNode::UseDecl { name, path, .. } if name == "trace" && path == "std/\"trace\"")
            }));
            assert_eq!(
                use_decl_path_segments(&declarations, "warn"),
                &[
                    fol_parser::ast::UsePathSegment {
                        separator: None,
                        spelling: "std".to_string(),
                    },
                    fol_parser::ast::UsePathSegment {
                        separator: Some(UsePathSeparator::Slash),
                        spelling: "'warn'".to_string(),
                    },
                ],
                "Quoted use paths should preserve inner quoted segment spelling"
            );
            assert_eq!(
                use_decl_path_segments(&declarations, "trace"),
                &[
                    fol_parser::ast::UsePathSegment {
                        separator: None,
                        spelling: "std".to_string(),
                    },
                    fol_parser::ast::UsePathSegment {
                        separator: Some(UsePathSeparator::Slash),
                        spelling: "\"trace\"".to_string(),
                    },
                ],
                "Quoted use paths should preserve inner opposite-family quotes per segment"
            );
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_use_declaration_braced_paths_preserve_inner_opposite_quotes() {
    let temp_root = unique_temp_root("braced_inner_quotes");
    fs::create_dir_all(&temp_root).expect("Should create temp use-path fixture dir");
    let fixture = temp_root.join("braced_inner_quotes.fol");
    fs::write(
        &fixture,
        "use warn: mod = {\"std/'warn'\"};\nuse trace: mod = {'std/\"trace\"'};\n",
    )
    .expect("Should write temp braced quoted use-path fixture");

    let mut file_stream = FileStream::from_file(
        fixture
            .to_str()
            .expect("Braced use-path fixture path should be UTF-8"),
    )
    .expect("Should read temp braced quoted use-path fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should preserve inner opposite-family quotes in braced use paths");

    fs::remove_dir_all(&temp_root).ok();

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| {
                matches!(node, AstNode::UseDecl { name, path, .. } if name == "warn" && path == "std/'warn'")
            }));
            assert!(declarations.iter().any(|node| {
                matches!(node, AstNode::UseDecl { name, path, .. } if name == "trace" && path == "std/\"trace\"")
            }));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_use_declaration_preserves_mixed_separator_path_structure() {
    let temp_root = unique_temp_root("mixed_separator_segments");
    fs::create_dir_all(&temp_root).expect("Should create temp use-path fixture dir");
    let fixture = temp_root.join("mixed_separator_segments.fol");
    fs::write(&fixture, "use warn: mod = std::log/warn;\n")
        .expect("Should write temp mixed-separator use-path fixture");

    let mut file_stream = FileStream::from_file(
        fixture
            .to_str()
            .expect("Mixed-separator use-path fixture path should be UTF-8"),
    )
    .expect("Should read temp mixed-separator use-path fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should preserve mixed-separator use path structure");

    fs::remove_dir_all(&temp_root).ok();

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| {
                matches!(node, AstNode::UseDecl { name, path, .. } if name == "warn" && path == "std::log/warn")
            }));
            assert_eq!(
                use_decl_path_segments(&declarations, "warn"),
                &[
                    fol_parser::ast::UsePathSegment {
                        separator: None,
                        spelling: "std".to_string(),
                    },
                    fol_parser::ast::UsePathSegment {
                        separator: Some(UsePathSeparator::DoubleColon),
                        spelling: "log".to_string(),
                    },
                    fol_parser::ast::UsePathSegment {
                        separator: Some(UsePathSeparator::Slash),
                        spelling: "warn".to_string(),
                    },
                ],
                "Mixed use path separators should survive AST lowering without string re-parsing"
            );
        }
        _ => panic!("Expected program node"),
    }
}
