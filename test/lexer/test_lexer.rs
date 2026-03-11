// Comprehensive tests for fol-lexer module

use fol_lexer::{lexer::stage3::Elements, token::KEYWORD, token::*};
use fol_stream::FileStream;

fn tokenize_file(path: &str) -> Vec<(KEYWORD, String)> {
    let mut file_stream =
        FileStream::from_file(path).unwrap_or_else(|_| panic!("Should be able to read {}", path));

    let mut lexer = Elements::init(&mut file_stream);
    let mut tokens = Vec::new();

    // Extract tokens until EOF
    for _ in 0..10_000 {
        match lexer.curr(false) {
            Ok(token) => {
                let keyword = token.key();
                let content = token.con().to_string();

                if keyword == KEYWORD::Void(VOID::EndFile) {
                    tokens.push((keyword, content));
                    break;
                }

                tokens.push((keyword, content));
                if lexer.bump().is_none() {
                    break;
                }
            }
            Err(_) => break,
        }
    }

    assert!(
        tokens.len() < 10_000,
        "Tokenization did not terminate for {}",
        path
    );

    tokens
}

#[cfg(test)]
mod lexer_tests {
    use super::*;

    #[test]
    fn test_keywords() {
        let tokens = tokenize_file("test/lexer/keywords.fol");

        // Filter out spaces and EOF
        let keywords: Vec<_> = tokens
            .iter()
            .filter(|(key, _)| !key.is_space() && !key.is_eof())
            .collect();

        assert!(keywords.len() >= 10, "Should have multiple keywords");

        // Check for specific keywords
        let keyword_strings: Vec<String> = keywords
            .iter()
            .map(|(_, content)| content.clone())
            .collect();

        assert!(
            keyword_strings.contains(&"use".to_string()),
            "Should contain 'use' keyword"
        );
        assert!(
            keyword_strings.contains(&"var".to_string()),
            "Should contain 'var' keyword"
        );
        assert!(
            keyword_strings.contains(&"fun".to_string()),
            "Should contain 'fun' keyword"
        );
        assert!(
            keyword_strings.contains(&"pro".to_string()),
            "Should contain 'pro' keyword"
        );
        assert!(
            keyword_strings.contains(&"let".to_string()),
            "Should contain 'let' keyword"
        );
        assert!(
            keyword_strings.contains(&"typ".to_string()),
            "Should contain 'typ' keyword"
        );
        assert!(
            keyword_strings.contains(&"std".to_string()),
            "Should contain 'std' keyword"
        );
        assert!(
            keyword_strings.contains(&"log".to_string()),
            "Should contain 'log' keyword"
        );
        assert!(
            keyword_strings.contains(&"cast".to_string()),
            "Should contain 'cast' keyword"
        );
        assert!(
            keyword_strings.contains(&"on".to_string()),
            "Should contain 'on' keyword"
        );
        assert!(
            keyword_strings.contains(&"while".to_string()),
            "Should contain 'while' keyword"
        );
        assert!(
            keyword_strings.contains(&"async".to_string()),
            "Should contain 'async' keyword"
        );
        assert!(
            keyword_strings.contains(&"await".to_string()),
            "Should contain 'await' keyword"
        );
        assert!(
            keyword_strings.contains(&"select".to_string()),
            "Should contain 'select' keyword"
        );
    }

    #[test]
    fn test_stage3_starts_on_first_real_token() {
        let mut file_stream =
            FileStream::from_file("test/lexer/mixed.fol").expect("Should read mixed.fol");
        let lexer = Elements::init(&mut file_stream);
        let token = lexer
            .curr(false)
            .expect("Stage 3 lexer should expose the first token immediately");

        assert_eq!(token.loc().row(), 1, "First token should not be synthetic");
        assert_eq!(token.loc().col(), 1, "First token should start at column 1");
        assert_eq!(token.con(), "var", "First token should be the first real token");
    }

