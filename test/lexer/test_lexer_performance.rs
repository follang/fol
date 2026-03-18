use super::*;
use std::time::Instant;

#[test]
fn test_lexer_performance() {
    // Use the more complex legacy test file from the unified test tree
    let test_file = "test/legacy/main/main.fol";

    if std::path::Path::new(test_file).exists() {
        let start = Instant::now();
        let tokens = tokenize_file(test_file);
        let duration = start.elapsed();

        let code_tokens = tokens
            .iter()
            .filter(|(key, _)| !key.is_space() && !key.is_void())
            .count();

        println!("Lexed {} tokens in {:?}", code_tokens, duration);

        assert!(tokens.len() > 50, "Should tokenize substantial file");
        assert!(
            duration.as_millis() < 100,
            "Should tokenize reasonably quickly"
        );
    }
}

#[test]
fn test_lexer_stages() {
    // Test that we can create lexer at different stages
    let mut file_stream =
        FileStream::from_file("test/lexer/mixed.fol").expect("Should read mixed.fol");

    // Test Stage 3 (final stage)
    let _lexer = Elements::init(&mut file_stream);

    // Just verify we can create the lexer without errors by reading current token.
    assert!(
        _lexer.curr(false).is_ok(),
        "Should be able to read current token in stage 3 lexer"
    );
}
