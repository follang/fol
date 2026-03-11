use super::*;
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