    #[test]
    fn test_literals() {
        let tokens = tokenize_file("test/lexer/literals.fol");

        // Filter out spaces and EOF
        let literals: Vec<_> = tokens.iter().filter(|(key, _)| key.is_literal()).collect();

        assert!(!literals.is_empty(), "Should have literal tokens");

        let literal_strings: Vec<String> = literals
            .iter()
            .map(|(_, content)| content.clone())
            .collect();

        // Check for different types of literals
        let has_decimal = literal_strings
            .iter()
            .any(|s| s.chars().all(|c| c.is_ascii_digit()));
        let has_float = literal_strings.iter().any(|s| s.contains('.'));
        let _has_string = literal_strings
            .iter()
            .any(|s| s.starts_with('"') && s.ends_with('"'));

        assert!(has_decimal || has_float, "Should have numeric literals");
        println!("Found literals: {:?}", literal_strings);
    }

    #[test]
    fn test_float_literal_payload_is_preserved() {
        let tokens = tokenize_file("test/lexer/literals.fol");

        assert!(
            tokens.iter().any(|(key, content)| {
                matches!(key, KEYWORD::Literal(LITERAL::Float)) && content == "3.14"
            }),
            "Lexer should preserve the full float payload for 3.14"
        );
    }

    #[test]
    fn test_uppercase_prefixed_numeric_literals_tokenize_as_numbers() {
        let tokens = tokenize_file("test/lexer/literals_uppercase.fol");
        let significant: Vec<(KEYWORD, String)> = tokens
            .into_iter()
            .filter(|(key, _)| !key.is_space() && !key.is_eof())
            .collect();

        assert_eq!(
            significant,
            vec![
                (KEYWORD::Literal(LITERAL::Hexal), "0X1A".to_string()),
                (KEYWORD::Literal(LITERAL::Octal), "0O17".to_string()),
                (KEYWORD::Literal(LITERAL::Binary), "0B1010".to_string()),
            ],
            "Uppercase numeric prefixes should tokenize the same as lowercase forms"
        );
    }

    #[test]
    fn test_numeric_literal_payloads_preserve_original_spelling() {
        let tokens = tokenize_file("test/lexer/literals_spelling.fol");
        let significant: Vec<(KEYWORD, String)> = tokens
            .into_iter()
            .filter(|(key, _)| !key.is_space() && !key.is_eof())
            .collect();

        assert_eq!(
            significant,
            vec![
                (KEYWORD::Literal(LITERAL::Deciaml), "1_000".to_string()),
                (KEYWORD::Literal(LITERAL::Hexal), "0xCA_FE".to_string()),
                (KEYWORD::Literal(LITERAL::Octal), "0o7_7".to_string()),
                (KEYWORD::Literal(LITERAL::Binary), "0b1010_0001".to_string()),
                (KEYWORD::Literal(LITERAL::Hexal), "0XCA_FE".to_string()),
                (KEYWORD::Literal(LITERAL::Octal), "0O7_7".to_string()),
                (KEYWORD::Literal(LITERAL::Binary), "0B1010_0001".to_string()),
            ],
            "Numeric literal payloads should preserve source spelling for supported forms"
        );
    }

    #[test]
    fn test_leading_dot_float_tokenizes_as_float() {
        let tokens = tokenize_file("test/lexer/leading_dot_float.fol");
        let significant: Vec<(KEYWORD, String)> = tokens
            .into_iter()
            .filter(|(key, _)| !key.is_space() && !key.is_eof())
            .collect();

        assert_eq!(
            significant,
            vec![(KEYWORD::Literal(LITERAL::Float), ".5".to_string())],
            "A leading-dot numeric literal should tokenize as a float"
        );
    }

    #[test]
    fn test_negative_numbers_keep_minus_as_a_separate_token() {
        let tokens = tokenize_file("test/lexer/negative_numbers.fol");
        let significant: Vec<(KEYWORD, String)> = tokens
            .into_iter()
            .filter(|(key, _)| !key.is_space() && !key.is_eof())
            .collect();

        assert_eq!(
            significant,
            vec![
                (KEYWORD::Symbol(SYMBOL::Minus), "-".to_string()),
                (KEYWORD::Literal(LITERAL::Deciaml), "42".to_string()),
                (KEYWORD::Symbol(SYMBOL::Minus), "-".to_string()),
                (KEYWORD::Literal(LITERAL::Float), "3.5".to_string()),
            ],
            "Minus should remain a separate token so unary negation stays parser-level"
        );
    }

