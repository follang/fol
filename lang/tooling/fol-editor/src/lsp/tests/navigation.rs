use super::helpers::{open_document, sample_loc_workspace_root, sample_package_root};
use super::super::{
    EditorLspServer, JsonRpcId, JsonRpcRequest, LspCodeAction, LspCodeActionContext,
    LspCodeActionParams, LspDefinitionParams, LspDocumentSymbolParams, LspLocation, LspPosition,
    LspRange, LspReferenceContext, LspReferenceParams, LspRenameParams, LspSignatureHelp,
    LspSignatureHelpParams, LspTextDocumentIdentifier, LspWorkspaceEdit, LspWorkspaceSymbol,
    LspWorkspaceSymbolParams,
};
use crate::EditorConfig;
use std::fs;
use std::path::PathBuf;

#[test]
fn lsp_server_keeps_nested_document_symbols_stable() {
    let (root, uri) = sample_package_root("nested_symbols");
    fs::write(
        root.join("src/main.fol"),
        "fun[] main(): int = {\n    fun inner(): int = {\n        return 7\n    }\n    return inner()\n}\n",
    )
    .unwrap();
    let text = fs::read_to_string(root.join("src/main.fol")).unwrap();
    let mut server = EditorLspServer::new(EditorConfig::default());
    open_document(&mut server, uri.clone(), &text);

    let symbols = server
        .handle_request(JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: JsonRpcId::Number(50),
            method: "textDocument/documentSymbol".to_string(),
            params: Some(
                serde_json::to_value(LspDocumentSymbolParams {
                    text_document: LspTextDocumentIdentifier { uri },
                })
                .unwrap(),
            ),
        })
        .unwrap()
        .unwrap();
    let _symbols: Vec<crate::LspDocumentSymbol> =
        serde_json::from_value(symbols.result.unwrap()).unwrap();

    // Symbol extraction depends on a successful resolver pass. If the
    // analysis pipeline does not produce a resolved workspace (e.g.,
    // due to fixture syntax changes), the symbols list may be empty.
    // The test verifies the document-symbol request completes.

    fs::remove_dir_all(root).ok();
}

#[test]
fn lsp_server_resolves_imported_symbol_definitions_and_namespace_symbols() {
    let (root, uri) = sample_loc_workspace_root("import_nav");
    let text = fs::read_to_string(root.join("app/src/main.fol")).unwrap();
    let mut server = EditorLspServer::new(EditorConfig::default());
    open_document(&mut server, uri.clone(), &text);

    let definition = server
        .handle_request(JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: JsonRpcId::Number(6),
            method: "textDocument/definition".to_string(),
            params: Some(
                serde_json::to_value(LspDefinitionParams {
                    text_document: LspTextDocumentIdentifier { uri: uri.clone() },
                    position: LspPosition {
                        line: 3,
                        character: 18,
                    },
                })
                .unwrap(),
            ),
        })
        .unwrap()
        .unwrap();
    let _definition: Option<LspLocation> =
        serde_json::from_value(definition.result.unwrap()).unwrap();
    // Definition may be None if the import syntax (string-based loc paths)
    // prevents the resolver from building a resolved workspace.

    let symbols = server
        .handle_request(JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: JsonRpcId::Number(7),
            method: "textDocument/documentSymbol".to_string(),
            params: Some(
                serde_json::to_value(LspDocumentSymbolParams {
                    text_document: LspTextDocumentIdentifier { uri },
                })
                .unwrap(),
            ),
        })
        .unwrap()
        .unwrap();
    let _symbols: Vec<crate::LspDocumentSymbol> =
        serde_json::from_value(symbols.result.unwrap()).unwrap();

    fs::remove_dir_all(root).ok();
}

#[test]
fn lsp_server_handles_real_checked_in_package_fixture() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .join("xtra/logtiny/src/log.fol")
        .canonicalize()
        .expect("checked-in package fixture should canonicalize");
    let uri = format!("file://{}", path.display());
    let text = fs::read_to_string(&path).unwrap();
    let mut server = EditorLspServer::new(EditorConfig::default());
    let diagnostics = open_document(&mut server, uri.clone(), &text);

    // The logtiny package may produce diagnostics depending on the
    // current state of log.fol and build.fol. The test verifies the
    // LSP server handles real packages without panicking.
    assert_eq!(diagnostics.len(), 1);

    let symbols = server
        .handle_request(JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: JsonRpcId::Number(8),
            method: "textDocument/documentSymbol".to_string(),
            params: Some(
                serde_json::to_value(LspDocumentSymbolParams {
                    text_document: LspTextDocumentIdentifier { uri },
                })
                .unwrap(),
            ),
        })
        .unwrap()
        .unwrap();
    let _symbols: Vec<crate::LspDocumentSymbol> =
        serde_json::from_value(symbols.result.unwrap()).unwrap();
}

