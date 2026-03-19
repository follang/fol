use super::*;

#[test]
fn test_backticks_surface_as_parser_visible_comments() {
    let tokens = tokenize_file("test/lexer/backticks.fol");
    let comments: Vec<(KEYWORD, String)> = tokens
        .into_iter()
        .filter(|(key, _)| key.is_comment())
        .collect();

    assert_eq!(
        comments,
        vec![(KEYWORD::Comment(COMMENT::Backtick), "`macroish`".to_string())],
        "Backtick-delimited comments should now survive through the parser-facing lexer output"
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
fn test_stage2_preserves_backtick_and_doc_comments() {
    let backtick_tokens = tokenize_stage2_file("test/lexer/backticks.fol");
    let doc_tokens = tokenize_stage2_file("test/lexer/doc_comments.fol");

    assert!(
        backtick_tokens
            .iter()
            .any(|(key, content)| *key == KEYWORD::Comment(COMMENT::Backtick) && content == "`macroish`"),
        "Stage 2 should preserve ordinary backtick comments as explicit comment tokens"
    );
    assert!(
        doc_tokens.iter().any(|(key, content)| {
            *key == KEYWORD::Comment(COMMENT::Doc) && content == "`[doc] module docs`"
        }),
        "Stage 2 should preserve doc comments instead of collapsing them to whitespace"
    );
    assert!(
        doc_tokens
            .iter()
            .filter(|(key, _)| *key == KEYWORD::Comment(COMMENT::Doc))
            .count()
            == 2,
        "Stage 2 should keep both doc comments as comment tokens"
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
fn test_multiline_backtick_comments_remain_parser_visible() {
    let tokens = tokenize_file("test/lexer/backtick_multiline_comments.fol");
    let significant: Vec<(KEYWORD, String)> = tokens
        .into_iter()
        .filter(|(key, _)| !key.is_void() && !key.is_eof())
        .collect();

    assert_eq!(
        significant,
        vec![
            (
                KEYWORD::Comment(COMMENT::Backtick),
                "`note one\nnote two`".to_string(),
            ),
            (KEYWORD::Keyword(BUILDIN::Var), "var".to_string()),
            (KEYWORD::Identifier, "after".to_string()),
            (KEYWORD::Symbol(SYMBOL::Equal), "=".to_string()),
            (KEYWORD::Literal(LITERAL::Decimal), "1".to_string()),
            (KEYWORD::Symbol(SYMBOL::Semi), "; ".to_string()),
        ],
        "Multiline backtick comments should remain visible in the parser-facing lexer output"
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
    let temp_root = unique_temp_root("multiline_quote");
    std::fs::create_dir_all(&temp_root).expect("Should create multiline quoted fixture root");
    let temp_path = temp_root.join("fixture.fol");
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
    std::fs::remove_dir_all(&temp_root).ok();
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
fn test_comments_reach_parser_facing_lexer_output() {
    let tokens = tokenize_file("test/lexer/comments.fol");
    let comment_tokens: Vec<_> = tokens.iter().filter(|(key, _)| key.is_comment()).collect();

    assert!(
        !comment_tokens.is_empty(),
        "Stage 3 should expose ordinary comments as parser-visible tokens for later AST retention"
    );
    assert!(
        comment_tokens.iter().any(|(key, content)| {
            *key == KEYWORD::Comment(COMMENT::SlashLine)
                && content == "// single line comment"
        }),
        "Parser-facing comment tokens should retain their original raw spelling"
    );
}

#[test]
fn test_doc_comments_stay_distinct_in_parser_facing_lexer_output() {
    let tokens = tokenize_file("test/lexer/doc_comments.fol");
    let doc_comments: Vec<(KEYWORD, String)> = tokens
        .iter()
        .filter(|(key, _)| *key == KEYWORD::Comment(COMMENT::Doc))
        .map(|(key, content)| (key.clone(), content.clone()))
        .collect();

    assert_eq!(
        doc_comments,
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
        "Backtick doc comments should remain distinguishable from ordinary comments in parser-facing lexer output"
    );
}

#[test]
fn test_doc_comments_do_not_disturb_surrounding_code_tokens() {
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
        "Preserved doc comments should still remain non-semantic for ordinary code tokenization"
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
    let comment_tokens: Vec<(KEYWORD, String)> = tokens
        .iter()
        .filter(|(key, _)| *key == KEYWORD::Comment(COMMENT::SlashLine))
        .map(|(key, content)| (key.clone(), content.clone()))
        .collect();
    let significant: Vec<(KEYWORD, String)> = tokens
        .into_iter()
        .filter(|(key, _)| !key.is_void() && !key.is_comment() && !key.is_eof())
        .collect();

    assert!(
        !comment_tokens.is_empty(),
        "Slash line comments should stay preserved as explicit compatibility comment tokens"
    );
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
    let comment_tokens: Vec<(KEYWORD, String)> = tokens
        .iter()
        .filter(|(key, _)| *key == KEYWORD::Comment(COMMENT::SlashBlock))
        .map(|(key, content)| (key.clone(), content.clone()))
        .collect();
    let significant: Vec<(KEYWORD, String)> = tokens
        .into_iter()
        .filter(|(key, _)| !key.is_void() && !key.is_comment() && !key.is_eof())
        .collect();

    assert!(
        !comment_tokens.is_empty(),
        "Slash block comments should stay preserved as explicit compatibility comment tokens"
    );
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
    assert_eq!(colon_count, 1, "Should tokenize the return-type ':' separator");
    assert!(
        significant
            .iter()
            .any(|(k, _)| matches!(k, KEYWORD::Symbol(SYMBOL::Root))),
        "Should tokenize '/' as the error-type separator"
    );

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
