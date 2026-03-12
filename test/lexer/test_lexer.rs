// Comprehensive tests for fol-lexer module

use fol_lexer::{
    lexer::{stage0, stage1, stage2},
    lexer::stage3::Elements,
    token::KEYWORD,
    token::*,
};
use fol_stream::FileStream;
use fol_types::SLIDER;
use std::time::{SystemTime, UNIX_EPOCH};

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

fn tokenize_stage1_file(path: &str) -> Vec<(KEYWORD, String)> {
    let mut file_stream =
        FileStream::from_file(path).unwrap_or_else(|_| panic!("Should be able to read {}", path));
    let mut lexer = stage1::Elements::init(&mut file_stream);
    let mut tokens = Vec::new();

    let _ = lexer.bump();

    for _ in 0..10_000 {
        match lexer.curr() {
            Ok(token) => {
                let keyword = token.key().clone();
                let content = token.con().to_string();
                tokens.push((keyword.clone(), content));
                if keyword == KEYWORD::Void(VOID::EndFile) {
                    break;
                }
                if lexer.bump().is_none() {
                    break;
                }
            }
            Err(_) => break,
        }
    }

    assert!(
        tokens.len() < 10_000,
        "Stage 1 tokenization did not terminate for {}",
        path
    );

    tokens
}

fn tokenize_stage2_file(path: &str) -> Vec<(KEYWORD, String)> {
    let mut file_stream =
        FileStream::from_file(path).unwrap_or_else(|_| panic!("Should be able to read {}", path));
    let mut lexer = stage2::Elements::init(&mut file_stream);
    let mut tokens = Vec::new();

    let _ = lexer.bump();

    for _ in 0..10_000 {
        match lexer.curr(false) {
            Ok(token) => {
                let keyword = token.key().clone();
                let content = token.con().to_string();
                tokens.push((keyword.clone(), content));
                if keyword == KEYWORD::Void(VOID::EndFile) {
                    break;
                }
                if lexer.bump().is_none() {
                    break;
                }
            }
            Err(_) => break,
        }
    }

    assert!(
        tokens.len() < 10_000,
        "Stage 2 tokenization did not terminate for {}",
        path
    );

    tokens
}

fn unique_temp_root(label: &str) -> std::path::PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("System time should be after unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "fol_lexer_{}_{}_{}",
        label,
        std::process::id(),
        stamp
    ))
}