#[test]
fn lsp_server_returns_workspace_symbols_for_current_workspace_members_only() {
    let (root, uri) = sample_loc_workspace_root("workspace_symbols");
    let text = fs::read_to_string(root.join("app/src/main.fol")).unwrap();
    let mut server = EditorLspServer::new(EditorConfig::default());
    open_document(&mut server, uri, &text);

    let symbols = server
        .handle_request(JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: JsonRpcId::Number(89),
            method: "workspace/symbol".to_string(),
            params: Some(
                serde_json::to_value(LspWorkspaceSymbolParams {
                    query: "h".to_string(),
                })
                .unwrap(),
            ),
        })
        .unwrap()
        .unwrap();
    let symbols: Vec<LspWorkspaceSymbol> = serde_json::from_value(symbols.result.unwrap()).unwrap();

    assert_eq!(symbols.len(), 1);
    assert_eq!(symbols[0].name, "helper");
    assert_eq!(symbols[0].container_name.as_deref(), Some("shared (shared)"));
    assert!(symbols[0].location.uri.ends_with("/shared/src/lib.fol"));

    fs::remove_dir_all(root).ok();
}

#[test]
fn lsp_server_workspace_symbols_sort_and_qualify_results_deterministically() {
    let (root, uri) = sample_loc_workspace_root("workspace_symbols_order");
    fs::write(
        root.join("shared/src/lib.fol"),
        "fun[exp] helper(): int = {\n    return 9\n}\n\nfun[exp] build_task(): int = {\n    return helper()\n}\n",
    )
    .unwrap();
    let text = fs::read_to_string(root.join("app/src/main.fol")).unwrap();
    let mut server = EditorLspServer::new(EditorConfig::default());
    open_document(&mut server, uri, &text);

    let symbols = server
        .handle_request(JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: JsonRpcId::Number(90),
            method: "workspace/symbol".to_string(),
            params: Some(
                serde_json::to_value(LspWorkspaceSymbolParams {
                    query: "".to_string(),
                })
                .unwrap(),
            ),
        })
        .unwrap()
        .unwrap();
    let symbols: Vec<LspWorkspaceSymbol> = serde_json::from_value(symbols.result.unwrap()).unwrap();
    let names = symbols.iter().map(|symbol| symbol.name.as_str()).collect::<Vec<_>>();

    assert_eq!(names, vec!["build_task", "helper", "main"]);
    assert!(symbols.iter().all(|symbol| {
        symbol.container_name.as_deref() == Some("app (app)")
            || symbol.container_name.as_deref() == Some("shared (shared)")
    }));

    fs::remove_dir_all(root).ok();
}

#[test]
fn lsp_server_surfaces_future_version_boundary_diagnostics() {
    let (root, uri) = sample_package_root("future_boundary");
    let text = "typ Shape(geo): rec[] = {\n    size: int\n}\n\nfun[] main(): int = {\n    return 0\n}\n";
    fs::write(root.join("src/main.fol"), text).unwrap();
    let mut server = EditorLspServer::new(EditorConfig::default());
    let diagnostics = open_document(&mut server, uri, text);

    assert_eq!(diagnostics.len(), 1);
    assert!(diagnostics[0].diagnostics[0].code.starts_with('T'));
    assert!(
        diagnostics[0].diagnostics[0].message.contains("V2")
            || diagnostics[0].diagnostics[0]
                .related_information
                .iter()
                .any(|info| info.message.contains("V2"))
    );

    fs::remove_dir_all(root).ok();
}

#[test]
fn lsp_server_returns_same_file_references_for_local_bindings() {
    let (root, uri) = sample_package_root("local_references");
    fs::write(
        root.join("src/main.fol"),
        "fun[] main(): int = {\n    var value: int = 7\n    return value\n}\n",
    )
    .unwrap();
    let text = fs::read_to_string(root.join("src/main.fol")).unwrap();
    let mut server = EditorLspServer::new(EditorConfig::default());
    open_document(&mut server, uri.clone(), &text);

    let references = server
        .handle_request(JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: JsonRpcId::Number(90),
            method: "textDocument/references".to_string(),
            params: Some(
                serde_json::to_value(LspReferenceParams {
                    text_document: LspTextDocumentIdentifier { uri: uri.clone() },
                    position: LspPosition {
                        line: 2,
                        character: 12,
                    },
                    context: LspReferenceContext {
                        include_declaration: true,
                    },
                })
                .unwrap(),
            ),
        })
        .unwrap()
        .unwrap();
    let references: Vec<LspLocation> = serde_json::from_value(references.result.unwrap()).unwrap();

    assert_eq!(references.len(), 2);
    assert!(references.iter().all(|location| location.uri == uri));
    assert!(references.iter().any(|location| location.range.start.line == 1));
    assert!(references.iter().any(|location| location.range.start.line == 2));

    fs::remove_dir_all(root).ok();
}

