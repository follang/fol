use super::*;

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