fn tokenize_folder_contents(files: &[(&str, &str)]) -> Vec<(KEYWORD, String)> {
    use std::fs;

    let temp_root = unique_temp_root("folder");
    fs::create_dir_all(&temp_root).expect("Should create temp lexer fixture dir");
    for (relative, content) in files {
        let path = temp_root.join(relative);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("Should create lexer fixture parent directories");
        }
        fs::write(&path, content).expect("Should write lexer fixture file");
    }

    let mut file_stream = FileStream::from_folder(
        temp_root
            .to_str()
            .expect("Lexer fixture folder path should be valid utf-8"),
    )
    .expect("Should create lexer stream from folder fixture");
    let mut lexer = Elements::init(&mut file_stream);
    let mut tokens = Vec::new();

    for _ in 0..10_000 {
        let token = lexer
            .curr(false)
            .expect("Lexer should expose tokens for temp folder fixture");
        let keyword = token.key();
        let content = token.con().to_string();
        tokens.push((keyword.clone(), content));
        if keyword.is_eof() {
            break;
        }
        if lexer.bump().is_none() {
            break;
        }
    }

    fs::remove_dir_all(&temp_root).ok();
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
    fn test_keyword_recognition_is_exact_case_only() {
        let tokens = tokenize_file("test/lexer/keyword_case_edges.fol");
        let significant: Vec<(KEYWORD, String)> = tokens
            .into_iter()
            .filter(|(key, _)| !key.is_space() && !key.is_eof())
            .collect();

        assert_eq!(
            significant,
            vec![
                (KEYWORD::Identifier, "Fun".to_string()),
                (KEYWORD::Keyword(BUILDIN::Fun), "fun".to_string()),
                (KEYWORD::Identifier, "WHILE".to_string()),
                (KEYWORD::Keyword(BUILDIN::While), "while".to_string()),
                (KEYWORD::Identifier, "LOG".to_string()),
                (KEYWORD::Keyword(BUILDIN::Log), "log".to_string()),
                (KEYWORD::Identifier, "Select".to_string()),
                (KEYWORD::Keyword(BUILDIN::Select), "select".to_string()),
            ],
            "Keyword recognition should remain exact-case only until the lexer contract changes intentionally"
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
    fn test_cross_file_boundaries_surface_as_explicit_void_tokens() {
        let tokens = tokenize_folder_contents(&[("a.fol", "alpha"), ("b.fol", "beta")]);
        let significant: Vec<(KEYWORD, String)> = tokens
            .into_iter()
            .filter(|(key, _)| !key.is_space() && !key.is_eof())
            .collect();

        assert_eq!(
            significant,
            vec![
                (KEYWORD::Identifier, "alpha".to_string()),
                (KEYWORD::Void(VOID::Boundary), String::new()),
                (KEYWORD::Identifier, "beta".to_string()),
            ],
            "Adjacent files should be separated by an explicit boundary token instead of a synthetic newline character"
        );
    }

    #[test]
    fn test_unterminated_quotes_stop_at_file_boundaries() {
        let tokens =
            tokenize_folder_contents(&[("a.fol", "\"unterminated"), ("b.fol", "beta")]);
        let significant: Vec<(KEYWORD, String)> = tokens
            .into_iter()
            .filter(|(key, _)| !key.is_space() && !key.is_eof())
            .collect();

        assert_eq!(
            significant,
            vec![
                (KEYWORD::Illegal, "\"unterminated".to_string()),
                (KEYWORD::Void(VOID::Boundary), String::new()),
                (KEYWORD::Identifier, "beta".to_string()),
            ],
            "Quoted spans must stop at source boundaries instead of consuming the next file"
        );
    }

    #[test]
    fn test_unterminated_backtick_comments_stop_at_file_boundaries() {
        let tokens = tokenize_folder_contents(&[("a.fol", "`unterminated"), ("b.fol", "beta")]);
        let significant: Vec<(KEYWORD, String)> = tokens
            .into_iter()
            .filter(|(key, _)| !key.is_space() && !key.is_eof())
            .collect();

        assert_eq!(
            significant,
            vec![
                (KEYWORD::Illegal, "`unterminated".to_string()),
                (KEYWORD::Void(VOID::Boundary), String::new()),
                (KEYWORD::Identifier, "beta".to_string()),
            ],
            "Backtick comments must stop at source boundaries instead of consuming the next file"
        );
    }

    #[test]
    fn test_unterminated_slash_block_comments_stop_at_file_boundaries() {
        let tokens = tokenize_folder_contents(&[("a.fol", "/*unterminated"), ("b.fol", "beta")]);
        let significant: Vec<(KEYWORD, String)> = tokens
            .into_iter()
            .filter(|(key, _)| !key.is_space() && !key.is_eof())
            .collect();

        assert_eq!(
            significant,
            vec![
                (KEYWORD::Illegal, "/*unterminated".to_string()),
                (KEYWORD::Void(VOID::Boundary), String::new()),
                (KEYWORD::Identifier, "beta".to_string()),
            ],
            "Slash block comments must stop at source boundaries instead of consuming the next file"
        );
    }

    #[test]
    fn test_identifier_number_boundaries_do_not_merge_across_files() {
        let tokens = tokenize_folder_contents(&[("a.fol", "alpha"), ("b.fol", "42")]);
        let significant: Vec<(KEYWORD, String)> = tokens
            .into_iter()
            .filter(|(key, _)| !key.is_space() && !key.is_eof())
            .collect();

        assert_eq!(
            significant,
            vec![
                (KEYWORD::Identifier, "alpha".to_string()),
                (KEYWORD::Void(VOID::Boundary), String::new()),
                (KEYWORD::Literal(LITERAL::Decimal), "42".to_string()),
            ],
            "Cross-file boundaries must keep identifiers and following numeric literals separate"
        );
    }

    #[test]
    fn test_identifier_string_boundaries_do_not_merge_across_files() {
        let tokens = tokenize_folder_contents(&[("a.fol", "alpha"), ("b.fol", "\"beta\"")]);
        let significant: Vec<(KEYWORD, String)> = tokens
            .into_iter()
            .filter(|(key, _)| !key.is_space() && !key.is_eof())
            .collect();

        assert_eq!(
            significant,
            vec![
                (KEYWORD::Identifier, "alpha".to_string()),
                (KEYWORD::Void(VOID::Boundary), String::new()),
                (KEYWORD::Literal(LITERAL::CookedQuoted), "\"beta\"".to_string()),
            ],
            "Cross-file boundaries must keep identifiers and following quoted literals separate"
        );
    }

    #[test]
    fn test_identifier_comment_boundaries_do_not_merge_across_files() {
        let tokens = tokenize_folder_contents(&[("a.fol", "alpha"), ("b.fol", "`hidden`beta")]);
        let significant: Vec<(KEYWORD, String)> = tokens
            .into_iter()
            .filter(|(key, _)| !key.is_space() && !key.is_eof())
            .collect();

        assert_eq!(
            significant,
            vec![
                (KEYWORD::Identifier, "alpha".to_string()),
                (KEYWORD::Void(VOID::Boundary), String::new()),
                (KEYWORD::Identifier, "beta".to_string()),
            ],
            "Cross-file boundaries must keep identifiers and following comment delimiters separate"
        );
    }

    #[test]
    fn test_identifier_operator_boundaries_do_not_merge_across_files() {
        let tokens = tokenize_folder_contents(&[("a.fol", "alpha"), ("b.fol", "+beta")]);
        let significant: Vec<(KEYWORD, String)> = tokens
            .into_iter()
            .filter(|(key, _)| !key.is_space() && !key.is_eof())
            .collect();

        assert_eq!(
            significant,
            vec![
                (KEYWORD::Identifier, "alpha".to_string()),
                (KEYWORD::Void(VOID::Boundary), String::new()),
                (KEYWORD::Symbol(SYMBOL::Plus), "+".to_string()),
                (KEYWORD::Identifier, "beta".to_string()),
            ],
            "Cross-file boundaries must keep identifiers and following operators separate"
        );
    }

    #[test]
    fn test_invalid_prefixed_numeric_literals_stop_at_file_boundaries() {
        let tokens = tokenize_folder_contents(&[("a.fol", "0x"), ("b.fol", "CAFE")]);
        let significant: Vec<(KEYWORD, String)> = tokens
            .into_iter()
            .filter(|(key, _)| !key.is_space() && !key.is_eof())
            .collect();

        assert_eq!(
            significant,
            vec![
                (KEYWORD::Illegal, "0x".to_string()),
                (KEYWORD::Void(VOID::Boundary), String::new()),
                (KEYWORD::Identifier, "CAFE".to_string()),
            ],
            "Malformed prefixed numerics must stop at a file boundary instead of consuming the next file's content"
        );
    }

    #[test]
    fn test_invalid_decimal_literals_stop_at_file_boundaries() {
        let tokens = tokenize_folder_contents(&[("a.fol", "1_"), ("b.fol", "2")]);
        let significant: Vec<(KEYWORD, String)> = tokens
            .into_iter()
            .filter(|(key, _)| !key.is_space() && !key.is_eof())
            .collect();

        assert_eq!(
            significant,
            vec![
                (KEYWORD::Illegal, "1_".to_string()),
                (KEYWORD::Void(VOID::Boundary), String::new()),
                (KEYWORD::Literal(LITERAL::Decimal), "2".to_string()),
            ],
            "Malformed decimal literals must stop at a file boundary instead of consuming the next file's content"
        );
    }

    #[test]
    fn test_cross_file_boundaries_keep_second_file_locations_real() {
        use std::fs;

        let temp_root = unique_temp_root("boundary_locations");
        fs::create_dir_all(&temp_root).expect("Should create temp lexer fixture dir");
        fs::write(temp_root.join("a.fol"), "alpha").expect("Should write first fixture file");
        fs::write(temp_root.join("b.fol"), "beta").expect("Should write second fixture file");

        let mut file_stream = FileStream::from_folder(
            temp_root
                .to_str()
                .expect("Lexer fixture folder path should be valid utf-8"),
        )
        .expect("Should create lexer stream from folder fixture");
        let mut lexer = Elements::init(&mut file_stream);

        let first = lexer
            .curr(false)
            .expect("Lexer should expose the first file token");
        assert_eq!(first.key(), KEYWORD::Identifier);
        assert_eq!(first.con(), "alpha");
        assert_eq!(first.loc().row(), 1);
        assert_eq!(first.loc().col(), 1);
        assert!(
            first
                .loc()
                .source()
                .expect("First token should carry a source path")
                .path(false)
                .ends_with("a.fol"),
            "First token should stay anchored to the first file"
        );

        lexer.bump();
        let boundary = lexer
            .curr(false)
            .expect("Lexer should expose the explicit file boundary token");
        assert_eq!(boundary.key(), KEYWORD::Void(VOID::Boundary));
        assert_eq!(boundary.loc().row(), 1);
        assert_eq!(boundary.loc().col(), 0);
        assert!(
            boundary
                .loc()
                .source()
                .expect("Boundary token should still identify the incoming file")
                .path(false)
                .ends_with("b.fol"),
            "Boundary token should point at the incoming file without pretending to be a real source character"
        );

        lexer.bump();
        let second = lexer
            .curr(false)
            .expect("Lexer should expose the second file token after the boundary");
        assert_eq!(second.key(), KEYWORD::Identifier);
        assert_eq!(second.con(), "beta");
        assert_eq!(second.loc().row(), 1);
        assert_eq!(second.loc().col(), 1);
        assert!(
            second
                .loc()
                .source()
                .expect("Second token should carry a source path")
                .path(false)
                .ends_with("b.fol"),
            "Second-file tokens must keep their real source path after the explicit boundary token"
        );

        fs::remove_dir_all(&temp_root).ok();
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
                (KEYWORD::Literal(LITERAL::Hexadecimal), "0X1A".to_string()),
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
                (KEYWORD::Literal(LITERAL::Decimal), "1_000".to_string()),
                (KEYWORD::Literal(LITERAL::Hexadecimal), "0xCA_FE".to_string()),
                (KEYWORD::Literal(LITERAL::Octal), "0o7_7".to_string()),
                (KEYWORD::Literal(LITERAL::Binary), "0b1010_0001".to_string()),
                (KEYWORD::Literal(LITERAL::Hexadecimal), "0XCA_FE".to_string()),
                (KEYWORD::Literal(LITERAL::Octal), "0O7_7".to_string()),
                (KEYWORD::Literal(LITERAL::Binary), "0B1010_0001".to_string()),
            ],
            "Numeric literal payloads should preserve source spelling for supported forms"
        );
    }

    #[test]
    fn test_token_payloads_follow_the_front_end_contract() {
        let tokens = tokenize_file("test/lexer/payload_shapes.fol");

        assert!(
            tokens
                .iter()
                .any(|(key, content)| matches!(key, KEYWORD::Keyword(BUILDIN::Var)) && content == "var"),
            "Keywords should keep their source spelling as payload"
        );
        assert!(
            tokens
                .iter()
                .any(|(key, content)| key.is_ident() && content == "name"),
            "Identifiers should keep their source spelling as payload"
        );
        assert!(
            tokens
                .iter()
                .any(|(key, content)| matches!(key, KEYWORD::Symbol(SYMBOL::Equal)) && content == "="),
            "Single-character symbols should keep their source spelling as payload"
        );
        assert!(
            tokens
                .iter()
                .any(|(key, content)| matches!(key, KEYWORD::Operator(OPERATOR::Addeq)) && content == "+="),
            "Folded operators should keep their combined source spelling as payload"
        );
        assert!(
            tokens.iter().any(|(key, content)| {
                matches!(key, KEYWORD::Literal(LITERAL::Decimal)) && content == "42"
            }),
            "Numeric literals should keep their source spelling as payload"
        );
        assert!(
            tokens.iter().any(|(key, content)| {
                matches!(key, KEYWORD::Literal(LITERAL::CookedQuoted)) && content == "\"hi\""
            }),
            "Quoted literals should keep delimiters in payload"
        );
        assert!(
            tokens
                .iter()
                .filter(|(key, _)| key.is_void() && !key.is_eof())
                .all(|(_, content)| content == " "),
            "Ignorable separators should normalize to a single-space payload"
        );
        let eof_payload = &tokens.last().expect("Token stream should end with EOF").1;
        assert!(
            eof_payload.ends_with('\0'),
            "EOF payload should always retain the explicit sentinel"
        );
        assert!(
            eof_payload.trim_end_matches('\0').chars().all(|ch| ch == ' '),
            "Any content before the EOF sentinel should only be normalized trailing separator payload"
        );
    }

    #[test]
    fn test_supported_numeric_families_tokenize_with_expected_kinds() {
        let tokens = tokenize_file("test/lexer/numeric_families.fol");
        let significant: Vec<(KEYWORD, String)> = tokens
            .into_iter()
            .filter(|(key, _)| !key.is_space() && !key.is_eof())
            .collect();

        assert_eq!(
            significant,
            vec![
                (KEYWORD::Literal(LITERAL::Decimal), "42".to_string()),
                (KEYWORD::Literal(LITERAL::Float), "3.14".to_string()),
                (KEYWORD::Literal(LITERAL::Float), ".5".to_string()),
                (KEYWORD::Literal(LITERAL::Hexadecimal), "0x1A".to_string()),
                (KEYWORD::Literal(LITERAL::Octal), "0o777".to_string()),
                (KEYWORD::Literal(LITERAL::Binary), "0b1010".to_string()),
            ],
            "Decimal, float, leading-dot float, hex, octal, and binary literals should keep their expected token kinds"
        );
    }

    #[test]
    fn test_underscored_numeric_families_preserve_payload_spelling() {
        let tokens = tokenize_file("test/lexer/underscored_numeric_families.fol");
        let significant: Vec<(KEYWORD, String)> = tokens
            .into_iter()
            .filter(|(key, _)| !key.is_space() && !key.is_eof())
            .collect();

        assert_eq!(
            significant,
            vec![
                (KEYWORD::Literal(LITERAL::Decimal), "1_000".to_string()),
                (KEYWORD::Literal(LITERAL::Float), "12_3.4_5".to_string()),
                (KEYWORD::Literal(LITERAL::Hexadecimal), "0xCA_FE".to_string()),
                (KEYWORD::Literal(LITERAL::Octal), "0o7_7".to_string()),
                (KEYWORD::Literal(LITERAL::Binary), "0b1010_0001".to_string()),
            ],
            "Supported underscored numeric families should preserve their original source spelling at the lexer boundary"
        );
    }

    #[test]
    fn test_invalid_hex_literals_become_single_illegal_tokens() {
        let tokens = tokenize_file("test/lexer/invalid_hex_literals.fol");
        let significant: Vec<(KEYWORD, String)> = tokens
            .into_iter()
            .filter(|(key, _)| !key.is_space() && !key.is_eof())
            .collect();

        assert_eq!(
            significant,
            vec![
                (KEYWORD::Illegal, "0x".to_string()),
                (KEYWORD::Illegal, "0x_1".to_string()),
                (KEYWORD::Illegal, "0x1_".to_string()),
                (KEYWORD::Illegal, "0x1G".to_string()),
                (KEYWORD::Illegal, "0x1__2".to_string()),
                (KEYWORD::Illegal, "0X_FF".to_string()),
            ],
            "Malformed hexadecimal literals should stay one illegal token instead of splitting into partial numeric and trailing junk"
        );
    }

    #[test]
    fn test_invalid_octal_literals_become_single_illegal_tokens() {
        let tokens = tokenize_file("test/lexer/invalid_octal_literals.fol");
        let significant: Vec<(KEYWORD, String)> = tokens
            .into_iter()
            .filter(|(key, _)| !key.is_space() && !key.is_eof())
            .collect();

        assert_eq!(
            significant,
            vec![
                (KEYWORD::Illegal, "0o".to_string()),
                (KEYWORD::Illegal, "0o_1".to_string()),
                (KEYWORD::Illegal, "0o7_".to_string()),
                (KEYWORD::Illegal, "0o78".to_string()),
                (KEYWORD::Illegal, "0o1__2".to_string()),
                (KEYWORD::Illegal, "0O_77".to_string()),
            ],
            "Malformed octal literals should stay one illegal token instead of splitting into partial numeric and trailing junk"
        );
    }

    #[test]
    fn test_invalid_binary_literals_become_single_illegal_tokens() {
        let tokens = tokenize_file("test/lexer/invalid_binary_literals.fol");
        let significant: Vec<(KEYWORD, String)> = tokens
            .into_iter()
            .filter(|(key, _)| !key.is_space() && !key.is_eof())
            .collect();

        assert_eq!(
            significant,
            vec![
                (KEYWORD::Illegal, "0b".to_string()),
                (KEYWORD::Illegal, "0b_1".to_string()),
                (KEYWORD::Illegal, "0b1_".to_string()),
                (KEYWORD::Illegal, "0b102".to_string()),
                (KEYWORD::Illegal, "0b10__01".to_string()),
                (KEYWORD::Illegal, "0B_1".to_string()),
            ],
            "Malformed binary literals should stay one illegal token instead of splitting into partial numeric and trailing junk"
        );
    }

    #[test]
    fn test_invalid_decimal_literals_become_single_illegal_tokens() {
        let tokens = tokenize_file("test/lexer/invalid_decimal_literals.fol");
        let significant: Vec<(KEYWORD, String)> = tokens
            .into_iter()
            .filter(|(key, _)| !key.is_space() && !key.is_eof())
            .collect();

        assert_eq!(
            significant,
            vec![
                (KEYWORD::Illegal, "1_".to_string()),
                (KEYWORD::Illegal, "1__2".to_string()),
                (KEYWORD::Illegal, "12__".to_string()),
                (KEYWORD::Illegal, "0__0".to_string()),
            ],
            "Malformed decimal literals should stay one illegal token instead of silently accepting repeated or trailing underscores"
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
    fn test_trailing_dot_float_tokenizes_as_float() {
        let tokens = tokenize_file("test/lexer/trailing_dot_float.fol");
        let significant: Vec<(KEYWORD, String)> = tokens
            .into_iter()
            .filter(|(key, _)| !key.is_space() && !key.is_eof())
            .collect();

        assert_eq!(
            significant,
            vec![(KEYWORD::Literal(LITERAL::Float), "1.".to_string())],
            "A decimal literal followed by a trailing dot should tokenize as a float"
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
                (KEYWORD::Literal(LITERAL::Decimal), "42".to_string()),
                (KEYWORD::Symbol(SYMBOL::Minus), "-".to_string()),
                (KEYWORD::Literal(LITERAL::Float), "3.5".to_string()),
            ],
            "Minus should remain a separate token so unary negation stays parser-level"
        );
    }

    #[test]
    fn test_imaginary_suffixes_remain_out_of_scope_for_numeric_tokenization() {
        let tokens = tokenize_file("test/lexer/imaginary_out_of_scope.fol");
        let significant: Vec<(KEYWORD, String)> = tokens
            .into_iter()
            .filter(|(key, _)| !key.is_space() && !key.is_eof())
            .collect();

        assert_eq!(
            significant,
            vec![
                (KEYWORD::Literal(LITERAL::Decimal), "1".to_string()),
                (KEYWORD::Identifier, "i".to_string()),
                (KEYWORD::Literal(LITERAL::Float), "3.5".to_string()),
                (KEYWORD::Identifier, "i".to_string()),
            ],
            "Imaginary-unit suffixes should stay outside the supported numeric families for now"
        );
    }

    #[test]
    fn test_backticks_are_ignorable_comments() {
        let tokens = tokenize_file("test/lexer/backticks.fol");
        let significant: Vec<(KEYWORD, String)> = tokens
            .into_iter()
            .filter(|(key, _)| !key.is_space() && !key.is_eof())
            .collect();

        assert_eq!(
            significant,
            Vec::<(KEYWORD, String)>::new(),
            "Backtick-delimited comments should be fully ignorable at the parser-facing lexer boundary"
        );
    }

    #[test]
    fn test_stage1_backticks_are_classified_as_comment_tokens() {
        let tokens = tokenize_stage1_file("test/lexer/backticks.fol");
        let comments: Vec<(KEYWORD, String)> = tokens
            .into_iter()
            .filter(|(key, _)| key.is_comment())
            .collect();

        assert_eq!(
            comments,
            vec![(KEYWORD::Comment(COMMENT::Backtick), "`macroish`".to_string(),)],
            "Stage 1 should classify backtick-delimited spans explicitly before later stages normalize them away"
        );
    }

    #[test]
    fn test_stage1_doc_comments_are_classified_separately() {
        let tokens = tokenize_stage1_file("test/lexer/doc_comments.fol");
        let comments: Vec<(KEYWORD, String)> = tokens
            .into_iter()
            .filter(|(key, _)| key.is_comment())
            .collect();

        assert_eq!(
            comments,
            vec![
                (
                    KEYWORD::Comment(COMMENT::Doc),
                    "`[doc] module docs`".to_string(),
                ),
                (
                    KEYWORD::Comment(COMMENT::Doc),
                    "`[doc] block docs`".to_string(),
                ),
            ],
            "Stage 1 should detect the book's [doc] prefix explicitly even while doc comments stay deferred later in the pipeline"
        );
    }

    #[test]
    fn test_stage1_slash_line_comments_use_compatibility_comment_kind() {
        let tokens = tokenize_stage1_file("test/lexer/slash_line_comments.fol");
        let comments: Vec<(KEYWORD, String)> = tokens
            .into_iter()
            .filter(|(key, _)| key.is_comment())
            .collect();

        assert_eq!(
            comments,
            vec![
                (
                    KEYWORD::Comment(COMMENT::SlashLine),
                    "// compatibility line comment".to_string(),
                ),
                (
                    KEYWORD::Comment(COMMENT::SlashLine),
                    "// trailing compatibility comment".to_string(),
                ),
            ],
            "Slash line comments should stay on an explicit compatibility-only internal comment kind"
        );
    }

    #[test]
    fn test_stage1_slash_block_comments_use_compatibility_comment_kind() {
        let tokens = tokenize_stage1_file("test/lexer/slash_block_comments.fol");
        let comments: Vec<(KEYWORD, String)> = tokens
            .into_iter()
            .filter(|(key, _)| key.is_comment())
            .collect();

        assert_eq!(
            comments,
            vec![(
                KEYWORD::Comment(COMMENT::SlashBlock),
                "/* compatibility\n   block comment */".to_string(),
            )],
            "Slash block comments should stay on an explicit compatibility-only internal comment kind with an exact span"
        );
    }

    #[test]
    fn test_stage2_normalizes_backtick_and_doc_comments_back_to_void_tokens() {
        let backtick_tokens = tokenize_stage2_file("test/lexer/backticks.fol");
        let doc_tokens = tokenize_stage2_file("test/lexer/doc_comments.fol");

        assert!(
            backtick_tokens.iter().any(|(key, content)| key.is_space() && content == " "),
            "Stage 2 should collapse ordinary backtick comments back to normalized void separators"
        );
        assert!(
            !backtick_tokens.iter().any(|(key, _)| key.is_comment()),
            "Stage 2 should not leak backtick comment tokens past the internal classification boundary"
        );
        assert!(
            doc_tokens
                .iter()
                .filter(|(key, content)| key.is_space() && content == " ")
                .count()
                >= 2,
            "Stage 2 should also collapse doc comments back to normalized void separators"
        );
        assert!(
            !doc_tokens.iter().any(|(key, _)| key.is_comment()),
            "Stage 2 should not leak doc comment tokens past the internal classification boundary"
        );
    }

    #[test]
    fn test_stage1_multiline_backtick_comments_preserve_exact_span() {
        let tokens = tokenize_stage1_file("test/lexer/backtick_multiline_comments.fol");
        let comments: Vec<(KEYWORD, String)> = tokens
            .into_iter()
            .filter(|(key, _)| key.is_comment())
            .collect();

        assert_eq!(
            comments,
            vec![(
                KEYWORD::Comment(COMMENT::Backtick),
                "`note one\nnote two`".to_string(),
            )],
            "Stage 1 should preserve the exact multiline backtick span before later stages normalize it away"
        );
    }

    #[test]
    fn test_multiline_backtick_comments_remain_parser_ignorable() {
        let tokens = tokenize_file("test/lexer/backtick_multiline_comments.fol");
        let significant: Vec<(KEYWORD, String)> = tokens
            .into_iter()
            .filter(|(key, _)| !key.is_void() && !key.is_eof())
            .collect();

        assert_eq!(
            significant,
            vec![
                (KEYWORD::Keyword(BUILDIN::Var), "var".to_string()),
                (KEYWORD::Identifier, "after".to_string()),
                (KEYWORD::Symbol(SYMBOL::Equal), "=".to_string()),
                (KEYWORD::Literal(LITERAL::Decimal), "1".to_string()),
                (KEYWORD::Symbol(SYMBOL::Semi), "; ".to_string()),
            ],
            "Multiline backtick comments should stay ignorable in the parser-facing lexer output"
        );
    }

    #[test]
    fn test_quoted_payloads_preserve_escape_spelling_without_validation() {
        let tokens = tokenize_file("test/lexer/escape_payloads.fol");
        let significant: Vec<(KEYWORD, String)> = tokens
            .into_iter()
            .filter(|(key, _)| !key.is_space() && !key.is_eof())
            .collect();

        assert_eq!(
            significant,
            vec![
                (KEYWORD::Literal(LITERAL::CookedQuoted), "\"line\\n\"".to_string()),
                (KEYWORD::Literal(LITERAL::CookedQuoted), "\"quote\\\"\"".to_string()),
                (KEYWORD::Literal(LITERAL::CookedQuoted), "\"bad\\q\"".to_string()),
            ],
            "Cooked quoted payloads should preserve both conventional and unknown escape spellings verbatim at the lexer boundary"
        );
    }

    #[test]
    fn test_quoted_payloads_keep_physical_newlines_without_continuation_semantics() {
        let temp_path = std::env::temp_dir().join(format!(
            "fol_lexer_multiline_quote_{}_{}.fol",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("System time should be after unix epoch")
                .as_nanos()
        ));
        std::fs::write(&temp_path, "\"line one\nline two\"")
            .expect("Should write multiline quoted lexer fixture");

        let tokens = tokenize_file(
            temp_path
                .to_str()
                .expect("Multiline quoted fixture path should be valid utf-8"),
        );
        let significant: Vec<(KEYWORD, String)> = tokens
            .into_iter()
            .filter(|(key, _)| !key.is_space() && !key.is_eof())
            .collect();

        assert_eq!(
            significant,
            vec![(
                KEYWORD::Literal(LITERAL::CookedQuoted),
                "\"line one\nline two\"".to_string()
            )],
            "Cooked quoted content should keep physical newlines inside the token payload instead of using a special continuation rule"
        );

        std::fs::remove_file(&temp_path).ok();
    }

    #[test]
    fn test_quoted_literal_payloads_keep_delimiters() {
        let tokens = tokenize_file("test/lexer/literals.fol");

        assert!(
            tokens.iter().any(|(key, content)| {
                matches!(key, KEYWORD::Literal(LITERAL::CookedQuoted)) && content == "\"hello\""
            }),
            "String literal payload should keep its double-quote delimiters"
        );
        assert!(
            tokens.iter().any(|(key, content)| {
                matches!(key, KEYWORD::Literal(LITERAL::RawQuoted)) && content == "'c'"
            }),
            "Single-quoted literal payload should keep its delimiters on its own token family"
        );
    }

    #[test]
    fn test_single_and_double_quotes_no_longer_share_one_literal_kind() {
        let tokens = tokenize_file("test/lexer/literals.fol");

        assert!(
            tokens.iter().any(|(key, content)| {
                matches!(key, KEYWORD::Literal(LITERAL::CookedQuoted)) && content == "\"hello\""
            }),
            "Double-quoted text should stay on the string token family"
        );
        assert!(
            tokens.iter().any(|(key, content)| {
                matches!(key, KEYWORD::Literal(LITERAL::RawQuoted)) && content == "'c'"
            }),
            "Single-quoted text should no longer be conflated with double-quoted text"
        );
    }

    #[test]
    fn test_stage1_cooked_and_raw_quotes_follow_different_delimiter_rules() {
        let tokens = tokenize_stage1_file("test/lexer/cooked_raw_quote_boundaries.fol");
        let significant: Vec<(KEYWORD, String)> = tokens
            .into_iter()
            .filter(|(key, _)| !key.is_void())
            .collect();

        assert_eq!(
            significant,
            vec![
                (
                    KEYWORD::Literal(LITERAL::CookedQuoted),
                    "\"a\\\" b\"".to_string(),
                ),
                (KEYWORD::Literal(LITERAL::RawQuoted), "'a\\'".to_string()),
                (KEYWORD::Literal(LITERAL::RawQuoted), "'b'".to_string()),
            ],
            "Cooked quotes should treat backslash-delimiter pairs as escaped content, while raw quotes should stop at the next single quote"
        );
    }

    #[test]
    fn test_literal_family_fixtures_keep_distinct_token_families() {
        let cooked = tokenize_file("test/parser/simple_literal_cooked_family.fol");
        let raw = tokenize_file("test/parser/simple_literal_raw_family.fol");

        let cooked_significant: Vec<(KEYWORD, String)> = cooked
            .into_iter()
            .filter(|(key, _)| !key.is_void())
            .collect();
        let raw_significant: Vec<(KEYWORD, String)> = raw
            .into_iter()
            .filter(|(key, _)| !key.is_void())
            .collect();

        assert_eq!(
            cooked_significant,
            vec![
                (KEYWORD::Literal(LITERAL::CookedQuoted), "\"a\"".to_string()),
                (
                    KEYWORD::Literal(LITERAL::CookedQuoted),
                    "\"beta\"".to_string(),
                ),
            ],
            "Cooked-family parser fixtures should still surface as cooked quoted tokens before parser lowering"
        );
        assert_eq!(
            raw_significant,
            vec![
                (KEYWORD::Literal(LITERAL::RawQuoted), "'z'".to_string()),
                (KEYWORD::Literal(LITERAL::RawQuoted), "'omega'".to_string()),
            ],
            "Raw-family parser fixtures should still surface as raw quoted tokens before parser lowering"
        );
    }

    #[test]
    fn test_cooked_fixture_payloads_preserve_multiline_and_escape_spelling() {
        let multiline = tokenize_file("test/parser/simple_literal_multiline_cooked.fol");
        let escapes = tokenize_file("test/parser/simple_literal_cooked_escape_quotes.fol");
        let unicode = tokenize_file("test/parser/simple_literal_cooked_unicode_escapes.fol");

        let multiline_significant: Vec<(KEYWORD, String)> = multiline
            .into_iter()
            .filter(|(key, _)| !key.is_void())
            .collect();
        let escape_significant: Vec<(KEYWORD, String)> = escapes
            .into_iter()
            .filter(|(key, _)| !key.is_void())
            .collect();
        let unicode_significant: Vec<(KEYWORD, String)> = unicode
            .into_iter()
            .filter(|(key, _)| !key.is_void())
            .collect();

        assert_eq!(
            multiline_significant,
            vec![(
                KEYWORD::Literal(LITERAL::CookedQuoted),
                "\"foo\\\n    bar\"".to_string(),
            )],
            "Multiline cooked fixtures should keep the physical continuation spelling in the lexer payload"
        );
        assert_eq!(
            escape_significant,
            vec![
                (
                    KEYWORD::Literal(LITERAL::CookedQuoted),
                    "\"say \\\"hi\\\"\"".to_string(),
                ),
                (
                    KEYWORD::Literal(LITERAL::CookedQuoted),
                    "\"\\\\path\"".to_string(),
                ),
            ],
            "Cooked quote/backslash fixtures should preserve the source escape spelling until parser lowering"
        );
        assert_eq!(
            unicode_significant,
            vec![
                (
                    KEYWORD::Literal(LITERAL::CookedQuoted),
                    "\"\\65\"".to_string(),
                ),
                (
                    KEYWORD::Literal(LITERAL::CookedQuoted),
                    "\"\\x41\"".to_string(),
                ),
                (
                    KEYWORD::Literal(LITERAL::CookedQuoted),
                    "\"\\u0041\"".to_string(),
                ),
                (
                    KEYWORD::Literal(LITERAL::CookedQuoted),
                    "\"\\u{41}\"".to_string(),
                ),
            ],
            "Cooked unicode fixtures should preserve each raw escape spelling in the lexer payload"
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
                (KEYWORD::Literal(LITERAL::Decimal), "5".to_string()),
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
    fn test_doc_comments_are_deferred_with_normal_comments() {
        let tokens = tokenize_file("test/lexer/doc_comments.fol");
        let significant: Vec<(KEYWORD, String)> = tokens
            .into_iter()
            .filter(|(key, _)| !key.is_void() && !key.is_comment())
            .collect();

        assert_eq!(
            significant,
            vec![
                (KEYWORD::Keyword(BUILDIN::Var), "var".to_string()),
                (KEYWORD::Identifier, "alpha".to_string()),
                (KEYWORD::Symbol(SYMBOL::Equal), "=".to_string()),
                (KEYWORD::Literal(LITERAL::Decimal), "1".to_string()),
                (KEYWORD::Keyword(BUILDIN::Var), "var".to_string()),
                (KEYWORD::Identifier, "beta".to_string()),
                (KEYWORD::Symbol(SYMBOL::Equal), "=".to_string()),
                (KEYWORD::Literal(LITERAL::Decimal), "2".to_string()),
            ],
            "Backtick doc comments should stay deferred and should not surface as a parser-visible token family yet"
        );
    }

    #[test]
    fn test_comment_delimiters_inside_literals_do_not_start_comments() {
        let tokens = tokenize_file("test/lexer/comment_delimiters_in_literals.fol");
        let significant: Vec<(KEYWORD, String)> = tokens
            .into_iter()
            .filter(|(key, _)| !key.is_void() && !key.is_eof())
            .collect();

        assert_eq!(
            significant,
            vec![
                (
                    KEYWORD::Literal(LITERAL::CookedQuoted),
                    "\"literal with `backticks` and // slash\"".to_string(),
                ),
                (
                    KEYWORD::Literal(LITERAL::RawQuoted),
                    "'quoted with /* block */ and `ticks`'".to_string(),
                ),
                (KEYWORD::Keyword(BUILDIN::Var), "var".to_string()),
                (KEYWORD::Identifier, "done".to_string()),
                (KEYWORD::Symbol(SYMBOL::Equal), "=".to_string()),
                (KEYWORD::Literal(LITERAL::Decimal), "1".to_string()),
                (KEYWORD::Symbol(SYMBOL::Semi), "; ".to_string()),
            ],
            "Comment delimiters inside quoted literals should stay literal payload, not open comments"
        );
    }

    #[test]
    fn test_slash_line_comments_remain_supported_as_compatibility_comments() {
        let tokens = tokenize_file("test/lexer/slash_line_comments.fol");
        let significant: Vec<(KEYWORD, String)> = tokens
            .into_iter()
            .filter(|(key, _)| !key.is_void() && !key.is_eof())
            .collect();

        assert_eq!(
            significant,
            vec![
                (KEYWORD::Keyword(BUILDIN::Var), "var".to_string()),
                (KEYWORD::Identifier, "alpha".to_string()),
                (KEYWORD::Symbol(SYMBOL::Equal), "=".to_string()),
                (KEYWORD::Literal(LITERAL::Decimal), "1".to_string()),
                (KEYWORD::Symbol(SYMBOL::Semi), "; ".to_string()),
            ],
            "Slash line comments remain a compatibility comment surface and should stay ignorable"
        );
    }

    #[test]
    fn test_slash_block_comments_remain_supported_as_compatibility_comments() {
        let tokens = tokenize_file("test/lexer/slash_block_comments.fol");
        let significant: Vec<(KEYWORD, String)> = tokens
            .into_iter()
            .filter(|(key, _)| !key.is_void() && !key.is_eof())
            .collect();

        assert_eq!(
            significant,
            vec![
                (KEYWORD::Keyword(BUILDIN::Var), "var".to_string()),
                (KEYWORD::Identifier, "beta".to_string()),
                (KEYWORD::Symbol(SYMBOL::Equal), "=".to_string()),
                (KEYWORD::Literal(LITERAL::Decimal), "2".to_string()),
                (KEYWORD::Symbol(SYMBOL::Semi), "; ".to_string()),
            ],
            "Slash block comments remain a compatibility comment surface and should stay ignorable"
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
    fn test_stage0_synthetic_eof_uses_explicit_out_of_band_location() {
        let mut file_stream =
            FileStream::from_file("test/stream/basic.fol").expect("Should read basic file");
        let mut chars = stage0::Elements::init(&mut file_stream);
        let mut eof = None;

        for _ in 0..10_000 {
            let part = chars
                .bump()
                .expect("Stage0 should expose a part while scanning to EOF")
                .expect("Stage0 should not fail for a basic fixture");
            if part.0 == '\0' {
                eof = Some(part.1);
                break;
            }
        }

        let eof = eof.expect("Stage0 should eventually reach its synthetic EOF marker");
        assert_eq!(
            eof.row(),
            1,
            "Synthetic stage0 EOF should use the explicit row chosen in the source generator"
        );
        assert_eq!(
            eof.col(),
            0,
            "Synthetic stage0 EOF should remain out-of-band instead of pretending to be a real character location"
        );
    }

    #[test]
    fn test_stage0_window_stays_bounded_while_draining() {
        let mut file_stream =
            FileStream::from_file("test/stream/basic.fol").expect("Should read basic file");
        let mut chars = stage0::Elements::init(&mut file_stream);
        let mut saw_eof = false;

        for _ in 0..10_000 {
            assert_eq!(chars.prev_vec().len(), SLIDER);
            assert_eq!(chars.next_vec().len(), SLIDER);

            let Some(part) = chars.bump() else {
                break;
            };
            let part = part.expect("Stage0 should not fail while draining a basic fixture");
            if part.0 == '\0' {
                saw_eof = true;
            }
        }

        assert!(saw_eof, "Stage0 should drain through its explicit EOF marker");
        assert_eq!(chars.prev_vec().len(), SLIDER);
        assert_eq!(chars.next_vec().len(), SLIDER);
        assert!(
            chars.bump().is_none(),
            "Stage0 should terminate cleanly once its bounded window is fully drained"
        );
    }

    #[test]
    fn test_stage0_emits_explicit_file_boundaries_with_real_second_file_locations() {
        use std::fs;

        let temp_root = unique_temp_root("stage0_boundaries");
        fs::create_dir_all(&temp_root).expect("Should create temp stage0 fixture dir");
        fs::write(temp_root.join("a.fol"), "a").expect("Should write first stage0 fixture");
        fs::write(temp_root.join("b.fol"), "b").expect("Should write second stage0 fixture");

        let mut file_stream = FileStream::from_folder(
            temp_root
                .to_str()
                .expect("Stage0 fixture folder path should be valid utf-8"),
        )
        .expect("Should create stage0 stream from folder fixture");
        let mut chars = stage0::Elements::init(&mut file_stream);
        let mut seen = Vec::new();

        for _ in 0..10_000 {
            let Some(part) = chars.bump() else {
                break;
            };
            let part = part.expect("Stage0 should not fail for multi-file boundary fixture");
            seen.push(part);
            if seen.len() >= 3 {
                break;
            }
        }

        assert_eq!(seen.len(), 3, "Stage0 should expose first char, boundary, second char");

        assert_eq!(seen[0].0, 'a');
        assert_eq!(seen[0].1.row(), 1);
        assert_eq!(seen[0].1.col(), 1);
        assert!(
            seen[0]
                .1
                .source()
                .expect("First char should keep a source path")
                .path(false)
                .ends_with("a.fol"),
            "First character should remain anchored to the first file"
        );

        assert_eq!(seen[1].0, stage0::SOURCE_BOUNDARY_CHAR);
        assert_eq!(seen[1].1.row(), 1);
        assert_eq!(seen[1].1.col(), 0);
        assert!(
            seen[1]
                .1
                .source()
                .expect("Boundary should carry the incoming file path")
                .path(false)
                .ends_with("b.fol"),
            "Boundary marker should stay anchored to the incoming file instead of pretending to belong to the previous one"
        );

        assert_eq!(seen[2].0, 'b');
        assert_eq!(seen[2].1.row(), 1);
        assert_eq!(seen[2].1.col(), 1);
        assert!(
            seen[2]
                .1
                .source()
                .expect("Second char should keep a source path")
                .path(false)
                .ends_with("b.fol"),
            "Second character should remain anchored to the second file"
        );
    }

    #[test]
    fn test_stage1_window_stays_bounded_while_draining() {
        let mut file_stream =
            FileStream::from_file("test/stream/basic.fol").expect("Should read basic file");
        let mut elements = stage1::Elements::init(&mut file_stream);
        let mut saw_eof = false;

        for _ in 0..10_000 {
            assert_eq!(elements.prev_vec().len(), SLIDER);
            assert_eq!(elements.next_vec().len(), SLIDER);

            let Some(part) = elements.bump() else {
                break;
            };
            let part = part.expect("Stage1 should not fail while draining a basic fixture");
            if part.key().is_eof() {
                saw_eof = true;
            }
        }

        assert!(saw_eof, "Stage1 should drain through its explicit EOF token");
        assert_eq!(elements.prev_vec().len(), SLIDER);
        assert_eq!(elements.next_vec().len(), SLIDER);
        assert!(
            elements.bump().is_none(),
            "Stage1 should terminate cleanly once its bounded window is fully drained"
        );
    }

    #[test]
    fn test_stage2_window_stays_bounded_while_draining() {
        let mut file_stream =
            FileStream::from_file("test/stream/basic.fol").expect("Should read basic file");
        let mut elements = stage2::Elements::init(&mut file_stream);
        let mut saw_eof = false;

        for _ in 0..10_000 {
            assert_eq!(elements.prev_vec().len(), SLIDER);
            assert_eq!(elements.next_vec().len(), SLIDER);

            let Some(part) = elements.bump() else {
                break;
            };
            let part = part.expect("Stage2 should not fail while draining a basic fixture");
            if part.key().is_eof() {
                saw_eof = true;
            }
        }

        assert!(saw_eof, "Stage2 should drain through its explicit EOF token");
        assert_eq!(elements.prev_vec().len(), SLIDER);
        assert_eq!(elements.next_vec().len(), SLIDER);
        assert!(
            elements.bump().is_none(),
            "Stage2 should terminate cleanly once its bounded window is fully drained"
        );
    }

    #[test]
    fn test_stage3_window_stays_bounded_while_draining() {
        let mut file_stream =
            FileStream::from_file("test/stream/basic.fol").expect("Should read basic file");
        let mut elements = Elements::init(&mut file_stream);
        let mut saw_eof = false;

        for _ in 0..10_000 {
            assert_eq!(elements.prev_vec().len(), SLIDER);
            assert_eq!(elements.next_vec().len(), SLIDER);

            let Some(part) = elements.bump() else {
                break;
            };
            let part = part.expect("Stage3 should not fail while draining a basic fixture");
            if part.key().is_eof() {
                saw_eof = true;
            }
        }

        assert!(saw_eof, "Stage3 should drain through its explicit EOF token");
        assert_eq!(elements.prev_vec().len(), SLIDER);
        assert_eq!(elements.next_vec().len(), SLIDER);
        assert!(
            elements.bump().is_none(),
            "Stage3 should terminate cleanly once its bounded window is fully drained"
        );
    }

    #[test]
    fn test_nonexistent_file_error() {
        let result = std::panic::catch_unwind(|| tokenize_file("test/lexer/nonexistent.fol"));

        assert!(result.is_err(), "Should panic on nonexistent file");
    }

    #[test]
    fn test_unrecognized_non_ascii_character_returns_lexer_error() {
        let temp_path = std::env::temp_dir().join(format!(
            "fol_lexer_bad_char_{}_{}.fol",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("System time should be after unix epoch")
                .as_nanos()
        ));
        std::fs::write(&temp_path, "é").expect("Should write malformed lexer fixture");

        let mut file_stream = FileStream::from_file(
            temp_path
                .to_str()
                .expect("Malformed lexer fixture path should be valid utf-8"),
        )
        .expect("Should open malformed lexer fixture");
        let lexer = Elements::init(&mut file_stream);

        let error = lexer
            .curr(false)
            .expect_err("Non-ASCII characters outside the supported lexer classes should stay hard errors");
        let message = error.to_string();

        assert!(
            message.contains("is not a recognized character"),
            "Unexpected lexer error message for unsupported character: {}",
            message
        );

        std::fs::remove_file(&temp_path).ok();
    }

    #[test]
    fn test_unrecognized_ascii_control_character_returns_lexer_error() {
        let temp_path = std::env::temp_dir().join(format!(
            "fol_lexer_bad_ascii_control_{}_{}.fol",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("System time should be after unix epoch")
                .as_nanos()
        ));
        std::fs::write(&temp_path, b"\x7f")
            .expect("Should write malformed ascii-control lexer fixture");

        let mut file_stream = FileStream::from_file(
            temp_path
                .to_str()
                .expect("Malformed lexer fixture path should be valid utf-8"),
        )
        .expect("Should open malformed ascii-control lexer fixture");
        let lexer = Elements::init(&mut file_stream);

        let error = lexer.curr(false).expect_err(
            "Unsupported ASCII control characters should stay hard lexer errors",
        );
        let message = error.to_string();

        assert!(
            message.contains("is not a recognized character"),
            "Unexpected lexer error message for unsupported ASCII control character: {}",
            message
        );

        std::fs::remove_file(&temp_path).ok();
    }

    #[test]
    fn test_unterminated_string_literal_becomes_illegal_token() {
        let tokens = tokenize_file("test/lexer/unterminated_string.fol");

        assert!(
            tokens.iter().any(|(key, _)| key.is_illegal()),
            "Unterminated quoted content should surface as an illegal token"
        );
    }

    #[test]
    fn test_unterminated_single_quoted_literal_becomes_illegal_token() {
        let tokens = tokenize_file("test/lexer/unterminated_single_quote.fol");

        assert!(
            tokens.iter().any(|(key, _)| key.is_illegal()),
            "Single-quoted unterminated content should use the same illegal-token path as double-quoted content"
        );
    }

    #[test]
    fn test_unterminated_backtick_comment_becomes_illegal_token() {
        let temp_path = std::env::temp_dir().join(format!(
            "fol_lexer_unterminated_backtick_comment_{}_{}.fol",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("System time should be after unix epoch")
                .as_nanos()
        ));
        std::fs::write(&temp_path, "`macroish")
            .expect("Should write unterminated backtick comment lexer fixture");

        let tokens = tokenize_file(
            temp_path
                .to_str()
                .expect("Unterminated backtick comment fixture path should be valid utf-8"),
        );

        assert!(
            tokens.iter().any(|(key, _)| key.is_illegal()),
            "Unterminated backtick comments should use the same illegal-token path as other malformed comment spans"
        );

        std::fs::remove_file(&temp_path).ok();
    }

    #[test]
    fn test_unterminated_slash_block_comment_becomes_illegal_token() {
        let temp_path = std::env::temp_dir().join(format!(
            "fol_lexer_unterminated_block_comment_{}_{}.fol",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("System time should be after unix epoch")
                .as_nanos()
        ));
        std::fs::write(&temp_path, "/* missing close")
            .expect("Should write unterminated block-comment lexer fixture");

        let tokens = tokenize_file(
            temp_path
                .to_str()
                .expect("Unterminated block-comment fixture path should be valid utf-8"),
        );

        assert!(
            tokens.iter().any(|(key, _)| key.is_illegal()),
            "Unterminated slash block comments should use the same illegal-token path as other malformed comment spans"
        );

        std::fs::remove_file(&temp_path).ok();
    }

    #[test]
    fn test_identifiers_with_repeated_underscore_runs_become_illegal_tokens() {
        let tokens = tokenize_file("test/lexer/identifier_repeated_underscores.fol");
        let significant: Vec<(KEYWORD, String)> = tokens
            .into_iter()
            .filter(|(key, _)| !key.is_space() && !key.is_eof())
            .collect();

        assert_eq!(
            significant,
            vec![
                (KEYWORD::Identifier, "good_name".to_string()),
                (KEYWORD::Illegal, "bad__name".to_string()),
                (KEYWORD::Illegal, "__hidden".to_string()),
                (KEYWORD::Identifier, "good_2".to_string()),
            ],
            "Identifiers with repeated underscore runs should be illegal while single underscores remain valid separators"
        );
    }

    #[test]
    fn test_identifier_edge_cases_follow_current_front_end_contract() {
        let tokens = tokenize_file("test/lexer/identifier_edge_cases.fol");
        let significant: Vec<(KEYWORD, String)> = tokens
            .into_iter()
            .filter(|(key, _)| !key.is_space() && !key.is_eol() && !key.is_eof())
            .collect();

        assert_eq!(
            significant,
            vec![
                (KEYWORD::Identifier, "_".to_string()),
                (KEYWORD::Identifier, "_hidden".to_string()),
                (KEYWORD::Identifier, "good_name".to_string()),
                (KEYWORD::Illegal, "bad__name".to_string()),
                (KEYWORD::Identifier, "Fun".to_string()),
                (KEYWORD::Keyword(BUILDIN::Fun), "fun".to_string()),
            ],
            "Current identifier edges should stay explicit: '_' is still a parser-relevant identifier surface, repeated underscores are illegal, and keyword matching remains exact-case"
        );
    }

}
