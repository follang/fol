use super::helpers::{copied_example_package_root, open_document};
use super::super::{
    EditorLspServer, JsonRpcId, JsonRpcRequest, LspCompletionContext, LspCompletionList,
    LspCompletionParams, LspPosition, LspSemanticTokens, LspSemanticTokensParams,
    LspTextDocumentIdentifier,
};
use crate::EditorConfig;
use std::fs;

fn decode_semantic_tokens(data: &[u32]) -> Vec<(u32, u32, u32, u32, u32)> {
    let mut decoded = Vec::new();
    let mut line = 0_u32;
    let mut start = 0_u32;
    for chunk in data.chunks_exact(5) {
        let delta_line = chunk[0];
        let delta_start = chunk[1];
        if delta_line == 0 {
            start += delta_start;
        } else {
            line += delta_line;
            start = delta_start;
        }
        decoded.push((line, start, chunk[2], chunk[3], chunk[4]));
    }
    decoded
}

#[test]
fn lsp_server_opens_real_model_example_packages_cleanly() {
    for example in [
        "examples/core_defer",
        "examples/alloc_defaults",
        "examples/std_echo_min",
    ] {
        let (root, uri) = copied_example_package_root(example);
        let text = fs::read_to_string(root.join("src/main.fol")).unwrap();
        let mut server = EditorLspServer::new(EditorConfig::default());
        let diagnostics = open_document(&mut server, uri, &text);

        assert!(
            diagnostics.iter().all(|published| published.diagnostics.is_empty()),
            "real example '{example}' should open without editor diagnostics: {diagnostics:#?}"
        );

        fs::remove_dir_all(root).ok();
    }
}

#[test]
fn lsp_server_reports_model_aware_diagnostics_for_real_example_roots() {
    let cases = [
        (
            "examples/core_defer",
            "fun[] main(): str = {\n    return \"bad\";\n};\n",
            Some("str requires heap support and is unavailable in 'fol_model = core'"),
        ),
        (
            "examples/alloc_defaults",
            "fun[] main(): int = {\n    return .echo(7);\n};\n",
            Some("'.echo(...)' requires 'fol_model = std'"),
        ),
        (
            "examples/std_echo_min",
            "fun[] main(): int = {\n    var shown: int = .echo(9);\n    return 9;\n};\n",
            None,
        ),
    ];

    for (example, source, expected_message) in cases {
        let (root, uri) = copied_example_package_root(example);
        fs::write(root.join("src/main.fol"), source).unwrap();
        let mut server = EditorLspServer::new(EditorConfig::default());
        let diagnostics = open_document(&mut server, uri, source);
        let messages = diagnostics
            .iter()
            .flat_map(|published| published.diagnostics.iter())
            .map(|diagnostic| diagnostic.message.as_str())
            .collect::<Vec<_>>();

        match expected_message {
            Some(expected_message) => assert!(
                messages.iter().any(|message| message.contains(expected_message)),
                "example '{example}' should surface model-aware error '{expected_message}', got: {messages:?}"
            ),
            None => assert!(
                messages.is_empty(),
                "std example '{example}' should stay quiet under legal hosted surfaces: {messages:?}"
            ),
        }

        fs::remove_dir_all(root).ok();
    }
}

#[test]
fn lsp_server_returns_semantic_tokens_for_real_model_examples() {
    for example in [
        "examples/core_defer",
        "examples/alloc_defaults",
        "examples/std_echo_min",
    ] {
        let (root, uri) = copied_example_package_root(example);
        let text = fs::read_to_string(root.join("src/main.fol")).unwrap();
        let mut server = EditorLspServer::new(EditorConfig::default());
        open_document(&mut server, uri.clone(), &text);

        let response = server
            .handle_request(JsonRpcRequest {
                jsonrpc: "2.0".to_string(),
                id: JsonRpcId::Number(980),
                method: "textDocument/semanticTokens/full".to_string(),
                params: Some(
                    serde_json::to_value(LspSemanticTokensParams {
                        text_document: LspTextDocumentIdentifier { uri: uri.clone() },
                    })
                    .unwrap(),
                ),
            })
            .unwrap()
            .unwrap();
        let tokens: LspSemanticTokens = serde_json::from_value(response.result.unwrap()).unwrap();
        let decoded = decode_semantic_tokens(&tokens.data);
        let kinds = decoded.iter().map(|token| token.3).collect::<Vec<_>>();

        assert!(
            !decoded.is_empty(),
            "semantic tokens should not be empty for real example '{example}'"
        );
        assert!(
            kinds.contains(&2),
            "semantic tokens for '{example}' should include a function token: {decoded:?}"
        );
        assert!(
            kinds.contains(&4),
            "semantic tokens for '{example}' should include a variable token: {decoded:?}"
        );

        fs::remove_dir_all(root).ok();
    }
}

#[test]
fn lsp_server_respects_model_completion_when_opened_at_real_example_roots() {
    let cases = [
        (
            "examples/core_defer",
            "fun[] main(): int = {\n    var value: ;\n    return 0;\n};\n",
            LspPosition {
                line: 1,
                character: 15,
            },
            None,
            vec!["int", "arr", "opt", "err"],
            vec!["str", "seq", "vec", "set", "map", "echo"],
        ),
        (
            "examples/alloc_defaults",
            "fun[] main(): int = {\n    return .;\n};\n",
            LspPosition {
                line: 1,
                character: 12,
            },
            Some(LspCompletionContext {
                trigger_kind: Some(2),
                trigger_character: Some(".".to_string()),
            }),
            vec!["len", "eq", "not"],
            vec!["echo"],
        ),
        (
            "examples/std_echo_min",
            "fun[] main(): int = {\n    return .;\n};\n",
            LspPosition {
                line: 1,
                character: 12,
            },
            Some(LspCompletionContext {
                trigger_kind: Some(2),
                trigger_character: Some(".".to_string()),
            }),
            vec!["len", "echo", "eq", "not"],
            vec![],
        ),
    ];

    for (example, source, position, context, present, absent) in cases {
        let (root, uri) = copied_example_package_root(example);
        fs::write(root.join("src/main.fol"), source).unwrap();
        let mut server = EditorLspServer::new(EditorConfig::default());
        open_document(&mut server, uri.clone(), source);

        let completion = server
            .handle_request(JsonRpcRequest {
                jsonrpc: "2.0".to_string(),
                id: JsonRpcId::Number(981),
                method: "textDocument/completion".to_string(),
                params: Some(
                    serde_json::to_value(LspCompletionParams {
                        text_document: LspTextDocumentIdentifier { uri: uri.clone() },
                        position,
                        context,
                    })
                    .unwrap(),
                ),
            })
            .unwrap()
            .unwrap();
        let labels = serde_json::from_value::<LspCompletionList>(completion.result.unwrap())
            .unwrap()
            .items
            .into_iter()
            .map(|item| item.label)
            .collect::<Vec<_>>();

        for label in present {
            assert!(
                labels.iter().any(|candidate| candidate == label),
                "example '{example}' should expose completion '{label}', got: {labels:?}"
            );
        }
        for label in absent {
            assert!(
                !labels.iter().any(|candidate| candidate == label),
                "example '{example}' should hide completion '{label}', got: {labels:?}"
            );
        }

        fs::remove_dir_all(root).ok();
    }
}
