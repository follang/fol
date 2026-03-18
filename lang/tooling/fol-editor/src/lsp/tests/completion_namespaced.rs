use super::helpers::{open_document, sample_loc_workspace_root, sample_package_root};
use super::super::{
    completion_helpers::completion_context, completion_helpers::CompletionContext,
    EditorLspServer, JsonRpcId, JsonRpcNotification, JsonRpcRequest, LspCompletionContext,
    LspCompletionList, LspCompletionParams, LspDefinitionParams,
    LspDidChangeTextDocumentParams, LspDidCloseTextDocumentParams,
    LspDidOpenTextDocumentParams, LspDocumentSymbolParams, LspHover, LspHoverParams,
    LspInitializeResult, LspLocation, LspPosition, LspPublishDiagnosticsParams,
    LspTextDocumentContentChangeEvent, LspTextDocumentIdentifier, LspTextDocumentItem,
    LspVersionedTextDocumentIdentifier,
};
use crate::{EditorConfig, EditorDocument, EditorDocumentUri};
use std::fs;
use std::path::PathBuf;


#[test]
fn lsp_server_returns_supported_v1_dot_intrinsics() {
    let (root, uri) = sample_package_root("completion_dot_intrinsics");
    fs::write(
        root.join("src/main.fol"),
        "fun[] main(): int = {\n    return .\n}\n",
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
        "fun[] main(): int = {\n    return .\n}\n",
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
fn lsp_server_locks_dot_intrinsic_completion_matrix() {
    let (root, uri) = sample_package_root("completion_dot_matrix");
    fs::write(
        root.join("src/main.fol"),
        "fun[] main(): int = {\n    return .\n}\n",
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