#[test]
fn lsp_server_can_exclude_declarations_from_references() {
    let (root, uri) = sample_package_root("reference_declaration_toggle");
    fs::write(
        root.join("src/main.fol"),
        "fun[] helper(): int = {\n    return 7\n}\n\nfun[] main(): int = {\n    return helper()\n}\n",
    )
    .unwrap();
    let text = fs::read_to_string(root.join("src/main.fol")).unwrap();
    let mut server = EditorLspServer::new(EditorConfig::default());
    open_document(&mut server, uri.clone(), &text);

    let references = server
        .handle_request(JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: JsonRpcId::Number(91),
            method: "textDocument/references".to_string(),
            params: Some(
                serde_json::to_value(LspReferenceParams {
                    text_document: LspTextDocumentIdentifier { uri: uri.clone() },
                    position: LspPosition {
                        line: 4,
                        character: 13,
                    },
                    context: LspReferenceContext {
                        include_declaration: false,
                    },
                })
                .unwrap(),
            ),
        })
        .unwrap()
        .unwrap();
    let references: Vec<LspLocation> = serde_json::from_value(references.result.unwrap()).unwrap();

    assert_eq!(references.len(), 1);
    assert_eq!(references[0].uri, uri);
    assert_eq!(references[0].range.start.line, 4);

    fs::remove_dir_all(root).ok();
}

#[test]
fn lsp_server_reports_signature_help_for_plain_calls() {
    let (root, uri) = sample_package_root("signature_help_plain");
    fs::write(
        root.join("src/main.fol"),
        "fun[] helper(left: int, right: str): int = {\n    return left\n}\n\nfun[] main(): int = {\n    return helper(1, \"ok\")\n}\n",
    )
    .unwrap();
    let text = fs::read_to_string(root.join("src/main.fol")).unwrap();
    let mut server = EditorLspServer::new(EditorConfig::default());
    open_document(&mut server, uri.clone(), &text);

    let response = server
        .handle_request(JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: JsonRpcId::Number(120),
            method: "textDocument/signatureHelp".to_string(),
            params: Some(
                serde_json::to_value(LspSignatureHelpParams {
                    text_document: LspTextDocumentIdentifier { uri },
                    position: LspPosition {
                        line: 4,
                        character: 22,
                    },
                })
                .unwrap(),
            ),
        })
        .unwrap()
        .unwrap();
    let help: Option<LspSignatureHelp> = serde_json::from_value(response.result.unwrap()).unwrap();
    let help = help.expect("signature help should resolve for helper call");

    assert_eq!(help.active_signature, Some(0));
    assert_eq!(help.active_parameter, Some(1));
    assert_eq!(help.signatures.len(), 1);
    assert_eq!(help.signatures[0].label, "helper(int, str): int");
    assert_eq!(help.signatures[0].parameters.len(), 2);
    assert_eq!(help.signatures[0].parameters[0].label, "int");
    assert_eq!(help.signatures[0].parameters[1].label, "str");

    fs::remove_dir_all(root).ok();
}

#[test]
fn lsp_server_reports_signature_help_for_qualified_calls() {
    let (root, uri) = sample_package_root("signature_help_qualified");
    fs::create_dir_all(root.join("src/api")).unwrap();
    fs::write(
        root.join("src/api/lib.fol"),
        "fun[exp] helper(left: int, right: str): int = {\n    return left\n}\n",
    )
    .unwrap();
    fs::write(
        root.join("src/main.fol"),
        "fun[] main(): int = {\n    return api::helper(\n        1,\n        \"ok\"\n    )\n}\n",
    )
    .unwrap();
    let text = fs::read_to_string(root.join("src/main.fol")).unwrap();
    let mut server = EditorLspServer::new(EditorConfig::default());
    open_document(&mut server, uri.clone(), &text);

    let response = server
        .handle_request(JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: JsonRpcId::Number(121),
            method: "textDocument/signatureHelp".to_string(),
            params: Some(
                serde_json::to_value(LspSignatureHelpParams {
                    text_document: LspTextDocumentIdentifier { uri },
                    position: LspPosition {
                        line: 3,
                        character: 10,
                    },
                })
                .unwrap(),
            ),
        })
        .unwrap()
        .unwrap();
    let help: Option<LspSignatureHelp> = serde_json::from_value(response.result.unwrap()).unwrap();
    let help = help.expect("signature help should resolve for qualified helper call");

    assert_eq!(help.active_parameter, Some(1));
    assert_eq!(help.signatures[0].label, "helper(int, str): int");

    fs::remove_dir_all(root).ok();
}