    #[test]
    fn test_backticks_remain_operator_any_tokens() {
        let tokens = tokenize_file("test/lexer/backticks.fol");
        let significant: Vec<(KEYWORD, String)> = tokens
            .into_iter()
            .filter(|(key, _)| !key.is_space() && !key.is_eof())
            .collect();

        assert_eq!(
            significant,
            vec![(KEYWORD::Operator(OPERATOR::ANY), "`macroish`".to_string())],
            "Backticks should remain opaque operator-like tokens until the language gives them a narrower meaning"
        );
    }

    #[test]
    fn test_quoted_literal_payloads_keep_delimiters() {
        let tokens = tokenize_file("test/lexer/literals.fol");

        assert!(
            tokens.iter().any(|(key, content)| {
                matches!(key, KEYWORD::Literal(LITERAL::Stringy)) && content == "\"hello\""
            }),
            "String literal payload should keep its double-quote delimiters"
        );
        assert!(
            tokens.iter().any(|(key, content)| {
                matches!(key, KEYWORD::Literal(LITERAL::Stringy)) && content == "'c'"
            }),
            "Single-quoted literal payload should keep its delimiters"
        );
    }

    #[test]
    fn test_symbols() {
        let tokens = tokenize_file("test/lexer/symbols.fol");

        let symbols: Vec<_> = tokens
            .iter()
            .filter(|(key, _)| matches!(key, KEYWORD::Symbol(_)))
            .collect();

        assert!(!symbols.is_empty(), "Should have symbol tokens");

        let symbol_strings: Vec<String> =
            symbols.iter().map(|(_, content)| content.clone()).collect();
        let normalized: Vec<String> = symbol_strings
            .iter()
            .map(|s| s.trim().to_string())
            .collect();

        // Check for specific symbols - escape braces in format strings
        assert!(
            normalized.contains(&"{".to_string()),
            "Should contain opening brace"
        );
        assert!(
            normalized.contains(&"}".to_string()),
            "Should contain closing brace"
        );
        assert!(normalized.contains(&"(".to_string()), "Should contain '('");
        assert!(normalized.contains(&")".to_string()), "Should contain ')'");
        assert!(normalized.contains(&";".to_string()), "Should contain ';'");
        assert!(normalized.contains(&",".to_string()), "Should contain ','");

        println!("Found symbols: {:?}", symbol_strings);
    }

    #[test]
    fn test_identifiers() {
        let tokens = tokenize_file("test/lexer/identifiers.fol");

        let identifiers: Vec<_> = tokens.iter().filter(|(key, _)| key.is_ident()).collect();

        assert!(!identifiers.is_empty(), "Should have identifier tokens");

        let identifier_strings: Vec<String> = identifiers
            .iter()
            .map(|(_, content)| content.clone())
            .collect();

        // Check for different identifier patterns
        let has_simple = identifier_strings.iter().any(|s| s == "foo" || s == "bar");
        let _has_underscore = identifier_strings.iter().any(|s| s.contains('_'));
        let _has_camel_case = identifier_strings
            .iter()
            .any(|s| s.chars().any(|c| c.is_uppercase()));

        println!("Found identifiers: {:?}", identifier_strings);
        assert!(has_simple, "Should have simple identifiers");
    }

