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
fn test_stage2_seek_ignore_looks_back_through_space_tokens() {
    use std::fs;

    let temp_root = unique_temp_root("stage2_seek_ignore");
    fs::create_dir_all(&temp_root).expect("Should create temp stage2 seek fixture dir");
    let fixture = temp_root.join("stage2_seek_ignore.fol");
    fs::write(&fixture, "alpha beta\n").expect("Should write stage2 seek fixture");

    let mut file_stream = FileStream::from_file(
        fixture
            .to_str()
            .expect("Stage2 seek fixture path should be UTF-8"),
    )
    .expect("Should open stage2 seek fixture");
    let mut elements = stage2::Elements::init(&mut file_stream);
    let _ = elements.bump();

    for _ in 0..32 {
        let token = elements
            .curr(false)
            .expect("Stage2 should expose a current token while seeking");
        if token.con() == "beta" {
            break;
        }
        let _ = elements.bump();
    }

    assert_eq!(
        elements
            .curr(false)
            .expect("Stage2 should land on the second identifier")
            .con(),
        "beta"
    );
    assert!(
        elements
            .seek(0, false)
            .expect("Stage2 seek(false) should expose the immediate previous token")
            .key()
            .is_space(),
        "Stage2 seek(false) should still see the immediate space token"
    );
    assert_eq!(
        elements
            .seek(0, true)
            .expect("Stage2 seek(true) should skip ignorable spaces when looking behind")
            .con(),
        "alpha",
        "Stage2 seek(true) should inspect the reverse window instead of the forward window"
    );

    fs::remove_dir_all(&temp_root).ok();
}