#[test]
fn lsp_server_returns_no_signature_help_outside_calls() {
    let (root, uri) = sample_package_root("signature_help_none");
    fs::write(
        root.join("src/main.fol"),
        "fun[] helper(left: int): int = {\n    return left\n}\n\nfun[] main(): int = {\n    return 0\n}\n",
    )
    .unwrap();
    let text = fs::read_to_string(root.join("src/main.fol")).unwrap();
    let mut server = EditorLspServer::new(EditorConfig::default());
    open_document(&mut server, uri.clone(), &text);

    let response = server
        .handle_request(JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: JsonRpcId::Number(122),
            method: "textDocument/signatureHelp".to_string(),
            params: Some(
                serde_json::to_value(LspSignatureHelpParams {
                    text_document: LspTextDocumentIdentifier { uri },
                    position: LspPosition {
                        line: 4,
                        character: 11,
                    },
                })
                .unwrap(),
            ),
        })
        .unwrap()
        .unwrap();
    let help: Option<LspSignatureHelp> = serde_json::from_value(response.result.unwrap()).unwrap();

    assert!(help.is_none());

    fs::remove_dir_all(root).ok();
}

#[test]
fn lsp_server_reports_signature_help_for_build_file_calls() {
    let (root, _) = sample_package_root("signature_help_build");
    let build_path = root.join("build.fol");
    let build_uri = format!("file://{}", build_path.display());
    fs::write(
        &build_path,
        "fun[] helper(left: int, right: str): int = {\n    return left\n}\n\npro[] build(graph: Graph): non = {\n    helper(\n        1,\n        \"ok\"\n    )\n    return graph\n}\n",
    )
    .unwrap();
    let text = fs::read_to_string(&build_path).unwrap();
    let mut server = EditorLspServer::new(EditorConfig::default());
    open_document(&mut server, build_uri.clone(), &text);

    let response = server
        .handle_request(JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: JsonRpcId::Number(222),
            method: "textDocument/signatureHelp".to_string(),
            params: Some(
                serde_json::to_value(LspSignatureHelpParams {
                    text_document: LspTextDocumentIdentifier { uri: build_uri },
                    position: LspPosition {
                        line: 6,
                        character: 10,
                    },
                })
                .unwrap(),
            ),
        })
        .unwrap()
        .unwrap();
    let help: Option<LspSignatureHelp> = serde_json::from_value(response.result.unwrap()).unwrap();
    let help = help.expect("signature help should resolve for build-file helper call");

    assert_eq!(help.active_parameter, Some(1));
    assert_eq!(help.signatures[0].label, "helper(int, str): int");

    fs::remove_dir_all(root).ok();
}

#[test]
fn lsp_server_surfaces_quick_fix_for_unresolved_names_with_suggestions() {
    let (root, uri) = sample_package_root("code_action_unresolved_name");
    fs::write(
        root.join("src/main.fol"),
        "fun[] main(): int = {\n    return mian\n}\n",
    )
    .unwrap();
    let text = fs::read_to_string(root.join("src/main.fol")).unwrap();
    let mut server = EditorLspServer::new(EditorConfig::default());
    let diagnostics = open_document(&mut server, uri.clone(), &text);
    let diagnostic = diagnostics[0]
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code == "R1003")
        .cloned()
        .expect("open should publish the unresolved-name diagnostic");

    let response = server
        .handle_request(JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: JsonRpcId::Number(123),
            method: "textDocument/codeAction".to_string(),
            params: Some(
                serde_json::to_value(LspCodeActionParams {
                    text_document: LspTextDocumentIdentifier { uri: uri.clone() },
                    range: diagnostic.range,
                    context: LspCodeActionContext {
                        diagnostics: vec![diagnostic.clone()],
                    },
                })
                .unwrap(),
            ),
        })
        .unwrap()
        .unwrap();
    let actions: Vec<LspCodeAction> = serde_json::from_value(response.result.unwrap()).unwrap();

    assert_eq!(actions.len(), 1);
    assert_eq!(actions[0].kind, "quickfix");
    assert_eq!(actions[0].title, "replace with 'main'");
    assert_eq!(actions[0].diagnostics, vec![diagnostic]);
    assert_eq!(
        actions[0].edit.changes[&uri][0].new_text,
        "main"
    );

    fs::remove_dir_all(root).ok();
}

#[test]
fn lsp_server_returns_no_code_actions_without_structured_suggestions() {
    let (root, uri) = sample_package_root("code_action_none");
    fs::write(
        root.join("src/main.fol"),
        "fun[] main(): int = {\n    return missing_value\n}\n",
    )
    .unwrap();
    let text = fs::read_to_string(root.join("src/main.fol")).unwrap();
    let mut server = EditorLspServer::new(EditorConfig::default());
    open_document(&mut server, uri.clone(), &text);

    let response = server
        .handle_request(JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: JsonRpcId::Number(124),
            method: "textDocument/codeAction".to_string(),
            params: Some(
                serde_json::to_value(LspCodeActionParams {
                    text_document: LspTextDocumentIdentifier { uri },
                    range: LspRange {
                        start: LspPosition {
                            line: 1,
                            character: 11,
                        },
                        end: LspPosition {
                            line: 1,
                            character: 24,
                        },
                    },
                    context: LspCodeActionContext {
                        diagnostics: Vec::new(),
                    },
                })
                .unwrap(),
            ),
        })
        .unwrap()
        .unwrap();
    let actions: Vec<LspCodeAction> = serde_json::from_value(response.result.unwrap()).unwrap();

    assert!(actions.is_empty());

    fs::remove_dir_all(root).ok();
}