    #[test]
    fn test_comments() {
        let tokens = tokenize_file("test/lexer/comments.fol");

        let comments: Vec<_> = tokens.iter().filter(|(key, _)| key.is_comment()).collect();

        // Comments might be filtered out at lexer level, so this test verifies
        // that the lexer can handle files with comments without errors
        assert!(
            !tokens.is_empty(),
            "Should successfully tokenize file with comments"
        );

        // Check that we have some non-comment tokens (the actual code)
        let non_void_tokens: Vec<_> = tokens
            .iter()
            .filter(|(key, _)| !key.is_void() && !key.is_comment())
            .collect();

        assert!(
            !non_void_tokens.is_empty(),
            "Should have actual code tokens besides comments"
        );

        println!(
            "Total tokens: {}, Comments: {}, Code tokens: {}",
            tokens.len(),
            comments.len(),
            non_void_tokens.len()
        );
    }

    #[test]
    fn test_comments_do_not_disturb_surrounding_code_tokens() {
        let tokens = tokenize_file("test/lexer/comments.fol");
        let significant: Vec<(KEYWORD, String)> = tokens
            .into_iter()
            .filter(|(key, _)| !key.is_void() && !key.is_comment())
            .collect();

        assert_eq!(
            significant,
            vec![
                (KEYWORD::Keyword(BUILDIN::Var), "var".to_string()),
                (KEYWORD::Identifier, "x".to_string()),
                (KEYWORD::Symbol(SYMBOL::Equal), "=".to_string()),
                (KEYWORD::Literal(LITERAL::Deciaml), "5".to_string()),
                (KEYWORD::Symbol(SYMBOL::Semi), "; ".to_string()),
            ],
            "Line and block comments should be ignorable without disturbing code token order"
        );
    }

    #[test]
    fn test_comments_are_fully_ignorable_and_void_payloads_are_normalized() {
        let tokens = tokenize_file("test/lexer/comments.fol");
        let comment_tokens: Vec<_> = tokens.iter().filter(|(key, _)| key.is_comment()).collect();
        let normalized_voids: Vec<_> = tokens
            .iter()
            .filter(|(key, _)| key.is_void() && !key.is_eof())
            .collect();

        assert!(
            comment_tokens.is_empty(),
            "Stage 3 should not expose ordinary comments as parser-visible tokens"
        );
        assert!(
            normalized_voids.iter().all(|(_, content)| content == " "),
            "Ignorable separators should normalize to a single-space payload"
        );
    }

    #[test]
    fn test_mixed_content() {
        let tokens = tokenize_file("test/lexer/mixed.fol");

        // Filter out spaces
        let code_tokens: Vec<_> = tokens
            .iter()
            .filter(|(key, _)| !key.is_space() && !key.is_eof())
            .collect();

        assert!(
            code_tokens.len() > 10,
            "Should have substantial number of tokens"
        );

        // Check for variety of token types
        let has_keywords = code_tokens.iter().any(|(key, _)| key.is_buildin());
        let has_identifiers = code_tokens.iter().any(|(key, _)| key.is_ident());
        let has_symbols = code_tokens
            .iter()
            .any(|(key, _)| matches!(key, KEYWORD::Symbol(_)));
        let _has_literals = code_tokens.iter().any(|(key, _)| key.is_literal());

        assert!(has_keywords, "Should have keywords");
        assert!(has_identifiers, "Should have identifiers");
        assert!(has_symbols, "Should have symbols");

        // Print token sequence for debugging
        println!("Mixed content tokens:");
        for (i, (key, content)) in code_tokens.iter().enumerate() {
            println!("  {}: {:?} = '{}'", i, key, content);
        }
    }

    #[test]
    fn test_bracket_matching() {
        let tokens = tokenize_file("test/lexer/mixed.fol");

        let brackets: Vec<_> = tokens.iter().filter(|(key, _)| key.is_bracket()).collect();

        let open_brackets = brackets
            .iter()
            .filter(|(key, _)| key.is_open_bracket())
            .count();
        let close_brackets = brackets
            .iter()
            .filter(|(key, _)| key.is_close_bracket())
            .count();

        println!(
            "Open brackets: {}, Close brackets: {}",
            open_brackets, close_brackets
        );
        assert_eq!(
            open_brackets, close_brackets,
            "Should have matching brackets"
        );
    }