#[test]
fn test_stage2_jump_returns_cleanly_after_stream_drain() {
    let mut file_stream =
        FileStream::from_file("test/stream/basic.fol").expect("Should read basic file");
    let mut elements = stage2::Elements::init(&mut file_stream);

    while elements.bump().is_some() {}

    assert!(
        elements.jump(0, false).is_ok(),
        "Stage2 jump() should return cleanly instead of unwrapping None after the stream is drained"
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
fn test_stage3_seek_ignore_looks_back_through_space_tokens() {
    use std::fs;

    let temp_root = unique_temp_root("stage3_seek_ignore");
    fs::create_dir_all(&temp_root).expect("Should create temp stage3 seek fixture dir");
    let fixture = temp_root.join("stage3_seek_ignore.fol");
    fs::write(&fixture, "alpha beta\n").expect("Should write stage3 seek fixture");

    let mut file_stream = FileStream::from_file(
        fixture
            .to_str()
            .expect("Stage3 seek fixture path should be UTF-8"),
    )
    .expect("Should open stage3 seek fixture");
    let mut elements = Elements::init(&mut file_stream);

    for _ in 0..32 {
        let token = elements
            .curr(false)
            .expect("Stage3 should expose a current token while seeking");
        if token.con() == "beta" {
            break;
        }
        let _ = elements.bump();
    }

    assert_eq!(
        elements
            .curr(false)
            .expect("Stage3 should land on the second identifier")
            .con(),
        "beta"
    );
    assert!(
        elements
            .seek(0, false)
            .expect("Stage3 seek(false) should expose the immediate previous token")
            .key()
            .is_space(),
        "Stage3 seek(false) should still see the immediate space token"
    );
    assert_eq!(
        elements
            .seek(0, true)
            .expect("Stage3 seek(true) should skip ignorable spaces when looking behind")
            .con(),
        "alpha",
        "Stage3 seek(true) should inspect the reverse window instead of the forward window"
    );

    fs::remove_dir_all(&temp_root).ok();
}

#[test]
fn test_stage3_jump_returns_cleanly_after_stream_drain() {
    let mut file_stream =
        FileStream::from_file("test/stream/basic.fol").expect("Should read basic file");
    let mut elements = Elements::init(&mut file_stream);

    while elements.bump().is_some() {}

    assert!(
        elements.jump(0, false).is_ok(),
        "Stage3 jump() should return cleanly instead of unwrapping None after the stream is drained"
    );
}

#[test]
fn test_nonexistent_file_error() {
    let result = std::panic::catch_unwind(|| tokenize_file("test/lexer/nonexistent.fol"));

    assert!(result.is_err(), "Should panic on nonexistent file");
}

#[test]
fn test_unrecognized_non_ascii_character_returns_lexer_error() {
    let temp_root = unique_temp_root("bad_char");
    std::fs::create_dir_all(&temp_root).expect("Should create malformed lexer fixture root");
    let temp_path = temp_root.join("fixture.fol");
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
    std::fs::remove_dir_all(&temp_root).ok();
}

#[test]
fn test_unrecognized_ascii_control_character_returns_lexer_error() {
    let temp_root = unique_temp_root("bad_ascii_control");
    std::fs::create_dir_all(&temp_root)
        .expect("Should create malformed ascii-control fixture root");
    let temp_path = temp_root.join("fixture.fol");
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
    std::fs::remove_dir_all(&temp_root).ok();
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
    let temp_root = unique_temp_root("unterminated_backtick_comment");
    std::fs::create_dir_all(&temp_root)
        .expect("Should create unterminated backtick comment fixture root");
    let temp_path = temp_root.join("fixture.fol");
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
    std::fs::remove_dir_all(&temp_root).ok();
}

#[test]
fn test_unterminated_slash_block_comment_becomes_illegal_token() {
    let temp_root = unique_temp_root("unterminated_block_comment");
    std::fs::create_dir_all(&temp_root)
        .expect("Should create unterminated block-comment fixture root");
    let temp_path = temp_root.join("fixture.fol");
    std::fs::write(&temp_path, "/* missing close")
        .expect("Should write unterminated block-comment lexer fixture");

    let path_str = temp_path
        .to_str()
        .expect("Unterminated block-comment fixture path should be valid utf-8");
    let mut file_stream = FileStream::from_file(path_str)
        .expect("Should open unterminated block-comment fixture");
    let mut lexer = Elements::init(&mut file_stream);
    let mut found_error = false;
    for _ in 0..10_000 {
        match lexer.curr(false) {
            Ok(token) => {
                if token.key() == KEYWORD::Void(VOID::EndFile) {
                    break;
                }
                if lexer.bump().is_none() {
                    break;
                }
            }
            Err(e) => {
                let msg = e.to_string();
                assert!(
                    msg.contains("unterminated block comment"),
                    "Unterminated slash block comment should produce a specific error, got: {}",
                    msg
                );
                found_error = true;
                break;
            }
        }
    }

    assert!(
        found_error,
        "Unterminated slash block comment at EOF should produce a lexer error"
    );

    std::fs::remove_file(&temp_path).ok();
    std::fs::remove_dir_all(&temp_root).ok();
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

#[test]
fn test_location_visualize_survives_missing_source_file() {
    let missing_path = unique_temp_root("missing_visualize_file")
        .join("gone.fol")
        .to_string_lossy()
        .to_string();
    let source = Some(Source::new(missing_path.clone()));
    let mut location = Location::default();
    location.adjust(4, 2);
    location.set_len(3);
    location.set_source(&source);

    let visualization = location.visualize(&source);

    assert!(
        visualization.contains("source file unavailable"),
        "Missing source files should render a fallback message instead of panicking: {}",
        visualization
    );
    assert!(
        visualization.contains(&missing_path),
        "Fallback visualization should mention the missing file path"
    );
}

#[test]
fn test_location_visualize_survives_missing_source_line() {
    use std::fs;

    let temp_root = unique_temp_root("missing_visualize_line");
    fs::create_dir_all(&temp_root).expect("Should create temp location fixture dir");
    let file_path = temp_root.join("short.fol");
    fs::write(&file_path, "only one line\n").expect("Should write short source fixture");

    let source = Some(Source::new(
        file_path
            .to_str()
            .expect("Location fixture path should be UTF-8")
            .to_string(),
    ));
    let mut location = Location::default();
    location.adjust(5, 1);
    location.set_len(1);
    location.set_source(&source);

    let visualization = location.visualize(&source);

    fs::remove_dir_all(&temp_root).ok();

    assert!(
        visualization.contains("<source line unavailable>"),
        "Missing source lines should render a fallback line instead of panicking: {}",
        visualization
    );
}