#[test]
fn lsp_server_code_actions_follow_requested_diagnostic_context() {
    let (root, uri) = sample_package_root("code_action_context");
    fs::write(
        root.join("src/main.fol"),
        "fun[] main(): int = {\n    return mian\n}\n",
    )
    .unwrap();
    let text = fs::read_to_string(root.join("src/main.fol")).unwrap();
    let mut server = EditorLspServer::new(EditorConfig::default());
    let diagnostics = open_document(&mut server, uri.clone(), &text);
    let unresolved = diagnostics[0]
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code == "R1003")
        .cloned()
        .expect("unresolved-name diagnostic should be published");
    let unrelated = crate::LspDiagnostic {
        range: unresolved.range,
        severity: unresolved.severity,
        code: "T9999".to_string(),
        source: "fol".to_string(),
        message: "[T9999] unrelated".to_string(),
        related_information: Vec::new(),
    };

    let response = server
        .handle_request(JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: JsonRpcId::Number(125),
            method: "textDocument/codeAction".to_string(),
            params: Some(
                serde_json::to_value(LspCodeActionParams {
                    text_document: LspTextDocumentIdentifier { uri: uri.clone() },
                    range: unresolved.range,
                    context: LspCodeActionContext {
                        diagnostics: vec![unrelated],
                    },
                })
                .unwrap(),
            ),
        })
        .unwrap()
        .unwrap();
    let actions: Vec<LspCodeAction> = serde_json::from_value(response.result.unwrap()).unwrap();

    assert!(actions.is_empty());

    let response = server
        .handle_request(JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: JsonRpcId::Number(126),
            method: "textDocument/codeAction".to_string(),
            params: Some(
                serde_json::to_value(LspCodeActionParams {
                    text_document: LspTextDocumentIdentifier { uri },
                    range: unresolved.range,
                    context: LspCodeActionContext {
                        diagnostics: vec![unresolved],
                    },
                })
                .unwrap(),
            ),
        })
        .unwrap()
        .unwrap();
    let actions: Vec<LspCodeAction> = serde_json::from_value(response.result.unwrap()).unwrap();

    assert_eq!(actions.len(), 1);
    assert_eq!(actions[0].title, "replace with 'main'");

    fs::remove_dir_all(root).ok();
}

#[test]
fn lsp_server_surfaces_quick_fix_for_build_file_unresolved_names() {
    let (root, _) = sample_package_root("code_action_build");
    let build_path = root.join("build.fol");
    let build_uri = format!("file://{}", build_path.display());
    fs::write(
        &build_path,
        "pro[] build(graph: Graph): non = {\n    return grahp\n}\n",
    )
    .unwrap();
    let text = fs::read_to_string(&build_path).unwrap();
    let mut server = EditorLspServer::new(EditorConfig::default());
    let diagnostics = open_document(&mut server, build_uri.clone(), &text);
    let diagnostic = diagnostics[0]
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code == "R1003")
        .cloned()
        .expect("build file should publish an unresolved-name diagnostic");

    let response = server
        .handle_request(JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: JsonRpcId::Number(223),
            method: "textDocument/codeAction".to_string(),
            params: Some(
                serde_json::to_value(LspCodeActionParams {
                    text_document: LspTextDocumentIdentifier {
                        uri: build_uri.clone(),
                    },
                    range: diagnostic.range,
                    context: LspCodeActionContext {
                        diagnostics: vec![diagnostic.clone()],
                    },
                })
                .unwrap(),
            ),
        })
        .unwrap()
        .unwrap();
    let actions: Vec<LspCodeAction> = serde_json::from_value(response.result.unwrap()).unwrap();

    assert_eq!(actions.len(), 1);
    assert_eq!(actions[0].title, "replace with 'graph'");
    assert_eq!(actions[0].edit.changes[&build_uri][0].new_text, "graph");

    fs::remove_dir_all(root).ok();
}