    #[test]
    fn test_token_position_tracking() {
        let mut file_stream =
            FileStream::from_file("test/lexer/mixed.fol").expect("Should read mixed.fol");

        let mut lexer = Elements::init(&mut file_stream);

        // Test first few tokens have proper location info
        for i in 0..5 {
            match lexer.curr(false) {
                Ok(token) => {
                    let loc = token.loc();
                    assert!(loc.row() >= 1, "Row should be at least 1");
                    assert!(loc.col() >= 1, "Column should be at least 1");

                    println!(
                        "Token {}: '{}' at row {}, col {}",
                        i,
                        token.con(),
                        loc.row(),
                        loc.col()
                    );

                    if lexer.bump().is_none() {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
    }

    #[test]
    fn test_fun_error_signature_tokens() {
        let tokens = tokenize_file("test/lexer/fun_error_signature.fol");

        let significant: Vec<(KEYWORD, String)> = tokens
            .into_iter()
            .filter(|(key, _)| !key.is_space() && !key.is_eof())
            .collect();

        assert!(
            significant
                .iter()
                .any(|(k, c)| { matches!(k, KEYWORD::Keyword(BUILDIN::Fun)) && c.trim() == "fun" }),
            "Should tokenize 'fun' as buildin keyword"
        );
        assert!(
            significant
                .iter()
                .any(|(k, c)| k.is_ident() && c.trim() == "foo"),
            "Should tokenize function name as identifier"
        );
        assert!(
            significant
                .iter()
                .any(|(k, _)| matches!(k, KEYWORD::Operator(OPERATOR::Dotdotdot))),
            "Should tokenize '...' as Dotdotdot operator"
        );

        let colon_count = significant
            .iter()
            .filter(|(k, _)| matches!(k, KEYWORD::Symbol(SYMBOL::Colon)))
            .count();
        assert_eq!(colon_count, 2, "Should tokenize both ':' separators");

        assert!(
            significant
                .iter()
                .any(|(k, c)| k.is_ident() && c.trim() == "T"),
            "Should tokenize return type identifier T"
        );
        assert!(
            significant
                .iter()
                .any(|(k, c)| k.is_ident() && c.trim() == "E"),
            "Should tokenize error type identifier E"
        );
        assert!(
            significant
                .iter()
                .any(|(k, _)| matches!(k, KEYWORD::Symbol(SYMBOL::Equal))),
            "Should tokenize '='"
        );
    }
}

// Performance tests for lexer
#[cfg(test)]
mod lexer_performance_tests {
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
}

// Error handling tests
#[cfg(test)]
mod lexer_error_tests {
    use super::*;

    #[test]
    fn test_empty_file_lexing() {
        let tokens = tokenize_file("test/stream/empty.fol");

        // Should have at least EOF token
        assert!(!tokens.is_empty(), "Should have at least EOF token");

        let last_token = &tokens[tokens.len() - 1];
        assert!(last_token.0.is_eof(), "Last token should be EOF");
    }

    #[test]
    fn test_empty_file_starts_at_explicit_eof_token() {
        let mut file_stream =
            FileStream::from_file("test/stream/empty.fol").expect("Should read empty file");
        let lexer = Elements::init(&mut file_stream);
        let token = lexer
            .curr(false)
            .expect("Empty-file lexer should still expose a current token");

        assert!(token.key().is_eof(), "Empty file should start at EOF");
        assert!(
            token.loc().row() <= 1,
            "EOF location should stay explicit and stable for empty files"
        );
    }

    #[test]
    fn test_nonexistent_file_error() {
        let result = std::panic::catch_unwind(|| tokenize_file("test/lexer/nonexistent.fol"));

        assert!(result.is_err(), "Should panic on nonexistent file");
    }

    #[test]
    fn test_unterminated_string_literal_becomes_illegal_token() {
        let tokens = tokenize_file("test/lexer/unterminated_string.fol");

        assert!(
            tokens.iter().any(|(key, _)| key.is_illegal()),
            "Unterminated quoted content should surface as an illegal token"
        );
    }
}
