use super::*;
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

fn unique_temp_root(label: &str) -> std::path::PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("System time should be after unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "fol_parser_file_boundaries_{}_{}_{}",
        label,
        std::process::id(),
        stamp
    ))
}

fn parse_program_from_folder(
    root: &std::path::Path,
) -> Result<AstNode, Vec<Box<dyn fol_types::Glitch>>> {
    let mut file_stream = FileStream::from_folder(
        root.to_str()
            .expect("Temporary folder fixture path should be UTF-8"),
    )
    .expect("Should build a parser folder fixture stream");
    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    parser.parse(&mut lexer)
}

fn write_folder_fixture(root: &std::path::Path, files: &[(&str, &str)]) {
    fs::create_dir_all(root).expect("Should create temporary parser folder fixture");
    for (name, source) in files {
        fs::write(root.join(name), source).expect("Should write temporary parser folder file");
    }
}

#[test]
fn test_complete_top_level_declarations_remain_separate_across_folder_boundaries() {
    let temp_root = unique_temp_root("complete_top_level");
    write_folder_fixture(
        &temp_root,
        &[("00_a.fol", "var a = 1\n"), ("10_b.fol", "var b = 2\n")],
    );

    let ast = parse_program_from_folder(&temp_root)
        .expect("Complete declarations in separate files should still parse");

    fs::remove_dir_all(&temp_root).ok();

    match ast {
        AstNode::Program { declarations } => {
            assert!(matches!(
                declarations.as_slice(),
                [AstNode::VarDecl { name: first, .. }, AstNode::VarDecl { name: second, .. }]
                    if first == "a" && second == "b"
            ));
        }
        other => panic!("Expected program node, got {other:?}"),
    }
}

#[test]
fn test_binding_initializer_cannot_continue_into_next_file() {
    let temp_root = unique_temp_root("split_binding_value");
    write_folder_fixture(
        &temp_root,
        &[("00_a.fol", "var a =\n"), ("10_b.fol", "1\n")],
    );

    let parse_result = parse_program_from_folder(&temp_root);

    fs::remove_dir_all(&temp_root).ok();

    assert!(
        parse_result.is_err(),
        "A binding initializer should not continue across a file boundary"
    );
}

#[test]
fn test_routine_header_cannot_continue_into_next_file() {
    let temp_root = unique_temp_root("split_routine_header");
    write_folder_fixture(
        &temp_root,
        &[
            ("00_a.fol", "fun add(value: int)\n"),
            ("10_b.fol", ": int = { return value }\n"),
        ],
    );

    let parse_result = parse_program_from_folder(&temp_root);

    fs::remove_dir_all(&temp_root).ok();

    assert!(
        parse_result.is_err(),
        "A routine signature should not continue across a file boundary"
    );
}

#[test]
fn test_use_path_cannot_continue_into_next_file() {
    let temp_root = unique_temp_root("split_use_path");
    write_folder_fixture(
        &temp_root,
        &[
            ("00_a.fol", "use file: loc = std::fs::\n"),
            ("10_b.fol", "File\n"),
        ],
    );

    let parse_result = parse_program_from_folder(&temp_root);

    fs::remove_dir_all(&temp_root).ok();

    assert!(
        parse_result.is_err(),
        "A use path should not continue across a file boundary"
    );
}

#[test]
fn test_block_body_cannot_continue_into_next_file() {
    let temp_root = unique_temp_root("split_block_body");
    write_folder_fixture(
        &temp_root,
        &[
            ("00_a.fol", "fun value(): int = { return 1\n"),
            ("10_b.fol", "}\n"),
        ],
    );

    let parse_result = parse_program_from_folder(&temp_root);

    fs::remove_dir_all(&temp_root).ok();

    assert!(
        parse_result.is_err(),
        "An open block body should not continue across a file boundary"
    );
}