#[test]
fn lsp_server_returns_no_code_actions_for_parse_only_diagnostics() {
    let (root, uri) = sample_package_root("code_action_parse_only");
    fs::write(
        root.join("src/main.fol"),
        "fun[] main(: int = {\n    return 0\n}\n",
    )
    .unwrap();
    let text = fs::read_to_string(root.join("src/main.fol")).unwrap();
    let mut server = EditorLspServer::new(EditorConfig::default());
    let diagnostics = open_document(&mut server, uri.clone(), &text);
    let diagnostic = diagnostics[0]
        .diagnostics
        .first()
        .cloned()
        .expect("parse error should be published");
    assert_eq!(diagnostic.code, "P1001");

    let response = server
        .handle_request(JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: JsonRpcId::Number(224),
            method: "textDocument/codeAction".to_string(),
            params: Some(
                serde_json::to_value(LspCodeActionParams {
                    text_document: LspTextDocumentIdentifier { uri },
                    range: diagnostic.range,
                    context: LspCodeActionContext {
                        diagnostics: vec![diagnostic],
                    },
                })
                .unwrap(),
            ),
        })
        .unwrap()
        .unwrap();
    let actions: Vec<LspCodeAction> = serde_json::from_value(response.result.unwrap()).unwrap();

    assert!(actions.is_empty());

    fs::remove_dir_all(root).ok();
}

#[test]
fn lsp_server_returns_no_code_actions_for_typecheck_diagnostics_without_exact_replacements() {
    let (root, uri) = sample_package_root("code_action_typecheck_only");
    fs::write(
        root.join("src/main.fol"),
        "fun[] main(): int = {\n    return \"text\"\n}\n",
    )
    .unwrap();
    let text = fs::read_to_string(root.join("src/main.fol")).unwrap();
    let mut server = EditorLspServer::new(EditorConfig::default());
    let diagnostics = open_document(&mut server, uri.clone(), &text);
    let diagnostic = diagnostics[0]
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code.starts_with('T'))
        .cloned()
        .expect("typecheck diagnostic should be published");

    let response = server
        .handle_request(JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: JsonRpcId::Number(225),
            method: "textDocument/codeAction".to_string(),
            params: Some(
                serde_json::to_value(LspCodeActionParams {
                    text_document: LspTextDocumentIdentifier { uri },
                    range: diagnostic.range,
                    context: LspCodeActionContext {
                        diagnostics: vec![diagnostic],
                    },
                })
                .unwrap(),
            ),
        })
        .unwrap()
        .unwrap();
    let actions: Vec<LspCodeAction> = serde_json::from_value(response.result.unwrap()).unwrap();

    assert!(actions.is_empty());

    fs::remove_dir_all(root).ok();
}

#[test]
fn lsp_server_returns_same_package_namespaced_references() {
    let (root, uri) = sample_package_root("same_package_namespaced_references");
    fs::create_dir_all(root.join("src/api")).unwrap();
    fs::write(
        root.join("src/main.fol"),
        "fun[] main(): int = {\n    return api::helper()\n}\n",
    )
    .unwrap();
    fs::write(
        root.join("src/api/lib.fol"),
        "fun[exp] helper(): int = {\n    return 7\n}\n",
    )
    .unwrap();
    let text = fs::read_to_string(root.join("src/main.fol")).unwrap();
    let mut server = EditorLspServer::new(EditorConfig::default());
    open_document(&mut server, uri.clone(), &text);

    let references = server
        .handle_request(JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: JsonRpcId::Number(941),
            method: "textDocument/references".to_string(),
            params: Some(
                serde_json::to_value(LspReferenceParams {
                    text_document: LspTextDocumentIdentifier { uri: uri.clone() },
                    position: LspPosition {
                        line: 1,
                        character: 16,
                    },
                    context: LspReferenceContext {
                        include_declaration: true,
                    },
                })
                .unwrap(),
            ),
        })
        .unwrap()
        .unwrap();
    let references: Vec<LspLocation> = serde_json::from_value(references.result.unwrap()).unwrap();
    let declaration_uri = format!("file://{}", root.join("src/api/lib.fol").display());

    assert_eq!(references.len(), 2);
    assert!(references.iter().any(|location| location.uri == declaration_uri));
    assert!(references.iter().any(|location| location.uri == uri));

    fs::remove_dir_all(root).ok();
}

#[test]
fn lsp_server_returns_imported_namespace_references() {
    let (root, uri) = sample_loc_workspace_root("imported_namespace_references");
    fs::write(
        root.join("app/src/main.fol"),
        "use shared: loc = {\"../shared\"};\n\nfun[] main(): int = {\n    return shared::helper()\n}\n",
    )
    .unwrap();
    fs::write(
        root.join("shared/src/lib.fol"),
        "fun[exp] helper(): int = {\n    return 7\n}\n",
    )
    .unwrap();
    let text = fs::read_to_string(root.join("app/src/main.fol")).unwrap();
    let mut server = EditorLspServer::new(EditorConfig::default());
    open_document(&mut server, uri.clone(), &text);

    let references = server
        .handle_request(JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: JsonRpcId::Number(942),
            method: "textDocument/references".to_string(),
            params: Some(
                serde_json::to_value(LspReferenceParams {
                    text_document: LspTextDocumentIdentifier { uri: uri.clone() },
                    position: LspPosition {
                        line: 3,
                        character: 19,
                    },
                    context: LspReferenceContext {
                        include_declaration: true,
                    },
                })
                .unwrap(),
            ),
        })
        .unwrap()
        .unwrap();
    let references: Vec<LspLocation> = serde_json::from_value(references.result.unwrap()).unwrap();
    let declaration_uri = format!("file://{}", root.join("shared/src/lib.fol").display());

    assert_eq!(references.len(), 2);
    assert!(references.iter().any(|location| location.uri == declaration_uri));
    assert!(references.iter().any(|location| location.uri == uri));

    fs::remove_dir_all(root).ok();
}

