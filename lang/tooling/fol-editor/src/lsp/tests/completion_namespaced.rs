use super::helpers::{open_document, sample_package_root};
use super::super::{
    EditorLspServer, JsonRpcId, JsonRpcRequest, LspCompletionContext, LspCompletionList,
    LspCompletionParams, LspPosition, LspTextDocumentIdentifier,
};
use crate::EditorConfig;
use std::fs;

#[test]
fn lsp_server_returns_supported_v1_dot_intrinsics() {
    let (root, uri) = sample_package_root("completion_dot_intrinsics");
    fs::write(
        root.join("src/main.fol"),
        "fun[] main(): int = {\n    return .;\n};\n",
    )
    .unwrap();
    let text = fs::read_to_string(root.join("src/main.fol")).unwrap();
    let mut server = EditorLspServer::new(EditorConfig::default());
    open_document(&mut server, uri.clone(), &text);

    let completion = server
        .handle_request(JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: JsonRpcId::Number(44),
            method: "textDocument/completion".to_string(),
            params: Some(
                serde_json::to_value(LspCompletionParams {
                    text_document: LspTextDocumentIdentifier { uri: uri.clone() },
                    position: LspPosition {
                        line: 1,
                        character: 12,
                    },
                    context: Some(LspCompletionContext {
                        trigger_kind: Some(2),
                        trigger_character: Some(".".to_string()),
                    }),
                })
                .unwrap(),
            ),
        })
        .unwrap()
        .unwrap();

    let completion: LspCompletionList =
        serde_json::from_value(completion.result.unwrap()).unwrap();
    let labels = completion
        .items
        .iter()
        .map(|item| item.label.as_str())
        .collect::<Vec<_>>();
    assert!(labels.contains(&"len"));
    assert!(labels.contains(&"echo"));
    assert!(labels.contains(&"eq"));
    assert!(labels.contains(&"not"));

    fs::remove_dir_all(root).ok();
}

#[test]
fn lsp_server_uses_conservative_dot_fallback_for_incomplete_contexts() {
    let (root, uri) = sample_package_root("completion_dot_fallback");
    fs::write(
        root.join("src/main.fol"),
        "fun[] main(): int = {\n    return .;\n};\n",
    )
    .unwrap();
    let text = fs::read_to_string(root.join("src/main.fol")).unwrap();
    let mut server = EditorLspServer::new(EditorConfig::default());
    open_document(&mut server, uri.clone(), &text);

    let completion = server
        .handle_request(JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: JsonRpcId::Number(45),
            method: "textDocument/completion".to_string(),
            params: Some(
                serde_json::to_value(LspCompletionParams {
                    text_document: LspTextDocumentIdentifier { uri: uri.clone() },
                    position: LspPosition {
                        line: 1,
                        character: 12,
                    },
                    context: None,
                })
                .unwrap(),
            ),
        })
        .unwrap()
        .unwrap();

    let completion: LspCompletionList =
        serde_json::from_value(completion.result.unwrap()).unwrap();
    let labels = completion
        .items
        .iter()
        .map(|item| item.label.as_str())
        .collect::<Vec<_>>();
    assert!(labels.contains(&"len"));
    assert!(labels.contains(&"echo"));
    assert!(labels.contains(&"eq"));
    assert!(labels.contains(&"not"));
    assert!(!labels.contains(&"panic"));

    fs::remove_dir_all(root).ok();
}

