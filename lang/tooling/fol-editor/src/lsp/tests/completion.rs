use super::helpers::{open_document, sample_loc_workspace_root, sample_package_root};
use super::super::{
    EditorLspServer, JsonRpcId, JsonRpcRequest, LspCompletionList, LspCompletionParams,
    LspPosition, LspTextDocumentIdentifier,
};
use crate::EditorConfig;
use std::fs;

#[test]
fn lsp_server_handles_completion_requests() {
    let (root, uri) = sample_package_root("completion_request");
    fs::write(
        root.join("src/main.fol"),
        "fun[] main(): int = {\n    var value: int = 7\n    return value\n}\n",
    )
    .unwrap();
    let text = fs::read_to_string(root.join("src/main.fol")).unwrap();
    let mut server = EditorLspServer::new(EditorConfig::default());
    open_document(&mut server, uri.clone(), &text);

    let completion = server
        .handle_request(JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: JsonRpcId::Number(30),
            method: "textDocument/completion".to_string(),
            params: Some(
                serde_json::to_value(LspCompletionParams {
                    text_document: LspTextDocumentIdentifier { uri: uri.clone() },
                    position: LspPosition {
                        line: 2,
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
    assert!(!completion.is_incomplete);
    assert!(completion.items.iter().any(|item| item.label == "value"));
    assert!(
        completion
            .items
            .iter()
            .find(|item| item.label == "value")
            .and_then(|item| item.detail.as_deref())
            == Some("binding")
    );

    fs::remove_dir_all(root).ok();
}

#[test]
fn lsp_server_keeps_plain_completion_available_when_typecheck_fails() {
    let (root, uri) = sample_package_root("completion_type_error_fallback");
    fs::write(
        root.join("src/main.fol"),
        "fun[] helper(): int = {\n    return 7\n}\n\nfun[] main(): int = {\n    var value: int = helper()\n    value = \"oops\"\n    return value\n}\n",
    )
    .unwrap();
    let text = fs::read_to_string(root.join("src/main.fol")).unwrap();
    let mut server = EditorLspServer::new(EditorConfig::default());
    let diagnostics = open_document(&mut server, uri.clone(), &text);
    assert!(!diagnostics.is_empty());

    let completion = server
        .handle_request(JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: JsonRpcId::Number(30),
            method: "textDocument/completion".to_string(),
            params: Some(
                serde_json::to_value(LspCompletionParams {
                    text_document: LspTextDocumentIdentifier { uri: uri.clone() },
                    position: LspPosition {
                        line: 6,
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
    assert!(completion.items.iter().any(|item| item.label == "value"));
    assert!(completion.items.iter().any(|item| item.label == "helper"));

    fs::remove_dir_all(root).ok();
}

#[test]
fn lsp_server_returns_routine_parameter_completions() {
    let (root, uri) = sample_package_root("completion_params");
    fs::write(
        root.join("src/main.fol"),
        "fun[] main(total: int): int = {\n    return total\n}\n",
    )
    .unwrap();
    let text = fs::read_to_string(root.join("src/main.fol")).unwrap();
    let mut server = EditorLspServer::new(EditorConfig::default());
    open_document(&mut server, uri.clone(), &text);

    let completion = server
        .handle_request(JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: JsonRpcId::Number(31),
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
    assert!(completion.items.iter().any(|item| item.label == "total"));
    assert!(
        completion
            .items
            .iter()
            .find(|item| item.label == "total")
            .and_then(|item| item.detail.as_deref())
            == Some("parameter")
    );

    fs::remove_dir_all(root).ok();
}

#[test]
fn lsp_server_returns_builtin_type_completions_in_type_positions() {
    let (root, uri) = sample_package_root("completion_builtin_types");
    fs::write(
        root.join("src/main.fol"),
        "fun[] main(): int = {\n    var value: \n    return 0\n}\n",
    )
    .unwrap();
    let text = fs::read_to_string(root.join("src/main.fol")).unwrap();
    let mut server = EditorLspServer::new(EditorConfig::default());
    open_document(&mut server, uri.clone(), &text);

    let completion = server
        .handle_request(JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: JsonRpcId::Number(36),
            method: "textDocument/completion".to_string(),
            params: Some(
                serde_json::to_value(LspCompletionParams {
                    text_document: LspTextDocumentIdentifier { uri: uri.clone() },
                    position: LspPosition {
                        line: 1,
                        character: 15,
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
    assert!(labels.contains(&"int"));
    assert!(labels.contains(&"str"));
    assert!(labels.contains(&"never"));

    fs::remove_dir_all(root).ok();
}

#[test]
fn lsp_server_returns_visible_named_type_completions_in_type_positions() {
    let (root, uri) = sample_loc_workspace_root("completion_named_types");
    fs::write(
        root.join("app/src/main.fol"),
        "use shared: loc = {\"../shared\"};\n\nfun[] main(): shared::Status = {\n    var report: shared::Report = \n    return shared::Pending\n}\n",
    )
    .unwrap();
    fs::write(
        root.join("shared/src/lib.fol"),
        "typ[exp] Status: ent = {\n    case Pending\n}\n\ntyp[exp] Report: rec = {\n    value: int\n}\n",
    )
    .unwrap();
    let text = fs::read_to_string(root.join("app/src/main.fol")).unwrap();
    let mut server = EditorLspServer::new(EditorConfig::default());
    open_document(&mut server, uri.clone(), &text);

    let completion = server
        .handle_request(JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: JsonRpcId::Number(37),
            method: "textDocument/completion".to_string(),
            params: Some(
                serde_json::to_value(LspCompletionParams {
                    text_document: LspTextDocumentIdentifier { uri: uri.clone() },
                    position: LspPosition {
                        line: 3,
                        character: 31,
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
    assert!(labels.contains(&"Status"));
    assert!(labels.contains(&"Report"));

    fs::remove_dir_all(root).ok();
}

#[test]
fn lsp_server_locks_type_completion_matrix() {
    let (root, uri) = sample_loc_workspace_root("completion_type_matrix");
    fs::write(
        root.join("app/src/main.fol"),
        "use shared: loc = {\"../shared\"};\n\ntyp[] Local: rec = {\n    value: int\n}\n\nfun[] main(): int = {\n    var target: \n    return 0\n}\n",
    )
    .unwrap();
    fs::write(
        root.join("shared/src/lib.fol"),
        "typ[exp] Status: ent = {\n    case Pending\n}\n\ntyp[exp] Report: rec = {\n    value: int\n}\n",
    )
    .unwrap();
    let text = fs::read_to_string(root.join("app/src/main.fol")).unwrap();
    let mut server = EditorLspServer::new(EditorConfig::default());
    open_document(&mut server, uri.clone(), &text);

    let completion = server
        .handle_request(JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: JsonRpcId::Number(38),
            method: "textDocument/completion".to_string(),
            params: Some(
                serde_json::to_value(LspCompletionParams {
                    text_document: LspTextDocumentIdentifier { uri: uri.clone() },
                    position: LspPosition {
                        line: 9,
                        character: 16,
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
    assert!(labels.contains(&"int"));
    assert!(labels.contains(&"str"));
    assert!(labels.contains(&"Local"));
    assert!(labels.contains(&"Status"));
    assert!(labels.contains(&"Report"));

    fs::remove_dir_all(root).ok();
}

#[test]
fn lsp_server_prefers_builtin_types_ahead_of_named_type_items() {
    let (root, uri) = sample_package_root("completion_type_order");
    fs::write(
        root.join("src/main.fol"),
        "typ[] Aardvark: rec = {\n    value: int\n}\n\nfun[] main(): int = {\n    var target: \n    return 0\n}\n",
    )
    .unwrap();
    let text = fs::read_to_string(root.join("src/main.fol")).unwrap();
    let mut server = EditorLspServer::new(EditorConfig::default());
    open_document(&mut server, uri.clone(), &text);

    let completion = server
        .handle_request(JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: JsonRpcId::Number(381),
            method: "textDocument/completion".to_string(),
            params: Some(
                serde_json::to_value(LspCompletionParams {
                    text_document: LspTextDocumentIdentifier { uri: uri.clone() },
                    position: LspPosition {
                        line: 4,
                        character: 16,
                    },
                    context: None,
                })
                .unwrap(),
            ),
        })
        .unwrap()
        .unwrap();

    let summary = serde_json::from_value::<LspCompletionList>(completion.result.unwrap())
        .unwrap()
        .items
        .into_iter()
        .filter(|item| matches!(item.label.as_str(), "int" | "Aardvark"))
        .map(|item| format!("{}:{}", item.label, item.detail.unwrap_or_default()))
        .collect::<Vec<_>>();
    assert_eq!(summary, vec!["int:builtin type", "Aardvark:type"]);

    fs::remove_dir_all(root).ok();
}

#[test]
fn lsp_server_returns_same_package_namespace_members_after_qualification() {
    let (root, uri) = sample_package_root("completion_namespace_local");
    fs::write(
        root.join("src/main.fol"),
        "fun[] main(): int = {\n    return api::\n}\n",
    )
    .unwrap();
    fs::create_dir_all(root.join("src/api")).unwrap();
    fs::write(
        root.join("src/api/lib.fol"),
        "fun[exp] helper(): int = {\n    return 7\n}\n",
    )
    .unwrap();
    let text = fs::read_to_string(root.join("src/main.fol")).unwrap();
    let mut server = EditorLspServer::new(EditorConfig::default());
    open_document(&mut server, uri.clone(), &text);

    let completion = server
        .handle_request(JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: JsonRpcId::Number(39),
            method: "textDocument/completion".to_string(),
            params: Some(
                serde_json::to_value(LspCompletionParams {
                    text_document: LspTextDocumentIdentifier { uri: uri.clone() },
                    position: LspPosition {
                        line: 1,
                        character: 16,
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
    assert!(completion.items.iter().any(|item| item.label == "helper"));

    fs::remove_dir_all(root).ok();
}

#[test]
fn lsp_server_keeps_local_and_imported_namespace_members_separate() {
    let (root, uri) = sample_loc_workspace_root("completion_namespace_separation");
    fs::create_dir_all(root.join("app/src/api")).unwrap();
    fs::write(
        root.join("app/src/main.fol"),
        "use shared: loc = {\"../shared\"};\n\nfun[] main(): int = {\n    return api::helper() + shared::\n}\n",
    )
    .unwrap();
    fs::write(
        root.join("app/src/api/lib.fol"),
        "fun[exp] local_helper(): int = {\n    return 7\n}\n",
    )
    .unwrap();
    fs::write(
        root.join("shared/src/lib.fol"),
        "fun[exp] helper(): int = {\n    return 9\n}\n",
    )
    .unwrap();
    let text = fs::read_to_string(root.join("app/src/main.fol")).unwrap();
    let mut server = EditorLspServer::new(EditorConfig::default());
    open_document(&mut server, uri.clone(), &text);

    let completion = server
        .handle_request(JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: JsonRpcId::Number(40),
            method: "textDocument/completion".to_string(),
            params: Some(
                serde_json::to_value(LspCompletionParams {
                    text_document: LspTextDocumentIdentifier { uri: uri.clone() },
                    position: LspPosition {
                        line: 3,
                        character: 16,
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
    assert!(labels.contains(&"local_helper"));
    assert!(!labels.contains(&"helper"));

    fs::remove_dir_all(root).ok();
}

#[test]
fn lsp_server_locks_loc_and_same_package_namespace_completion() {
    let (root, uri) = sample_loc_workspace_root("completion_namespace_matrix");
    fs::create_dir_all(root.join("app/src/api/tools")).unwrap();
    fs::write(
        root.join("app/src/main.fol"),
        "use shared: loc = {\"../shared\"};\n\nfun[] main(): int = {\n    return api::\n}\n",
    )
    .unwrap();
    fs::write(
        root.join("app/src/api/lib.fol"),
        "fun[exp] helper(): int = {\n    return 7\n}\n",
    )
    .unwrap();
    fs::write(
        root.join("app/src/api/tools/lib.fol"),
        "fun[exp] leaf(): int = {\n    return 8\n}\n",
    )
    .unwrap();
    fs::write(
        root.join("shared/src/lib.fol"),
        "fun[exp] helper(): int = {\n    return 9\n}\n",
    )
    .unwrap();
    let text = fs::read_to_string(root.join("app/src/main.fol")).unwrap();
    let mut server = EditorLspServer::new(EditorConfig::default());
    open_document(&mut server, uri.clone(), &text);

    let local_completion = server
        .handle_request(JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: JsonRpcId::Number(41),
            method: "textDocument/completion".to_string(),
            params: Some(
                serde_json::to_value(LspCompletionParams {
                    text_document: LspTextDocumentIdentifier { uri: uri.clone() },
                    position: LspPosition {
                        line: 3,
                        character: 16,
                    },
                    context: None,
                })
                .unwrap(),
            ),
        })
        .unwrap()
        .unwrap();
    let local_completion: LspCompletionList =
        serde_json::from_value(local_completion.result.unwrap()).unwrap();
    let local_labels = local_completion
        .items
        .iter()
        .map(|item| item.label.as_str())
        .collect::<Vec<_>>();
    assert!(local_labels.contains(&"helper"));
    assert!(local_labels.contains(&"tools"));

    let imported_completion = server
        .handle_request(JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: JsonRpcId::Number(42),
            method: "textDocument/completion".to_string(),
            params: Some(
                serde_json::to_value(LspCompletionParams {
                    text_document: LspTextDocumentIdentifier { uri: uri.clone() },
                    position: LspPosition {
                        line: 3,
                        character: 35,
                    },
                    context: None,
                })
                .unwrap(),
            ),
        })
        .unwrap()
        .unwrap();
    let imported_completion: LspCompletionList =
        serde_json::from_value(imported_completion.result.unwrap()).unwrap();
    let imported_labels = imported_completion
        .items
        .iter()
        .map(|item| item.label.as_str())
        .collect::<Vec<_>>();
    assert!(imported_labels.contains(&"helper"));
    assert!(!imported_labels.contains(&"tools"));

    fs::remove_dir_all(root).ok();
}
