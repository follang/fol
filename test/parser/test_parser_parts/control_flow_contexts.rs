use super::*;
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

fn unique_temp_root(label: &str) -> std::path::PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("System time should be after unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "fol_parser_control_flow_{}_{}_{}",
        label,
        std::process::id(),
        stamp
    ))
}

fn parse_first_error_from_source(label: &str, source: &str) -> ParseError {
    let temp_root = unique_temp_root(label);
    fs::create_dir_all(&temp_root).expect("Should create temporary control-flow fixture dir");
    let fixture = temp_root.join("control_flow_context.fol");
    fs::write(&fixture, source).expect("Should write temporary control-flow fixture");

    let mut file_stream = FileStream::from_file(
        fixture
            .to_str()
            .expect("Temporary control-flow fixture path should be UTF-8"),
    )
    .expect("Should open temporary control-flow fixture");
    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let errors = parser
        .parse(&mut lexer)
        .expect_err("Parser should reject out-of-context control flow");

    fs::remove_dir_all(&temp_root).ok();

    errors
        .first()
        .and_then(|error| error.as_ref().as_any().downcast_ref::<ParseError>())
        .cloned()
        .expect("First parser error should be ParseError")
}

#[test]
fn test_top_level_return_is_rejected_outside_routine_context() {
    let error = parse_first_error_from_source("top_level_return", "return 1;\n");

    assert!(
        error
            .to_string()
            .contains("'return' is only allowed inside routines"),
        "Top-level return should fail with routine-context wording, got: {}",
        error
    );
    assert_eq!(error.line(), 1, "Top-level return should point at its own line");
    assert_eq!(
        error.column(),
        1,
        "Top-level return should point at the return keyword itself"
    );
}

#[test]
fn test_branch_body_return_is_rejected_without_routine_context() {
    let error = parse_first_error_from_source(
        "branch_body_return",
        "when(value) {\n    case(ready) {\n        return 1;\n    }\n}\n",
    );

    assert!(
        error
            .to_string()
            .contains("'return' is only allowed inside routines"),
        "Branch-body return should fail with routine-context wording, got: {}",
        error
    );
    assert_eq!(error.line(), 3, "Branch-body return should report the return line");
    assert_eq!(
        error.column(),
        9,
        "Branch-body return should point at the nested return keyword"
    );
}

#[test]
fn test_top_level_break_is_rejected_outside_loop_context() {
    let error = parse_first_error_from_source("top_level_break", "break;\n");

    assert!(
        error
            .to_string()
            .contains("'break' is only allowed inside loops"),
        "Top-level break should fail with loop-context wording, got: {}",
        error
    );
    assert_eq!(error.line(), 1, "Top-level break should point at its own line");
    assert_eq!(
        error.column(),
        1,
        "Top-level break should point at the break keyword itself"
    );
}

#[test]
fn test_routine_break_is_rejected_without_loop_context() {
    let error = parse_first_error_from_source(
        "routine_break",
        "fun bad(): int = {\n    break;\n}\n",
    );

    assert!(
        error
            .to_string()
            .contains("'break' is only allowed inside loops"),
        "Routine break should fail with loop-context wording, got: {}",
        error
    );
    assert_eq!(error.line(), 2, "Routine break should report the break line");
    assert_eq!(
        error.column(),
        5,
        "Routine break should point at the nested break keyword"
    );
}