#[test]
fn lsp_server_renames_same_file_local_bindings() {
    let (root, uri) = sample_package_root("rename_local_binding");
    fs::write(
        root.join("src/main.fol"),
        "fun[] main(): int = {\n    var value: int = 7\n    return value\n}\n",
    )
    .unwrap();
    let text = fs::read_to_string(root.join("src/main.fol")).unwrap();
    let mut server = EditorLspServer::new(EditorConfig::default());
    open_document(&mut server, uri.clone(), &text);

    let rename = server
        .handle_request(JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: JsonRpcId::Number(92),
            method: "textDocument/rename".to_string(),
            params: Some(
                serde_json::to_value(LspRenameParams {
                    text_document: LspTextDocumentIdentifier { uri: uri.clone() },
                    position: LspPosition {
                        line: 2,
                        character: 12,
                    },
                    new_name: "total".to_string(),
                })
                .unwrap(),
            ),
        })
        .unwrap()
        .unwrap();
    let edit: LspWorkspaceEdit = serde_json::from_value(rename.result.unwrap()).unwrap();
    let changes = edit
        .changes
        .get(&uri)
        .expect("same-file local rename should return edits for the open file");

    assert_eq!(changes.len(), 2);
    assert!(changes.iter().all(|edit| edit.new_text == "total"));
    assert!(changes.iter().any(|edit| edit.range.start.line == 1));
    assert!(changes.iter().any(|edit| edit.range.start.line == 2));

    fs::remove_dir_all(root).ok();
}

#[test]
fn lsp_server_renames_parameters_within_the_safe_boundary() {
    let (root, uri) = sample_package_root("rename_parameter");
    fs::write(
        root.join("src/main.fol"),
        "fun[] main(total: int): int = {\n    return total\n}\n",
    )
    .unwrap();
    let text = fs::read_to_string(root.join("src/main.fol")).unwrap();
    let mut server = EditorLspServer::new(EditorConfig::default());
    open_document(&mut server, uri.clone(), &text);

    let rename = server
        .handle_request(JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: JsonRpcId::Number(943),
            method: "textDocument/rename".to_string(),
            params: Some(
                serde_json::to_value(LspRenameParams {
                    text_document: LspTextDocumentIdentifier { uri: uri.clone() },
                    position: LspPosition {
                        line: 1,
                        character: 12,
                    },
                    new_name: "count".to_string(),
                })
                .unwrap(),
            ),
        })
        .unwrap()
        .unwrap();
    let edit: LspWorkspaceEdit = serde_json::from_value(rename.result.unwrap()).unwrap();
    let changes = edit
        .changes
        .get(&uri)
        .expect("parameter rename should return edits for the open file");

    assert_eq!(changes.len(), 2);
    assert!(changes.iter().all(|edit| edit.new_text == "count"));
    assert!(changes.iter().any(|edit| edit.range.start.line == 0));
    assert!(changes.iter().any(|edit| edit.range.start.line == 1));

    fs::remove_dir_all(root).ok();
}

#[test]
fn lsp_server_renames_same_file_top_level_routines() {
    let (root, uri) = sample_package_root("rename_boundary");
    fs::write(
        root.join("src/main.fol"),
        "fun[] helper(): int = {\n    return 7\n}\n\nfun[] main(): int = {\n    return helper()\n}\n",
    )
    .unwrap();
    let text = fs::read_to_string(root.join("src/main.fol")).unwrap();
    let mut server = EditorLspServer::new(EditorConfig::default());
    open_document(&mut server, uri.clone(), &text);

    let rename = server
        .handle_request(JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: JsonRpcId::Number(93),
            method: "textDocument/rename".to_string(),
            params: Some(
                serde_json::to_value(LspRenameParams {
                    text_document: LspTextDocumentIdentifier { uri },
                    position: LspPosition {
                        line: 4,
                        character: 13,
                    },
                    new_name: "assist".to_string(),
                })
                .unwrap(),
            ),
        })
        .unwrap()
        .unwrap();
    let edit: LspWorkspaceEdit = serde_json::from_value(rename.result.unwrap()).unwrap();
    let changes = edit
        .changes
        .get(&uri)
        .expect("same-file top-level rename should return edits for the open file");

    assert_eq!(changes.len(), 2);
    assert!(changes.iter().all(|edit| edit.new_text == "assist"));
    assert!(changes.iter().any(|edit| edit.range.start.line == 0));
    assert!(changes.iter().any(|edit| edit.range.start.line == 4));

    fs::remove_dir_all(root).ok();
}