#[test]
fn lsp_server_uses_lsp_dot_trigger_when_document_text_is_not_yet_synced() {
    let (root, uri) = sample_package_root("completion_dot_lsp_context");
    fs::write(
        root.join("src/main.fol"),
        "fun[] main(): int = {\n    return \n};\n",
    )
    .unwrap();
    let text = fs::read_to_string(root.join("src/main.fol")).unwrap();
    let mut server = EditorLspServer::new(EditorConfig::default());
    open_document(&mut server, uri.clone(), &text);

    let completion = server
        .handle_request(JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: JsonRpcId::Number(48),
            method: "textDocument/completion".to_string(),
            params: Some(
                serde_json::to_value(LspCompletionParams {
                    text_document: LspTextDocumentIdentifier { uri: uri.clone() },
                    position: LspPosition {
                        line: 1,
                        character: 12,
                    },
                    context: Some(LspCompletionContext {
                        trigger_kind: Some(2),
                        trigger_character: Some(".".to_string()),
                    }),
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
    assert!(labels.contains(&"len".to_string()));
    assert!(labels.contains(&"echo".to_string()));
    assert!(!labels.contains(&"main".to_string()));

    fs::remove_dir_all(root).ok();
}

#[test]
fn lsp_server_locks_dot_intrinsic_completion_matrix() {
    let (root, uri) = sample_package_root("completion_dot_matrix");
    fs::write(
        root.join("src/main.fol"),
        "fun[] main(): int = {\n    return .;\n};\n",
    )
    .unwrap();
    let text = fs::read_to_string(root.join("src/main.fol")).unwrap();
    let mut server = EditorLspServer::new(EditorConfig::default());
    open_document(&mut server, uri.clone(), &text);

    let completion = server
        .handle_request(JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: JsonRpcId::Number(46),
            method: "textDocument/completion".to_string(),
            params: Some(
                serde_json::to_value(LspCompletionParams {
                    text_document: LspTextDocumentIdentifier { uri: uri.clone() },
                    position: LspPosition {
                        line: 1,
                        character: 12,
                    },
                    context: Some(LspCompletionContext {
                        trigger_kind: Some(2),
                        trigger_character: Some(".".to_string()),
                    }),
                })
                .unwrap(),
            ),
        })
        .unwrap()
        .unwrap();

    let completion: LspCompletionList =
        serde_json::from_value(completion.result.unwrap()).unwrap();
    let labels = completion
        .items
        .iter()
        .map(|item| item.label.as_str())
        .collect::<Vec<_>>();
    assert!(labels.contains(&"len"));
    assert!(labels.contains(&"echo"));
    assert!(labels.contains(&"eq"));
    assert!(labels.contains(&"nq"));
    assert!(labels.contains(&"lt"));
    assert!(labels.contains(&"le"));
    assert!(labels.contains(&"gt"));
    assert!(labels.contains(&"ge"));
    assert!(labels.contains(&"not"));
    assert!(!labels.contains(&"panic"));

    fs::remove_dir_all(root).ok();
}

#[test]
fn lsp_server_filters_echo_from_core_and_alloc_dot_completion() {
    for (model, expected_echo) in [("core", false), ("alloc", false), ("std", true)] {
        let (root, uri) = sample_package_root(&format!("completion_dot_model_{model}"));
        fs::write(
            root.join("build.fol"),
            format!(
                concat!(
                    "pro[] build(graph: Graph): non = {{\n",
                    "    graph.add_exe({{ name = \"demo\", root = \"src/main.fol\", fol_model = \"{}\" }});\n",
                    "}};\n",
                ),
                model
            ),
        )
        .unwrap();
        fs::write(
            root.join("src/main.fol"),
            "fun[] main(): int = {\n    return .;\n};\n",
        )
        .unwrap();
        let text = fs::read_to_string(root.join("src/main.fol")).unwrap();
        let mut server = EditorLspServer::new(EditorConfig::default());
        open_document(&mut server, uri.clone(), &text);

        let completion = server
            .handle_request(JsonRpcRequest {
                jsonrpc: "2.0".to_string(),
                id: JsonRpcId::Number(461),
                method: "textDocument/completion".to_string(),
                params: Some(
                    serde_json::to_value(LspCompletionParams {
                        text_document: LspTextDocumentIdentifier { uri: uri.clone() },
                        position: LspPosition {
                            line: 1,
                            character: 12,
                        },
                        context: Some(LspCompletionContext {
                            trigger_kind: Some(2),
                            trigger_character: Some(".".to_string()),
                        }),
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
        assert_eq!(
            labels.contains(&"echo".to_string()),
            expected_echo,
            "dot completion echo visibility should track fol_model={model}"
        );

        fs::remove_dir_all(root).ok();
    }
}