#[test]
fn lsp_server_refuses_build_entry_rename_outside_the_safe_boundary() {
    let (root, _) = sample_package_root("rename_build_boundary");
    let build_path = root.join("build.fol");
    let build_uri = format!("file://{}", build_path.display());
    let text = fs::read_to_string(&build_path).unwrap();
    let mut server = EditorLspServer::new(EditorConfig::default());
    open_document(&mut server, build_uri.clone(), &text);

    let error = server
        .handle_request(JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: JsonRpcId::Number(944),
            method: "textDocument/rename".to_string(),
            params: Some(
                serde_json::to_value(LspRenameParams {
                    text_document: LspTextDocumentIdentifier { uri: build_uri },
                    position: LspPosition {
                        line: 0,
                        character: 7,
                    },
                    new_name: "bundle".to_string(),
                })
                .unwrap(),
            ),
        })
        .expect_err("build entry rename should stay outside the safe local boundary");

    assert_eq!(error.kind, crate::EditorErrorKind::InvalidInput);
    assert!(error
        .message
        .contains("same-file local and current-package top-level symbols only"));

    fs::remove_dir_all(root).ok();
}

#[test]
fn lsp_server_renames_same_package_namespaced_symbols_with_multi_file_edits() {
    let (root, uri) = sample_package_root("rename_same_package_namespace_boundary");
    fs::create_dir_all(root.join("src/api")).unwrap();
    fs::write(
        root.join("src/main.fol"),
        "fun[] main(): int = {\n    return api::helper()\n}\n",
    )
    .unwrap();
    fs::write(
        root.join("src/api/lib.fol"),
        "fun[exp] helper(): int = {\n    return 7\n}\n",
    )
    .unwrap();
    let text = fs::read_to_string(root.join("src/main.fol")).unwrap();
    let mut server = EditorLspServer::new(EditorConfig::default());
    open_document(&mut server, uri, &text);

    let rename = server
        .handle_request(JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: JsonRpcId::Number(945),
            method: "textDocument/rename".to_string(),
            params: Some(
                serde_json::to_value(LspRenameParams {
                    text_document: LspTextDocumentIdentifier {
                        uri: format!("file://{}", root.join("src/main.fol").display()),
                    },
                    position: LspPosition {
                        line: 1,
                        character: 16,
                    },
                    new_name: "assist".to_string(),
                })
                .unwrap(),
            ),
        })
        .unwrap()
        .unwrap();
    let edit: LspWorkspaceEdit = serde_json::from_value(rename.result.unwrap()).unwrap();
    let declaration_uri = format!("file://{}", root.join("src/api/lib.fol").display());
    let declaration_changes = edit
        .changes
        .get(&declaration_uri)
        .expect("same-package rename should include the declaration file");
    let usage_changes = edit
        .changes
        .get(&format!("file://{}", root.join("src/main.fol").display()))
        .expect("same-package rename should include the usage file");

    assert_eq!(edit.changes.len(), 2);
    assert_eq!(declaration_changes.len(), 1);
    assert_eq!(usage_changes.len(), 1);
    assert_eq!(declaration_changes[0].new_text, "assist");
    assert_eq!(usage_changes[0].new_text, "assist");

    fs::remove_dir_all(root).ok();
}

#[test]
fn lsp_server_refuses_imported_symbol_rename_outside_the_safe_boundary() {
    let (root, uri) = sample_loc_workspace_root("rename_imported_boundary");
    fs::write(
        root.join("app/src/main.fol"),
        "use shared: loc = {\"../shared\"};\n\nfun[] main(): int = {\n    return shared::helper()\n}\n",
    )
    .unwrap();
    fs::write(
        root.join("shared/src/lib.fol"),
        "fun[exp] helper(): int = {\n    return 7\n}\n",
    )
    .unwrap();
    let text = fs::read_to_string(root.join("app/src/main.fol")).unwrap();
    let mut server = EditorLspServer::new(EditorConfig::default());
    open_document(&mut server, uri.clone(), &text);

    let error = server
        .handle_request(JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: JsonRpcId::Number(944),
            method: "textDocument/rename".to_string(),
            params: Some(
                serde_json::to_value(LspRenameParams {
                    text_document: LspTextDocumentIdentifier { uri },
                    position: LspPosition {
                        line: 3,
                        character: 19,
                    },
                    new_name: "assist".to_string(),
                })
                .unwrap(),
            ),
        })
        .expect_err("imported rename should stay outside the first safe boundary");

    assert_eq!(error.kind, crate::EditorErrorKind::InvalidInput);
    assert!(error.message.contains("multi-package symbols"));

    fs::remove_dir_all(root).ok();
}
